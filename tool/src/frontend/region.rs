use crate::backend::{Backend, CurrentEntity};
use crate::entity::region::{MapInfo, Region};
use crate::frontend::util::{
    combo_box_row, format_button_text, num_row, num_row_2d, text_row, DrawAsTooltip,
};
use crate::frontend::{DrawEntity, Frontend};
use crate::holder::DataHolder;
use eframe::egui::{Button, Color32, Context, DragValue, Key, ScrollArea, Stroke, Ui};
use std::sync::RwLock;


impl DrawEntity<(), ()> for Region {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        _ctx: &Context,
        _action: &RwLock<()>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_height(400.);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    text_row(ui, &mut self.name, "Name");

                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .region_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });
                });

                num_row_2d(ui, &mut self.world_map_square, "Map Square");

                ui.horizontal(|ui| {
                    ui.label("Z min");
                    ui.add(DragValue::new(&mut self.z_range[1]));
                    ui.add_space(5.);
                    ui.label("Z max");
                    ui.add(DragValue::new(&mut self.z_range[0]));
                });

                num_row(ui, &mut self.color_code, "Color Code");
                combo_box_row(ui, &mut self.continent, "Continent");
                num_row(ui, &mut self.current_layer, "Current Layer");
                num_row(ui, &mut self.total_layers, "Total Layers");
            });

            ui.separator();

            ui.vertical(|ui| {
                if ui
                    .checkbox(&mut self.map_info.is_some(), "Map Info")
                    .changed()
                {
                    if self.map_info.is_some() {
                        self.map_info = None;
                    } else {
                        self.map_info = Some(MapInfo::default())
                    }
                }

                if let Some(v) = &mut self.map_info {
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut v.button_pos.is_some(), "Button").changed() {
                            if v.button_pos.is_some() {
                                v.button_pos = None;
                            } else {
                                v.button_pos = Some([0, 0]);
                            }
                        }

                        if let Some(vv) = &mut v.button_pos {
                            num_row_2d(ui, vv, "Position");
                        }
                    });

                    num_row_2d(ui, &mut v.pos, "Position");
                    num_row_2d(ui, &mut v.size, "Size");
                    num_row_2d(ui, &mut v.center, "Center");

                    num_row(ui, &mut v.scale, "Scale");
                    text_row(ui, &mut v.texture, "Texture");
                }
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl Frontend {
    pub fn draw_region_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id)) in self
            .backend
            .edit_params
            .get_opened_region_info()
            .iter()
            .enumerate()
        {
            let label = format!("[{}] {}", id.0, title);

            let mut button = Button::new(format_button_text(&label))
                .fill(Color32::from_rgb(99, 73, 47))
                .min_size([150., 10.].into());

            let is_current = CurrentEntity::Region(i) == self.backend.edit_params.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!("Region: {label}"))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_region(i);
            }

            if ui.button("❌").clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.close_region(i);
            }

            ui.separator();
        }
    }

    pub(crate) fn draw_region_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);

            if ui.button("    New Region    ").clicked() && backend.dialog.is_none() {
                backend.edit_params.create_new_region();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.region_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_regions();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.region_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for i in range {
                            let q = &backend.filter_params.region_catalog[i];

                            if ui
                                .button(format!(
                                    "ID: {}\nMap: {}_{}\n{}",
                                    q.id.0, q.world_map_square[0], q.world_map_square[1], q.name
                                ))
                                .clicked()
                                && backend.dialog.is_none()
                            {
                                backend.edit_params.open_region(
                                    q.id,
                                    &mut backend.holders.game_data_holder.region_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}

impl DrawAsTooltip for Region {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("[{}]\n {}", self.id.0, self.name));
    }
}
