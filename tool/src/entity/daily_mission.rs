use crate::backend::util::Localized;
use crate::common::{DailyMissionId, ItemId, PlayerClass};
use crate::entity::{CommonEntity, GetEditParams};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

impl GetEditParams<()> for DailyMission {
    fn edit_params(&self) {}
}

impl CommonEntity<DailyMissionId> for DailyMission {
    fn name(&self) -> String {
        self.name.ru.clone()
    }

    fn desc(&self) -> String {
        self.desc.ru.clone()
    }

    fn id(&self) -> DailyMissionId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: DailyMissionId) -> Self {
        DailyMission {
            id,

            reward_id: 0,
            name: (
                "Новое Ежедневное Задание".to_string(),
                "New Daily Mission".to_string(),
            )
                .into(),
            desc: ("Description".to_string(), "Описание".to_string()).into(),
            category: ("Onetime".to_string(), "Onetime".to_string()).into(),
            category_type: 0,
            allowed_classes: None,
            repeat_type: Default::default(),
            unk2: 0,
            unk3: 0,
            unk4: 0,
            unk5: 0,
            unk6: 0,
            unk7: vec![],
            unk8: vec![],
            rewards: vec![],

            _changed: false,
            _deleted: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct DailyMissionUnk7 {
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
}

#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone, PartialEq)]
pub struct DailyMissionReward {
    pub item_id: ItemId,
    pub count: u32,
}

#[derive(
    Serialize,
    Deserialize,
    Display,
    Debug,
    EnumIter,
    Eq,
    PartialEq,
    Copy,
    Clone,
    FromPrimitive,
    ToPrimitive,
    Default,
)]
pub enum DailyMissionRepeatType {
    #[default]
    Unk,
    Daily,
    Weekly,
    Monthly,
    OneTime,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct DailyMission {
    pub id: DailyMissionId,
    pub reward_id: u32,
    pub name: Localized<String>,
    pub desc: Localized<String>,
    pub category: Localized<String>,
    pub category_type: u32,

    pub allowed_classes: Option<Vec<PlayerClass>>,

    pub repeat_type: DailyMissionRepeatType,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
    pub unk7: Vec<DailyMissionUnk7>,
    pub unk8: Vec<u32>,
    pub rewards: Vec<DailyMissionReward>,

    #[serde(skip)]
    pub _changed: bool,
    #[serde(skip)]
    pub _deleted: bool,
}
