use crate::backend::Backend;
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps};
use crate::backend::entity_impl::item::armor::ArmorAction;
use crate::backend::holder::{DataHolder, HolderMapOps, HolderOps};
use crate::entity::GameEntityT;
use crate::entity::item::armor::{
    Armor, ArmorMeshAdditionalF, ArmorMeshInfo, ArmorMeshes, CurrentArmorMesh,
};
use crate::frontend::entity_impl::EntityInfoState;
use crate::frontend::util::{
    Draw, DrawActioned, DrawCtx, DrawUtils, bool_row, close_entity_button, combo_box_row,
    format_button_text, num_row, num_row_optional, text_row_c,
};
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Button, Color32, Context, Response, ScrollArea, Stroke, Ui};
use std::sync::RwLock;

impl DrawEntity<ArmorAction, ()> for Armor {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<ArmorAction>,
        holders: &mut DataHolder,
        _params: &mut (),
    ) {
        let init_rect = ui.min_size();

        ui.horizontal(|ui| {
            self.base_info.draw_ctx(ui, ctx, holders, init_rect);

            ui.vertical(|ui| {
                ui.set_width(400.);

                combo_box_row(ui, &mut self.armor_type, "Armor Type");
                text_row_c(ui, &mut self.attack_effect, "Effect");

                ui.horizontal(|ui| {
                    num_row(ui, &mut self.unk1, "Unk1");
                    bool_row(ui, &mut self.unk2, "Unk2");
                });

                num_row(ui, &mut self.mp_bonus, "Mp Bonus");
                num_row(ui, &mut self.hide_mask, "Hide Mask");

                num_row_optional(
                    ui,
                    &mut self.set_enchant_effect_id.0,
                    "Enchanted Set Effect Id",
                    "",
                    u8::MAX,
                );

                combo_box_row(ui, &mut self.underwater_body_type1, "Underwater 1");
                combo_box_row(ui, &mut self.underwater_body_type2, "Underwater 2");

                ui.scope(|ui| {
                    ui.set_height(160.);

                    self.item_sound.draw_vertical(
                        ui,
                        "Sounds",
                        |v| {
                            *action.write().unwrap() = ArmorAction::RemoveSound(v);
                        },
                        holders,
                        true,
                        true,
                    );
                });

                ui.add_space(10.);

                self.mesh_info.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Mesh Params   ",
                    &format!("Mesh Params {}", self.base_info.name.ru),
                    &format!("{} armor_mesh_params", self.base_info.id.0),
                    init_rect,
                );

                ui.separator();
            });
        });

        ui.separator();
    }
}

impl DrawActioned<(), CurrentArmorMesh> for ArmorMeshes {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        _action: &RwLock<()>,
        params: &mut CurrentArmorMesh,
    ) {
        combo_box_row(ui, params, "");
        ui.separator();
        self[*params].draw(ui, holders);
    }
}

impl Draw for ArmorMeshInfo {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(300.);
                self.base.unk1.draw_vertical_nc(ui, "Meshes", holders);
                ui.add_space(10.);
                self.base.unk2.draw_vertical_nc(ui, "Textures", holders);
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(400.);
                self.additional
                    .unk1
                    .draw_vertical_nc(ui, "Additional Meshes", holders);
                self.additional
                    .unk5
                    .draw_vertical_nc(ui, "Additional Textures", holders);
                text_row_c(ui, &mut self.additional.unk6, "Unk6");
            });
        })
        .response
    }
}

impl Draw for ArmorMeshAdditionalF {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        text_row_c(ui, &mut self.unk2, "");

        ui.horizontal(|ui| {
            num_row(ui, &mut self.unk3, "Unk3");
            num_row(ui, &mut self.unk4, "Unk4");
        })
        .response
    }
}

impl Frontend {
    pub fn draw_armor_tabs(&mut self, ui: &mut Ui) {
        for (i, (title, id, is_changed)) in self
            .backend
            .editors
            .get_opened_armor_info()
            .iter()
            .enumerate()
        {
            let mut button = Button::new(format_button_text(&format!(
                "{}[{}] {}",
                if *is_changed { "*" } else { "" },
                id.0,
                title
            )))
            .fill(Color32::from_rgb(77, 47, 99))
            .min_size([150., 10.].into());

            let is_current = CurrentEntity::Armor(i) == self.backend.editors.current_entity;

            if is_current {
                button = button.stroke(Stroke::new(1.0, Color32::LIGHT_GRAY));
            }

            if ui
                .add(button)
                .on_hover_text(format!(
                    "Armor: [{}] {}{}",
                    id.0,
                    title,
                    if *is_changed { "\nModified!" } else { "" },
                ))
                .clicked()
                && !self.backend.dialog_showing
            {
                self.backend.editors.set_current_armor(i);
            }

            close_entity_button(ui, CurrentEntity::Armor(i), &mut self.backend, *is_changed);

            ui.separator();
        }
    }

    pub(crate) fn draw_armor_selector(backend: &mut Backend, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);

            let holder = &mut backend.holders.game_data_holder.armor_holder;
            let item_holder = &mut backend.holders.game_data_holder.item_holder;
            let catalog = &mut backend.entity_catalogs.armor;
            let filter_mode = &mut backend.entity_catalogs.filter_mode;
            let edit_params = &mut backend.editors;

            if catalog
                .draw_search_and_add_buttons(ui, holder, filter_mode, catalog.len())
                .clicked()
            {
                edit_params.create_new_armor();
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
                            .armor
                            .opened
                            .iter()
                            .enumerate()
                            .find(|(_, v)| v.inner.initial_id == q.id)
                        {
                            has_unsaved_changes = v.is_changed();

                            if edit_params.current_entity == CurrentEntity::Armor(ind) {
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
                                    edit_params.close_if_opened(GameEntityT::Armor(q.id));
                                } else {
                                    edit_params.open_armor(q.id, holder);
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
                        item_holder.remove(&id);
                        edit_params.close_if_opened(GameEntityT::Armor(id));
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
