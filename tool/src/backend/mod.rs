pub mod dat_loader;
pub mod entity_catalog;
pub mod entity_editor;
pub mod entity_impl;
pub mod holder;
pub mod log_holder;
pub mod server_side;
mod util;

use crate::backend::holder::{ChroniclesProtocol, DataHolder, GameDataHolder, HolderMapOps};
use crate::backend::server_side::ServerDataHolder;
use crate::data::{AnimationComboId, DailyMissionId, HuntingZoneId, ItemId, ItemSetId, NpcId, QuestId, RaidInfoId, RecipeId, RegionId, SkillId};
use crate::entity::{CommonEntity, Entity};
use crate::logs_mut;
use dat_loader::DatLoader;
use dat_loader::{get_loader_from_holder, load_game_data_holder};
use entity_catalog::EntityCatalogsHolder;
use entity_editor::{CurrentEntity, EditParams, EditParamsCommonOps, WindowParams};
use log_holder::LogHolderParams;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use strum::IntoEnumIterator;

use crate::VERSION;

const AUTO_SAVE_INTERVAL: Duration = Duration::from_secs(30);
const CHANGE_CHECK_INTERVAL: Duration = Duration::from_millis(500);
const CONFIG_FILE_NAME: &str = "./config.ron";

pub struct Backend {
    pub config: Config,
    pub holders: DataHolder,
    pub entity_catalogs: EntityCatalogsHolder,
    pub dialog: Dialog,
    pub dialog_showing: bool,
    pub edit_params: EditParams,

    has_unwrote_changes: bool,

    pub logs: WindowParams<LogHolderParams, (), (), ()>,

    tasks: Tasks,
}

impl Backend {
    pub fn init() -> Self {
        let config = Self::load_config();

        let (game_data_holder, warnings) = if let Some(path) = &config.system_folder_path {
            load_game_data_holder(path, ChroniclesProtocol::GrandCrusade110).unwrap()
        } else {
            (GameDataHolder::default(), vec![])
        };

        let server_data_holder = if let Some(path) = &config.server_quests_java_classes_path {
            ServerDataHolder::load(path)
        } else {
            ServerDataHolder::default()
        };

        let edit_params = if let Ok(f) = File::open(format!("./v{VERSION}.asave")) {
            if let Ok(mut d) = bincode::deserialize_from::<File, EditParams>(f) {
                for v in Entity::iter() {
                    d.reset_initial(v, &game_data_holder)
                }

                d
            } else {
                EditParams::default()
            }
        } else {
            EditParams::default()
        };

        logs_mut().reset(warnings);

        let mut r = Self {
            config,

            holders: DataHolder {
                game_data_holder,
                server_data_holder,
            },
            entity_catalogs: EntityCatalogsHolder::new(),
            dialog: Dialog::None,

            dialog_showing: false,
            has_unwrote_changes: false,

            tasks: Tasks::init(),
            edit_params,
            logs: WindowParams::default(),
        };

        r.update_last_ids();

        r
    }

    pub fn save_to_dat(&mut self) {
        let mut loader = get_loader_from_holder(&self.holders.game_data_holder);

        loader.serialize_to_binary().unwrap();

        self.set_unchanged();
    }

    fn load_config() -> Config {
        let config_path = Path::new(CONFIG_FILE_NAME);
        if let Ok(mut f) = File::open(config_path) {
            let mut d = "".to_string();
            if f.read_to_string(&mut d).is_ok() {
                if let Ok(mut c) = ron::from_str(&d) {
                    ServerDataHolder::validate_paths(&mut c);
                    GameDataHolder::validate_paths(&mut c);

                    c.dump();

                    return c;
                }
            }
        }

        let c = Config::default();
        c.dump();

        c
    }

    fn get_current_entity(&self) -> Option<&dyn EditParamsCommonOps> {
        match self.edit_params.current_entity {
            CurrentEntity::Quest(i) => Some(&self.edit_params.quests.opened[i]),
            CurrentEntity::Skill(i) => Some(&self.edit_params.skills.opened[i]),
            CurrentEntity::Npc(i) => Some(&self.edit_params.npcs.opened[i]),
            CurrentEntity::Weapon(i) => Some(&self.edit_params.weapons.opened[i]),
            CurrentEntity::EtcItem(i) => Some(&self.edit_params.etc_items.opened[i]),
            CurrentEntity::Armor(i) => Some(&self.edit_params.armor.opened[i]),
            CurrentEntity::ItemSet(i) => Some(&self.edit_params.item_sets.opened[i]),
            CurrentEntity::Recipe(i) => Some(&self.edit_params.recipes.opened[i]),
            CurrentEntity::HuntingZone(i) => Some(&self.edit_params.hunting_zones.opened[i]),
            CurrentEntity::Region(i) => Some(&self.edit_params.regions.opened[i]),
            CurrentEntity::RaidInfo(i) => Some(&self.edit_params.raid_info.opened[i]),
            CurrentEntity::DailyMission(i) => Some(&self.edit_params.daily_mission.opened[i]),
            CurrentEntity::AnimationCombo(i) => Some(&self.edit_params.animation_combo.opened[i]),

            CurrentEntity::None => None,
        }
    }
    fn get_current_entity_mut(&mut self) -> Option<&mut dyn EditParamsCommonOps> {
        match self.edit_params.current_entity {
            CurrentEntity::Quest(i) => Some(&mut self.edit_params.quests.opened[i]),
            CurrentEntity::Skill(i) => Some(&mut self.edit_params.skills.opened[i]),
            CurrentEntity::Npc(i) => Some(&mut self.edit_params.npcs.opened[i]),
            CurrentEntity::Weapon(i) => Some(&mut self.edit_params.weapons.opened[i]),
            CurrentEntity::EtcItem(i) => Some(&mut self.edit_params.etc_items.opened[i]),
            CurrentEntity::Armor(i) => Some(&mut self.edit_params.armor.opened[i]),
            CurrentEntity::ItemSet(i) => Some(&mut self.edit_params.item_sets.opened[i]),
            CurrentEntity::Recipe(i) => Some(&mut self.edit_params.recipes.opened[i]),
            CurrentEntity::HuntingZone(i) => Some(&mut self.edit_params.hunting_zones.opened[i]),
            CurrentEntity::Region(i) => Some(&mut self.edit_params.regions.opened[i]),
            CurrentEntity::RaidInfo(i) => Some(&mut self.edit_params.raid_info.opened[i]),
            CurrentEntity::DailyMission(i) => Some(&mut self.edit_params.daily_mission.opened[i]),
            CurrentEntity::AnimationCombo(i) => Some(&mut self.edit_params.animation_combo.opened[i]),

            CurrentEntity::None => None,
        }
    }

    fn update_last_ids(&mut self) {
        self.entity_catalogs.quest.filter = "".to_string();
        self.filter_quests();

        self.entity_catalogs.skill.filter = "".to_string();
        self.filter_skills();

        self.entity_catalogs.npc.filter = "".to_string();
        self.filter_npcs();

        self.entity_catalogs.weapon.filter = "".to_string();
        self.filter_weapons();

        self.entity_catalogs.etc_item.filter = "".to_string();
        self.filter_etc_items();

        self.entity_catalogs.armor.filter = "".to_string();
        self.filter_armor();

        self.entity_catalogs.item_set.filter = "".to_string();
        self.filter_item_sets();

        self.entity_catalogs.recipe.filter = "".to_string();
        self.filter_recipes();

        self.entity_catalogs.hunting_zone.filter = "".to_string();
        self.filter_hunting_zones();

        self.entity_catalogs.region.filter = "".to_string();
        self.filter_regions();

        self.entity_catalogs.raid_info.filter = "".to_string();
        self.filter_raid_info();

        self.entity_catalogs.daily_mission.filter = "".to_string();
        self.filter_daily_mission();

        self.entity_catalogs.animation_combo.filter = "".to_string();
        self.filter_animation_combo();

        self.edit_params.quests.next_id =
            if let Some(last) = self.entity_catalogs.quest.catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.skills.next_id =
            if let Some(last) = self.entity_catalogs.skill.catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.npcs.next_id = if let Some(last) = self.entity_catalogs.npc.catalog.last()
        {
            last.id.0 + 1
        } else {
            0
        };

        let items_max_id = if let Some(last) = self.entity_catalogs.weapon.catalog.last() {
            last.id.0 + 1
        } else {
            0
        }
        .max(
            if let Some(last) = self.entity_catalogs.armor.catalog.last() {
                last.id.0 + 1
            } else {
                0
            },
        )
        .max(
            if let Some(last) = self.entity_catalogs.etc_item.catalog.last() {
                last.id.0 + 1
            } else {
                0
            },
        );

        self.edit_params.weapons.next_id = items_max_id;
        self.edit_params.armor.next_id = items_max_id;
        self.edit_params.etc_items.next_id = items_max_id;

        self.edit_params.item_sets.next_id =
            if let Some(last) = self.entity_catalogs.item_set.catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.recipes.next_id =
            if let Some(last) = self.entity_catalogs.recipe.catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.hunting_zones.next_id =
            if let Some(last) = self.entity_catalogs.hunting_zone.catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.regions.next_id =
            if let Some(last) = self.entity_catalogs.region.catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.raid_info.next_id =
            if let Some(last) = self.entity_catalogs.raid_info.catalog.last() {
                last.id.0 + 1
            } else {
                0
            };
    }

    pub fn auto_save(&mut self, force: bool) {
        if !force
            && SystemTime::now()
                .duration_since(self.tasks.last_auto_save)
                .unwrap()
                < AUTO_SAVE_INTERVAL
        {
            return;
        }

        if let Ok(mut out) = File::create(format!("./v{VERSION}.asave")) {
            out.write_all(&bincode::serialize(&self.edit_params).unwrap())
                .unwrap();
        }

        self.tasks.last_auto_save = SystemTime::now();
    }

    pub fn check_change(&mut self) {
        if SystemTime::now()
            .duration_since(self.tasks.last_change_check)
            .unwrap()
            < CHANGE_CHECK_INTERVAL
        {
            return;
        }

        if let Some(v) = self.get_current_entity_mut() {
            v.check_change();
        }

        self.tasks.last_change_check = SystemTime::now();
    }

    fn proceed_actions(&mut self) {
        if let Some(v) = self.get_current_entity_mut() {
            v.handle_actions();
        }
    }

    pub fn on_update(&mut self) {
        self.proceed_actions();
        self.logs.inner.sync();
        self.auto_save(false);
        self.check_change();
    }

    pub fn update_system_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let path = path.to_str().unwrap().to_string();

            if let Ok((h, w)) = load_game_data_holder(&path, ChroniclesProtocol::GrandCrusade110) {
                self.holders.game_data_holder = h;

                self.edit_params.current_entity = CurrentEntity::None;

                logs_mut().reset(w);

                self.update_last_ids();

                self.config.system_folder_path = Some(path);
                self.config.dump();
            }
        }
    }

    pub fn update_textures_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let path = path.to_str().unwrap().to_string();

            self.config.textures_folder_path = Some(path);
            self.config.dump();
        }
    }

    pub fn update_quests_java_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let path = path.to_str().unwrap().to_string();
            self.holders.server_data_holder = ServerDataHolder::load(&path);

            self.config.server_quests_java_classes_path = Some(path);
            self.config.dump();
        }
    }

    pub fn update_npc_spawn_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let path = path.to_str().unwrap().to_string();
            self.config.server_spawn_root_folder_path = Some(path);
            self.config.dump();
        }
    }

    pub fn current_entity_changed(&mut self, check: bool) -> bool {
        if let Some(v) = self.get_current_entity_mut() {
            if check {
                v.check_change();
            }

            v.is_changed()
        } else {
            false
        }
    }

    pub fn import_entity_from_ron_string(&mut self, val: &str) {
        if let Some(v) = self.get_current_entity_mut() {
            if let Err(e) = v.set_wrapped_entity_from_ron_string(val) {
                self.show_dialog(Dialog::ShowWarning(format!("{e:?}")));
            }
        }
    }

    pub fn export_entity_as_ron_string(&self) -> Option<String> {
        self.get_current_entity()
            .map(|v| v.get_wrapped_entity_as_ron_string())
    }

    pub fn save_current_entity(&mut self) {
        if !self.current_entity_changed(true) {
            return;
        }

        match self.edit_params.current_entity {
            CurrentEntity::Npc(index) => {
                let new_entity = self.edit_params.npcs.opened.get(index).unwrap();

                if new_entity.inner.inner.id.0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Npc ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .npc_holder
                    .get(&new_entity.inner.inner.id)
                {
                    if new_entity.inner.initial_id == old_entity.id || old_entity.deleted() {
                        self.save_npc_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmNpcSave {
                            message: format!(
                                "Npc with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            npc_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_npc_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::Quest(index) => {
                let new_entity = self.edit_params.quests.opened.get(index).unwrap();

                if new_entity.inner.inner.id.0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Quest ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .quest_holder
                    .get(&new_entity.inner.inner.id)
                {
                    if new_entity.inner.initial_id == old_entity.id || old_entity.deleted() {
                        self.save_quest_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmQuestSave {
                            message: format!(
                                "Quest with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            quest_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_quest_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::Skill(index) => {
                let new_entity = self.edit_params.skills.opened.get(index).unwrap();

                if new_entity.inner.inner.id.0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Skill ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .skill_holder
                    .get(&new_entity.inner.inner.id)
                {
                    if new_entity.inner.initial_id == old_entity.id || old_entity.deleted() {
                        self.save_skill_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmSkillSave {
                            message: format!(
                                "Skill with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            skill_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_skill_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::Weapon(index) => {
                let new_entity = self.edit_params.weapons.opened.get(index).unwrap();

                if new_entity.inner.inner.id().0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Item ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .weapon_holder
                    .get(&new_entity.inner.inner.id())
                {
                    if new_entity.inner.initial_id == old_entity.id() || old_entity.deleted() {
                        self.save_weapon_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmWeaponSave {
                            message: format!(
                                "Item with ID {} already exists.\nOverwrite?",
                                old_entity.id().0
                            ),
                            item_id: old_entity.id(),
                        });

                        return;
                    }
                } else {
                    self.save_weapon_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::EtcItem(index) => {
                let new_entity = self.edit_params.etc_items.opened.get(index).unwrap();

                if new_entity.inner.inner.id().0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Item ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .item_holder
                    .get(&new_entity.inner.inner.id())
                {
                    if new_entity.inner.initial_id == old_entity.id {
                        self.save_etc_item_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmEtcSave {
                            message: format!(
                                "Item with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            item_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_etc_item_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::Armor(index) => {
                let new_entity = self.edit_params.armor.opened.get(index).unwrap();

                if new_entity.inner.inner.id().0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Item ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .item_holder
                    .get(&new_entity.inner.inner.id())
                {
                    if new_entity.inner.initial_id == old_entity.id {
                        self.save_armor_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmArmorSave {
                            message: format!(
                                "Item with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            item_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_armor_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::ItemSet(index) => {
                let new_entity = self.edit_params.item_sets.opened.get(index).unwrap();

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .item_set_holder
                    .get(&new_entity.inner.inner.id())
                {
                    if new_entity.inner.initial_id == old_entity.id() || old_entity.deleted() {
                        self.save_item_set_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmItemSetSave {
                            message: format!(
                                "Set with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            set_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_item_set_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::Recipe(index) => {
                let new_entity = self.edit_params.recipes.opened.get(index).unwrap();

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .recipe_holder
                    .get(&new_entity.inner.inner.id())
                {
                    if new_entity.inner.initial_id == old_entity.id() || old_entity.deleted() {
                        self.save_recipe_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmRecipeSave {
                            message: format!(
                                "Recipe with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            recipe_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_recipe_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::HuntingZone(index) => {
                let new_entity = self.edit_params.hunting_zones.opened.get(index).unwrap();

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .hunting_zone_holder
                    .get(&new_entity.inner.inner.id())
                {
                    if new_entity.inner.initial_id == old_entity.id() || old_entity.deleted() {
                        self.save_hunting_zone_object_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmHuntingZoneSave {
                            message: format!(
                                "HuntingZone with with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            hunting_zone_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_hunting_zone_object_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::Region(index) => {
                let new_entity = self.edit_params.regions.opened.get(index).unwrap();

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .region_holder
                    .get(&new_entity.inner.inner.id())
                {
                    if new_entity.inner.initial_id == old_entity.id() || old_entity.deleted() {
                        self.save_region_object_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmRegionSave {
                            message: format!(
                                "Region with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            region_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_region_object_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::RaidInfo(index) => {
                let new_entity = self.edit_params.raid_info.opened.get(index).unwrap();

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .raid_info_holder
                    .get(&new_entity.inner.inner.id())
                {
                    if new_entity.inner.initial_id == old_entity.id() || old_entity.deleted() {
                        self.save_raid_info_object_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmRaidInfoSave {
                            message: format!(
                                "Raid with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            raid_info_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_raid_info_object_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::DailyMission(index) => {
                let new_entity = self.edit_params.daily_mission.opened.get(index).unwrap();

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .daily_mission_holder
                    .get(&new_entity.inner.inner.id())
                {
                    if new_entity.inner.initial_id == old_entity.id() || old_entity.deleted() {
                        self.save_daily_mission_object_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmDailyMissionSave {
                            message: format!(
                                "Daily Mission with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            daily_mission_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_daily_mission_object_force(new_entity.inner.inner.clone());
                }
            }

                CurrentEntity::AnimationCombo(index) => {
                let new_entity = self.edit_params.animation_combo.opened.get(index).unwrap();

                if let Some(old_entity) = self
                    .holders
                    .game_data_holder
                    .animation_combo_holder
                    .get_by_secondary(&new_entity.inner.inner.name())
                {
                    if new_entity.inner.initial_id == old_entity.id() || old_entity.deleted() {
                        self.save_animation_combo_object_force(new_entity.inner.inner.clone());
                    } else {
                        self.show_dialog(Dialog::ConfirmAnimationComboSave {
                            message: format!(
                                "Animation Combo with name {} already exists.\nOverwrite?",
                                old_entity.name
                            ),
                            animation_combo_id: old_entity.id,
                        });

                        return;
                    }
                } else {
                    self.save_animation_combo_object_force(new_entity.inner.inner.clone());
                }
            }

            CurrentEntity::None => {
                return;
            }
        }

        if let Some(v) = self.get_current_entity_mut() {
            v.on_save();
        }
    }

    pub fn check_for_unwrote_changed(&mut self) {
        self.has_unwrote_changes = !self.holders.game_data_holder.changed_entities().is_empty();
    }

    fn set_unchanged(&mut self) {
        self.has_unwrote_changes = false;

        self.holders.game_data_holder.npc_holder.set_changed(false);
        self.holders
            .game_data_holder
            .quest_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .skill_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .weapon_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .armor_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .etc_item_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .item_set_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .recipe_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .hunting_zone_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .region_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .raid_info_holder
            .set_changed(false);
        self.holders
            .game_data_holder
            .animation_combo_holder
            .set_changed(false);

        self.holders.game_data_holder.game_string_table.was_changed = false;
        self.holders.game_data_holder.npc_strings.set_changed(false);
    }

    pub fn is_changed(&self) -> bool {
        self.has_unwrote_changes
    }

    pub fn answer(&mut self, answer: DialogAnswer) {
        match self.dialog {
            Dialog::ConfirmQuestSave { quest_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_quest_from_dlg(quest_id);
                }
            }

            Dialog::ConfirmSkillSave { skill_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_skill_from_dlg(skill_id);
                }
            }

            Dialog::ConfirmNpcSave { npc_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_npc_from_dlg(npc_id);
                }
            }

            Dialog::ConfirmWeaponSave { item_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_weapon_from_dlg(item_id);
                }
            }

            Dialog::ConfirmEtcSave { item_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_etc_item_from_dlg(item_id);
                }
            }

            Dialog::ConfirmArmorSave { item_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_armor_from_dlg(item_id);
                }
            }

            Dialog::ConfirmItemSetSave { set_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_item_set_from_dlg(set_id);
                }
            }

            Dialog::ConfirmRecipeSave { recipe_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_recipe_from_dlg(recipe_id);
                }
            }

            Dialog::ConfirmHuntingZoneSave {
                hunting_zone_id, ..
            } => {
                if answer == DialogAnswer::Confirm {
                    self.save_hunting_zone_from_dlg(hunting_zone_id);
                }
            }

            Dialog::ConfirmRegionSave { region_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_region_from_dlg(region_id);
                }
            }

            Dialog::ConfirmRaidInfoSave { raid_info_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_raid_info_from_dlg(raid_info_id);
                }
            }

            Dialog::ConfirmDailyMissionSave {
                daily_mission_id, ..
            } => {
                if answer == DialogAnswer::Confirm {
                    self.save_daily_mission_from_dlg(daily_mission_id);
                }
            }

            Dialog::ConfirmAnimationComboSave {
                animation_combo_id, ..
            } => {
                if answer == DialogAnswer::Confirm {
                    self.save_animation_combo_from_dlg(animation_combo_id);
                }
            }

            Dialog::ShowWarning(_) => {}

            Dialog::ConfirmClose(index) => {
                if answer == DialogAnswer::Confirm {
                    self.close_entity(index, true)
                }
            }

            Dialog::None => {}
        }

        self.dialog = Dialog::None;
    }

    fn show_dialog(&mut self, dialog: Dialog) {
        self.dialog = dialog;
    }

    pub fn no_dialog(&self) -> bool {
        matches!(self.dialog, Dialog::None)
    }

    pub fn close_current_entity(&mut self) {
        self.close_entity(self.edit_params.current_entity, false);
    }

    pub fn close_entity(&mut self, ind: CurrentEntity, force: bool) {
        match ind {
            CurrentEntity::Quest(index) => {
                if !force && self.edit_params.quests.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Quest(index)));

                    return;
                }

                self.edit_params.quests.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::Skill(index) => {
                if !force && self.edit_params.skills.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Skill(index)));

                    return;
                }

                self.edit_params.skills.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::Npc(index) => {
                if !force && self.edit_params.npcs.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Npc(index)));

                    return;
                }

                self.edit_params.npcs.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::Weapon(index) => {
                if !force && self.edit_params.weapons.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Weapon(index)));

                    return;
                }

                self.edit_params.weapons.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::EtcItem(index) => {
                if !force && self.edit_params.etc_items.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::EtcItem(index)));

                    return;
                }

                self.edit_params.etc_items.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::Armor(index) => {
                if !force && self.edit_params.armor.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Armor(index)));

                    return;
                }

                self.edit_params.armor.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::ItemSet(index) => {
                if !force && self.edit_params.item_sets.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::ItemSet(index)));

                    return;
                }

                self.edit_params.item_sets.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::Recipe(index) => {
                if !force && self.edit_params.recipes.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Recipe(index)));

                    return;
                }

                self.edit_params.recipes.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::HuntingZone(index) => {
                if !force && self.edit_params.hunting_zones.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::HuntingZone(index)));

                    return;
                }

                self.edit_params.hunting_zones.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::Region(index) => {
                if !force && self.edit_params.regions.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Region(index)));

                    return;
                }

                self.edit_params.regions.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::RaidInfo(index) => {
                if !force && self.edit_params.raid_info.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::RaidInfo(index)));

                    return;
                }

                self.edit_params.raid_info.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::DailyMission(index) => {
                if !force && self.edit_params.daily_mission.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::DailyMission(index)));

                    return;
                }

                self.edit_params.daily_mission.opened.remove(index);

                self.edit_params.find_opened_entity();
            }
            CurrentEntity::AnimationCombo(index) => {
                if !force && self.edit_params.animation_combo.opened[index].is_changed() {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::AnimationCombo(index)));

                    return;
                }

                self.edit_params.animation_combo.opened.remove(index);

                self.edit_params.find_opened_entity();
            }

            CurrentEntity::None => {}
        }
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub system_folder_path: Option<String>,
    pub textures_folder_path: Option<String>,
    pub server_quests_java_classes_path: Option<String>,
    pub server_spawn_root_folder_path: Option<String>,
}

impl Config {
    fn dump(&self) {
        let mut f = File::create(Path::new(CONFIG_FILE_NAME)).unwrap();
        f.write_all(
            (ron::ser::to_string_pretty::<Config>(self, ron::ser::PrettyConfig::default())
                .unwrap())
            .as_ref(),
        )
        .unwrap();
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

struct Tasks {
    last_auto_save: SystemTime,
    last_change_check: SystemTime,
}

impl Tasks {
    fn init() -> Self {
        Self {
            last_auto_save: SystemTime::now(),
            last_change_check: SystemTime::now(),
        }
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Eq, PartialEq)]
pub enum DialogAnswer {
    Confirm,
    Abort,
}

pub enum Dialog {
    None,
    ConfirmQuestSave {
        message: String,
        quest_id: QuestId,
    },
    ConfirmSkillSave {
        message: String,
        skill_id: SkillId,
    },
    ConfirmNpcSave {
        message: String,
        npc_id: NpcId,
    },
    ConfirmWeaponSave {
        message: String,
        item_id: ItemId,
    },
    ConfirmEtcSave {
        message: String,
        item_id: ItemId,
    },
    ConfirmArmorSave {
        message: String,
        item_id: ItemId,
    },
    ConfirmItemSetSave {
        message: String,
        set_id: ItemSetId,
    },
    ConfirmRecipeSave {
        message: String,
        recipe_id: RecipeId,
    },
    ConfirmHuntingZoneSave {
        message: String,
        hunting_zone_id: HuntingZoneId,
    },
    ConfirmRegionSave {
        message: String,
        region_id: RegionId,
    },
    ConfirmRaidInfoSave {
        message: String,
        raid_info_id: RaidInfoId,
    },
    ConfirmDailyMissionSave {
        message: String,
        daily_mission_id: DailyMissionId,
    },
    ConfirmAnimationComboSave {
        message: String,
        animation_combo_id: AnimationComboId,
    },

    ShowWarning(String),
    ConfirmClose(CurrentEntity),
}

impl Dialog {
    pub fn is_none(&self) -> bool {
        matches!(self, Dialog::None)
    }
}

pub trait HandleAction {
    fn handle_action(&mut self);
}
