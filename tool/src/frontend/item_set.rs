use crate::backend::item_set::ItemSetAction;
use crate::backend::{Backend, CurrentEntity};
use crate::entity::item_set::{ItemSet, ItemSetEnchantInfo};
use crate::entity::CommonEntity;
use crate::frontend::util::{format_button_text, num_row, DrawAsTooltip};
use crate::frontend::{DrawEntity, Frontend, ADD_ICON, DELETE_ICON};
use crate::holder::DataHolder;
use eframe::egui;
use eframe::egui::{Button, Color32, Context, Key, ScrollArea, Stroke, TextEdit, Ui, Widget};
use std::sync::RwLock;

impl DrawEntity<ItemSetAction, ()> for ItemSet {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        _ctx: &Context,
        action: &RwLock<ItemSetAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_height(400.);

            ui.vertical(|ui| {
                ui.set_width(300.);
                ui.horizontal(|ui| {
                    ui.label(format!("Base Parts[{}]", self.base_items.len()));

                    if ui.button(ADD_ICON).clicked() {
                        *action.write().unwrap() = ItemSetAction::AddBaseSetLevel;
                    }
                });

                ui.add_space(6.0);

                ui.push_id(ui.next_auto_id(), |ui| {
                    ui.set_max_height(200.);
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.base_descriptions.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "{} Item{}",
                                    i + 1,
                                    if i == 0 { "" } else { "s" }
                                ));

                                if ui.button(DELETE_ICON).clicked() {
                                    *action.write().unwrap() = ItemSetAction::RemoveBaseSetLevel(i);
                                }
                            });

                            TextEdit::multiline(v).desired_rows(1).ui(ui);

                            ui.add_space(6.0);
                        }
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(format!("Item Groups[{}]", self.base_items.len()));

                    if ui.button(ADD_ICON).clicked() {
                        *action.write().unwrap() = ItemSetAction::AddBaseItemGroup;
                    }
                });

                ui.add_space(6.);

                ui.push_id(ui.next_auto_id(), |ui| {
                    ui.set_width(300.);
                    ui.set_max_height(150.);
                    ScrollArea::vertical().show(ui, |ui| {
                        for v in self.base_items.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("Group {}", v.0 + 1,));

                                if ui.button(DELETE_ICON).clicked() {
                                    *action.write().unwrap() =
                                        ItemSetAction::RemoveBaseItemGroup(v.0);
                                }
                            });
                            ui.push_id(ui.next_auto_id(), |ui| {
                                ScrollArea::horizontal().show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        for vv in v.1.iter_mut().enumerate() {
                                            ui.add(egui::DragValue::new(&mut vv.1 .0)).on_hover_ui(
                                                |ui| {
                                                    holders
                                                        .game_data_holder
                                                        .item_holder
                                                        .get(vv.1)
                                                        .draw_as_tooltip(ui);
                                                },
                                            );

                                            if ui.button(DELETE_ICON).clicked() {
                                                *action.write().unwrap() =
                                                    ItemSetAction::RemoveBaseGroupItem(v.0, vv.0);
                                            }
                                        }
                                        if ui.button(ADD_ICON).clicked() {
                                            *action.write().unwrap() =
                                                ItemSetAction::AddBaseGroupItem(v.0);
                                        }
                                    });

                                    ui.separator();
                                });
                            });
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(300.);

                ui.horizontal(|ui| {
                    ui.label(format!("Additional Parts[{}]", self.additional_items.len()));

                    if ui.button(ADD_ICON).clicked() {
                        *action.write().unwrap() = ItemSetAction::AddAdditionalSetLevel;
                    }
                });

                ui.add_space(6.0);

                ui.push_id(ui.next_auto_id(), |ui| {
                    ui.set_max_height(200.);
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.additional_descriptions.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!(
                                    "{} Item{}",
                                    i + 1,
                                    if i == 0 { "" } else { "s" }
                                ));

                                if ui.button(DELETE_ICON).clicked() {
                                    *action.write().unwrap() =
                                        ItemSetAction::RemoveAdditionalSetLevel(i);
                                }
                            });

                            TextEdit::multiline(v).desired_rows(1).ui(ui);

                            ui.add_space(6.0);
                        }
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(format!("Item Groups[{}]", self.additional_items.len()));

                    if ui.button(ADD_ICON).clicked() {
                        *action.write().unwrap() = ItemSetAction::AddAdditionalItemGroup;
                    }
                });

                ui.add_space(6.);

                ui.push_id(ui.next_auto_id(), |ui| {
                    ui.set_width(300.);
                    ui.set_max_height(150.);

                    ScrollArea::vertical().show(ui, |ui| {
                        for v in self.additional_items.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("Group {}", v.0 + 1,));

                                if ui.button(DELETE_ICON).clicked() {
                                    *action.write().unwrap() =
                                        ItemSetAction::RemoveAdditionalItemGroup(v.0);
                                }
                            });
                            ui.push_id(ui.next_auto_id(), |ui| {
                                ScrollArea::horizontal().show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        for vv in v.1.iter_mut().enumerate() {
                                            ui.add(egui::DragValue::new(&mut vv.1 .0)).on_hover_ui(
                                                |ui| {
                                                    holders
                                                        .game_data_holder
                                                        .item_holder
                                                        .get(vv.1)
                                                        .draw_as_tooltip(ui);
                                                },
                                            );

                                            if ui.button(DELETE_ICON).clicked() {
                                                *action.write().unwrap() =
                                                    ItemSetAction::RemoveAdditionalGroupItem(
                                                        v.0, vv.0,
                                                    );
                                            }
                                        }
                                        if ui.button(ADD_ICON).clicked() {
                                            *action.write().unwrap() =
                                                ItemSetAction::AddAdditionalGroupItem(v.0);
                                        }
                                    });

                                    ui.separator();
                                });
                            });
                        }
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(300.);

                num_row(ui, &mut self.unk1, "Unk1");
                num_row(ui, &mut self.unk1, "Unk2");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Enchant Bonuses");

                    if ui.button(ADD_ICON).clicked() {
                        self.enchant_info.push(ItemSetEnchantInfo {
                            enchant_level: if let Some(v) = self.enchant_info.last() {
                                v.enchant_level + 1
                            } else {
                                6
                            },
                            enchant_description: "New Description".to_string(),
                        })
                    }
                    if ui.button("-").clicked() {
                        self.enchant_info.pop();
                    }
                });

                for v in &mut self.enchant_info {
                    num_row(ui, &mut v.enchant_level, "+");
                    ui.text_edit_multiline(&mut v.enchant_description);
                }
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl Frontend {
    pub fn draw_item_set_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id)) in self
            .backend
            .edit_params
            .get_opened_item_sets_info()
            .iter()
            .enumerate()
        {
            let label = format!("[{}] {}", id.0, title);

            let mut button = Button::new(format_button_text(&label))
                .fill(Color32::from_rgb(99, 47, 88))
                .min_size([150., 10.].into());

            let is_current = CurrentEntity::ItemSet(i) == self.backend.edit_params.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!("Set: {label}"))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_item_set(i);
            }

            if ui.button("❌").clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.close_item_set(i);
            }

            ui.separator();
        }
    }

    pub(crate) fn draw_item_set_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Set    ").clicked() && backend.dialog.is_none() {
                backend.edit_params.create_new_item_set();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.item_set_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_item_sets();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.item_set_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for i in range {
                            let q = &backend.filter_params.item_set_catalog[i];

                            if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                                && backend.dialog.is_none()
                            {
                                backend.edit_params.open_item_set(
                                    q.id,
                                    &mut backend.holders.game_data_holder.item_set_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}

impl DrawAsTooltip for ItemSet {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("ID: {}", self.id.0));

        ui.label(self.desc());
    }
}
