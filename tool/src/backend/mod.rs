pub mod hunting_zone;
pub mod item;
pub mod item_set;
pub mod npc;
pub mod quest;
pub mod recipe;
pub mod region;
pub mod skill;

use crate::backend::hunting_zone::{HuntingZoneEditor, HuntingZoneInfo};
use crate::backend::item::armor::{ArmorEditor, ArmorInfo};
use crate::backend::item::etc_item::{EtcItemEditor, EtcItemInfo};
use crate::backend::item::weapon::{WeaponEditor, WeaponInfo};
use crate::backend::item_set::{ItemSetEditor, ItemSetInfo};
use crate::backend::npc::{NpcEditor, NpcInfo};
use crate::backend::quest::{QuestEditor, QuestInfo};
use crate::backend::recipe::{RecipeEditor, RecipeInfo};
use crate::backend::region::{RegionEditor, RegionInfo};
use crate::backend::skill::{SkillEditor, SkillInfo};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{Duration, SystemTime};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::dat_loader::DatLoader;
use crate::dat_loader::{get_loader_from_holder, load_game_data_holder};
use crate::data::{HuntingZoneId, ItemId, ItemSetId, NpcId, QuestId, RecipeId, RegionId, SkillId};
use crate::entity::{CommonEntity, Entity};
use crate::holder::{ChroniclesProtocol, DataHolder, FHashMap, GameDataHolder};
use crate::server_side::ServerDataHolder;
use crate::{logs_mut, VERSION};

const AUTO_SAVE_INTERVAL: Duration = Duration::from_secs(30);
const CHANGE_CHECK_INTERVAL: Duration = Duration::from_millis(500);
const CONFIG_FILE_NAME: &str = "./config.ron";

pub struct Backend {
    pub config: Config,
    pub holders: DataHolder,
    pub filter_params: FilterParams,
    pub dialog: Dialog,
    pub dialog_showing: bool,
    pub edit_params: EditParams,

    has_unsaved_changes: bool,

    pub logs: WindowParams<LogHolderParams, (), (), ()>,

    tasks: Tasks,
}

trait EditParamsCommonOps {
    fn is_changed(&self) -> bool;
    fn update_initial(&mut self);
    fn check_change(&mut self);
    fn handle_actions(&mut self);
}

impl<Entity: PartialEq + Clone, EntityId, EditAction, EditParams> EditParamsCommonOps
    for ChangeTrackedParams<Entity, EntityId, EditAction, EditParams>
where
    WindowParams<Entity, EntityId, EditAction, EditParams>: HandleAction,
{
    fn is_changed(&self) -> bool {
        self.changed
    }

    fn update_initial(&mut self) {
        self.changed = false;
        self.initial = self.inner.inner.clone();
    }

    fn check_change(&mut self) {
        self.changed = self.inner.inner != self.initial;
    }

    fn handle_actions(&mut self) {
        self.inner.handle_action()
    }
}

impl Backend {
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

            CurrentEntity::None => None,
        }
    }
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
            filter_params: FilterParams::default(),
            dialog: Dialog::None,

            dialog_showing: false,
            has_unsaved_changes: false,

            tasks: Tasks::init(),
            edit_params,
            logs: WindowParams::default(),
        };

        r.update_last_ids();

        r
    }

    pub(crate) fn save_to_dat(&mut self) {
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

    fn update_last_ids(&mut self) {
        self.filter_params.quest_filter_string = "".to_string();
        self.filter_quests();
        self.filter_params.skill_filter_string = "".to_string();
        self.filter_skills();
        self.filter_params.npc_filter_string = "".to_string();
        self.filter_npcs();
        self.filter_params.weapon_filter_string = "".to_string();
        self.filter_weapons();
        self.filter_params.etc_item_filter_string = "".to_string();
        self.filter_etc_items();
        self.filter_params.armor_filter_string = "".to_string();
        self.filter_armor();
        self.filter_params.item_set_filter_string = "".to_string();
        self.filter_item_sets();
        self.filter_params.recipe_filter_string = "".to_string();
        self.filter_recipes();
        self.filter_params.hunting_zone_filter_string = "".to_string();
        self.filter_hunting_zones();
        self.filter_params.region_filter_string = "".to_string();
        self.filter_regions();

        self.edit_params.quests.next_id =
            if let Some(last) = self.filter_params.quest_catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.skills.next_id =
            if let Some(last) = self.filter_params.skill_catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.npcs.next_id = if let Some(last) = self.filter_params.npc_catalog.last() {
            last.id.0 + 1
        } else {
            0
        };

        self.edit_params.weapons.next_id =
            if let Some(last) = self.filter_params.weapon_catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.armor.next_id = if let Some(last) = self.filter_params.armor_catalog.last()
        {
            last.id.0 + 1
        } else {
            0
        };

        self.edit_params.etc_items.next_id =
            if let Some(last) = self.filter_params.etc_item_catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.item_sets.next_id =
            if let Some(last) = self.filter_params.item_set_catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.recipes.next_id =
            if let Some(last) = self.filter_params.recipe_catalog.last() {
                last.id.0 + 1
            } else {
                0
            };

        self.edit_params.hunting_zones.next_id =
            if let Some(last) = self.filter_params.hunting_zone_catalog.last() {
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

    pub fn is_current_entity_changed(&self) -> bool {
        if let Some(v) = self.get_current_entity() {
            v.is_changed()
        } else {
            false
        }
    }

    pub fn fill_current_entity_from_ron(&mut self, val: &str) {
        match self.edit_params.current_entity {
            CurrentEntity::Quest(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.quests.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }
            CurrentEntity::Skill(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.skills.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }
            CurrentEntity::Npc(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.npcs.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }
            CurrentEntity::Weapon(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.weapons.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }
            CurrentEntity::EtcItem(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.etc_items.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }
            CurrentEntity::Armor(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.armor.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }
            CurrentEntity::ItemSet(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.item_sets.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }
            CurrentEntity::Recipe(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.recipes.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }
            CurrentEntity::HuntingZone(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.hunting_zones.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }
            CurrentEntity::Region(i) => {
                let r = ron::from_str(val);

                if let Ok(c) = r {
                    self.edit_params.regions.opened[i].inner = c;
                } else {
                    self.show_dialog(Dialog::ShowWarning(format!("{r:?}")));
                }
            }

            CurrentEntity::None => {}
        }
    }

    pub fn current_entity_as_ron(&self) -> Option<String> {
        match self.edit_params.current_entity {
            CurrentEntity::Quest(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.quests.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),
            CurrentEntity::Npc(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.npcs.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),
            CurrentEntity::Skill(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.skills.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),
            CurrentEntity::Weapon(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.weapons.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),
            CurrentEntity::EtcItem(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.etc_items.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),
            CurrentEntity::Armor(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.armor.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),
            CurrentEntity::ItemSet(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.item_sets.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),
            CurrentEntity::Recipe(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.recipes.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),
            CurrentEntity::HuntingZone(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.hunting_zones.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),
            CurrentEntity::Region(i) => Some(
                ron::ser::to_string_pretty(
                    &self.edit_params.regions.opened[i].inner,
                    PrettyConfig::default().struct_names(true),
                )
                .unwrap(),
            ),

            CurrentEntity::None => None,
        }
    }

    pub fn save_current_entity(&mut self) {
        if !self.is_current_entity_changed() {
            return;
        }

        match self.edit_params.current_entity {
            CurrentEntity::Npc(index) => {
                let new_entity = self.edit_params.npcs.opened.get(index).unwrap();

                if new_entity.inner.inner.id.0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Npc ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_npc) = self
                    .holders
                    .game_data_holder
                    .npc_holder
                    .get(&new_entity.inner.inner.id)
                {
                    if new_entity.inner.initial_id == new_entity.inner.inner.id {
                        self.save_npc_force(new_entity.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmNpcSave {
                            message: format!(
                                "Npc with ID {} already exists.\nOverwrite?",
                                old_npc.id.0
                            ),
                            npc_id: old_npc.id,
                        });
                    }
                } else {
                    self.save_npc_force(new_entity.inner.inner.clone());
                    self.set_changed();
                }
            }

            CurrentEntity::Quest(index) => {
                let new_quest = self.edit_params.quests.opened.get(index).unwrap();

                if new_quest.inner.inner.id.0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Quest ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_quest) = self
                    .holders
                    .game_data_holder
                    .quest_holder
                    .get(&new_quest.inner.inner.id)
                {
                    if new_quest.inner.initial_id == new_quest.inner.inner.id {
                        self.save_quest_force(new_quest.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmQuestSave {
                            message: format!(
                                "Quest with ID {} already exists.\nOverwrite?",
                                old_quest.id.0
                            ),
                            quest_id: old_quest.id,
                        });
                    }
                } else {
                    self.save_quest_force(new_quest.inner.inner.clone());
                    self.set_changed();
                }
            }

            CurrentEntity::Skill(index) => {
                let new_skill = self.edit_params.skills.opened.get(index).unwrap();

                if new_skill.inner.inner.id.0 == 0 {
                    self.show_dialog(Dialog::ShowWarning("Skill ID can't be 0!".to_string()));

                    return;
                }

                if let Some(old_skill) = self
                    .holders
                    .game_data_holder
                    .skill_holder
                    .get(&new_skill.inner.inner.id)
                {
                    if new_skill.inner.initial_id == new_skill.inner.inner.id {
                        self.save_skill_force(new_skill.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmSkillSave {
                            message: format!(
                                "Skill with ID {} already exists.\nOverwrite?",
                                old_skill.id.0
                            ),
                            skill_id: old_skill.id,
                        });
                    }
                } else {
                    self.save_skill_force(new_skill.inner.inner.clone());
                    self.set_changed();
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
                    if new_entity.inner.initial_id == new_entity.inner.inner.id() {
                        self.save_weapon_force(new_entity.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmWeaponSave {
                            message: format!(
                                "Item with ID {} already exists.\nOverwrite?",
                                old_entity.id().0
                            ),
                            item_id: old_entity.id(),
                        });
                    }
                } else {
                    self.save_weapon_force(new_entity.inner.inner.clone());
                    self.set_changed();
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
                    if new_entity.inner.initial_id == new_entity.inner.inner.id() {
                        self.save_etc_item_force(new_entity.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmEtcSave {
                            message: format!(
                                "Item with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            item_id: old_entity.id,
                        });
                    }
                } else {
                    self.save_etc_item_force(new_entity.inner.inner.clone());
                    self.set_changed();
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
                    if new_entity.inner.initial_id == new_entity.inner.inner.id() {
                        self.save_armor_force(new_entity.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmArmorSave {
                            message: format!(
                                "Item with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            item_id: old_entity.id,
                        });
                    }
                } else {
                    self.save_armor_force(new_entity.inner.inner.clone());
                    self.set_changed();
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
                    if new_entity.inner.initial_id == new_entity.inner.inner.id() {
                        self.save_item_set_force(new_entity.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmItemSetSave {
                            message: format!(
                                "Set with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            set_id: old_entity.id,
                        });
                    }
                } else {
                    self.save_item_set_force(new_entity.inner.inner.clone());
                    self.set_changed();
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
                    if new_entity.inner.initial_id == new_entity.inner.inner.id() {
                        self.save_recipe_force(new_entity.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmRecipeSave {
                            message: format!(
                                "Recipe with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            recipe_id: old_entity.id,
                        });
                    }
                } else {
                    self.save_recipe_force(new_entity.inner.inner.clone());
                    self.set_changed();
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
                    if new_entity.inner.initial_id == new_entity.inner.inner.id() {
                        self.save_hunting_zone_object_force(new_entity.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmHuntingZoneSave {
                            message: format!(
                                "Map Object with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            hunting_zone_id: old_entity.id,
                        });
                    }
                } else {
                    self.save_hunting_zone_object_force(new_entity.inner.inner.clone());
                    self.set_changed();
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
                    if new_entity.inner.initial_id == new_entity.inner.inner.id() {
                        self.save_region_object_force(new_entity.inner.inner.clone());
                        self.set_changed();
                    } else {
                        self.show_dialog(Dialog::ConfirmRegionSave {
                            message: format!(
                                "Map Object with ID {} already exists.\nOverwrite?",
                                old_entity.id.0
                            ),
                            region_id: old_entity.id,
                        });
                    }
                } else {
                    self.save_region_object_force(new_entity.inner.inner.clone());
                    self.set_changed();
                }
            }

            CurrentEntity::None => {
                return;
            }
        }

        if let Some(v) = self.get_current_entity_mut() {
            v.update_initial();
        }
    }

    fn set_changed(&mut self) {
        self.has_unsaved_changes = true;
    }
    fn set_unchanged(&mut self) {
        self.has_unsaved_changes = false;
    }
    pub fn is_changed(&self) -> bool {
        self.has_unsaved_changes
    }

    pub fn answer(&mut self, answer: DialogAnswer) {
        match self.dialog {
            Dialog::ConfirmQuestSave { quest_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_quest_from_dlg(quest_id);
                    self.set_changed();
                }
            }

            Dialog::ConfirmSkillSave { skill_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_skill_from_dlg(skill_id);
                    self.set_changed();
                }
            }

            Dialog::ConfirmNpcSave { npc_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_npc_from_dlg(npc_id);
                    self.set_changed();
                }
            }

            Dialog::ConfirmWeaponSave { item_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_weapon_from_dlg(item_id);
                    self.set_changed();
                }
            }

            Dialog::ConfirmEtcSave { item_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_etc_item_from_dlg(item_id);
                    self.set_changed();
                }
            }

            Dialog::ConfirmArmorSave { item_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_armor_from_dlg(item_id);
                    self.set_changed();
                }
            }

            Dialog::ConfirmItemSetSave { set_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_item_set_from_dlg(set_id);
                    self.set_changed();
                }
            }

            Dialog::ConfirmRecipeSave { recipe_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_recipe_from_dlg(recipe_id);
                    self.set_changed();
                }
            }

            Dialog::ConfirmHuntingZoneSave {
                hunting_zone_id, ..
            } => {
                if answer == DialogAnswer::Confirm {
                    self.save_hunting_zone_from_dlg(hunting_zone_id);
                    self.set_changed();
                }
            }

            Dialog::ConfirmRegionSave { region_id, .. } => {
                if answer == DialogAnswer::Confirm {
                    self.save_region_from_dlg(region_id);
                    self.set_changed();
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
        match self.dialog {
            Dialog::None => true,
            _ => false,
        }
    }

    pub fn close_entity(&mut self, ind: CurrentEntity, force: bool) {
        match ind {
            CurrentEntity::Quest(index) => {
                if !force && self.edit_params.quests.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Quest(index)));

                    return;
                }

                self.edit_params.quests.opened.remove(index);

                if let CurrentEntity::Quest(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.quests.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity =
                            CurrentEntity::Quest(curr_index.max(1) - 1)
                    }
                }
            }
            CurrentEntity::Skill(index) => {
                if !force && self.edit_params.skills.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Skill(index)));

                    return;
                }

                self.edit_params.skills.opened.remove(index);

                if let CurrentEntity::Skill(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.skills.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity =
                            CurrentEntity::Skill(curr_index.max(1) - 1)
                    }
                }
            }
            CurrentEntity::Npc(index) => {
                if !force && self.edit_params.npcs.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Npc(index)));

                    return;
                }

                self.edit_params.npcs.opened.remove(index);

                if let CurrentEntity::Npc(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.npcs.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity = CurrentEntity::Npc(curr_index.max(1) - 1)
                    }
                }
            }
            CurrentEntity::Weapon(index) => {
                if !force && self.edit_params.weapons.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Weapon(index)));

                    return;
                }

                self.edit_params.weapons.opened.remove(index);

                if let CurrentEntity::Weapon(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.weapons.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity =
                            CurrentEntity::Weapon(curr_index.max(1) - 1)
                    }
                }
            }
            CurrentEntity::EtcItem(index) => {
                if !force && self.edit_params.etc_items.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::EtcItem(index)));

                    return;
                }

                self.edit_params.etc_items.opened.remove(index);

                if let CurrentEntity::EtcItem(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.etc_items.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity =
                            CurrentEntity::EtcItem(curr_index.max(1) - 1)
                    }
                }
            }
            CurrentEntity::Armor(index) => {
                if !force && self.edit_params.armor.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Armor(index)));

                    return;
                }

                self.edit_params.armor.opened.remove(index);

                if let CurrentEntity::Armor(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.armor.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity =
                            CurrentEntity::Armor(curr_index.max(1) - 1)
                    }
                }
            }
            CurrentEntity::ItemSet(index) => {
                if !force && self.edit_params.item_sets.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::ItemSet(index)));

                    return;
                }

                self.edit_params.item_sets.opened.remove(index);

                if let CurrentEntity::ItemSet(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.item_sets.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity =
                            CurrentEntity::ItemSet(curr_index.max(1) - 1)
                    }
                }
            }
            CurrentEntity::Recipe(index) => {
                if !force && self.edit_params.recipes.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Recipe(index)));

                    return;
                }

                self.edit_params.recipes.opened.remove(index);

                if let CurrentEntity::Recipe(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.recipes.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity =
                            CurrentEntity::Recipe(curr_index.max(1) - 1)
                    }
                }
            }
            CurrentEntity::HuntingZone(index) => {
                if !force && self.edit_params.hunting_zones.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::HuntingZone(index)));

                    return;
                }

                self.edit_params.hunting_zones.opened.remove(index);

                if let CurrentEntity::HuntingZone(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.hunting_zones.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity =
                            CurrentEntity::HuntingZone(curr_index.max(1) - 1)
                    }
                }
            }
            CurrentEntity::Region(index) => {
                if !force && self.edit_params.regions.opened[index].changed {
                    self.edit_params.current_entity = ind;
                    self.show_dialog(Dialog::ConfirmClose(CurrentEntity::Region(index)));

                    return;
                }

                self.edit_params.regions.opened.remove(index);

                if let CurrentEntity::Region(curr_index) = self.edit_params.current_entity {
                    if self.edit_params.regions.opened.is_empty() {
                        self.edit_params.find_opened_entity();
                    } else if curr_index >= index {
                        self.edit_params.current_entity =
                            CurrentEntity::Region(curr_index.max(1) - 1)
                    }
                }
            }

            CurrentEntity::None => {}
        }
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Debug)]
pub struct LogHolder {
    pub(crate) producers: HashSet<String>,
    pub(crate) max_level: LogLevel,
    pub(crate) synchronized: bool,

    pub(crate) logs: Vec<Log>,
}

impl LogHolder {
    pub const ALL: &'static str = "All";

    pub(crate) fn new() -> Self {
        let mut producers = HashSet::new();
        producers.insert(Self::ALL.to_string());

        Self {
            producers,
            max_level: LogLevel::Info,
            synchronized: false,
            logs: vec![],
        }
    }

    pub fn reset(&mut self, logs: Vec<Log>) {
        let mut producers = HashSet::new();
        producers.insert(Self::ALL.to_string());

        let mut max_level = LogLevel::Info;

        for v in &logs {
            producers.insert(v.producer.clone());
            max_level = v.level.max(max_level);
        }

        *self = LogHolder {
            producers,
            max_level,
            synchronized: false,
            logs,
        };
    }

    pub fn add(&mut self, log: Log) {
        self.producers.insert(log.producer.clone());
        self.max_level = log.level.max(self.max_level);
        self.logs.push(log);
        self.synchronized = false;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Display, EnumIter)]
#[repr(u8)]
pub enum LogLevelFilter {
    Info,
    Warning,
    Error,
    All = 255,
}

#[derive(Clone, Debug)]
pub struct Log {
    pub level: LogLevel,
    pub producer: String,
    pub log: String,
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

pub struct LogHolderParams {
    pub(crate) producer_filter: String,
    pub(crate) producers: HashSet<String>,
    pub(crate) max_log_level: LogLevel,
    pub(crate) level_filter: LogLevelFilter,
}

impl Default for LogHolderParams {
    fn default() -> Self {
        LogHolderParams {
            producer_filter: LogHolder::ALL.to_string(),
            producers: logs_mut().producers.clone(),
            max_log_level: LogLevel::Info,
            level_filter: LogLevelFilter::All,
        }
    }
}

impl LogHolderParams {
    fn sync(&mut self) {
        let mut v = logs_mut();

        if v.synchronized {
            return;
        }

        self.producers = v.producers.clone();
        self.max_log_level = v.max_level;

        v.synchronized = true;

        drop(v);

        if !self.producers.contains(&self.producer_filter) {
            self.producer_filter = LogHolder::ALL.to_string();
        }
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowParams<Inner, InitialId, Action, Params> {
    pub(crate) inner: Inner,
    pub(crate) opened: bool,
    pub(crate) initial_id: InitialId,
    pub(crate) action: RwLock<Action>,
    pub(crate) params: Params,
}

impl<Inner: PartialEq, InitialId, Action, Params> PartialEq
    for WindowParams<Inner, InitialId, Action, Params>
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<Inner: Clone, OriginalId: Clone, Action: Default, Params: Clone> Clone
    for WindowParams<Inner, OriginalId, Action, Params>
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            opened: false,
            initial_id: self.initial_id.clone(),
            action: RwLock::new(Action::default()),
            params: self.params.clone(),
        }
    }
}

impl<T: Default, Action: Default, InitialId: Default, Params: Default> Default
    for WindowParams<T, InitialId, Action, Params>
{
    fn default() -> Self {
        Self {
            inner: T::default(),
            opened: false,
            initial_id: InitialId::default(),
            action: RwLock::new(Action::default()),
            params: Params::default(),
        }
    }
}

impl<T, Action: Default, InitialId: Default, Params: Default>
    WindowParams<T, InitialId, Action, Params>
{
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            opened: false,
            initial_id: InitialId::default(),
            action: RwLock::new(Action::default()),
            params: Params::default(),
        }
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Default)]
pub struct FilterParams {
    pub npc_filter_string: String,
    pub npc_catalog: Vec<NpcInfo>,

    pub quest_filter_string: String,
    pub quest_catalog: Vec<QuestInfo>,

    pub skill_filter_string: String,
    pub skill_catalog: Vec<SkillInfo>,

    pub weapon_filter_string: String,
    pub weapon_catalog: Vec<WeaponInfo>,

    pub etc_item_filter_string: String,
    pub etc_item_catalog: Vec<EtcItemInfo>,

    pub armor_filter_string: String,
    pub armor_catalog: Vec<ArmorInfo>,

    pub item_set_filter_string: String,
    pub item_set_catalog: Vec<ItemSetInfo>,

    pub recipe_filter_string: String,
    pub recipe_catalog: Vec<RecipeInfo>,

    pub hunting_zone_filter_string: String,
    pub hunting_zone_catalog: Vec<HuntingZoneInfo>,

    pub region_filter_string: String,
    pub region_catalog: Vec<RegionInfo>,
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Serialize, Deserialize, Default, Eq, PartialEq, Copy, Clone)]
pub enum CurrentEntity {
    #[default]
    None,
    Quest(usize),
    Skill(usize),
    Npc(usize),
    Weapon(usize),
    EtcItem(usize),
    Armor(usize),
    ItemSet(usize),
    Recipe(usize),
    HuntingZone(usize),
    Region(usize),
}

impl CurrentEntity {
    pub fn is_some(&self) -> bool {
        *self != CurrentEntity::None
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Serialize, Deserialize)]
pub struct ChangeTrackedParams<Entity, EntityId, EditAction, EditParams> {
    pub inner: WindowParams<Entity, EntityId, EditAction, EditParams>,
    pub initial: Entity,
    pub changed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EntityEditParams<Entity, EntityId, EditAction, EditParams> {
    next_id: u32,
    pub opened: Vec<ChangeTrackedParams<Entity, EntityId, EditAction, EditParams>>,
}

impl<Entity, EntityId, EditAction, EditParams> Default
    for EntityEditParams<Entity, EntityId, EditAction, EditParams>
{
    fn default() -> Self {
        Self {
            next_id: 0,
            opened: vec![],
        }
    }
}

pub trait CommonEditorOps<Entity, EntityId: Hash + Eq, Action, Params> {
    fn reset_initial(&mut self, map: &FHashMap<EntityId, Entity>);
    fn get_opened_info(&self) -> Vec<(String, EntityId, bool)>;
    fn open(&mut self, id: EntityId, holder: &mut FHashMap<EntityId, Entity>) -> Option<usize>;
    fn add(&mut self, e: Entity, original_id: EntityId) -> usize;
    fn add_new(&mut self) -> usize;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<
        Entity: CommonEntity<EntityId, EditParams> + Clone,
        EntityId: From<u32> + Copy + Clone + Hash + Eq,
        EditAction: Default,
        EditParams,
    > CommonEditorOps<Entity, EntityId, EditAction, EditParams>
    for EntityEditParams<Entity, EntityId, EditAction, EditParams>
{
    fn reset_initial(&mut self, map: &FHashMap<EntityId, Entity>) {
        for v in &mut self.opened {
            if let Some(ent) = map.get(&v.inner.initial_id) {
                v.initial = ent.clone();
            }
        }
    }

    fn get_opened_info(&self) -> Vec<(String, EntityId, bool)> {
        self.opened
            .iter()
            .map(|v| (v.inner.inner.name(), v.inner.inner.id(), v.changed))
            .collect()
    }

    fn open(&mut self, id: EntityId, holder: &mut FHashMap<EntityId, Entity>) -> Option<usize> {
        for (i, q) in self.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                return Some(i);
            }
        }

        if let Some(q) = holder.get(&id) {
            return Some(self.add(q.clone(), q.id()));
        }

        None
    }

    fn add(&mut self, e: Entity, original_id: EntityId) -> usize {
        self.opened.push(ChangeTrackedParams {
            initial: e.clone(),
            inner: WindowParams {
                params: e.edit_params(),
                inner: e,
                initial_id: original_id,
                opened: false,
                action: RwLock::new(Default::default()),
            },
            changed: false,
        });

        self.opened.len() - 1
    }

    fn add_new(&mut self) -> usize {
        let id = EntityId::from(self.next_id);
        self.add(Entity::new(id), id);

        self.next_id += 1;

        self.opened.len() - 1
    }

    fn len(&self) -> usize {
        self.opened.len()
    }

    fn is_empty(&self) -> bool {
        self.opened.is_empty()
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Serialize, Deserialize, Default)]
pub struct EditParams {
    pub npcs: NpcEditor,
    pub quests: QuestEditor,
    pub skills: SkillEditor,
    pub weapons: WeaponEditor,
    pub armor: ArmorEditor,
    pub etc_items: EtcItemEditor,
    pub item_sets: ItemSetEditor,
    pub recipes: RecipeEditor,
    pub hunting_zones: HuntingZoneEditor,
    pub regions: RegionEditor,

    pub current_entity: CurrentEntity,
}

impl EditParams {
    fn reset_initial(&mut self, entity: Entity, holders: &GameDataHolder) {
        match entity {
            Entity::Quest => self.quests.reset_initial(&holders.quest_holder),
            Entity::Skill => self.skills.reset_initial(&holders.skill_holder),
            Entity::Npc => self.npcs.reset_initial(&holders.npc_holder),
            Entity::Weapon => self.weapons.reset_initial(&holders.weapon_holder),
            Entity::Armor => self.armor.reset_initial(&holders.armor_holder),
            Entity::EtcItem => self.etc_items.reset_initial(&holders.etc_item_holder),
            Entity::ItemSet => self.item_sets.reset_initial(&holders.item_set_holder),
            Entity::Recipe => self.recipes.reset_initial(&holders.recipe_holder),
            Entity::HuntingZone => self
                .hunting_zones
                .reset_initial(&holders.hunting_zone_holder),
            Entity::Region => self.regions.reset_initial(&holders.region_holder),
        }
    }
    fn find_opened_entity(&mut self) {
        if !self.quests.is_empty() {
            self.current_entity = CurrentEntity::Quest(self.quests.len() - 1);
        } else if !self.skills.is_empty() {
            self.current_entity = CurrentEntity::Skill(self.skills.len() - 1);
        } else if !self.npcs.is_empty() {
            self.current_entity = CurrentEntity::Npc(self.npcs.len() - 1);
        } else if !self.weapons.is_empty() {
            self.current_entity = CurrentEntity::Weapon(self.weapons.len() - 1);
        } else if !self.armor.is_empty() {
            self.current_entity = CurrentEntity::Armor(self.armor.len() - 1);
        } else if !self.etc_items.is_empty() {
            self.current_entity = CurrentEntity::EtcItem(self.etc_items.len() - 1);
        } else if !self.item_sets.is_empty() {
            self.current_entity = CurrentEntity::ItemSet(self.item_sets.len() - 1);
        } else if !self.recipes.is_empty() {
            self.current_entity = CurrentEntity::Recipe(self.recipes.len() - 1);
        } else if !self.hunting_zones.is_empty() {
            self.current_entity = CurrentEntity::HuntingZone(self.hunting_zones.len() - 1);
        } else if !self.regions.is_empty() {
            self.current_entity = CurrentEntity::Region(self.regions.len() - 1);
        } else {
            self.current_entity = CurrentEntity::None;
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
