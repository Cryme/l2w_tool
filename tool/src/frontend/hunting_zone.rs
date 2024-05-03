use crate::backend::hunting_zone::HuntingZoneAction;
use crate::backend::{Backend, CurrentEntity};
use crate::entity::hunting_zone::HuntingZone;
use crate::frontend::util::{
    combo_box_row, format_button_text, num_row, num_row_optional, text_row, text_row_multiline,
    Draw, DrawAsTooltip, DrawUtils,
};
use crate::frontend::{DrawEntity, Frontend};
use crate::holder::DataHolder;
use eframe::egui::{Button, Color32, Context, DragValue, Key, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawEntity<HuntingZoneAction, ()> for HuntingZone {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        _ctx: &Context,
        action: &RwLock<HuntingZoneAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_height(400.);

            ui.vertical(|ui| {
                ui.set_width(300.);

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.name, "Name");
                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .hunting_zone_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });
                });
                text_row_multiline(ui, &mut self.desc, "Description");

                ui.separator();

                combo_box_row(ui, &mut self.zone_type, "Zone Type");

                ui.horizontal(|ui| {
                    ui.label("Recommended Level Range");
                    ui.add(DragValue::new(&mut self.lvl_min));
                    ui.add(DragValue::new(&mut self.lvl_max));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Star Npc Loc");
                    self.start_npc_loc.draw(ui, holders);
                });

                num_row_optional(ui, &mut self.npc_id.0, "Start Npc", "Id", 0).on_hover_ui(|ui| {
                    if self.npc_id.0 > 0 {
                        holders
                            .game_data_holder
                            .npc_holder
                            .get(&self.npc_id)
                            .draw_as_tooltip(ui)
                    }
                });

                ui.separator();

                num_row(ui, &mut self.region_id.0, "Region Id");
                num_row(ui, &mut self.search_zone_id.0, "Search Zone Id");
                num_row_optional(ui, &mut self.instant_zone_id.0, "Instant Zone", "Id", 0);
            });

            ui.separator();

            self.quests.draw_vertical(
                ui,
                "Quests",
                |v| *action.write().unwrap() = HuntingZoneAction::RemoveQuest(v),
                holders,
                true,
                false,
            );

            ui.separator();
        });

        ui.separator();
    }
}

impl Frontend {
    pub fn draw_hunting_zone_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id)) in self
            .backend
            .edit_params
            .get_opened_hunting_zone_info()
            .iter()
            .enumerate()
        {
            let label = format!("[{}] {}", id.0, title);

            let mut button = Button::new(format_button_text(&label))
                .fill(Color32::from_rgb(99, 94, 47))
                .min_size([150., 10.].into());

            let is_current =
                CurrentEntity::HuntingZone(i) == self.backend.edit_params.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!("Hunting Zone: {label}"))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_hunting_zone(i);
            }

            if ui.button("❌").clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.close_hunting_zone(i);
            }

            ui.separator();
        }
    }

    pub(crate) fn draw_hunting_zone_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Hunting Zone    ").clicked() && backend.dialog.is_none() {
                backend.edit_params.create_new_hunting_zone();
            }

            ui.horizontal(|ui| {
                let l =
                    ui.text_edit_singleline(&mut backend.filter_params.hunting_zone_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_hunting_zones();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.hunting_zone_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for i in range {
                            let q = &backend.filter_params.hunting_zone_catalog[i];

                            if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                                && backend.dialog.is_none()
                            {
                                backend.edit_params.open_hunting_zone(
                                    q.id,
                                    &mut backend.holders.game_data_holder.hunting_zone_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}

impl DrawAsTooltip for HuntingZone {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(format!("[{}]\n {}", self.id.0, self.name));

            if !self.desc.is_empty() {
                ui.label(self.desc.to_string());
            }
        });
    }
}
