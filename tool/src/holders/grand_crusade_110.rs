#![allow(clippy::needless_borrow)]

use crate::backend::{StepAction, WindowParams};
use crate::data::{HuntingZoneId, InstantZoneId, ItemId, Location, NpcId, QuestId, SearchZoneId, SkillId, VisualEffectId};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::{
    GoalType, MarkType, Quest, QuestCategory, QuestReward, QuestStep, QuestType, StepGoal, Unk1,
    Unk2, UnkQLevel,
};
use crate::holders::{GameDataHolder, Loader};
use crate::util::l2_reader::{deserialize_dat, save_dat, DatVariant, deserialize_dat_with_string_dict};
use crate::util::{Color, ReadUnreal, UnrealCasts, UnrealReader, UnrealWriter, WriteUnreal, ASCF, BYTE, DWORD, FLOC, LONG, SHORT, STR, WORD, USHORT, FLOAT, UVEC};
use eframe::egui::Color32;
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use walkdir::DirEntry;
use crate::entity::skill::{EnchantInfo, EnchantLevelInfo, Skill, SkillLevelInfo, SkillType};

#[derive(Default)]
pub struct Loader110 {
    game_data_name: Vec<String>,
    dat_paths: HashMap<String, DirEntry>,

    npcs: HashMap<NpcId, Npc>,
    npc_strings: HashMap<u32, String>,
    items: HashMap<ItemId, Item>,
    quests: HashMap<QuestId, Quest>,
    hunting_zones: HashMap<HuntingZoneId, HuntingZone>,
    skills: HashMap<SkillId, Skill>,
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
        self.load_skills()?;

        println!("======================================");
        println!("\tLoaded {} Npcs", self.npcs.len());
        println!("\tLoaded {} Npc Strings", self.npc_strings.len());
        println!("\tLoaded {} Items", self.items.len());
        println!("\tLoaded {} Hunting Zones", self.hunting_zones.len());
        println!("\tLoaded {} Quests", self.quests.len());
        println!("\tLoaded {} Skills", self.skills.len());
        println!("======================================");

        Ok(())
    }

    fn from_holder(game_data_holder: &GameDataHolder) -> Self {
        Self {
            dat_paths: game_data_holder.initial_dat_paths.clone(),
            quests: game_data_holder.quest_holder.clone(),
            ..Default::default()
        }
    }

    fn serialize_to_binary(&self) -> std::io::Result<()> {
        let mut res = Vec::new();

        let mut vals: Vec<_> = self.quests.values().collect();
        vals.sort_by(|a, b| a.id.cmp(&b.id));

        for quest in vals {
            for step in QuestNameDat::from_quest(quest) {
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

    fn load_skills(&mut self) -> Result<(), ()> {
        let mut d = "".to_string();

        File::open("./skill_ids.txt").unwrap().read_to_string(&mut d).unwrap();

        let mut ids = HashSet::new();

        for line in d.split("\n") {
            ids.insert(u32::from_str(line).unwrap());
        }

        let skill_grp = deserialize_dat::<SkillGrpDat>(
            self.dat_paths
                .get(&"skillgrp.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let (skill_name_table, skill_name) = deserialize_dat_with_string_dict::<SkillNameTableRecord, SkillNameDat>(
            self.dat_paths
                .get(&"skillname-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let mut string_dict = HashMap::new();

        for SkillNameTableRecord{val, id} in skill_name_table {
            string_dict.insert(id, val.0.clone());
        }

        string_dict.insert(u32::MAX, "NOT EXIST".to_string());

        if skill_grp.is_empty() {
            return Ok(())
        }

        let mut current_id = skill_grp.first().unwrap().id;
        let mut current_grps = vec![];

        let mut treed_names: HashMap<u32, HashMap<i16, HashMap<i16, SkillNameDat>>> = HashMap::new();

        for name in skill_name {
            if let Some(level) = treed_names.get_mut(&name.id) {
                if let Some(sub_level) = level.get_mut(&name.sub_level) {
                    sub_level.insert(name.sub_level, name);
                } else {
                    let mut sub_level = HashMap::new();
                    sub_level.insert(name.sub_level, name);
                    level.insert(name.level, sub_level);
                }
            } else {
                let mut level = HashMap::new();
                let mut sub_level = HashMap::new();
                sub_level.insert(name.sub_level, name);
                level.insert(name.level, sub_level);
                treed_names.insert(name.id, level);
            }
        }

        for record in skill_grp {
            if !ids.contains(&(record.id as u32)) {
                continue
            }

            if record.id != current_id {
                self.build_skill(&current_grps, &treed_names, &string_dict);

                current_id = record.id;
                current_grps.clear();
            }

            current_grps.push(record);
        }

        if !current_grps.is_empty() {
            self.build_skill(&current_grps, &treed_names, &string_dict);
        }

        Ok(())
    }

    fn get_name_record_or_default<'a>(
        &self,
        id: u32,
        level: i16,
        sub_level: i16,
        skill_names: &'a HashMap<u32, HashMap<i16, HashMap<i16, SkillNameDat>>>,
    ) -> &'a SkillNameDat {
        const DEFAULT_SKILL_NAME_DAT: SkillNameDat = SkillNameDat {
            id: 0,
            level: 0,
            sub_level: 0,
            prev_id: 0,
            prev_level: 0,
            prev_sub_level: 0,
            name: u32::MAX,
            desc: u32::MAX,
            desc_params: u32::MAX,
            enchant_name: u32::MAX,
            enchant_name_params: u32::MAX,
            enchant_desc: u32::MAX,
            enchant_desc_params: u32::MAX,
        };

        if let Some(a1) = skill_names.get(&id) {
            if let Some(a2) = a1.get(&level) {
                if let Some(a3) = a2.get(&sub_level) {
                    return a3;
                }
            }
        }

        &DEFAULT_SKILL_NAME_DAT
    }

    fn build_skill(&mut self, skill_grps: &Vec<SkillGrpDat>, skill_names: &HashMap<u32, HashMap<i16, HashMap<i16, SkillNameDat>>>, string_dict: &HashMap<u32, String>) {
        let first_grp = skill_grps.first().unwrap();
        let first_name = self.get_name_record_or_default(first_grp.id as u32, first_grp.level as i16, first_grp.sub_level, skill_names);

        let mut skill = Skill {
            id: SkillId(first_grp.id as u32),
            name: string_dict.get(&first_name.name).unwrap().clone(),
            description: string_dict.get(&first_name.desc).unwrap().clone(),
            skill_type: SkillType::from_u8(first_grp.skill_type).unwrap(),
            resist_cast: first_grp.resist_cast,
            magic_type: first_grp.magic_type,
            cast_style: first_grp.cast_style,
            skill_magic_type: first_grp.skill_magic_type,
            origin_skill: SkillId(first_grp.origin_skill as u32),
            is_double: first_grp.is_double == 1,
            animation: first_grp.animation.0.iter().map(|v| self.game_data_name.get(*v as usize).unwrap().clone()).collect(),
            visual_effect: VisualEffectId(first_grp.skill_visual_effect as u32),
            icon: self.game_data_name[first_grp.icon as usize].clone(),
            icon_panel: self.game_data_name[first_grp.icon_panel as usize].clone(),
            cast_bar_text_is_red: first_grp.cast_bar_text_is_red == 1,
            rumble_self: first_grp.rumble_self,
            rumble_target: first_grp.rumble_target,
            skill_levels: vec![],
            is_debuff: first_grp.debuff == 1,
        };

        let mut levels = vec![];
        let mut enchants: HashMap<u8, Vec<EnchantInfo>> = HashMap::new();

        for v in skill_grps.iter() {
            let skill_name = self.get_name_record_or_default(v.id as u32, v.level as i16, v.sub_level, skill_names);

            if v.sub_level == 0 {
                levels.push(SkillLevelInfo {
                    description_params: string_dict.get(&skill_name.desc_params).unwrap().clone(),
                    mp_cost: v.mp_consume,
                    hp_cost: v.hp_consume,
                    cast_range: v.cast_range,
                    hit_time: v.hit_time,
                    cool_time: v.cool_time,
                    reuse_delay: v.reuse_delay,
                    effect_point: v.effect_point,
                    available_enchants: vec![],
                });
            } else {
                let variant = v.sub_level / 1000;

                let enchant_level = EnchantLevelInfo {
                    description_params: string_dict.get(&skill_name.desc_params).unwrap().clone(),
                    enchant_name_params: string_dict.get(&skill_name.enchant_name_params).unwrap().clone(),
                    enchant_description_params: string_dict.get(&skill_name.enchant_desc_params).unwrap().clone(),
                    mp_cost: v.mp_consume,
                    hp_cost: v.hp_consume,
                    cast_range: v.cast_range,
                    hit_time: v.hit_time,
                    cool_time: v.cool_time,
                    reuse_delay: v.reuse_delay,
                    effect_point: v.effect_point,
                };

                if let Some(curr_level_enchants) = enchants.get_mut(&v.level){
                    if let Some(ei) = curr_level_enchants.get_mut(variant as  usize) {
                        ei.enchant_levels.push(enchant_level);
                    } else {
                        curr_level_enchants.push(
                            EnchantInfo {
                                enchant_type: variant as u32,
                                description: string_dict.get(&skill_name.desc).unwrap().clone(),
                                enchant_name: string_dict.get(&skill_name.enchant_name).unwrap().clone(),
                                enchant_description: string_dict.get(&skill_name.enchant_desc).unwrap().clone(),
                                is_debuff: v.debuff == 1,
                                enchant_levels: vec![enchant_level]
                            }
                        );
                    };
                } else {
                    enchants.insert(
                        v.level,
                        vec![EnchantInfo {
                            enchant_type: variant as u32,
                            description: string_dict.get(&skill_name.desc).unwrap().clone(),
                            enchant_name: string_dict.get(&skill_name.enchant_name).unwrap().clone(),
                            enchant_description: string_dict.get(&skill_name.enchant_desc).unwrap().clone(),
                            is_debuff: v.debuff == 1,
                            enchant_levels: vec![enchant_level]
                        }]
                    );
                }
            }
        };

        if levels.is_empty() {
            return;
        }

        for (key, value) in enchants {
            levels[key as usize - 1].available_enchants = value;
        }

        skill.skill_levels = levels;

        self.skills.insert(skill.id, skill);
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

        let vals = deserialize_dat::<QuestNameDat>(
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

    fn construct_quest(&mut self, current_steps: &Vec<QuestNameDat>) {
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
        let vals = deserialize_dat::<ItemNameDat>(
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
        let vals = deserialize_dat::<NpcStringDat>(
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
        let vals = deserialize_dat::<NpcNameDat>(
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
struct L2GameDataNameDat {
    value: STR,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct NpcStringDat {
    id: DWORD,
    value: ASCF,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct NpcNameDat {
    id: DWORD,
    name: ASCF,
    title: ASCF,
    title_color: Color,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct ItemNameDat {
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
struct QuestNameDat {
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

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
struct SkillGrpDat {
    id: USHORT,
    level: BYTE,
    sub_level: SHORT,
    skill_type: BYTE,
    //Выяснить чо такое
    resist_cast: BYTE,
    //Выяснить чо такое
    magic_type: BYTE,
    mp_consume: SHORT, //level
    cast_range: DWORD, //level
    //Выяснить какие есть
    cast_style: BYTE,
    hit_time: FLOAT, //level
    cool_time: FLOAT, //level
    reuse_delay: FLOAT, //level
    //Выяснить чо такое
    effect_point: DWORD, //level
    //Выяснить чо такое
    skill_magic_type: BYTE,
    //Выяснить чо такое
    origin_skill: SHORT,
    //Выяснить чо такое
    is_double: BYTE,
    //Собрать возможные, почему массив?
    animation: UVEC<DWORD>,
    skill_visual_effect: DWORD,
    icon: DWORD,
    icon_panel: DWORD,
    //Проверить бывает ли больше 1
    debuff: BYTE, //enchant override
    cast_bar_text_is_red: BYTE,
    //Для какого лвла эта заточка
    enchant_skill_level: BYTE, //enchant
    //Иконка варианта заточки
    enchant_icon: DWORD, //enchant
    hp_consume: SHORT, //level
    //Выяснить чо такое
    rumble_self: BYTE,
    //Выяснить чо такое
    rumble_target: BYTE,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
struct SkillNameTableRecord {
    val: ASCF,
    id: DWORD,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default, Copy)]
struct SkillNameDat {
    id: DWORD,
    level: SHORT,
    sub_level: SHORT,
    prev_id: DWORD,
    prev_level: SHORT,
    prev_sub_level: SHORT,
    name: DWORD,
    desc: DWORD,
    desc_params: DWORD,
    enchant_name: DWORD,
    enchant_name_params: DWORD,
    enchant_desc: DWORD,
    enchant_desc_params: DWORD,
}