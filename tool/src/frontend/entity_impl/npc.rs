use crate::backend::entity_editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::entity_impl::npc::{
    NpcAction, NpcMeshAction, NpcSkillAnimationAction, NpcSoundAction,
};
use crate::backend::holder::DataHolder;
use crate::backend::Backend;
use crate::entity::npc::{
    Npc, NpcAdditionalParts, NpcDecorationEffect, NpcEquipParams, NpcMeshParams, NpcProperty,
    NpcSkillAnimation, NpcSoundParams, NpcSummonParams,
};
use crate::entity::EntityT;
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::num_value::NumberValue;
use crate::frontend::util::{
    bool_row, close_entity_button, combo_box_row, format_button_text, num_row, num_row_optional,
    text_row, Draw, DrawActioned, DrawUtils,
};
use crate::frontend::{DrawAsTooltip, DrawEntity, Frontend, ADD_ICON};
use eframe::egui;
use eframe::egui::color_picker::{color_edit_button_srgba, Alpha};
use eframe::egui::{Button, Color32, Context, Response, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawEntity<NpcAction, ()> for Npc {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<NpcAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        let init_rect = ui.min_size();

        ui.horizontal(|ui| {
            ui.set_width(800.);
            ui.vertical(|ui| {
                ui.set_width(450.);

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.name, "Name");

                    ui.add_space(5.);

                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .npc_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.title, "Title");

                    ui.add_space(5.);

                    ui.add(egui::Label::new("Color"));
                    color_edit_button_srgba(ui, &mut self.title_color, Alpha::Opaque);
                });

                ui.horizontal(|ui| {
                    num_row_optional(ui, &mut self.npc_type, "Npc Type", "", u16::MAX);

                    ui.add_space(5.);

                    text_row(ui, &mut self.unreal_script_class, "Unreal Script");
                });

                ui.horizontal(|ui| {
                    bool_row(ui, &mut self.social, "Social");
                    bool_row(ui, &mut self.show_hp, "Show HP");
                });

                ui.horizontal(|ui| {
                    num_row(ui, &mut self.org_hp, "HP");

                    ui.add_space(5.);

                    num_row(ui, &mut self.org_mp, "MP");
                });

                text_row(ui, &mut self.icon, "Npc Icon");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Property Skills"));
                    if ui.button(ADD_ICON).clicked() {
                        self.properties.push(Default::default());
                    }
                });

                ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (i, v) in self.properties.iter_mut().enumerate() {
                            v.draw(ui, holders).on_hover_ui(|ui| {
                                holders
                                    .game_data_holder
                                    .skill_holder
                                    .get(&v.id)
                                    .map(|s| (s, (v.level - 1) as usize))
                                    .draw_as_tooltip(ui)
                            });

                            if ui.button(" - ").clicked() {
                                *action.write().unwrap() = NpcAction::RemoveProperty(i);
                            }

                            ui.separator();
                        }
                    });
                    ui.add_space(8.0);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Quests"));
                    if ui.button(ADD_ICON).clicked() {
                        self.quest_infos.push(Default::default());
                    }
                });

                ui.scope(|ui| {
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (i, v) in self.quest_infos.iter_mut().enumerate() {
                                ui.vertical(|ui| {
                                    num_row(ui, &mut v.id.0, "Id").on_hover_ui(|ui| {
                                        holders
                                            .game_data_holder
                                            .quest_holder
                                            .get(&v.id)
                                            .draw_as_tooltip(ui)
                                    });

                                    num_row(ui, &mut v.step, "Step")
                                });

                                if ui.button(" - ").clicked() {
                                    *action.write().unwrap() = NpcAction::RemoveQuest(i);
                                }

                                ui.separator();
                            }
                        });
                        ui.add_space(8.0);
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                self.mesh_params.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Mesh Params   ",
                    &format!("Mesh Params {}", self.name),
                    &format!("{} npc_mesh", self.id.0),
                    init_rect,
                );

                self.sound_params.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Sound Params   ",
                    &format!("Sound Params {}", self.name),
                    &format!("{} npc_sound", self.id.0),
                    init_rect,
                );

                self.summon_params.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Summon Params   ",
                    &format!("Summon Params {}", self.name),
                    &format!("{} npc_summon", self.id.0),
                    init_rect,
                );

                self.equipment_params.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Equipment Params   ",
                    &format!("Equipment Params {}", self.name),
                    &format!("{} npc_equip", self.id.0),
                    init_rect,
                );

                self.additional_parts.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Additional Parts   ",
                    &format!("Additional Parts {}", self.name),
                    &format!("{} npc_additional_parts", self.id.0),
                    init_rect,
                );

                self.skill_animations.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Skill Animations   ",
                    &format!("Skill Animations {}", self.name),
                    &format!("{} npc_skill_animations", self.id.0),
                    init_rect,
                );
            });

            ui.separator();
        });
        ui.separator();
    }
}

impl DrawAsTooltip for Npc {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            if !self.title.is_empty() {
                ui.colored_label(self.title_color, self.title.to_string());
            };

            ui.label(format!("{} [{}]", self.name, self.id.0));
        });
    }
}

impl DrawActioned<NpcSkillAnimationAction, ()> for Vec<NpcSkillAnimation> {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<NpcSkillAnimationAction>,
        _params: &mut (),
    ) {
        ui.set_height(100.);

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button(ADD_ICON).clicked() {
                        self.push(Default::default());
                    }
                });

                for (i, v) in self.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        num_row(ui, &mut v.id.0, "Id").on_hover_ui(|ui| {
                            holders
                                .game_data_holder
                                .skill_holder
                                .get(&v.id)
                                .draw_as_tooltip(ui)
                        });

                        ui.add_space(5.0);

                        text_row(ui, &mut v.animation, "Animation");

                        if ui.button(" - ").clicked() {
                            *action.write().unwrap() =
                                NpcSkillAnimationAction::RemoveSkillAnimation(i);
                        }
                    });
                }
            });
        });
    }
}

impl DrawActioned<(), ()> for Option<NpcAdditionalParts> {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        _action: &RwLock<()>,
        _params: &mut (),
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Enabled");

                if ui
                    .add(egui::Checkbox::new(&mut self.is_some(), ""))
                    .clicked()
                {
                    *self = if self.is_some() {
                        None
                    } else {
                        Some(NpcAdditionalParts::default())
                    }
                }
            });

            if let Some(part) = self {
                text_row(ui, &mut part.class, "Unreal Script");

                num_row(ui, &mut part.chest.0, "Chest").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.chest)
                        .draw_as_tooltip(ui)
                });

                num_row(ui, &mut part.legs.0, "Legs").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.legs)
                        .draw_as_tooltip(ui)
                });

                num_row(ui, &mut part.gloves.0, "Gloves").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.gloves)
                        .draw_as_tooltip(ui)
                });

                num_row(ui, &mut part.feet.0, "Feet").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.feet)
                        .draw_as_tooltip(ui)
                });

                num_row(ui, &mut part.back.0, "Back").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.back)
                        .draw_as_tooltip(ui)
                });

                num_row(ui, &mut part.hair_accessory.0, "Hair Accessory").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.hair_accessory)
                        .draw_as_tooltip(ui)
                });

                num_row(ui, &mut part.hair_style, "Hair Style");

                num_row(ui, &mut part.right_hand.0, "Right Hand").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.right_hand)
                        .draw_as_tooltip(ui)
                });

                num_row(ui, &mut part.left_hand.0, "Left Hand").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.left_hand)
                        .draw_as_tooltip(ui)
                });

                ui.separator();
            }
        });
    }
}

impl DrawActioned<NpcMeshAction, ()> for NpcMeshParams {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<NpcMeshAction>,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_width(800.);
            ui.set_height(400.);
            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(400.);

                    text_row(ui, &mut self.mesh, "Mesh");

                    ui.separator();

                    self.textures.draw_vertical(
                        ui,
                        "Textures",
                        |v| *action.write().unwrap() = NpcMeshAction::RemoveMeshTexture(v),
                        holders,
                        false,
                        false,
                    );

                    ui.separator();

                    self.additional_textures.draw_vertical(
                        ui,
                        "Additional Textures",
                        |v| {
                            *action.write().unwrap() = NpcMeshAction::RemoveMeshAdditionalTexture(v)
                        },
                        holders,
                        false,
                        false,
                    );

                    ui.separator();

                    self.decorations.draw_vertical(
                        ui,
                        "Decorations",
                        |v| *action.write().unwrap() = NpcMeshAction::RemoveMeshDecoration(v),
                        holders,
                        false,
                        false,
                    );
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                text_row(ui, &mut self.attack_effect, "Attack Effect");
                num_row(ui, &mut self.speed, "Speed");
                num_row(ui, &mut self.run_speed, "Run Speed");
                num_row(ui, &mut self.walk_speed, "Walk Speed");
                num_row(ui, &mut self.draw_scale, "Draw Scale");
                num_row(ui, &mut self.use_zoomincam, "Use Zoom In Cam");

                ui.horizontal(|ui| {
                    ui.label("Collision radius ");
                    ui.add(NumberValue::new(&mut self.collision_radius_1));
                    ui.add(NumberValue::new(&mut self.collision_radius_2));
                });

                ui.horizontal(|ui| {
                    ui.label("Collision height");
                    ui.add(NumberValue::new(&mut self.collision_height_1));
                    ui.add(NumberValue::new(&mut self.collision_height_2));
                });
            })
        });
    }
}

impl DrawActioned<(), ()> for NpcEquipParams {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        _action: &RwLock<()>,
        _params: &mut (),
    ) {
        ui.vertical(|ui| {
            num_row(ui, &mut self.left_hand.0, "Left Hand").on_hover_ui(|ui| {
                holders
                    .game_data_holder
                    .item_holder
                    .get(&self.left_hand)
                    .draw_as_tooltip(ui)
            });

            num_row(ui, &mut self.right_hand.0, "Right Hand").on_hover_ui(|ui| {
                holders
                    .game_data_holder
                    .item_holder
                    .get(&self.right_hand)
                    .draw_as_tooltip(ui)
            });

            num_row(ui, &mut self.chest.0, "Chest").on_hover_ui(|ui| {
                holders
                    .game_data_holder
                    .item_holder
                    .get(&self.chest)
                    .draw_as_tooltip(ui)
            });
        });
    }
}

impl DrawActioned<(), ()> for NpcSummonParams {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        _holders: &DataHolder,
        _action: &RwLock<()>,
        _params: &mut (),
    ) {
        ui.vertical(|ui| {
            combo_box_row(ui, &mut self.summon_type, "Type");
            num_row(ui, &mut self.max_count, "Max Count");
            num_row(ui, &mut self.grade, "Grade");
            num_row(ui, &mut self.silhouette, "Silhouette");
        });
    }
}

impl DrawActioned<NpcSoundAction, ()> for NpcSoundParams {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<NpcSoundAction>,
        _params: &mut (),
    ) {
        ui.vertical(|ui| {
            ui.set_width(800.);
            ui.set_height(400.);

            ui.horizontal(|ui| {
                num_row(ui, &mut self.vol, "Volume");
                num_row(ui, &mut self.rad, "Radius");
                num_row(ui, &mut self.random, "Random");
                num_row(ui, &mut self.priority, "Priority");
            });

            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    self.damage_sound.draw_vertical(
                        ui,
                        "Damage",
                        |v| *action.write().unwrap() = NpcSoundAction::RemoveSoundDamage(v),
                        holders,
                        false,
                        false,
                    );

                    ui.separator();

                    self.attack_sound.draw_vertical(
                        ui,
                        "Attack",
                        |v| *action.write().unwrap() = NpcSoundAction::RemoveSoundAttack(v),
                        holders,
                        false,
                        false,
                    );

                    ui.separator();
                });

                ui.separator();

                ui.horizontal(|ui| {
                    self.defence_sound.draw_vertical(
                        ui,
                        "Defence",
                        |v| *action.write().unwrap() = NpcSoundAction::RemoveSoundDefence(v),
                        holders,
                        false,
                        false,
                    );

                    ui.separator();

                    self.dialog_sound.draw_vertical(
                        ui,
                        "Dialog",
                        |v| *action.write().unwrap() = NpcSoundAction::RemoveSoundDialog(v),
                        holders,
                        false,
                        false,
                    );

                    ui.separator();
                });
            });

            ui.separator();
        });
    }
}

impl Draw for NpcProperty {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.vertical(|ui| {
            let r = num_row(ui, &mut self.id.0, "Id");

            r.union(num_row(ui, &mut self.level, "Level"))
        })
        .inner
    }
}

impl Draw for NpcDecorationEffect {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.horizontal(|ui| {
            let r = text_row(ui, &mut self.effect, "Effect");

            ui.add_space(5.);

            r.union(num_row(ui, &mut self.scale, "Scale"))
        })
        .response
    }
}

impl Frontend {
    pub fn draw_npc_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .edit_params
            .get_opened_npcs_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(47, 99, 96))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::Npc(i) == self.backend.edit_params.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Npc: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_npc(i);
            }

            close_entity_button(ui, CurrentEntity::Npc(i), &mut self.backend, *is_changed);

            ui.separator();
        }
    }

    pub(crate) fn draw_npc_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.npc_holder;
            let catalog = &mut backend.entity_catalogs.npc;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.edit_params;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_npc();
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
                            .npcs
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::Npc(ind) {
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
                                    edit_params.close_if_opened(EntityT::Npc(q.id));
                                } else {
                                    edit_params.open_npc(q.id, holder);
                                }
                            }
                        });
                    }
                });
            });

            if let Some(id) = changed {
                if let Some(v) = holder.inner.get_mut(&id) {
                    v._deleted = !v._deleted;

                    if v._deleted {
                        edit_params.close_if_opened(EntityT::Npc(id));
                        holder.inc_deleted();
                    } else {
                        holder.dec_deleted();
                    }

                    catalog.filter(holder, *filter_mode);

                    backend.check_for_unwrote_changed();
                }
            }
        });
    }
}
