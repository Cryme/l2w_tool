#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::backend::holder::HolderMapOps;
use crate::backend::script_ext::ChangedEntities;
use crate::backend::Backend;
use crate::common::{SkillId};
use rhai::plugin::*;
use rhai::{Engine, TypeBuilder};
use crate::backend::editor::WindowParams;
use crate::entity::skill::{EnchantInfo, EnchantLevelInfo, PriorSkill, RacesSkillSoundInfo, Skill, SkillLevelInfo, SkillSoundInfo, SkillType, SkillUseCondition, SoundInfo};

impl Skill {
    fn get_sound_info(&mut self) -> SkillSoundInfo {
        self.sound_info.inner.clone()
    }
    fn set_sound_info(&mut self, val: SkillSoundInfo) {
        self.sound_info.inner = val;
    }

    fn get_use_condition(&mut self) -> Option<SkillUseCondition> {
        let Some(v) = &self.use_condition else { return None };

        Some(v.inner.clone())
    }
    fn set_use_condition(&mut self, val: Option<SkillUseCondition>) {
        let Some(val) = val else {
            self.use_condition = None;

            return
        };

        let Some(s) = &mut self.use_condition else {
            self.use_condition = Some(WindowParams::new(val));

            return;
        };

        s.inner = val;
    }

    /// Additional API's
    pub fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder.on_print(|v| format!("Skill(id: {}, name: {})", v.id.0, v.name));
        builder.with_get_set("sound_info", Self::get_sound_info, Self::set_sound_info);
        builder.with_get_set("use_condition", Self::get_use_condition, Self::set_use_condition);
    }
}
impl SkillId {
    /// Additional API's
    pub fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder.on_print(|v| format!("SkillId({})", v.0));
    }
}

pub fn proceed(backend: &mut Backend, entities: Vec<Skill>) {
    for mut v in entities {
        v._changed = true;
        backend.editors.force_update_skill(&v);
        backend.save_skill_force(v);
    }
}

pub fn reg(engine: &mut Engine, changed_entities_ptr: *mut ChangedEntities, ptr: *const Backend) {
    //Eq Overloads
    {
        engine.register_fn("==", |lhs: SkillId, rhs: i64| -> bool {
            lhs.0 as i64 == rhs
        });
        engine.register_fn("==", |lhs: i64, rhs: SkillId| -> bool {
            lhs == rhs.0 as i64
        });
    }

    unsafe {
        engine.register_fn("save", move |x: Skill| {
            (*changed_entities_ptr).skill.push(x);
        });
        engine.register_fn("delete", move |mut x: Skill| {
            x._deleted = true;

            (*changed_entities_ptr).skill.push(x);
        });

        engine.register_fn("skill_list", move || -> Dynamic {
            (*ptr)
                .holders
                .game_data_holder
                .skill_holder
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .into()
        });
    }

    engine.build_type::<Skill>();

    engine.build_type::<SkillUseCondition>();
    engine.build_type::<PriorSkill>();

    engine.build_type::<SkillLevelInfo>();

    engine.build_type::<SkillSoundInfo>();
    engine.build_type::<SoundInfo>();
    engine.build_type::<RacesSkillSoundInfo>();

    engine.build_type::<EnchantInfo>();
    engine.build_type::<EnchantLevelInfo>();

    engine.build_type::<SkillId>();
    engine.register_fn("set_id", |v: &mut Skill, id: i64| {
        v.id.0 = id as u32;
    });

    engine
        .register_type_with_name::<SkillType>("SkillType")
        .register_static_module("SkillType", exported_module!(skill_type_module).into());
}


#[export_module]
mod skill_type_module {
    use strum::IntoEnumIterator;

    pub const Physical: SkillType = SkillType::Physical;
    pub const Magical: SkillType = SkillType::Magical;
    pub const Buff: SkillType = SkillType::Buff;
    pub const Debuff: SkillType = SkillType::Debuff;
    pub const ClanActive: SkillType = SkillType::ClanActive;
    pub const ItemActive: SkillType = SkillType::ItemActive;
    pub const Toggle: SkillType = SkillType::Toggle;
    pub const Transformation: SkillType = SkillType::Transformation;
    pub const AlsoToggle: SkillType = SkillType::AlsoToggle;
    pub const EquipmentPassive: SkillType = SkillType::EquipmentPassive;
    pub const Abilities: SkillType = SkillType::Abilities;
    pub const Race: SkillType = SkillType::Race;
    pub const Additional: SkillType = SkillType::Additional;
    pub const ClanPassive: SkillType = SkillType::ClanPassive;
    pub const ItemPassive: SkillType = SkillType::ItemPassive;

    #[rhai_fn(global, get = "skill_type", pure)]
    pub fn get_type(val: &mut SkillType) -> String {
        val.to_string()
    }

    pub fn all_variants() -> Vec<SkillType> {
        SkillType::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(val: &mut SkillType) -> String {
        format!("{val:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(item_quality: &mut SkillType, other: SkillType) -> bool {
        item_quality == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(item_quality: &mut SkillType, other: SkillType) -> bool {
        item_quality != &other
    }
}