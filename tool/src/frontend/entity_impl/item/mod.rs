use crate::backend::entity_impl::item::{ItemAdditionalInfoAction, ItemDropInfoAction};
use crate::backend::holder::{DataHolder, HolderMapOps};
use crate::entity::item::{
    Item, ItemAdditionalInfo, ItemBaseInfo, ItemBattleStats, ItemDefaultAction, ItemDropInfo,
    ItemDropMeshInfo, ItemIcons,
};
use crate::frontend::ADD_ICON;
use crate::frontend::util::{
    Draw, DrawActioned, DrawAsTooltip, DrawCtx, DrawUtils, bool_row, combo_box_row, num_row,
    num_row_optional, text_row, text_row_c, text_row_multiline,
};
use eframe::egui;
use eframe::egui::{Context, Response, ScrollArea, TextWrapMode, Ui, Vec2};
use std::sync::RwLock;
use strum::IntoEnumIterator;

mod armor;
mod etc_item;
mod weapon;

impl DrawCtx for ItemBaseInfo {
    fn draw_ctx(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        holders: &mut DataHolder,
        init_rect: Vec2,
    ) -> Response {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(450.);
                ui.horizontal(|ui| {
                    text_row_c(ui, &mut self.name[holders.localization], "Name");

                    ui.add_space(5.);

                    num_row(ui, &mut self.id.0, "Id").on_hover_ui(|ui| {
                        holders
                            .game_data_holder
                            .item_holder
                            .get(&self.id)
                            .draw_as_tooltip(ui);
                    });
                });

                text_row(
                    ui,
                    &mut self.additional_name[holders.localization],
                    "Additional Name",
                );

                ui.horizontal(|ui| {
                    combo_box_row(ui, &mut self.crystal_type, "Grade");
                    combo_box_row(ui, &mut self.body_part, "Body part");
                });

                ui.horizontal(|ui| {
                    combo_box_row(ui, &mut self.quality, "Quality");
                    combo_box_row(ui, &mut self.color, "Color");
                });

                text_row_multiline(ui, &mut self.desc[holders.localization], "Description");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.add(egui::Label::new("Action"));

                    egui::ComboBox::from_id_salt(ui.next_auto_id())
                        .selected_text(self.default_action.label_text())
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                            ui.set_min_width(20.0);

                            for t in ItemDefaultAction::iter() {
                                ui.selectable_value(&mut self.default_action, t, t.label_text());
                            }
                        });
                });

                text_row_c(ui, &mut self.tooltip_texture, "Tooltip Texture");
                text_row_c(ui, &mut self.equip_sound, "Equip Sound");

                ui.horizontal(|ui| {
                    num_row(ui, &mut self.popup, "Popup");
                    num_row(ui, &mut self.use_order, "Use Order");
                });

                num_row_optional(ui, &mut self.set_id.0, "Set", "Id", u16::MAX as u32).on_hover_ui(
                    |ui| {
                        holders
                            .game_data_holder
                            .item_set_holder
                            .get(&self.set_id)
                            .draw_as_tooltip(ui);
                    },
                );

                ui.separator();

                combo_box_row(ui, &mut self.keep_type, "Keep");
                combo_box_row(ui, &mut self.inventory_type, "Inventory");
                combo_box_row(ui, &mut self.material, "Material");
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(200.);
                bool_row(ui, &mut self.is_trade, "Can trade");
                bool_row(ui, &mut self.is_drop, "Can drop");
                bool_row(ui, &mut self.is_destruct, "Can destroy");
                bool_row(ui, &mut self.is_private_store, "Can sell in private store");
                bool_row(ui, &mut self.is_npc_trade, "Can sell to Npc");
                bool_row(ui, &mut self.is_commission_store, "Can sell in commission");
                bool_row(ui, &mut self.crystallizable, "Can crystallize");

                ui.separator();

                num_row_optional(ui, &mut self.durability, "Durability", "", u16::MAX);
                num_row(ui, &mut self.weight, "Weight");
                num_row(ui, &mut self.default_price, "Default Price");
                bool_row(ui, &mut self.is_premium, "Is Premium");
                bool_row(ui, &mut self.is_blessed, "Is Blessed");

                ui.separator();

                num_row(ui, &mut self.property_params, "Property Params");

                self.icons.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Icons   ",
                    &format!("Icons {}", self.name[holders.localization]),
                    &format!("{} item_icons", self.id.0),
                    init_rect,
                );

                self.additional_info.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Additional   ",
                    &format!("Additional {}", self.name[holders.localization]),
                    &format!("{} item_additional", self.id.0),
                    init_rect,
                );

                self.drop_info.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Drop   ",
                    &format!("Drop {}", self.name[holders.localization]),
                    &format!("{} item_drop", self.id.0),
                    init_rect,
                );
                self.battle_stats.draw_as_button(
                    ui,
                    ctx,
                    holders,
                    "   Battle Params   ",
                    &format!("Battle Params {}", self.name[holders.localization]),
                    &format!("{} item_battle_params", self.id.0),
                    init_rect,
                );
            });

            ui.separator();
        })
        .response
    }
}

impl DrawActioned<(), ()> for ItemIcons {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        _holders: &DataHolder,
        _action: &RwLock<()>,
        _params: &mut (),
    ) {
        ui.vertical(|ui| {
            text_row_c(ui, &mut self.icon_1, "Icon 1");
            text_row_c(ui, &mut self.icon_2, "Icon 2");
            text_row_c(ui, &mut self.icon_3, "Icon 3");
            text_row_c(ui, &mut self.icon_4, "Icon 4");
            text_row_c(ui, &mut self.icon_5, "Icon 5");
            text_row_c(ui, &mut self.icon_panel, "Icon Panel");
        });
    }
}

impl DrawActioned<ItemAdditionalInfoAction, ()> for ItemAdditionalInfo {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<ItemAdditionalInfoAction>,
        _params: &mut (),
    ) {
        ui.vertical(|ui| {
            bool_row(ui, &mut self.has_animation, "Has Animation");
            bool_row(ui, &mut self.hide_cloak, "Hide Cloak");
            bool_row(ui, &mut self.unk, "Unk");
            bool_row(ui, &mut self.hide_armor, "Hide Armor");
            self.include_items.draw_horizontal(
                ui,
                "Include Items",
                |v| *action.write().unwrap() = ItemAdditionalInfoAction::RemoveItem(v),
                holders,
                true,
            );
            num_row_optional(ui, &mut self.max_energy, "Max Energy", "", u32::MAX);
            text_row_c(ui, &mut self.look_change, "Look Change");
        });
    }
}

impl DrawActioned<(), ()> for ItemBattleStats {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        _holders: &DataHolder,
        _action: &RwLock<()>,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                num_row(ui, &mut self.p_defense, "PDef");
                num_row(ui, &mut self.m_defense, "MDef");
                num_row(ui, &mut self.p_avoid, "P Evasion");
                num_row(ui, &mut self.m_avoid, "M Evasion");
                num_row(ui, &mut self.shield_defense, "Shield Def");
                num_row(ui, &mut self.shield_defense_rate, "Shield Rate")
            });

            ui.separator();

            ui.vertical(|ui| {
                num_row(ui, &mut self.p_attack, "PAtck");
                num_row(ui, &mut self.m_attack, "MAtck");
                num_row(ui, &mut self.p_hit, "P Accuracy");
                num_row(ui, &mut self.m_hit, "M Accuracy");
                num_row(ui, &mut self.p_critical, "P Crit");
                num_row(ui, &mut self.m_critical, "M Crit");
            });

            ui.separator();

            ui.vertical(|ui| {
                num_row(ui, &mut self.speed, "Speed");
                num_row(ui, &mut self.p_attack_speed, "Attack Speed");
                num_row(ui, &mut self.property_params, "Prop params");
            });
        });
    }
}
impl DrawActioned<ItemDropInfoAction, ()> for ItemDropInfo {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<ItemDropInfoAction>,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            ui.set_width(600.);
            ui.set_height(200.);

            ui.vertical(|ui| {
                ui.set_width(300.);

                combo_box_row(ui, &mut self.drop_type, "Type");
                combo_box_row(ui, &mut self.drop_animation_type, "Animation Type");

                num_row(ui, &mut self.drop_radius, "Radius");
                num_row(ui, &mut self.drop_height, "Height");

                text_row_c(
                    ui,
                    &mut self.complete_item_drop_sound,
                    "Complete Item Drop Sound",
                );

                text_row_c(ui, &mut self.drop_sound, "Drop Sound");
            });

            self.drop_mesh_info.draw_vertical(
                ui,
                "Meshes",
                |v| *action.write().unwrap() = ItemDropInfoAction::RemoveMesh(v),
                holders,
                true,
                true,
            );
        });
    }
}

impl Draw for ItemDropMeshInfo {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.vertical(|ui| {
            text_row_c(ui, &mut self.mesh, "Mesh");
            ui.horizontal(|ui| {
                ui.label("Textures");
                if ui.button(ADD_ICON).clicked() {
                    self.textures.push(Default::default());
                }
                if !self.textures.is_empty() && ui.button("-").clicked() {
                    self.textures.pop();
                }
            });

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        for v in &mut self.textures {
                            ui.horizontal(|ui| {
                                ui.add_space(10.);
                                ui.text_edit_singleline(v.as_mut_string());
                                ui.add_space(6.);
                            });
                        }
                    });
                });
            });
        })
        .response
    }
}

impl DrawAsTooltip for Item {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("{} [{}]", self.name, self.id.0));

        if !self.desc.is_empty() {
            ui.label(self.desc.to_string());
        };
    }
}
