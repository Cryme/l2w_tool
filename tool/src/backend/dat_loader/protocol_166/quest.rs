use crate::backend::dat_loader::protocol_166::CoordsXYZ;
use crate::common::{HuntingZoneId, ItemId, NpcId, PlayerClass, QuestId};
use crate::entity::quest::{
    GoalType, MarkType, Quest, QuestCategory, QuestReward, QuestStep, QuestType, StepGoal, Unk1,
    Unk2, UnkQLevel,
};

use l2_rw::ue2_rw::{ASCF, DWORD, LONG};
use l2_rw::{DatVariant, deserialize_dat, save_dat};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::Localization;
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::backend::log_holder::{Log, LogLevel};
use eframe::egui::Pos2;
use r#macro::{ReadUnreal, WriteUnreal};
use num_traits::{FromPrimitive, ToPrimitive};
use std::thread;
use std::thread::JoinHandle;

impl GameDataHolder {
    pub fn serialize_quests_to_binary(&mut self) -> JoinHandle<Vec<Log>> {
        let mut res = Vec::new();

        let mut vals: Vec<_> = self.quest_holder.values().filter(|v| !v._deleted).collect();
        vals.sort_by(|a, b| a.id.cmp(&b.id));

        let eu = if let Some(dir) = self.dat_paths.get(&"questname-eu.dat".to_string()) {
            Some((
                dir.clone(),
                vals.iter()
                    .map(|v| QuestNameDat::from_entity(v, Localization::EU))
                    .flatten()
                    .collect::<Vec<QuestNameDat>>(),
            ))
        } else {
            None
        };

        for quest in vals {
            for step in QuestNameDat::from_entity(quest, Localization::RU) {
                res.push(step);
            }
        }

        let quest_path = self
            .dat_paths
            .get(&"questname-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            let mut log = if let Err(e) = save_dat(
                quest_path.path(),
                DatVariant::<(), QuestNameDat>::Array(res),
            ) {
                vec![Log::from_loader_e(e)]
            } else {
                vec![Log::from_loader_i("Quest Name RU saved")]
            };

            if let Some((dir, dats)) = eu {
                log.push(
                    if let Err(e) =
                        save_dat(dir.path(), DatVariant::<(), QuestNameDat>::Array(dats))
                    {
                        Log::from_loader_e(e)
                    } else {
                        Log::from_loader_i("Quest Name EU saved")
                    },
                );
            }

            log
        })
    }
    pub fn load_quests(&mut self) -> Result<Vec<Log>, ()> {
        let mut warnings = vec![];

        let quest_name_ru = deserialize_dat::<QuestNameDat>(
            self.dat_paths
                .get(&"questname-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let quest_name_eu =
            if let Some(eu_dir) = self.dat_paths.get(&"questname-eu.dat".to_string()) {
                let dats = deserialize_dat::<QuestNameDat>(eu_dir.path())?;

                if dats.len() != quest_name_ru.len() {
                    warnings.push(Log {
                    level: LogLevel::Error,
                    producer: "Quest Loader".to_string(),
                    log: format!(
                        "Quest Name EU rows count({}) doesn't match Quest Name RU rows count({})",
                        dats.len(),
                        quest_name_ru.len()
                    ),
                });
                    None
                } else {
                    Some(dats)
                }
            } else {
                None
            };

        let mut current_id = if let Some(v) = quest_name_ru.first() {
            v.id
        } else {
            0u32
        };

        let mut current_steps_ru = Vec::new();
        let mut current_steps_eu = Vec::new();

        for (i, v) in quest_name_ru.iter().enumerate() {
            if v.id != current_id {
                warnings.extend(self.construct_quest(&current_steps_ru, &current_steps_eu));
                current_steps_ru.clear();
                current_steps_eu.clear();

                current_id = v.id;
            }

            current_steps_ru.push(v);

            if let Some(eu) = &quest_name_eu {
                current_steps_eu.push(&eu[i])
            }
        }

        warnings.extend(self.construct_quest(&current_steps_ru, &current_steps_eu));

        Ok(warnings)
    }

    pub fn construct_quest(
        &mut self,
        current_steps_ru: &[&QuestNameDat],
        current_steps_eu: &[&QuestNameDat],
    ) -> Vec<Log> {
        let mut warnings = vec![];

        if current_steps_ru.is_empty() {
            return warnings;
        }

        let steps = current_steps_ru
            .iter()
            .enumerate()
            .map(|(i, v)| {
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
                    if let Some((idx, _)) = current_steps_ru
                        .iter()
                        .enumerate()
                        .find(|(_, v)| v.level == *i)
                    {
                        prev_steps.push(idx);
                    }
                }

                QuestStep {
                    title: (
                        v.sub_name.to_string(),
                        current_steps_eu
                            .get(i)
                            .map_or("NOT_EXIST".to_string(), |v| v.sub_name.to_string()),
                    )
                        .into(),
                    label: (
                        v.entity_name.to_string(),
                        current_steps_eu
                            .get(i)
                            .map_or("NOT_EXIST".to_string(), |v| v.entity_name.to_string()),
                    )
                        .into(),
                    desc: (
                        v.desc.to_string(),
                        current_steps_eu
                            .get(i)
                            .map_or("NOT_EXIST".to_string(), |v| v.desc.to_string()),
                    )
                        .into(),
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

        let first_ru = current_steps_ru[0];
        let first_eu = current_steps_eu.get(0);

        let rewards = first_ru
            .reward_ids
            .iter()
            .enumerate()
            .map(|(i, v)| QuestReward {
                reward_id: ItemId(*v),
                count: if let Some(v) = first_ru.reward_nums.get(i) {
                    *v
                } else {
                    warnings.push(Log {
                        level: LogLevel::Warning,
                        producer: "Quest Loader".to_string(),
                        log: format!(
                            "Quest[{}]: no reward count for item[{}]. Set to 0",
                            first_ru.id, v
                        ),
                    });
                    0
                },
            })
            .collect();

        let x = Quest {
            id: QuestId(first_ru.id),
            title: (
                first_ru.title.to_string(),
                first_eu
                    .map_or("NOT_EXIST".to_string(), |v| v.title.to_string())
                    .into(),
            )
                .into(),
            intro: (
                first_ru.intro.to_string(),
                first_eu
                    .map_or("NOT_EXIST".to_string(), |v| v.intro.to_string())
                    .into(),
            )
                .into(),
            requirements: (
                first_ru.requirements.to_string(),
                first_eu
                    .map_or("NOT_EXIST".to_string(), |v| v.requirements.to_string())
                    .into(),
            )
                .into(),
            steps,
            quest_type: QuestType::from_u32(first_ru.quest_type).unwrap(),
            category: QuestCategory::from_u32(first_ru.category).unwrap(),
            mark_type: MarkType::from_u32(first_ru.mark_type)
                .unwrap_or_else(|| panic!("unknown mark type {}", first_ru.mark_type)),
            min_lvl: first_ru.lvl_min,
            max_lvl: first_ru.lvl_max,
            allowed_classes: if first_ru.class_limit.is_empty() {
                None
            } else {
                Some(
                    first_ru
                        .class_limit
                        .iter()
                        .map(|v| {
                            PlayerClass::from_u32(*v).unwrap_or_else(|| panic!("Unk type: {v}"))
                        })
                        .collect(),
                )
            },
            required_completed_quest_id: QuestId(first_ru.cleared_quest),
            search_zone_id: HuntingZoneId(first_ru.search_zone_id),
            _is_clan_pet_quest: first_ru.clan_pet_quest == 1,
            start_npc_loc: first_ru.start_npc_loc.into(),
            start_npc_ids: first_ru.start_npc_ids.iter().map(|v| NpcId(*v)).collect(),
            rewards,
            quest_items: first_ru.quest_items.iter().map(|v| ItemId(*v)).collect(),
            _faction_id: first_ru.faction_id,
            _faction_level_min: first_ru.faction_level_min,
            _faction_level_max: first_ru.faction_level_max,

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
    fn from_entity(quest: &Quest, localization: Localization) -> Vec<Self> {
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
                title: (&quest.title[localization]).into(),
                sub_name: if step.stage < 1_000 {
                    (&step.title[localization]).into()
                } else {
                    ASCF::empty()
                },
                desc: (&step.desc[localization]).into(),
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
                entity_name: (&step.label[localization]).into(),
                get_item_in_quest: step._get_item_in_step.into(),
                unk_1: step.unk_1.to_u32().unwrap(),
                unk_2: step.unk_2.to_u32().unwrap(),
                start_npc_ids: quest.start_npc_ids.iter().map(|v| v.0).collect(),
                start_npc_loc: quest.start_npc_loc.into(),
                requirements: (&quest.requirements[localization]).into(),
                intro: (&quest.intro[localization]).into(),
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
