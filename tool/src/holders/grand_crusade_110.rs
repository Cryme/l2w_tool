#![allow(clippy::needless_borrow)]

use crate::backend::{StepAction, WindowParams};
use crate::data::{HuntingZoneId, InstantZoneId, ItemId, Location, NpcId, QuestId, SearchZoneId};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::{
    GoalType, MarkType, Quest, QuestCategory, QuestReward, QuestStep, QuestType, StepGoal, Unk1,
    Unk2, UnkQLevel,
};
use crate::holders::{GameDataHolder, Loader};
use crate::util::l2_reader::{deserialize_dat, save_dat, DatVariant};
use crate::util::{
    Color, ReadUnreal, UnrealCasts, UnrealReader, UnrealWriter, WriteUnreal, ASCF, BYTE, DWORD,
    FLOC, LONG, SHORT, STR, WORD,
};
use eframe::egui::Color32;
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::HashMap;
use std::path::Path;
use walkdir::DirEntry;

#[derive(Default)]
pub struct Loader110 {
    game_data_name: Vec<String>,
    dat_paths: HashMap<String, DirEntry>,

    npcs: HashMap<NpcId, Npc>,
    npc_strings: HashMap<u32, String>,
    items: HashMap<ItemId, Item>,
    quests: HashMap<QuestId, Quest>,
    hunting_zones: HashMap<HuntingZoneId, HuntingZone>,
}

impl Loader for Loader110 {
    fn get_quests(&self) -> HashMap<QuestId, Quest> {
        self.quests.clone()
    }

    fn get_npcs(&self) -> HashMap<NpcId, Npc> {
        self.npcs.clone()
    }

    fn get_npc_strings(&self) -> HashMap<u32, String> {
        self.npc_strings.clone()
    }

    fn get_items(&self) -> HashMap<ItemId, Item> {
        self.items.clone()
    }

    fn get_hunting_zones(&self) -> HashMap<HuntingZoneId, HuntingZone> {
        self.hunting_zones.clone()
    }

    fn load(&mut self, dat_paths: HashMap<String, DirEntry>) -> Result<(), ()> {
        let Some(path) = dat_paths.get(&"l2gamedataname.dat".to_string()) else {
            return Err(());
        };

        self.game_data_name = Self::load_game_data_name(path.path())?;
        self.dat_paths = dat_paths;

        self.load_npcs()?;
        self.load_npc_strings()?;
        self.load_items()?;
        self.load_hunting_zones()?;
        self.load_quests()?;

        Ok(())
    }

    fn from_holder(game_data_holder: &GameDataHolder) -> Self {
        Self {
            game_data_name: Default::default(),
            dat_paths: game_data_holder.initial_dat_paths.clone(),
            npcs: Default::default(),
            npc_strings: Default::default(),
            items: Default::default(),
            quests: game_data_holder.quest_holder.clone(),
            hunting_zones: Default::default(),
        }
    }

    fn serialize_to_binary(&self) -> std::io::Result<()> {
        let mut res = Vec::new();

        let mut vals: Vec<_> = self.quests.values().collect();
        vals.sort_by(|a, b| a.id.cmp(&b.id));

        for quest in vals {
            for step in QuestName::from_quest(quest) {
                res.push(step);
            }
        }

        save_dat(
            self.dat_paths
                .get(&"questname-ru.dat".to_string())
                .unwrap()
                .path(),
            DatVariant::Array(res),
        )?;

        Ok(())
    }
}

impl Loader110 {
    fn load_game_data_name(path: &Path) -> Result<Vec<String>, ()> {
        deserialize_dat(path)
    }

    fn load_hunting_zones(&mut self) -> Result<(), ()> {
        let vals = deserialize_dat::<HuntingZoneDat>(
            self.dat_paths
                .get(&"huntingzone-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in vals {
            self.hunting_zones.insert(
                HuntingZoneId(v.id),
                HuntingZone {
                    id: HuntingZoneId(v.id),
                    name: v.name.0.clone(),
                    desc: v.description.0.clone(),
                    _search_zone_id: SearchZoneId(v.search_zone_id),
                    _instant_zone_id: InstantZoneId(v.instance_zone_id),
                },
            );
        }

        Ok(())
    }

    fn load_quests(&mut self) -> Result<Vec<DWORD>, ()> {
        let mut order = Vec::new();

        let vals = deserialize_dat::<QuestName>(
            self.dat_paths
                .get(&"questname-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let mut current_id = if let Some(v) = vals.get(0) {
            v.id
        } else {
            0u32
        };
        let mut current_steps = Vec::new();

        for v in vals {
            if order.is_empty() || v.id != *order.last().unwrap() {
                order.push(v.id)
            }

            if v.id == current_id {
                current_steps.push(v);
            } else {
                self.construct_quest(&current_steps);
                current_steps.clear();
                current_id = v.id;
                current_steps.push(v);
            }
        }

        self.construct_quest(&current_steps);

        Ok(order)
    }

    fn construct_quest(&mut self, current_steps: &Vec<QuestName>) {
        if current_steps.is_empty() {
            return;
        }

        let mut last_finish_id = u32::MAX;

        let steps = current_steps
            .iter()
            // .filter(|v| v.level != u32::MAX)
            .map(|v| {
                let goals = v
                    .goal_ids
                    .iter()
                    .enumerate()
                    .map(|(i, g)| {
                        let (id, goal) = GoalType::from_pair(*g, v.goal_types[i]);
                        StepGoal {
                            target_id: id,
                            goal_type: goal,
                            count: v.goal_nums[i],
                        }
                    })
                    .collect();

                if v.level > 1_000 {
                    last_finish_id = v.level.min(last_finish_id);
                }

                return WindowParams {
                    inner: QuestStep {
                        title: if v.level > 1_000 {
                            "FINISH".to_string()
                        } else {
                            v.sub_name.0.clone()
                        },
                        label: v.entity_name.0.clone(),
                        desc: v.desc.0.clone(),
                        goals,
                        location: v.target_loc.into(),
                        additional_locations: v
                            .additional_locations
                            .iter()
                            .map(|v| (*v).into())
                            .collect(),
                        unk_q_level: v
                            .q_levels
                            .iter()
                            .map(|l| UnkQLevel::from_u32(*l).unwrap())
                            .collect(),
                        _get_item_in_step: v.get_item_in_quest == 1,
                        unk_1: Unk1::from_u32(v.unk_1).unwrap(),
                        unk_2: Unk2::from_u32(v.unk_2).unwrap(),
                        prev_steps: v.pre_level.clone(),
                        level: v.level,
                    },

                    original_id: (),
                    opened: false,
                    action: StepAction::None,
                };
            })
            .collect();

        let first = &current_steps[0];

        let rewards = first
            .reward_ids
            .iter()
            .enumerate()
            .map(|(i, v)| QuestReward {
                reward_id: ItemId(*v),
                count: first.reward_nums[i],
            })
            .collect();

        let x = Quest {
            id: QuestId(first.id),
            title: first.title.0.clone(),
            intro: first.intro.0.clone(),
            requirements: first.requirements.0.clone(),
            steps,
            last_finish_step_id: last_finish_id,
            quest_type: QuestType::from_u32(first.quest_type).unwrap(),
            category: QuestCategory::from_u32(first.category).unwrap(),
            mark_type: MarkType::from_u32(first.mark_type).unwrap(),
            min_lvl: first.lvl_min,
            max_lvl: first.lvl_max,
            allowed_classes: if first.class_limit.is_empty() {
                None
            } else {
                Some(
                    first
                        .class_limit
                        .iter()
                        .map(|v| unsafe { std::mem::transmute(*v as u8) })
                        .collect(),
                )
            },
            required_completed_quest_id: QuestId(first.cleared_quest),
            search_zone_id: HuntingZoneId(first.search_zone_id),
            _is_clan_pet_quest: first.clan_pet_quest == 1,
            start_npc_loc: first.start_npc_loc.into(),
            start_npc_ids: first.start_npc_ids.iter().map(|v| NpcId(*v)).collect(),
            rewards,
            quest_items: first.quest_items.iter().map(|v| ItemId(*v)).collect(),
            _faction_id: first.faction_id,
            _faction_level_min: first.faction_level_min,
            _faction_level_max: first.faction_level_max,

            java_class: None,
        };

        self.quests.insert(x.id, x);
    }

    fn load_items(&mut self) -> Result<(), ()> {
        let vals = deserialize_dat::<ItemName>(
            self.dat_paths
                .get(&"itemname-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in vals {
            let x = Item {
                id: ItemId(v.id),
                name: if let Some(name) = self.game_data_name.get(v.name_link as usize) {
                    name.clone()
                } else {
                    format!("NameNotFound[{}]", v.name_link)
                },
                desc: v.description.0,
            };

            self.items.insert(x.id, x);
        }

        Ok(())
    }

    fn load_npc_strings(&mut self) -> Result<(), ()> {
        let vals = deserialize_dat::<NpcString>(
            self.dat_paths
                .get(&"npcstring-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in vals {
            self.npc_strings.insert(v.id, v.value.0);
        }

        Ok(())
    }

    fn load_npcs(&mut self) -> Result<(), ()> {
        let vals = deserialize_dat::<NpcName>(
            self.dat_paths
                .get(&"npcname-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in vals {
            let npc = Npc {
                id: NpcId(v.id),
                name: v.name.0,
                title: v.title.0,
                title_color: Color32::from_rgba_premultiplied(
                    v.title_color.r,
                    v.title_color.g,
                    v.title_color.b,
                    v.title_color.a,
                ),
            };

            self.npcs.insert(npc.id, npc);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct L2GameDataName {
    value: STR,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct NpcString {
    id: DWORD,
    value: ASCF,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct NpcName {
    id: DWORD,
    name: ASCF,
    title: ASCF,
    title_color: Color,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
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

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
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

impl QuestName {
    fn from_quest(quest: &Quest) -> Vec<Self> {
        let mut res = Vec::with_capacity(quest.steps.len() + 1);

        for step in quest.steps.iter().map(|v| &v.inner) {
            let goals: Vec<_> = step
                .goals
                .iter()
                .map(|v| {
                    let c = v.goal_type.as_pair(v.target_id);

                    (c.0, c.1, v.count)
                })
                .collect();

            res.push(Self {
                tag: 1,
                id: quest.id.0,
                level: step.level,
                title: ASCF(quest.title.clone()),
                sub_name: ASCF(if step.level < 1_000 {
                    step.title.clone()
                } else {
                    "".to_string()
                }),
                desc: ASCF(step.desc.clone()),
                goal_ids: goals.iter().map(|v| v.0).collect(),
                goal_types: goals.iter().map(|v| v.1).collect(),
                goal_nums: goals.iter().map(|v| v.2).collect(),
                target_loc: step.location.into(),
                additional_locations: step
                    .additional_locations
                    .iter()
                    .map(|v| (*v).into())
                    .collect(),
                q_levels: step
                    .unk_q_level
                    .iter()
                    .map(|v| v.to_u32().unwrap())
                    .collect(),
                lvl_min: quest.min_lvl,
                lvl_max: quest.max_lvl,
                quest_type: quest.quest_type.to_u32().unwrap(),
                entity_name: ASCF(step.label.clone()),
                get_item_in_quest: step._get_item_in_step.to_u32_bool(),
                unk_1: step.unk_1.to_u32().unwrap(),
                unk_2: step.unk_2.to_u32().unwrap(),
                start_npc_ids: quest.start_npc_ids.iter().map(|v| v.0).collect(),
                start_npc_loc: quest.start_npc_loc.into(),
                requirements: ASCF(quest.requirements.clone()),
                intro: ASCF(quest.intro.clone()),
                class_limit: if let Some(c) = &quest.allowed_classes {
                    c.iter().map(|v| (*v as u8) as DWORD).collect()
                } else {
                    vec![]
                },
                quest_items: quest.quest_items.iter().map(|v| v.0).collect(),
                clan_pet_quest: quest._is_clan_pet_quest.to_u32_bool(),
                cleared_quest: quest.required_completed_quest_id.0,
                mark_type: quest.mark_type.to_u32().unwrap(),
                search_zone_id: quest.search_zone_id.0,
                category: quest.category.to_u32().unwrap(),
                reward_ids: quest.rewards.iter().map(|v| v.reward_id.0).collect(),
                reward_nums: quest.rewards.iter().map(|v| v.count).collect(),
                pre_level: step.prev_steps.clone(),
                faction_id: quest._faction_id,
                faction_level_min: quest._faction_level_min,
                faction_level_max: quest._faction_level_max,
            });
        }

        res
    }
}

impl From<FLOC> for Location {
    fn from(val: FLOC) -> Self {
        Location {
            x: val.x as i32,
            y: val.y as i32,
            z: val.z as i32,
        }
    }
}
impl From<Location> for FLOC {
    fn from(val: Location) -> Self {
        FLOC {
            x: val.x as f32,
            y: val.y as f32,
            z: val.z as f32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
struct HuntingZoneDat {
    id: DWORD,
    zone_type: DWORD,
    min_recommended_level: DWORD,
    max_recommended_level: DWORD,
    start_npc_loc: FLOC,
    description: ASCF,
    search_zone_id: DWORD,
    name: ASCF,
    region_id: WORD,
    npc_id: DWORD,
    quest_ids: Vec<WORD>,
    instance_zone_id: DWORD,
}
