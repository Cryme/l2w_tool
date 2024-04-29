use crate::backend::quest::StepAction;
use crate::backend::WindowParams;
use crate::data::{HuntingZoneId, ItemId, NpcId, QuestId};
use crate::entity::quest::{
    GoalType, MarkType, Quest, QuestCategory, QuestReward, QuestStep, QuestType, StepGoal, Unk1,
    Unk2, UnkQLevel,
};
use crate::holder::grand_crusade_110::Loader110;
use crate::util::l2_reader::{deserialize_dat, save_dat, DatVariant};
use crate::util::{
    ReadUnreal, UnrealCasts, UnrealReader, UnrealWriter, WriteUnreal, ASCF, DWORD, FLOC, LONG,
};
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::sync::RwLock;
use std::thread;
use std::thread::JoinHandle;

impl Loader110 {
    pub fn serialize_quests_to_binary(&mut self) -> JoinHandle<()> {
        let mut res = Vec::new();

        let mut vals: Vec<_> = self.quests.values().collect();
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
                println!("{e:?}");
            } else {
                println!("Quest Name saved");
            }
        })
    }
    pub fn load_quests(&mut self) -> Result<Vec<u32>, ()> {
        let mut order = Vec::new();

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

    pub fn construct_quest(&mut self, current_steps: &Vec<QuestNameDat>) {
        if current_steps.is_empty() {
            return;
        }

        let mut last_finish_id = u32::MAX;

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

                    initial_id: (),
                    opened: false,
                    action: RwLock::new(StepAction::None),
                    params: (),
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
                count: if let Some(v) = first.reward_nums.get(i) {
                    *v
                } else {
                    println!("Corrupted Quest {}", first.id);
                    0
                },
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

impl QuestNameDat {
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
