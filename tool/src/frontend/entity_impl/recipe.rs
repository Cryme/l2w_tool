use crate::backend::editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::entity_impl::recipe::RecipeAction;
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::backend::Backend;
use crate::entity::recipe::{Recipe, RecipeMaterial};
use crate::entity::GameEntityT;
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::{
    bool_row, close_entity_button, format_button_text, num_row, num_row_optional, text_row, Draw,
    DrawAsTooltip, DrawUtils,
};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, Response, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawEntity<RecipeAction, ()> for Recipe {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        _ctx: &Context,
        action: &RwLock<RecipeAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_height(400.);

            ui.vertical(|ui| {
                ui.set_width(300.);

                ui.horizontal(|ui| {
                    text_row(ui, &mut self.name, "Name");
                    num_row(ui, &mut self.level, "Lvl");
                    num_row(ui, &mut self.id.0, "Id");
                });

                num_row(ui, &mut self.recipe_item.0, "Recipe Item Id").on_hover_ui(|ui| {
                    holders
                        .game_data_holder
                        .item_holder
                        .get(&self.recipe_item)
                        .draw_as_tooltip(ui);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    num_row(ui, &mut self.product.0, "Product Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .item_holder
                            .get(&self.product)
                            .draw_as_tooltip(ui);
                    });
                    num_row(ui, &mut self.product_count, "Count");
                    bool_row(ui, &mut self.is_multiple_product, "Multiple");
                });

                bool_row(ui, &mut self.show_tree, "Show Tree");
                num_row(ui, &mut self.mp_consume, "Mp Consume");
                num_row(ui, &mut self.success_rate, "Success Rate");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(300.);

                self.materials.draw_vertical(
                    ui,
                    "Ingredients",
                    |v| *action.write().unwrap() = RecipeAction::DeleteIngredient(v),
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

impl Draw for RecipeMaterial {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
            holders
                .game_data_holder
                .item_holder
                .get(&self.id)
                .draw_as_tooltip(ui);
        });

        num_row(ui, &mut self.count, "Count");
        num_row_optional(ui, &mut self.recipe_id.0, "Recipe", " Id", 0).on_hover_ui(|ui| {
            if self.recipe_id.0 > 0 {
                holders
                    .game_data_holder
                    .recipe_holder
                    .get(&self.recipe_id)
                    .draw_as_tooltip(ui);
            }
        })
    }
}

impl Frontend {
    pub fn draw_recipe_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .editors
            .get_opened_recipes_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(50, 99, 47))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::Recipe(i) == self.backend.editors.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Recipe: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.editors.set_current_recipe(i);
            }

            close_entity_button(ui, CurrentEntity::Recipe(i), &mut self.backend, *is_changed);

            ui.separator();
        }
    }

    pub(crate) fn draw_recipe_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.recipe_holder;
            let catalog = &mut backend.entity_catalogs.recipe;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.editors;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_recipe();
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
                            .recipes
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::Recipe(ind) {
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
                                    edit_params.close_if_opened(GameEntityT::Recipe(q.id));
                                } else {
                                    edit_params.open_recipe(q.id, holder);
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
                        edit_params.close_if_opened(GameEntityT::Recipe(id));
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

impl DrawAsTooltip for Recipe {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("ID: {}\n{}", self.id.0, self.name));
    }
}
