use crate::backend::{Backend, QuestAction, StepAction};
use crate::data::{Location, PlayerClass};
use crate::holders::GameDataHolder;
use crate::quest::{
    GoalType, MarkType, Quest, QuestCategory, QuestStep, QuestType, StepGoal, Unk1, Unk2, UnkQLevel,
};
use eframe::egui;
use eframe::egui::{ScrollArea, Ui};
use strum::IntoEnumIterator;

pub struct Frontend {
    backend: Backend,
}

impl Location {
    fn build(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(egui::DragValue::new(&mut self.x));

            ui.label("Y");
            ui.add(egui::DragValue::new(&mut self.y));

            ui.label("Z");
            ui.add(egui::DragValue::new(&mut self.z));
        });
    }
}

impl StepGoal {
    fn build(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Type");

            egui::ComboBox::from_id_source(ui.next_auto_id())
                .selected_text(format!("{}", self.goal_type))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(20.0);

                    for t in GoalType::iter() {
                        ui.selectable_value(&mut self.goal_type, t, format!("{t}"));
                    }
                });
        });

        ui.horizontal(|ui| {
            match self.goal_type {
                GoalType::KillNpc => {
                    ui.label("Monster Id");
                }
                GoalType::CollectItem => {
                    ui.label("Item Id");
                }
                GoalType::Other => {
                    ui.label("Npc String Id");
                }
                _ => {
                    ui.label(format!("{}", self.goal_type));
                },
            }
            ui.add(egui::DragValue::new(&mut self.target_id));
        });

        if self.goal_type != GoalType::Other {
            ui.horizontal(|ui| {
                ui.label("Count");
                ui.add(egui::DragValue::new(&mut self.count));
            });
        }
    }
}

impl QuestStep {
    fn build(&mut self, ui: &mut Ui, step_index: usize, action: &mut StepAction) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.label("Title");
                ui.text_edit_singleline(&mut self.title);
                ui.label("Label");
                ui.text_edit_singleline(&mut self.label);
                ui.label("Description");
                ui.text_edit_multiline(&mut self.desc);

                ui.separator();

                ui.label("Target Display Name");
                ui.text_edit_singleline(&mut self.target_display_name);
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(150.);
                ui.set_max_height(200.);

                ui.horizontal(|ui| {
                    ui.label(format!("Goals: {}", self.goals.len()));
                    if ui.button("+").clicked() {
                        self.add_goal();
                    }
                });

                ui.separator();

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.goals.iter_mut().enumerate() {
                            v.build(ui);

                            if ui.button("🗑").clicked() {
                                *action = StepAction::RemoveGoal(i);
                            }

                            ui.separator();
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label("Locations");

                ui.separator();

                ui.label("Base");
                self.location.build(ui);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Additional");

                    if ui.button("+").clicked() {
                        self.add_additional_location();
                    }
                });

                for (i, location) in self.additional_locations.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        location.build(ui);
                        if ui.button("🗑").clicked() {
                            *action = StepAction::RemoveAdditionalLocation(i);
                        }
                    });
                }
            });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Unknown 1");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.unk_1))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in Unk1::iter() {
                                ui.selectable_value(&mut self.unk_1, t, format!("{t}"));
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Unknown 2");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.unk_2))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in Unk2::iter() {
                                ui.selectable_value(&mut self.unk_2, t, format!("{t}"));
                            }
                        });
                });
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
                                if ui.button(format!("🗑")).clicked() {
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
                        self.prev_step_indexes.push(0);
                    };
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.prev_step_indexes.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.add(egui::DragValue::new(v)).changed() {
                                    if *v > step_index {
                                        *v = 0;
                                    }
                                }

                                if ui.button(format!("🗑")).clicked() {
                                    *action = StepAction::RemovePrevStepIndex(i);
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
    fn build(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        action: &mut QuestAction,
        holder: &mut GameDataHolder,
    ) {
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Name"));
                    ui.add(egui::TextEdit::singleline(&mut self.title));
                    ui.add_space(5.);
                    ui.add(egui::Label::new("Id"));
                    ui.add(egui::DragValue::new(&mut self.id.0));
                });

                ui.label("Intro");
                ui.text_edit_multiline(&mut self.intro);
                ui.label("Requirements");
                ui.text_edit_multiline(&mut self.requirements);
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(150.);

                ui.horizontal(|ui| {
                    ui.label("Quest Type");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.quest_type))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in QuestType::iter() {
                                ui.selectable_value(&mut self.quest_type, t, format!("{t}"));
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Category");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.category))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in QuestCategory::iter() {
                                ui.selectable_value(&mut self.category, t, format!("{t}"));
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Mark Type");

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.mark_type))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in MarkType::iter() {
                                ui.selectable_value(&mut self.mark_type, t, format!("{t}"));
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Min lvl"));
                    ui.add_space(2.);

                    if self.min_lvl > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.min_lvl = 0;
                        }

                        ui.add(egui::DragValue::new(&mut self.min_lvl));
                    } else {
                        if ui.checkbox(&mut false, "").changed() {
                            self.min_lvl = 1;
                        }
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
                    } else {
                        if ui.checkbox(&mut false, "").changed() {
                            self.max_lvl = 1;
                        }
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
                        ));
                    } else {
                        if ui.checkbox(&mut false, "").changed() {
                            self.required_completed_quest_id.0 = 1;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Search Zone ID"));

                    if self.search_zone_id.0 > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.search_zone_id.0 = 0;
                        }

                        ui.add(egui::DragValue::new(&mut self.search_zone_id.0));
                    } else {
                        if ui.checkbox(&mut false, "").changed() {
                            self.search_zone_id.0 = 1;
                        }
                    }
                });

                if ui.button("Edit JAVA Class").clicked() {
                    if self.java_class.is_none() {
                        holder.set_java_class(self);
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
                    } else {
                        if ui.checkbox(&mut false, "").changed() {
                            self.allowed_classes = Some(vec![]);
                        }
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
                                if ui.button(format!("{class}")).clicked() {
                                    allowed_classes.remove(i);
                                }
                            }
                        });
                    });
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.label("Start Npc Location");
                self.start_npc_loc.build(ui);

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Start Npc Ids");
                    if ui.button("+").clicked() {
                        self.add_start_npc_id();
                    }
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, id) in self.start_npc_ids.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut id.0));
                                if ui.button("🗑").clicked() {
                                    *action = QuestAction::RemoveStartNpcId(i);
                                }
                            });
                        }
                    });
                });
            });

            ui.separator();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.set_height(250.);

            ui.vertical(|ui| {
                ui.set_width(100.);
                ui.horizontal(|ui| {
                    ui.label("Quest Items");
                    if ui.button("+").clicked() {
                        self.add_quest_item();
                    }
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, id) in self.quest_items.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut id.0));
                                if ui.button("🗑").clicked() {
                                    *action = QuestAction::RemoveQuestItem(i);
                                }
                            });
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(100.);

                ui.horizontal(|ui| {
                    ui.label("Rewards");
                    if ui.button("+").clicked() {
                        self.add_reward();
                    }
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, reward) in self.rewards.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label("ID");
                                ui.add(egui::DragValue::new(&mut reward.reward_id.0));

                                ui.label("Count");
                                ui.add(egui::DragValue::new(&mut reward.count));

                                if ui.button("🗑").clicked() {
                                    *action = QuestAction::RemoveReward(i);
                                }
                            });
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(100.);

                ui.horizontal(|ui| {
                    ui.label(format!("Steps: {}", self.steps.len()));

                    if ui.button("+").clicked() {
                        self.add_step();
                    }
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, step) in self.steps.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.button(format!("{}", step.inner.title)).clicked() {
                                    if !step.opened {
                                        step.opened = true;
                                    }
                                }

                                if ui.button("🗑").clicked() {
                                    *action = QuestAction::RemoveStep(i);
                                }
                            });
                        }
                    });
                });

                for (i, step) in self.steps.iter_mut().enumerate() {
                    if step.opened {
                        egui::Window::new(format!("{} [{i}]", step.inner.title))
                            .id(egui::Id::new(10000 * self.id.0 + i as u32))
                            .open(&mut step.opened)
                            .show(ctx, |ui| {
                                step.inner.build(ui, i, &mut step.action);
                            });
                    }
                }
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl Frontend {
    fn build_quest_editor(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let Some(quest) = &mut self.backend.current_quest else {
            return;
        };

        quest
            .inner
            .build(ui, ctx, &mut quest.action, &mut self.backend.holder);
    }

    fn build_top_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Options", |ui| {
            if ui.button("New Quest").clicked() {
                self.backend.create_new_quest();
                ui.close_menu();
            }
        });
    }

    pub fn init(ctx: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&ctx.egui_ctx);

        Self {
            backend: Backend::init(),
        }
    }
}

impl eframe::App for Frontend {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.backend.remove_deleted();

        egui::CentralPanel::default().show(&ctx, |ui| {
            self.build_top_menu(ui);

            self.build_quest_editor(ui, ctx);
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../Nunito-Black.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
