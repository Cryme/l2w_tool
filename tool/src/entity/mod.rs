use strum_macros::{Display, EnumIter};

pub mod hunting_zone;
pub mod item;
pub mod npc;
pub mod quest;
pub mod skill;

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone)]
pub enum Entity {
    Quest,
    Skill,
    Npc,
}
