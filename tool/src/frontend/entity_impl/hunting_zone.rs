use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, WindowParams};
use crate::backend::entity_impl::hunting_zone::{HuntingZoneAction, MapObjectAction};
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::backend::Backend;
use crate::entity::hunting_zone::{HuntingZone, MapObject};
use crate::entity::GameEntityT;
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::{
    close_entity_button, combo_box_row, format_button_text, num_row, num_row_2d, num_row_optional,
    text_row, text_row_c, text_row_multiline, Draw, DrawActioned, DrawAsTooltip, DrawUtils,
};
use crate::frontend::{DrawEntity, Frontend, ADD_ICON, DELETE_ICON};
use eframe::egui::{Button, Color32, Context, DragValue, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawActioned<MapObjectAction, ()> for MapObject {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<MapObjectAction>,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                text_row_c(ui, &mut self.icon_texture, "Icon");
                text_row_c(ui, &mut self.icon_texture_over, "Icon Over");
                text_row_c(ui, &mut self.icon_texture_pressed, "Icon Pressed");

                num_row_2d(ui, &mut self.world_pos, "Icon World Position");
                num_row_2d(ui, &mut self.size, "Icon Size");
                num_row_2d(ui, &mut self.desc_offset, "Desc Offset");

                text_row_c(ui, &mut self.desc_font_name, "Desc Font Name");
            });

            ui.separator();

            self.unk1.draw_vertical(
                ui,
                "Unk1",
                |v| *action.write().unwrap() = MapObjectAction::RemoveUnk1(v),
                holders,
                true,
                false,
            );
        });
    }
}

impl DrawEntity<HuntingZoneAction, ()> for HuntingZone {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<HuntingZoneAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        let init_rect = ui.min_size();

        ui.horizontal(|ui| {
            ui.set_height(400.);

            ui.vertical(|ui| {
                ui.set_width(300.);

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.name, "Name");
                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .hunting_zone_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });
                });
                text_row_multiline(ui, &mut self.desc, "Description");

                ui.separator();

                combo_box_row(ui, &mut self.zone_type, "Zone Type");

                ui.horizontal(|ui| {
                    ui.label("Recommended Level Range");
                    ui.add(DragValue::new(&mut self.lvl_min));
                    ui.add(DragValue::new(&mut self.lvl_max));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Star Npc Loc");
                    self.start_npc_loc.draw(ui, holders);
                });

                num_row_optional(ui, &mut self.npc_id.0, "Start Npc", "Id", 0).on_hover_ui(|ui| {
                    if self.npc_id.0 > 0 {
                        holders
                            .game_data_holder
                            .npc_holder
                            .get(&self.npc_id)
                            .draw_as_tooltip(ui)
                    }
                });

                ui.separator();

                num_row(ui, &mut self.second_id, "Secondary Id")
                    .on_hover_text("Used for linking with World Map Objects (minimapregion)");
                num_row(ui, &mut self.search_zone_id.0, "Region Id")
                    .on_hover_text("Used for search by region in map interface");
                num_row_optional(ui, &mut self.instant_zone_id.0, "Instant Zone", "Id", 0);
            });

            ui.separator();

            self.quests.draw_vertical(
                ui,
                "Quests",
                |v| *action.write().unwrap() = HuntingZoneAction::RemoveQuest(v),
                holders,
                true,
                false,
            );

            ui.separator();
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Map Objects");
                    if ui.button(ADD_ICON).clicked() {
                        self.world_map_objects.push(WindowParams::default())
                    }
                });

                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.world_map_objects.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                let t = format!("Objet {}", i);
                                v.draw_as_button(ui, ctx, holders, &t, &t, &t, init_rect);

                                if ui.button(DELETE_ICON).clicked() {
                                    *action.write().unwrap() =
                                        HuntingZoneAction::RemoveMapObject(i);
                                }
                            });
                        }
                    });
                });
            });
            ui.separator();
        });

        ui.separator();
    }
}

impl Frontend {
    pub fn draw_hunting_zone_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .editors
            .get_opened_hunting_zone_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(47, 99, 74))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::HuntingZone(i) == self.backend.editors.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Hunting Zone: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.editors.set_current_hunting_zone(i);
            }

            close_entity_button(
                ui,
                CurrentEntity::HuntingZone(i),
                &mut self.backend,
                *is_changed,
            );

            ui.separator();
        }
    }

    pub(crate) fn draw_hunting_zone_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.hunting_zone_holder;
            let catalog = &mut backend.entity_catalogs.hunting_zone;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.editors;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_hunting_zone();
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
                            .hunting_zones
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::HuntingZone(ind) {
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
                                    edit_params.close_if_opened(GameEntityT::HuntingZone(q.id));
                                } else {
                                    edit_params.open_hunting_zone(q.id, holder);
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
                        edit_params.close_if_opened(GameEntityT::HuntingZone(id));
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

impl DrawAsTooltip for HuntingZone {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(format!("[{}]\n {}", self.id.0, self.name));

            if !self.desc.is_empty() {
                ui.label(self.desc.to_string());
            }
        });
    }
}
