use crate::backend::{
    Holders, SkillAction, SkillEditWindowParams, SkillEnchantAction, SkillEnchantEditWindowParams,
};
use crate::entity::skill::{EnchantInfo, Skill, SkillType};
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
        _action: &mut SkillAction,
        holders: &mut Holders,
        edit_params: &mut SkillEditWindowParams,
    ) {
        ui.horizontal(|ui| {
            ui.set_width(600.);

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

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Animation"));
                    ui.add(egui::TextEdit::singleline(&mut self.animations[0]));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Icon"));
                    ui.add(egui::TextEdit::singleline(&mut self.icon));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Icon Panel"));
                    ui.add(egui::TextEdit::singleline(&mut self.icon_panel));
                });
            });

            ui.separator();

            ui.add_space(5.);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Level"));
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
                });

                let cur_level = &mut self.skill_levels[edit_params.current_level_index];

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Description Params"));
                    ui.add(egui::TextEdit::singleline(
                        &mut cur_level.description_params,
                    ));
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("MP"));
                            ui.add(egui::DragValue::new(&mut cur_level.mp_cost));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("HP"));
                            ui.add(egui::DragValue::new(&mut cur_level.hp_cost));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Cast Range"));
                            ui.add(egui::DragValue::new(&mut cur_level.cast_range));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Hit Time"));
                            ui.add(egui::DragValue::new(&mut cur_level.hit_time));
                        });
                    });

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Cooldown"));
                            ui.add(egui::DragValue::new(&mut cur_level.cool_time));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Reuse Delay"));
                            ui.add(egui::DragValue::new(&mut cur_level.reuse_delay));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Effect Point"));
                            ui.add(egui::DragValue::new(&mut cur_level.effect_point));
                        });
                    });
                });

                if !cur_level.available_enchants.is_empty() {
                    ui.horizontal(|ui| {
                        for i in 0..3.min(cur_level.available_enchants.len()) {
                            if ui
                                .button(format!(
                                    "[{}] {}",
                                    cur_level.available_enchants[i].inner.enchant_type,
                                    cur_level.available_enchants[i].inner.enchant_name
                                ))
                                .clicked()
                            {
                                cur_level.available_enchants[i].opened = true;
                            }
                            ui.button("🗑").clicked();
                            ui.separator();
                        }
                    });
                }

                if cur_level.available_enchants.len() > 3 {
                    ui.horizontal(|ui| {
                        for i in 3..6.min(cur_level.available_enchants.len()) {
                            if ui
                                .button(format!(
                                    "[{}] {}",
                                    cur_level.available_enchants[i].inner.enchant_type,
                                    cur_level.available_enchants[i].inner.enchant_name
                                ))
                                .clicked()
                            {
                                cur_level.available_enchants[i].opened = true;
                            }
                            ui.button("🗑").clicked();
                            ui.separator();
                        }
                    });
                }

                if cur_level.available_enchants.len() > 6 {
                    ui.horizontal(|ui| {
                        for i in 6..100.min(cur_level.available_enchants.len()) {
                            if ui
                                .button(format!(
                                    "[{}] {}",
                                    cur_level.available_enchants[i].inner.enchant_type,
                                    cur_level.available_enchants[i].inner.enchant_name
                                ))
                                .clicked()
                            {
                                cur_level.available_enchants[i].opened = true;
                            }
                            ui.button("🗑").clicked();
                            ui.separator();
                        }
                    });
                }

                for enchant in &mut cur_level.available_enchants {
                    if enchant.opened {
                        egui::Window::new(format!(
                            "{} [{}]",
                            enchant.inner.enchant_name, enchant.inner.enchant_type
                        ))
                        .id(egui::Id::new(
                            10000 * self.id.0 + cur_level.level * 100 + enchant.inner.enchant_type,
                        ))
                        .open(&mut enchant.opened)
                        .show(ctx, |ui| {
                            enchant
                                .inner
                                .build(ui, &mut enchant.action, &mut enchant.params);
                        });
                    }
                }
            });
        });

        ui.separator();
    }
}

impl EnchantInfo {
    pub(crate) fn build(
        &mut self,
        ui: &mut Ui,
        _action: &mut SkillEnchantAction,
        edit_params: &mut SkillEnchantEditWindowParams,
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

                ui.separator();

                ui.add(egui::Label::new("Enchant Description"));
                ui.add(egui::TextEdit::multiline(&mut self.enchant_description));

                ui.separator();

                ui.add(egui::Label::new("Skill Description Override"));
                ui.add(egui::TextEdit::multiline(&mut self.skill_description));

                ui.separator();
            });

            ui.separator();
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Level"));
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
                });

                let cur_level = &mut self.enchant_levels[0];

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Enchant Description Params"));
                    ui.add(egui::TextEdit::singleline(
                        &mut cur_level.enchant_description_params,
                    ));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Enchant Name Params"));
                    ui.add(egui::TextEdit::singleline(
                        &mut cur_level.enchant_name_params,
                    ));
                });

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Skill Description Params"));
                    ui.add(egui::TextEdit::singleline(
                        &mut cur_level.skill_description_params,
                    ));
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("MP"));
                            ui.add(egui::DragValue::new(&mut cur_level.mp_cost));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("HP"));
                            ui.add(egui::DragValue::new(&mut cur_level.hp_cost));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Cast Range"));
                            ui.add(egui::DragValue::new(&mut cur_level.cast_range));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Hit Time"));
                            ui.add(egui::DragValue::new(&mut cur_level.hit_time));
                        });
                    });

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Cooldown"));
                            ui.add(egui::DragValue::new(&mut cur_level.cool_time));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Reuse Delay"));
                            ui.add(egui::DragValue::new(&mut cur_level.reuse_delay));
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new("Effect Point"));
                            ui.add(egui::DragValue::new(&mut cur_level.effect_point));
                        });
                    });
                });
            });
            ui.separator();
        });
    }
}

impl Frontend {
    pub(crate) fn build_skill_selector(&mut self, ui: &mut Ui, max_height: f32) {
        ui.vertical(|ui| {
            ui.set_width(150.);
            ui.set_max_height(max_height);

            if ui.button("    New Skill    ").clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.create_new_skill();
            }

            ui.horizontal(|ui| {
                let l =
                    ui.text_edit_singleline(&mut self.backend.filter_params.skill_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    self.backend.filter_skills();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for q in &self.backend.filter_params.skill_catalog {
                        if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                            && !self.backend.dialog_showing
                        {
                            self.backend.edit_params.open_skill(
                                q.id,
                                &mut self.backend.holders.game_data_holder.skill_holder,
                            );
                        }
                    }
                });
            });
        });
    }
}
