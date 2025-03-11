use crate::backend::dat_loader::grand_crusade_166::{L2GeneralStringTable, L2SkillStringTable};
use crate::backend::editor::WindowParams;
use crate::backend::entity_impl::skill::{SkillEnchantAction, SkillEnchantEditWindowParams};
use crate::common::{ItemId, SkillId};
use crate::entity::skill::{
    EnchantInfo, EnchantLevelInfo, EquipStatus, PriorSkill, RacesSkillSoundInfo, Skill,
    SkillLevelInfo, SkillSoundInfo, SkillType, SkillUseCondition, SoundInfo, StatComparisonType,
    StatConditionType,
};

use l2_rw::ue2_rw::{ASCF, BYTE, DWORD, FLOAT, INT, SHORT, USHORT, UVEC};
use l2_rw::{deserialize_dat, deserialize_dat_with_string_dict, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::dat_loader::L2StringTable;
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::backend::log_holder::{Log, LogLevel};
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::RwLock;
use std::thread;
use std::thread::JoinHandle;

impl MSConditionDataDat {
    fn fill_from_enchant_level(&self, enchant_level: &EnchantLevelInfo, enchant_type: u32) -> Self {
        let mut c = self.clone();
        let mp_cost = enchant_level.mp_cost / 3;

        c.sub_level = (enchant_type * 1000 + enchant_level.level) as USHORT;
        c.hp_consume = enchant_level.hp_cost;
        c.mp_consume1 = mp_cost;
        c.mp_consume2 = mp_cost * 2;

        c
    }
    fn fill_from_level(&self, level: &SkillLevelInfo) -> Self {
        let mut c = self.clone();
        let mp_cost = level.mp_cost / 3;

        c.level = level.level as BYTE;
        c.hp_consume = level.hp_cost;
        c.mp_consume1 = mp_cost;
        c.mp_consume2 = mp_cost * 2;

        c
    }
    fn from_skill(skill: &Skill) -> Option<Self> {
        if let Some(cond) = &skill.use_condition {
            let cond = &cond.inner;

            Some(Self {
                id: skill.id.0,
                level: 0,
                sub_level: 0,
                mask: cond.mask,
                equip_type: cond.equipment_condition.to_u8().unwrap(),
                attack_item_type: UVEC {
                    _i: PhantomData,
                    inner: cond.weapon_types.clone(),
                },
                stat_type: cond.stat_condition_type.to_u8().unwrap(),
                stat_percentage: cond.stat_percentage,
                up: cond.comparison_type.to_u8().unwrap(),
                hp_consume: 0,
                mp_consume1: 0,
                mp_consume2: 0,
                item_id: cond.consumable_item_id.0,
                item_count: cond.item_count as SHORT,
                caster_prior_skill_list: cond
                    .caster_prior_skill
                    .iter()
                    .map(|v| PriorSkillDat {
                        id: v.id.0 as USHORT,
                        level: v.level,
                        sub_level: v.sub_level,
                    })
                    .collect(),
                target_prior_skill_list: cond
                    .target_prior_skill
                    .iter()
                    .map(|v| PriorSkillDat {
                        id: v.id.0 as USHORT,
                        level: v.level,
                        sub_level: v.sub_level,
                    })
                    .collect(),
            })
        } else {
            None
        }
    }
}

impl GameDataHolder {
    pub fn serialize_skills_to_binary(&mut self) -> JoinHandle<Vec<Log>> {
        let mut logs = vec![];

        let mut skill_grp = vec![];
        let mut skill_string_table = L2SkillStringTable::from_vec(vec![]);
        let mut skill_name = vec![];
        let mut skill_sound = vec![];
        let mut skill_sound_src = vec![];
        let mut ms_condition = vec![];

        let mut vals: Vec<_> = self.skill_holder.values().filter(|v| !v._deleted).collect();
        vals.sort_by(|a, b| a.id.cmp(&b.id));

        for skill in vals {
            if skill.skill_levels.is_empty() {
                continue;
            }

            let cond = MSConditionDataDat::from_skill(skill);

            skill_sound.push(skill.sound_data(&mut self.game_string_table));
            skill_sound_src.push(skill.sound_source_data());

            let mut base_skill_grp = SkillGrpDat::default();
            let mut base_skill_name = SkillNameDat::default();

            base_skill_grp.fill_from_skill(skill, &mut self.game_string_table);
            base_skill_name.fill_from_skill(skill, &mut skill_string_table);

            let mut first = true;
            for level in &skill.skill_levels {
                let mut base_skill_grp = base_skill_grp.clone();
                let mut base_skill_name = base_skill_name;

                let cond = if let Some(c) = &cond {
                    let cc = c.fill_from_level(level);
                    ms_condition.push(cc.clone());

                    Some(cc)
                } else {
                    None
                };

                base_skill_grp.fill_from_level(level, &mut self.game_string_table, first);
                base_skill_name.fill_from_level(level, &mut skill_string_table, first);

                skill_grp.push(base_skill_grp.clone());
                skill_name.push(base_skill_name);

                first = false;

                for enchant in &level.available_enchants {
                    let enchant = &enchant.inner;
                    let mut base_skill_grp = base_skill_grp.clone();
                    let mut base_skill_name = base_skill_name;

                    base_skill_grp.fill_from_enchant(
                        enchant,
                        &mut self.game_string_table,
                        level.level,
                    );
                    base_skill_name.fill_from_enchant(
                        enchant,
                        &mut skill_string_table,
                        level.level,
                    );

                    for enchant_level in &enchant.enchant_levels {
                        if let Some(c) = &cond {
                            ms_condition.push(
                                c.fill_from_enchant_level(enchant_level, enchant.enchant_type),
                            );
                        };

                        base_skill_grp.fill_from_enchant_level(
                            enchant_level,
                            &mut self.game_string_table,
                            enchant.enchant_type,
                        );
                        base_skill_name.fill_from_enchant_level(
                            enchant_level,
                            &mut skill_string_table,
                            enchant.enchant_type,
                        );

                        skill_grp.push(base_skill_grp.clone());
                        skill_name.push(base_skill_name);
                    }
                }
            }
        }

        let skill_sound_src_path = self
            .dat_paths
            .get(&"skillsoundsource.dat".to_string())
            .unwrap()
            .clone();
        let ms_condition_path = self
            .dat_paths
            .get(&"msconditiondata.dat".to_string())
            .unwrap()
            .clone();
        let skill_name_path = self
            .dat_paths
            .get(&"skillname-ru.dat".to_string())
            .unwrap()
            .clone();
        let skill_grp_path = self
            .dat_paths
            .get(&"skillgrp.dat".to_string())
            .unwrap()
            .clone();
        let skill_sound_path = self
            .dat_paths
            .get(&"skillsoundgrp.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            let ms_condition_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    ms_condition_path.path(),
                    DatVariant::<(), MSConditionDataDat>::Array(ms_condition),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("Ms Condition saved")
                }
            });
            let skill_name_handel = thread::spawn(move || {
                if let Err(e) = save_dat(
                    skill_name_path.path(),
                    DatVariant::DoubleArray(
                        SkillNameTableRecord::from_table(skill_string_table),
                        skill_name,
                    ),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("Skill Name saved")
                }
            });
            let skill_grp_handel = thread::spawn(move || {
                if let Err(e) = save_dat(
                    skill_grp_path.path(),
                    DatVariant::<(), SkillGrpDat>::Array(skill_grp),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("Skill Grp saved")
                }
            });
            let skill_sound_handel = thread::spawn(move || {
                if let Err(e) = save_dat(
                    skill_sound_path.path(),
                    DatVariant::<(), SkillSoundDat>::Array(skill_sound),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("Skill Sound saved")
                }
            });
            let skill_sound_src_handel = thread::spawn(move || {
                if let Err(e) = save_dat(
                    skill_sound_src_path.path(),
                    DatVariant::<(), SkillSoundSourceDat>::Array(skill_sound_src),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("Skill Sound Src saved")
                }
            });

            logs.push(skill_name_handel.join().unwrap());
            logs.push(skill_grp_handel.join().unwrap());
            logs.push(skill_sound_handel.join().unwrap());
            logs.push(skill_sound_src_handel.join().unwrap());
            logs.push(ms_condition_handle.join().unwrap());

            logs
        })
    }
    pub fn load_skills(&mut self) -> Result<Vec<Log>, ()> {
        let mut warnings = vec![];

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
            string_dict.insert(id, val.to_string());
        }

        string_dict.insert(u32::MAX, "NOT EXIST".to_string());

        let mut current_id = skill_grp.first().unwrap().id;
        let mut current_grps = vec![];

        let mut treed_names: HashMap<u16, HashMap<u8, HashMap<u16, SkillNameDat>>> =
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

        let skill_condition_dat = deserialize_dat::<MSConditionDataDat>(
            self.dat_paths
                .get(&"msconditiondata.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let mut treed_conditions: HashMap<u16, HashMap<u8, HashMap<u16, MSConditionDataDat>>> =
            HashMap::new();

        for condition in skill_condition_dat {
            if let Some(level) = treed_conditions.get_mut(&(condition.id as u16)) {
                if let Some(sub_level) = level.get_mut(&condition.level) {
                    sub_level.insert(condition.sub_level, condition);
                } else {
                    let l = condition.level;
                    let mut sub_level = HashMap::new();
                    sub_level.insert(condition.sub_level, condition);
                    level.insert(l, sub_level);
                }
            } else {
                let mut level = HashMap::new();
                let mut sub_level = HashMap::new();
                let l = condition.level;
                let cid = condition.id as u16;
                sub_level.insert(condition.sub_level, condition);
                level.insert(l, sub_level);
                treed_conditions.insert(cid, level);
            }
        }

        for record in skill_grp {
            if record.id != current_id {
                if let Some(l) = self.build_skill(
                    &current_grps,
                    &treed_names,
                    &string_dict,
                    &sound_map,
                    &sound_source_map,
                    &treed_conditions,
                ) {
                    warnings.push(l);
                }

                current_id = record.id;
                current_grps.clear();
            }

            current_grps.push(record);
        }

        if !current_grps.is_empty() {
            if let Some(l) = self.build_skill(
                &current_grps,
                &treed_names,
                &string_dict,
                &sound_map,
                &sound_source_map,
                &treed_conditions,
            ) {
                warnings.push(l);
            }
        }

        Ok(warnings)
    }

    fn get_name_record_or_default<'a>(
        &self,
        id: u16,
        level: u8,
        sub_level: u16,
        skill_names: &'a HashMap<u16, HashMap<u8, HashMap<u16, SkillNameDat>>>,
    ) -> &'a SkillNameDat {
        const DEFAULT_SKILL_NAME_DAT: SkillNameDat = SkillNameDat {
            id: 0,
            level: 0,
            sub_level: 0,
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
        skill_names: &HashMap<u16, HashMap<u8, HashMap<u16, SkillNameDat>>>,
        string_dict: &HashMap<u32, String>,
        sound_map: &HashMap<u32, SkillSoundDat>,
        sound_source_map: &HashMap<u32, SkillSoundSourceDat>,
        treed_condition: &HashMap<u16, HashMap<u8, HashMap<u16, MSConditionDataDat>>>,
    ) -> Option<Log> {
        let first_grp = skill_grps.first().unwrap();
        let first_name = self.get_name_record_or_default(
            first_grp.id,
            first_grp.level,
            first_grp.sub_level,
            skill_names,
        );

        let first_condition = if let Some(v) = treed_condition.get(&first_grp.id) {
            let Some(first_condition) = v.get(&first_grp.level) else {
                return Some(Log {
                    level: LogLevel::Error,
                    producer: "Skill Loader".to_string(),
                    log: format!(
                        "Skill[{}]: No record in mscondition for level {}. Skipped",
                        first_grp.id, first_grp.level
                    ),
                });
            };

            let Some(first_condition) = first_condition.get(&(first_grp.sub_level)) else {
                return Some(Log {
                    level: LogLevel::Error,
                    producer: "Skill Loader".to_string(),
                    log: format!(
                        "Skill[{}]: No record in mscondition for sub-level {} of level {}. Skipped",
                        first_grp.id, first_grp.sub_level, first_grp.level,
                    ),
                });
            };

            Some(WindowParams::new(SkillUseCondition {
                mask: first_condition.mask,
                equipment_condition: EquipStatus::from_u8(first_condition.equip_type).unwrap(),
                weapon_types: first_condition.attack_item_type.inner.clone(),
                stat_condition_type: StatConditionType::from_u8(first_condition.stat_type).unwrap(),
                stat_percentage: first_condition.stat_percentage,
                comparison_type: StatComparisonType::from_u8(first_condition.up).unwrap(),
                consumable_item_id: ItemId(first_condition.item_id),
                item_count: first_condition.item_count as u16,
                caster_prior_skill: first_condition
                    .caster_prior_skill_list
                    .iter()
                    .map(|v| PriorSkill {
                        id: SkillId(v.id as u32),
                        level: v.level,
                        sub_level: v.sub_level,
                    })
                    .collect(),
                target_prior_skill: first_condition
                    .target_prior_skill_list
                    .iter()
                    .map(|v| PriorSkill {
                        id: SkillId(v.id as u32),
                        level: v.level,
                        sub_level: v.sub_level,
                    })
                    .collect(),
            }))
        } else {
            None
        };

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
            skill_type: SkillType::from_u8(first_grp.operate_type).unwrap(),
            resist_cast: first_grp.resist_cast,
            magic_type: first_grp.magic_type,
            cast_style: first_grp.cast_style,
            skill_magic_type: first_grp.skill_magic_type,
            origin_skill: SkillId(first_grp.origin_skill as u32),
            is_double: first_grp.is_double == 1,
            animations: first_grp
                .animation
                .inner
                .iter()
                .map(|v| self.game_string_table.get(v).unwrap())
                .cloned()
                .collect(),
            visual_effect: self
                .game_string_table
                .get_o(&first_grp.skill_visual_effect)
                .clone(),
            icon: self.game_string_table.get_o(&first_grp.icon).clone(),
            icon_panel: self.game_string_table[first_grp.icon_panel as usize].clone(),
            cast_bar_text_is_red: first_grp.icon_type == 1,
            rumble_self: first_grp.rumble_self,
            rumble_target: first_grp.rumble_target,
            skill_levels: vec![],
            is_debuff: first_grp.debuff == 1,
            sound_info: WindowParams::new(SkillSoundInfo {
                spell_effect_1: SoundInfo {
                    sound: self.game_string_table[sound.spell_1_effect as usize].clone(),
                    vol: sound.spell_1_vol,
                    rad: sound.spell_1_rad,
                    delay: sound.spell_1_delay,
                    source: sound_source.spell_1_effect,
                },
                spell_effect_2: SoundInfo {
                    sound: self.game_string_table[sound.spell_2_effect as usize].clone(),
                    vol: sound.spell_2_vol,
                    rad: sound.spell_2_rad,
                    delay: sound.spell_2_delay,
                    source: sound_source.spell_2_effect,
                },
                spell_effect_3: SoundInfo {
                    sound: self.game_string_table[sound.spell_3_effect as usize].clone(),
                    vol: sound.spell_3_vol,
                    rad: sound.spell_3_rad,
                    delay: sound.spell_3_delay,
                    source: sound_source.spell_3_effect,
                },
                shot_effect_1: SoundInfo {
                    sound: self.game_string_table[sound.shot_1_effect as usize].clone(),
                    vol: sound.shot_1_vol,
                    rad: sound.shot_1_rad,
                    delay: sound.shot_1_delay,
                    source: sound_source.shot_1_effect,
                },
                shot_effect_2: SoundInfo {
                    sound: self.game_string_table[sound.shot_2_effect as usize].clone(),
                    vol: sound.shot_2_vol,
                    rad: sound.shot_2_rad,
                    delay: sound.shot_2_delay,
                    source: sound_source.shot_2_effect,
                },
                shot_effect_3: SoundInfo {
                    sound: self.game_string_table[sound.shot_3_effect as usize].clone(),
                    vol: sound.shot_3_vol,
                    rad: sound.shot_3_rad,
                    delay: sound.shot_3_delay,
                    source: sound_source.shot_3_effect,
                },
                exp_effect_1: SoundInfo {
                    sound: self.game_string_table[sound.exp_1_effect as usize].clone(),
                    vol: sound.exp_1_vol,
                    rad: sound.exp_1_rad,
                    delay: sound.exp_1_delay,
                    source: sound_source.exp_1_effect,
                },
                exp_effect_2: SoundInfo {
                    sound: self.game_string_table[sound.exp_2_effect as usize].clone(),
                    vol: sound.exp_2_vol,
                    rad: sound.exp_2_rad,
                    delay: sound.exp_2_delay,
                    source: sound_source.exp_2_effect,
                },
                exp_effect_3: SoundInfo {
                    sound: self.game_string_table[sound.exp_3_effect as usize].clone(),
                    vol: sound.exp_3_vol,
                    rad: sound.exp_3_rad,
                    delay: sound.exp_3_delay,
                    source: sound_source.exp_3_effect,
                },
                sound_before_cast: RacesSkillSoundInfo {
                    mfighter: self.game_string_table[sound.mfighter_cast as usize].clone(),
                    ffighter: self.game_string_table[sound.ffighter_cast as usize].clone(),
                    mmagic: self.game_string_table[sound.mmagic_cast as usize].clone(),
                    fmagic: self.game_string_table[sound.fmagic_cast as usize].clone(),
                    melf: self.game_string_table[sound.melf_cast as usize].clone(),
                    felf: self.game_string_table[sound.felf_cast as usize].clone(),
                    mdark_elf: self.game_string_table[sound.mdark_elf_cast as usize].clone(),
                    fdark_elf: self.game_string_table[sound.fdark_elf_cast as usize].clone(),
                    mdwarf: self.game_string_table[sound.mdwarf_cast as usize].clone(),
                    fdwarf: self.game_string_table[sound.fdwarf_cast as usize].clone(),
                    morc: self.game_string_table[sound.morc_cast as usize].clone(),
                    forc: self.game_string_table[sound.forc_cast as usize].clone(),
                    mshaman: self.game_string_table[sound.mshaman_cast as usize].clone(),
                    fshaman: self.game_string_table[sound.fshaman_cast as usize].clone(),
                    mkamael: self.game_string_table[sound.mkamael_cast as usize].clone(),
                    fkamael: self.game_string_table[sound.fkamael_cast as usize].clone(),
                    mertheia: self.game_string_table[sound.mertheia_cast as usize].clone(),
                    fertheia: self.game_string_table[sound.fertheia_cast as usize].clone(),
                },
                sound_after_cast: RacesSkillSoundInfo {
                    mfighter: self.game_string_table[sound.mfighter_magic as usize].clone(),
                    ffighter: self.game_string_table[sound.ffighter_magic as usize].clone(),
                    mmagic: self.game_string_table[sound.mmagic_magic as usize].clone(),
                    fmagic: self.game_string_table[sound.fmagic_magic as usize].clone(),
                    melf: self.game_string_table[sound.melf_magic as usize].clone(),
                    felf: self.game_string_table[sound.felf_magic as usize].clone(),
                    mdark_elf: self.game_string_table[sound.mdark_elf_magic as usize].clone(),
                    fdark_elf: self.game_string_table[sound.fdark_elf_magic as usize].clone(),
                    mdwarf: self.game_string_table[sound.mdwarf_magic as usize].clone(),
                    fdwarf: self.game_string_table[sound.fdwarf_magic as usize].clone(),
                    morc: self.game_string_table[sound.morc_magic as usize].clone(),
                    forc: self.game_string_table[sound.forc_magic as usize].clone(),
                    mshaman: self.game_string_table[sound.mshaman_magic as usize].clone(),
                    fshaman: self.game_string_table[sound.fshaman_magic as usize].clone(),
                    mkamael: self.game_string_table[sound.mkamael_magic as usize].clone(),
                    fkamael: self.game_string_table[sound.fkamael_magic as usize].clone(),
                    mertheia: self.game_string_table[sound.mertheia_magic as usize].clone(),
                    fertheia: self.game_string_table[sound.fertheia_magic as usize].clone(),
                },
                mextra_throw: self.game_string_table[sound.mextra_throw as usize].clone(),
                fextra_throw: self.game_string_table[sound.fextra_throw as usize].clone(),
                vol: sound.cast_volume,
                rad: sound.cast_rad,
            }),
            use_condition: first_condition,
            ..Default::default()
        };

        let mut levels = vec![];
        let mut enchants: HashMap<u8, HashMap<u16, EnchantInfo>> = HashMap::new();

        for v in skill_grps.iter() {
            let skill_name = self.get_name_record_or_default(
                v.id,
                v.level,
                v.sub_level,
                skill_names,
            );

            if v.sub_level == 0 {
                let desc = if skill_name.desc == first_name.desc {
                    None
                } else {
                    let c = string_dict.get(&skill_name.desc).unwrap();
                    if c.is_empty() || c == "\0" {
                        None
                    } else {
                        Some(c.clone())
                    }
                };

                let level_name = if skill_name.name == first_name.name {
                    None
                } else {
                    Some(string_dict.get(&skill_name.name).unwrap().clone())
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
                    icon: if v.icon == first_grp.icon {
                        None
                    } else {
                        Some(self.game_string_table.get(&v.icon).unwrap().clone())
                    },
                    icon_panel: if v.icon_panel == first_grp.icon_panel {
                        None
                    } else {
                        Some(self.game_string_table.get(&v.icon_panel).unwrap().clone())
                    },
                    name: level_name,
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
                    icon: if v.icon == first_grp.icon {
                        None
                    } else {
                        Some(self.game_string_table.get(&v.icon).unwrap().clone())
                    },
                    icon_panel: if v.icon_panel == first_grp.icon_panel {
                        None
                    } else {
                        Some(self.game_string_table.get(&v.icon_panel).unwrap().clone())
                    },
                };

                if let Some(curr_level_enchants) = enchants.get_mut(&v.level) {
                    if let Some(ei) = curr_level_enchants.get_mut(&variant) {
                        ei.enchant_levels.push(enchant_level);
                    } else {
                        let desc = if skill_name.desc == first_name.desc {
                            None
                        } else {
                            let c = string_dict.get(&skill_name.desc).unwrap();
                            if c.is_empty() || c == "\0" {
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
                                    .game_string_table
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
                        let c = string_dict.get(&skill_name.desc).unwrap();
                        if c.is_empty() || c == "\0" {
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
                                .game_string_table
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
            return None;
        }

        for (key, mut value) in enchants {
            let mut inner_keys: Vec<_> = value.drain().collect();
            inner_keys.sort_by(|(a_i, _), (b_i, _)| a_i.cmp(b_i));

            for (_, v) in inner_keys {
                levels[key as usize - 1]
                    .available_enchants
                    .push(WindowParams {
                        params: SkillEnchantEditWindowParams {
                            current_level_index: v.enchant_levels.len() - 1,
                        },
                        inner: v,
                        opened: false,
                        initial_id: (),
                        action: RwLock::new(SkillEnchantAction::None),
                    });
            }
        }

        skill.skill_levels = levels;

        self.skill_holder.insert(skill.id, skill);

        None
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
            mfighter_cast: string_table
                .get_index(&self.sound_info.inner.sound_before_cast.mfighter),
            ffighter_cast: string_table
                .get_index(&self.sound_info.inner.sound_before_cast.ffighter),
            mmagic_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mmagic),
            fmagic_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fmagic),
            melf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.melf),
            felf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.felf),
            mdark_elf_cast: string_table
                .get_index(&self.sound_info.inner.sound_before_cast.mdark_elf),
            fdark_elf_cast: string_table
                .get_index(&self.sound_info.inner.sound_before_cast.fdark_elf),
            mdwarf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mdwarf),
            fdwarf_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fdwarf),
            morc_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.morc),
            forc_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.forc),
            mshaman_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mshaman),
            fshaman_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fshaman),
            mkamael_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.mkamael),
            fkamael_cast: string_table.get_index(&self.sound_info.inner.sound_before_cast.fkamael),
            mertheia_cast: string_table
                .get_index(&self.sound_info.inner.sound_before_cast.mertheia),
            fertheia_cast: string_table
                .get_index(&self.sound_info.inner.sound_before_cast.fertheia),
            mextra_throw: string_table.get_index(&self.sound_info.inner.mextra_throw),
            mfighter_magic: string_table
                .get_index(&self.sound_info.inner.sound_after_cast.mfighter),
            ffighter_magic: string_table
                .get_index(&self.sound_info.inner.sound_after_cast.ffighter),
            mmagic_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mmagic),
            fmagic_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fmagic),
            melf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.melf),
            felf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.felf),
            mdark_elf_magic: string_table
                .get_index(&self.sound_info.inner.sound_after_cast.mdark_elf),
            fdark_elf_magic: string_table
                .get_index(&self.sound_info.inner.sound_after_cast.fdark_elf),
            mdwarf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mdwarf),
            fdwarf_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fdwarf),
            morc_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.morc),
            forc_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.forc),
            mshaman_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mshaman),
            fshaman_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fshaman),
            mkamael_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.mkamael),
            fkamael_magic: string_table.get_index(&self.sound_info.inner.sound_after_cast.fkamael),
            mertheia_magic: string_table
                .get_index(&self.sound_info.inner.sound_after_cast.mertheia),
            fertheia_magic: string_table
                .get_index(&self.sound_info.inner.sound_after_cast.fertheia),
            fextra_throw: string_table.get_index(&self.sound_info.inner.fextra_throw),
            cast_volume: self.sound_info.inner.vol,
            cast_rad: self.sound_info.inner.rad,
        }
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct SkillGrpDat {
    id: USHORT,
    level: BYTE,
    sub_level: USHORT,
    icon_type: BYTE,
    //Выяснить чо такое
    magic_type: BYTE,
    //Выяснить чо такое
    operate_type: BYTE,
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
    animation: UVEC<DWORD, DWORD>,
    skill_visual_effect: DWORD,
    icon: DWORD,
    icon_panel: DWORD,
    //Проверить бывает ли больше 1
    debuff: BYTE, //enchant override
    resist_cast: BYTE,
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

impl SkillGrpDat {
    #[inline]
    fn fill_from_enchant_level(
        &mut self,
        enchant_level: &EnchantLevelInfo,
        game_data_name: &mut L2GeneralStringTable,
        enchant_type: u32,
    ) {
        self.sub_level = (enchant_type * 1000 + enchant_level.level) as USHORT;
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
    fn fill_from_enchant(
        &mut self,
        enchant: &EnchantInfo,
        game_data_name: &mut L2GeneralStringTable,
        level: u32,
    ) {
        self.debuff = enchant.is_debuff.into();
        self.enchant_icon = game_data_name.get_index(&enchant.enchant_icon);
        self.enchant_skill_level = level as BYTE;
    }
    #[inline]
    fn fill_from_level(
        &mut self,
        level: &SkillLevelInfo,
        game_data_name: &mut L2GeneralStringTable,
        first: bool,
    ) {
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
        self.operate_type = skill.skill_type.to_u8().unwrap();
        self.resist_cast = skill.resist_cast;
        self.magic_type = skill.magic_type;
        self.cast_style = skill.cast_style;
        self.skill_magic_type = skill.skill_magic_type;
        self.origin_skill = skill.origin_skill.0 as SHORT;
        self.is_double = skill.is_double.into();
        self.animation = UVEC {
            _i: PhantomData,
            inner: skill
                .animations
                .iter()
                .map(|v| game_data_name.get_index(&v.to_string()))
                .collect(),
        };
        self.skill_visual_effect = game_data_name.get_index(&skill.visual_effect);
        self.icon = game_data_name.get_index(&skill.icon);
        self.icon_panel = game_data_name.get_index(&skill.icon_panel);
        self.debuff = skill.is_debuff.into();
        self.icon_type = skill.is_debuff.into();
        self.enchant_skill_level = 0;
        self.enchant_icon = game_data_name.get_index("None");
        self.rumble_self = skill.rumble_self;
        self.rumble_target = skill.rumble_target;
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct SkillNameTableRecord {
    val: ASCF,
    id: DWORD,
}

impl SkillNameTableRecord {
    pub(crate) fn from_table(value: L2SkillStringTable) -> Vec<Self> {
        let mut keys: Vec<_> = value.keys().collect();
        keys.sort();

        let mut res = Vec::with_capacity(keys.len());

        for key in keys {
            res.push(Self {
                val: value.get(key).unwrap().into(),
                id: *key,
            })
        }

        res
    }
}

#[derive(Debug, Copy, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct SkillNameDat {
    id: USHORT,
    level: BYTE,
    sub_level: USHORT,
    name: DWORD,
    desc: DWORD,
    desc_params: DWORD,
    enchant_name: DWORD,
    enchant_name_params: DWORD,
    enchant_desc: DWORD,
    enchant_desc_params: DWORD,
}

impl SkillNameDat {
    #[inline]
    fn fill_from_enchant_level(
        &mut self,
        enchant_level: &EnchantLevelInfo,
        skill_string_table: &mut L2SkillStringTable,
        enchant_type: u32,
    ) {
        self.enchant_name_params = skill_string_table.get_index(&enchant_level.enchant_name_params);
        self.enchant_desc_params =
            skill_string_table.get_index(&enchant_level.enchant_description_params);
        self.desc_params = skill_string_table.get_index(&enchant_level.skill_description_params);

        self.sub_level = (enchant_type * 1000 + enchant_level.level) as USHORT;
    }
    #[inline]
    fn fill_from_enchant(
        &mut self,
        enchant: &EnchantInfo,
        skill_string_table: &mut L2SkillStringTable,
        level: u32,
    ) {
        self.enchant_desc = skill_string_table.get_index(&enchant.enchant_description);
        self.enchant_name = skill_string_table.get_index(&enchant.enchant_name);
        self.desc = skill_string_table.get_index(if let Some(v) = &enchant.skill_description {
            v
        } else {
            ""
        });
    }
    #[inline]
    fn fill_from_level(
        &mut self,
        level: &SkillLevelInfo,
        skill_string_table: &mut L2SkillStringTable,
        first: bool,
    ) {
        self.level = level.level as BYTE;

        if !first {
            if let Some(v) = &level.name {
                self.name = skill_string_table.get_index(v);
            }

            self.desc = skill_string_table.get_index(if let Some(v) = &level.description {
                v
            } else {
                ""
            });
        }
        self.desc_params = skill_string_table.get_index(&level.description_params);
    }
    #[inline]
    fn fill_from_skill(&mut self, skill: &Skill, skill_string_table: &mut L2SkillStringTable) {
        self.id = skill.id.0 as USHORT;
        self.sub_level = 0;
        self.name = skill_string_table.get_index(&skill.name);
        self.desc = skill_string_table.get_index(&skill.description);
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default, Copy)]
pub struct SkillSoundDat {
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
pub struct SkillSoundSourceDat {
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

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct MSConditionDataDat {
    id: DWORD,
    level: BYTE,
    sub_level: USHORT,
    mask: SHORT,
    equip_type: BYTE,
    attack_item_type: UVEC<BYTE, BYTE>,
    stat_type: BYTE,
    stat_percentage: BYTE,
    up: BYTE,
    hp_consume: SHORT,
    mp_consume1: SHORT,
    mp_consume2: SHORT,
    item_id: DWORD,
    item_count: SHORT,
    caster_prior_skill_list: Vec<PriorSkillDat>,
    target_prior_skill_list: Vec<PriorSkillDat>,
}

#[derive(Debug, Copy, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct PriorSkillDat {
    id: USHORT,
    level: BYTE,
    sub_level: USHORT,
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
