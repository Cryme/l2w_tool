use crate::backend::{Backend, Holders, QuestAction, StepAction, WindowParams};
use crate::data::{ItemId, NpcId, PlayerClass};
use crate::entity::quest::{
    GoalType, MarkType, Quest, QuestCategory, QuestReward, QuestStep, QuestType, StepGoal, Unk1,
    Unk2, UnkQLevel,
};
use crate::frontend::util::{
    combo_box_row, num_row, text_row, text_row_multiline, Draw, DrawUtils,
};
use crate::frontend::{BuildAsTooltip, Frontend, DELETE_ICON};
use eframe::egui;
use eframe::egui::{Key, Response, ScrollArea, Ui};
use std::sync::RwLock;
use strum::IntoEnumIterator;

impl BuildAsTooltip for Quest {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("[{}]\n{}", self.id.0, self.title));
    }
}

impl Draw for StepGoal {
    fn draw(&mut self, ui: &mut Ui, holders: &Holders) -> Response {
        combo_box_row(ui, &mut self.goal_type, GoalType::iter(), "Type");

        let r = ui.horizontal(|ui| {
            match self.goal_type {
                GoalType::KillNpc => {
                    num_row(ui, &mut self.target_id, "Monster Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .npc_holder
                            .get(&NpcId(self.target_id))
                            .build_as_tooltip(ui);
                    });
                }
                GoalType::CollectItem => {
                    num_row(ui, &mut self.target_id, "Item Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .item_holder
                            .get(&ItemId(self.target_id))
                            .build_as_tooltip(ui);
                    });
                }
                GoalType::Other => {
                    num_row(ui, &mut self.target_id, "Npc String Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .npc_strings
                            .get(&self.target_id)
                            .build_as_tooltip(ui);
                    });
                }
            };
        });

        if self.goal_type != GoalType::Other {
            num_row(ui, &mut self.count, "Count");
        };

        r.response
    }
}

impl QuestStep {
    fn build(
        &mut self,
        ui: &mut Ui,
        step_index: u32,
        action: &RwLock<StepAction>,
        holders: &mut Holders,
    ) {
        ui.vertical(|ui| {
            text_row(ui, &mut self.title, "Title");
            text_row(ui, &mut self.label, "Label");
            text_row_multiline(ui, &mut self.desc, "Description");
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_height(100.);

                self.goals.draw_vertical(
                    ui,
                    &format!("Goals: {}", self.goals.len()),
                    |v| *action.write().unwrap() = StepAction::RemoveGoal(v),
                    holders,
                    true,
                );
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label("Locations");

                ui.separator();

                ui.label("Base");
                self.location.draw(ui, holders);

                ui.separator();

                self.additional_locations.draw_vertical(
                    ui,
                    &format!("Additional: {}", self.additional_locations.len()),
                    |v| *action.write().unwrap() = StepAction::RemoveAdditionalLocation(v),
                    holders,
                    true,
                );
            });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                combo_box_row(ui, &mut self.unk_1, Unk1::iter(), "Unknown 1");
                combo_box_row(ui, &mut self.unk_2, Unk2::iter(), "Unknown 2");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Unknown 3");
                    ui.menu_button("+", |ui| {
                        for v in UnkQLevel::iter() {
                            if ui.button(format!("{v}")).clicked() {
                                self.unk_q_level.push(v);
                                ui.close_menu();
                            }
                        }
                    });
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.unk_q_level.clone().iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{v}"));
                                if ui.button(DELETE_ICON.to_string()).clicked() {
                                    self.unk_q_level.remove(i);
                                }
                            });
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Previous Steps");
                    if ui.button("+").clicked() {
                        self.prev_steps.push(0);
                    };
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.prev_steps.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.add(egui::DragValue::new(v)).changed() && *v > step_index {
                                    *v = 0;
                                }

                                if ui.button(DELETE_ICON.to_string()).clicked() {
                                    *action.write().unwrap() = StepAction::RemovePrevStepIndex(i);
                                }
                            });
                        }
                    });
                });
            });
        });

        ui.separator();
    }
}

impl Quest {
    pub(crate) fn build(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        action: &RwLock<QuestAction>,
        holders: &mut Holders,
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.title, "Name");
                    ui.add_space(5.);
                    num_row(ui, &mut self.id.0, "Id");
                });

                text_row_multiline(ui, &mut self.intro, "Intro");
                text_row_multiline(ui, &mut self.requirements, "Requirements");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(150.);

                combo_box_row(ui, &mut self.quest_type, QuestType::iter(), "Quest Type");
                combo_box_row(ui, &mut self.category, QuestCategory::iter(), "Category");
                combo_box_row(ui, &mut self.mark_type, MarkType::iter(), "Mark Type");

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Min lvl"));
                    ui.add_space(2.);

                    if self.min_lvl > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.min_lvl = 0;
                        }

                        ui.add(egui::DragValue::new(&mut self.min_lvl));
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.min_lvl = 1;
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Max lvl"));

                    if self.max_lvl > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.max_lvl = 0;
                        }

                        ui.add(egui::DragValue::new(&mut self.max_lvl));
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.max_lvl = 1;
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Prev Quest ID"));

                    if self.required_completed_quest_id.0 > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.required_completed_quest_id.0 = 0;
                        }

                        ui.add(egui::DragValue::new(
                            &mut self.required_completed_quest_id.0,
                        ))
                        .on_hover_ui(|ui| {
                            holders
                                .game_data_holder
                                .quest_holder
                                .get(&self.required_completed_quest_id)
                                .build_as_tooltip(ui);
                        });
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.required_completed_quest_id.0 = 1;
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Search Zone ID"));

                    if self.search_zone_id.0 > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.search_zone_id.0 = 0;
                        }

                        ui.add(egui::DragValue::new(&mut self.search_zone_id.0))
                            .on_hover_ui(|ui| {
                                holders
                                    .game_data_holder
                                    .hunting_zone_holder
                                    .get(&self.search_zone_id)
                                    .build_as_tooltip(ui)
                            });
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.search_zone_id.0 = 1;
                    }
                });

                if ui.button("Edit JAVA Class").clicked() {
                    if self.java_class.is_none() {
                        holders.set_java_class(self);
                    }

                    if let Some(v) = &mut self.java_class {
                        v.opened = true;
                    }
                }

                if let Some(class) = &mut self.java_class {
                    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());

                    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                        let mut layout_job = egui_extras::syntax_highlighting::highlight(
                            ui.ctx(),
                            &theme,
                            string,
                            "java",
                        );
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    egui::Window::new(format!("{} Java Class", self.title))
                        .id(egui::Id::new(2_000_000 + self.id.0))
                        .open(&mut class.opened)
                        .show(ctx, |ui| {
                            ScrollArea::vertical().show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut class.inner)
                                        .font(egui::TextStyle::Monospace) // for cursor height
                                        .code_editor()
                                        .desired_rows(10)
                                        .lock_focus(true)
                                        .desired_width(f32::INFINITY)
                                        .layouter(&mut layouter),
                                );
                            });
                        });
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(150.);

                ui.horizontal(|ui| {
                    ui.label("Allowed Classes");
                    if self.allowed_classes.is_some() {
                        if ui.checkbox(&mut true, "").changed() {
                            self.allowed_classes = None;
                        }
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.allowed_classes = Some(vec![]);
                    }

                    if let Some(allowed_classes) = &mut self.allowed_classes {
                        ui.menu_button("+", |ui| {
                            ui.push_id(ui.next_auto_id(), |ui| {
                                ScrollArea::vertical().show(ui, |ui| {
                                    let mut classes: Vec<_> = PlayerClass::iter()
                                        .filter(|v| !allowed_classes.contains(v))
                                        .collect();
                                    classes.sort_by(|a, b| format!("{a}").cmp(&format!("{b}")));

                                    for v in classes {
                                        if ui.button(format!("{v}")).clicked() {
                                            allowed_classes.push(v);
                                        }
                                    }
                                });
                            });
                        });
                    }
                });

                if let Some(allowed_classes) = &mut self.allowed_classes {
                    ui.push_id(ui.next_auto_id(), |ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            for (i, class) in allowed_classes.clone().iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{class}"));
                                    if ui.button(DELETE_ICON).clicked() {
                                        allowed_classes.remove(i);
                                    }
                                });
                            }
                        });
                    });
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.label("Start Npc Location");
                self.start_npc_loc.draw(ui, holders);

                ui.separator();

                self.start_npc_ids.draw_vertical(
                    ui,
                    "Start Npc Ids",
                    |v| {
                        *action.write().unwrap() = QuestAction::RemoveStartNpcId(v);
                    },
                    holders,
                    true,
                );
            });

            ui.separator();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.set_height(250.);

            self.quest_items.draw_vertical(
                ui,
                "Quest Items",
                |v| {
                    *action.write().unwrap() = QuestAction::RemoveQuestItem(v);
                },
                holders,
                true,
            );

            ui.separator();

            self.rewards.draw_vertical(
                ui,
                "Rewards",
                |v| {
                    *action.write().unwrap() = QuestAction::RemoveReward(v);
                },
                holders,
                true,
            );

            ui.separator();

            self.steps.draw_vertical(
                ui,
                &format!("Steps: {}", self.steps.len()),
                |v| {
                    *action.write().unwrap() = QuestAction::RemoveStep(v);
                },
                holders,
                true,
            );

            for (i, step) in self.steps.iter_mut().enumerate() {
                if step.opened {
                    egui::Window::new(format!("{} [{i}]", step.inner.title))
                        .id(egui::Id::new(10000 * self.id.0 + i as u32))
                        .open(&mut step.opened)
                        .show(ctx, |ui| {
                            step.inner.build(ui, i as u32, &mut step.action, holders);
                        });
                }
            }
        });

        ui.separator();
    }
}

impl Draw for WindowParams<QuestStep, (), StepAction, ()> {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        let button = ui.button(self.inner.title.to_string());
        if button.clicked() {
            self.opened = true;
        }

        button
    }
}
impl Draw for QuestReward {
    fn draw(&mut self, ui: &mut Ui, holders: &Holders) -> Response {
        ui.horizontal(|ui| {
            ui.label("ID");

            ui.add(egui::DragValue::new(&mut self.reward_id.0))
                .on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&self.reward_id)
                        .build_as_tooltip(ui)
                });

            ui.label("Count");
            ui.add(egui::DragValue::new(&mut self.count));
        })
        .response
    }
}

impl Frontend {
    pub(crate) fn build_quest_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Quest    ").clicked() && !backend.dialog_showing {
                backend.edit_params.create_new_quest();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.quest_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_quests();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.quest_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for i in range {
                            let q = &backend.filter_params.quest_catalog[i];

                            if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                                && !backend.dialog_showing
                            {
                                backend.edit_params.open_quest(
                                    q.id,
                                    &mut backend.holders.game_data_holder.quest_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}
