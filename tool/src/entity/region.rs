use crate::backend::region::MapObjectAction;
use crate::backend::WindowParams;
use crate::data::RegionId;
use crate::entity::CommonEntity;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

impl CommonEntity<RegionId, ()> for Region {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn desc(&self) -> String {
        "".to_string()
    }

    fn id(&self) -> RegionId {
        self.id
    }

    fn edit_params(&self) {}

    fn new(id: RegionId) -> Self {
        Region {
            id,
            name: "".to_string(),
            world_map_square: [15, 15],
            z_range: [30_000., -30_000.],
            map_info: None,
            color_code: 0,
            continent: Default::default(),
            current_layer: 0,
            total_layers: 0,
            world_map_objects: vec![],
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
pub enum Continent {
    #[default]
    Aden,
    Gracia,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Region {
    pub(crate) id: RegionId,
    pub(crate) name: String,

    pub(crate) world_map_square: [u16; 2],
    pub(crate) z_range: [f32; 2],

    pub(crate) map_info: Option<MapInfo>,

    pub(crate) color_code: u16,
    pub(crate) continent: Continent,

    pub(crate) current_layer: u16,
    pub(crate) total_layers: u16,

    pub(crate) world_map_objects: Vec<WindowParams<MapObject, (), MapObjectAction, ()>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct MapInfo {
    pub(crate) button_pos: Option<[i16; 2]>,
    pub(crate) pos: [i32; 2],
    pub(crate) size: [u16; 2],
    pub(crate) center: [i32; 2],
    pub(crate) scale: f32,
    pub(crate) texture: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
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
