use crate::backend::{
    Backend, Holders, NpcAction, NpcMeshAction, NpcSkillAnimationAction, NpcSoundAction,
};
use crate::entity::npc::{
    Npc, NpcAdditionalParts, NpcDecorationEffect, NpcEquipParams, NpcMeshParams, NpcProperty,
    NpcSkillAnimation, NpcSoundParams, NpcSummonParams,
};
use crate::frontend::util::{num_row, text_row, Build, Draw, DrawUtils};
use crate::frontend::{BuildAsTooltip, Frontend, ADD_ICON};
use eframe::egui;
use eframe::egui::color_picker::{color_edit_button_srgba, Alpha};
use eframe::egui::{Key, Response, ScrollArea, Ui};
use std::sync::RwLock;

impl BuildAsTooltip for Npc {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            if !self.title.is_empty() {
                ui.colored_label(self.title_color, self.title.to_string());
            };

            ui.label(format!("{} [{}]", self.name, self.id.0));
        });
    }
}

impl Npc {
    pub(crate) fn build(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        action: &RwLock<NpcAction>,
        holders: &mut Holders,
    ) {
        ui.horizontal(|ui| {
            ui.set_width(800.);
            ui.vertical(|ui| {
                ui.set_width(450.);

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.name, "Name");

                    ui.add_space(5.);

                    num_row(ui, &mut self.id.0, "Id");
                });

                ui.separator();

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.title, "Title");

                    ui.add_space(5.);

                    ui.add(egui::Label::new("Color"));
                    color_edit_button_srgba(ui, &mut self.title_color, Alpha::Opaque);
                });

                ui.horizontal(|ui| {
                    num_row(ui, &mut self.npc_type, "Npc Type");

                    ui.add_space(5.);

                    text_row(ui, &mut self.unreal_script_class, "Unreal Script");
                });

                num_row(ui, &mut self.social, "Social");

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Show HP"));
                    ui.add(egui::Checkbox::new(&mut self.show_hp, ""));

                    ui.add_space(5.);

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
                            v.draw(ui, &holders).on_hover_ui(|ui| {
                                holders
                                    .game_data_holder
                                    .skill_holder
                                    .get(&v.id)
                                    .map(|s| (s, (v.level - 1) as usize))
                                    .build_as_tooltip(ui)
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
                                            .build_as_tooltip(ui)
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
                self.mesh_params.build(
                    ui,
                    ctx,
                    holders,
                    "   Mesh Params   ",
                    &format!("Mesh Params {}", self.name),
                    &format!("{} npc_mesh", self.id.0),
                );

                self.sound_params.build(
                    ui,
                    ctx,
                    holders,
                    "   Sound Params   ",
                    &format!("Sound Params {}", self.name),
                    &format!("{} npc_sound", self.id.0),
                );

                self.summon_params.build(
                    ui,
                    ctx,
                    holders,
                    "   Summon Params   ",
                    &format!("Summon Params {}", self.name),
                    &format!("{} npc_summon", self.id.0),
                );

                self.equipment_params.build(
                    ui,
                    ctx,
                    holders,
                    "   Equipment Params   ",
                    &format!("Equipment Params {}", self.name),
                    &format!("{} npc_equip", self.id.0),
                );

                self.additional_parts.build(
                    ui,
                    ctx,
                    holders,
                    "   Additional Parts   ",
                    &format!("Additional Parts {}", self.name),
                    &format!("{} npc_additional_parts", self.id.0),
                );

                self.skill_animations.build(
                    ui,
                    ctx,
                    holders,
                    "   Skill Animations   ",
                    &format!("Skill Animations {}", self.name),
                    &format!("{} npc_skill_animations", self.id.0),
                );
            });
        });
    }
}

impl Build<NpcSkillAnimationAction> for Vec<NpcSkillAnimation> {
    fn build(&mut self, ui: &mut Ui, holders: &Holders, action: &RwLock<NpcSkillAnimationAction>) {
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
                                .build_as_tooltip(ui)
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

impl Build<()> for Option<NpcAdditionalParts> {
    fn build(&mut self, ui: &mut Ui, holders: &Holders, _action: &RwLock<()>) {
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
                text_row(ui, &mut part.class, "Unreal Class");

                num_row(ui, &mut part.chest.0, "Chest").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.chest)
                        .build_as_tooltip(ui)
                });

                num_row(ui, &mut part.legs.0, "Legs").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.legs)
                        .build_as_tooltip(ui)
                });

                num_row(ui, &mut part.gloves.0, "Gloves").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.gloves)
                        .build_as_tooltip(ui)
                });

                num_row(ui, &mut part.feet.0, "Feet").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.feet)
                        .build_as_tooltip(ui)
                });

                num_row(ui, &mut part.back.0, "Back").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.back)
                        .build_as_tooltip(ui)
                });

                num_row(ui, &mut part.hair_accessory.0, "Hair Accessory").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.hair_accessory)
                        .build_as_tooltip(ui)
                });

                num_row(ui, &mut part.hair_style, "Hair Style");

                num_row(ui, &mut part.right_hand.0, "Right Hand").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.right_hand)
                        .build_as_tooltip(ui)
                });

                num_row(ui, &mut part.left_hand.0, "Left Hand").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&part.left_hand)
                        .build_as_tooltip(ui)
                });

                ui.separator();
            }
        });
    }
}

impl Build<NpcMeshAction> for NpcMeshParams {
    fn build(&mut self, ui: &mut Ui, holders: &Holders, action: &RwLock<NpcMeshAction>) {
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
                    );

                    ui.separator();

                    self.decorations.draw_vertical(
                        ui,
                        "Decorations",
                        |v| *action.write().unwrap() = NpcMeshAction::RemoveMeshDecoration(v),
                        holders,
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
                    ui.add(egui::DragValue::new(&mut self.collision_radius_1));
                    ui.add(egui::DragValue::new(&mut self.collision_radius_2));
                });

                ui.horizontal(|ui| {
                    ui.label("Collision height");
                    ui.add(egui::DragValue::new(&mut self.collision_height_1));
                    ui.add(egui::DragValue::new(&mut self.collision_height_2));
                });
            })
        });
    }
}

impl Build<()> for NpcEquipParams {
    fn build(&mut self, ui: &mut Ui, holders: &Holders, _action: &RwLock<()>) {
        ui.vertical(|ui| {
            num_row(ui, &mut self.left_hand.0, "Left Hand").on_hover_ui(|ui| {
                holders
                    .game_data_holder
                    .item_holder
                    .get(&self.left_hand)
                    .build_as_tooltip(ui)
            });

            num_row(ui, &mut self.right_hand.0, "Right Hand").on_hover_ui(|ui| {
                holders
                    .game_data_holder
                    .item_holder
                    .get(&self.right_hand)
                    .build_as_tooltip(ui)
            });

            num_row(ui, &mut self.chest.0, "Chest").on_hover_ui(|ui| {
                holders
                    .game_data_holder
                    .item_holder
                    .get(&self.chest)
                    .build_as_tooltip(ui)
            });
        });
    }
}

impl Build<()> for NpcSummonParams {
    fn build(&mut self, ui: &mut Ui, _holders: &Holders, _action: &RwLock<()>) {
        ui.vertical(|ui| {
            num_row(ui, &mut self.summon_type, "Type");
            num_row(ui, &mut self.max_count, "Max Count");
            num_row(ui, &mut self.grade, "Grade");
            num_row(ui, &mut self.silhouette, "Silhouette");
        });
    }
}

impl Build<NpcSoundAction> for NpcSoundParams {
    fn build(&mut self, ui: &mut Ui, holders: &Holders, action: &RwLock<NpcSoundAction>) {
        ui.horizontal(|ui| {
            ui.set_width(800.);
            ui.set_height(400.);

            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        num_row(ui, &mut self.vol, "Volume");
                        num_row(ui, &mut self.rad, "Radius");
                    });
                    ui.horizontal(|ui| {
                        num_row(ui, &mut self.random, "Random");
                        num_row(ui, &mut self.priority, "Priority");
                    });

                    ui.horizontal(|ui| {
                        self.damage_sound.draw_vertical(
                            ui,
                            "Damage",
                            |v| *action.write().unwrap() = NpcSoundAction::RemoveSoundDamage(v),
                            holders,
                            false,
                        );

                        self.attack_sound.draw_vertical(
                            ui,
                            "Attack",
                            |v| *action.write().unwrap() = NpcSoundAction::RemoveSoundAttack(v),
                            holders,
                            false,
                        );
                    });

                    ui.horizontal(|ui| {
                        self.defence_sound.draw_vertical(
                            ui,
                            "Defence",
                            |v| *action.write().unwrap() = NpcSoundAction::RemoveSoundDefence(v),
                            holders,
                            false,
                        );
                        self.dialog_sound.draw_vertical(
                            ui,
                            "Dialog",
                            |v| *action.write().unwrap() = NpcSoundAction::RemoveSoundDialog(v),
                            holders,
                            false,
                        );
                    });
                });
            });
        });
    }
}

impl Draw for NpcProperty {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        ui.vertical(|ui| {
            let r = num_row(ui, &mut self.id.0, "Id");

            r.union(num_row(ui, &mut self.level, "Level"))
        })
        .inner
    }
}

impl Draw for NpcDecorationEffect {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        ui.horizontal(|ui| {
            let r = text_row(ui, &mut self.effect, "Effect");

            ui.add_space(5.);

            r.union(num_row(ui, &mut self.scale, "Scale"))
        })
        .response
    }
}

impl Frontend {
    pub(crate) fn build_npc_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Npc    ").clicked() && !backend.dialog_showing {
                backend.edit_params.npcs.add_new_npc();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.npc_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_npcs();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.npc_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for v in range {
                            let info = &backend.filter_params.npc_catalog[v];
                            if ui
                                .button(format!("ID: {}\n{}", info.id.0, info.name))
                                .clicked()
                                && !backend.dialog_showing
                            {
                                backend.edit_params.open_npc(
                                    info.id,
                                    &mut backend.holders.game_data_holder.npc_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}
