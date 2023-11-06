// use serde::{Deserialize, Serialize};
// use crate::data::SkillId;
//
// #[derive(Serialize, Deserialize)]
// pub struct NpcSkill {
//     id: SkillId,
//     level: u32,
// }
//
// #[derive(Serialize, Deserialize)]
// pub struct NpcTemplate {
//     npc_id: u32,
//     name: String,
//     title: String,
//     level: u32,
//
//     #[serde(rename="type")]
//     npc_type: String,
//     #[serde(rename="ai_type")]
//     ai_type: String,
//     #[serde(rename="type")]
//     is_aggressive: bool,
//     #[serde(rename="undying")]
//     is_undying: bool,
//     #[serde(rename="corpse_time")]
//     corpse_time_milliseconds: u32,
//     #[serde(rename="aggroRange")]
//     aggro_range: u32,
//     #[serde(rename="punishOnLvlDiff")]
//     punish_on_lvl_diff: bool,
//
//     #[serde(rename="rewardExp")]
//     reward_exp: f32,
//     #[serde(rename="rewardSp")]
//     reward_sp : f32,
//     #[serde(rename="rewardRp")]
//     reward_rp : f32,
//
//     #[serde(rename="enchant")]
//     enchant: u32,
//     #[serde(rename="rhand")]
//     rhand : u32,
//     #[serde(rename="lhand")]
//     lhand : u32,
//     #[serde(rename="chest")]
//     chest : u32,
//
//     #[serde(rename="htm_root")]
//     htm_root : f32,
//     #[serde(rename="castle_id")]
//     castle_id : f32,
//     #[serde(rename="soulshot_count")]
//     soul_shot_count : f32,
//     #[serde(rename="spiritshot_count")]
//     spirit_shot_count : f32,
//
//     #[serde(rename="def_run")]
//     def_run: f32,
//     #[serde(rename="def_walk")]
//     def_walk: f32,
//
//
//     #[serde(flatten)]
//     base_stats: BaseStats,
//
//     attributes: Attributes,
//
//     skills: Vec<NpcSkill>
//
//     // #[serde(flatten)]
//     // additional_params: HashMap<String, Value>,
// }
//
// pub struct Attributes {
//     fire: u32,
//     water: u32,
//     wind: u32,
//     earth: u32,
//     holy: u32,
//     unholy: u32,
// }
//
// pub struct BaseStats {
//     #[serde(rename = "baseINT")]
//     int: u32,
//     #[serde(rename = "baseSTR")]
//     str: u32,
//     #[serde(rename = "baseCON")]
//     con: u32,
//     #[serde(rename = "baseMEN")]
//     mne: u32,
//     #[serde(rename = "baseDEX")]
//     dex: u32,
//     #[serde(rename = "baseWIT")]
//     wit: u32,
//     #[serde(rename = "baseHpMax")]
//     base_hp_max: f32,
//     #[serde(rename = "baseCpMax")]
//     base_cp_max: f32,
//     #[serde(rename = "baseMpMax")]
//     base_mp_max: f32,
//     #[serde(rename = "baseHpReg")]
//     base_hp_reg: f32,
//     #[serde(rename = "baseCpReg")]
//     base_cp_reg: f32,
//     #[serde(rename = "baseMpReg")]
//     base_mp_reg: f32,
//     #[serde(rename = "basePAtk")]
//     base_patk: f32,
//     #[serde(rename = "baseMAtk")]
//     base_matk: f32,
//     #[serde(rename = "basePDef")]
//     base_pdef: f32,
//     #[serde(rename = "baseMDef")]
//     base_mdef: f32,
//     #[serde(rename = "basePatkSpd")]
//     base_patk_spd: f32,
//     #[serde(rename = "baseMAtkSpd")]
//     base_matk_spd: f32,
//     #[serde(rename = "baseShldDef")]
//     base_shld_def: f32,
//     #[serde(rename = "baseAtkRange")]
//     base_atk_range: f32,
//     #[serde(rename = "baseShldRate")]
//     base_shld_rate: f32,
//     #[serde(rename = "baseCritRate")]
//     base_crit_rate: f32,
//     #[serde(rename = "baseMCritRate")]
//     base_mcrit_rate: f32,
//     #[serde(rename = "baseRunSpd")]
//     base_run_spd: f32,
//     #[serde(rename = "baseWalkSpd")]
//     base_walk_spd: f32,
//     #[serde(rename = "baseWaterRunSpd")]
//     base_water_run_spd: f32,
//     #[serde(rename = "baseWaterWalkSpd")]
//     base_water_walk_spd: f32,
//     #[serde(rename = "baseAttributeAttack")]
//     base_attribute_attack: f32,
//     #[serde(rename = "baseAttributeDefence")]
//     base_attribute_defence: f32,
//     #[serde(rename = "baseHitModify")]
//     base_hit_modify: f32,
//     #[serde(rename = "baseAvoidModify")]
//     base_avoid_modify: f32,
//     #[serde(rename = "collision_radius")]
//     collision_radius: f32,
//     #[serde(rename = "collision_height")]
//     collision_height: f32,
//     #[serde(rename = "baseAttackType")]
//     base_attack_type: f32,
//     #[serde(rename = "base_hp_rate")]
//     base_hp_rate: f32,
// }
