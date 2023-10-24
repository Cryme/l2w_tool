use crate::data::{ItemId, Location, NpcId, QuestId, ZoneId};
use crate::holders::{parse_dat, ChroniclesProtocol, GameDataHolder};
use crate::npc::Npc;
use crate::util::{Color, FromReader, BYTE, DWORD, SHORT, STR, FLOAT, LONG, ASCF};
use r#macro::FromReader;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use eframe::egui::Color32;
use num_traits::FromPrimitive;
use walkdir::DirEntry;
use crate::backend::{StepAction, WindowParams};
use crate::item::Item;
use crate::quest::{GoalType, MarkType, Quest, QuestCategory, QuestReward, QuestStep, QuestType, StepGoal, Unk1, Unk2, UnkQLevel};
use crate::util::UnrealValueFromReader;

struct Loader {
    game_data_name: Vec<String>,
    dat_paths: HashMap<String, DirEntry>,

    npc_holder: HashMap<NpcId, Npc>,
    npc_strings: HashMap<u32, String>,
    item_holder: HashMap<ItemId, Item>,
    quest_holder: HashMap<QuestId, Quest>,
}

impl Loader {
    fn load(dat_paths: HashMap<String, DirEntry>) -> Result<Self, ()>{
        let Some(path) = dat_paths.get(&"l2gamedataname.dat".to_string()) else {
            return Err(())
        };

        let mut h = Self {
            game_data_name: Self::load_game_data_name(&path.path())?,
            dat_paths,

            npc_holder: Default::default(),
            npc_strings: Default::default(),
            item_holder: Default::default(),
            quest_holder: Default::default(),
        };

        h.load_npcs()?;
        h.load_npc_strings()?;
        h.load_items()?;
        h.load_quests()?;

        Ok(h)
    }

    fn load_game_data_name(path: &Path) -> Result<Vec<String>, ()> {
        parse_dat(&path)
    }

    fn load_quests(&mut self) -> Result<(), ()> {
        let vals = parse_dat::<QuestName>(&self.dat_paths.get(&"questname-ru.dat".to_string()).unwrap().path())?;

        let mut current_id = if let Some(v) = vals.get(0) {v.id} else { 0u32 };
        let mut current_steps = Vec::new();

        for v in vals {
            if v.id == current_id{
                current_steps.push(v);
            } else {
                self.construct_quest(&current_steps);
                current_steps.clear();
                current_id = v.id;
                current_steps.push(v);
            }
        }

        self.construct_quest(&current_steps);

        Ok(())
    }

    fn construct_quest(&mut self, current_steps: &Vec<QuestName>) {
        if current_steps.len() == 0 {
            return;
        }

        let steps = current_steps.iter().map(|v| {
            let goals = v.goal_ids.iter().enumerate().map(|(i, g)| {
                StepGoal {
                    target_id: *g,
                    goal_type: GoalType::from_u32(v.goal_types[i]).unwrap(),
                    count: v.goal_nums[i],
                }
            }).collect();

            return WindowParams{
                inner: QuestStep {
                    title: v.sub_name.0.clone(),
                    label: v.entity_name.0.clone(),
                    desc: v.desc.0.clone(),
                    target_display_name: v.entity_name.0.clone(),
                    goals,
                    location: v.target_loc.into(),
                    additional_locations: v.additional_locations.iter().map(|v| (*v).into()).collect(),
                    unk_q_level: v.q_levels.iter().map(|l| UnkQLevel::from_u32(*l).unwrap()).collect(),
                    get_item_in_step: v.get_item_in_quest == 1,
                    unk_1: Unk1::from_u32(v.unk_1).unwrap(),
                    unk_2: Unk2::from_u32(v.unk_2).unwrap(),
                    prev_step_indexes: v.pre_level.iter().map(|i| *i as usize).collect(),
                },
                opened: false,
                action: StepAction::None,
            }
        }).collect();

        let last = &current_steps[current_steps.len() - 1];

        let rewards = last.reward_ids.iter().enumerate().map(|(i, v)| QuestReward{
            reward_id: ItemId(*v),
            count: last.reward_nums[i]
        }).collect();

        let x = Quest {
            id: QuestId(last.id),
            title: last.title.0.clone(),
            intro: last.intro.0.clone(),
            requirements: last.requirements.0.clone(),
            steps,
            quest_type: QuestType::from_u32(last.quest_type).unwrap(),
            category: QuestCategory::from_u32(last.category).unwrap(),
            mark_type: MarkType::from_u32(last.mark_type).unwrap(),
            min_lvl: last.lvl_min,
            max_lvl: last.lvl_max,
            allowed_classes: None,
            required_completed_quest_id: QuestId(last.cleared_quest),
            search_zone_id: ZoneId(last.search_zone_id),
            _is_clan_pet_quest: last.clan_pet_quest == 1,
            start_npc_loc: last.start_npc_loc.into(),
            start_npc_ids: last.start_npc_ids.iter().map(|v| NpcId(*v)).collect(),
            rewards,
            quest_items: last.quest_items.iter().map(|v| ItemId(*v)).collect(),
            faction_id: last.faction_id,
            faction_level_min: last.faction_level_min,
            faction_level_max: last.faction_level_max,

            java_class: None,
        };

        self.quest_holder.insert(x.id, x);
    }

    fn load_items(&mut self) -> Result<(), ()> {
        let vals = parse_dat::<ItemName>(&self.dat_paths.get(&"itemname-ru.dat".to_string()).unwrap().path())?;

        for v in vals {
            let x = Item {
                id: ItemId(v.id),
                name: if let Some(name) = self.game_data_name.get(v.name_link as usize) { name.clone() } else { format!("NameNotFound[{}]", v.name_link) },
                desc: v.description.0,
            };

            self.item_holder.insert(
                x.id,
                x
            );
        }

        Ok(())
    }


    fn load_npc_strings(&mut self) -> Result<(), ()> {
        let vals = parse_dat::<NpcString>(&self.dat_paths.get(&"npcstring-ru.dat".to_string()).unwrap().path())?;

        for v in vals {
            self.npc_strings.insert(v.id, v.value.0);
        }

        Ok(())
    }

    fn load_npcs(&mut self) -> Result<(), ()> {
        let vals = parse_dat::<NpcName>(&self.dat_paths.get(&"npcname-ru.dat".to_string()).unwrap().path())?;

        for v in vals {
            let npc = Npc{
                id: NpcId(v.id),
                name: v.name.0,
                title: v.title.0,
                title_color: Color32::from_rgba_premultiplied(v.title_color.r, v.title_color.g, v.title_color.b, v.title_color.a),
            };

            self.npc_holder.insert(
                npc.id,
                npc
            );
        }

        Ok(())
    }
}

pub fn load_holder(dat_paths: HashMap<String, DirEntry>) -> Result<GameDataHolder, ()> {
    let loader = Loader::load(dat_paths)?;

    Ok(GameDataHolder {
        protocol_version: ChroniclesProtocol::GrandCrusade110,

        npc_holder: loader.npc_holder,
        npc_strings: loader.npc_strings,
        item_holder: loader.item_holder,
        quest_holder: loader.quest_holder,

        java_classes_holder: Default::default(),
    })
}

#[derive(Debug, Clone, PartialEq, FromReader)]
struct L2GameDataName {
    value: STR,
}

#[derive(Debug, Clone, PartialEq, FromReader)]
struct NpcString {
    id: DWORD,
    value: ASCF,
}

#[derive(Debug, Clone, PartialEq, FromReader)]
struct NpcName {
    id: DWORD,
    name: ASCF,
    title: ASCF,
    title_color: Color,
}

#[derive(Debug, Clone, PartialEq, FromReader)]
struct ItemName {
    id: DWORD,
    name_link: DWORD,
    additional_name: ASCF,
    description: ASCF,
    popup: SHORT,
    default_action: ASCF,
    use_order: DWORD,
    set_id: SHORT,
    color: BYTE,
    tooltip_texture_link: DWORD,
    is_trade: BYTE,
    is_drop: BYTE,
    is_destruct: BYTE,
    is_private_store: BYTE,
    keep_type: BYTE,
    is_npc_trade: BYTE,
    is_commission_store: BYTE,
}

#[derive(Debug, Clone, PartialEq, FromReader, Default)]
struct QuestName {
    tag: DWORD,
    id: DWORD,
    level: DWORD,
    title: ASCF,
    sub_name: ASCF,
    desc: ASCF,
    goal_ids: Vec<DWORD>,
    goal_types: Vec<DWORD>,
    goal_nums: Vec<DWORD>,
    target_loc: FLOC,
    additional_locations: Vec<FLOC>,
    q_levels: Vec<DWORD>,
    lvl_min: DWORD,
    lvl_max: DWORD,
    quest_type: DWORD,
    entity_name: ASCF,
    get_item_in_quest: DWORD,
    unk_1: DWORD,
    unk_2: DWORD,
    start_npc_ids: Vec<DWORD>,
    start_npc_loc: FLOC,
    requirements: ASCF,
    intro: ASCF,
    class_limit: Vec<DWORD>,
    quest_items: Vec<DWORD>,
    clan_pet_quest: DWORD,
    cleared_quest: DWORD,
    mark_type: DWORD,
    search_zone_id: DWORD,
    category: DWORD,
    reward_ids: Vec<DWORD>,
    reward_nums: Vec<LONG>,
    pre_level: Vec<DWORD>,
    faction_id: DWORD,
    faction_level_min: DWORD,
    faction_level_max: DWORD,
}

#[derive(Debug, Copy, Clone, PartialEq, FromReader, Default)]
struct FLOC {
    x: FLOAT,
    y: FLOAT,
    z: FLOAT,
}

impl Into<Location> for FLOC {
    fn into(self) -> Location {
        Location {
            x: self.x as i32,
            y: self.y as i32,
            z: self.z as i32,
        }
    }
}