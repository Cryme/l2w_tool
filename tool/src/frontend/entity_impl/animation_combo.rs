use crate::backend::Backend;
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::entity::GameEntityT;
use crate::entity::animation_combo::AnimationCombo;
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::{
    DrawAsTooltip, bool_row, close_entity_button, format_button_text, text_row, text_row_c,
};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawEntity<(), ()> for AnimationCombo {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        _ctx: &Context,
        _action: &RwLock<()>,
        _holders: &mut DataHolder,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_height(400.);

            ui.vertical(|ui| {
                ui.set_width(200.);
                text_row_c(ui, &mut self.name, "Name");

                text_row(ui, &mut self.anim_0, "Anim 1");
                text_row(ui, &mut self.anim_1, "Anim 2");
                text_row(ui, &mut self.anim_2, "Anim 3");

                bool_row(ui, &mut self.loop_p, "Loop");
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl Frontend {
    pub fn draw_animation_combo_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .editors
            .get_opened_animation_combo_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(54, 67, 105))
            .min_size([150., 10.].into());

            let is_current =
                CurrentEntity::AnimationCombo(i) == self.backend.editors.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "AnimationCombo: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.editors.set_current_animation_combo(i);
            }

            close_entity_button(
                ui,
                CurrentEntity::AnimationCombo(i),
                &mut self.backend,
                *is_changed,
            );

            ui.separator();
        }
    }

    pub(crate) fn draw_animation_combo_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.animation_combo_holder;
            let catalog = &mut backend.entity_catalogs.animation_combo;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.editors;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_animation_combo();
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
                            .animation_combo
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::AnimationCombo(ind) {
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
                                    edit_params.close_if_opened(GameEntityT::AnimationCombo(q.id));
                                } else {
                                    edit_params.open_animation_combo(q.id, holder);
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
                        edit_params.close_if_opened(GameEntityT::AnimationCombo(id));
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

impl DrawAsTooltip for AnimationCombo {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("Animation Combo [{}]", self.name));
    }
}
