use crate::backend::Backend;
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::entity_impl::item::weapon::{
    WeaponAction, WeaponEnchantAction, WeaponSoundAction, WeaponVariationAction,
};
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::entity::GameEntityT;
use crate::entity::item::weapon::{
    Weapon, WeaponEnchantInfo, WeaponEnchantParams, WeaponMeshInfo, WeaponSounds,
    WeaponVariationInfo,
};
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::{
    Draw, DrawActioned, DrawCtx, DrawUtils, bool_row, close_entity_button, combo_box_row,
    format_button_text, num_row, text_row_c,
};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, Response, ScrollArea, Stroke, Ui};
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
        let init_rect = ui.min_size();

        ui.horizontal(|ui| {
            self.base_info.draw_ctx(ui, ctx, holders, init_rect);

            ui.vertical(|ui| {
                ui.set_width(450.);

                ui.horizontal(|ui| {
                    combo_box_row(ui, &mut self.weapon_type, "Type");
                    combo_box_row(ui, &mut self.character_animation_type, "Animation");
                });
                combo_box_row(ui, &mut self.mp_consume, "Mp Consume");

                ui.separator();

                combo_box_row(ui, &mut self.random_damage, "Random Damage");
                num_row(ui, &mut self.ertheia_fists_scale, "Scale for Ertheia");

                ui.separator();

                ui.scope(|ui| {
                    ui.set_height(if self.mesh_info.len() > 1 { 180. } else { 140. });

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

                ui.separator();

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        num_row(ui, &mut self.soulshot_count, "Soulshot Count");
                        num_row(ui, &mut self.spiritshot_count, "Spiritshot Count");
                        num_row(ui, &mut self.curvature, "Curvature");
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        bool_row(ui, &mut self.unk, "Unk");
                        bool_row(ui, &mut self.is_hero_weapon, "No Olympiad Use");
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
                        &format!("Enchant Params {}", self.base_info.name.ru),
                        &format!("{} weapon_enchant_params", self.base_info.id.0),
                        init_rect,
                    );

                    self.variation_info.draw_as_button(
                        ui,
                        ctx,
                        holders,
                        "   Variation Params   ",
                        &format!("Variation Params {}", self.base_info.name.ru),
                        &format!("{} weapon_variation_params", self.base_info.id.0),
                        init_rect,
                    );

                    self.sound.draw_as_button(
                        ui,
                        ctx,
                        holders,
                        "   Sounds   ",
                        &format!("Sounds {}", self.base_info.name.ru),
                        &format!("{} weapon_sounds", self.base_info.id.0),
                        init_rect,
                    );
                });
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl DrawActioned<WeaponSoundAction, ()> for WeaponSounds {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<WeaponSoundAction>,
        _params: &mut (),
    ) {
        self.0.draw_vertical(
            ui,
            "Attack Sounds",
            |v| {
                *action.write().unwrap() = WeaponSoundAction::RemoveSound(v);
            },
            holders,
            false,
            false,
        );
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
                text_row_c(ui, &mut self.effect, "Effect");

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
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        ui.vertical(|ui| {
            text_row_c(ui, &mut self.mesh, "Mesh");
            self.texture.draw_vertical_nc(ui, "Textures", holders)
        })
        .response
    }
}

impl Frontend {
    pub fn draw_weapon_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .editors
            .get_opened_weapons_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(99, 47, 47))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::Weapon(i) == self.backend.editors.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Weapon: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.editors.set_current_weapon(i);
            }

            close_entity_button(ui, CurrentEntity::Weapon(i), &mut self.backend, *is_changed);

            ui.separator();
        }
    }

    pub(crate) fn draw_weapon_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.weapon_holder;
            let item_holder = &mut backend.holders.game_data_holder.item_holder;
            let catalog = &mut backend.entity_catalogs.weapon;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.editors;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_weapon();
            }

            ui.separator();

            let mut changed = None;

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(ui, 36., catalog.catalog.len(), |ui, range| {
                    ui.set_width(width - 5.);

                    for v in range {
                        let q = &catalog.catalog[v];

                        let mut has_unsaved_changes = false;

                        let info_state = if let Some((ind, v)) = edit_params
                            .weapons
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::Weapon(ind) {
                                EntityInfoState::Current
                            } else {
                                EntityInfoState::Opened
                            }
                        } else {
                            EntityInfoState::Nothing
                        };

                        ui.horizontal(|ui| {
                            if q.draw_catalog_buttons(
                                ui,
                                &mut changed,
                                info_state,
                                has_unsaved_changes,
                            )
                            .clicked()
                                && backend.dialog.is_none()
                                && !q.deleted
                            {
                                if ui.input(|i| i.modifiers.ctrl) && !has_unsaved_changes {
                                    edit_params.close_if_opened(GameEntityT::Weapon(q.id));
                                } else {
                                    edit_params.open_weapon(q.id, holder);
                                }
                            }
                        });
                    }
                });
            });

            if let Some(id) = changed {
                if let Some(v) = holder.get_mut(&id) {
                    v._deleted = !v._deleted;

                    if v._deleted {
                        item_holder.remove(&id);
                        edit_params.close_if_opened(GameEntityT::Weapon(id));
                        holder.inc_deleted();
                    } else {
                        item_holder.insert(id, (&(*v)).into());
                        holder.dec_deleted();
                    }

                    catalog.filter(holder, *filter_mode);

                    backend.check_for_unwrote_changed();
                }
            }
        });
    }
}
