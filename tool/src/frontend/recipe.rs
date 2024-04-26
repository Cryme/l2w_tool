use crate::backend::recipe::RecipeAction;
use crate::backend::{Backend, CurrentOpenedEntity, Holders};
use crate::entity::recipe::{Recipe, RecipeMaterial};
use crate::frontend::util::{bool_row, num_row, text_row, Draw, DrawAsTooltip, DrawUtils};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, Key, Response, ScrollArea, Ui};
use std::sync::RwLock;

impl DrawEntity<RecipeAction, ()> for Recipe {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        _ctx: &Context,
        action: &RwLock<RecipeAction>,
        holders: &mut Holders,
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
    fn draw(&mut self, ui: &mut Ui, holders: &Holders) -> Response {
        num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
            holders
                .game_data_holder
                .item_holder
                .get(&self.id)
                .draw_as_tooltip(ui);
        });

        num_row(ui, &mut self.count, "Count");
        num_row(ui, &mut self.unk, "Unk")
    }
}

impl Frontend {
    pub fn draw_recipe_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id)) in self
            .backend
            .edit_params
            .get_opened_recipes_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format!("{} [{}]", title, id.0));

            let is_current =
                CurrentOpenedEntity::Recipe(i) == self.backend.edit_params.current_opened_entity;

            if is_current {
                button = button.fill(Color32::from_rgb(42, 70, 83));
            }

            if ui.add(button).clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.set_current_recipe(i);
            }

            if is_current && ui.button("Save").clicked() {
                self.backend.save_current_entity();
            }

            if ui.button("❌").clicked() && !self.backend.dialog_showing {
                self.backend.edit_params.close_recipe(i);
            }

            ui.separator();
        }
    }

    pub(crate) fn draw_recipe_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Set    ").clicked() && backend.dialog.is_none() {
                backend.edit_params.create_new_recipe();
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.recipe_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_recipes();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.recipe_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for i in range {
                            let q = &backend.filter_params.recipe_catalog[i];

                            if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                                && backend.dialog.is_none()
                            {
                                backend.edit_params.open_recipe(
                                    q.id,
                                    &mut backend.holders.game_data_holder.recipe_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}

impl DrawAsTooltip for Recipe {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("ID: {}\n{}", self.id.0, self.name));
    }
}
