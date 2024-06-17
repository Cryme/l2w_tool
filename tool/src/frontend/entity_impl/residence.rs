use crate::backend::entity_editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::entity_impl::residence::ResidenceAction;
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::backend::Backend;
use crate::entity::EntityT;
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::{close_entity_button, format_button_text, DrawAsTooltip, text_row, num_row, text_row_multiline};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, ScrollArea, Stroke, Ui};
use std::sync::RwLock;
use crate::entity::residence::Residence;

impl DrawEntity<ResidenceAction, ()> for Residence {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        _ctx: &Context,
        _action: &RwLock<ResidenceAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_height(400.);

            ui.vertical(|ui| {
                ui.set_width(300.);

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.name, "Name");
                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .residence_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui)
                    });
                });
                num_row(ui, &mut self.region_id, "Region Id");

                text_row_multiline(ui, &mut self.desc, "Description");
                text_row(ui, &mut self.territory, "Territory");

                text_row(ui, &mut self.merc_name, "Merchant Name");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(300.);

                text_row(ui, &mut self.mark, "Mark");
                text_row(ui, &mut self.mark_grey, "Mark Grey");
                text_row(ui, &mut self.flag_icon, "Flag");
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl Frontend {
    pub fn draw_residence_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .edit_params
            .get_opened_residences_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(64, 110, 79))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::Residence(i) == self.backend.edit_params.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Residence: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_residence(i);
            }

            close_entity_button(ui, CurrentEntity::Residence(i), &mut self.backend, *is_changed);

            ui.separator();
        }
    }

    pub(crate) fn draw_residence_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.residence_holder;
            let catalog = &mut backend.entity_catalogs.residence;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.edit_params;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_residence();
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
                            .residences
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::Residence(ind) {
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
                                    edit_params.close_if_opened(EntityT::Residence(q.id));
                                } else {
                                    edit_params.open_residence(q.id, holder);
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
                        edit_params.close_if_opened(EntityT::Residence(id));
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

impl DrawAsTooltip for Residence {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("ID: {}\n{}", self.id.0, self.name));
    }
}
