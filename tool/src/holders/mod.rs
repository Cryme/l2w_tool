use crate::backend::Config;
use crate::data::{HuntingZoneId, ItemId, NpcId, QuestId, SkillId};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::skill::Skill;
use crate::holders::grand_crusade_110::{L2GeneralStringTable, Loader110};
use std::collections::hash_map::{Keys, Values};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

mod grand_crusade_110;

pub trait Loader {
    fn get_quests(&self) -> FHashMap<QuestId, Quest>;
    fn get_skills(&self) -> FHashMap<SkillId, Skill>;
    fn get_npcs(&self) -> FHashMap<NpcId, Npc>;
    fn get_npc_strings(&self) -> FHashMap<u32, String>;
    fn get_items(&self) -> FHashMap<ItemId, Item>;
    fn get_hunting_zones(&self) -> FHashMap<HuntingZoneId, HuntingZone>;
    fn get_string_table(&self) -> L2GeneralStringTable;
    fn load(&mut self, dat_paths: HashMap<String, DirEntry>) -> Result<(), ()>;
    fn from_holder(game_data_holder: &GameDataHolder) -> Self;
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
    loader.load(dat_paths.clone())?;

    let hldr = GameDataHolder {
        protocol_version: ChroniclesProtocol::GrandCrusade110,
        initial_dat_paths: dat_paths,

        npc_holder: loader.get_npcs(),
        npc_strings: loader.get_npc_strings(),
        item_holder: loader.get_items(),
        quest_holder: loader.get_quests(),
        skill_holder: loader.get_skills(),
        hunting_zone_holder: loader.get_hunting_zones(),
        game_string_table: loader.get_string_table(),
    };

    Ok(hldr)
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum ChroniclesProtocol {
    #[default]
    GrandCrusade110,
}

pub struct NpcInfo {
    pub(crate) id: NpcId,
    pub(crate) name: String,
}

impl From<&Npc> for NpcInfo {
    fn from(value: &Npc) -> Self {
        NpcInfo {
            id: value.id,
            name: value.name.clone(),
        }
    }
}

pub struct QuestInfo {
    pub(crate) id: QuestId,
    pub(crate) name: String,
}

impl From<&Quest> for QuestInfo {
    fn from(value: &Quest) -> Self {
        QuestInfo {
            id: value.id,
            name: value.title.clone(),
        }
    }
}

pub struct SkillInfo {
    pub(crate) id: SkillId,
    pub(crate) name: String,
}

impl From<&Skill> for SkillInfo {
    fn from(value: &Skill) -> Self {
        SkillInfo {
            id: value.id,
            name: value.name.clone(),
        }
    }
}

#[derive(Default)]
pub struct GameDataHolder {
    pub protocol_version: ChroniclesProtocol,
    pub initial_dat_paths: HashMap<String, DirEntry>,

    pub npc_holder: FHashMap<NpcId, Npc>,
    pub npc_strings: FHashMap<u32, String>,
    pub item_holder: FHashMap<ItemId, Item>,
    pub quest_holder: FHashMap<QuestId, Quest>,
    pub skill_holder: FHashMap<SkillId, Skill>,
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
