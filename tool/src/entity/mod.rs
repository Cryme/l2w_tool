use crate::backend::holder::DictItem;
use crate::common::{
    AnimationComboId, DailyMissionId, HuntingZoneId, ItemId, ItemSetId, NpcId, QuestId, RaidInfoId,
    RecipeId, RegionId, ResidenceId, SkillId,
};
use std::fmt::{Debug, Display, Formatter};
use strum_macros::{Display, EnumIter};

pub mod animation_combo;
pub mod daily_mission;
pub mod hunting_zone;
pub mod item;
pub mod item_set;
pub mod npc;
pub mod quest;
pub mod raid_info;
pub mod recipe;
pub mod region;
pub mod residence;
pub mod skill;

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone)]
pub enum Dictionary {
    SystemStrings,
    NpcStrings,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Entity {
    GameEntity(GameEntity),
    Dictionary(Dictionary),
}

impl From<Dictionary> for Entity {
    fn from(value: Dictionary) -> Self {
        Self::Dictionary(value)
    }
}

impl From<GameEntity> for Entity {
    fn from(value: GameEntity) -> Self {
        Self::GameEntity(value)
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Entity::GameEntity(e) => std::fmt::Display::fmt(&e, f),
            Entity::Dictionary(e) => std::fmt::Display::fmt(&e, f),
        }
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Entity::GameEntity(e) => std::fmt::Debug::fmt(&e, f),
            Entity::Dictionary(e) => std::fmt::Debug::fmt(&e, f),
        }
    }
}

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone)]
pub enum GameEntity {
    Npc,
    Quest,
    Skill,
    Weapon,
    Armor,
    EtcItem,
    ItemSet,
    Recipe,
    HuntingZone,
    Region,
    RaidInfo,
    DailyMission,
    AnimationCombo,
    Residence,
}

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone)]
pub enum GameEntityT {
    Quest(QuestId),
    Skill(SkillId),
    Npc(NpcId),
    Weapon(ItemId),
    Armor(ItemId),
    EtcItem(ItemId),
    ItemSet(ItemSetId),
    Recipe(RecipeId),
    HuntingZone(HuntingZoneId),
    Region(RegionId),
    RaidInfo(RaidInfoId),
    DailyMission(DailyMissionId),
    AnimationCombo(AnimationComboId),
    Residence(ResidenceId),
}

pub trait GetEditParams<EditParams> {
    fn edit_params(&self) -> EditParams;
}

pub trait CommonEntity<EntityId> {
    fn name(&self) -> String;
    fn desc(&self) -> String;
    fn id(&self) -> EntityId;
    fn changed(&self) -> bool;
    fn deleted(&self) -> bool;
    fn new(id: EntityId) -> Self;
}

impl CommonEntity<u32> for String {
    fn name(&self) -> String {
        unreachable!()
    }

    fn desc(&self) -> String {
        unreachable!()
    }

    fn id(&self) -> u32 {
        unreachable!()
    }

    fn changed(&self) -> bool {
        unreachable!()
    }

    fn deleted(&self) -> bool {
        unreachable!()
    }

    fn new(_: u32) -> Self {
        unreachable!()
    }
}

impl CommonEntity<u32> for DictItem<u32, String> {
    fn name(&self) -> String {
        unreachable!()
    }

    fn desc(&self) -> String {
        unreachable!()
    }

    fn id(&self) -> u32 {
        unreachable!()
    }

    fn changed(&self) -> bool {
        unreachable!()
    }

    fn deleted(&self) -> bool {
        unreachable!()
    }

    fn new(_: u32) -> Self {
        unreachable!()
    }
}
