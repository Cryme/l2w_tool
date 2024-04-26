use crate::backend::item::etc_item::EtcItemAction;
use crate::backend::{Backend, CurrentOpenedEntity, Holders};
use crate::entity::item::etc_item::{EtcItem, EtcMeshInfo};
use crate::frontend::util::{combo_box_row, text_row, Draw, DrawCtx, DrawUtils};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, Key, Response, ScrollArea, Ui};
use std::sync::RwLock;

impl DrawEntity<EtcItemAction, ()> for EtcItem {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<EtcItemAction>,
        holders: &mut Holders,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            self.base_info.draw_ctx(ui, ctx, holders);

            ui.vertical(|ui| {
                combo_box_row(ui, &mut self.etc_item_type, "Etc Type");
                combo_box_row(ui, &mut self.consume_type, "Consume Type");

                self.mesh_info.draw_vertical(
                    ui,
                    "Mesh",
                    |v| {
                        *action.write().unwrap() = EtcItemAction::RemoveMesh(v);
                    },
                    holders,
                    true,
                    true,
                );

                self.battle_stats.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Battle Params   ",
                    &format!("Battle Params {}", self.base_info.name),
                    &format!("{} etc_battle_params", self.base_info.id.0),
                );
            });
        });

        ui.separator();
    }
}

impl Draw for EtcMeshInfo {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        ui.vertical(|ui| {
            text_row(ui, &mut self.mesh, "Mesh");
            text_row(ui, &mut self.texture, "Texture")
        })
        .inner
    }
}

impl Frontend {
    pub fn draw_etc_items_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id)) in self
            .backend
            .edit_params
            .get_opened_etc_items_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format!("{} [{}]", title, id.0));

            let is_current =
                CurrentOpenedEntity::EtcItem(i) == self.backend.edit_params.current_opened_entity;

            if is_current {
                button = button.fill(Color32::from_rgb(42, 70, 83));
            }

            if ui.add(button).clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.set_current_etc_item(i);
            }

            if is_current && ui.button("Save").clicked() {
                self.backend.save_current_entity();
            }

            if ui.button("❌").clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.close_etc_item(i);
            }

            ui.separator();
        }
    }

    pub(crate) fn draw_etc_item_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Etc Item    ").clicked() && backend.dialog.is_none() {
                backend.edit_params.create_new_etc_item();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.etc_item_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_etc_items();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.etc_item_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for i in range {
                            let q = &backend.filter_params.etc_item_catalog[i];

                            if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                                && backend.dialog.is_none()
                            {
                                backend.edit_params.open_etc_item(
                                    q.id,
                                    &mut backend.holders.game_data_holder.etc_item_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}
