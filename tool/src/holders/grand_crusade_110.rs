#![allow(clippy::needless_borrow)]
use crate::backend::{SkillEnchantAction, SkillEnchantEditWindowParams, StepAction, WindowParams};
use crate::data::{
    HuntingZoneId, InstantZoneId, ItemId, Location, NpcId, QuestId, SearchZoneId, SkillId,
    VisualEffectId,
};
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::quest::{
    GoalType, MarkType, Quest, QuestCategory, QuestReward, QuestStep, QuestType, StepGoal, Unk1,
    Unk2, UnkQLevel,
};
use crate::entity::skill::{EnchantInfo, EnchantLevelInfo, RacesSkillSoundInfo, Skill, SkillAnimation, SkillLevelInfo, SkillSoundInfo, SkillType, SoundInfo};
use crate::holders::{GameDataHolder, Loader};
use crate::util::l2_reader::{
    deserialize_dat, deserialize_dat_with_string_dict, save_dat, DatVariant,
};
use crate::util::{Color, ReadUnreal, UnrealCasts, UnrealReader, UnrealWriter, WriteUnreal, ASCF, BYTE, DWORD, FLOAT, FLOC, INT, LONG, SHORT, STR, USHORT, UVEC, WORD, L2StringTable};
use eframe::egui::Color32;
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Keys;
use std::fs::File;
use std::io::Read;
use std::ops::Index;
use std::path::Path;
use std::str::FromStr;
use std::thread;
use walkdir::DirEntry;
use crate::frontend::IS_SAVING;


#[derive(Default, Clone)]
pub struct L2GeneralStringTable {
    was_changed: bool,
    next_index: u32,
    inner: HashMap<u32, String>,
    reverse_map: HashMap<String, u32>,
}


impl L2GeneralStringTable {
    fn to_vec(&self) -> Vec<String>{
        let mut k: Vec<_> = self.keys().collect();
        k.sort();

        let mut res = Vec::with_capacity(k.len());

        for key in k {
            res.push( self.inner.get(key).unwrap().clone());
        }

        res
    }
}

impl L2StringTable for L2GeneralStringTable {
    fn keys(&self) -> Keys<u32, String> {
        self.inner.keys()
    }

    fn get(&self, key: &u32) -> Option<&String>{
        self.inner.get(key)
    }

    fn from_vec(values: Vec<String>) -> Self {
        let mut s = Self::default();

        for v in values {
            s.add(v);
        }

        s
    }

    fn get_index(&mut self, mut value: &str) -> u32 {
        const NONE_STR: &str = &"None";

        if value == "" {
            value = &NONE_STR
        }

        if let Some(i) = self.reverse_map.get(&value.to_lowercase()) {
            *i
        } else {
            self.was_changed = true;
            self.add(value.to_string())
        }
    }

    fn add(&mut self, value: String) -> u32 {
        self.reverse_map.insert(value.to_lowercase(), self.next_index);
        self.inner.insert(self.next_index, value);
        self.next_index += 1;

        self.next_index - 1
    }
}


#[derive(Default, Clone)]
pub struct L2SkillStringTable {
    next_index: u32,
    inner: HashMap<u32, String>,
    reverse_map: HashMap<String, u32>,
}

impl L2StringTable for L2SkillStringTable {
    fn keys(&self) -> Keys<u32, String> {
        self.inner.keys()
    }
    fn get(&self, key: &u32) -> Option<&String>{
        self.inner.get(key)
    }

    fn from_vec(values: Vec<String>) -> Self {
        let mut s = Self::default();

        for v in values {
            s.add(v);
        }

        s
    }

    fn get_index(&mut self, value: &str) -> u32 {
        if let Some(i) = self.reverse_map.get(value) {
            *i
        } else {
            self.add(value.to_string())
        }
    }

    fn add(&mut self, value: String) -> u32 {
        self.reverse_map.insert(value.clone(), self.next_index);
        self.inner.insert(self.next_index, value);
        self.next_index += 1;

        self.next_index - 1
    }
}

impl Index<usize> for L2SkillStringTable {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(&(index as u32)).unwrap()
    }
}
impl Index<u32> for L2SkillStringTable {
    type Output = String;

    fn index(&self, index: u32) -> &Self::Output {
        self.inner.get(&index ).unwrap()
    }
}
impl Index<usize> for L2GeneralStringTable {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(&(index as u32)).unwrap()
    }
}
impl Index<u32> for L2GeneralStringTable {
    type Output = String;

    fn index(&self, index: u32) -> &Self::Output {
        self.inner.get(&index ).unwrap()
    }
}

#[derive(Default)]
pub struct Loader110 {
    game_data_name: L2GeneralStringTable,
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
    fn get_skills(&self) -> HashMap<SkillId, Skill> {
        self.skills.clone()
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
    fn get_string_table(&self) -> L2GeneralStringTable {
        self.game_data_name.clone()
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
            game_data_name: game_data_holder.game_string_table.clone(),
            skills: game_data_holder.skill_holder.clone(),
            ..Default::default()
        }
    }

    fn serialize_to_binary(
        &mut self,
    ) -> std::io::Result<()> {
        *IS_SAVING.write().unwrap() = true;

        let mut res = Vec::new();

        let mut vals: Vec<_> = self.quests.values().collect();
        vals.sort_by(|a, b| a.id.cmp(&b.id));

        for quest in vals {
            for step in QuestNameDat::from_quest(quest) {
                res.push(step);
            }
        }

        let mut skill_grp = vec![];
        let mut skill_string_table = L2SkillStringTable::from_vec(vec![]);

        let mut skill_name = vec![];
        let mut skill_sound = vec![];
        let mut skill_sound_src = vec![];

        let mut vals: Vec<_> = self.skills.values().collect();
        vals.sort_by(|a, b| a.id.cmp(&b.id));

        for skill in vals {
            if skill.skill_levels.is_empty() {
                continue;
            }

            skill_sound.push(skill.sound_data(&mut self.game_data_name));
            skill_sound_src.push(skill.sound_source_data());

            let mut base_skill_grp = SkillGrpDat::default();
            let mut base_skill_name = SkillNameDat::default();

            base_skill_grp.fill_from_skill(skill, &mut self.game_data_name);
            base_skill_name.fill_from_skill(skill, &mut skill_string_table);

            let mut first = true;
            for level in &skill.skill_levels {
                let mut base_skill_grp = base_skill_grp.clone();
                let mut base_skill_name = base_skill_name.clone();

                base_skill_grp.fill_from_level(level, &mut self.game_data_name, first);
                base_skill_name.fill_from_level(level, &mut skill_string_table, first);

                skill_grp.push(base_skill_grp.clone());
                skill_name.push(base_skill_name);

                first = false;

                for enchant in &level.available_enchants {
                    let enchant = &enchant.inner;
                    let mut base_skill_grp = base_skill_grp.clone();
                    let mut base_skill_name = base_skill_name.clone();

                    base_skill_grp.fill_from_enchant(&enchant, &mut self.game_data_name, level.level);
                    base_skill_name.fill_from_enchant(&enchant, &mut skill_string_table, level.level);

                    for enchant_level in &enchant.enchant_levels {
                        base_skill_grp.fill_from_enchant_level(&enchant_level, &mut self.game_data_name, enchant.enchant_type);
                        base_skill_name.fill_from_enchant_level(&enchant_level, &mut skill_string_table, enchant.enchant_type);

                        skill_grp.push(base_skill_grp.clone());
                        skill_name.push(base_skill_name);
                    }
                }
            }
        }

        let quest_path = self.dat_paths
            .get(&"questname-ru.dat".to_string())
            .unwrap()
            .clone();
        let l2_game_data_name = self.dat_paths
            .get(&"l2gamedataname.dat".to_string())
            .unwrap()
            .clone();
        let v = self.game_data_name.to_vec();
        let skill_sound_src_path = self.dat_paths
            .get(&"skillsoundsource.dat".to_string())
            .unwrap()
            .clone();
        let skill_name_path = self.dat_paths
            .get(&"skillname-ru.dat".to_string())
            .unwrap()
            .clone();
        let skill_grp_path = self.dat_paths
            .get(&"skillgrp.dat".to_string())
            .unwrap()
            .clone();
        let skill_sound_path = self.dat_paths
            .get(&"skillsoundgrp.dat".to_string())
            .unwrap()
            .clone();

        let gdn_changed = self.game_data_name.was_changed;

        thread::spawn(move || {
            let skill_name_handel = thread::spawn(move || {
                if let Err(e) = save_dat(
                    skill_name_path.path(),
                    DatVariant::DoubleArray(SkillNameTableRecord::from_table(skill_string_table), skill_name),
                ) {
                    println!("{e:?}");
                } else {
                    println!("Skill Name saved");
                }
            });
            let skill_grp_handel = thread::spawn(move || {
                if let Err(e) = save_dat(
                    skill_grp_path.path(),
                    DatVariant::<(), SkillGrpDat>::Array(skill_grp),
                ) {
                    println!("{e:?}");
                } else {
                    println!("Skill Grp saved");
                }
            });
            let skill_sound_handel = thread::spawn(move || {
                if let Err(e) = save_dat(
                    skill_sound_path.path(),
                    DatVariant::<(), SkillSoundDat>::Array(skill_sound),
                ) {
                    println!("{e:?}");
                } else {
                    println!("Skill Sound saved");
                }
            });
            let skill_sound_src_handel = thread::spawn(move || {
                if let Err(e) = save_dat(
                    skill_sound_src_path.path(),
                    DatVariant::<(), SkillSoundSourceDat>::Array(skill_sound_src),
                ) {
                    println!("{e:?}");
                } else {
                    println!("Skill Sound Src saved");
                }
            });
            let quest_handel = thread::spawn(move || {
                if let Err(e) = save_dat(
                    quest_path.path(),
                    DatVariant::<(), QuestNameDat>::Array(res),
                ) {
                    println!("{e:?}");
                } else {
                    println!("Quest Name saved");
                }
            });

            let gdn_handel = if gdn_changed {
                Some(thread::spawn(move || {
                    if let Err(e) = save_dat(
                        l2_game_data_name.path(),
                        DatVariant::<(), String>::Array(v),
                    ) {
                        println!("{e:?}");
                    } else {
                        println!("Game Data Name saved");
                    }
                }))
            } else {
                None
            };

            if let Some(c) = gdn_handel {
                let _ = c.join();
            }

            let _ = skill_name_handel.join();
            let _ = skill_grp_handel.join();
            let _ = skill_sound_handel.join();
            let _ = skill_sound_src_handel.join();
            let _ = quest_handel.join();

            println!("Binaries Saved");

            *IS_SAVING.write().unwrap() = false;
        });

        Ok(())
    }
}

impl SkillNameTableRecord {
    fn from_table(value: L2SkillStringTable) -> Vec<Self> {
        let mut keys: Vec<_> = value.keys().collect();
        keys.sort();

        let mut res = Vec::with_capacity(keys.len());

        for key in keys {
            res.push(Self{ val: ASCF(value.get(key).unwrap().clone()), id: *key })
        }

        res
    }
}

impl Skill {
    fn sound_source_data(&self) -> SkillSoundSourceDat {
        SkillSoundSourceDat {
            id: self.id.0,
            spell_1_effect: self.sound_info.inner.spell_effect_1.source,
            spell_2_effect: self.sound_info.inner.spell_effect_2.source,
            spell_3_effect: self.sound_info.inner.spell_effect_3.source,
            shot_1_effect: self.sound_info.inner.shot_effect_1.source,
            shot_2_effect: self.sound_info.inner.shot_effect_2.source,
            shot_3_effect: self.sound_info.inner.shot_effect_3.source,
            exp_1_effect: self.sound_info.inner.exp_effect_1.source,
            exp_2_effect: self.sound_info.inner.exp_effect_1.source,
            exp_3_effect: self.sound_info.inner.exp_effect_1.source,
        }
    }
    fn sound_data(&self, string_table: &mut L2GeneralStringTable) -> SkillSoundDat {
        SkillSoundDat {
            id: self.id.0,
            level: 1,
            spell_1_effect: string_table.get_index(&self.sound_info.inner.spell_effect_1.sound),
            spell_2_effect: string_table.get_index(&self.sound_info.inner.spell_effect_2.sound),
            spell_3_effect: string_table.get_index(&self.sound_info.inner.spell_effect_3.sound),
            spell_1_vol: self.sound_info.inner.spell_effect_1.vol,
            spell_1_rad: self.sound_info.inner.spell_effect_1.rad,
            spell_1_delay: self.sound_info.inner.spell_effect_1.delay,
            spell_2_vol: self.sound_info.inner.spell_effect_2.vol,
            spell_2_rad: self.sound_info.inner.spell_effect_2.rad,
            spell_2_delay: self.sound_info.inner.spell_effect_2.delay,
            spell_3_vol: self.sound_info.inner.spell_effect_3.vol,
            spell_3_rad: self.sound_info.inner.spell_effect_3.rad,
            spell_3_delay: self.sound_info.inner.spell_effect_3.delay,
            shot_1_effect: string_table.get_index(&self.sound_info.inner.shot_effect_1.sound),
            shot_2_effect: string_table.get_index(&self.sound_info.inner.shot_effect_2.sound),
            shot_3_effect: string_table.get_index(&self.sound_info.inner.shot_effect_3.sound),
            shot_1_vol: self.sound_info.inner.shot_effect_1.vol,
            shot_1_rad: self.sound_info.inner.shot_effect_1.rad,
            shot_1_delay: self.sound_info.inner.shot_effect_1.delay,
            shot_2_vol: self.sound_info.inner.shot_effect_2.vol,
            shot_2_rad: self.sound_info.inner.shot_effect_2.rad,
            shot_2_delay: self.sound_info.inner.shot_effect_2.delay,
            shot_3_vol: self.sound_info.inner.shot_effect_3.vol,
            shot_3_rad: self.sound_info.inner.shot_effect_3.rad,
            shot_3_delay: self.sound_info.inner.shot_effect_3.delay,
            exp_1_effect: string_table.get_index(&self.sound_info.inner.exp_effect_1.sound),
            exp_2_effect: string_table.get_index(&self.sound_info.inner.exp_effect_2.sound),
            exp_3_effect: string_table.get_index(&self.sound_info.inner.exp_effect_3.sound),
            exp_1_vol: self.sound_info.inner.shot_effect_1.vol,
            exp_1_rad: self.sound_info.inner.shot_effect_1.rad,
            exp_1_delay: self.sound_info.inner.shot_effect_1.delay,
            exp_2_vol: self.sound_info.inner.shot_effect_2.vol,
            exp_2_rad: self.sound_info.inner.shot_effect_2.rad,
            exp_2_delay: self.sound_info.inner.shot_effect_2.delay,
            exp_3_vol: self.sound_info.inner.shot_effect_1.vol,
            exp_3_rad: self.sound_info.inner.shot_effect_2.rad,
            exp_3_delay: self.sound_info.inner.shot_effect_3.delay,
            mfighter_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mfighter),
            ffighter_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.ffighter),
            mmagic_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mmagic),
            fmagic_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fmagic),
            melf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.melf),
            felf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.felf),
            mdark_elf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mdark_elf),
            fdark_elf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fdark_elf),
            mdwarf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mdwarf),
            fdwarf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fdwarf),
            morc_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.morc),
            forc_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.forc),
            mshaman_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mshaman),
            fshaman_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fshaman),
            mkamael_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mkamael),
            fkamael_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fkamael),
            mertheia_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mertheia),
            fertheia_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fertheia),
            mextra_throw: string_table.get_index(&self.sound_info.inner.mextra_throw),
            mfighter_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mfighter),
            ffighter_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.ffighter),
            mmagic_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mmagic),
            fmagic_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fmagic),
            melf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.melf),
            felf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.felf),
            mdark_elf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mdark_elf),
            fdark_elf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fdark_elf),
            mdwarf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mdwarf),
            fdwarf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fdwarf),
            morc_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.morc),
            forc_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.forc),
            mshaman_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mshaman),
            fshaman_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fshaman),
            mkamael_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mkamael),
            fkamael_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fkamael),
            mertheia_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mertheia),
            fertheia_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fertheia),
            fextra_throw: string_table.get_index(&self.sound_info.inner.fextra_throw),
            cast_volume: self.sound_info.inner.vol,
            cast_rad: self.sound_info.inner.rad,
        }
    }
}

impl SkillNameDat {
    #[inline]
    fn fill_from_enchant_level(&mut self, enchant_level: &EnchantLevelInfo, skill_string_table: &mut L2SkillStringTable, enchant_type: u32) {
        self.enchant_name_params = skill_string_table.get_index(&enchant_level.enchant_name_params);
        self.enchant_desc_params = skill_string_table.get_index(&enchant_level.enchant_description_params);
        self.desc_params = skill_string_table.get_index(&enchant_level.skill_description_params);

        self.sub_level = (enchant_type * 1000 + enchant_level.level) as SHORT;
        self.prev_sub_level = if enchant_level.level > 1 { self.sub_level - 1 } else { 0 };
    }
    #[inline]
    fn fill_from_enchant(&mut self, enchant: &EnchantInfo, skill_string_table: &mut L2SkillStringTable, level: u32) {
        self.enchant_desc = skill_string_table.get_index(&enchant.enchant_description);
        self.enchant_name = skill_string_table.get_index(&enchant.enchant_name);
        self.desc = skill_string_table.get_index(if let Some(v) = &enchant.skill_description {v} else { &"" });
        self.prev_level = level as SHORT;
    }
    #[inline]
    fn fill_from_level(&mut self, level: &SkillLevelInfo, skill_string_table: &mut L2SkillStringTable, first: bool) {
        self.level = level.level as SHORT;
        self.prev_level = (level.level - 1) as SHORT;
        if !first {
            self.prev_id = self.id;
            self.prev_sub_level = 0i16;
            self.desc = skill_string_table.get_index(if let Some(v) = &level.description { v } else { &"\0" });
        }
        self.desc_params = skill_string_table.get_index(&level.description_params);
    }
    #[inline]
    fn fill_from_skill(&mut self, skill: &Skill, skill_string_table: &mut L2SkillStringTable) {
        self.id = skill.id.0;
        self.sub_level = 0;
        self.prev_id = 0;
        self.prev_level = -1;
        self.prev_sub_level = -1;
        self.name = skill_string_table.get_index(&skill.name);
        self.desc = skill_string_table.get_index(&skill.description);
    }
}

impl SkillGrpDat {
    #[inline]
    fn fill_from_enchant_level(&mut self, enchant_level: &EnchantLevelInfo, game_data_name: &mut L2GeneralStringTable, enchant_type: u32) {
        self.sub_level = (enchant_type*1000 + enchant_level.level) as SHORT;
        self.mp_consume = enchant_level.mp_cost;
        self.cast_range = enchant_level.cast_range;
        self.hit_time = enchant_level.hit_time;
        self.cool_time = enchant_level.cool_time;
        self.reuse_delay = enchant_level.reuse_delay;
        self.effect_point = enchant_level.effect_point;
        self.hp_consume = enchant_level.hp_cost;

        if let Some(v) = &enchant_level.icon {
            self.icon = game_data_name.get_index(v);
        }

        if let Some(v) = &enchant_level.icon_panel {
            self.icon_panel = game_data_name.get_index(v);
        }
    }
    #[inline]
    fn fill_from_enchant(&mut self, enchant: &EnchantInfo, game_data_name: &mut L2GeneralStringTable, level: u32) {
        self.debuff = enchant.is_debuff.to_u32_bool() as BYTE;
        self.enchant_icon = game_data_name.get_index(&enchant.enchant_icon);
        self.enchant_skill_level = level as BYTE;
    }
    #[inline]
    fn fill_from_level(&mut self, level: &SkillLevelInfo, game_data_name: &mut L2GeneralStringTable, first: bool) {
        self.level = level.level as BYTE;
        self.mp_consume = level.mp_cost;
        self.cast_range = level.cast_range;
        self.hit_time = level.hit_time;
        self.cool_time = level.cool_time;
        self.reuse_delay = level.reuse_delay;
        self.effect_point = level.effect_point;
        self.hp_consume = level.hp_cost;

        if !first {
            if let Some(v) = &level.icon {
                self.icon = game_data_name.get_index(v);
            }

            if let Some(v) = &level.icon_panel {
                self.icon_panel = game_data_name.get_index(v);
            }
        }
    }
    #[inline]
    fn fill_from_skill(&mut self, skill: &Skill, game_data_name: &mut L2GeneralStringTable) {
        self.id = skill.id.0 as USHORT;
        self.skill_type = skill.skill_type.to_u8().unwrap();
        self.resist_cast = skill.resist_cast;
        self.magic_type = skill.magic_type;
        self.cast_style = skill.cast_style;
        self.skill_magic_type = skill.skill_magic_type;
        self.origin_skill = skill.origin_skill.0 as SHORT;
        self.is_double = skill.is_double.to_u32_bool() as BYTE;
        self.animation = UVEC(skill.animations.iter().map(|v| game_data_name.get_index(&v.to_string())).collect());
        self.skill_visual_effect = skill.visual_effect.0;
        self.icon = game_data_name.get_index(&skill.icon);
        self.icon_panel = game_data_name.get_index(&skill.icon_panel);
        self.debuff = skill.is_debuff.to_u32_bool() as BYTE;
        self.cast_bar_text_is_red = skill.is_debuff.to_u32_bool() as BYTE;
        self.enchant_skill_level = 0;
        self.enchant_icon = game_data_name.get_index(&"None");
        self.rumble_self = skill.rumble_self;
        self.rumble_target = skill.rumble_target;
    }
}

impl Loader110 {
    fn load_game_data_name(path: &Path) -> Result<L2GeneralStringTable, ()> {
        match deserialize_dat(path) {
            Ok(r) => Ok(L2GeneralStringTable::from_vec(r)),
            Err(e) => Err(e)
        }
    }

    fn load_skills(&mut self) -> Result<(), ()> {
        let mut d = "".to_string();

        //TODO: Remove!
        let mut ids = HashSet::new();
        {
            File::open("./skill_ids.txt")
                .unwrap()
                .read_to_string(&mut d)
                .unwrap();

            for line in d.split('\n') {
                ids.insert(u32::from_str(line).unwrap());
            }
        }

        let skill_grp = deserialize_dat::<SkillGrpDat>(
            self.dat_paths
                .get(&"skillgrp.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let skill_sound = deserialize_dat::<SkillSoundDat>(
            self.dat_paths
                .get(&"skillsoundgrp.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let mut sound_map = HashMap::new();

        for s in skill_sound {
            sound_map.insert(s.id, s);
        }

        let skill_sound_source = deserialize_dat::<SkillSoundSourceDat>(
            self.dat_paths
                .get(&"skillsoundsource.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let mut sound_source_map = HashMap::new();

        for s in skill_sound_source {
            sound_source_map.insert(s.id, s);
        }

        let (skill_name_table, skill_name) =
            deserialize_dat_with_string_dict::<SkillNameTableRecord, SkillNameDat>(
                self.dat_paths
                    .get(&"skillname-ru.dat".to_string())
                    .unwrap()
                    .path(),
            )?;

        let mut string_dict = HashMap::new();

        for SkillNameTableRecord { val, id } in skill_name_table {
            string_dict.insert(id, val.0.clone());
        }

        string_dict.insert(u32::MAX, "NOT EXIST".to_string());

        if skill_grp.is_empty() {
            return Ok(());
        }

        let mut current_id = skill_grp.first().unwrap().id;
        let mut current_grps = vec![];

        let mut treed_names: HashMap<u32, HashMap<i16, HashMap<i16, SkillNameDat>>> =
            HashMap::new();

        for name in skill_name {
            if let Some(level) = treed_names.get_mut(&name.id) {
                if let Some(sub_level) = level.get_mut(&name.level) {
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

        let mut anim = HashSet::new();

        for record in skill_grp {
            record
                .animation
                .0
                .iter()
                .for_each(|v|
                    {
                        anim.insert(self.game_data_name.get(v).unwrap().to_uppercase());
                    });

            if !ids.contains(&(record.id as u32)) {
                continue;
            }

            if record.id != current_id {
                self.build_skill(
                    &current_grps,
                    &treed_names,
                    &string_dict,
                    &sound_map,
                    &sound_source_map,
                );

                current_id = record.id;
                current_grps.clear();
            }

            current_grps.push(record);
        }

        if !current_grps.is_empty() {
            self.build_skill(
                &current_grps,
                &treed_names,
                &string_dict,
                &sound_map,
                &sound_source_map,
            );
        }
        let mut c: Vec<_> = anim.iter().collect();
        c.sort();

        println!("{c:#?}");
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

    fn build_skill(
        &mut self,
        skill_grps: &[SkillGrpDat],
        skill_names: &HashMap<u32, HashMap<i16, HashMap<i16, SkillNameDat>>>,
        string_dict: &HashMap<u32, String>,
        sound_map: &HashMap<u32, SkillSoundDat>,
        sound_source_map: &HashMap<u32, SkillSoundSourceDat>,
    ) {
        let first_grp = skill_grps.first().unwrap();
        let first_name = self.get_name_record_or_default(
            first_grp.id as u32,
            first_grp.level as i16,
            first_grp.sub_level,
            skill_names,
        );

        let sound = if let Some(s) = sound_map.get(&(first_grp.id as u32)) {
            s
        } else {
            &SOUND_DEFAULT
        };

        let sound_source = if let Some(s) = sound_source_map.get(&(first_grp.id as u32)) {
            s
        } else {
            &SOUND_SOURCE_DEFAULT
        };

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
            animations: first_grp
                .animation
                .0
                .iter()
                .map(|v|
                     {
                         let key = self.game_data_name.get(v).unwrap().to_uppercase();
                         SkillAnimation::from_str(&key).expect(&format!("{}", key))
                     })
                .collect(),
            visual_effect: VisualEffectId(first_grp.skill_visual_effect),
            icon: self.game_data_name[first_grp.icon as usize].clone(),
            icon_panel: self.game_data_name[first_grp.icon_panel as usize].clone(),
            cast_bar_text_is_red: first_grp.cast_bar_text_is_red == 1,
            rumble_self: first_grp.rumble_self,
            rumble_target: first_grp.rumble_target,
            skill_levels: vec![],
            is_debuff: first_grp.debuff == 1,
            sound_info: WindowParams {
                inner: SkillSoundInfo {
                    spell_effect_1: SoundInfo {
                        sound: self.game_data_name[sound.spell_1_effect as usize].clone(),
                        vol: sound.spell_1_vol,
                        rad: sound.spell_1_rad,
                        delay: sound.spell_1_delay,
                        source: sound_source.spell_1_effect,
                    },
                    spell_effect_2: SoundInfo {
                        sound: self.game_data_name[sound.spell_2_effect as usize].clone(),
                        vol: sound.spell_2_vol,
                        rad: sound.spell_2_rad,
                        delay: sound.spell_2_delay,
                        source: sound_source.spell_2_effect,
                    },
                    spell_effect_3: SoundInfo {
                        sound: self.game_data_name[sound.spell_3_effect as usize].clone(),
                        vol: sound.spell_3_vol,
                        rad: sound.spell_3_rad,
                        delay: sound.spell_3_delay,
                        source: sound_source.spell_3_effect,
                    },
                    shot_effect_1: SoundInfo {
                        sound: self.game_data_name[sound.shot_1_effect as usize].clone(),
                        vol: sound.shot_1_vol,
                        rad: sound.shot_1_rad,
                        delay: sound.shot_1_delay,
                        source: sound_source.shot_1_effect,
                    },
                    shot_effect_2: SoundInfo {
                        sound: self.game_data_name[sound.shot_2_effect as usize].clone(),
                        vol: sound.shot_2_vol,
                        rad: sound.shot_2_rad,
                        delay: sound.shot_2_delay,
                        source: sound_source.shot_2_effect,
                    },
                    shot_effect_3: SoundInfo {
                        sound: self.game_data_name[sound.shot_3_effect as usize].clone(),
                        vol: sound.shot_3_vol,
                        rad: sound.shot_3_rad,
                        delay: sound.shot_3_delay,
                        source: sound_source.shot_3_effect,
                    },
                    exp_effect_1: SoundInfo {
                        sound: self.game_data_name[sound.exp_1_effect as usize].clone(),
                        vol: sound.exp_1_vol,
                        rad: sound.exp_1_rad,
                        delay: sound.exp_1_delay,
                        source: sound_source.exp_1_effect,
                    },
                    exp_effect_2: SoundInfo {
                        sound: self.game_data_name[sound.exp_2_effect as usize].clone(),
                        vol: sound.exp_2_vol,
                        rad: sound.exp_2_rad,
                        delay: sound.exp_2_delay,
                        source: sound_source.exp_2_effect,
                    },
                    exp_effect_3: SoundInfo {
                        sound: self.game_data_name[sound.exp_3_effect as usize].clone(),
                        vol: sound.exp_3_vol,
                        rad: sound.exp_3_rad,
                        delay: sound.exp_3_delay,
                        source: sound_source.exp_3_effect,
                    },
                    sound_before_cast: RacesSkillSoundInfo {
                        mfighter: self.game_data_name[sound.mfighter_cast as usize].clone(),
                        ffighter: self.game_data_name[sound.ffighter_cast as usize].clone(),
                        mmagic: self.game_data_name[sound.mmagic_cast as usize].clone(),
                        fmagic: self.game_data_name[sound.fmagic_cast as usize].clone(),
                        melf: self.game_data_name[sound.melf_cast as usize].clone(),
                        felf: self.game_data_name[sound.felf_cast as usize].clone(),
                        mdark_elf: self.game_data_name[sound.mdark_elf_cast as usize].clone(),
                        fdark_elf: self.game_data_name[sound.fdark_elf_cast as usize].clone(),
                        mdwarf: self.game_data_name[sound.mdwarf_cast as usize].clone(),
                        fdwarf: self.game_data_name[sound.fdwarf_cast as usize].clone(),
                        morc: self.game_data_name[sound.morc_cast as usize].clone(),
                        forc: self.game_data_name[sound.forc_cast as usize].clone(),
                        mshaman: self.game_data_name[sound.mshaman_cast as usize].clone(),
                        fshaman: self.game_data_name[sound.fshaman_cast as usize].clone(),
                        mkamael: self.game_data_name[sound.mkamael_cast as usize].clone(),
                        fkamael: self.game_data_name[sound.fkamael_cast as usize].clone(),
                        mertheia: self.game_data_name[sound.mertheia_cast as usize].clone(),
                        fertheia: self.game_data_name[sound.fertheia_cast as usize].clone(),
                    },
                    sound_after_cast: RacesSkillSoundInfo {
                        mfighter: self.game_data_name[sound.mfighter_magic as usize].clone(),
                        ffighter: self.game_data_name[sound.ffighter_magic as usize].clone(),
                        mmagic: self.game_data_name[sound.mmagic_magic as usize].clone(),
                        fmagic: self.game_data_name[sound.fmagic_magic as usize].clone(),
                        melf: self.game_data_name[sound.melf_magic as usize].clone(),
                        felf: self.game_data_name[sound.felf_magic as usize].clone(),
                        mdark_elf: self.game_data_name[sound.mdark_elf_magic as usize].clone(),
                        fdark_elf: self.game_data_name[sound.fdark_elf_magic as usize].clone(),
                        mdwarf: self.game_data_name[sound.mdwarf_magic as usize].clone(),
                        fdwarf: self.game_data_name[sound.fdwarf_magic as usize].clone(),
                        morc: self.game_data_name[sound.morc_magic as usize].clone(),
                        forc: self.game_data_name[sound.forc_magic as usize].clone(),
                        mshaman: self.game_data_name[sound.mshaman_magic as usize].clone(),
                        fshaman: self.game_data_name[sound.fshaman_magic as usize].clone(),
                        mkamael: self.game_data_name[sound.mkamael_magic as usize].clone(),
                        fkamael: self.game_data_name[sound.fkamael_magic as usize].clone(),
                        mertheia: self.game_data_name[sound.mertheia_magic as usize].clone(),
                        fertheia: self.game_data_name[sound.fertheia_magic as usize].clone(),
                    },
                    mextra_throw: self.game_data_name[sound.mextra_throw as usize].clone(),
                    fextra_throw: self.game_data_name[sound.fextra_throw as usize].clone(),
                    vol: sound.cast_volume,
                    rad: sound.cast_rad,
                },
                opened: false,
                original_id: (),
                action: (),
                params: (),
            },
        };

        let mut levels = vec![];
        let mut enchants: HashMap<u8, HashMap<i16, EnchantInfo>> = HashMap::new();

        for v in skill_grps.iter() {
            let skill_name = self.get_name_record_or_default(
                v.id as u32,
                v.level as i16,
                v.sub_level,
                skill_names,
            );

            if v.sub_level == 0 {
                let desc = if skill_name.desc == first_name.desc {
                    None
                } else {
                    let c = string_dict
                        .get(&skill_name.desc)
                        .unwrap();
                    if c == "" || c =="\0" {
                        None
                    } else {
                        Some(c.clone())
                    }
                };

                levels.push(SkillLevelInfo {
                    level: v.level as u32,
                    description_params: string_dict.get(&skill_name.desc_params).unwrap().clone(),
                    mp_cost: v.mp_consume,
                    hp_cost: v.hp_consume,
                    cast_range: v.cast_range,
                    hit_time: v.hit_time,
                    cool_time: v.cool_time,
                    reuse_delay: v.reuse_delay,
                    effect_point: v.effect_point,
                    icon: if v.icon == first_grp.icon { None } else { Some(self.game_data_name.get(&v.icon).unwrap().clone()) },
                    icon_panel: if v.icon_panel == first_grp.icon_panel { None } else { Some(self.game_data_name.get(&v.icon_panel).unwrap().clone()) },
                    description: desc,
                    available_enchants: vec![],
                });
            } else {
                let variant = v.sub_level / 1000;

                let enchant_level = EnchantLevelInfo {
                    level: v.sub_level as u32 - variant as u32 * 1000,
                    skill_description_params: string_dict
                        .get(&skill_name.desc_params)
                        .unwrap()
                        .clone(),
                    enchant_name_params: string_dict
                        .get(&skill_name.enchant_name_params)
                        .unwrap()
                        .clone(),
                    enchant_description_params: string_dict
                        .get(&skill_name.enchant_desc_params)
                        .unwrap()
                        .clone(),
                    mp_cost: v.mp_consume,
                    hp_cost: v.hp_consume,
                    cast_range: v.cast_range,
                    hit_time: v.hit_time,
                    cool_time: v.cool_time,
                    reuse_delay: v.reuse_delay,
                    effect_point: v.effect_point,
                    icon: if v.icon == first_grp.icon { None } else { Some(self
                        .game_data_name
                        .get(&v.icon)
                        .unwrap()
                        .clone()) },
                    icon_panel: if v.icon_panel == first_grp.icon_panel { None } else { Some(self
                        .game_data_name
                        .get(&v.icon_panel)
                        .unwrap()
                        .clone()) },
                };

                if let Some(curr_level_enchants) = enchants.get_mut(&v.level) {
                    if let Some(ei) = curr_level_enchants.get_mut(&variant) {
                        ei.enchant_levels.push(enchant_level);
                    } else {
                        let desc = if skill_name.desc == first_name.desc {
                            None
                        } else {
                            let c = string_dict
                                .get(&skill_name.desc)
                                .unwrap();
                            if c == "" {
                                None
                            } else {
                                Some(c.clone())
                            }
                        };

                        curr_level_enchants.insert(
                            variant,
                            EnchantInfo {
                                enchant_type: variant as u32,
                                skill_description: desc,
                                enchant_name: string_dict
                                    .get(&skill_name.enchant_name)
                                    .unwrap()
                                    .clone(),
                                enchant_icon: self
                                    .game_data_name
                                    .get(&v.enchant_icon)
                                    .unwrap()
                                    .clone(),
                                enchant_description: string_dict
                                    .get(&skill_name.enchant_desc)
                                    .unwrap()
                                    .clone(),
                                is_debuff: v.debuff == 1,
                                enchant_levels: vec![enchant_level],
                            },
                        );
                    };
                } else {
                    let mut infos = HashMap::new();

                    let desc = if skill_name.desc == first_name.desc {
                        None
                    } else {
                        let c = string_dict
                            .get(&skill_name.desc)
                            .unwrap();
                        if c == "" {
                            None
                        } else {
                            Some(c.clone())
                        }
                    };

                    infos.insert(
                        variant,
                        EnchantInfo {
                            enchant_type: variant as u32,
                            skill_description: desc,
                            enchant_name: string_dict
                                .get(&skill_name.enchant_name)
                                .unwrap()
                                .clone(),
                            enchant_icon: self
                                .game_data_name
                                .get(&v.enchant_icon)
                                .unwrap()
                                .clone(),
                            enchant_description: string_dict
                                .get(&skill_name.enchant_desc)
                                .unwrap()
                                .clone(),
                            is_debuff: v.debuff == 1,
                            enchant_levels: vec![enchant_level],
                        },
                    );

                    enchants.insert(v.level, infos);
                }
            }
        }

        if levels.is_empty() {
            return;
        }

        for (key, mut value) in enchants {
            let mut inner_keys: Vec<_> = value.drain().collect();
            inner_keys.sort_by(|(a_i, _), (b_i, _)| a_i.cmp(&b_i));

            for (_, v) in inner_keys {
                levels[key as usize - 1]
                    .available_enchants
                    .push(WindowParams {
                        params: SkillEnchantEditWindowParams {
                            current_level_index: v.enchant_levels.len() - 1,
                        },
                        inner: v,
                        opened: false,
                        original_id: (),
                        action: SkillEnchantAction::None,
                    });
            }
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
                name: if let Some(name) = self.game_data_name.get(&v.name_link) {
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
    hit_time: FLOAT,    //level
    cool_time: FLOAT,   //level
    reuse_delay: FLOAT, //level
    //Выяснить чо такое
    effect_point: INT, //level
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
    hp_consume: SHORT,   //level
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

#[derive(Debug, Copy, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
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

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default, Copy)]
struct SkillSoundDat {
    id: DWORD,
    level: DWORD,

    spell_1_effect: DWORD,
    spell_2_effect: DWORD,
    spell_3_effect: DWORD,
    spell_1_vol: FLOAT,
    spell_1_rad: FLOAT,
    spell_1_delay: FLOAT,
    spell_2_vol: FLOAT,
    spell_2_rad: FLOAT,
    spell_2_delay: FLOAT,
    spell_3_vol: FLOAT,
    spell_3_rad: FLOAT,
    spell_3_delay: FLOAT,

    shot_1_effect: DWORD,
    shot_2_effect: DWORD,
    shot_3_effect: DWORD,
    shot_1_vol: FLOAT,
    shot_1_rad: FLOAT,
    shot_1_delay: FLOAT,
    shot_2_vol: FLOAT,
    shot_2_rad: FLOAT,
    shot_2_delay: FLOAT,
    shot_3_vol: FLOAT,
    shot_3_rad: FLOAT,
    shot_3_delay: FLOAT,

    exp_1_effect: DWORD,
    exp_2_effect: DWORD,
    exp_3_effect: DWORD,
    exp_1_vol: FLOAT,
    exp_1_rad: FLOAT,
    exp_1_delay: FLOAT,
    exp_2_vol: FLOAT,
    exp_2_rad: FLOAT,
    exp_2_delay: FLOAT,
    exp_3_vol: FLOAT,
    exp_3_rad: FLOAT,
    exp_3_delay: FLOAT,

    mfighter_cast: DWORD,
    ffighter_cast: DWORD,
    mmagic_cast: DWORD,
    fmagic_cast: DWORD,
    melf_cast: DWORD,
    felf_cast: DWORD,
    mdark_elf_cast: DWORD,
    fdark_elf_cast: DWORD,
    mdwarf_cast: DWORD,
    fdwarf_cast: DWORD,
    morc_cast: DWORD,
    forc_cast: DWORD,
    mshaman_cast: DWORD,
    fshaman_cast: DWORD,
    mkamael_cast: DWORD,
    fkamael_cast: DWORD,
    mertheia_cast: DWORD,
    fertheia_cast: DWORD,

    mextra_throw: DWORD,

    mfighter_magic: DWORD,
    ffighter_magic: DWORD,
    mmagic_magic: DWORD,
    fmagic_magic: DWORD,
    melf_magic: DWORD,
    felf_magic: DWORD,
    mdark_elf_magic: DWORD,
    fdark_elf_magic: DWORD,
    mdwarf_magic: DWORD,
    fdwarf_magic: DWORD,
    morc_magic: DWORD,
    forc_magic: DWORD,
    mshaman_magic: DWORD,
    fshaman_magic: DWORD,
    mkamael_magic: DWORD,
    fkamael_magic: DWORD,
    mertheia_magic: DWORD,
    fertheia_magic: DWORD,

    fextra_throw: DWORD,

    cast_volume: FLOAT,
    cast_rad: FLOAT,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default, Copy)]
struct SkillSoundSourceDat {
    id: DWORD,
    spell_1_effect: DWORD,
    spell_2_effect: DWORD,
    spell_3_effect: DWORD,
    shot_1_effect: DWORD,
    shot_2_effect: DWORD,
    shot_3_effect: DWORD,
    exp_1_effect: DWORD,
    exp_2_effect: DWORD,
    exp_3_effect: DWORD,
}

const SOUND_DEFAULT: SkillSoundDat = SkillSoundDat {
    id: 0,
    level: 0,
    spell_1_effect: 0,
    spell_2_effect: 0,
    spell_3_effect: 0,
    spell_1_vol: 0.0,
    spell_1_rad: 0.0,
    spell_1_delay: 0.0,
    spell_2_vol: 0.0,
    spell_2_rad: 0.0,
    spell_2_delay: 0.0,
    spell_3_vol: 0.0,
    spell_3_rad: 0.0,
    spell_3_delay: 0.0,
    shot_1_effect: 0,
    shot_2_effect: 0,
    shot_3_effect: 0,
    shot_1_vol: 0.0,
    shot_1_rad: 0.0,
    shot_1_delay: 0.0,
    shot_2_vol: 0.0,
    shot_2_rad: 0.0,
    shot_2_delay: 0.0,
    shot_3_vol: 0.0,
    shot_3_rad: 0.0,
    shot_3_delay: 0.0,
    exp_1_effect: 0,
    exp_2_effect: 0,
    exp_3_effect: 0,
    exp_1_vol: 0.0,
    exp_1_rad: 0.0,
    exp_1_delay: 0.0,
    exp_2_vol: 0.0,
    exp_2_rad: 0.0,
    exp_2_delay: 0.0,
    exp_3_vol: 0.0,
    exp_3_rad: 0.0,
    exp_3_delay: 0.0,
    mfighter_cast: 0,
    ffighter_cast: 0,
    mmagic_cast: 0,
    fmagic_cast: 0,
    melf_cast: 0,
    felf_cast: 0,
    mdark_elf_cast: 0,
    fdark_elf_cast: 0,
    mdwarf_cast: 0,
    fdwarf_cast: 0,
    morc_cast: 0,
    forc_cast: 0,
    mshaman_cast: 0,
    fshaman_cast: 0,
    mkamael_cast: 0,
    fkamael_cast: 0,
    mertheia_cast: 0,
    fertheia_cast: 0,
    mextra_throw: 0,
    mfighter_magic: 0,
    ffighter_magic: 0,
    mmagic_magic: 0,
    fmagic_magic: 0,
    melf_magic: 0,
    felf_magic: 0,
    mdark_elf_magic: 0,
    fdark_elf_magic: 0,
    mdwarf_magic: 0,
    fdwarf_magic: 0,
    morc_magic: 0,
    forc_magic: 0,
    mshaman_magic: 0,
    fshaman_magic: 0,
    mkamael_magic: 0,
    fkamael_magic: 0,
    mertheia_magic: 0,
    fertheia_magic: 0,
    fextra_throw: 0,
    cast_volume: 0.0,
    cast_rad: 0.0,
};

const SOUND_SOURCE_DEFAULT: SkillSoundSourceDat = SkillSoundSourceDat {
    id: 0,
    spell_1_effect: 0,
    spell_2_effect: 0,
    spell_3_effect: 0,
    shot_1_effect: 0,
    shot_2_effect: 0,
    shot_3_effect: 0,
    exp_1_effect: 0,
    exp_2_effect: 0,
    exp_3_effect: 0,
};
