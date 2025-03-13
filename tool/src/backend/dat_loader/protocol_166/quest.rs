use crate::backend::dat_loader::protocol_166::CoordsXYZ;
use crate::common::{HuntingZoneId, ItemId, NpcId, PlayerClass, QuestId};
use crate::entity::quest::{
    GoalType, MarkType, Quest, QuestCategory, QuestReward, QuestStep, QuestType, StepGoal, Unk1,
    Unk2, UnkQLevel,
};

use l2_rw::ue2_rw::{ASCF, DWORD, LONG};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::backend::log_holder::{Log, LogLevel};
use eframe::egui::Pos2;
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl GameDataHolder {
    pub fn serialize_quests_to_binary(&mut self) -> JoinHandle<Vec<Log>> {
        let mut res = Vec::new();

        let mut vals: Vec<_> = self.quest_holder.values().filter(|v| !v._deleted).collect();
        vals.sort_by(|a, b| a.id.cmp(&b.id));

        for quest in vals {
            for step in QuestNameDat::from_quest(quest) {
                res.push(step);
            }
        }

        let quest_path = self
            .dat_paths
            .get(&"questname-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) = save_dat(
                quest_path.path(),
                DatVariant::<(), QuestNameDat>::Array(res),
            ) {
                vec![Log::from_loader_e(e)]
            } else {
                vec![Log::from_loader_i("Quest Name saved")]
            }
        })
    }
    pub fn load_quests(&mut self) -> Result<Vec<Log>, ()> {
        let vals = deserialize_dat::<QuestNameDat>(
            self.dat_paths
                .get(&"questname-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let mut current_id = if let Some(v) = vals.first() {
            v.id
        } else {
            0u32
        };
        let mut current_steps = Vec::new();

        let mut warnings = vec![];

        for v in vals {
            if v.id == current_id {
                current_steps.push(v);
            } else {
                warnings.extend(self.construct_quest(&current_steps));
                current_steps.clear();
                current_id = v.id;
                current_steps.push(v);
            }
        }

        warnings.extend(self.construct_quest(&current_steps));

        Ok(warnings)
    }

    pub fn construct_quest(&mut self, current_steps: &[QuestNameDat]) -> Vec<Log> {
        let mut warnings = vec![];

        if current_steps.is_empty() {
            return warnings;
        }

        let steps = current_steps
            .iter()
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

                let mut prev_steps = vec![];

                for i in &v.pre_level {
                    if let Some((idx, _)) = current_steps
                        .iter()
                        .enumerate()
                        .find(|(_, v)| v.level == *i)
                    {
                        prev_steps.push(idx);
                    }
                }

                QuestStep {
                    title: v.sub_name.to_string(),
                    label: v.entity_name.to_string(),
                    desc: v.desc.to_string(),
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
                    prev_steps,
                    stage: v.level,
                    pos: Pos2::default(),
                    collapsed: true,
                }
            })
            .collect();

        let first = &current_steps[0];

        let rewards = first
            .reward_ids
            .iter()
            .enumerate()
            .map(|(i, v)| QuestReward {
                reward_id: ItemId(*v),
                count: if let Some(v) = first.reward_nums.get(i) {
                    *v
                } else {
                    warnings.push(Log {
                        level: LogLevel::Warning,
                        producer: "Quest Loader".to_string(),
                        log: format!(
                            "Quest[{}]: no reward count for item[{}]. Set to 0",
                            first.id, v
                        ),
                    });
                    0
                },
            })
            .collect();

        let x = Quest {
            id: QuestId(first.id),
            title: first.title.to_string(),
            intro: first.intro.to_string(),
            requirements: first.requirements.to_string(),
            steps,
            quest_type: QuestType::from_u32(first.quest_type).unwrap(),
            category: QuestCategory::from_u32(first.category).unwrap(),
            mark_type: MarkType::from_u32(first.mark_type)
                .unwrap_or_else(|| panic!("unknown mark type {}", first.mark_type)),
            min_lvl: first.lvl_min,
            max_lvl: first.lvl_max,
            allowed_classes: if first.class_limit.is_empty() {
                None
            } else {
                Some(
                    first
                        .class_limit
                        .iter()
                        .map(|v| {
                            PlayerClass::from_u32(*v).unwrap_or_else(|| panic!("Unk type: {v}"))
                        })
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
            ..Default::default()
        };

        self.quest_holder.insert(x.id, x);

        warnings
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct QuestNameDat {
    tag: DWORD,
    id: DWORD,
    level: DWORD,
    title: ASCF,
    sub_name: ASCF,
    desc: ASCF,
    goal_ids: Vec<DWORD>,
    goal_types: Vec<DWORD>,
    goal_nums: Vec<DWORD>,
    target_loc: CoordsXYZ,
    additional_locations: Vec<CoordsXYZ>,
    q_levels: Vec<DWORD>,
    lvl_min: DWORD,
    lvl_max: DWORD,
    quest_type: DWORD,
    entity_name: ASCF,
    get_item_in_quest: DWORD,
    unk_1: DWORD,
    unk_2: DWORD,
    start_npc_ids: Vec<DWORD>,
    start_npc_loc: CoordsXYZ,
    requirements: ASCF,
    intro: ASCF,
    class_limit: Vec<DWORD>,
    quest_items: Vec<DWORD>,
    clan_pet_quest: DWORD,
    cleared_quest: DWORD,
    mark_type: DWORD,
    category_id: DWORD,
    priority_level: DWORD,
    search_zone_id: DWORD,
    category: DWORD,
    reward_ids: Vec<DWORD>,
    reward_nums: Vec<LONG>,
    pre_level: Vec<DWORD>,
    faction_id: DWORD,
    faction_level_min: DWORD,
    faction_level_max: DWORD,
}

impl QuestNameDat {
    fn from_quest(quest: &Quest) -> Vec<Self> {
        let mut res = Vec::with_capacity(quest.steps.len() + 1);

        for step in quest.steps.iter() {
            let goals: Vec<_> = step
                .goals
                .iter()
                .map(|v| {
                    let c = v.goal_type.as_pair(v.target_id);

                    (c.0, c.1, v.count)
                })
                .collect();

            let prev_steps = step
                .prev_steps
                .iter()
                .map(|i| quest.steps[*i].stage)
                .collect::<Vec<_>>();

            res.push(Self {
                tag: 1,
                id: quest.id.0,
                level: step.stage,
                title: (&quest.title).into(),
                sub_name: if step.stage < 1_000 {
                    (&step.title).into()
                } else {
                    ASCF::empty()
                },
                desc: (&step.desc).into(),
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
                entity_name: (&step.label).into(),
                get_item_in_quest: step._get_item_in_step.into(),
                unk_1: step.unk_1.to_u32().unwrap(),
                unk_2: step.unk_2.to_u32().unwrap(),
                start_npc_ids: quest.start_npc_ids.iter().map(|v| v.0).collect(),
                start_npc_loc: quest.start_npc_loc.into(),
                requirements: (&quest.requirements).into(),
                intro: (&quest.intro).into(),
                class_limit: if let Some(c) = &quest.allowed_classes {
                    c.iter().map(|v| v.to_u32().unwrap()).collect()
                } else {
                    vec![]
                },
                quest_items: quest.quest_items.iter().map(|v| v.0).collect(),
                clan_pet_quest: quest._is_clan_pet_quest.into(),
                cleared_quest: quest.required_completed_quest_id.0,
                mark_type: quest.mark_type.to_u32().unwrap(),
                category_id: quest.category_id,
                priority_level: quest.priority_level,
                search_zone_id: quest.search_zone_id.0,
                category: quest.category.to_u32().unwrap(),
                reward_ids: quest.rewards.iter().map(|v| v.reward_id.0).collect(),
                reward_nums: quest.rewards.iter().map(|v| v.count).collect(),
                pre_level: prev_steps,
                faction_id: quest._faction_id,
                faction_level_min: quest._faction_level_min,
                faction_level_max: quest._faction_level_max,
            });
        }

        res
    }
}
