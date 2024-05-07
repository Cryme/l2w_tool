#![allow(dead_code)]

use crate::backend::{TerritoryInfo, WORLD_SQUARE_SIZE_I32};
use crate::backend::{
    coord_to_map_square_raw, Spawn, SpawnFilter, SpawnHolder, TerritoryInfoRegion, WORLD_SIZE,
    WORLD_SQUARE_SIZE,
};
use crate::spawn_parser::L2_SERVER_ROOT_SPAWN_FOLDER;
use eframe::egui;
use eframe::egui::{Color32, InnerResponse, Pos2, Rect, RichText, TextEdit, TextureId, Ui, Vec2};
use eframe::epaint::Hsva;
// use plot::{
//     log_grid_spacer, MarkerShape, Plot, PlotImage, PlotItem, PlotPoint, PlotPoints, Points, Polygon,
// };
use crate::plot::items::{
    MarkerShape, PlotImage, PlotItem, PlotPoint, PlotPoints, Points, Polygon,
};
use crate::plot::{log_grid_spacer, Plot};
use std::path::Path;
use std::rc::Rc;
use std::string::ToString;
use std::sync::RwLock;
use std::vec;
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::util::TimeHms;

const CREATE_ZONE_PATTERN: &str = "x: %X%, y: %Y%, z_min: %ZMIN%, z_max: %ZMAX%";

#[inline(always)]
fn auto_color(seed: usize, alpha: f32) -> Color32 {
    let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0;
    let h = seed as f32 * golden_ratio;

    Hsva::new(h, 0.85, 0.5, alpha).into()
}

impl TerritoryInfo {
    pub(crate) fn as_boxed_polygons(
        &self,
        color: Color32,
        is_random: bool,
        prefix: &str,
    ) -> Vec<Box<dyn PlotItem>> {
        const BANNED_REGION_COLOR: Color32 = Color32::from_rgba_premultiplied(105, 1, 51, 255);

        let mut res: Vec<Box<dyn PlotItem>> = vec![];

        res.push(Box::new(
            Polygon::new(PlotPoints::new_with_z(
                self.region
                    .iter()
                    .map(|v| [v.x as f64, -v.y as f64, v.z_min as f64, v.z_max as f64])
                    .collect(),
            ))
            .name(format!(
                "{prefix}{}{}{}",
                if is_random { "\n🎲 " } else { "" },
                if !is_random && self.name.is_some() {
                    "\n"
                } else {
                    ""
                },
                if let Some(name) = &self.name {
                    name
                } else {
                    ""
                }
            ))
            .stroke((1.0, color))
            .fill_color(Color32::TRANSPARENT),
        ));

        for v in &self.banned_regions {
            res.push(Box::new(
                Polygon::new(PlotPoints::new_with_z(
                    v.iter()
                        .map(|v| [v.x as f64, -v.y as f64, v.z_min as f64, v.z_max as f64])
                        .collect(),
                ))
                .name(if let Some(n) = &self.name {
                    format!("!{n}")
                } else {
                    "🚫".to_string()
                })
                .stroke((1.0, BANNED_REGION_COLOR))
                .fill_color(Color32::TRANSPARENT),
            ));
        }

        res
    }
}

#[derive(Default, Eq, PartialEq, EnumIter, Display, Copy, Clone)]
pub enum CreateZoneType {
    #[default]
    #[strum(to_string = "Spawn")]
    SpawnPolygon,
    #[strum(to_string = "Zone")]
    Zone,
    #[strum(to_string = "Custom")]
    Custom,
}

pub struct Frontend {
    holder: SpawnHolder,
    lines: Vec<Vec<Pos2>>,
    zoom: f32,
    x_translate: f32,
    y_translate: f32,
    next_auto_color_idx: usize,
    filtered_regions: Rc<RwLock<Vec<Box<dyn PlotItem>>>>,
    spawn_search_zone: Rc<RwLock<Option<Rect>>>,
    search_npc_id: String,
    npc_format_fn: Box<dyn Fn(u32) -> String>,
    drawing_polygon: Rc<RwLock<Vec<[f64; 2]>>>,
    is_in_create_mode: bool,
    create_zone_type: CreateZoneType,
    create_zone_pattern: String,
    z_min: i32,
    z_max: i32,
}

impl Frontend {
    fn build_editor(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        Plot::new("l2_map-shmap", self.spawn_search_zone.clone())
            .data_aspect(1.0)
            .x_grid_spacer(log_grid_spacer(WORLD_SQUARE_SIZE_I32 as i64))
            .y_grid_spacer(log_grid_spacer(WORLD_SQUARE_SIZE_I32 as i64))
            .coords_to_square_fn(Box::new(|v| {
                coord_to_map_square_raw(v.x as i32, -v.y as i32)
            }))
            .label_text_color(Some(Color32::WHITE))
            .show(
                ui,
                |_| {},
                self.filtered_regions.clone(),
                self.is_in_create_mode,
                self.drawing_polygon.clone(),
            );

        if self.is_in_create_mode {
            egui::Window::new("New Polygon")
                .id(egui::Id::new("_creating_polygon_"))
                .resizable(false)
                .collapsible(true)
                .show(ctx, |ui| {
                    ui.set_width(400.);

                    ui.horizontal(|ui| {
                        ui.label("Z Min:");
                        ui.add(egui::DragValue::new(&mut self.z_min));
                        ui.add_space(5.0);
                        ui.label("Z Max:");
                        ui.add(egui::DragValue::new(&mut self.z_max));
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new("Type"));
                        egui::ComboBox::from_id_source(ui.next_auto_id())
                            .selected_text(format!("{}", &mut self.create_zone_type))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(20.0);

                                for t in CreateZoneType::iter() {
                                    ui.selectable_value(
                                        &mut self.create_zone_type,
                                        t,
                                        format!("{t}"),
                                    );
                                }
                            });

                        if self.create_zone_type == CreateZoneType::Custom {
                            ui.add(
                                TextEdit::singleline(&mut self.create_zone_pattern)
                                    .hint_text(CREATE_ZONE_PATTERN),
                            );
                        }
                    });
                    ui.separator();

                    let mut c = "".to_string();
                    let pts = self.drawing_polygon.read().unwrap();

                    match &self.create_zone_type {
                        CreateZoneType::SpawnPolygon => {
                            c.push_str("<territory>");

                            if pts.len() > 1 {
                                for v in &pts[0..pts.len() - 1] {
                                    c.push_str(&format!(
                                        "\n\t<add x=\"{}\" y=\"{}\" zmin=\"{}\" zmax=\"{}\" />",
                                        v[0] as i32, -v[1] as i32, self.z_min, self.z_max
                                    ))
                                }
                            }
                            c.push_str("\n</territory>");
                        }
                        CreateZoneType::Zone => {
                            c.push_str("<zone name=\"NEW_ZONE\" type=\"TYPE\">\n\t<polygon>");
                            if pts.len() > 1 {
                                for v in &pts[0..pts.len() - 1] {
                                    c.push_str(&format!(
                                        "\n\t\t<coords loc=\"{} {} {} {}\"/>",
                                        v[0] as i32, -v[1] as i32, self.z_min, self.z_max
                                    ))
                                }
                            }
                            c.push_str("\n\t</polygon>\n</zone>");
                        }
                        CreateZoneType::Custom => {
                            if pts.len() > 1 {
                                for v in &pts[0..pts.len() - 1] {
                                    c.push_str("\n\t");
                                    c.push_str(
                                        &self
                                            .create_zone_pattern
                                            .replace("%X%", &(v[0] as i32).to_string())
                                            .replace("%Y%", &(v[1] as i32).to_string())
                                            .replace("%ZMIN%", &self.z_min.to_string())
                                            .replace("%ZMAX%", &self.z_max.to_string()),
                                    );
                                }
                            }
                        }
                    }

                    ui.label(RichText::new(&c).color(Color32::WHITE));
                });
        }
    }

    fn build_top_menu(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label("NpcId: ");

            ui.scope(|ui| {
                ui.set_width(50.);
                if ui.text_edit_singleline(&mut self.search_npc_id).changed() {
                    self.on_npc_search_update();
                }
            });
            ui.add_space(10.);
            ui.label("Create Mode");
            ui.checkbox(&mut self.is_in_create_mode, "");
        });
    }

    fn check_for_zone_update(&mut self) {
        let mut r_zone = None;
        {
            let mut zone = self.spawn_search_zone.write().unwrap();
            std::mem::swap(&mut r_zone, &mut *zone);
        }

        if let Some(z) = r_zone {
            self.search_npc_id = "".to_string();

            if z.area() == 0. {
                self.filter_spawns(SpawnFilter::FullSquare(coord_to_map_square_raw(
                    z.min.x as i32,
                    z.min.y as i32,
                )))
            } else {
                self.filter_spawns(SpawnFilter::InZone(z));
            }
        }
    }

    fn on_npc_search_update(&mut self) {
        self.search_npc_id = self
            .search_npc_id
            .chars()
            .filter(|char| char.is_ascii_digit())
            .collect();

        self.filter_spawns(SpawnFilter::ByNpcId(self.search_npc_id.parse().unwrap()))
    }

    fn filter_spawns(&mut self, filter: SpawnFilter) {
        let spawn_infos = match filter {
            SpawnFilter::FullSquare(v) => self.holder.get_square_spawns(v),
            SpawnFilter::InZone(z) => self.holder.get_zone_spawns(&z),
            SpawnFilter::ByNpcId(id) => self.holder.get_npc_spawns(id),
        };

        let mut regions = self.filtered_regions.write().unwrap();

        *regions = vec![regions.remove(0)];

        for (i, spawn_info) in spawn_infos.iter().enumerate() {
            let npcs_info = if spawn_info.npc.len() == 1 {
                format!("Npc: {}", (self.npc_format_fn)(spawn_info.npc[0].id))
            } else {
                format!(
                    "NpcIds: {:?}",
                    spawn_info
                        .npc
                        .iter()
                        .map(|v| (self.npc_format_fn)(v.id))
                        .collect::<Vec<String>>()
                )
            };

            let spawn_meta = format!(
                "{}\nPeriod of Day: {}\nGroup: {}\nCount: {}\nRespawn: {} random: {}\nFile: {}",
                npcs_info,
                spawn_info.period_of_day.name(),
                spawn_info.group,
                spawn_info.count,
                TimeHms::new(spawn_info.respawn_sec as u64),
                TimeHms::new(spawn_info.respawn_random_sec as u64),
                if let Some(v) = &spawn_info.file_name {
                    v.split(L2_SERVER_ROOT_SPAWN_FOLDER).nth(1).unwrap()
                } else {
                    ""
                },
            );

            let mut pts = vec![];

            let color = auto_color(i, 1.0);

            for spawn in &spawn_info.spawns {
                match spawn {
                    Spawn::Point(v) => pts.push([v.x as f64, -v.y as f64, v.z as f64, v.z as f64]),
                    Spawn::Territory(v) => match &v.territory {
                        TerritoryInfoRegion::Named(name) => {
                            if let Some(info) = self.holder.territories.get(name) {
                                regions.extend(info.as_boxed_polygons(color, false, &spawn_meta));
                            }
                        }
                        TerritoryInfoRegion::Inlined(info) => {
                            regions.extend(info.as_boxed_polygons(color, false, &spawn_meta));
                        }
                    },
                    Spawn::RandomTerritory(v) => {
                        for territory_name in v {
                            if let Some(info) = self.holder.territories.get(territory_name) {
                                regions.extend(info.as_boxed_polygons(color, true, &spawn_meta));
                            }
                        }
                    }
                }
            }

            regions.push(Box::new(
                Points::new(pts)
                    .name(&spawn_meta)
                    .filled(true)
                    .radius(3.0)
                    .shape(MarkerShape::Circle)
                    .color(color),
            ));
        }
    }

    pub fn init<T: AsRef<Path>>(
        spawn_root_folder_path: T,
        map_texture_id: impl Into<TextureId>,
        npc_format_fn: Box<dyn Fn(u32) -> String>,
    ) -> anyhow::Result<Self> {
        let holder = SpawnHolder::try_init(spawn_root_folder_path)?;

        Ok(Self {
            holder,
            lines: Vec::new(),
            zoom: 0.5,
            x_translate: 0.0,
            y_translate: 0.0,
            next_auto_color_idx: 0,
            filtered_regions: Rc::new(RwLock::new(vec![Box::new(PlotImage::new(
                map_texture_id,
                PlotPoint::new(-WORLD_SQUARE_SIZE * 1.5, 0),
                Vec2::new(WORLD_SIZE.x, WORLD_SIZE.y),
            ))])),
            spawn_search_zone: Rc::new(RwLock::new(None)),
            search_npc_id: "".to_string(),
            npc_format_fn,
            drawing_polygon: Rc::new(RwLock::new(vec![[0., 0.]])),
            is_in_create_mode: false,
            create_zone_type: Default::default(),
            create_zone_pattern: CREATE_ZONE_PATTERN.to_string(),
            z_min: 0,
            z_max: 0,
        })
    }

    pub fn show(&mut self, ctx: &egui::Context, ui: &mut Ui) -> InnerResponse<()> {
        self.check_for_zone_update();

        ui.vertical(|ui| {
            self.build_top_menu(ui, ctx);
            ui.separator();
            self.build_editor(ui, ctx);
        })
    }
}

impl eframe::App for Frontend {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show(ctx, ui);
        });
    }
}
