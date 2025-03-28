use crate::backend::Backend;
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, WindowParams};
use crate::backend::entity_impl::skill::{
    SkillAction, SkillEditWindowParams, SkillEnchantAction, SkillEnchantEditWindowParams,
    SkillUceConditionAction,
};
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::common::ItemId;
use crate::entity::GameEntityT;
use crate::entity::skill::{
    EnchantInfo, EnchantLevelInfo, EquipStatus, PriorSkill, RacesSkillSoundInfo, Skill,
    SkillLevelInfo, SkillSoundInfo, SkillUseCondition, SoundInfo, StatConditionType,
};
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::num_value::NumberValue;
use crate::frontend::util::{
    Draw, DrawActioned, DrawUtils, bool_row, close_entity_button, combo_box_row,
    format_button_text, num_row, num_tooltip_row, text_row_c,
};
use crate::frontend::{ADD_ICON, DELETE_ICON, DrawAsTooltip, DrawEntity, Frontend};
use eframe::egui;
use eframe::egui::{
    Button, Color32, Context, Response, ScrollArea, Stroke, TextWrapMode, Ui, Vec2,
};
use std::sync::RwLock;

impl DrawEntity<SkillAction, SkillEditWindowParams> for Skill {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<SkillAction>,
        holders: &mut DataHolder,
        edit_params: &mut SkillEditWindowParams,
    ) {
        let init_rect = ui.min_size();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(300.);
                ui.horizontal(|ui| {
                    text_row_c(ui, &mut self.name, "Name");
                    ui.add_space(5.);
                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .skill_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });
                });

                ui.add(egui::TextEdit::multiline(self.description.as_mut_string()));

                ui.separator();

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            combo_box_row(ui, &mut self.skill_type, "Skill Type");
                        });

                        num_row(ui, &mut self.resist_cast, "Resist Cast");
                        num_row(ui, &mut self.magic_type, "Magic Type");
                        num_row(ui, &mut self.cast_style, "Cast Style");
                        num_row(ui, &mut self.skill_magic_type, "Skill Magic Type");
                        bool_row(ui, &mut self.is_debuff, "Is Debuff");
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        num_row(ui, &mut self.origin_skill.0, "Origin Skill").on_hover_ui(|ui| {
                            holders
                                .game_data_holder
                                .skill_holder
                                .get(&self.origin_skill)
                                .draw_as_tooltip(ui);
                        });

                        bool_row(ui, &mut self.is_double, "Is Double");

                        text_row_c(ui, &mut self.visual_effect, "Visual Effect");

                        bool_row(ui, &mut self.cast_bar_text_is_red, "Red Cast Bar");

                        num_tooltip_row(ui, &mut self.rumble_self, "Rumble Self", "??");

                        num_tooltip_row(ui, &mut self.rumble_target, "Rumble Target", "??");
                    });
                });

                ui.separator();

                text_row_c(ui, &mut self.animations[0], "Animation");

                text_row_c(ui, &mut self.icon, "Icon");
                text_row_c(ui, &mut self.icon_panel, "Icon Panel");

                ui.separator();

                ui.horizontal(|ui| {
                    self.sound_info.draw_as_button(
                        ui,
                        ctx,
                        holders,
                        "   Sounds Params   ",
                        &format!("Sounds Params {}", self.name),
                        &format!("{} skill_sound", self.id.0),
                        init_rect,
                    );

                    ui.label("Use Condition");

                    if ui.checkbox(&mut self.use_condition.is_some(), "").changed() {
                        if self.use_condition.is_some() {
                            self.use_condition = None;
                        } else {
                            self.use_condition = Some(WindowParams::default());
                        }
                    }

                    if let Some(cond) = &mut self.use_condition {
                        cond.draw_as_button(
                            ui,
                            ctx,
                            holders,
                            "   Edit   ",
                            &format!("Condition Params {}", self.name),
                            &format!("{} skill_condition", self.id.0),
                            init_rect,
                        );
                    }
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(600.);

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Level"));
                    if !self.skill_levels.is_empty() {
                        egui::ComboBox::from_id_salt(ui.next_auto_id())
                            .selected_text(format!(
                                "{}",
                                self.skill_levels[edit_params.current_level_index].level
                            ))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                ui.set_min_width(20.0);

                                for i in 0..self.skill_levels.len() {
                                    ui.selectable_value(
                                        &mut edit_params.current_level_index,
                                        i,
                                        format!("{}", self.skill_levels[i].level),
                                    );
                                }
                            });

                        if ui.button(DELETE_ICON).clicked() {
                            *action.write().unwrap() = SkillAction::DeleteLevel;
                        }
                    }

                    if ui.button(ADD_ICON).clicked() {
                        *action.write().unwrap() = SkillAction::AddLevel;
                    }
                });

                ui.separator();

                if !self.skill_levels.is_empty() {
                    self.skill_levels[edit_params.current_level_index]
                        .draw(action, ui, ctx, holders, init_rect)
                }
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl DrawAsTooltip for Skill {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!(
            "[{}]\n{}\n{}",
            self.id.0, self.name, self.description
        ));
    }
}

impl DrawAsTooltip for (&Skill, usize) {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        let s = self.0.skill_levels.get(self.1);
        ui.label(format!(
            "[{}]\n{}\n{}",
            self.0.id.0,
            if let Some(Some(n)) = s.map(|v| &v.name) {
                n
            } else {
                &self.0.name
            },
            if let Some(Some(n)) = s.map(|v| &v.description) {
                n
            } else {
                &self.0.description
            },
        ));
    }
}

impl DrawActioned<SkillUceConditionAction, ()> for SkillUseCondition {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<SkillUceConditionAction>,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(150.);
                ui.horizontal(|ui| {
                    combo_box_row(ui, &mut self.stat_condition_type, "Stat Condition");

                    if self.stat_condition_type != StatConditionType::None {
                        combo_box_row(ui, &mut self.comparison_type, "");
                    }

                    ui.add(NumberValue::new(&mut self.stat_percentage));
                    ui.label("%");
                });

                ui.separator();

                num_row(ui, &mut self.mask, "Mask");

                combo_box_row(ui, &mut self.equipment_condition, "Equip Type");

                if self.equipment_condition == EquipStatus::Weapon {
                    self.weapon_types.draw_horizontal(
                        ui,
                        "Weapon types",
                        |v| {
                            *action.write().unwrap() = SkillUceConditionAction::DeleteWeapon(v);
                        },
                        holders,
                        true,
                    );
                }

                ui.separator();

                if bool_row(
                    ui,
                    &mut (self.consumable_item_id != ItemId::NONE),
                    "Consumable Item",
                )
                .changed()
                {
                    if self.consumable_item_id == ItemId::NONE {
                        self.consumable_item_id = ItemId(1);
                        self.item_count = 1;
                    } else {
                        self.consumable_item_id = ItemId::NONE;
                        self.item_count = 0;
                    }
                }

                if self.consumable_item_id != ItemId::NONE {
                    ui.horizontal(|ui| {
                        num_row(ui, &mut self.consumable_item_id.0, "Id").on_hover_ui(|ui| {
                            holders
                                .game_data_holder
                                .item_holder
                                .get(&self.consumable_item_id)
                                .draw_as_tooltip(ui)
                        });

                        num_row(ui, &mut self.item_count, "Count");
                    });
                }
            });

            ui.separator();

            self.caster_prior_skill.draw_vertical(
                ui,
                "Effects on Caster",
                |v| {
                    *action.write().unwrap() = SkillUceConditionAction::DeleteEffectOnCaster(v);
                },
                holders,
                true,
                false,
            );

            ui.separator();

            self.target_prior_skill.draw_vertical(
                ui,
                "Effects on Target",
                |v| {
                    *action.write().unwrap() = SkillUceConditionAction::DeleteEffectOnTarget(v);
                },
                holders,
                true,
                false,
            );
        });
    }
}

impl Draw for PriorSkill {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
            holders
                .game_data_holder
                .skill_holder
                .get(&self.id)
                .draw_as_tooltip(ui)
        });
        num_row(ui, &mut self.level, "Level");
        num_row(ui, &mut self.sub_level, "Sub level")
    }
}

impl SkillLevelInfo {
    fn draw(
        &mut self,
        action: &RwLock<SkillAction>,
        ui: &mut Ui,
        ctx: &egui::Context,
        holders: &DataHolder,
        init_rect: Vec2,
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.);

                text_row_c(ui, &mut self.description_params, "Description Params");

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        num_row(ui, &mut self.mp_cost, "MP");
                        num_row(ui, &mut self.hp_cost, "HP");
                        num_row(ui, &mut self.cast_range, "Cast Range");
                        num_row(ui, &mut self.hit_time, "Hit Time");
                    });

                    ui.vertical(|ui| {
                        num_row(ui, &mut self.cool_time, "Cooldown");
                        num_row(ui, &mut self.reuse_delay, "Reuse Delay");
                        num_row(ui, &mut self.effect_point, "Effect Point");
                    });
                });
            });

            ui.separator();

            if self.level > 1 {
                ui.vertical(|ui| {
                    ui.label("Overrides");
                    ui.add_space(5.);
                    ui.horizontal(|ui| {
                        if bool_row(ui, &mut self.icon.is_some(), "Icon").changed() {
                            if self.icon.is_some() {
                                self.icon = None;
                            } else {
                                self.icon = Some("".into());
                            }
                        }

                        if let Some(v) = &mut self.icon {
                            ui.text_edit_singleline(v.as_mut_string());
                        }
                    });

                    ui.horizontal(|ui| {
                        if bool_row(ui, &mut self.icon_panel.is_some(), "Icon Panel").changed() {
                            if self.icon_panel.is_some() {
                                self.icon_panel = None;
                            } else {
                                self.icon_panel = Some("".into());
                            }
                        }

                        if let Some(v) = &mut self.icon_panel {
                            ui.text_edit_singleline(v.as_mut_string());
                        }
                    });

                    ui.horizontal(|ui| {
                        if bool_row(ui, &mut self.name.is_some(), "Name").changed() {
                            if self.name.is_some() {
                                self.name = None;
                            } else {
                                self.name = Some("".into());
                            }
                        }

                        if let Some(v) = &mut self.name {
                            ui.text_edit_singleline(v.as_mut_string());
                        }
                    });

                    if bool_row(ui, &mut self.description.is_some(), "Description").changed() {
                        if self.description.is_some() {
                            self.description = None;
                        } else {
                            self.description = Some("".into());
                        }
                    }

                    if let Some(v) = &mut self.description {
                        ui.text_edit_multiline(v.as_mut_string());
                    }
                });
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Enchants");

            if ui.button(ADD_ICON).clicked() {
                *action.write().unwrap() = SkillAction::AddEnchant
            }
        });

        if !self.available_enchants.is_empty() {
            ui.horizontal(|ui| {
                for i in 0..3.min(self.available_enchants.len()) {
                    let title = format!(
                        "[{}] {}",
                        self.available_enchants[i].inner.enchant_type,
                        self.available_enchants[i].inner.enchant_name
                    );

                    self.available_enchants[i].draw_as_button(
                        ui,
                        ctx,
                        holders,
                        &title,
                        &title,
                        &format!("{} skill_enchant_{}", self.level, i),
                        init_rect,
                    );

                    if ui.button(DELETE_ICON).clicked() {
                        *action.write().unwrap() = SkillAction::DeleteEnchant(i);
                    }
                    ui.separator();
                }
            });
        }

        if self.available_enchants.len() > 3 {
            ui.horizontal(|ui| {
                for i in 3..6.min(self.available_enchants.len()) {
                    let title = format!(
                        "[{}] {}",
                        self.available_enchants[i].inner.enchant_type,
                        self.available_enchants[i].inner.enchant_name
                    );

                    self.available_enchants[i].draw_as_button(
                        ui,
                        ctx,
                        holders,
                        &title,
                        &title,
                        &format!("{} skill_enchant_{}", self.level, i),
                        init_rect,
                    );

                    if ui.button(DELETE_ICON).clicked() {
                        *action.write().unwrap() = SkillAction::DeleteEnchant(i);
                    }
                    ui.separator();
                }
            });
        }

        if self.available_enchants.len() > 6 {
            ui.horizontal(|ui| {
                for i in 6..100.min(self.available_enchants.len()) {
                    let title = format!(
                        "[{}] {}",
                        self.available_enchants[i].inner.enchant_type,
                        self.available_enchants[i].inner.enchant_name
                    );

                    self.available_enchants[i].draw_as_button(
                        ui,
                        ctx,
                        holders,
                        &title,
                        &title,
                        &format!("{} skill_enchant_{}", self.level, i),
                        init_rect,
                    );

                    if ui.button(DELETE_ICON).clicked() {
                        *action.write().unwrap() = SkillAction::DeleteEnchant(i);
                    }
                    ui.separator();
                }
            });
        }
    }
}

impl DrawActioned<SkillEnchantAction, SkillEnchantEditWindowParams> for EnchantInfo {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        _holders: &DataHolder,
        _action: &RwLock<SkillEnchantAction>,
        params: &mut SkillEnchantEditWindowParams,
    ) {
        ui.horizontal(|ui| {
            ui.set_width(600.);

            ui.vertical(|ui| {
                ui.set_width(300.);

                ui.horizontal(|ui| {
                    num_row(ui, &mut self.enchant_type, "Type");

                    ui.separator();

                    bool_row(ui, &mut self.is_debuff, "Is Debuff");
                });

                text_row_c(ui, &mut self.enchant_name, "Name");
                text_row_c(ui, &mut self.enchant_icon, "Enchant Icon");

                ui.separator();

                ui.add(egui::Label::new("Enchant Description"));
                ui.add(egui::TextEdit::multiline(
                    self.enchant_description.as_mut_string(),
                ));

                ui.separator();

                if bool_row(
                    ui,
                    &mut self.skill_description.is_some(),
                    "Skill Description Override",
                )
                .changed()
                {
                    if self.skill_description.is_some() {
                        self.skill_description = None;
                    } else {
                        self.skill_description = Some("".into());
                    }
                }

                if let Some(v) = &mut self.skill_description {
                    ui.text_edit_multiline(v.as_mut_string());
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Level"));

                    if !self.enchant_levels.is_empty() {
                        egui::ComboBox::from_id_salt(ui.next_auto_id())
                            .selected_text(format!(
                                "{}",
                                self.enchant_levels[params.current_level_index].level
                            ))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                ui.set_min_width(20.0);

                                for i in 0..self.enchant_levels.len() {
                                    ui.selectable_value(
                                        &mut params.current_level_index,
                                        i,
                                        format!("{}", self.enchant_levels[i].level),
                                    );
                                }
                            });

                        if ui.button(" - ").clicked() {
                            self.enchant_levels.pop();
                            params.current_level_index = self.enchant_levels.len() - 1;
                            // *action.write().unwrap() = SkillAction::DeleteEnchantLevel(index);
                        }
                    }

                    if ui.button(ADD_ICON).clicked() {
                        self.enchant_levels
                            .push(if let Some(v) = self.enchant_levels.last() {
                                let mut next = v.clone();
                                next.level += 1;

                                next
                            } else {
                                EnchantLevelInfo::default()
                            });
                        params.current_level_index = self.enchant_levels.len() - 1;
                        // *action.write().unwrap() = SkillAction::AddEnchantLevel(index);
                    }
                });

                if !self.enchant_levels.is_empty() {
                    self.enchant_levels[params.current_level_index].draw(ui);
                }
            });

            ui.separator();
        });
    }
}

impl EnchantLevelInfo {
    fn draw(&mut self, ui: &mut Ui) {
        text_row_c(
            ui,
            &mut self.enchant_description_params,
            "Enchant Description Params",
        );
        text_row_c(ui, &mut self.enchant_name_params, "Enchant Name Params");
        text_row_c(
            ui,
            &mut self.skill_description_params,
            "Skill Description Params",
        );

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                num_row(ui, &mut self.mp_cost, "MP");
                num_row(ui, &mut self.hp_cost, "HP");
                num_row(ui, &mut self.cast_range, "Cast Range");
                num_row(ui, &mut self.hit_time, "Hit Time");
            });

            ui.vertical(|ui| {
                num_row(ui, &mut self.cool_time, "Cooldown");
                num_row(ui, &mut self.reuse_delay, "Reuse Delay");
                num_row(ui, &mut self.effect_point, "Effect Point");
            });
        });

        ui.horizontal(|ui| {
            if bool_row(ui, &mut self.icon.is_some(), "Icon").changed() {
                if self.icon.is_some() {
                    self.icon = None;
                } else {
                    self.icon = Some("".into());
                }
            }

            if let Some(v) = &mut self.icon {
                ui.text_edit_singleline(v.as_mut_string());
            }
        });

        ui.horizontal(|ui| {
            if bool_row(ui, &mut self.icon_panel.is_some(), "Icon Panel").changed() {
                if self.icon_panel.is_some() {
                    self.icon_panel = None;
                } else {
                    self.icon_panel = Some("".into());
                }
            }

            if let Some(v) = &mut self.icon_panel {
                ui.text_edit_singleline(v.as_mut_string());
            }
        });
    }
}

impl SoundInfo {
    pub(crate) fn draw(&mut self, ui: &mut Ui, title: &str) {
        ui.vertical(|ui| {
            text_row_c(ui, &mut self.sound, &format!("{} Sound", title));

            ui.horizontal(|ui| {
                num_row(ui, &mut self.vol, "Vol");
                num_row(ui, &mut self.rad, "Rad");
                num_row(ui, &mut self.delay, "Delay");
                num_row(ui, &mut self.source, "Source");
            });
        });
    }
}

impl RacesSkillSoundInfo {
    pub(crate) fn draw(&mut self, ui: &mut Ui, title: &str) {
        ui.vertical(|ui| {
            ui.set_width(200.);

            ui.add(egui::Label::new(title));

            ui.separator();

            text_row_c(ui, &mut self.mfighter, "M Fighter");
            text_row_c(ui, &mut self.ffighter, "F Fighter");

            text_row_c(ui, &mut self.mmagic, "M Magic");
            text_row_c(ui, &mut self.fmagic, "F Magic");

            text_row_c(ui, &mut self.melf, "M Elf");
            text_row_c(ui, &mut self.felf, "F Elf");

            text_row_c(ui, &mut self.mdark_elf, "M Dark Elf");
            text_row_c(ui, &mut self.fdark_elf, "F Dark Elf");

            text_row_c(ui, &mut self.mdwarf, "M Dwarf");
            text_row_c(ui, &mut self.fdwarf, "F Dwarf");

            text_row_c(ui, &mut self.morc, "M Orc");
            text_row_c(ui, &mut self.forc, "F Orc");

            text_row_c(ui, &mut self.mshaman, "M Shaman");
            text_row_c(ui, &mut self.fshaman, "F Shaman");

            text_row_c(ui, &mut self.mkamael, "M Kamael");
            text_row_c(ui, &mut self.fkamael, "F Kamael");

            text_row_c(ui, &mut self.mertheia, "M Ertheia");
            text_row_c(ui, &mut self.fertheia, "F Ertheia");
        });
    }
}

impl DrawActioned<(), ()> for SkillSoundInfo {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        _holders: &DataHolder,
        _action: &RwLock<()>,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_width(800.);

            ui.vertical(|ui| {
                ui.set_width(350.);

                ui.horizontal(|ui| {
                    num_row(ui, &mut self.vol, "Vol");

                    ui.separator();

                    num_row(ui, &mut self.rad, "Rad");
                });

                ui.separator();

                text_row_c(ui, &mut self.mextra_throw, "M Extra Throw");
                text_row_c(ui, &mut self.fextra_throw, "F Extra Throw");

                ui.separator();

                self.spell_effect_1.draw(ui, "Spell Effect 1");
                ui.separator();
                self.spell_effect_2.draw(ui, "Spell Effect 2");
                ui.separator();
                self.spell_effect_3.draw(ui, "Spell Effect 3");
                ui.separator();

                self.shot_effect_1.draw(ui, "Shot Effect 1");
                ui.separator();
                self.shot_effect_2.draw(ui, "Shot Effect 2");
                ui.separator();
                self.shot_effect_3.draw(ui, "Shot Effect 3");
                ui.separator();

                self.exp_effect_1.draw(ui, "Exp Effect 1");
                ui.separator();
                self.exp_effect_2.draw(ui, "Exp Effect 2");
                ui.separator();
                self.exp_effect_3.draw(ui, "Exp Effect 3");
                ui.separator();
            });

            ui.separator();

            self.sound_before_cast.draw(ui, "Cast Info");

            ui.separator();

            self.sound_after_cast.draw(ui, "Magic Info");
        });

        ui.separator();
    }
}

impl Frontend {
    pub fn draw_skill_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .editors
            .get_opened_skills_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(47, 73, 99))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::Skill(i) == self.backend.editors.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Skill: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.editors.set_current_skill(i);
            }

            close_entity_button(ui, CurrentEntity::Skill(i), &mut self.backend, *is_changed);

            ui.separator();
        }
    }
    pub(crate) fn draw_skill_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.skill_holder;
            let catalog = &mut backend.entity_catalogs.skill;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.editors;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_skill();
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
                            .skills
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::Skill(ind) {
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
                                    edit_params.close_if_opened(GameEntityT::Skill(q.id));
                                } else {
                                    edit_params.open_skill(q.id, holder);
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
                        edit_params.close_if_opened(GameEntityT::Skill(id));
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
