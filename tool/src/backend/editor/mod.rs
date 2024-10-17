use crate::backend::editor::dictionaries::DictEditors;
use crate::backend::entity_impl::animation_combo::AnimationComboEditor;
use crate::backend::entity_impl::daily_missions::DailyMissionEditor;
use crate::backend::entity_impl::ensoul_option::EnsoulOptionEditor;
use crate::backend::entity_impl::hunting_zone::HuntingZoneEditor;
use crate::backend::entity_impl::item::armor::ArmorEditor;
use crate::backend::entity_impl::item::etc_item::EtcItemEditor;
use crate::backend::entity_impl::item::weapon::WeaponEditor;
use crate::backend::entity_impl::item_set::ItemSetEditor;
use crate::backend::entity_impl::npc::NpcEditor;
use crate::backend::entity_impl::quest::QuestEditor;
use crate::backend::entity_impl::raid_info::RaidInfoEditor;
use crate::backend::entity_impl::recipe::RecipeEditor;
use crate::backend::entity_impl::region::RegionEditor;
use crate::backend::entity_impl::residence::ResidenceEditor;
use crate::backend::entity_impl::skill::SkillEditor;
use crate::entity::GameEntity;
use ron::de::SpannedError;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::ops::Index;
use std::sync::RwLock;
use strum_macros::Display;

pub mod dictionaries;
pub mod entity;

pub trait EditParamsCommonOps {
    fn is_changed(&self) -> bool;
    fn on_save(&mut self);
    fn check_change(&mut self);
    fn handle_actions(&mut self);
    fn get_wrapped_entity_as_ron_string(&self) -> String;
    fn set_wrapped_entity_from_ron_string(&mut self, val: &str) -> Result<(), SpannedError>;
}

#[derive(Serialize, Deserialize, Default, Eq, PartialEq, Copy, Clone)]
pub enum CurrentEntity {
    #[default]
    None,
    Quest(usize),
    Skill(usize),
    Npc(usize),
    Weapon(usize),
    EtcItem(usize),
    Armor(usize),
    ItemSet(usize),
    Recipe(usize),
    HuntingZone(usize),
    Region(usize),
    RaidInfo(usize),
    DailyMission(usize),
    AnimationCombo(usize),
    Residence(usize),
    EnsoulOption(usize),
}

impl CurrentEntity {
    pub fn is_some(&self) -> bool {
        *self != CurrentEntity::None
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Editors {
    pub npcs: NpcEditor,
    pub quests: QuestEditor,
    pub skills: SkillEditor,
    pub weapons: WeaponEditor,
    pub armor: ArmorEditor,
    pub etc_items: EtcItemEditor,
    pub item_sets: ItemSetEditor,
    pub recipes: RecipeEditor,
    pub hunting_zones: HuntingZoneEditor,
    pub regions: RegionEditor,
    pub raid_info: RaidInfoEditor,
    pub daily_mission: DailyMissionEditor,
    pub animation_combo: AnimationComboEditor,
    pub residences: ResidenceEditor,
    pub ensoul_options: EnsoulOptionEditor,

    pub current_entity: CurrentEntity,

    #[serde(skip)]
    pub dictionaries: DictEditors,
}

impl Index<GameEntity> for Editors {
    type Output = dyn AgnosticEditorOps;

    fn index(&self, index: GameEntity) -> &Self::Output {
        match index {
            GameEntity::Npc => &self.npcs,
            GameEntity::Quest => &self.quests,
            GameEntity::Skill => &self.skills,
            GameEntity::Weapon => &self.weapons,
            GameEntity::Armor => &self.armor,
            GameEntity::EtcItem => &self.etc_items,
            GameEntity::ItemSet => &self.item_sets,
            GameEntity::Recipe => &self.recipes,
            GameEntity::HuntingZone => &self.hunting_zones,
            GameEntity::Region => &self.regions,
            GameEntity::RaidInfo => &self.raid_info,
            GameEntity::DailyMission => &self.daily_mission,
            GameEntity::AnimationCombo => &self.animation_combo,
            GameEntity::Residence => &self.residences,
            GameEntity::EnsoulOption => &self.ensoul_options,
        }
    }
}

pub trait AgnosticEditorOps {
    fn next_id(&self) -> u32;
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Debug)]
pub struct WindowParams<Inner, InitialId, Action, Params> {
    pub(crate) inner: Inner,
    pub(crate) opened: bool,
    pub(crate) initial_id: InitialId,
    pub(crate) action: RwLock<Action>,
    pub(crate) params: Params,
}

impl<Inner: Serialize, InitialId: Serialize, Action: Serialize, Params: Serialize>
    WindowParams<Inner, InitialId, Action, Params>
{
    fn serialize_full<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("WindowParams", 5)?;

        state.serialize_field("inner", &self.inner)?;
        state.serialize_field("opened", &self.opened)?;
        state.serialize_field("initial_id", &self.initial_id)?;
        state.serialize_field("action", &self.action)?;
        state.serialize_field("params", &self.params)?;

        state.end()
    }
}

impl<Inner: Serialize, InitialId, Action, Params> Serialize
    for WindowParams<Inner, InitialId, Action, Params>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, Inner, InitialId, Action, Params> WindowParams<Inner, InitialId, Action, Params>
where
    Inner: Deserialize<'de>,
    InitialId: Deserialize<'de>,
    Action: Deserialize<'de>,
    Params: Deserialize<'de>,
{
    fn deserialize_full<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WindowParamsVisitor<Inner, InitialId, Action, Params> {
            _inner: PhantomData<Inner>,
            _id: PhantomData<InitialId>,
            _action: PhantomData<Action>,
            _params: PhantomData<Params>,
        }

        #[derive(Deserialize, Display)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Inner,
            Opened,
            InitialId,
            Action,
            Params,
        }

        impl<'de, Inner, InitialId, Action, Params> Visitor<'de>
            for WindowParamsVisitor<Inner, InitialId, Action, Params>
        where
            Inner: Deserialize<'de>,
            InitialId: Deserialize<'de>,
            Action: Deserialize<'de>,
            Params: Deserialize<'de>,
        {
            type Value = WindowParams<Inner, InitialId, Action, Params>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WindowParams")
            }

            fn visit_seq<V>(
                self,
                mut seq: V,
            ) -> Result<WindowParams<Inner, InitialId, Action, Params>, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let inner = seq
                    .next_element::<Inner>()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let opened = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let initial_id = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let action = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let params = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;

                Ok(WindowParams {
                    inner,
                    opened,
                    initial_id,
                    action,
                    params,
                })
            }

            fn visit_map<V>(
                self,
                mut map: V,
            ) -> Result<WindowParams<Inner, InitialId, Action, Params>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut inner = None;
                let mut opened = None;
                let mut initial_id = None;
                let mut action = None;
                let mut params = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Inner => {
                            if inner.is_some() {
                                return Err(de::Error::duplicate_field("inner"));
                            }
                            inner = Some(map.next_value()?);
                        }
                        Field::Opened => {
                            if opened.is_some() {
                                return Err(de::Error::duplicate_field("opened"));
                            }
                            opened = Some(map.next_value()?);
                        }
                        Field::InitialId => {
                            if initial_id.is_some() {
                                return Err(de::Error::duplicate_field("initial_id"));
                            }
                            initial_id = Some(map.next_value()?);
                        }
                        Field::Action => {
                            if action.is_some() {
                                return Err(de::Error::duplicate_field("action"));
                            }
                            action = Some(map.next_value()?);
                        }
                        Field::Params => {
                            if params.is_some() {
                                return Err(de::Error::duplicate_field("params"));
                            }
                            params = Some(map.next_value()?);
                        }
                    }
                }

                let inner = inner.ok_or_else(|| de::Error::missing_field("inner"))?;
                let opened = opened.ok_or_else(|| de::Error::missing_field("opened"))?;
                let initial_id =
                    initial_id.ok_or_else(|| de::Error::missing_field("initial_id"))?;
                let action = action.ok_or_else(|| de::Error::missing_field("action"))?;
                let params = params.ok_or_else(|| de::Error::missing_field("params"))?;

                Ok(WindowParams {
                    inner,
                    opened,
                    initial_id,
                    action,
                    params,
                })
            }
        }

        const FIELDS: &[&str] = &["inner", "opened", "initial_id", "action", "params"];

        deserializer.deserialize_struct(
            "WindowParams",
            FIELDS,
            WindowParamsVisitor {
                _inner: Default::default(),
                _id: Default::default(),
                _action: Default::default(),
                _params: Default::default(),
            },
        )
    }
}

impl<'de, Inner, InitialId, Action, Params> Deserialize<'de>
    for WindowParams<Inner, InitialId, Action, Params>
where
    Inner: Deserialize<'de>,
    InitialId: Default,
    Action: Default,
    Params: Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            inner: Inner::deserialize(deserializer)?,
            opened: false,
            initial_id: InitialId::default(),
            action: RwLock::new(Action::default()),
            params: Params::default(),
        })
    }
}

impl<Inner: PartialEq, InitialId, Action, Params> PartialEq
    for WindowParams<Inner, InitialId, Action, Params>
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<Inner: Clone, OriginalId: Clone, Action: Default, Params: Clone> Clone
    for WindowParams<Inner, OriginalId, Action, Params>
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            opened: false,
            initial_id: self.initial_id.clone(),
            action: RwLock::new(Action::default()),
            params: self.params.clone(),
        }
    }
}

impl<T: Default, Action: Default, InitialId: Default, Params: Default> Default
    for WindowParams<T, InitialId, Action, Params>
{
    fn default() -> Self {
        Self {
            inner: T::default(),
            opened: false,
            initial_id: InitialId::default(),
            action: RwLock::new(Action::default()),
            params: Params::default(),
        }
    }
}

impl<T, Action: Default, InitialId: Default, Params: Default>
    WindowParams<T, InitialId, Action, Params>
{
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            opened: false,
            initial_id: InitialId::default(),
            action: RwLock::new(Action::default()),
            params: Params::default(),
        }
    }
}
