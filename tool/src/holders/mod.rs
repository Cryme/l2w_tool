use crate::backend::Config;
use crate::data::{HuntingZoneId, ItemId, NpcId, QuestId, SkillId};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::entity::skill::Skill;
use crate::holders::grand_crusade_110::{L2GeneralStringTable, Loader110};
use std::collections::HashMap;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

mod grand_crusade_110;

pub trait Loader {
    fn get_quests(&self) -> HashMap<QuestId, Quest>;
    fn get_skills(&self) -> HashMap<SkillId, Skill>;
    fn get_npcs(&self) -> HashMap<NpcId, Npc>;
    fn get_npc_strings(&self) -> HashMap<u32, String>;
    fn get_items(&self) -> HashMap<ItemId, Item>;
    fn get_hunting_zones(&self) -> HashMap<HuntingZoneId, HuntingZone>;
    fn get_string_table(&self) -> L2GeneralStringTable;
    fn load(&mut self, dat_paths: HashMap<String, DirEntry>) -> Result<(), ()>;
    fn from_holder(game_data_holder: &GameDataHolder) -> Self;
    fn serialize_to_binary(&mut self, quests: bool, skills: bool) -> std::io::Result<()>;
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

    Ok(GameDataHolder {
        protocol_version: ChroniclesProtocol::GrandCrusade110,
        initial_dat_paths: dat_paths,

        npc_holder: loader.get_npcs(),
        npc_strings: loader.get_npc_strings(),
        item_holder: loader.get_items(),
        quest_holder: loader.get_quests(),
        skill_holder: loader.get_skills(),
        hunting_zone_holder: loader.get_hunting_zones(),
        game_string_table: loader.get_string_table(),
    })
}

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum ChroniclesProtocol {
    #[default]
    GrandCrusade110,
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

    pub npc_holder: HashMap<NpcId, Npc>,
    pub npc_strings: HashMap<u32, String>,
    pub item_holder: HashMap<ItemId, Item>,
    pub quest_holder: HashMap<QuestId, Quest>,
    pub skill_holder: HashMap<SkillId, Skill>,
    pub hunting_zone_holder: HashMap<HuntingZoneId, HuntingZone>,
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
