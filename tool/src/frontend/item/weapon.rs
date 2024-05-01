use crate::backend::item::weapon::{WeaponAction, WeaponEnchantAction, WeaponVariationAction};
use crate::backend::{Backend, CurrentEntity};
use crate::entity::item::weapon::{
    Weapon, WeaponEnchantInfo, WeaponEnchantParams, WeaponMeshInfo, WeaponVariationInfo,
};
use crate::frontend::util::{
    bool_row, combo_box_row, format_button_text, num_row, text_row, Draw, DrawActioned, DrawCtx,
    DrawUtils,
};
use crate::frontend::{DrawEntity, Frontend};
use crate::holder::DataHolder;
use eframe::egui::{Button, Color32, Context, Key, Response, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawEntity<WeaponAction, ()> for Weapon {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<WeaponAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            self.base_info.draw_ctx(ui, ctx, holders);

            ui.vertical(|ui| {
                ui.set_width(400.);
                combo_box_row(ui, &mut self.weapon_type, "Type");
                combo_box_row(ui, &mut self.character_animation_type, "Animation");
                combo_box_row(ui, &mut self.mp_consume, "Mp Consume");

                ui.separator();

                combo_box_row(ui, &mut self.random_damage, "Random Damage");
                num_row(ui, &mut self.ertheia_fists_scale, "Ertheia Fist Scale");

                ui.separator();

                ui.scope(|ui| {
                    ui.set_height(105.);

                    self.mesh_info.draw_vertical(
                        ui,
                        "Meshes",
                        |v| {
                            *action.write().unwrap() = WeaponAction::RemoveMesh(v);
                        },
                        holders,
                        true,
                        true,
                    );
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        num_row(ui, &mut self.soulshot_count, "Soulshot Count");
                        num_row(ui, &mut self.spiritshot_count, "Spiritshot Count");
                        num_row(ui, &mut self.curvature, "Curvature");
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        bool_row(ui, &mut self.unk, "Unk");
                        bool_row(ui, &mut self.can_equip_hero, "No Olympiad Use");
                        bool_row(ui, &mut self.is_magic_weapon, "Is Magic weapon");
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    bool_row(ui, &mut self.can_ensoul, "Ensoulable");
                    if self.can_ensoul {
                        num_row(ui, &mut self.ensoul_count, "Count");
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    self.enchant_info.draw_as_button(
                        ui,
                        ctx,
                        holders,
                        "   Enchant Params   ",
                        &format!("Enchant Params {}", self.base_info.name),
                        &format!("{} weapon_enchant_params", self.base_info.id.0),
                    );

                    self.variation_info.draw_as_button(
                        ui,
                        ctx,
                        holders,
                        "   Variation Params   ",
                        &format!("Variation Params {}", self.base_info.name),
                        &format!("{} weapon_variation_params", self.base_info.id.0),
                    );
                });
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl DrawActioned<WeaponVariationAction, ()> for WeaponVariationInfo {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<WeaponVariationAction>,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Effects");

                num_row(ui, &mut self.effect_1, "1");
                num_row(ui, &mut self.effect_2, "2");
                num_row(ui, &mut self.effect_3, "3");
                num_row(ui, &mut self.effect_4, "4");
                num_row(ui, &mut self.effect_5, "5");
                num_row(ui, &mut self.effect_6, "6");
            });

            ui.separator();

            self.icon.draw_vertical(
                ui,
                "Icons",
                |v| {
                    *action.write().unwrap() = WeaponVariationAction::RemoveIcon(v);
                },
                holders,
                true,
                false,
            )
        });
    }
}

impl DrawActioned<WeaponEnchantAction, ()> for WeaponEnchantInfo {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<WeaponEnchantAction>,
        _params: &mut (),
    ) {
        ui.vertical(|ui| {
            num_row(ui, &mut self.junk, "Junk");
            self.params.draw_vertical(
                ui,
                "Params",
                |v| {
                    *action.write().unwrap() = WeaponEnchantAction::RemoveEnchant(v);
                },
                holders,
                true,
                true,
            )
        });
    }
}

impl Draw for WeaponEnchantParams {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(300.);
                text_row(ui, &mut self.effect, "Effect");

                ui.horizontal(|ui| {
                    ui.label("Effect Offset");
                    ui.add_space(5.);
                    self.effect_offset.draw(ui, holders);
                });

                num_row(ui, &mut self.effect_scale, "Effect Scale");
                num_row(ui, &mut self.effect_velocity, "Effect Velocity");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Mesh Offset");
                    ui.add_space(5.);
                    self.mesh_offset.draw(ui, holders);
                });

                ui.horizontal(|ui| {
                    ui.label("Mesh Scale");
                    ui.add_space(5.);
                    self.mesh_scale.draw(ui, holders);
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(300.);

                ui.horizontal(|ui| {
                    ui.label("Particle Offset");
                    ui.add_space(5.);
                    self.particle_offset.draw(ui, holders);
                });

                num_row(ui, &mut self.particle_scale, "Particle Scale");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Ring Offset");
                    ui.add_space(5.);
                    self.ring_offset.draw(ui, holders);
                });
                ui.horizontal(|ui| {
                    ui.label("Ring Scale");
                    ui.add_space(5.);
                    self.ring_scale.draw(ui, holders);
                });
            });
        })
        .response
    }
}

impl Draw for WeaponMeshInfo {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.vertical(|ui| {
            text_row(ui, &mut self.mesh, "Mesh");
            text_row(ui, &mut self.texture, "Texture");
            num_row(ui, &mut self.unk, "Unk")
        })
        .inner
    }
}

impl Frontend {
    pub fn draw_weapon_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id)) in self
            .backend
            .edit_params
            .get_opened_weapons_info()
            .iter()
            .enumerate()
        {
            let label = format!("[{}] {}", id.0, title);

            let mut button = Button::new(format_button_text(&label))
                .fill(Color32::from_rgb(99, 47, 47))
                .min_size([150., 10.].into());

            let is_current = CurrentEntity::Weapon(i) == self.backend.edit_params.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!("Weapon: {label}"))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_weapon(i);
            }

            if ui.button("❌").clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.close_weapon(i);
            }

            ui.separator();
        }
    }

    pub(crate) fn draw_weapon_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Weapon    ").clicked() && backend.dialog.is_none() {
                backend.edit_params.create_new_weapon();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.weapon_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_weapons();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.weapon_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for i in range {
                            let q = &backend.filter_params.weapon_catalog[i];

                            if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                                && backend.dialog.is_none()
                            {
                                backend.edit_params.open_weapon(
                                    q.id,
                                    &mut backend.holders.game_data_holder.weapon_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}
