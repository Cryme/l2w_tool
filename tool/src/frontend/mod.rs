mod npc;
mod quest;
mod skill;
mod spawn_editor;
mod util;

use crate::backend::{Backend, CurrentOpenedEntity, Dialog, DialogAnswer, Holders};
use crate::data::{ItemId, Location, NpcId};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::Entity;
use crate::frontend::egui::special_emojis::GITHUB;
use crate::frontend::spawn_editor::SpawnEditor;
use crate::frontend::util::{BuildAsTooltip, Draw};
use eframe::egui::{Button, Color32, Image, Response, ScrollArea, TextureId, Ui};
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
pub const WORLD_MAP: &[u8] = include_bytes!("../../../files/map.png");

const DELETE_ICON: &str = "🗑";
const ADD_ICON: &str = "➕";

lazy_static! {
    pub static ref IS_SAVING: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
}

impl BuildAsTooltip for Item {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("{} [{}]", self.name, self.id.0));

        if !self.desc.is_empty() {
            ui.label(self.desc.to_string());
        };
    }
}

impl BuildAsTooltip for HuntingZone {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(format!("[{}]\n {}", self.id.0, self.name));

            if !self.desc.is_empty() {
                ui.label(self.desc.to_string());
            }
        });
    }
}

trait DrawEntity<T> {
    fn build(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        action: &RwLock<T>,
        holders: &mut Holders,
    );

    fn build_selector(backend: &mut Backend, ui: &mut Ui, max_height: f32, width: f32);
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

impl Draw for NpcId {
    fn draw(&mut self, ui: &mut Ui, holders: &Holders) -> Response {
        ui.add(egui::DragValue::new(&mut self.0)).on_hover_ui(|ui| {
            holders
                .game_data_holder
                .npc_holder
                .get(self)
                .build_as_tooltip(ui)
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
                .build_as_tooltip(ui)
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
                    format!(
                        "{}",
                        if let Some(n) = c.get(&v) {
                            n
                        } else {
                            "Not Exist"
                        }
                    )
                }),
            );
        }

        self.backend.update_npc_spawn_path(path);
    }

    fn build_editor(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        match self.backend.edit_params.current_opened_entity {
            CurrentOpenedEntity::Npc(index) => {
                let npc = &mut self.backend.edit_params.npcs.opened[index];

                npc.inner
                    .build(ui, ctx, &npc.action, &mut self.backend.holders);
            }

            CurrentOpenedEntity::Quest(index) => {
                let quest = &mut self.backend.edit_params.quests.opened[index];

                quest
                    .inner
                    .build(ui, ctx, &quest.action, &mut self.backend.holders);
            }

            CurrentOpenedEntity::Skill(index) => {
                let e = &mut self.backend.edit_params.skills.opened[index];

                e.inner
                    .build(ui, ctx, &e.action, &mut self.backend.holders, &mut e.params);
            }

            CurrentOpenedEntity::None => {}
        }
    }

    fn build_tabs(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    for (i, (title, id)) in self
                        .backend
                        .edit_params
                        .get_opened_quests_info()
                        .iter()
                        .enumerate()
                    {
                        let mut button = Button::new(format!("{} [{}]", title, id.0));

                        if CurrentOpenedEntity::Quest(i)
                            == self.backend.edit_params.current_opened_entity
                        {
                            button = button.fill(Color32::from_rgb(42, 70, 83));
                        }

                        if ui.add(button).clicked() && !self.backend.dialog_showing {
                            self.backend.edit_params.set_current_quest(i);
                        }
                        if ui.button("❌").clicked() {
                            self.backend.edit_params.close_quest(i);
                        }

                        ui.separator();
                    }

                    for (i, (title, id)) in self
                        .backend
                        .edit_params
                        .get_opened_skills_info()
                        .iter()
                        .enumerate()
                    {
                        let mut button = Button::new(format!("{} [{}]", title, id.0));

                        if CurrentOpenedEntity::Skill(i)
                            == self.backend.edit_params.current_opened_entity
                        {
                            button = button.fill(Color32::from_rgb(42, 70, 83));
                        }

                        if ui.add(button).clicked() && !self.backend.dialog_showing {
                            self.backend.edit_params.set_current_skill(i);
                        }

                        if ui.button("❌").clicked() {
                            self.backend.edit_params.close_skill(i);
                        }

                        ui.separator();
                    }

                    for (i, (title, id)) in self
                        .backend
                        .edit_params
                        .get_opened_npcs_info()
                        .iter()
                        .enumerate()
                    {
                        let mut button = Button::new(format!("{} [{}]", title, id.0));

                        if CurrentOpenedEntity::Npc(i)
                            == self.backend.edit_params.current_opened_entity
                        {
                            button = button.fill(Color32::from_rgb(42, 70, 83));
                        }

                        if ui.add(button).clicked() && !self.backend.dialog_showing {
                            self.backend.edit_params.set_current_npc(i);
                        }

                        if ui.button("❌").clicked() {
                            self.backend.edit_params.close_npc(i);
                        }

                        ui.separator();
                    }
                });
            });

            if self.backend.edit_params.current_opened_entity.is_some() {
                if ui.button("Save").clicked() {
                    self.backend.save_current_entity();
                }
            }
        });

        if self.backend.edit_params.current_opened_entity.is_some() {
            ui.separator();
        }
    }

    fn build_top_menu(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.menu_button(" ⚙ ", |ui| {
                if ui.button("Set L2 System Folder").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.backend.update_system_path(path)
                    }
                }
                if ui.button("Set server quests Java classes folder").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.backend.update_quests_java_path(path)
                    }
                }
                if ui.button("Set server NPC spawn root folder").clicked() {
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
                .on_hover_text("Search/Edit/Create")
                .clicked()
            {
                self.search_params.search_showing = true;
            }

            if let Some(p) = &self.backend.config.server_spawn_root_folder_path {
                let mut c = HashMap::new();

                for npc in self.backend.holders.game_data_holder.npc_holder.values() {
                    c.insert(npc.id.0, format!("{} [{}]", npc.name, npc.id.0));
                }

                if ui.button(" 🗺 ").on_hover_text("Spawn Editor").clicked() {
                    self.spawn_editor.show(
                        p,
                        Box::new(move |v| {
                            format!(
                                "{}",
                                if let Some(n) = c.get(&v) {
                                    n
                                } else {
                                    "Not Exist"
                                }
                            )
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
                    {};

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://armor.png",
                            ARMOR_ICON,
                        )))
                        .on_hover_text("Armor")
                        .clicked()
                    {};

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://etc.png",
                            ETC_ICON,
                        )))
                        .on_hover_text("Etc")
                        .clicked()
                    {};

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://set.png",
                            SET_ICON,
                        )))
                        .on_hover_text("Sets")
                        .clicked()
                    {};

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://recipe.png",
                            RECIPE_ICON,
                        )))
                        .on_hover_text("Recipes")
                        .clicked()
                    {};
                });

                ui.separator();

                match self.search_params.current_entity {
                    Entity::Npc => {
                        Self::build_npc_selector(
                            &mut self.backend,
                            ui,
                            ctx.screen_rect().height() - 130.,
                            LIBRARY_WIDTH,
                        );
                    }

                    Entity::Quest => {
                        Self::build_quest_selector(
                            &mut self.backend,
                            ui,
                            ctx.screen_rect().height() - 130.,
                            LIBRARY_WIDTH,
                        );
                    }

                    Entity::Skill => Self::build_skill_selector(
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

            self.build_tabs(ui, ctx);

            self.build_editor(ui, ctx);

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
