use crate::backend::weapon::WeaponAction;
use crate::backend::{Backend, Holders};
use crate::entity::item::weapon::Weapon;
use crate::frontend::{DrawEntity, Frontend};
use eframe::egui::{Context, Key, ScrollArea, Ui};
use std::sync::RwLock;

impl DrawEntity<WeaponAction, ()> for Weapon {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        action: &RwLock<WeaponAction>,
        holders: &mut Holders,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {});
    }
}

impl Frontend {
    pub(crate) fn draw_weapon_selector(
        backend: &mut Backend,
        ui: &mut Ui,
        max_height: f32,
        width: f32,
    ) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_max_height(max_height);

            if ui.button("    New Weapon    ").clicked() && !backend.dialog_showing {
                // backend.edit_params.create_new_quest(); TODO:
            }

            ui.horizontal(|ui| {
                let l = ui.text_edit_singleline(&mut backend.filter_params.weapon_filter_string);
                if ui.button("🔍").clicked()
                    || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
                {
                    backend.filter_weapons();
                }
            });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show_rows(
                    ui,
                    20.,
                    backend.filter_params.weapon_catalog.len(),
                    |ui, range| {
                        ui.set_width(width - 5.);

                        for i in range {
                            let q = &backend.filter_params.weapon_catalog[i];

                            if ui.button(format!("ID: {}\n{}", q.id.0, q.name)).clicked()
                                && !backend.dialog_showing
                            {
                                backend.edit_params.open_weapon(
                                    q.id,
                                    &mut backend.holders.game_data_holder.weapon_holder,
                                );
                            }
                        }
                    },
                );
            });
        });
    }
}
