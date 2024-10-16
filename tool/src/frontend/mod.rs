mod entity_impl;
mod map_icons_editor;
mod spawn_editor;
mod util;
mod script_runner;

use crate::backend::editor::{entity::ChangeTrackedParams, CurrentEntity, WindowParams};
use crate::backend::entity_catalog::{EntityCatalog, EntityInfo, FilterMode};
use crate::backend::holder::{ChangeStatus, DataHolder, DictEditItem, HolderMapOps};
use crate::backend::log_holder::{LogHolder, LogHolderParams, LogLevel, LogLevelFilter};
use crate::backend::{Backend, Dialog, DialogAnswer};
use crate::common::{ItemId, Location, NpcId, Position, QuestId};
use crate::entity::{CommonEntity, Dictionary, GameEntity};
use crate::frontend::map_icons_editor::MapIconsEditor;
use crate::frontend::spawn_editor::SpawnEditor;
use crate::frontend::util::num_value::NumberValue;
use crate::frontend::util::{combo_box_row, num_row, Draw, DrawActioned, DrawAsTooltip};
use crate::logs;
use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui::{
    Align2, Button, Color32, FontFamily, Image, Key, Modifiers, Response, RichText, ScrollArea,
    TextWrapMode, TextureId, Ui, Vec2,
};
use eframe::{egui, glow};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use strum::IntoEnumIterator;
use crate::frontend::script_runner::ScriptRunner;

const QUEST_ICON: &[u8] = include_bytes!("../../../files/quest.png");
const SKILL_ICON: &[u8] = include_bytes!("../../../files/skill.png");
const NPC_ICON: &[u8] = include_bytes!("../../../files/npc.png");
const WEAPON_ICON: &[u8] = include_bytes!("../../../files/weapon.png");
const ARMOR_ICON: &[u8] = include_bytes!("../../../files/armor.png");
const ETC_ICON: &[u8] = include_bytes!("../../../files/etc.png");
const SET_ICON: &[u8] = include_bytes!("../../../files/set.png");
const RECIPE_ICON: &[u8] = include_bytes!("../../../files/recipe.png");
const REGION_ICON: &[u8] = include_bytes!("../../../files/region.png");
const HUNTING_ZONE_ICON: &[u8] = include_bytes!("../../../files/hunting_zone.png");
const RAID_INFO_ICON: &[u8] = include_bytes!("../../../files/raid_info.png");
const DAILY_MISSION_ICON: &[u8] = include_bytes!("../../../files/daily_mission.png");
const ANIMATION_COMBO_ICON: &[u8] = include_bytes!("../../../files/animation_combo.png");
const RESIDENCE_ICON: &[u8] = include_bytes!("../../../files/residence.png");

pub const NOT_FOUND: &[u8] = include_bytes!("../../../files/none.png");

pub const WORLD_MAP: &[u8] = include_bytes!("../../../files/map.png");
pub const INGAME_WORLD_MAP: &[u8] = include_bytes!("../../../files/map_d.png");

const DELETE_ICON: &str = "🗑";
const ADD_ICON: &str = "➕";

pub(crate) static IS_SAVING: AtomicBool = AtomicBool::new(false);

struct GlobalSearchParams {
    pub search_showing: bool,
    pub current_entity: GameEntity,
}

pub struct Frontend {
    backend: Backend,
    search_params: GlobalSearchParams,
    spawn_editor: SpawnEditor,
    map_icons_editor: MapIconsEditor,
    script_runner: ScriptRunner,
    allow_close: bool,
    ask_close: bool,

    show_npc_string_editor: bool,
    show_system_string_editor: bool,
    show_script_runner: bool,
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
        match self.backend.editors.current_entity {
            CurrentEntity::Npc(index) => self.backend.editors.npcs.opened[index].draw_window(
                ui,
                ctx,
                &mut self.backend.holders,
            ),

            CurrentEntity::Quest(index) => self.backend.editors.quests.opened[index].draw_window(
                ui,
                ctx,
                &mut self.backend.holders,
            ),

            CurrentEntity::Skill(index) => self.backend.editors.skills.opened[index].draw_window(
                ui,
                ctx,
                &mut self.backend.holders,
            ),

            CurrentEntity::Weapon(index) => self.backend.editors.weapons.opened[index].draw_window(
                ui,
                ctx,
                &mut self.backend.holders,
            ),

            CurrentEntity::EtcItem(index) => self.backend.editors.etc_items.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::Armor(index) => self.backend.editors.armor.opened[index].draw_window(
                ui,
                ctx,
                &mut self.backend.holders,
            ),

            CurrentEntity::ItemSet(index) => self.backend.editors.item_sets.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::Recipe(index) => self.backend.editors.recipes.opened[index].draw_window(
                ui,
                ctx,
                &mut self.backend.holders,
            ),

            CurrentEntity::HuntingZone(index) => self.backend.editors.hunting_zones.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::Region(index) => self.backend.editors.regions.opened[index].draw_window(
                ui,
                ctx,
                &mut self.backend.holders,
            ),

            CurrentEntity::RaidInfo(index) => self.backend.editors.raid_info.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::DailyMission(index) => self.backend.editors.daily_mission.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::AnimationCombo(index) => self.backend.editors.animation_combo.opened
                [index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::Residence(index) => self.backend.editors.residences.opened[index]
                .draw_window(ui, ctx, &mut self.backend.holders),

            CurrentEntity::None => {}
        }

        self.draw_dict_editors(ctx);
    }

    fn draw_tabs(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        if self.backend.editors.current_entity.is_some() {
            ui.horizontal(|ui| {
                ui.separator();

                let mut button =
                    Button::new(RichText::new("\u{f058}").family(FontFamily::Name("icons".into())));

                if self.backend.current_entity_changed(false) {
                    button = button.fill(Color32::from_rgb(152, 80, 0));
                }

                if ui
                    .add(button)
                    .on_hover_text("Save changes\n(Ctrl+S)")
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
                    if let Some(v) = self.backend.export_entity_as_ron_string() {
                        ui.output_mut(|o| o.copied_text = v);
                    }
                }

                if ui
                    .button(RichText::new("\u{f56f}").family(FontFamily::Name("icons".into())))
                    .on_hover_text("Import from RON format")
                    .clicked()
                {
                    self.backend.import_entity_from_ron_string(
                        &ClipboardContext::new().unwrap().get_contents().unwrap(),
                    );
                }

                ui.separator();

                ui.vertical(|ui| {
                    ui.push_id(ui.next_auto_id(), |ui| {
                        ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                self.draw_quest_tabs(ui);
                                self.draw_skill_tabs(ui);
                                self.draw_npc_tabs(ui);
                                self.draw_weapon_tabs(ui);
                                self.draw_armor_tabs(ui);
                                self.draw_etc_items_tabs(ui);
                                self.draw_item_set_tabs(ui);
                                self.draw_recipe_tabs(ui);
                                self.draw_hunting_zone_tabs(ui);
                                self.draw_region_tabs(ui);
                                self.draw_raid_info_tabs(ui);
                                self.draw_daily_missions_tabs(ui);
                                self.draw_animation_combo_tabs(ui);
                                self.draw_residence_tabs(ui);
                            });
                        });
                    });
                });

                //handle shortcuts
                {
                    if ui
                        .ctx()
                        .input_mut(|i| i.consume_key(Modifiers::CTRL, Key::S))
                    {
                        self.backend.save_current_entity();
                    } else if ui
                        .ctx()
                        .input_mut(|i| i.consume_key(Modifiers::CTRL, Key::W))
                    {
                        self.backend.close_current_entity();
                    }
                }
            });
            ui.separator();
        }
    }

    fn draw_top_menu(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.menu_button(
                RichText::new(" \u{f013} ").family(FontFamily::Name("icons".into())),
                |ui| {
                    if ui.button("Select L2 system folder").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.backend.update_system_path(path)
                        }
                    }
                    if ui.button("Select .ron dumps folder").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.backend.update_ron_dumps_path(path)
                        }
                    }
                    if ui
                        .button("Select textures folders")
                        .on_hover_text("Textures should be unpacked as TGA/PNG")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.backend.update_textures_path(path)
                        }
                    }
                    if ui.button("Select GS quest classes folder").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.backend.update_quests_java_path(path)
                        }
                    }
                    if ui.button("Select GS spawn folder").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.update_npc_spawn_path(path)
                        }
                    }
                },
            )
            .response
            .on_hover_text("Settings");

            let mut b =
                Button::new(RichText::new(" \u{f0c7} ").family(FontFamily::Name("icons".into())));

            if self.backend.is_changed() {
                b = b.fill(Color32::from_rgb(152, 80, 0));
            }

            if ui
                .add(b)
                .on_hover_text(if self.backend.is_changed() {
                    format!(
                        "Write changes to .dat\n\nChanged:\n{:#?}",
                        self.backend.holders.game_data_holder.changed_entities()
                    )
                } else {
                    "No changes to save".to_string()
                })
                .clicked()
                && self.backend.is_changed()
            {
                self.backend.save_to_dat();
                ui.close_menu();
            }

            if let Some(p) = &self.backend.config.ron_dumps_folder_path {
                if ui
                    .add(Button::new(
                        RichText::new(" \u{f019} ").family(FontFamily::Name("icons".into())),
                    ))
                    .on_hover_text("Write all data to .ron")
                    .clicked()
                {
                    self.backend.save_to_ron(p);

                    ui.close_menu();
                }
            }

            if ui
                .button(RichText::new(" \u{f02d} ").family(FontFamily::Name("icons".into())))
                .on_hover_text("Entity Catalog")
                .clicked()
            {
                self.search_params.search_showing = true;
            }

            if ui
                .button(RichText::new(" \u{f121} ").family(FontFamily::Name("icons".into())))
                .on_hover_text("Run Script")
                .clicked()
            {
                self.script_runner.opened = true;
            }

            ui.menu_button(
                RichText::new(" \u{f1a7} ").family(FontFamily::Name("icons".into())),
                |ui| {
                    if ui.button("Edit System Strings").clicked() && !self.show_system_string_editor
                    {
                        self.show_system_string_editor = true;
                        self.backend.fill_system_strings_editor();
                        ui.close_menu();
                    }

                    if ui.button("Edit Npc Strings").clicked() && !self.show_npc_string_editor {
                        self.show_npc_string_editor = true;
                        self.backend.fill_npc_strings_editor();
                        ui.close_menu();
                    }
                },
            )
            .response
            .on_hover_text("Dictionaries");

            if let Some(p) = &self.backend.config.server_spawn_root_folder_path {
                if ui
                    .button(RichText::new(" \u{f279} ").family(FontFamily::Name("icons".into())))
                    .on_hover_text("Spawn Viewer")
                    .clicked()
                {
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

            if let Some(p) = &self.backend.config.textures_folder_path {
                if ui
                    .button(RichText::new(" \u{f5a0} ").family(FontFamily::Name("icons".into())))
                    .on_hover_text("Map Icons Editor")
                    .clicked()
                {
                    if !self.map_icons_editor.showing {
                        self.map_icons_editor.init(
                            self.backend
                                .holders
                                .game_data_holder
                                .hunting_zone_holder
                                .values(),
                            p,
                            ctx,
                        );

                        self.map_icons_editor.showing = true;
                    } else {
                        self.map_icons_editor.showing = false;
                    }
                }
            }

            self.backend.logs.draw_as_button_tooltip(
                ui,
                ctx,
                &self.backend.holders,
                RichText::new(self.backend.logs.inner.max_log_level)
                    .family(FontFamily::Name("icons".into()))
                    .color(self.backend.logs.inner.max_log_level),
                "Logs",
                "app_logs",
                "Logs",
            );
        });
    }

    fn draw_entity_library(&mut self, ctx: &egui::Context) {
        const LIBRARY_WIDTH: f32 = 392.;

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
                        self.search_params.current_entity = GameEntity::Quest;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://skill.png",
                            SKILL_ICON,
                        )))
                        .on_hover_text("Skills")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::Skill;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://npc.png",
                            NPC_ICON,
                        )))
                        .on_hover_text("Npcs")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::Npc;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://weapon.png",
                            WEAPON_ICON,
                        )))
                        .on_hover_text("Weapon")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::Weapon;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://armor.png",
                            ARMOR_ICON,
                        )))
                        .on_hover_text("Armor/Jewelry")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::Armor;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://etc.png",
                            ETC_ICON,
                        )))
                        .on_hover_text("Etc Items")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::EtcItem;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://set.png",
                            SET_ICON,
                        )))
                        .on_hover_text("Sets")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::ItemSet;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://recipe.png",
                            RECIPE_ICON,
                        )))
                        .on_hover_text("Recipes")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::Recipe;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://hunting_zone.png",
                            HUNTING_ZONE_ICON,
                        )))
                        .on_hover_text("Hunting Zones")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::HuntingZone;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://region.png",
                            REGION_ICON,
                        )))
                        .on_hover_text("Regions")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::Region;
                    };
                });

                ui.horizontal(|ui| {
                    ui.set_height(32.);

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://raid_info.png",
                            RAID_INFO_ICON,
                        )))
                        .on_hover_text("Raid Info")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::RaidInfo;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://daily_mission.png",
                            DAILY_MISSION_ICON,
                        )))
                        .on_hover_text("Daily Missions")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::DailyMission;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://animation_combo.png",
                            ANIMATION_COMBO_ICON,
                        )))
                        .on_hover_text("Animation Combo")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::AnimationCombo;
                    };

                    if ui
                        .add(egui::ImageButton::new(Image::from_bytes(
                            "bytes://residence.png",
                            RESIDENCE_ICON,
                        )))
                        .on_hover_text("Residence")
                        .clicked()
                    {
                        self.search_params.current_entity = GameEntity::Residence;
                    };
                });

                ui.separator();

                match self.search_params.current_entity {
                    GameEntity::Npc => {
                        Self::draw_npc_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::Quest => {
                        Self::draw_quest_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::Skill => {
                        Self::draw_skill_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::Weapon => {
                        Self::draw_weapon_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::EtcItem => {
                        Self::draw_etc_item_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::Armor => {
                        Self::draw_armor_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::ItemSet => {
                        Self::draw_item_set_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::Recipe => {
                        Self::draw_recipe_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::HuntingZone => {
                        Self::draw_hunting_zone_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::Region => {
                        Self::draw_region_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::RaidInfo => {
                        Self::draw_raid_info_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::DailyMission => {
                        Self::draw_daily_missions_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::AnimationCombo => {
                        Self::draw_animation_combo_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }

                    GameEntity::Residence => {
                        Self::draw_residence_selector(&mut self.backend, ui, LIBRARY_WIDTH)
                    }
                }
            });
    }

    fn draw_spawn_editor(&mut self, ctx: &egui::Context) {
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
    }

    fn draw(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_dialog(ctx, ui.min_size());

            self.draw_entity_library(ctx);

            self.draw_script_runner(ui, &ctx);

            self.draw_top_menu(ui, ctx);

            ui.separator();

            self.draw_tabs(ui, ctx);

            self.draw_editor(ui, ctx);

            if IS_SAVING.load(Ordering::Relaxed) {
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

    fn handle_close(&mut self, ctx: &egui::Context) {
        if self.backend.is_changed()
            && !self.allow_close
            && ctx.input(|i| i.viewport().close_requested())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.ask_close = true;
        }

        if self.ask_close {
            egui::Window::new("Discard Changes?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.set_height(65.);
                    ui.set_width(300.);
                    ui.vertical_centered(|ui| {
                        ui.label("Changes are unwritten to .dat\nAre you sure?");

                        ui.add_space(5.);

                        ui.horizontal(|ui| {
                            ui.add_space(105.);

                            if ui.button(" No ").clicked() {
                                self.allow_close = false;
                                self.ask_close = false;
                            }

                            ui.add_space(10.);

                            if ui.button(" Yes ").clicked() {
                                self.ask_close = false;
                                self.allow_close = true;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });
                });
        }
    }

    fn show_dialog(&mut self, ctx: &egui::Context, rect: Vec2) {
        match &self.backend.dialog {
            Dialog::ConfirmNpcSave { message, .. }
            | Dialog::ConfirmQuestSave { message, .. }
            | Dialog::ConfirmWeaponSave { message, .. }
            | Dialog::ConfirmEtcSave { message, .. }
            | Dialog::ConfirmArmorSave { message, .. }
            | Dialog::ConfirmItemSetSave { message, .. }
            | Dialog::ConfirmRecipeSave { message, .. }
            | Dialog::ConfirmHuntingZoneSave { message, .. }
            | Dialog::ConfirmRegionSave { message, .. }
            | Dialog::ConfirmRaidInfoSave { message, .. }
            | Dialog::ConfirmDailyMissionSave { message, .. }
            | Dialog::ConfirmAnimationComboSave { message, .. }
            | Dialog::ConfirmResidenceSave { message, .. }
            | Dialog::ConfirmSkillSave { message, .. } => {
                let m = message.clone();

                egui::Window::new("Confirm Overwrite")
                    .default_pos([rect.x / 2.0, rect.y / 2.0])
                    .pivot(Align2::CENTER_CENTER)
                    .id(egui::Id::new("_confirm_"))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.set_height(60.);
                            ui.set_width(300.);

                            ui.label(m);

                            ui.add_space(5.);

                            ui.horizontal(|ui| {
                                ui.add_space(100.);
                                if ui.button("Cancel").clicked() {
                                    self.backend.answer(DialogAnswer::Abort);
                                }
                                if ui.button("Confirm").clicked() {
                                    self.backend.answer(DialogAnswer::Confirm);
                                }
                            });
                        })
                    });
            }

            Dialog::ShowWarning(warn) => {
                let m = warn.clone();

                egui::Window::new("Warning!")
                    .id(egui::Id::new("_warn_"))
                    .collapsible(false)
                    .resizable(false)
                    .default_pos([rect.x / 2.0, rect.y / 2.0])
                    .pivot(Align2::CENTER_CENTER)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(m);

                            if ui.button("   Ok   ").clicked() {
                                self.backend.answer(DialogAnswer::Confirm);
                            }
                        })
                    });
            }

            Dialog::ConfirmClose(_) => {
                egui::Window::new("Confirm Close")
                    .default_pos([rect.x / 2.0, rect.y / 2.0])
                    .pivot(Align2::CENTER_CENTER)
                    .id(egui::Id::new("_close_entity_"))
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_height(65.);
                        ui.set_width(300.);
                        ui.vertical_centered(|ui| {
                            ui.label("There are unsaved changes!\nAre you sure?");

                            ui.add_space(5.);

                            ui.horizontal(|ui| {
                                ui.add_space(100.);
                                if ui.button("Close").clicked() {
                                    self.backend.answer(DialogAnswer::Confirm);
                                }
                                if ui.button("Cancel").clicked() {
                                    self.backend.answer(DialogAnswer::Abort);
                                }
                            });
                        })
                    });
            }

            Dialog::None => {}
        }
    }

    pub fn init(
        world_map_texture_id: TextureId,
        ingame_world_map_texture_id: TextureId,
        not_found_texture_id: TextureId,
    ) -> Self {
        let backend = Backend::init();
        let spawn_editor = SpawnEditor::init(world_map_texture_id);

        Self {
            map_icons_editor: MapIconsEditor::new(
                ingame_world_map_texture_id,
                not_found_texture_id,
            ),
            backend,
            search_params: GlobalSearchParams {
                search_showing: false,
                current_entity: GameEntity::Quest,
            },
            spawn_editor,
            allow_close: false,
            ask_close: false,

            show_system_string_editor: false,
            show_npc_string_editor: false,
            show_script_runner: false,
            script_runner: ScriptRunner::new(),
        }
    }
}

impl eframe::App for Frontend {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.backend.on_update();

        self.draw_spawn_editor(ctx);

        self.map_icons_editor.draw(ctx);

        self.draw(ctx);

        self.handle_close(ctx);

        if self.script_runner.execute_requested {
            self.script_runner.execute_requested = false;

            self.script_runner.output = self.backend.run_script(&self.script_runner.script);
        }
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        self.backend.auto_save(true);
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

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

impl<
        Inner: DrawEntity<Action, Params> + Serialize + DeserializeOwned,
        OriginalId: Default + Serialize + DeserializeOwned,
        Action: Default + Serialize + DeserializeOwned,
        Params: Default + Serialize + DeserializeOwned,
    > DrawWindow for ChangeTrackedParams<Inner, OriginalId, Action, Params>
{
    fn draw_window(&mut self, ui: &mut Ui, ctx: &egui::Context, holders: &mut DataHolder) {
        self.inner.draw_window(ui, ctx, holders);
    }
}

impl Draw for Location {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(NumberValue::new(&mut self.x));

            ui.label("Y");
            ui.add(NumberValue::new(&mut self.y));

            ui.label("Z");
            ui.add(NumberValue::new(&mut self.z));
        })
        .response
    }
}

impl Draw for Position {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(NumberValue::new(&mut self.x));

            ui.label("Y");
            ui.add(NumberValue::new(&mut self.y));

            ui.label("Z");
            ui.add(NumberValue::new(&mut self.z));
        })
        .response
    }
}

impl Draw for NpcId {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        ui.add(NumberValue::new(&mut self.0)).on_hover_ui(|ui| {
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
        ui.add(NumberValue::new(&mut self.0)).on_hover_ui(|ui| {
            holders
                .game_data_holder
                .item_holder
                .get(self)
                .draw_as_tooltip(ui)
        })
    }
}

impl From<LogLevel> for Color32 {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Info => Color32::from_rgb(196, 210, 221),
            LogLevel::Warning => Color32::from_rgb(238, 146, 62),
            LogLevel::Error => Color32::from_rgb(238, 62, 62),
        }
    }
}

impl DrawActioned<(), ()> for LogHolderParams {
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
            egui::ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(&self.producer_filter)
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
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
                    for log in logs().logs.iter().filter(|v| {
                        let a = self.producer_filter == LogHolder::ALL
                            || self.producer_filter == v.producer;
                        let b = self.level_filter == LogLevelFilter::All
                            || self.level_filter as u8 == v.level as u8;

                        a && b
                    }) {
                        ui.horizontal(|ui| {
                            ui.label(&log.producer);
                            ui.label(RichText::new(&log.log).color(log.level));
                        });

                        ui.add_space(5.0);
                    }
                });

                ui.add_space(5.0);
            });
        });
    }
}

impl From<LogLevel> for String {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Info => " \u{f0eb} ".to_string(),
            LogLevel::Warning => " \u{f071} ".to_string(),
            LogLevel::Error => " \u{f071} ".to_string(),
        }
    }
}

impl Draw for QuestId {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response {
        num_row(ui, &mut self.0, "Id").on_hover_ui(|ui| {
            holders
                .game_data_holder
                .quest_holder
                .get(self)
                .draw_as_tooltip(ui)
        })
    }
}

impl<Entity: CommonEntity<EntityId> + Clone, EntityId: Hash + Ord + Copy + Clone>
    EntityCatalog<Entity, EntityId>
where
    EntityInfo<Entity, EntityId>: for<'a> From<&'a Entity> + Ord,
{
    pub fn draw_search_and_add_buttons<Map: HolderMapOps<EntityId, Entity>>(
        &mut self,
        ui: &mut Ui,
        holder: &Map,
        filter_mode: &mut FilterMode,
        catalog_size: usize,
    ) -> Response {
        ui.horizontal(|ui| {
            let l = ui.text_edit_singleline(&mut self.filter);
            if ui.button("🔍").clicked()
                || (l.lost_focus() && l.ctx.input(|i| i.key_pressed(Key::Enter)))
            {
                self.filter(holder, *filter_mode);
            }
        });

        if !self.history.is_empty() {
            let mut c = false;
            egui::ComboBox::from_id_salt(ui.next_auto_id())
                .width(ui.spacing().text_edit_width)
                .selected_text(self.history.last().unwrap())
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                    for t in self.history.iter().rev() {
                        if ui
                            .selectable_value(&mut self.filter, t.clone(), t)
                            .clicked()
                        {
                            c = true;
                        };
                    }
                });

            if c {
                self.filter(holder, *filter_mode);
            }
        }

        let response = ui
            .horizontal(|ui| {
                ui.label("Show");

                egui::ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(format!("{}", filter_mode))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.set_min_width(20.0);

                        for t in FilterMode::iter() {
                            if ui
                                .selectable_value(filter_mode, t, format!("{t}"))
                                .clicked()
                            {
                                self.filter(holder, *filter_mode);
                            }
                        }
                    });
                ui.label(format!("Count: {catalog_size}"));

                ui.button("+ New")
            })
            .inner;

        response
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

impl<
        ID: Hash + Eq + Ord + Copy + Display,
        T: Clone + Ord + Eq + Display + eframe::egui::TextBuffer,
    > DictEditItem<ID, T>
{
    fn draw(&mut self, ui: &mut Ui, changed: &mut ChangeStatus, ident: usize) {
        ui.horizontal(|ui| {
            let mut t = RichText::new(format!(
                "{}ID: {:<ident$}",
                if self.changed { "*" } else { " " },
                self.id
            ))
            .monospace();

            if self.changed || !self.matches_initial {
                t = t.color(Color32::from_rgb(242, 192, 124));
            }

            ui.label(t);

            let field =
                ui.add(egui::TextEdit::singleline(&mut self.item).desired_width(f32::INFINITY));

            if field.changed() {
                *changed = self.check_changed_status();
            }

            if self.changed || !self.matches_initial {
                field.on_hover_text(format!(
                    "{}{}{}",
                    if self.changed {
                        format!("Previous: {}", self.previous)
                    } else {
                        "".to_string()
                    },
                    if self.changed && !self.matches_initial {
                        "\n"
                    } else {
                        ""
                    },
                    if !self.matches_initial {
                        format!("Initial: {}", self.initial)
                    } else {
                        "".to_string()
                    },
                ));
            }
        });
    }
}

impl Frontend {
    fn draw_dict_editors(&mut self, ctx: &egui::Context) {
        if self.show_system_string_editor {
            let mut save = false;
            let mut apply_search = false;

            let dict = &mut self.backend.editors.dictionaries.system_strings;
            let dict_changed = dict.changed();

            egui::Window::new(if dict_changed {
                "System Strings *"
            } else {
                "System Strings"
            })
            .id(egui::Id::new("_system_strings_editor_"))
            .collapsible(true)
            .resizable(true)
            .open(&mut self.show_system_string_editor)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(400.);

                    ui.horizontal(|ui| {
                        if ui.button("Add new").clicked() {
                            dict.add_new()
                        }

                        if ui
                            .button("Save")
                            .on_hover_text(if dict_changed {
                                "Save changes"
                            } else {
                                "No changes"
                            })
                            .clicked()
                        {
                            if dict_changed {
                                save = true;
                            }
                        }

                        if ui
                            .text_edit_singleline(&mut dict.search)
                            .on_hover_text("Search by id\nr:start\nr:start-end")
                            .changed()
                        {
                            apply_search = true;
                        }
                    });

                    ui.separator();

                    ui.push_id(ui.next_auto_id(), |ui| {
                        ScrollArea::vertical().show_rows(
                            ui,
                            21.,
                            dict.filtered_indexes.len(),
                            |ui, range| {
                                let mut changed = ChangeStatus::Same;

                                for i in &dict.filtered_indexes[range] {
                                    dict.items[*i].draw(ui, &mut changed, 5);
                                }

                                match changed {
                                    ChangeStatus::BecameChanged => dict.inc_changed(),
                                    ChangeStatus::BecameUnChanged => dict.dec_changed(),
                                    ChangeStatus::Same => (),
                                }
                            },
                        );
                    });
                });
            });

            if save {
                self.backend.store_dict(Dictionary::SystemStrings);
            }

            if apply_search {
                self.backend.apply_search(Dictionary::SystemStrings);
            }
        }

        if self.show_npc_string_editor {
            let mut save = false;
            let mut apply_search = false;

            let dict = &mut self.backend.editors.dictionaries.npc_strings;
            let dict_changed = dict.changed();

            egui::Window::new(if dict_changed {
                "Npc Strings *"
            } else {
                "Npc Strings"
            })
            .id(egui::Id::new("_npc_strings_editor_"))
            .collapsible(true)
            .resizable(true)
            .open(&mut self.show_npc_string_editor)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(700.);

                    ui.horizontal(|ui| {
                        if ui.button("Add new").clicked() {
                            dict.add_new()
                        }

                        if ui
                            .button("Save")
                            .on_hover_text(if dict_changed {
                                "Save changes"
                            } else {
                                "No changes"
                            })
                            .clicked()
                        {
                            if dict_changed {
                                save = true;
                            }
                        }

                        if ui
                            .text_edit_singleline(&mut dict.search)
                            .on_hover_text("Search by id\nr:start\nr:start-end")
                            .changed()
                        {
                            apply_search = true;
                        }
                    });

                    ui.separator();

                    ui.push_id(ui.next_auto_id(), |ui| {
                        ScrollArea::vertical().show_rows(
                            ui,
                            21.,
                            dict.filtered_indexes.len(),
                            |ui, range| {
                                let mut changed = ChangeStatus::Same;

                                for i in &dict.filtered_indexes[range] {
                                    dict.items[*i].draw(ui, &mut changed, 8);
                                }

                                match changed {
                                    ChangeStatus::BecameChanged => dict.inc_changed(),
                                    ChangeStatus::BecameUnChanged => dict.dec_changed(),
                                    ChangeStatus::Same => (),
                                }
                            },
                        );
                    });
                });
            });

            if save {
                self.backend.store_dict(Dictionary::NpcStrings);
            }

            if apply_search {
                self.backend.apply_search(Dictionary::NpcStrings);
            }
        }
    }
}
