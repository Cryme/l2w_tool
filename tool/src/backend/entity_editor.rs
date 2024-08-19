use crate::backend::entity_impl::animation_combo::AnimationComboEditor;
use crate::backend::entity_impl::daily_missions::DailyMissionEditor;
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
use crate::backend::entity_impl::skill::SkillEditor;
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::backend::HandleAction;
use crate::entity::{CommonEntity, Entity, EntityT, GetEditParams};
use ron::de::SpannedError;
use ron::ser::PrettyConfig;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::ops::Index;
use std::sync::RwLock;
use crate::backend::entity_impl::residence::ResidenceEditor;

pub trait EditParamsCommonOps {
    fn is_changed(&self) -> bool;
    fn on_save(&mut self);
    fn check_change(&mut self);
    fn handle_actions(&mut self);
    fn get_wrapped_entity_as_ron_string(&self) -> String;
    fn set_wrapped_entity_from_ron_string(&mut self, val: &str) -> Result<(), SpannedError>;
}

impl<
        Entity: PartialEq + Clone + Serialize + DeserializeOwned,
        EntityId: From<u32>,
        EditAction,
        EditParams,
    > EditParamsCommonOps for ChangeTrackedParams<Entity, EntityId, EditAction, EditParams>
where
    WindowParams<Entity, EntityId, EditAction, EditParams>:
        HandleAction + Serialize + DeserializeOwned,
{
    fn is_changed(&self) -> bool {
        self.changed || self.is_new
    }

    fn on_save(&mut self) {
        self.changed = false;
        self.is_new = false;
        self.initial = self.inner.inner.clone();
    }

    fn check_change(&mut self) {
        self.changed = self.inner.inner != self.initial;
    }

    fn handle_actions(&mut self) {
        self.inner.handle_action()
    }

    fn get_wrapped_entity_as_ron_string(&self) -> String {
        ron::ser::to_string_pretty(&self.inner, PrettyConfig::default().struct_names(true)).unwrap()
    }

    fn set_wrapped_entity_from_ron_string(&mut self, val: &str) -> Result<(), SpannedError> {
        let r = ron::from_str(val);

        match r {
            Ok(r) => {
                self.inner = r;
                self.inner.initial_id = u32::MAX.into();
                self.is_new = true;

                Ok(())
            }
            Err(e) => Err(e),
        }
    }
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
}

impl CurrentEntity {
    pub fn is_some(&self) -> bool {
        *self != CurrentEntity::None
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChangeTrackedParams<Entity, EntityId, EditAction, EditParams> {
    pub inner: WindowParams<Entity, EntityId, EditAction, EditParams>,
    pub initial: Entity,
    changed: bool,
    pub is_new: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EntityEditParams<Entity, EntityId, EditAction, EditParams> {
    #[serde(skip)]
    pub(crate) next_id: u32,
    pub opened: Vec<ChangeTrackedParams<Entity, EntityId, EditAction, EditParams>>,
}

impl<Entity, EntityId, EditAction, EditParams> Default
    for EntityEditParams<Entity, EntityId, EditAction, EditParams>
{
    fn default() -> Self {
        Self {
            next_id: 0,
            opened: vec![],
        }
    }
}

pub trait CommonEditorOps<
    Entity: CommonEntity<EntityId> + Clone,
    EntityId: Hash + Eq + Copy + Clone,
    Action,
    Params,
>
{
    fn reset_initial<Map: HolderMapOps<EntityId, Entity>>(&mut self, map: &Map);
    fn get_opened_info(&self) -> Vec<(String, EntityId, bool)>;
    fn add(&mut self, e: Entity, original_id: EntityId, is_new: bool) -> usize;
    fn add_new(&mut self) -> usize;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<
        Entity: CommonEntity<EntityId> + GetEditParams<EditParams> + Clone,
        EntityId: From<u32> + Copy + Clone + Hash + Eq,
        EditAction: Default,
        EditParams,
    > CommonEditorOps<Entity, EntityId, EditAction, EditParams>
    for EntityEditParams<Entity, EntityId, EditAction, EditParams>
{
    fn reset_initial<Map: HolderMapOps<EntityId, Entity>>(&mut self, map: &Map) {
        for v in &mut self.opened {
            if let Some(ent) = map.get(&v.inner.initial_id) {
                v.initial = ent.clone();
                v.is_new = false;
            } else {
                v.is_new = true;
                v.inner.initial_id = EntityId::from(u32::MAX)
            }
        }
    }

    fn get_opened_info(&self) -> Vec<(String, EntityId, bool)> {
        self.opened
            .iter()
            .map(|v| {
                (
                    v.inner.inner.name(),
                    v.inner.inner.id(),
                    v.changed || v.is_new,
                )
            })
            .collect()
    }

    fn add(&mut self, e: Entity, original_id: EntityId, is_new: bool) -> usize {
        self.opened.push(ChangeTrackedParams {
            initial: e.clone(),
            inner: WindowParams {
                params: e.edit_params(),
                inner: e,
                initial_id: original_id,
                opened: false,
                action: RwLock::new(Default::default()),
            },
            changed: false,
            is_new,
        });

        self.opened.len() - 1
    }

    fn add_new(&mut self) -> usize {
        let id = EntityId::from(self.next_id);
        self.add(Entity::new(id), id, true);

        self.next_id += 1;

        self.opened.len() - 1
    }

    fn len(&self) -> usize {
        self.opened.len()
    }

    fn is_empty(&self) -> bool {
        self.opened.is_empty()
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct EditParams {
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

    pub current_entity: CurrentEntity,
}

impl Index<Entity> for EditParams {
    type Output = dyn AgnosticEditorOps;

    fn index(&self, index: Entity) -> &Self::Output {
        match index {
            Entity::Npc => &self.npcs,
            Entity::Quest => &self.quests,
            Entity::Skill => &self.skills,
            Entity::Weapon => &self.weapons,
            Entity::Armor => &self.armor,
            Entity::EtcItem => &self.etc_items,
            Entity::ItemSet => &self.item_sets,
            Entity::Recipe => &self.recipes,
            Entity::HuntingZone => &self.hunting_zones,
            Entity::Region => &self.regions,
            Entity::RaidInfo => &self.raid_info,
            Entity::DailyMission => &self.daily_mission,
            Entity::AnimationCombo => &self.animation_combo,
            Entity::Residence => &self.residences,
        }
    }
}

pub trait AgnosticEditorOps {
    fn next_id(&self) -> u32;
}

impl<Entity: CommonEntity<EntityId> + Clone, EntityId: Hash + Eq + Copy + Clone, Action, Params> AgnosticEditorOps for EntityEditParams<Entity, EntityId, Action, Params> {
    fn next_id(&self) -> u32 {
        self.next_id
    }
}
impl EditParams {
    pub fn close_if_opened(&mut self, entity: EntityT) {
        match entity {
            EntityT::Quest(id) => {
                if let Some((i, _)) = self
                    .quests
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.quests.opened.remove(i);
                }
            }
            EntityT::Skill(id) => {
                if let Some((i, _)) = self
                    .skills
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.skills.opened.remove(i);
                }
            }
            EntityT::Npc(id) => {
                if let Some((i, _)) = self
                    .npcs
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.npcs.opened.remove(i);
                }
            }
            EntityT::Weapon(id) => {
                if let Some((i, _)) = self
                    .weapons
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.weapons.opened.remove(i);
                }
            }
            EntityT::Armor(id) => {
                if let Some((i, _)) = self
                    .armor
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.armor.opened.remove(i);
                }
            }
            EntityT::EtcItem(id) => {
                if let Some((i, _)) = self
                    .etc_items
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.etc_items.opened.remove(i);
                }
            }
            EntityT::ItemSet(id) => {
                if let Some((i, _)) = self
                    .item_sets
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.etc_items.opened.remove(i);
                }
            }
            EntityT::Recipe(id) => {
                if let Some((i, _)) = self
                    .recipes
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.recipes.opened.remove(i);
                }
            }
            EntityT::HuntingZone(id) => {
                if let Some((i, _)) = self
                    .hunting_zones
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.hunting_zones.opened.remove(i);
                }
            }
            EntityT::Region(id) => {
                if let Some((i, _)) = self
                    .regions
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.regions.opened.remove(i);
                }
            }
            EntityT::RaidInfo(id) => {
                if let Some((i, _)) = self
                    .raid_info
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.raid_info.opened.remove(i);
                }
            }
            EntityT::DailyMission(id) => {
                if let Some((i, _)) = self
                    .daily_mission
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.daily_mission.opened.remove(i);
                }
            }
            EntityT::AnimationCombo(id) => {
                if let Some((i, _)) = self
                    .animation_combo
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.animation_combo.opened.remove(i);
                }
            }
            EntityT::Residence(id) => {
                if let Some((i, _)) = self
                    .residences
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.animation_combo.opened.remove(i);
                }
            }
        }

        self.find_opened_entity();
    }
    pub fn reset_initial(&mut self, entity: Entity, holders: &GameDataHolder) {
        match entity {
            Entity::Quest => self.quests.reset_initial(&holders.quest_holder),
            Entity::Skill => self.skills.reset_initial(&holders.skill_holder),
            Entity::Npc => self.npcs.reset_initial(&holders.npc_holder),
            Entity::Weapon => self.weapons.reset_initial(&holders.weapon_holder),
            Entity::Armor => self.armor.reset_initial(&holders.armor_holder),
            Entity::EtcItem => self.etc_items.reset_initial(&holders.etc_item_holder),
            Entity::ItemSet => self.item_sets.reset_initial(&holders.item_set_holder),
            Entity::Recipe => self.recipes.reset_initial(&holders.recipe_holder),
            Entity::HuntingZone => self
                .hunting_zones
                .reset_initial(&holders.hunting_zone_holder),
            Entity::Region => self.regions.reset_initial(&holders.region_holder),
            Entity::RaidInfo => self.raid_info.reset_initial(&holders.raid_info_holder),
            Entity::DailyMission => self
                .daily_mission
                .reset_initial(&holders.daily_mission_holder),
            Entity::AnimationCombo => self
                .animation_combo
                .reset_initial(&holders.animation_combo_holder),
            Entity::Residence => self.residences.reset_initial(&holders.residence_holder),
        }
    }
    pub(crate) fn find_opened_entity(&mut self) {
        match self.current_entity {
            CurrentEntity::Quest(i) => {
                if !self.quests.opened.is_empty() {
                    self.current_entity = CurrentEntity::Quest(i.min(self.quests.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::Skill(i) => {
                if !self.skills.opened.is_empty() {
                    self.current_entity = CurrentEntity::Skill(i.min(self.skills.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::Npc(i) => {
                if !self.npcs.opened.is_empty() {
                    self.current_entity = CurrentEntity::Npc(i.min(self.npcs.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::Weapon(i) => {
                if !self.weapons.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::Weapon(i.min(self.weapons.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::EtcItem(i) => {
                if !self.etc_items.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::EtcItem(i.min(self.etc_items.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::Armor(i) => {
                if !self.armor.opened.is_empty() {
                    self.current_entity = CurrentEntity::Armor(i.min(self.armor.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::ItemSet(i) => {
                if !self.item_sets.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::ItemSet(i.min(self.item_sets.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::Recipe(i) => {
                if !self.recipes.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::Recipe(i.min(self.recipes.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::HuntingZone(i) => {
                if !self.hunting_zones.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::HuntingZone(i.min(self.hunting_zones.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::Region(i) => {
                if !self.regions.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::Region(i.min(self.regions.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::RaidInfo(i) => {
                if !self.raid_info.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::RaidInfo(i.min(self.raid_info.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::DailyMission(i) => {
                if !self.daily_mission.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::DailyMission(i.min(self.daily_mission.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::AnimationCombo(i) => {
                if !self.animation_combo.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::AnimationCombo(i.min(self.animation_combo.opened.len() - 1));

                    return;
                }
            }
            CurrentEntity::Residence(i) => {
                if !self.residences.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::Residence(i.min(self.residences.opened.len() - 1));

                    return;
                }
            }

            CurrentEntity::None => {}
        }

        if !self.quests.is_empty() {
            self.current_entity = CurrentEntity::Quest(self.quests.len() - 1);
        } else if !self.skills.is_empty() {
            self.current_entity = CurrentEntity::Skill(self.skills.len() - 1);
        } else if !self.npcs.is_empty() {
            self.current_entity = CurrentEntity::Npc(self.npcs.len() - 1);
        } else if !self.weapons.is_empty() {
            self.current_entity = CurrentEntity::Weapon(self.weapons.len() - 1);
        } else if !self.armor.is_empty() {
            self.current_entity = CurrentEntity::Armor(self.armor.len() - 1);
        } else if !self.etc_items.is_empty() {
            self.current_entity = CurrentEntity::EtcItem(self.etc_items.len() - 1);
        } else if !self.item_sets.is_empty() {
            self.current_entity = CurrentEntity::ItemSet(self.item_sets.len() - 1);
        } else if !self.recipes.is_empty() {
            self.current_entity = CurrentEntity::Recipe(self.recipes.len() - 1);
        } else if !self.hunting_zones.is_empty() {
            self.current_entity = CurrentEntity::HuntingZone(self.hunting_zones.len() - 1);
        } else if !self.regions.is_empty() {
            self.current_entity = CurrentEntity::Region(self.regions.len() - 1);
        } else if !self.raid_info.is_empty() {
            self.current_entity = CurrentEntity::RaidInfo(self.raid_info.len() - 1);
        } else if !self.daily_mission.is_empty() {
            self.current_entity = CurrentEntity::DailyMission(self.daily_mission.len() - 1);
        } else if !self.animation_combo.is_empty() {
            self.current_entity = CurrentEntity::AnimationCombo(self.animation_combo.len() - 1);
        } else if !self.residences.is_empty() {
            self.current_entity = CurrentEntity::Residence(self.residences.len() - 1);
        } else {
            self.current_entity = CurrentEntity::None;
        }
    }
}

/*
----------------------------------------------------------------------------------------------------
----------------------------------------------------------------------------------------------------
*/

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowParams<Inner, InitialId, Action, Params> {
    pub(crate) inner: Inner,
    pub(crate) opened: bool,
    pub(crate) initial_id: InitialId,
    pub(crate) action: RwLock<Action>,
    pub(crate) params: Params,
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
