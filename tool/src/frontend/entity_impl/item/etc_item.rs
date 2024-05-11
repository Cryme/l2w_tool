use crate::backend::entity_editor::CurrentEntity;
use crate::backend::entity_impl::item::etc_item::EtcItemAction;
use crate::backend::holder::DataHolder;
use crate::backend::Backend;
use crate::entity::item::etc_item::{EtcItem, EtcMeshInfo};
use crate::entity::EntityT;
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::{
    close_entity_button, combo_box_row, format_button_text, text_row, Draw, DrawCtx, DrawUtils,
};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, Response, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawEntity<EtcItemAction, ()> for EtcItem {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<EtcItemAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        let init_rect = ui.min_size();

        ui.horizontal(|ui| {
            self.base_info.draw_ctx(ui, ctx, holders, init_rect);

            ui.vertical(|ui| {
                ui.set_width(400.);

                combo_box_row(ui, &mut self.etc_item_type, "Etc Type");
                combo_box_row(ui, &mut self.consume_type, "Consume Type");

                self.mesh_info.draw_vertical(
                    ui,
                    "Mesh",
                    |v| {
                        *action.write().unwrap() = EtcItemAction::RemoveMesh(v);
                    },
                    holders,
                    true,
                    true,
                );
            });

            ui.separator();
        });

        ui.separator();
    }
}

impl Draw for EtcMeshInfo {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.vertical(|ui| {
            text_row(ui, &mut self.mesh, "Mesh");
            text_row(ui, &mut self.texture, "Texture")
        })
        .inner
    }
}

impl Frontend {
    pub fn draw_etc_items_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .edit_params
            .get_opened_etc_items_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(99, 85, 47))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::EtcItem(i) == self.backend.edit_params.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Etc: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.edit_params.set_current_etc_item(i);
            }

            close_entity_button(
                ui,
                CurrentEntity::EtcItem(i),
                &mut self.backend,
                *is_changed,
            );

            ui.separator();
        }
    }

    pub(crate) fn draw_etc_item_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.etc_item_holder;
            let item_holder = &mut backend.holders.game_data_holder.item_holder;
            let catalog = &mut backend.entity_catalogs.etc_item;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.edit_params;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_etc_item();
            }

            ui.separator();

            let mut changed = None;

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(ui, 36., catalog.catalog.len(), |ui, range| {
                    ui.set_width(width - 5.);

                    for v in range {
                        let q = &catalog.catalog[v];

                        let info_state = if let Some((ind, _)) = edit_params
                            .etc_items
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            if edit_params.current_entity == CurrentEntity::EtcItem(ind) {
                                EntityInfoState::Current
                            } else {
                                EntityInfoState::Opened
                            }
                        } else {
                            EntityInfoState::Nothing
                        };

                        ui.horizontal(|ui| {
                            if q.draw_catalog_buttons(ui, &mut changed, info_state)
                                .clicked()
                                && backend.dialog.is_none()
                                && !q.deleted
                            {
                                edit_params.open_etc_item(q.id, holder);
                            }
                        });
                    }
                });
            });

            if let Some(id) = changed {
                if let Some(v) = holder.inner.get_mut(&id) {
                    v._deleted = !v._deleted;

                    if v._deleted {
                        item_holder.remove(&id);
                        edit_params.close_if_opened(EntityT::EtcItem(id));
                        holder.inc_deleted();
                    } else {
                        item_holder.insert(id, (&(*v)).into());
                        holder.dec_deleted();
                    }

                    catalog.filter(holder, *filter_mode);

                    backend.check_for_unwrote_changed();
                }
            }
        });
    }
}
