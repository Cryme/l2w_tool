use std::future::Future;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize)]
pub struct ServerSkillData {
    #[serde(rename = "abnormal_effect")]
    abnormal_effect: String,
    #[serde(rename = "absorbPart")]
    absorb_part: f64,
    #[serde(rename = "activateRate")]
    activate_rate: i32,
    #[serde(rename = "baseValues")]
    base_values: String,
    #[serde(rename = "cancelTarget")]
    cancel_target: i32,
    #[serde(rename = "castCount")]
    cast_count: i32,
    #[serde(rename = "castRange")]
    cast_range: i32,
    #[serde(rename = "chainIndex")]
    chain_index: i32,
    #[serde(rename = "chainSkillId")]
    chain_skill_id: i32,
    #[serde(rename = "coolTime")]
    cool_time: i32,
    #[serde(rename = "corpse")]
    corpse: bool,
    #[serde(rename = "criticalRate")]
    critical_rate: i32,
    #[serde(rename = "delayedEffect")]
    delayed_effect: i32,
    #[serde(rename = "displayId")]
    display_id: i32,
    #[serde(rename = "displayLevel")]
    display_level: i32,
    #[serde(rename = "effectPoint")]
    effect_point: i32,
    #[serde(rename = "element")]
    element: String,
    #[serde(rename = "elementPower")]
    element_power: i32,
    #[serde(rename = "energyConsume")]
    energy_consume: i32,
    #[serde(rename = "flyRadius")]
    fly_radius: i32,
    #[serde(rename = "flyToBack")]
    fly_to_back: bool,
    #[serde(rename = "flyType")]
    fly_type: String,
    #[serde(rename = "hitCancelTime")]
    hit_cancel_time: i32,
    #[serde(rename = "hitTime")]
    hit_time: i32,
    #[serde(rename = "hpConsume")]
    hp_consume: i32,
    #[serde(rename = "icon")]
    icon: String,
    #[serde(rename = "increaseLevel")]
    increase_level: i32,
    #[serde(rename = "itemConsumeCount")]
    item_consume_count: String,
    #[serde(rename = "itemConsumeId")]
    item_consume_id: String,
    #[serde(rename = "lethal1")]
    lethal1: f64,
    #[serde(rename = "lethal2")]
    lethal2: f64,
    #[serde(rename = "level")]
    level: i32,
    #[serde(rename = "levelModifier")]
    level_modifier: i32,
    #[serde(rename = "mAtk")]
    m_atk: i32,
    #[serde(rename = "magicLevel")]
    magic_level: i32,
    #[serde(rename = "magicType")]
    magic_type: SkillMagicType,
    #[serde(rename = "maxEnchantLvls")]
    max_enchant_lvls: i32,
    #[serde(rename = "minPledgeClass")]
    min_pledge_class: i32,
    #[serde(rename = "minRank")]
    min_rank: i32,
    #[serde(rename = "mpConsume1")]
    mp_consume1: i32,
    #[serde(rename = "mpConsume2")]
    mp_consume2: i32,
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "negatePower")]
    negate_power: i32,
    #[serde(rename = "negateSkill")]
    negate_skill: i32,
    #[serde(rename = "nextAction")]
    next_action: String,
    #[serde(rename = "num_charges")]
    num_charges: i32,
    #[serde(rename = "operateType")]
    operate_type: SkillOpType,
    #[serde(rename = "power")]
    power: f64,
    #[serde(rename = "powerPvE")]
    power_pve: f64,
    #[serde(rename = "powerPvP")]
    power_pvp: f64,
    #[serde(rename = "protectBuffId")]
    protect_buff_id: i32,
    #[serde(rename = "referenceItemId")]
    reference_item_id: i32,
    #[serde(rename = "referenceItemMpConsume")]
    reference_item_mp_consume: i32,
    #[serde(rename = "removeEffectOnLoginIfNotInThisZones")]
    remove_effect_on_login_if_not_in_this_zones: String,
    #[serde(rename = "reuseDelay")]
    reuse_delay: u128,
    #[serde(rename = "reuseGroupId")]
    reuse_group_id: i32,
    #[serde(rename = "saveVs")]
    save_vs: BaseStats,
    #[serde(rename = "skillRadius")]
    skill_radius: i32,
    #[serde(rename = "skillType")]
    skill_type: SkillType,
    #[serde(rename = "skill_id")]
    skill_id: i32,
    #[serde(rename = "soulsConsume")]
    souls_consume: i32,
    #[serde(rename = "symbolId")]
    symbol_id: i32,
    #[serde(rename = "target")]
    target: SkillTargetType,
    #[serde(rename = "total_levels")]
    total_levels: i32,
    #[serde(rename = "vitConsume")]
    vit_consume: i32,

    #[serde(rename = "abnormal_instant")]
    abnormal_instant: bool,
    #[serde(rename = "altUse")]
    alt_use: bool,
    #[serde(rename = "basedOnTargetDebuff")]
    based_on_target_debuff: bool,
    #[serde(rename = "behind")]
    behind: bool,
    #[serde(rename = "cancelable")]
    cancelable: bool,
    #[serde(rename = "canCrit")]
    can_crit: bool,
    #[serde(rename = "canUseTeleport")]
    can_use_teleport: bool,
    #[serde(rename = "chargeBoost")]
    charge_boost: bool,
    #[serde(rename = "customBuffSlot")]
    custom_buff_slot: bool,
    #[serde(rename = "deathlink")]
    deathlink: bool,
    #[serde(rename = "disableOnOlympiad")]
    disable_on_olympiad: bool,
    #[serde(rename = "flyingTransformUsage")]
    flying_transform_usage: bool,
    #[serde(rename = "isBowRangeSkill")]
    is_bow_range_skill: bool,
    #[serde(rename = "isForceUse")]
    is_force_use: bool,
    #[serde(rename = "isHandler")]
    is_handler: bool,
    #[serde(rename = "isHeroic")]
    is_heroic: bool,
    #[serde(rename = "isHerb")]
    is_herb: bool,
    #[serde(rename = "isHideStartMessage")]
    is_hide_start_message: bool,
    #[serde(rename = "isHideUseMessage")]
    is_hide_use_message: bool,
    #[serde(rename = "isIgnoreInvul")]
    is_ignore_invul: bool,
    #[serde(rename = "isIgnoreResists")]
    is_ignore_resists: bool,
    #[serde(rename = "isNewbie")]
    is_newbie: bool,
    #[serde(rename = "isNotAffectedByMute")]
    is_not_affected_by_mute: bool,
    #[serde(rename = "isOffensive")]
    is_offensive: bool,
    #[serde(rename = "isPreservedOnDeath")]
    is_preserved_on_death: bool,
    #[serde(rename = "isPvm")]
    is_pvm: bool,
    #[serde(rename = "isPvpSkill")]
    is_pvp_skill: bool,
    #[serde(rename = "isReuseDelayPermanent")]
    is_reuse_delay_permanent: bool,
    #[serde(rename = "isSaveable")]
    is_saveable: bool,
    #[serde(rename = "isSelfDispellable")]
    is_self_dispellable: bool,
    #[serde(rename = "isSkillTimePermanent")]
    is_skill_time_permanent: bool,
    #[serde(rename = "isTrigger")]
    is_trigger: bool,
    #[serde(rename = "isUsingWhileCasting")]
    is_using_while_casting: bool,
    #[serde(rename = "noStartReuse")]
    no_start_reuse: bool,
    #[serde(rename = "overHit")]
    overhit: bool,
    #[serde(rename = "provoke")]
    provoke: bool,
    #[serde(rename = "reflectable")]
    reflectable: bool,
    #[serde(rename = "shieldignore")]
    shieldignore: bool,
    #[serde(rename = "soulBoost")]
    soul_boost: bool,
    #[serde(rename = "stopActor")]
    stop_actor: bool,
    #[serde(rename = "undeadOnly")]
    undead_only: bool,
    #[serde(rename = "useInTvtLikeEvents")]
    use_in_tvt_like_events: bool,
    #[serde(rename = "useSS")]
    use_ss: bool,
}


#[derive(Serialize, Deserialize, EnumIter)]
#[allow(non_camel_case_types)]
pub enum SkillTargetType {
    TARGET_ALLY,
    TARGET_AREA,
    TARGET_AREA_AIM_CORPSE,
    TARGET_AREA_CLAN,
    TARGET_AURA,
    TARGET_PET_AURA,
    TARGET_CHEST,
    TARGET_CLAN,
    TARGET_CLAN_ONLY,
    TARGET_CORPSE,
    TARGET_CORPSE_PLAYER,
    TARGET_ENEMY_PET,
    TARGET_ENEMY_SUMMON,
    TARGET_ENEMY_SERVITOR,
    TARGET_EVENT,
    TARGET_FLAGPOLE,
    TARGET_COMMCHANNEL,
    TARGET_HOLY,
    TARGET_ITEM,
    TARGET_MULTIFACE,
    TARGET_MULTIFACE_AURA,
    TARGET_TUNNEL,
    TARGET_NONE,
    TARGET_ONE,
    TARGET_OWNER,
    TARGET_PARTY,
    TARGET_PARTY_ONE,
    TARGET_PET,
    TARGET_SELF,
    TARGET_SIEGE,
    TARGET_UNLOCKABLE,
    TARGET_AREA_OWNER_AREA,
}

#[derive(Serialize, Deserialize, EnumIter)]
#[allow(non_camel_case_types)]
pub enum BaseStats {
    STR,
    INT,
    DEX,
    WIT,
    CON,
    MEN,
    NONE,
}

#[derive(Serialize, Deserialize, EnumIter)]
#[allow(non_camel_case_types)]
pub enum SkillOpType {
    OP_ACTIVE,
    OP_PASSIVE,
    OP_TOGGLE,
}

#[derive(Serialize, Deserialize, EnumIter)]
#[allow(non_camel_case_types)]
pub enum SkillTrait {
    NONE,
    BLEED,
    BOSS,
    DEATH,
    DERANGEMENT,
    ETC,
    GUST,
    HOLD,
    PARALYZE,
    PHYSICAL_BLOCKADE,
    POISON,
    SHOCK,
    SLEEP,
    VALAKAS,
}

#[derive(Serialize, Deserialize, EnumIter)]
#[allow(non_camel_case_types)]
pub enum SkillMagicType {
    PHYSIC,
    MAGIC,
    SPECIAL,
    MUSIC,
}

#[derive(Serialize, Deserialize, EnumIter)]
#[allow(non_camel_case_types)]
pub enum SkillType {
    AGGRESSION,
    AIEFFECTS,
    BALANCE,
    BLEED,
    BUFF,
    BUFF_CHARGER,
    CALL,
    CHAIN_HEAL,
    CHARGE_SOUL,
    CLAN_GATE,
    COMBATPOINTHEAL,
    CONT,
    CPDAM,
    CPHOT,
    CRAFT,
    DEATH_PENALTY,
    DECOY,
    DEBUFF,
    DELETE_HATE,
    DELETE_HATE_OF_ME,
    DESTROY_SUMMON,
    DEFUSE_TRAP,
    DETECT_TRAP,
    DISCORD,
    DOT,
    DRAIN,
    DRAIN_SOUL,
    EFFECT,
    EFFECTS_FROM_SKILLS,
    ENCHANT_ARMOR,
    ENCHANT_WEAPON,
    EXTRACT_STONE,
    EXTRACT,
    FEED_PET,
    HARDCODED,
    HARVESTING,
    HEAL,
    HEAL_PERCENT,
    HOT,
    INSTANT_JUMP,
    KAMAEL_WEAPON_EXCHANGE,
    LEARN_SKILL,
    LUCK,
    MANADAM,
    MANAHEAL,
    MANAHEAL_PERCENT,
    MDAM,
    MDOT,
    MPHOT,
    MUTE,
    NEGATE_EFFECTS,
    NEGATE_STATS,
    ADD_PC_BANG,
    NOTDONE,
    NOTUSED,
    PARALYZE,
    PASSIVE,
    PDAM,
    TRUEDMG,
    PET_SUMMON,
    POISON,
    RECALL,
    ESCAPE,
    REFILL,
    RESURRECT,
    RESTORE_ITEM,
    RIDE,
    ROOT,
    SELF_SACRIFICE,
    SHIFT_AGGRESSION,
    SLEEP,
    SOULSHOT,
    SOWING,
    SPHEAL,
    SPIRITSHOT,
    SPOIL,
    STEAL_BUFF,
    STUN,
    SUMMON,
    SUMMON_FLAG,
    SUMMON_ITEM,
    SWEEP,
    TAKECASTLE,
    TAKEFORTRESS,
    TAMECONTROL,
    TAKEFLAG,
    TELEPORT_NPC,
    TELEPORT,
    TRANSFORMATION,
    UNLOCK,
    WATCHER_GAZE,
    IMPRISON,
    VITALITY_HEAL,
    CLANCONTROL,
    SPAWN,
}
