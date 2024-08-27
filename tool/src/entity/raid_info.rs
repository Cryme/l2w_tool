use crate::common::{HuntingZoneId, NpcId, Position, RaidInfoId};
use crate::entity::{CommonEntity, GetEditParams};
use serde::{Deserialize, Serialize};

impl GetEditParams<()> for RaidInfo {
    fn edit_params(&self) {}
}

impl CommonEntity<RaidInfoId> for RaidInfo {
    fn name(&self) -> String {
        "Raid Info".to_string()
    }

    fn desc(&self) -> String {
        "".to_string()
    }

    fn id(&self) -> RaidInfoId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: RaidInfoId) -> Self {
        RaidInfo {
            id,

            raid_id: NpcId(1),
            raid_lvl: 0,
            search_zone_id: HuntingZoneId(1),
            loc: Default::default(),
            desc: "New raid".to_string(),
            recommended_level_min: 1,
            recommended_level_max: 85,

            _changed: false,
            _deleted: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct RaidInfo {
    pub id: RaidInfoId,
    pub raid_id: NpcId,
    pub raid_lvl: u32,
    pub search_zone_id: HuntingZoneId,
    pub loc: Position,
    pub desc: String,
    pub recommended_level_min: u8,
    pub recommended_level_max: u8,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}
