use crate::backend::entity_editor::WindowParams;
use crate::backend::entity_impl::quest::StepAction;
use crate::data::{HuntingZoneId, ItemId, Location, NpcId, PlayerClass, QuestId};
use crate::entity::{CommonEntity, GetEditParams};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use strum_macros::{Display, EnumIter};

impl GetEditParams<()> for Quest {
    fn edit_params(&self) {}
}

impl CommonEntity<QuestId> for Quest {
    fn name(&self) -> String {
        self.title.clone()
    }

    fn desc(&self) -> String {
        self.intro.clone()
    }

    fn id(&self) -> QuestId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: QuestId) -> Self {
        let mut c = Self {
            id,
            title: "New Quest".to_string(),
            intro: "".to_string(),
            requirements: "".to_string(),
            steps: vec![],
            last_finish_step_id: u32::MAX,
            quest_type: QuestType::Unk0,
            category: QuestCategory::Common,
            mark_type: MarkType::Unk1,
            min_lvl: 0,
            max_lvl: 0,
            allowed_classes: None,
            required_completed_quest_id: QuestId(0),
            search_zone_id: HuntingZoneId(0),
            _is_clan_pet_quest: false,
            start_npc_ids: vec![],
            start_npc_loc: Location::default(),
            rewards: vec![],
            quest_items: vec![],
            _faction_id: 0,
            _faction_level_min: 0,
            _faction_level_max: 0,
            java_class: None,

            _changed: false,
            _deleted: false,
        };

        c.add_normal_step();
        c.add_finish_step();

        c
    }
}

//Todo: разобраться
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
pub enum QuestType {
    #[default]
    Unk0,
    Unk1,
    Unk2,
    Unk3,
    Unk4,
    Unk5,
}

//Todo: разобраться
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
pub enum MarkType {
    #[default]
    Unk1,
    Unk2,
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
pub enum QuestCategory {
    #[default]
    Common,
    Unk1,
    Unk2,
    Unk3,
    Unk4,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Quest {
    pub id: QuestId,
    pub title: String,
    pub intro: String,
    pub requirements: String,
    pub steps: Vec<WindowParams<QuestStep, (), StepAction, ()>>,
    pub last_finish_step_id: u32,
    pub quest_type: QuestType,
    pub category: QuestCategory,
    pub mark_type: MarkType,
    pub min_lvl: u32,
    pub max_lvl: u32,
    pub allowed_classes: Option<Vec<PlayerClass>>,
    pub required_completed_quest_id: QuestId,
    pub search_zone_id: HuntingZoneId,
    pub(crate) _is_clan_pet_quest: bool,
    pub start_npc_loc: Location,
    pub start_npc_ids: Vec<NpcId>,
    pub rewards: Vec<QuestReward>,
    pub quest_items: Vec<ItemId>,
    ///unused
    pub(crate) _faction_id: u32,
    ///unused
    pub(crate) _faction_level_min: u32,
    ///unused
    pub(crate) _faction_level_max: u32,

    pub java_class: Option<WindowParams<String, (), (), ()>>,

    pub _changed: bool,
    pub _deleted: bool,
}

impl Quest {
    pub fn add_finish_step(&mut self) {
        self.last_finish_step_id -= 1;

        self.steps.push(WindowParams {
            inner: QuestStep {
                title: "FINISH".to_string(),
                desc: "".to_string(),
                goals: vec![],
                location: Location::default(),
                additional_locations: vec![],
                unk_q_level: vec![],
                _get_item_in_step: false,
                unk_1: Unk1::Unk0,
                unk_2: Unk2::Unk0,
                label: "".to_string(),
                prev_steps: vec![self.steps.len() as u32 - 2],
                level: self.last_finish_step_id,
            },

            initial_id: (),
            action: RwLock::new(StepAction::None),
            opened: false,
            params: (),
        });
    }

    pub fn add_normal_step(&mut self) {
        self.steps.push(WindowParams {
            inner: QuestStep {
                title: "Step Title".to_string(),
                desc: "Step Description".to_string(),
                goals: vec![],
                location: Location::default(),
                additional_locations: vec![],
                unk_q_level: vec![],
                _get_item_in_step: false,
                unk_1: Unk1::Unk0,
                unk_2: Unk2::Unk0,
                label: "Step Label".to_string(),
                prev_steps: vec![self.steps.len() as u32 - 2],
                level: self.steps.len() as u32 - 1,
            },

            initial_id: (),
            action: RwLock::new(StepAction::None),
            opened: false,
            params: (),
        });
    }
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
pub enum Unk1 {
    #[default]
    Unk0,
    Unk1,
    Unk2,
    Unk3,
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
pub enum Unk2 {
    #[default]
    Unk0,
    Unk1,
    Unk2,
    Unk3,
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
)]
pub enum UnkQLevel {
    Unk0,
    Unk1,
    Unk2,
    Unk3,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct QuestStep {
    pub title: String,
    pub label: String,
    pub desc: String,
    pub goals: Vec<StepGoal>,
    pub location: Location,
    pub additional_locations: Vec<Location>,
    //Todo: разобраться, в массиве встречаются числа от 0 до 3, может быть напимер [0, 0, 0, 0]
    pub unk_q_level: Vec<UnkQLevel>,
    ///True если будет получени предмет - квестовый или награда не важно
    pub(crate) _get_item_in_step: bool,
    ///Todo: разобраться
    ///
    ///Если больше 1, то всегда одинаковые
    pub unk_1: Unk1,
    pub unk_2: Unk2,
    pub prev_steps: Vec<u32>,
    pub level: u32,
}

#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone, PartialEq)]
pub struct QuestReward {
    pub reward_id: ItemId,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, Default)]
pub enum GoalType {
    ///Записывается как тип 0, а id цели прибаваляется к 1_000_000
    ///# Пример
    ///Охранник Гелиос просит Вас уничтожить монстров из Леса Разбойников. Цели охоты:
    ///* Горгулья Охотник (20241),
    ///* Василиск Тарлк (20573),
    ///* Старший Василиск Тарлк (20574)
    ///
    ///`[('1020241', '0', '15'), ('1020573', '0', '20'), ('1020574', '0', '20')]`
    #[default]
    KillNpc,
    ///Записывается как тип 0, а id как есть
    ///# Пример
    ///После того, как Вы уничтожили Разгневанного Духа, перед Вами появился Призрак Гнолла и выразил Вам свою благодарность.
    ///
    /// * Соберите еще несколько Знаков Благодарности (39508).
    ///
    ///`[('39508', '0', '5')]`
    CollectItem,
    ///Показывает нпс стринг, номер указывается в `target_id`, `count` должен быть 0
    Other,
}

impl GoalType {
    pub(crate) fn as_pair(&self, id: u32) -> (u32, u32) {
        match self {
            GoalType::KillNpc => (1_000_000 + id, 0),
            GoalType::CollectItem => (id, 0),
            GoalType::Other => (id, 1),
        }
    }

    pub(crate) fn from_pair(id: u32, s: u32) -> (u32, Self) {
        if s == 0 {
            if id > 1_000_000 {
                (id - 1_000_000, Self::KillNpc)
            } else {
                (id, Self::CollectItem)
            }
        } else if s == 1 {
            (id, Self::Other)
        } else {
            unreachable!()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct StepGoal {
    pub target_id: u32,
    pub goal_type: GoalType,
    pub count: u32,
}
