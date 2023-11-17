use crate::backend::{
    Backend, Holders, SkillAction, SkillEditWindowParams, SkillEnchantEditWindowParams,
    WindowParams,
};
use crate::data::{ItemId, ITEM_ID_NONE};
use crate::entity::skill::{
    EnchantInfo, EnchantLevelInfo, PriorSkill, RacesSkillSoundInfo, Skill, SkillAnimation,
    SkillLevelInfo, SkillSoundInfo, SkillType, SkillUseCondition, SoundInfo,
};
use crate::frontend::{BuildAsTooltip, Frontend};
use eframe::egui;
use eframe::egui::{Key, ScrollArea, Ui};
use strum::IntoEnumIterator;

impl BuildAsTooltip for Skill {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("[{}]\n{}", self.id.0, self.name));
    }
}

impl Skill {
    pub(crate) fn build(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        action: &mut SkillAction,
        holders: &mut Holders,
        edit_params: &mut SkillEditWindowParams,
    ) {
        ui.horizontal(|ui| {
            ui.set_width(800.);

            ui.vertical(|ui| {
                ui.set_width(300.);
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Name"));
                    ui.add(egui::TextEdit::singleline(&mut self.name));
                    ui.add_space(5.);
                    ui.add(egui::Label::new("Id"));
                    ui.add(egui::DragValue::new(&mut self.id.0));
                });

                ui.add(egui::TextEdit::multiline(&mut self.description));

                ui.separator();

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Skill Type");

                            egui::ComboBox::from_id_source(ui.next_auto_id())
                                .selected_text(format!("{}", self.skill_type))
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.set_min_width(20.0);

                                    for t in SkillType::iter() {
                                        ui.selectable_value(
                                            &mut self.skill_type,
                                            t,
                                            format!("{t}"),
                                        );
                                    }
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Resist Cast"));
                            ui.add(egui::DragValue::new(&mut self.resist_cast));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Magic Type"));
                            ui.add(egui::DragValue::new(&mut self.magic_type));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Cast Style"));
                            ui.add(egui::DragValue::new(&mut self.cast_style));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Skill Magic Type"));
                            ui.add(egui::DragValue::new(&mut self.skill_magic_type));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Is Debuff"));
                            ui.add(egui::Checkbox::new(&mut self.is_debuff, ""));
                        });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Origin Skill"));
                            ui.add(egui::DragValue::new(&mut self.origin_skill.0))
                                .on_hover_ui(|ui| {
                                    holders
                                        .game_data_holder
                                        .skill_holder
                                        .get(&self.origin_skill)
                                        .build_as_tooltip(ui);
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Is Double"));
                            ui.add(egui::Checkbox::new(&mut self.is_double, ""));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Visual Effect"));
                            ui.add(egui::DragValue::new(&mut self.visual_effect.0));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Red Cast Bar"));
                            ui.add(egui::Checkbox::new(&mut self.cast_bar_text_is_red, ""));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Rumble Self"));
                            ui.add(egui::DragValue::new(&mut self.rumble_self));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Rumble Target"));
                            ui.add(egui::DragValue::new(&mut self.rumble_target));
                        });
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Animation"));
                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.animations[0]))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in SkillAnimation::iter() {
                                ui.selectable_value(&mut self.animations[0], t, format!("{t}"));
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Icon"));
                    ui.add(egui::TextEdit::singleline(&mut self.icon));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Icon Panel"));
                    ui.add(egui::TextEdit::singleline(&mut self.icon_panel));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("   Edit Sounds   ").clicked() {
                        self.sound_info.opened = true;
                    }

                    if self.sound_info.opened {
                        egui::Window::new(format!("♫ {} ♫", self.name))
                            .id(egui::Id::new(format!("{} sound", self.id.0)))
                            .open(&mut self.sound_info.opened)
                            .show(ctx, |ui| {
                                self.sound_info.inner.build(ui);
                            });
                    }

                    ui.label("Use Condition");
                    let mut use_c = self.use_condition.is_some();
                    if ui.checkbox(&mut use_c, "").changed() {
                        if self.use_condition.is_some() {
                            self.use_condition = None;
                        } else {
                            self.use_condition = Some(WindowParams {
                                inner: SkillUseCondition::default(),
                                opened: false,
                                original_id: (),
                                action: (),
                                params: (),
                            });
                        }
                    }

                    if let Some(cond) = &mut self.use_condition {
                        if ui.button("   Edit   ").clicked() {
                            cond.opened = true;
                        }

                        if cond.opened {
                            egui::Window::new(format!("⚖ {} ⚖", self.name))
                                .id(egui::Id::new(format!("{} condition", self.id.0)))
                                .open(&mut cond.opened)
                                .show(ctx, |ui| {
                                    cond.inner.build(ui);
                                });
                        }
                    }
                });
            });

            ui.separator();

            ui.add_space(5.);

            ui.vertical(|ui| {
                ui.set_width(500.);

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Level"));
                    if !self.skill_levels.is_empty() {
                        egui::ComboBox::from_id_source(ui.next_auto_id())
                            .selected_text(format!(
                                "{}",
                                self.skill_levels[edit_params.current_level_index].level
                            ))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(20.0);

                                for i in 0..self.skill_levels.len() {
                                    ui.selectable_value(
                                        &mut edit_params.current_level_index,
                                        i,
                                        format!("{}", self.skill_levels[i].level),
                                    );
                                }
                            });

                        if ui.button("🗑").clicked() {
                            *action = SkillAction::DeleteLevel;
                        }
                    }

                    if ui.button("➕").clicked() {
                        *action = SkillAction::AddLevel;
                    }
                });

                if !self.skill_levels.is_empty() {
                    self.skill_levels[edit_params.current_level_index]
                        .build(action, ui, ctx, self.id.0)
                }
            });
        });

        ui.separator();
    }
}

impl SkillUseCondition {
    fn build(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(150.);
                ui.horizontal(|ui| {
                    ui.label("Stat type");
                    ui.add(egui::DragValue::new(&mut self.stat_type));
                    ui.label("%");
                    ui.add(egui::DragValue::new(&mut self.stat_percentage));
                });

                ui.horizontal(|ui| {
                    ui.label("Equip type");
                    ui.add(egui::DragValue::new(&mut self.equip_type));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Attack item type");
                    if ui.button("+").clicked() {
                        self.attack_item_type.push(0);
                    }
                    if ui.button("➖").clicked() {
                        self.attack_item_type.pop();
                    }
                });

                ui.horizontal(|ui| {
                    ui.push_id(ui.next_auto_id(), |ui| {
                        ScrollArea::horizontal().show(ui, |ui| {
                            for v in &mut self.attack_item_type {
                                ui.add(egui::DragValue::new(v));
                            }
                        })
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Up");
                    ui.add(egui::DragValue::new(&mut self.up));
                });

                let mut checked = self.item_id != ITEM_ID_NONE;
                ui.horizontal(|ui| {
                    ui.label("Item");
                    if ui.checkbox(&mut checked, "").changed() {
                        if self.item_id == ITEM_ID_NONE {
                            self.item_id = ItemId(1);
                            self.item_count = 1;
                        } else {
                            self.item_id = ITEM_ID_NONE;
                            self.item_count = 0;
                        }
                    }
                });
                if checked {
                    ui.horizontal(|ui| {
                        ui.label("Id");
                        ui.add(egui::DragValue::new(&mut self.item_id.0));
                        ui.label("Count");
                        ui.add(egui::DragValue::new(&mut self.item_count));
                    });
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(150.);
                ui.label("Caster skills");
                for v in &mut self.caster_prior_skill {
                    v.build(ui);
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(150.);
                ui.label("Target skills");
                for v in &mut self.target_prior_skill {
                    v.build(ui);
                }
            });
        });
    }
}

impl PriorSkill {
    fn build(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut self.unk1));
            ui.add(egui::DragValue::new(&mut self.unk2));
        });
    }
}

impl SkillLevelInfo {
    fn build(
        &mut self,
        skill_action: &mut SkillAction,
        ui: &mut Ui,
        ctx: &egui::Context,
        skill_id: u32,
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Description Params"));
                    ui.add(egui::TextEdit::singleline(&mut self.description_params));
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("MP"));
                            ui.add(egui::DragValue::new(&mut self.mp_cost));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("HP"));
                            ui.add(egui::DragValue::new(&mut self.hp_cost));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Cast Range"));
                            ui.add(egui::DragValue::new(&mut self.cast_range));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Hit Time"));
                            ui.add(egui::DragValue::new(&mut self.hit_time));
                        });
                    });

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Cooldown"));
                            ui.add(egui::DragValue::new(&mut self.cool_time));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Reuse Delay"));
                            ui.add(egui::DragValue::new(&mut self.reuse_delay));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Effect Point"));
                            ui.add(egui::DragValue::new(&mut self.effect_point));
                        });
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Icon");
                    if ui.checkbox(&mut self.icon.is_some(), "").changed() {
                        if self.icon.is_some() {
                            self.icon = None;
                        } else {
                            self.icon = Some("".to_string());
                        }
                    }

                    if let Some(v) = &mut self.icon {
                        ui.text_edit_singleline(v);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Icon Panel");
                    if ui.checkbox(&mut self.icon_panel.is_some(), "").changed() {
                        if self.icon_panel.is_some() {
                            self.icon_panel = None;
                        } else {
                            self.icon_panel = Some("".to_string());
                        }
                    }

                    if let Some(v) = &mut self.icon_panel {
                        ui.text_edit_singleline(v);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Description");
                    if ui.checkbox(&mut self.description.is_some(), "").changed() {
                        if self.description.is_some() {
                            self.description = None;
                        } else {
                            self.description = Some("".to_string());
                        }
                    }
                });

                if let Some(v) = &mut self.description {
                    ui.text_edit_multiline(v);
                }
            });
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Enchants");

            if ui.button("➕").clicked() {
                *skill_action = SkillAction::AddEnchant
            }
        });

        if !self.available_enchants.is_empty() {
            ui.horizontal(|ui| {
                for i in 0..3.min(self.available_enchants.len()) {
                    if ui
                        .button(format!(
                            "[{}] {}",
                            self.available_enchants[i].inner.enchant_type,
                            self.available_enchants[i].inner.enchant_name
                        ))
                        .clicked()
                    {
                        self.available_enchants[i].opened = true;
                    }
                    if ui.button("🗑").clicked() {
                        *skill_action = SkillAction::DeleteEnchant(i);
                    }
                    ui.separator();
                }
            });
        }

        if self.available_enchants.len() > 3 {
            ui.horizontal(|ui| {
                for i in 3..6.min(self.available_enchants.len()) {
                    if ui
                        .button(format!(
                            "[{}] {}",
                            self.available_enchants[i].inner.enchant_type,
                            self.available_enchants[i].inner.enchant_name
                        ))
                        .clicked()
                    {
                        self.available_enchants[i].opened = true;
                    }
                    if ui.button("🗑").clicked() {
                        *skill_action = SkillAction::DeleteEnchant(i);
                    }
                    ui.separator();
                }
            });
        }

        if self.available_enchants.len() > 6 {
            ui.horizontal(|ui| {
                for i in 6..100.min(self.available_enchants.len()) {
                    if ui
                        .button(format!(
                            "[{}] {}",
                            self.available_enchants[i].inner.enchant_type,
                            self.available_enchants[i].inner.enchant_name
                        ))
                        .clicked()
                    {
                        self.available_enchants[i].opened = true;
                    }
                    if ui.button("🗑").clicked() {
                        *skill_action = SkillAction::DeleteEnchant(i);
                    }
                    ui.separator();
                }
            });
        }

        for (i, enchant) in self.available_enchants.iter_mut().enumerate() {
            if enchant.opened {
                egui::Window::new(format!(
                    "{} [{}]",
                    enchant.inner.enchant_name, enchant.inner.enchant_type
                ))
                .id(egui::Id::new(
                    10000 * skill_id + self.level * 100 + enchant.inner.enchant_type,
                ))
                .open(&mut enchant.opened)
                .show(ctx, |ui| {
                    enchant
                        .inner
                        .build(ui, skill_action, &mut enchant.params, i);
                });
            }
        }
    }
}

impl EnchantInfo {
    pub(crate) fn build(
        &mut self,
        ui: &mut Ui,
        skill_action: &mut SkillAction,
        edit_params: &mut SkillEnchantEditWindowParams,
        index: usize,
    ) {
        ui.horizontal(|ui| {
            ui.set_width(600.);

            ui.vertical(|ui| {
                ui.set_width(300.);

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Type"));
                    ui.add(egui::DragValue::new(&mut self.enchant_type));
                    ui.separator();
                    ui.add(egui::Label::new("Is Debuff"));
                    ui.add(egui::Checkbox::new(&mut self.is_debuff, ""));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Name"));
                    ui.add(egui::TextEdit::singleline(&mut self.enchant_name));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Enchant Icon"));
                    ui.add(egui::TextEdit::singleline(&mut self.enchant_icon));
                });

                ui.separator();

                ui.add(egui::Label::new("Enchant Description"));
                ui.add(egui::TextEdit::multiline(&mut self.enchant_description));

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Skill Description Override");
                    if ui
                        .checkbox(&mut self.skill_description.is_some(), "")
                        .changed()
                    {
                        if self.skill_description.is_some() {
                            self.skill_description = None;
                        } else {
                            self.skill_description = Some("".to_string());
                        }
                    }
                });

                if let Some(v) = &mut self.skill_description {
                    ui.text_edit_multiline(v);
                }
            });

            ui.separator();
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Level"));

                    if !self.enchant_levels.is_empty() {
                        egui::ComboBox::from_id_source(ui.next_auto_id())
                            .selected_text(format!(
                                "{}",
                                self.enchant_levels[edit_params.current_level_index].level
                            ))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(20.0);

                                for i in 0..self.enchant_levels.len() {
                                    ui.selectable_value(
                                        &mut edit_params.current_level_index,
                                        i,
                                        format!("{}", self.enchant_levels[i].level),
                                    );
                                }
                            });

                        if ui.button("🗑").clicked() {
                            *skill_action = SkillAction::DeleteEnchantLevel(index);
                        }
                    }

                    if ui.button("➕").clicked() {
                        *skill_action = SkillAction::AddEnchantLevel(index);
                    }
                });

                if !self.enchant_levels.is_empty() {
                    self.enchant_levels[edit_params.current_level_index].build(ui);
                }
            });
            ui.separator();
        });
    }
}

impl EnchantLevelInfo {
    fn build(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Enchant Description Params"));
            ui.add(egui::TextEdit::singleline(
                &mut self.enchant_description_params,
            ));
        });

        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Enchant Name Params"));
            ui.add(egui::TextEdit::singleline(&mut self.enchant_name_params));
        });

        ui.horizontal(|ui| {
            ui.add(egui::Label::new("Skill Description Params"));
            ui.add(egui::TextEdit::singleline(
                &mut self.skill_description_params,
            ));
        });

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("MP"));
                    ui.add(egui::DragValue::new(&mut self.mp_cost));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("HP"));
                    ui.add(egui::DragValue::new(&mut self.hp_cost));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Cast Range"));
                    ui.add(egui::DragValue::new(&mut self.cast_range));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Hit Time"));
                    ui.add(egui::DragValue::new(&mut self.hit_time));
                });
            });

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Cooldown"));
                    ui.add(egui::DragValue::new(&mut self.cool_time));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Reuse Delay"));
                    ui.add(egui::DragValue::new(&mut self.reuse_delay));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Effect Point"));
                    ui.add(egui::DragValue::new(&mut self.effect_point));
                });
            });
        });

        ui.horizontal(|ui| {
            ui.label("Icon");
            if ui.checkbox(&mut self.icon.is_some(), "").changed() {
                if self.icon.is_some() {
                    self.icon = None;
                } else {
                    self.icon = Some("".to_string());
                }
            }

            if let Some(v) = &mut self.icon {
                ui.text_edit_singleline(v);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Icon Panel");
            if ui.checkbox(&mut self.icon_panel.is_some(), "").changed() {
                if self.icon_panel.is_some() {
                    self.icon_panel = None;
                } else {
                    self.icon_panel = Some("".to_string());
                }
            }

            if let Some(v) = &mut self.icon_panel {
                ui.text_edit_singleline(v);
            }
        });
    }
}

impl SoundInfo {
    pub(crate) fn build(&mut self, ui: &mut Ui, title: &str) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(format!("{} Sound", title)));
                ui.add(egui::TextEdit::singleline(&mut self.sound));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Label::new("Vol"));
                ui.add(egui::DragValue::new(&mut self.vol));

                ui.add(egui::Label::new("Rad"));
                ui.add(egui::DragValue::new(&mut self.rad));

                ui.add(egui::Label::new("Delay"));
                ui.add(egui::DragValue::new(&mut self.delay));

                ui.add(egui::Label::new("Source"));
                ui.add(egui::DragValue::new(&mut self.source));
            });
        });
    }
}

impl RacesSkillSoundInfo {
    pub(crate) fn build(&mut self, ui: &mut Ui, title: &str) {
        ui.vertical(|ui| {
            ui.set_width(200.);

            ui.add(egui::Label::new(title));
            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("M Fighter"));
                ui.add(egui::TextEdit::singleline(&mut self.mfighter));
            });
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("F Fighter"));
                ui.add(egui::TextEdit::singleline(&mut self.ffighter));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Label::new("M Magic"));
                ui.add(egui::TextEdit::singleline(&mut self.mmagic));
            });
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("F Magic"));
                ui.add(egui::TextEdit::singleline(&mut self.fmagic));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Label::new("M Elf"));
                ui.add(egui::TextEdit::singleline(&mut self.melf));
            });
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("F Elf"));
                ui.add(egui::TextEdit::singleline(&mut self.felf));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Label::new("M Dark Elf"));
                ui.add(egui::TextEdit::singleline(&mut self.mdark_elf));
            });
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("F Dark Elf"));
                ui.add(egui::TextEdit::singleline(&mut self.fdark_elf));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Label::new("M Dwarf"));
                ui.add(egui::TextEdit::singleline(&mut self.mdwarf));
            });
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("F Dwarf"));
                ui.add(egui::TextEdit::singleline(&mut self.fdwarf));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Label::new("M Orc"));
                ui.add(egui::TextEdit::singleline(&mut self.morc));
            });
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("F Orc"));
                ui.add(egui::TextEdit::singleline(&mut self.forc));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Label::new("M Shaman"));
                ui.add(egui::TextEdit::singleline(&mut self.mshaman));
            });
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("F Shaman"));
                ui.add(egui::TextEdit::singleline(&mut self.fshaman));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Label::new("M Kamael"));
                ui.add(egui::TextEdit::singleline(&mut self.mkamael));
            });
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("F Kamael"));
                ui.add(egui::TextEdit::singleline(&mut self.fkamael));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Label::new("M Ertheia"));
                ui.add(egui::TextEdit::singleline(&mut self.mertheia));
            });
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("F Ertheia"));
                ui.add(egui::TextEdit::singleline(&mut self.fertheia));
            });
        });
    }
}

impl SkillSoundInfo {
    pub(crate) fn build(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.set_width(800.);

            ui.vertical(|ui| {
                ui.set_width(350.);

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Vol"));
                    ui.add(egui::DragValue::new(&mut self.vol));

                    ui.separator();

                    ui.add(egui::Label::new("Rad"));
                    ui.add(egui::DragValue::new(&mut self.rad));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("M Extra Throw"));
                    ui.add(egui::TextEdit::singleline(&mut self.mextra_throw));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("F Extra Throw"));
                    ui.add(egui::TextEdit::singleline(&mut self.fextra_throw));
                });

                ui.separator();

                self.spell_effect_1.build(ui, "Spell Effect 1");
                ui.separator();
                self.spell_effect_2.build(ui, "Spell Effect 2");
                ui.separator();
                self.spell_effect_3.build(ui, "Spell Effect 3");
                ui.separator();

                self.shot_effect_1.build(ui, "Shot Effect 1");
                ui.separator();
                self.shot_effect_2.build(ui, "Shot Effect 2");
                ui.separator();
                self.shot_effect_3.build(ui, "Shot Effect 3");
                ui.separator();

                self.exp_effect_1.build(ui, "Exp Effect 1");
                ui.separator();
                self.exp_effect_2.build(ui, "Exp Effect 2");
                ui.separator();
                self.exp_effect_3.build(ui, "Exp Effect 3");
                ui.separator();
            });

            ui.separator();

            self.sound_before_cast.build(ui, "Cast Info");

            ui.separator();

            self.sound_after_cast.build(ui, "Magic Info");
        });

        ui.separator();
    }
}

impl Frontend {
    pub(crate) fn build_skill_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Skill    ").clicked() && !backend.dialog_showing {
                backend.edit_params.create_new_skill();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.skill_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_skills();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for q in &backend.filter_params.skill_catalog {
                        if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                            && !backend.dialog_showing
                        {
                            backend.edit_params.open_skill(
                                q.id,
                                &mut backend.holders.game_data_holder.skill_holder,
                            );
                        }
                    }
                });
            });
        });
    }
}
