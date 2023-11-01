use crate::backend::Config;
use crate::data::{HuntingZoneId, ItemId, NpcId, QuestId};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::Quest;
use crate::util::l2_reader::load_dat_file;
use crate::util::FromReader;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufReader, Cursor};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

mod grand_crusade_110;

pub trait Loader {
    fn get_quests(&self) -> HashMap<QuestId, Quest>;
    fn get_npcs(&self) -> HashMap<NpcId, Npc>;
    fn get_npc_strings(&self) -> HashMap<u32, String>;
    fn get_items(&self) -> HashMap<ItemId, Item>;
    fn get_hunting_zones(&self) -> HashMap<HuntingZoneId, HuntingZone>;
}

fn get_loader_for_protocol(
    dat_paths: HashMap<String, DirEntry>,
    protocol: ChroniclesProtocol,
) -> Result<impl Loader + Sized, ()> {
    Ok(match protocol {
        ChroniclesProtocol::GrandCrusade110 => grand_crusade_110::load_holder(dat_paths)?,
    })
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

    let loader = get_loader_for_protocol(dat_paths, protocol)?;

    Ok(GameDataHolder {
        protocol_version: ChroniclesProtocol::GrandCrusade110,

        npc_holder: loader.get_npcs(),
        npc_strings: loader.get_npc_strings(),
        item_holder: loader.get_items(),
        quest_holder: loader.get_quests(),
        hunting_zone_holder: loader.get_hunting_zones(),
    })
}

fn parse_dat<T: FromReader + Debug>(file_path: &Path) -> Result<Vec<T>, ()> {
    println!("Loading {file_path:?}...");
    let Ok(bytes) = load_dat_file(file_path) else {
        return Err(());
    };

    let mut reader = BufReader::new(Cursor::new(bytes));
    let count = u32::from_reader(&mut reader);

    println!("\tElements count: {count}");

    let mut res = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let t = T::from_reader(&mut reader);
        res.push(t);
    }

    Ok(res)
}

#[derive(Default)]
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

#[derive(Default)]
pub struct GameDataHolder {
    pub protocol_version: ChroniclesProtocol,

    pub npc_holder: HashMap<NpcId, Npc>,
    pub npc_strings: HashMap<u32, String>,
    pub item_holder: HashMap<ItemId, Item>,
    pub quest_holder: HashMap<QuestId, Quest>,
    pub hunting_zone_holder: HashMap<HuntingZoneId, HuntingZone>,
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
