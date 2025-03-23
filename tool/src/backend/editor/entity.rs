use super::*;
use crate::backend::HandleAction;
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::entity::{CommonEntity, GameEntity, GameEntityT, GetEditParams};
use ron::error::SpannedError;
use ron::ser::PrettyConfig;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hash;
use std::sync::RwLock;

#[derive(Serialize, Deserialize)]
pub struct ChangeTrackedParams<
    Entity: Serialize,
    EntityId: Default + Serialize + DeserializeOwned,
    EditAction: Default + Serialize + DeserializeOwned,
    EditParams: Default + Serialize + DeserializeOwned,
> {
    #[serde(serialize_with = "params_serializer")]
    #[serde(deserialize_with = "params_deserializer")]
    pub inner: WindowParams<Entity, EntityId, EditAction, EditParams>,
    pub initial: Entity,
    changed: bool,
    pub is_new: bool,
}

impl<
    Entity: PartialEq + Clone + Serialize + DeserializeOwned,
    EntityId: From<u32> + Default + Serialize + DeserializeOwned,
    EditAction: Default + Serialize + DeserializeOwned,
    EditParams: Default + Serialize + DeserializeOwned,
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

fn params_serializer<S, Entity, EntityId, EditAction, EditParams>(
    x: &crate::backend::editor::WindowParams<Entity, EntityId, EditAction, EditParams>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    Entity: Serialize,
    EntityId: Serialize,
    EditAction: Serialize,
    EditParams: Serialize,
{
    x.serialize_full(s)
}
fn params_deserializer<'de, D, Entity, EntityId, EditAction, EditParams>(
    d: D,
) -> Result<crate::backend::editor::WindowParams<Entity, EntityId, EditAction, EditParams>, D::Error>
where
    D: Deserializer<'de>,
    Entity: Deserialize<'de>,
    EntityId: Deserialize<'de>,
    EditAction: Deserialize<'de>,
    EditParams: Deserialize<'de>,
{
    WindowParams::deserialize_full(d)
}

#[derive(Serialize)]
pub struct EntityEditParams<
    Entity: Serialize,
    EntityId: Default + Serialize + DeserializeOwned,
    EditAction: Default + Serialize + DeserializeOwned,
    EditParams: Default + Serialize + DeserializeOwned,
> {
    #[serde(skip)]
    pub(crate) next_id: u32,
    pub opened: Vec<ChangeTrackedParams<Entity, EntityId, EditAction, EditParams>>,
}

impl<
    'de,
    Entity: Serialize + DeserializeOwned,
    EntityId: Default + Serialize + DeserializeOwned,
    EditAction: Default + Serialize + DeserializeOwned,
    EditParams: Default + Serialize + DeserializeOwned,
> Deserialize<'de> for EntityEditParams<Entity, EntityId, EditAction, EditParams>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let c = Vec::deserialize(deserializer)?;

        Ok(Self {
            next_id: 0,
            opened: c,
        })
    }
}

impl<
    Entity: Serialize,
    EntityId: Default + Serialize + DeserializeOwned,
    EditAction: Default + Serialize + DeserializeOwned,
    EditParams: Default + Serialize + DeserializeOwned,
> Default for EntityEditParams<Entity, EntityId, EditAction, EditParams>
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
    Entity: CommonEntity<EntityId> + GetEditParams<EditParams> + Clone + Serialize,
    EntityId: From<u32> + Copy + Clone + Hash + Eq + Default + Serialize + DeserializeOwned,
    EditAction: Default + Serialize + DeserializeOwned,
    EditParams: Default + Serialize + DeserializeOwned,
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

impl<
    Entity: CommonEntity<EntityId> + Clone + Serialize,
    EntityId: Hash + Eq + Copy + Clone + Default + Serialize + DeserializeOwned,
    Action: Default + Serialize + DeserializeOwned,
    Params: Default + Serialize + DeserializeOwned,
> AgnosticEditorOps for EntityEditParams<Entity, EntityId, Action, Params>
{
    fn next_id(&self) -> u32 {
        self.next_id
    }
}

impl Editors {
    pub fn close_if_opened(&mut self, entity: GameEntityT) {
        match entity {
            GameEntityT::Quest(id) => {
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
            GameEntityT::Skill(id) => {
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
            GameEntityT::Npc(id) => {
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
            GameEntityT::Weapon(id) => {
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
            GameEntityT::Armor(id) => {
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
            GameEntityT::EtcItem(id) => {
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
            GameEntityT::ItemSet(id) => {
                if let Some((i, _)) = self
                    .item_sets
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.item_sets.opened.remove(i);
                }
            }
            GameEntityT::Recipe(id) => {
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
            GameEntityT::HuntingZone(id) => {
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
            GameEntityT::Region(id) => {
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
            GameEntityT::RaidInfo(id) => {
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
            GameEntityT::DailyMission(id) => {
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
            GameEntityT::AnimationCombo(id) => {
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
            GameEntityT::Residence(id) => {
                if let Some((i, _)) = self
                    .residences
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.residences.opened.remove(i);
                }
            }
            GameEntityT::EnsoulOption(id) => {
                if let Some((i, _)) = self
                    .ensoul_options
                    .opened
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.inner.initial_id == id)
                {
                    self.ensoul_options.opened.remove(i);
                }
            }
        }

        self.find_opened_entity();
    }
    pub fn reset_initial(&mut self, entity: GameEntity, holders: &GameDataHolder) {
        match entity {
            GameEntity::Quest => self.quests.reset_initial(&holders.quest_holder),
            GameEntity::Skill => self.skills.reset_initial(&holders.skill_holder),
            GameEntity::Npc => self.npcs.reset_initial(&holders.npc_holder),
            GameEntity::Weapon => self.weapons.reset_initial(&holders.weapon_holder),
            GameEntity::Armor => self.armor.reset_initial(&holders.armor_holder),
            GameEntity::EtcItem => self.etc_items.reset_initial(&holders.etc_item_holder),
            GameEntity::ItemSet => self.item_sets.reset_initial(&holders.item_set_holder),
            GameEntity::Recipe => self.recipes.reset_initial(&holders.recipe_holder),
            GameEntity::HuntingZone => self
                .hunting_zones
                .reset_initial(&holders.hunting_zone_holder),
            GameEntity::Region => self.regions.reset_initial(&holders.region_holder),
            GameEntity::RaidInfo => self.raid_info.reset_initial(&holders.raid_info_holder),
            GameEntity::DailyMission => self
                .daily_mission
                .reset_initial(&holders.daily_mission_holder),
            GameEntity::AnimationCombo => self
                .animation_combo
                .reset_initial(&holders.animation_combo_holder),
            GameEntity::Residence => self.residences.reset_initial(&holders.residence_holder),
            GameEntity::EnsoulOption => self
                .ensoul_options
                .reset_initial(&holders.ensoul_option_holder),
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
            CurrentEntity::EnsoulOption(i) => {
                if !self.ensoul_options.opened.is_empty() {
                    self.current_entity =
                        CurrentEntity::EnsoulOption(i.min(self.ensoul_options.opened.len() - 1));

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
        } else if !self.ensoul_options.is_empty() {
            self.current_entity = CurrentEntity::EnsoulOption(self.ensoul_options.len() - 1);
        } else {
            self.current_entity = CurrentEntity::None;
        }
    }
}
