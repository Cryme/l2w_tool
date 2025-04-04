use crate::backend::Backend;
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::entity_impl::item_set::ItemSetAction;
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::entity::item_set::{ItemSet, ItemSetEnchantInfo};
use crate::entity::{CommonEntity, GameEntityT};
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::num_value::NumberValue;
use crate::frontend::util::{DrawAsTooltip, close_entity_button, format_button_text, num_row};
use crate::frontend::{ADD_ICON, DELETE_ICON, DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, ScrollArea, Stroke, TextEdit, Ui, Widget};
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
                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .item_set_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });

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
                                            ui.add(NumberValue::new(&mut vv.1.0)).on_hover_ui(
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
                                            ui.add(NumberValue::new(&mut vv.1.0)).on_hover_ui(
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
        for (i, (title, id, is_changed)) in self
            .backend
            .editors
            .get_opened_item_sets_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] Set",
                if *is_changed { "*" } else { "" },
                id.0,
            )))
            .fill(Color32::from_rgb(99, 47, 88))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::ItemSet(i) == self.backend.editors.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Set: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.editors.set_current_item_set(i);
            }

            close_entity_button(
                ui,
                CurrentEntity::ItemSet(i),
                &mut self.backend,
                *is_changed,
            );

            ui.separator();
        }
    }

    pub(crate) fn draw_item_set_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.item_set_holder;
            let catalog = &mut backend.entity_catalogs.item_set;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.editors;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_item_set();
            }

            ui.separator();

            let mut changed = None;

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(ui, 36., catalog.catalog.len(), |ui, range| {
                    ui.set_width(width - 5.);

                    let mut has_unsaved_changes = false;

                    for v in range {
                        let q = &catalog.catalog[v];

                        let info_state = if let Some((ind, v)) = edit_params
                            .item_sets
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::ItemSet(ind) {
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
                                    edit_params.close_if_opened(GameEntityT::ItemSet(q.id));
                                } else {
                                    edit_params.open_item_set(q.id, holder);
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
                        edit_params.close_if_opened(GameEntityT::ItemSet(id));
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

impl DrawAsTooltip for ItemSet {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("ID: {}", self.id.0));

        ui.label(self.desc());
    }
}
