use crate::data::{AnimationComboId, DailyMissionId, HuntingZoneId, ItemId, ItemSetId, NpcId, QuestId, RaidInfoId, RecipeId, RegionId, SkillId};
use strum_macros::{Display, EnumIter};

pub mod daily_mission;
pub mod hunting_zone;
pub mod item;
pub mod item_set;
pub mod npc;
pub mod quest;
pub mod raid_info;
pub mod recipe;
pub mod region;
pub mod skill;
pub mod animation_combo;

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone)]
pub enum Entity {
    Quest,
    Skill,
    Npc,
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
}

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone)]
pub enum EntityT {
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
