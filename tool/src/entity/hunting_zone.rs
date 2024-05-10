use crate::backend::entity_editor::WindowParams;
use crate::backend::entity_impl::hunting_zone::MapObjectAction;
use crate::data::{HuntingZoneId, InstantZoneId, Location, NpcId, QuestId, RegionId};
use crate::entity::{CommonEntity, GetEditParams};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

impl GetEditParams<()> for HuntingZone {
    fn edit_params(&self) {}
}

impl CommonEntity<HuntingZoneId> for HuntingZone {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn desc(&self) -> String {
        self.desc.clone()
    }

    fn id(&self) -> HuntingZoneId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: HuntingZoneId) -> Self {
        HuntingZone {
            id,
            name: "New Zone".to_string(),
            desc: "New Zone Desc".to_string(),
            zone_type: HuntingZoneType::FieldHuntingZoneSolo,
            lvl_min: 1,
            lvl_max: 85,
            start_npc_loc: Default::default(),
            npc_id: Default::default(),
            quests: vec![],
            second_id: Default::default(),
            search_zone_id: Default::default(),
            instant_zone_id: Default::default(),
            world_map_objects: vec![],

            _changed: false,
            _deleted: false,
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
    Dominion,
    FieldHuntingZoneSolo,
    FieldHuntingZoneParty,
    InstanceZoneSolo,
    InstanceZoneParty,
    Agit,
    Village,
    Etc,
    Castle,
    Fortress,
    FieldHuntingZoneSoloAndParty,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
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

    pub(crate) second_id: u16,
    pub(crate) search_zone_id: RegionId,
    pub(crate) instant_zone_id: InstantZoneId,

    pub(crate) world_map_objects: Vec<WindowParams<MapObject, (), MapObjectAction, ()>>,

    pub _changed: bool,
    pub _deleted: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct MapObject {
    pub(crate) icon_texture: String,
    pub(crate) icon_texture_over: String,
    pub(crate) icon_texture_pressed: String,

    pub(crate) world_pos: [i32; 2],

    pub(crate) size: [u16; 2],
    pub(crate) desc_offset: [i16; 2],
    pub(crate) desc_font_name: String,

    pub(crate) unk1: Vec<i32>,
}
