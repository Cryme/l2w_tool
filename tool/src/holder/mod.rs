use crate::backend::Config;
use crate::data::{HuntingZoneId, ItemId, ItemSetId, NpcId, QuestId, RecipeId, SkillId};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use crate::entity::item::Item;
use crate::entity::item_set::ItemSet;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::recipe::Recipe;
use crate::entity::skill::Skill;
use crate::holder::grand_crusade_110::{L2GeneralStringTable, Loader110};
use std::collections::hash_map::{Keys, Values};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

mod grand_crusade_110;

pub trait Loader {
    fn load(&mut self, dat_paths: HashMap<String, DirEntry>) -> Result<(), ()>;
    fn from_holder(game_data_holder: &GameDataHolder) -> Self;
    fn to_holder(self) -> GameDataHolder;
    fn serialize_to_binary(&mut self) -> std::io::Result<()>;
}

fn get_loader_for_protocol(protocol: ChroniclesProtocol) -> Result<impl Loader + Sized, ()> {
    Ok(match protocol {
        ChroniclesProtocol::GrandCrusade110 => Loader110::default(),
    })
}

pub fn get_loader_from_holder(holder: &GameDataHolder) -> impl Loader + Sized {
    match holder.protocol_version {
        ChroniclesProtocol::GrandCrusade110 => Loader110::from_holder(holder),
    }
}

pub fn load_game_data_holder(
    path: &str,
    protocol: ChroniclesProtocol,
) -> Result<GameDataHolder, ()> {
    let mut dat_paths = HashMap::new();

    for path in WalkDir::new(path).into_iter().flatten() {
        if let Ok(meta) = path.metadata() {
            if meta.is_file() && path.file_name().to_str().unwrap().ends_with(".dat") {
                dat_paths.insert(path.file_name().to_str().unwrap().to_lowercase(), path);
            }
        }
    }

    let mut loader = get_loader_for_protocol(protocol)?;
    loader.load(dat_paths)?;

    Ok(loader.to_holder())
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum ChroniclesProtocol {
    #[default]
    GrandCrusade110,
}

#[derive(Default)]
pub struct GameDataHolder {
    pub protocol_version: ChroniclesProtocol,
    pub initial_dat_paths: HashMap<String, DirEntry>,

    pub npc_holder: FHashMap<NpcId, Npc>,
    pub quest_holder: FHashMap<QuestId, Quest>,
    pub skill_holder: FHashMap<SkillId, Skill>,

    pub item_holder: HashMap<ItemId, Item>,
    pub weapon_holder: FHashMap<ItemId, Weapon>,
    pub armor_holder: FHashMap<ItemId, Armor>,
    pub etc_item_holder: FHashMap<ItemId, EtcItem>,

    pub item_set_holder: FHashMap<ItemSetId, ItemSet>,
    pub recipe_holder: FHashMap<RecipeId, Recipe>,

    pub npc_strings: FHashMap<u32, String>,
    pub hunting_zone_holder: FHashMap<HuntingZoneId, HuntingZone>,
    pub game_string_table: L2GeneralStringTable,
}

impl GameDataHolder {
    pub fn validate_paths(config: &mut Config) {
        if let Some(path) = &config.system_folder_path {
            if !Path::new(path).is_dir() {
                config.system_folder_path = None
            }
        }
    }

    pub fn get_npc_name(&self, id: &NpcId) -> String {
        if let Some(npc) = self.npc_holder.get(id) {
            npc.name.clone()
        } else {
            format!("{id:?} Not Exist!")
        }
    }

    pub fn get_item_name(&self, id: &ItemId) -> String {
        if let Some(item) = self.item_holder.get(id) {
            item.name.clone()
        } else {
            format!("{id:?} Not Exist!")
        }
    }
}

#[derive(Clone)]
pub struct FHashMap<K: Hash + Eq, V> {
    was_changed: bool,
    inner: HashMap<K, V>,
}

impl<K: Hash + Eq + Clone, V: Clone> FHashMap<K, V> {
    pub fn changed_or_empty(&self) -> FHashMap<K, V> {
        if self.was_changed {
            (*self).clone()
        } else {
            Self::new()
        }
    }
}

impl<K: Hash + Eq, V> Default for FHashMap<K, V> {
    fn default() -> Self {
        Self {
            was_changed: false,
            inner: HashMap::new(),
        }
    }
}

#[allow(unused)]
impl<K: Hash + Eq, V> FHashMap<K, V> {
    pub fn new() -> FHashMap<K, V> {
        Self {
            was_changed: false,
            inner: HashMap::new(),
        }
    }
    pub fn keys(&self) -> Keys<K, V> {
        self.inner.keys()
    }

    pub fn values(&self) -> Values<K, V> {
        self.inner.values()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: K, val: V) {
        self.inner.insert(key, val);
        self.was_changed = true;
    }

    pub fn is_unchanged(&self) -> bool {
        !self.was_changed
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}
