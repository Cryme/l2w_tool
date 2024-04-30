mod item;
mod item_set;
mod npc;
mod quest;
mod recipe;
mod skill;
mod spawn_editor;
mod util;

use crate::backend::{
    Backend, CurrentOpenedEntity, Dialog, DialogAnswer, LogHolder, LogLevel, LogLevelFilter,
    WindowParams,
};
use crate::data::{ItemId, Location, NpcId, Position};
use crate::egui::special_emojis::GITHUB;
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::Entity;
use crate::frontend::spawn_editor::SpawnEditor;
use crate::frontend::util::{combo_box_row, Draw, DrawActioned, DrawAsTooltip};
use crate::holder::DataHolder;
use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui::{Color32, FontFamily, Image, Response, RichText, ScrollArea, TextureId, Ui};
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
        holders: &mut DataHolder,
        params: &mut Params,
    );
}

trait DrawWindow {
    fn draw_window(&mut self, ui: &mut Ui, ctx: &egui::Context, holders: &mut DataHolder);
}

impl<Inner: DrawEntity<Action, Params>, OriginalId, Action, Params> DrawWindow
    for WindowParams<Inner, OriginalId, Action, Params>
{
    fn draw_window(&mut self, ui: &mut Ui, ctx: &egui::Context, holders: &mut DataHolder) {
        self.inner
            .draw_entity(ui, ctx, &self.action, holders, &mut self.params);
    }
}

impl Draw for Location {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
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
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
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
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
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
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
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
            ui.horizontal(|ui| {
                ui.separator();
                if ui
                    .button(RichText::new("\u{f058}").family(FontFamily::Name("icons".into())))
                    .on_hover_text("Save changes\n(in memory only, will not write them to disk!)")
                    .clicked()
                {
                    self.backend.save_current_entity();
                }
                ui.separator();

                if ui
                    .button(RichText::new("\u{f56e}").family(FontFamily::Name("icons".into())))
                    .on_hover_text("Export in RON format")
                    .clicked()
                {
                    if let Some(v) = self.backend.current_entity_as_ron() {
                        ui.output_mut(|o| o.copied_text = v);
                    }
                }

                if ui
                    .button(RichText::new("\u{f56f}").family(FontFamily::Name("icons".into())))
                    .on_hover_text("Import from RON format")
                    .clicked()
                {
                    self.backend.fill_current_entity_from_ron(
                        &ClipboardContext::new().unwrap().get_contents().unwrap(),
                    );
                }

                ui.separator();

                ui.vertical(|ui| {
                    ui.push_id(ui.next_auto_id(), |ui| {
                        ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                self.draw_quest_tabs(ui);
                                self.draw_npc_tabs(ui);
                                self.draw_skill_tabs(ui);
                                self.draw_weapon_tabs(ui);
                                self.draw_etc_items_tabs(ui);
                                self.draw_armor_tabs(ui);
                                self.draw_item_set_tabs(ui);
                                self.draw_recipe_tabs(ui);
                            });
                        });
                    });
                });
            });
            ui.separator();
        }
    }

    fn build_top_menu(&mut self, ui: &mut Ui, ctx: &egui::Context) {
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

            if ui.button(" 📚 ").on_hover_text(".dat Editor").clicked() {
                self.search_params.search_showing = true;
            }

            if let Some(p) = &self.backend.config.server_spawn_root_folder_path {
                if ui.button(" 🗺 ").on_hover_text("Spawn Viewer").clicked() {
                    if self.spawn_editor.editor.is_none() {
                        let mut c = HashMap::new();

                        for npc in self.backend.holders.game_data_holder.npc_holder.values() {
                            c.insert(npc.id.0, format!("{} [{}]", npc.name, npc.id.0));
                        }

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
                    } else {
                        self.spawn_editor.showing = true;
                    }
                }
            }

            self.backend.logs.draw_as_button_tooltip(
                ui,
                ctx,
                &self.backend.holders,
                RichText::new(&self.backend.logs.inner)
                    .family(FontFamily::Name("icons".into()))
                    .color(&self.backend.logs.inner.max_level),
                "Logs",
                "app_logs",
                "Logs",
            );
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

        if self.spawn_editor.showing {
            if let Some(v) = &mut self.spawn_editor.editor {
                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("_spawn_editor_"),
                    egui::ViewportBuilder::default()
                        .with_title("Spawn Viewer")
                        .with_inner_size([600.0, 300.0]),
                    |ctx, class| {
                        assert!(
                            class == egui::ViewportClass::Immediate,
                            "This egui backend doesn't support multiple viewports"
                        );

                        egui::CentralPanel::default().show(ctx, |ui| {
                            v.show(ctx, ui);
                        });

                        if ctx.input(|i| i.viewport().close_requested()) {
                            // Tell parent viewport that we should not show next frame:
                            self.spawn_editor.showing = false;
                        }
                    },
                );
            }
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

impl From<&LogLevel> for Color32 {
    fn from(value: &LogLevel) -> Self {
        match value {
            LogLevel::Info => Color32::from_rgb(196, 210, 221),
            LogLevel::Warning => Color32::from_rgb(238, 146, 62),
            LogLevel::Error => Color32::from_rgb(238, 62, 62),
        }
    }
}

impl DrawActioned<(), ()> for LogHolder {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        _holders: &DataHolder,
        _action: &RwLock<()>,
        _params: &mut (),
    ) {
        ui.horizontal(|ui| {
            combo_box_row(ui, &mut self.level_filter, "Level");
            ui.label("Producer");
            egui::ComboBox::from_id_source(ui.next_auto_id())
                .selected_text(&self.producer_filter)
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(20.0);

                    let mut c = self.producers.iter().collect::<Vec<&String>>();
                    c.sort();
                    for t in c {
                        ui.selectable_value(&mut self.producer_filter, t.clone(), t);
                    }
                });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.set_min_height(350.);
            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    for log in self.logs.iter().filter(|v| {
                        let a = self.producer_filter == "All" || self.producer_filter == v.producer;
                        let b = self.level_filter == LogLevelFilter::All
                            || self.level_filter as u8 == v.level as u8;

                        a && b
                    }) {
                        ui.horizontal(|ui| {
                            ui.label(&log.producer);
                            ui.label(RichText::new(&log.log).color(&log.level));
                        });

                        ui.add_space(5.0);
                    }
                });

                ui.add_space(5.0);
            });
        });
    }
}

impl From<&LogHolder> for String {
    fn from(value: &LogHolder) -> Self {
        match &value.max_level {
            LogLevel::Info => "\u{f0eb}".to_string(),
            LogLevel::Warning => "\u{f071}".to_string(),
            LogLevel::Error => "\u{f071}".to_string(),
        }
    }
}
