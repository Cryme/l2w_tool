#![allow(dead_code)]
use crate::data::{ItemId, NpcId, QuestId, SkillId};
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Npc {
    pub(crate) id: NpcId,
    pub(crate) name: String,
    pub(crate) title: String,
    pub(crate) title_color: Color32,
}

// #[derive(Clone, Serialize, Deserialize, Default, Debug)]
// pub struct Npc {
//     pub(crate) id: NpcId,
//     pub(crate) name: String,
//     pub(crate) title: String,
//     pub(crate) title_color: Color32,
//     pub(crate) npc_type: u16,
//     pub(crate) unreal_script_class: String,
//     pub(crate) model_params: NpcModelParams,
//     pub(crate) sound_params: NpcSoundParams,
//     pub(crate) summon_params: Option<SummonParams>,
//     pub(crate) equipment: NpcEquipParams,
//     pub(crate) skill_animations: Vec<NpcSkillAnimation>,
//     pub(crate) properties: Vec<u16>,
//     pub(crate) social: u8,
//     pub(crate) show_hp: bool,
//     pub(crate) org_hp: f64,
//     pub(crate) org_mp: f64,
//     pub(crate) icon: String,
// }

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcSkillAnimation {
    pub(crate) id: SkillId,
    pub(crate) animation: String,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct SummonParams {
    pub(crate) summon_type: u8,
    pub(crate) max_count: u8,
    pub(crate) grade: u8,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcDecorationEffect {
    pub(crate) effect: String,
    pub(crate) scale: f32,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcModelParams {
    pub(crate) mesh: String,
    pub(crate) texture: String,
    pub(crate) additional_textures: Vec<String>,
    pub(crate) decorations: Vec<NpcDecorationEffect>,
    pub(crate) attack_effect: String,
    pub(crate) speed: f32,
    pub(crate) draw_scale: f32,
    pub(crate) use_zoomincam: f32,
    pub(crate) run_speed: u8,
    pub(crate) walk_speed: u8,
    pub(crate) collision_radius_1: f32,
    pub(crate) collision_radius_2: f32,
    pub(crate) collision_height_1: f32,
    pub(crate) collision_height_2: f32,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct NpcEquipParams {
    left_hand: ItemId,
    right_hand: ItemId,
    chest: ItemId,
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
