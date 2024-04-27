use crate::backend::item::armor::ArmorAction;
use crate::backend::{Backend, CurrentOpenedEntity, Holders};
use crate::entity::item::armor::{
    Armor, ArmorMeshAdditionalF, ArmorMeshInfo, ArmorMeshes, CurrentArmorMesh,
};
use crate::frontend::util::{
    bool_row, combo_box_row, format_button_text, num_row, num_row_optional, text_row, Draw,
    DrawActioned, DrawCtx, DrawUtils,
};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, Key, Response, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawEntity<ArmorAction, ()> for Armor {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<ArmorAction>,
        holders: &mut Holders,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            self.base_info.draw_ctx(ui, ctx, holders);

            ui.vertical(|ui| {
                combo_box_row(ui, &mut self.armor_type, "Armor Type");
                text_row(ui, &mut self.attack_effect, "Effect");

                ui.horizontal(|ui| {
                    num_row(ui, &mut self.unk1, "Unk1");
                    bool_row(ui, &mut self.unk2, "Unk2");
                });

                num_row(ui, &mut self.mp_bonus, "Mp Bonus");
                num_row(ui, &mut self.hide_mask, "Hide Mask");

                num_row_optional(
                    ui,
                    &mut self.set_enchant_effect_id.0,
                    "Enchanted Set Effect Id",
                    "",
                    u8::MAX,
                );

                combo_box_row(ui, &mut self.underwater_body_type1, "Underwater 1");
                combo_box_row(ui, &mut self.underwater_body_type2, "Underwater 2");

                ui.scope(|ui| {
                    ui.set_height(160.);

                    self.item_sound.draw_vertical(
                        ui,
                        "Sounds",
                        |v| {
                            *action.write().unwrap() = ArmorAction::RemoveSound(v);
                        },
                        holders,
                        true,
                        true,
                    );
                });

                ui.add_space(10.);

                ui.horizontal(|ui| {
                    self.battle_stats.draw_as_button(
                        ui,
                        ctx,
                        holders,
                        "   Battle Params   ",
                        &format!("Battle Params {}", self.base_info.name),
                        &format!("{} armor_battle_params", self.base_info.id.0),
                    );

                    self.mesh_info.draw_as_button(
                        ui,
                        ctx,
                        holders,
                        "   Mesh Params   ",
                        &format!("Mesh Params {}", self.base_info.name),
                        &format!("{} armor_mesh_params", self.base_info.id.0),
                    );
                });
            });
        });

        ui.separator();
    }
}

impl DrawActioned<(), CurrentArmorMesh> for ArmorMeshes {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &Holders,
        _action: &RwLock<()>,
        params: &mut CurrentArmorMesh,
    ) {
        combo_box_row(ui, params, "");
        ui.separator();
        self[*params].draw(ui, holders);
    }
}

impl Draw for ArmorMeshInfo {
    fn draw(&mut self, ui: &mut Ui, holders: &Holders) -> Response {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(300.);
                self.base.unk1.draw_vertical_nc(ui, "Meshes", holders);
                ui.add_space(10.);
                self.base.unk2.draw_vertical_nc(ui, "Textures", holders);
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(400.);
                self.additional
                    .unk1
                    .draw_vertical_nc(ui, "Additional Meshes", holders);
                self.additional
                    .unk5
                    .draw_vertical_nc(ui, "Additional Textures", holders);
                text_row(ui, &mut self.additional.unk6, "Unk6");
            });
        })
        .response
    }
}

impl Draw for ArmorMeshAdditionalF {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        text_row(ui, &mut self.unk2, "");

        ui.horizontal(|ui| {
            num_row(ui, &mut self.unk3, "Unk3");
            num_row(ui, &mut self.unk4, "Unk4");
        })
        .response
    }
}

impl Frontend {
    pub fn draw_armor_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id)) in self
            .backend
            .edit_params
            .get_opened_armor_info()
            .iter()
            .enumerate()
        {
            let label = format!("[{}] {}", id.0, title);

            let mut button = Button::new(format_button_text(&label))
                .fill(Color32::from_rgb(77, 47, 99))
                .min_size([150., 10.].into());

            let is_current =
                CurrentOpenedEntity::Armor(i) == self.backend.edit_params.current_opened_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!("Armor: {label}"))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_armor(i);
            }

            if ui.button("❌").clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.close_armor(i);
            }

            ui.separator();
        }
    }

    pub(crate) fn draw_armor_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Armor    ").clicked() && backend.dialog.is_none() {
                backend.edit_params.create_new_armor();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.armor_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_armor();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.armor_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for i in range {
                            let q = &backend.filter_params.armor_catalog[i];

                            if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                                && backend.dialog.is_none()
                            {
                                backend.edit_params.open_armor(
                                    q.id,
                                    &mut backend.holders.game_data_holder.armor_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}
