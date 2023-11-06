use crate::backend::{StepAction, WindowParams};
use crate::data::{HuntingZoneId, ItemId, Location, NpcId, PlayerClass, QuestId};
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

//Todo: разобраться
#[derive(
    Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive,
)]
pub enum QuestType {
    Unk0,
    Unk1,
    Unk2,
    Unk3,
    Unk4,
    Unk5,
}

//Todo: разобраться
#[derive(
    Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive,
)]
pub enum MarkType {
    Unk1,
    Unk2,
}

#[derive(
    Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive,
)]
pub enum QuestCategory {
    Common,
    Unk1,
    Unk2,
    Unk3,
    Unk4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Quest {
    pub id: QuestId,
    pub title: String,
    pub intro: String,
    pub requirements: String,
    pub steps: Vec<WindowParams<QuestStep, (), StepAction>>,
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

    pub java_class: Option<WindowParams<String, (), ()>>,
}

impl Quest {
    pub fn new(id: u32) -> Self {
        Self {
            id: QuestId(id),
            title: "New Quest".to_string(),
            intro: "".to_string(),
            requirements: "".to_string(),
            steps: vec![],
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
        }
    }

    pub fn clone_and_unescape(&self) -> Self {
        let mut dub = self.clone();
        dub.unescape_special_characters();

        dub
    }

    pub fn clone_and_escape(&self) -> Self {
        let mut dub = self.clone();
        dub.escape_special_characters();

        dub
    }

    pub fn escape_special_characters(&mut self) {
        self.intro = self.intro.replace('\n', "\\n");
        self.requirements = self.requirements.replace('\n', "\\n");

        for s in &mut self.steps {
            s.inner.desc = s.inner.desc.replace('\n', "\\n");
        }
    }

    pub fn unescape_special_characters(&mut self) {
        self.intro = self.intro.replace("\\n", "\n");
        self.requirements = self.requirements.replace("\\n", "\n");

        for s in &mut self.steps {
            s.inner.desc = s.inner.desc.replace("\\n", "\n");
        }
    }

    pub fn add_start_npc_id(&mut self) {
        self.start_npc_ids.push(NpcId(0));
    }

    pub fn add_step(&mut self) {
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
                prev_step_indexes: vec![self.steps.len()],
            },

            original_id: (),
            action: StepAction::None,
            opened: false,
        });
    }

    pub fn add_reward(&mut self) {
        self.rewards.push(QuestReward {
            reward_id: ItemId(0),
            count: 0,
        });
    }

    pub fn add_quest_item(&mut self) {
        self.quest_items.push(ItemId(0));
    }
}

#[derive(
    Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive,
)]
pub enum Unk1 {
    Unk0,
    Unk1,
    Unk2,
    Unk3,
}

#[derive(
    Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive,
)]
pub enum Unk2 {
    Unk0,
    Unk1,
    Unk2,
    Unk3,
}

#[derive(
    Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive,
)]
pub enum UnkQLevel {
    Unk0,
    Unk1,
    Unk2,
    Unk3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub prev_step_indexes: Vec<usize>,
}

impl QuestStep {
    pub fn add_goal(&mut self) {
        self.goals.push(StepGoal {
            target_id: 0,
            goal_type: GoalType::CollectItem,
            count: 0,
        })
    }

    pub fn add_additional_location(&mut self) {
        self.additional_locations.push(Location::default());
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct QuestReward {
    pub reward_id: ItemId,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone)]
pub enum GoalType {
    ///Записывается как тип 0, а id цели прибаваляется к 1_000_000
    ///# Пример
    ///Охранник Гелиос просит Вас уничтожить монстров из Леса Разбойников. Цели охоты:
    ///* Горгулья Охотник (20241),
    ///* Василиск Тарлк (20573),
    ///* Старший Василиск Тарлк (20574)
    ///
    ///`[('1020241', '0', '15'), ('1020573', '0', '20'), ('1020574', '0', '20')]`
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

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub struct StepGoal {
    pub target_id: u32,
    pub goal_type: GoalType,
    pub count: u32,
}
