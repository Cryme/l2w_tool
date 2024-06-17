use crate::backend::entity_editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::backend::Backend;
use crate::entity::region::{MapInfo, Region};
use crate::entity::EntityT;
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::{
    close_entity_button, combo_box_row, format_button_text, num_row, num_row_2d, text_row,
    DrawAsTooltip,
};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, DragValue, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawEntity<(), ()> for Region {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        _ctx: &Context,
        _action: &RwLock<()>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_height(400.);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    text_row(ui, &mut self.name, "Name");

                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .region_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });
                });

                num_row_2d(ui, &mut self.world_map_square, "Map Square");

                ui.horizontal(|ui| {
                    ui.label("Z min");
                    ui.add(DragValue::new(&mut self.z_range[1]));
                    ui.add_space(5.);
                    ui.label("Z max");
                    ui.add(DragValue::new(&mut self.z_range[0]));
                });

                num_row(ui, &mut self.color_code, "Color Code");
                combo_box_row(ui, &mut self.continent, "Continent");
                num_row(ui, &mut self.current_layer, "Current Layer");
                num_row(ui, &mut self.total_layers, "Total Layers");
            });

            ui.separator();

            ui.vertical(|ui| {
                if ui
                    .checkbox(&mut self.map_info.is_some(), "Map Info")
                    .changed()
                {
                    if self.map_info.is_some() {
                        self.map_info = None;
                    } else {
                        self.map_info = Some(MapInfo::default())
                    }
                }

                if let Some(v) = &mut self.map_info {
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut v.button_pos.is_some(), "Button").changed() {
                            if v.button_pos.is_some() {
                                v.button_pos = None;
                            } else {
                                v.button_pos = Some([0, 0]);
                            }
                        }

                        if let Some(vv) = &mut v.button_pos {
                            num_row_2d(ui, vv, "Position");
                        }
                    });

                    num_row_2d(ui, &mut v.pos, "Position");
                    num_row_2d(ui, &mut v.size, "Size");
                    num_row_2d(ui, &mut v.center, "Center");

                    num_row(ui, &mut v.scale, "Scale");
                    text_row(ui, &mut v.texture, "Texture");
                }
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl Frontend {
    pub fn draw_region_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .edit_params
            .get_opened_region_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(99, 73, 47))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::Region(i) == self.backend.edit_params.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Region: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_region(i);
            }

            close_entity_button(ui, CurrentEntity::Region(i), &mut self.backend, *is_changed);

            ui.separator();
        }
    }

    pub(crate) fn draw_region_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.region_holder;
            let catalog = &mut backend.entity_catalogs.region;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.edit_params;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_region();
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
                            .regions
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::Region(ind) {
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
                                    edit_params.close_if_opened(EntityT::Region(q.id));
                                } else {
                                    edit_params.open_region(q.id, holder);
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
                        edit_params.close_if_opened(EntityT::Region(id));
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

impl DrawAsTooltip for Region {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("[{}]\n {}", self.id.0, self.name));
    }
}
