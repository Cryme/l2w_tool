#![allow(dead_code)]
use crate::backend::npc::{NpcMeshAction, NpcSkillAnimationAction, NpcSoundAction};
use crate::backend::WindowParams;
use crate::data::{ItemId, NpcId, QuestId, SkillId};
use crate::entity::CommonEntity;
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

impl CommonEntity<NpcId, ()> for Npc {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn desc(&self) -> String {
        "".to_string()
    }

    fn id(&self) -> NpcId {
        self.id
    }

    fn edit_params(&self) {}

    fn new(id: NpcId) -> Self {
        Self {
            id,
            name: "New NPC".to_string(),
            title: "".to_string(),
            title_color: Default::default(),
            npc_type: 0,
            unreal_script_class: "".to_string(),
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
            icon: "".to_string(),
            additional_parts: Default::default(),
            quest_infos: Default::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcAdditionalParts {
    pub(crate) class: String,
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

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcProperty {
    pub(crate) id: SkillId,
    pub(crate) level: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Npc {
    pub(crate) id: NpcId,
    pub(crate) name: String,
    pub(crate) title: String,
    pub(crate) title_color: Color32,
    pub(crate) npc_type: u16,
    pub(crate) unreal_script_class: String,
    pub(crate) social: bool,
    pub(crate) show_hp: bool,
    pub(crate) org_hp: f64,
    pub(crate) org_mp: f64,
    pub(crate) icon: String,
    pub(crate) properties: Vec<NpcProperty>,
    pub(crate) quest_infos: Vec<NpcQuestInfo>,

    pub(crate) mesh_params: WindowParams<NpcMeshParams, (), NpcMeshAction, ()>,
    pub(crate) sound_params: WindowParams<NpcSoundParams, (), NpcSoundAction, ()>,
    pub(crate) summon_params: WindowParams<NpcSummonParams, (), (), ()>,
    pub(crate) equipment_params: WindowParams<NpcEquipParams, (), (), ()>,
    pub(crate) additional_parts: WindowParams<Option<NpcAdditionalParts>, (), (), ()>,
    pub(crate) skill_animations:
        WindowParams<Vec<NpcSkillAnimation>, (), NpcSkillAnimationAction, ()>,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcSkillAnimation {
    pub(crate) id: SkillId,
    pub(crate) animation: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcQuestInfo {
    pub(crate) id: QuestId,
    pub(crate) step: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcSummonParams {
    pub(crate) summon_type: u8,
    pub(crate) max_count: u8,
    pub(crate) grade: u8,
    pub(crate) silhouette: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcDecorationEffect {
    pub(crate) effect: String,
    pub(crate) scale: f32,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcMeshParams {
    pub(crate) mesh: String,
    pub(crate) textures: Vec<String>,
    pub(crate) additional_textures: Vec<String>,
    pub(crate) decorations: Vec<NpcDecorationEffect>,
    pub(crate) attack_effect: String,
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

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcEquipParams {
    pub(crate) left_hand: ItemId,
    pub(crate) right_hand: ItemId,
    pub(crate) chest: ItemId,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcSoundParams {
    pub(crate) attack_sound: Vec<String>,
    pub(crate) defence_sound: Vec<String>,
    pub(crate) damage_sound: Vec<String>,
    pub(crate) dialog_sound: Vec<String>,
    pub(crate) vol: u8,
    pub(crate) rad: u8,
    pub(crate) random: u8,
    pub(crate) priority: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcQuest {
    pub(crate) id: QuestId,
    pub(crate) step: u32,
}
