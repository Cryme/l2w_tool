use crate::backend::entity_editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::entity_impl::daily_missions::DailyMissionAction;
use crate::backend::holder::{DataHolder, HolderMapOps};
use crate::backend::Backend;
use crate::data::PlayerClass;
use crate::entity::daily_mission::{DailyMission, DailyMissionReward, DailyMissionUnk7};
use crate::entity::EntityT;
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::num_value::NumberValue;
use crate::frontend::util::{
    close_entity_button, combo_box_row, format_button_text, num_row, text_row, text_row_multiline,
    Draw, DrawAsTooltip, DrawUtils,
};
use crate::frontend::{DrawEntity, Frontend, DELETE_ICON};
use eframe::egui::{Button, Color32, Context, Response, ScrollArea, Stroke, Ui};
use std::sync::RwLock;
use strum::IntoEnumIterator;

impl DrawEntity<DailyMissionAction, ()> for DailyMission {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        _ctx: &Context,
        action: &RwLock<DailyMissionAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_height(400.);

            ui.vertical(|ui| {
                ui.set_width(210.);

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.name, "Name");
                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .daily_mission_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });
                });

                num_row(ui, &mut self.reward_id, "Unk Id");
                text_row_multiline(ui, &mut self.desc, "Description");
                text_row_multiline(ui, &mut self.category, "Category");

                combo_box_row(ui, &mut self.repeat_type, "Repeat Type");
                num_row(ui, &mut self.unk2, "Type 2");
                num_row(ui, &mut self.unk3, "Unk");
                num_row(ui, &mut self.unk4, "Unk");
                num_row(ui, &mut self.unk5, "Unk");
                num_row(ui, &mut self.unk6, "Unk");
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

                self.unk7.draw_vertical(
                    ui,
                    "Unk7",
                    |v| *action.write().unwrap() = DailyMissionAction::RemoveUnk7(v),
                    holders,
                    true,
                    true,
                )
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(200.);

                self.rewards.draw_vertical(
                    ui,
                    "Rewards",
                    |v| *action.write().unwrap() = DailyMissionAction::RemoveReward(v),
                    holders,
                    true,
                    true,
                )
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(150.);
                self.unk8.draw_vertical_nc(ui, "Reset Info", holders);
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl Draw for DailyMissionReward {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        ui.horizontal(|ui| {
            ui.label("ID");

            ui.add(NumberValue::new(&mut self.item_id.0))
                .on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&self.item_id)
                        .draw_as_tooltip(ui)
                });

            ui.label("Count");
            ui.add(NumberValue::new(&mut self.count));
        })
        .response
    }
}

impl Draw for DailyMissionUnk7 {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                num_row(ui, &mut self.unk1, "Unk1");
                num_row(ui, &mut self.unk2, "Unk2");
            });

            ui.horizontal(|ui| {
                num_row(ui, &mut self.unk3, "Unk3");
                num_row(ui, &mut self.unk4, "Unk4");
            });
        })
        .response
    }
}

impl Frontend {
    pub fn draw_daily_missions_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .edit_params
            .get_opened_daily_missions_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(87, 47, 99))
            .min_size([150., 10.].into());

            let is_current =
                CurrentEntity::DailyMission(i) == self.backend.edit_params.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "DailyMission(): [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_daily_mission(i);
            }

            close_entity_button(
                ui,
                CurrentEntity::DailyMission(i),
                &mut self.backend,
                *is_changed,
            );

            ui.separator();
        }
    }

    pub(crate) fn draw_daily_missions_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.daily_mission_holder;
            let catalog = &mut backend.entity_catalogs.daily_mission;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.edit_params;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_daily_mission();
            }

            ui.separator();

            let mut deleted_status = None;

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(ui, 36., catalog.catalog.len(), |ui, range| {
                    ui.set_width(width - 5.);

                    for v in range {
                        let q = &catalog.catalog[v];

                        let mut has_unsaved_changes = false;

                        let info_state = if let Some((ind, v)) = edit_params
                            .daily_mission
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.inner.id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::DailyMission(ind) {
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
                                &mut deleted_status,
                                info_state,
                                has_unsaved_changes,
                            )
                            .clicked()
                                && backend.dialog.is_none()
                                && !q.deleted
                            {
                                if ui.input(|i| i.modifiers.ctrl) && !has_unsaved_changes {
                                    edit_params.close_if_opened(EntityT::DailyMission(q.id));
                                } else {
                                    edit_params.open_daily_mission(q.id, holder);
                                }
                            }
                        });
                    }
                });
            });

            if let Some(id) = deleted_status {
                if let Some(v) = holder.get_mut(&id) {
                    v._deleted = !v._deleted;

                    if v._deleted {
                        edit_params.close_if_opened(EntityT::DailyMission(id));
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

impl DrawAsTooltip for DailyMission {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("[{}] {}", self.id.0, self.name));
    }
}
