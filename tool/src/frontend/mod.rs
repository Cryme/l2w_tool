mod item;
mod item_set;
mod npc;
mod quest;
mod recipe;
mod skill;
mod spawn_editor;
mod util;

use crate::backend::{Backend, CurrentOpenedEntity, Dialog, DialogAnswer, Holders, WindowParams};
use crate::data::{ItemId, Location, NpcId, Position};
use crate::egui::special_emojis::GITHUB;
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::Entity;
use crate::frontend::spawn_editor::SpawnEditor;
use crate::frontend::util::{Draw, DrawAsTooltip};
use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui::{Context, Image, Response, ScrollArea, TextureId, Ui};
use eframe::{egui, glow};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

const QUEST_ICON: &[u8] = include_bytes!("../../../files/quest.png");
const SKILL_ICON: &[u8] = include_bytes!("../../../files/skill.png");
const NPC_ICON: &[u8] = include_bytes!("../../../files/npc.png");
const WEAPON_ICON: &[u8] = include_bytes!("../../../files/weapon.png");
const ARMOR_ICON: &[u8] = include_bytes!("../../../files/armor.png");
const ETC_ICON: &[u8] = include_bytes!("../../../files/etc.png");
const SET_ICON: &[u8] = include_bytes!("../../../files/set.png");
const RECIPE_ICON: &[u8] = include_bytes!("../../../files/recipe.png");
pub const WORLD_MAP: &[u8] = include_bytes!("../../../files/map_s.png");

const DELETE_ICON: &str = "🗑";
const ADD_ICON: &str = "➕";

lazy_static! {
    pub static ref IS_SAVING: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
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

trait DrawEntity<Action, Params> {
    fn draw_entity(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        action: &RwLock<Action>,
        holders: &mut Holders,
        params: &mut Params,
    );
}

trait DrawWindow {
    fn draw_window(&mut self, ui: &mut Ui, ctx: &egui::Context, holders: &mut Holders);
}

impl<Inner: DrawEntity<Action, Params>, OriginalId, Action, Params> DrawWindow
    for WindowParams<Inner, OriginalId, Action, Params>
{
    fn draw_window(&mut self, ui: &mut Ui, ctx: &Context, holders: &mut Holders) {
        self.inner
            .draw_entity(ui, ctx, &self.action, holders, &mut self.params);
    }
}

impl Draw for Location {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(egui::DragValue::new(&mut self.x));

            ui.label("Y");
            ui.add(egui::DragValue::new(&mut self.y));

            ui.label("Z");
            ui.add(egui::DragValue::new(&mut self.z));
        })
        .response
    }
}

impl Draw for Position {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(egui::DragValue::new(&mut self.x));

            ui.label("Y");
            ui.add(egui::DragValue::new(&mut self.y));

            ui.label("Z");
            ui.add(egui::DragValue::new(&mut self.z));
        })
        .response
    }
}

impl Draw for NpcId {
    fn draw(&mut self, ui: &mut Ui, holders: &Holders) -> Response {
        ui.add(egui::DragValue::new(&mut self.0)).on_hover_ui(|ui| {
            holders
                .game_data_holder
                .npc_holder
                .get(self)
                .draw_as_tooltip(ui)
        })
    }
}

impl Draw for ItemId {
    fn draw(&mut self, ui: &mut Ui, holders: &Holders) -> Response {
        ui.add(egui::DragValue::new(&mut self.0)).on_hover_ui(|ui| {
            holders
                .game_data_holder
                .item_holder
                .get(self)
                .draw_as_tooltip(ui)
        })
    }
}

struct GlobalSearchParams {
    pub search_showing: bool,
    pub current_entity: Entity,
}

pub struct Frontend {
    backend: Backend,
    search_params: GlobalSearchParams,
    spawn_editor: SpawnEditor,
}

impl Frontend {
    fn update_npc_spawn_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let mut c = HashMap::new();

            for npc in self.backend.holders.game_data_holder.npc_holder.values() {
                c.insert(npc.id.0, format!("{} [{}]", npc.name, npc.id.0));
            }
            self.spawn_editor.update_spawn_path(
                path.to_str().unwrap(),
                Box::new(move |v| {
                    if let Some(n) = c.get(&v) {
                        n.clone()
                    } else {
                        format!("Not Exist [{v}]")
                    }
                }),
            );
        }

        self.backend.update_npc_spawn_path(path);
    }

    fn draw_editor(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        match self.backend.edit_params.current_opened_entity {
            CurrentOpenedEntity::Npc(index) => self.backend.edit_params.npcs.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentOpenedEntity::Quest(index) => self.backend.edit_params.quests.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentOpenedEntity::Skill(index) => self.backend.edit_params.skills.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentOpenedEntity::Weapon(index) => self.backend.edit_params.weapons.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentOpenedEntity::EtcItem(index) => self.backend.edit_params.etc_items.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentOpenedEntity::Armor(index) => self.backend.edit_params.armor.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentOpenedEntity::ItemSet(index) => self.backend.edit_params.item_sets.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentOpenedEntity::Recipe(index) => self.backend.edit_params.recipes.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentOpenedEntity::None => {}
        }
    }

    fn draw_tabs(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        if self.backend.edit_params.current_opened_entity.is_some() {
            ui.vertical(|ui| {
                ui.set_height(30.);
                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.separator();
                            self.draw_quest_tabs(ui);
                            self.draw_npc_tabs(ui);
                            self.draw_skill_tabs(ui);
                            self.draw_weapon_tabs(ui);
                            self.draw_etc_items_tabs(ui);
                            self.draw_armor_tabs(ui);
                            self.draw_item_set_tabs(ui);
                            self.draw_recipe_tabs(ui);
                        });

                        ui.add_space(6.);
                    });
                });
            });
            ui.horizontal(|ui| {
                ui.separator();
                if ui
                    .button("Apply Changes")
                    .on_hover_text("Only save changes in memory\nWill not write them to disk")
                    .clicked()
                {
                    self.backend.save_current_entity();
                }
                ui.separator();

                if ui
                    .button("Copy as Ron")
                    .on_hover_text("Ron is human-readable format, like Json")
                    .clicked()
                {
                    if let Some(v) = self.backend.current_entity_as_ron() {
                        ui.output_mut(|o| o.copied_text = v);
                    }
                }

                if ui
                    .button("Fill from Ron")
                    .on_hover_text("Ron should be copied to clipboard")
                    .clicked()
                {
                    self.backend.fill_current_entity_from_ron(
                        &ClipboardContext::new().unwrap().get_contents().unwrap(),
                    );
                }

                ui.separator();
            });
            ui.separator();
        }
    }

    fn build_top_menu(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.menu_button(" ⚙ ", |ui| {
                if ui.button("Set L2 system folder").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.backend.update_system_path(path)
                    }
                }
                if ui.button("Set GS quest classes folder").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.backend.update_quests_java_path(path)
                    }
                }
                if ui.button("Set GS spawn folder").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.update_npc_spawn_path(path)
                    }
                }
            })
            .response
            .on_hover_text("Settings");

            if ui.button(" 💾 ").on_hover_text("Save to .dat").clicked() {
                self.backend.save_to_dat();
                ui.close_menu();
            }

            ui.menu_button(" ℹ ", |ui| {
                ui.set_width(10.);
                ui.hyperlink_to(
                    format!("{GITHUB}"),
                    "https://github.com/La2world-ru/l2_quest_editor",
                );
                ui.hyperlink_to("✉".to_string(), "https://t.me/CrymeAriven");
                ui.hyperlink_to("🎮".to_string(), "https://la2world.ru");
            });

            if ui
                .button(" 📚 ")
                .on_hover_text(".dat Editor")
                .clicked()
            {
                self.search_params.search_showing = true;
            }

            if let Some(p) = &self.backend.config.server_spawn_root_folder_path {
                let mut c = HashMap::new();

                for npc in self.backend.holders.game_data_holder.npc_holder.values() {
                    c.insert(npc.id.0, format!("{} [{}]", npc.name, npc.id.0));
                }

                if ui.button(" 🗺 ").on_hover_text("Spawn Viewer").clicked() {
                    self.spawn_editor.show(
                        p,
                        Box::new(move |v| {
                            (if let Some(n) = c.get(&v) {
                                n
                            } else {
                                "Not Exist"
                            })
                            .to_string()
                        }),
                    );
                }
            }
        });
    }

    fn show_dialog(&mut self, ctx: &egui::Context) {
        match &self.backend.dialog {
            Dialog::ConfirmNpcSave { message, .. }
            | Dialog::ConfirmQuestSave { message, .. }
            | Dialog::ConfirmWeaponSave { message, .. }
            | Dialog::ConfirmEtcSave { message, .. }
            | Dialog::ConfirmArmorSave { message, .. }
            | Dialog::ConfirmItemSetSave { message, .. }
            | Dialog::ConfirmRecipeSave { message, .. }
            | Dialog::ConfirmSkillSave { message, .. } => {
                let m = message.clone();

                egui::Window::new("Confirm")
                    .id(egui::Id::new("_confirm_"))
                    .movable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(m);

                            ui.horizontal_centered(|ui| {
                                if ui.button("Confirm").clicked() {
                                    self.backend.answer(DialogAnswer::Confirm);
                                }
                                if ui.button("Abort").clicked() {
                                    self.backend.answer(DialogAnswer::Abort);
                                }
                            });
                        })
                    });
            }

            Dialog::ShowWarning(warn) => {
                let m = warn.clone();

                egui::Window::new("Warning!")
                    .id(egui::Id::new("_warn_"))
                    .resizable(false)
                    .movable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(m);

                            if ui.button("   Ok   ").clicked() {
                                self.backend.answer(DialogAnswer::Confirm);
                            }
                        })
                    });
            }

            Dialog::None => {}
        }
    }

    pub fn init(world_map_texture_id: TextureId) -> Self {
        let backend = Backend::init();
        let spawn_editor = SpawnEditor::init(world_map_texture_id);

        Self {
            backend,
            search_params: GlobalSearchParams {
                search_showing: false,
                current_entity: Entity::Quest,
            },
            spawn_editor,
        }
    }
}

impl eframe::App for Frontend {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        const LIBRARY_WIDTH: f32 = 312.;

        self.backend.on_update();

        egui::Window::new("📚")
            .id(egui::Id::new("_search_"))
            .open(&mut self.search_params.search_showing)
            .show(ctx, |ui| {
                ui.set_width(LIBRARY_WIDTH);

                ui.horizontal(|ui| {
                    ui.set_height(32.);
                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://quest.png",
                            QUEST_ICON,
                        )))
                        .on_hover_text("Quests")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Quest;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://skill.png",
                            SKILL_ICON,
                        )))
                        .on_hover_text("Skills")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Skill;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://npc.png",
                            NPC_ICON,
                        )))
                        .on_hover_text("Npcs")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Npc;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://weapon.png",
                            WEAPON_ICON,
                        )))
                        .on_hover_text("Weapon")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Weapon;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://armor.png",
                            ARMOR_ICON,
                        )))
                        .on_hover_text("Armor/Jewelry")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Armor;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://etc.png",
                            ETC_ICON,
                        )))
                        .on_hover_text("Etc Items")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::EtcItem;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://set.png",
                            SET_ICON,
                        )))
                        .on_hover_text("Sets")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::ItemSet;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://recipe.png",
                            RECIPE_ICON,
                        )))
                        .on_hover_text("Recipes")
                        .clicked()
                    {
                        self.search_params.current_entity = Entity::Recipe;
                    };
                });

                ui.separator();

                match self.search_params.current_entity {
                    Entity::Npc => Self::draw_npc_selector(
                        &mut self.backend,
                        ui,
                        ctx.screen_rect().height() - 130.,
                        LIBRARY_WIDTH,
                    ),

                    Entity::Quest => Self::draw_quest_selector(
                        &mut self.backend,
                        ui,
                        ctx.screen_rect().height() - 130.,
                        LIBRARY_WIDTH,
                    ),

                    Entity::Skill => Self::draw_skill_selector(
                        &mut self.backend,
                        ui,
                        ctx.screen_rect().height() - 130.,
                        LIBRARY_WIDTH,
                    ),

                    Entity::Weapon => Self::draw_weapon_selector(
                        &mut self.backend,
                        ui,
                        ctx.screen_rect().height() - 130.,
                        LIBRARY_WIDTH,
                    ),

                    Entity::EtcItem => Self::draw_etc_item_selector(
                        &mut self.backend,
                        ui,
                        ctx.screen_rect().height() - 130.,
                        LIBRARY_WIDTH,
                    ),

                    Entity::Armor => Self::draw_armor_selector(
                        &mut self.backend,
                        ui,
                        ctx.screen_rect().height() - 130.,
                        LIBRARY_WIDTH,
                    ),

                    Entity::ItemSet => Self::draw_item_set_selector(
                        &mut self.backend,
                        ui,
                        ctx.screen_rect().height() - 130.,
                        LIBRARY_WIDTH,
                    ),

                    Entity::Recipe => Self::draw_recipe_selector(
                        &mut self.backend,
                        ui,
                        ctx.screen_rect().height() - 130.,
                        LIBRARY_WIDTH,
                    ),
                }
            });

        if let Some(v) = &mut self.spawn_editor.editor {
            egui::Window::new("🗺")
                .id(egui::Id::new("_spawn_editor_"))
                .open(&mut self.spawn_editor.showing)
                .show(ctx, |ui| v.show(ctx, ui));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_dialog(ctx);

            self.build_top_menu(ui, ctx);

            ui.separator();

            self.draw_tabs(ui, ctx);

            self.draw_editor(ui, ctx);

            if *IS_SAVING.read().unwrap() {
                egui::Window::new("SAVING IN PROGRESS")
                    .id(egui::Id::new("_saving_"))
                    .resizable(false)
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.spinner();
                        })
                    });
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        self.backend.auto_save(true);
    }
}
