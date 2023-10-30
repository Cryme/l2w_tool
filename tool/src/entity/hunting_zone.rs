use num_derive::FromPrimitive;
use strum_macros::{Display, EnumIter};
use crate::data::{HuntingZoneId, InstantZoneId, SearchZoneId};

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive)]
pub enum HuntingZoneType {
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

#[derive(Clone)]
pub struct HuntingZone {
    pub(crate) id: HuntingZoneId,
    pub(crate) name: String,
    pub(crate) desc: String,
    pub(crate) search_zone_id: SearchZoneId,
    pub(crate) instant_zone_id: InstantZoneId
}