use crate::data::NpcId;
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Npc {
    pub(crate) id: NpcId,
    pub(crate) name: String,
    pub(crate) title: String,
    pub(crate) title_color: Color32,
}

//TODO: поправить на сервере, убрать скилл 4416 который задает рассу в NpcParser 199 строка
#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NpcRace {
    None,
    Undead,
    MagicCreature,
    Beast,
    Animal,
    Plant,
    Humanoid,
    Spirit,
    Angel,
    Demon,
    Dragon,
    Giant,
    Bug,
    Fairy,
    Human,
    Elf,
    DarkElf,
    Orc,
    Dwarf,
    Other,
    NonLivingBeing,
    SiegeWeapon,
    DefendingArmy,
    Mercenary,
    UnknownCreature,
    Kamael,
}
