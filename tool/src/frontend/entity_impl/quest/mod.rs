pub mod steps_editor;

use crate::backend::Backend;
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::entity_impl::quest::QuestAction;
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::common::{ItemId, NpcId, PlayerClass};
use crate::entity::quest::{GoalType, Quest, QuestReward, StepGoal};
use crate::entity::{CommonEntity, GameEntityT, GetEditParams};
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::entity_impl::quest::steps_editor::{QuestStepsEditorParams, draw_node_editor};
use crate::frontend::util::num_value::NumberValue;
use crate::frontend::util::{
    Draw, DrawUtils, close_entity_button, combo_box_row, format_button_text, num_row, text_row,
    text_row_multiline,
};
use crate::frontend::{DELETE_ICON, DrawAsTooltip, DrawEntity, Frontend};
use eframe::egui;
use eframe::egui::scroll_area::ScrollBarVisibility;
use eframe::egui::{Button, Color32, Context, Response, ScrollArea, Stroke, Ui, Vec2};
use num_traits::pow;
use std::sync::RwLock;
use strum::IntoEnumIterator;

impl GetEditParams<QuestStepsEditorParams> for Quest {
    fn edit_params(&self) -> QuestStepsEditorParams {
        QuestStepsEditorParams::default()
    }
}

impl DrawEntity<QuestAction, QuestStepsEditorParams> for Quest {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<QuestAction>,
        holders: &mut DataHolder,
        params: &mut QuestStepsEditorParams,
    ) {
        ui.horizontal(|ui| {
            ui.set_height(230.);

            ui.vertical(|ui| {
                ui.set_width(200.);

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.title[holders.localization], "Name");
                    ui.add_space(5.);
                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .quest_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });
                });

                text_row_multiline(ui, &mut self.intro[holders.localization], "Intro");
                text_row_multiline(
                    ui,
                    &mut self.requirements[holders.localization],
                    "Requirements",
                );
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(185.);

                num_row(ui, &mut self.priority_level, "Priority Level");
                combo_box_row(ui, &mut self.quest_type, "Quest Type");
                num_row(ui, &mut self.category_id, "Category ID");
                combo_box_row(ui, &mut self.category, "Category");
                combo_box_row(ui, &mut self.mark_type, "Mark Type");

                ui.horizontal(|ui| {
                    ui.set_height(20.);
                    ui.add(egui::Label::new("Min lvl"));
                    ui.add_space(2.);

                    if self.min_lvl > 0 {
                        if ui.checkbox(&mut true, "").changed() {
                            self.min_lvl = 0;
                        }

                        ui.add(NumberValue::new(&mut self.min_lvl));
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

                        ui.add(NumberValue::new(&mut self.max_lvl));
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

                        ui.add(NumberValue::new(&mut self.required_completed_quest_id.0))
                            .on_hover_ui(|ui| {
                                holders
                                    .game_data_holder
                                    .quest_holder
                                    .get(&self.required_completed_quest_id)
                                    .draw_as_tooltip(ui);
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

                        ui.add(NumberValue::new(&mut self.search_zone_id.0))
                            .on_hover_ui(|ui| {
                                holders
                                    .game_data_holder
                                    .hunting_zone_holder
                                    .get(&self.search_zone_id)
                                    .draw_as_tooltip(ui)
                            });
                    } else if ui.checkbox(&mut false, "").changed() {
                        self.search_zone_id.0 = 1;
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Edit JAVA Class").clicked() {
                        if self.java_class.is_none() {
                            holders.set_java_class(self);
                        }

                        if let Some(v) = &mut self.java_class {
                            v.opened = true;
                        }
                    }

                    if ui.button("Edit steps").clicked() {
                        params.show = true;
                    }
                });

                if let Some(class) = &mut self.java_class {
                    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(
                        ui.ctx(),
                        &ui.ctx().theme().default_style(),
                    );

                    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                        let mut layout_job = egui_extras::syntax_highlighting::highlight(
                            ui.ctx(),
                            &ui.ctx().theme().default_style(),
                            &theme,
                            string,
                            "java",
                        );

                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    egui::Window::new(format!("{} Java Class", self.title[holders.localization]))
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
                    false,
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
                false,
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
                false,
            );

            ui.separator();

            let w_id = 100 * self.id.0 + 1;

            let mut show = params.show;

            egui::Window::new(format!("Steps: {}", &self.name()))
                .id(egui::Id::new(w_id))
                .resizable(true)
                .constrain(true)
                .default_size(Vec2::new(700., 400.))
                .collapsible(true)
                .title_bar(true)
                .scroll(true)
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .drag_to_scroll(false)
                .enabled(true)
                .open(&mut show)
                .show(ctx, |ui| {
                    draw_node_editor(ui, &mut self.steps, holders, w_id, params, action);
                });

            if show != params.show {
                params.show = show;
            }
        });

        ui.separator();
    }
}

impl Quest {
    pub fn sort_steps(&mut self) {
        if !self.steps_sorted {
            const X_INCREMENT: f32 = 250.;
            const Y_INCREMENT: f32 = 100.;

            let x_s = -50.;

            let mut x_offset = 0.;
            let mut y_offset = 10.;

            let mut last_prev = 99_999_999;

            self.steps.sort_by(|a, b| {
                a.prev_steps
                    .iter()
                    .max()
                    .unwrap_or(&0)
                    .cmp(b.prev_steps.iter().max().unwrap_or(&0))
            });

            for (i, step) in self.steps.iter_mut().enumerate() {
                let prev = step.prev_steps.iter().max().unwrap_or(&999_999_999);

                if *prev == last_prev {
                    x_offset += X_INCREMENT;
                } else {
                    x_offset = 100. + pow(-1.0, i) * x_s;

                    y_offset += Y_INCREMENT;
                }

                step.pos.x = x_offset;
                step.pos.y = y_offset;

                last_prev = *prev;
            }

            self.steps_sorted = true;
        }
    }
}

impl DrawAsTooltip for Quest {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("[{}]\n{}", self.id.0, self.title.ru));
    }
}

impl Draw for StepGoal {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        combo_box_row(ui, &mut self.goal_type, "Type");

        let r = ui.horizontal(|ui| {
            match self.goal_type {
                GoalType::KillNpc => {
                    num_row(ui, &mut self.target_id, "Monster Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .npc_holder
                            .get(&NpcId(self.target_id))
                            .draw_as_tooltip(ui);
                    });
                }
                GoalType::CollectItem => {
                    num_row(ui, &mut self.target_id, "Item Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .item_holder
                            .get(&ItemId(self.target_id))
                            .draw_as_tooltip(ui);
                    });
                }
                GoalType::Other => {
                    num_row(ui, &mut self.target_id, "Npc String Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .npc_strings
                            .get(&self.target_id)
                            .draw_as_tooltip(ui);
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

impl Draw for QuestReward {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        ui.horizontal(|ui| {
            ui.label("ID");

            ui.add(NumberValue::new(&mut self.reward_id.0))
                .on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&self.reward_id)
                        .draw_as_tooltip(ui)
                });

            ui.label("Count");
            ui.add(NumberValue::new(&mut self.count));
        })
        .response
    }
}

impl Frontend {
    pub fn draw_quest_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .editors
            .get_opened_quests_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(47, 56, 99))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::Quest(i) == self.backend.editors.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Quest: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.editors.set_current_quest(i);
            }

            close_entity_button(ui, CurrentEntity::Quest(i), &mut self.backend, *is_changed);

            ui.separator();
        }
    }

    pub(crate) fn draw_quest_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.quest_holder;
            let catalog = &mut backend.entity_catalogs.quest;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.editors;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_quest();
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
                            .quests
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::Quest(ind) {
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
                                    edit_params.close_if_opened(GameEntityT::Quest(q.id));
                                } else {
                                    edit_params.open_quest(q.id, holder);
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
                        edit_params.close_if_opened(GameEntityT::Quest(id));
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
