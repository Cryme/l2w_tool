use crate::data::{HuntingZoneId, InstantZoneId, Location, NpcId, QuestId, RegionId, SearchZoneId};
use crate::entity::CommonEntity;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

impl CommonEntity<HuntingZoneId, ()> for HuntingZone {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn desc(&self) -> String {
        self.desc.clone()
    }

    fn id(&self) -> HuntingZoneId {
        self.id
    }

    fn edit_params(&self) {}

    fn new(id: HuntingZoneId) -> Self {
        HuntingZone {
            id,
            name: "New Zone".to_string(),
            desc: "New Zone Desc".to_string(),
            zone_type: HuntingZoneType::Unk0,
            lvl_min: 1,
            lvl_max: 85,
            start_npc_loc: Default::default(),
            npc_id: Default::default(),
            quests: vec![],
            region_id: Default::default(),
            search_zone_id: Default::default(),
            instant_zone_id: Default::default(),
        }
    }
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
pub enum HuntingZoneType {
    #[default]
    Unk0,
    Unk1,
    Unk2,
    Unk3,
    Unk4,
    Unk5,
    Unk6,
    Unk7,
    Unk8,
    Unk9,
    Unk10,
    Unk11,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct HuntingZone {
    pub(crate) id: HuntingZoneId,
    pub(crate) name: String,
    pub(crate) desc: String,

    pub(crate) zone_type: HuntingZoneType,
    pub(crate) lvl_min: u32,
    pub(crate) lvl_max: u32,

    pub(crate) start_npc_loc: Location,
    pub(crate) npc_id: NpcId,

    pub(crate) quests: Vec<QuestId>,

    pub(crate) region_id: RegionId,
    pub(crate) search_zone_id: SearchZoneId,
    pub(crate) instant_zone_id: InstantZoneId,
}
