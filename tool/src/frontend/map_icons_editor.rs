use crate::common::HuntingZoneId;
use crate::entity::hunting_zone::{HuntingZone, MapObject};
use eframe::egui;
use eframe::egui::{Context, ImageSource, SizeHint, TextureId, TextureOptions, Vec2};
use eframe::emath::Float;
use egui_plot::{log_grid_spacer, Plot, PlotImage, PlotPoint};
use std::collections::hash_map::Values;

pub const WORLD_SQUARE_SIZE: f32 = 32768.0;
const WORLD_X_SQUARE_COUNT: u8 = 17;
const WORLD_Y_SQUARE_COUNT: u8 = 16;

pub const WORLD_SIZE: Vec2 = Vec2::new(
    WORLD_SQUARE_SIZE * WORLD_X_SQUARE_COUNT as f32,
    WORLD_SQUARE_SIZE * WORLD_Y_SQUARE_COUNT as f32,
);

pub struct MapIconsEditor {
    pub(crate) showing: bool,
    world_map_texture_id: TextureId,
    not_found_texture_id: TextureId,
    map_objects: Vec<PlotMapObject>,
    texture_folder_path: String,
}

impl MapIconsEditor {
    pub fn new(world_map_texture_id: TextureId, not_found_texture_id: TextureId) -> Self {
        Self {
            showing: false,
            world_map_texture_id,
            not_found_texture_id,
            map_objects: vec![],
            texture_folder_path: "".to_string(),
        }
    }

    pub fn init(
        &mut self,
        zones: Values<HuntingZoneId, HuntingZone>,
        texture_path: &str,
        ctx: &Context,
    ) {
        self.map_objects.clear();
        self.texture_folder_path = texture_path.to_string();

        for zone in zones {
            for object in &zone.world_map_objects {
                if object.inner.icon_texture == "None" {
                    continue;
                }

                let object = object.inner.clone();

                self.map_objects.push(PlotMapObject {
                    hunting_zone_id: zone.id,
                    icon_texture_id: self.load_image(&object.icon_texture, ctx),
                    icon_texture_over_id: self.load_image(&object.icon_texture_over, ctx),
                    icon_texture_pressed_id: self.load_image(&object.icon_texture_pressed, ctx),

                    map_object: object,
                })
            }
        }
    }

    fn load_image(&self, texture: &str, ctx: &Context) -> Option<TextureId> {
        let img = ImageSource::Uri(
            format!(
                "file://{}/{}.png",
                self.texture_folder_path,
                texture.replace('.', "/")
            )
            .into(),
        );

        img.load(ctx, TextureOptions::default(), SizeHint::Scale(1.0.ord()))
            .ok()
            .and_then(|v| v.texture_id())
    }

    pub fn draw(&mut self, ctx: &Context) {
        if self.showing {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("_icons_edtior_"),
                egui::ViewportBuilder::default()
                    .with_title("Map Icons Editor")
                    .with_inner_size([1000.0, 800.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default().show(ctx, |ui| {
                        Plot::new("_icons_editor_")
                            .data_aspect(1.0)
                            .x_grid_spacer(log_grid_spacer(WORLD_SQUARE_SIZE as i64))
                            .y_grid_spacer(log_grid_spacer(WORLD_SQUARE_SIZE as i64))
                            .show(ui, |plot_ui| {
                                plot_ui.add(PlotImage::new(
                                    self.world_map_texture_id,
                                    PlotPoint::new(-WORLD_SQUARE_SIZE * 1.5, 0),
                                    Vec2::new(WORLD_SIZE.x, WORLD_SIZE.y),
                                ));

                                for v in &self.map_objects {
                                    plot_ui.add(
                                        PlotImage::new(
                                            self.load_image(&v.map_object.icon_texture, ctx)
                                                .unwrap_or(self.not_found_texture_id),
                                            PlotPoint::new(
                                                v.map_object.world_pos[0],
                                                -v.map_object.world_pos[1],
                                            ),
                                            Vec2::new(
                                                (v.map_object.size[0] as f32) * 400.,
                                                (v.map_object.size[1] as f32) * 400.,
                                            ),
                                        )
                                        .highlight(true),
                                    );
                                }
                            });
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        // Tell parent viewport that we should not show next frame:
                        self.showing = false;
                    }
                },
            );
        }
    }
}

#[allow(unused)]
pub struct PlotMapObject {
    map_object: MapObject,
    hunting_zone_id: HuntingZoneId,
    icon_texture_id: Option<TextureId>,
    icon_texture_over_id: Option<TextureId>,
    icon_texture_pressed_id: Option<TextureId>,
}
