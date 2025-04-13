#![allow(dead_code)]
use crate::backend::editor::WindowParams;
use crate::backend::entity_impl::npc::{NpcMeshAction, NpcSkillAnimationAction, NpcSoundAction};
use crate::backend::util::{Localized, StringCow};
use crate::common::{ItemId, NpcId, QuestId, SkillId};
use crate::entity::{CommonEntity, GetEditParams};
use eframe::egui::Color32;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

impl GetEditParams<()> for Npc {
    fn edit_params(&self) {}
}

impl CommonEntity<NpcId> for Npc {
    fn name(&self) -> String {
        self.name.ru.clone()
    }

    fn desc(&self) -> String {
        "".to_string()
    }

    fn id(&self) -> NpcId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: NpcId) -> Self {
        Self {
            id,
            name: ("Новый НПС".to_string(), "New NPC".to_string()).into(),
            title: ("".to_string(), "".to_string()).into(),
            title_color: Default::default(),
            npc_type: 0,
            unreal_script_class: "".into(),
            mesh_params: Default::default(),
            sound_params: Default::default(),
            summon_params: Default::default(),
            equipment_params: Default::default(),
            skill_animations: Default::default(),
            properties: vec![],
            social: false,
            show_hp: false,
            org_hp: 0.0,
            org_mp: 0.0,
            icon: "".into(),
            additional_parts: Default::default(),
            quest_infos: Default::default(),

            _changed: false,
            _deleted: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcAdditionalParts {
    pub(crate) class: StringCow,
    pub(crate) chest: ItemId,
    pub(crate) legs: ItemId,
    pub(crate) gloves: ItemId,
    pub(crate) feet: ItemId,
    pub(crate) back: ItemId,
    pub(crate) hair_accessory: ItemId,
    pub(crate) hair_style: u32,
    pub(crate) right_hand: ItemId,
    pub(crate) left_hand: ItemId,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcProperty {
    pub(crate) id: SkillId,
    pub(crate) level: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Npc {
    pub(crate) id: NpcId,
    pub(crate) name: Localized<String>,
    pub(crate) title: Localized<String>,
    pub(crate) title_color: Color32,
    pub(crate) npc_type: u16,
    pub(crate) unreal_script_class: StringCow,
    pub(crate) social: bool,
    pub(crate) show_hp: bool,
    pub(crate) org_hp: f64,
    pub(crate) org_mp: f64,
    pub(crate) icon: StringCow,
    pub(crate) properties: Vec<NpcProperty>,
    pub(crate) quest_infos: Vec<NpcQuestInfo>,

    pub(crate) mesh_params: WindowParams<NpcMeshParams, (), NpcMeshAction, ()>,
    pub(crate) sound_params: WindowParams<NpcSoundParams, (), NpcSoundAction, ()>,
    pub(crate) summon_params: WindowParams<NpcSummonParams, (), (), ()>,
    pub(crate) equipment_params: WindowParams<NpcEquipParams, (), (), ()>,
    pub(crate) additional_parts: WindowParams<Option<NpcAdditionalParts>, (), (), ()>,
    pub(crate) skill_animations:
        WindowParams<Vec<NpcSkillAnimation>, (), NpcSkillAnimationAction, ()>,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcSkillAnimation {
    pub(crate) id: SkillId,
    pub(crate) animation: StringCow,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcQuestInfo {
    pub(crate) id: QuestId,
    pub(crate) step: u8,
}

#[derive(
    Serialize,
    Deserialize,
    Display,
    Debug,
    Default,
    EnumIter,
    Eq,
    PartialEq,
    Copy,
    Clone,
    FromPrimitive,
    ToPrimitive,
)]
pub enum SummonType {
    #[default]
    Attack,
    Defence,
    Support,
    Siege,
    Etc,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcSummonParams {
    pub(crate) summon_type: SummonType,
    pub(crate) max_count: u8,
    pub(crate) grade: u8,
    pub(crate) silhouette: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcDecorationEffect {
    pub(crate) effect: StringCow,
    pub(crate) scale: f32,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcMeshParams {
    pub(crate) mesh: StringCow,
    pub(crate) textures: Vec<StringCow>,
    pub(crate) additional_textures: Vec<StringCow>,
    pub(crate) decorations: Vec<NpcDecorationEffect>,
    pub(crate) attack_effect: StringCow,
    pub(crate) speed: f32,
    pub(crate) draw_scale: f32,
    pub(crate) use_zoomincam: f32,
    pub(crate) run_speed: u16,
    pub(crate) walk_speed: u16,
    pub(crate) collision_radius_1: f32,
    pub(crate) collision_radius_2: f32,
    pub(crate) collision_height_1: f32,
    pub(crate) collision_height_2: f32,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcEquipParams {
    pub(crate) left_hand: ItemId,
    pub(crate) right_hand: ItemId,
    pub(crate) chest: ItemId,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcSoundParams {
    pub(crate) attack_sound: Vec<StringCow>,
    pub(crate) defence_sound: Vec<StringCow>,
    pub(crate) damage_sound: Vec<StringCow>,
    pub(crate) dialog_sound: Vec<StringCow>,
    pub(crate) vol: u8,
    pub(crate) rad: u8,
    pub(crate) random: u8,
    pub(crate) priority: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct NpcQuest {
    pub(crate) id: QuestId,
    pub(crate) step: u32,
}
