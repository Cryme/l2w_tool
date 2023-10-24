use crate::data::NpcId;
use eframe::egui::Color32;

pub struct Npc {
    pub(crate) id: NpcId,
    pub(crate) name: String,
    pub(crate) title: String,
    pub(crate) title_color: Color32,
}
