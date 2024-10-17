use crate::backend::editor::entity::{CommonEditorOps, EntityEditParams};
use crate::backend::editor::{CurrentEntity, EditParamsCommonOps, Editors, WindowParams};
use crate::backend::entity_catalog::EntityInfo;
use crate::backend::entity_impl::item::{ItemAdditionalInfoAction, ItemDropInfoAction};
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::common::ItemId;
use crate::entity::item::armor::Armor;
use crate::entity::item::etc_item::EtcItem;
use crate::entity::item::weapon::Weapon;
use crate::entity::{CommonEntity, GameEntityT};
use serde::{Deserialize, Serialize};

pub type ArmorEditor = EntityEditParams<Armor, ItemId, ArmorAction, ()>;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum ArmorAction {
    #[default]
    None,
    RemoveSound(usize),
}

impl HandleAction for WindowParams<Armor, ItemId, ArmorAction, ()> {
    fn handle_action(&mut self) {
        let item = self;

        let mut action = item.action.write().unwrap();

        match *action {
            ArmorAction::RemoveSound(i) => {
                item.inner.item_sound.remove(i);
            }

            ArmorAction::None => {}
        }

        *action = ArmorAction::None;

        {
            let mut action = item.inner.base_info.additional_info.action.write().unwrap();
            match *action {
                ItemAdditionalInfoAction::RemoveItem(v) => {
                    item.inner
                        .base_info
                        .additional_info
                        .inner
                        .include_items
                        .remove(v);
                }

                ItemAdditionalInfoAction::None => {}
            }

            *action = ItemAdditionalInfoAction::None;
        }

        {
            let mut action = item.inner.base_info.drop_info.action.write().unwrap();
            match *action {
                ItemDropInfoAction::RemoveMesh(v) => {
                    item.inner
                        .base_info
                        .drop_info
                        .inner
                        .drop_mesh_info
                        .remove(v);
                }

                ItemDropInfoAction::None => {}
            }

            *action = ItemDropInfoAction::None;
        }
    }
}

impl Editors {
    pub fn force_update_armor(&mut self, item: &Armor) {
        for v in &mut self.armor.opened {
            if v.inner.inner.id() == item.id() {
                v.inner.inner = item.clone();
            }
        }
    }
    pub fn force_update_weapon(&mut self, item: &Weapon) {
        for v in &mut self.weapons.opened {
            if v.inner.inner.id() == item.id() {
                v.inner.inner = item.clone();
            }
        }
    }
    pub fn force_update_etc_item(&mut self, item: &EtcItem) {
        for v in &mut self.etc_items.opened {
            if v.inner.inner.id() == item.id() {
                v.inner.inner = item.clone();
            }
        }
    }

    pub fn get_opened_armor_info(&self) -> Vec<(String, ItemId, bool)> {
        self.armor.get_opened_info()
    }

    pub fn open_armor(&mut self, id: ItemId, holder: &mut FHashMap<ItemId, Armor>) {
        for (i, q) in self.armor.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::Armor(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity = CurrentEntity::Armor(self.armor.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_armor(&mut self, index: usize) {
        if index < self.armor.opened.len() {
            self.current_entity = CurrentEntity::Armor(index);
        }
    }

    pub fn create_new_armor(&mut self) {
        self.current_entity = CurrentEntity::Armor(self.armor.add_new());
    }
}

impl Backend {
    pub fn filter_armor(&mut self) {
        self.entity_catalogs.armor.filter(
            &self.holders.game_data_holder.armor_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_armor_from_dlg(&mut self, id: ItemId) {
        if let CurrentEntity::Armor(index) = self.editors.current_entity {
            let new_entity = self.editors.armor.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id();

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_armor_force(entity);
        }
    }

    pub(crate) fn save_armor_force(&mut self, mut v: Armor) {
        if let Some(vv) = self.holders.game_data_holder.armor_holder.get(&v.id()) {
            if *vv == v {
                return;
            }
        }
        v._changed = true;

        if self
            .holders
            .game_data_holder
            .weapon_holder
            .remove(&v.base_info.id)
            .is_some()
        {
            self.editors
                .close_if_opened(GameEntityT::Weapon(v.base_info.id));
            self.filter_weapons();
        }

        if self
            .holders
            .game_data_holder
            .etc_item_holder
            .remove(&v.base_info.id)
            .is_some()
        {
            self.editors
                .close_if_opened(GameEntityT::EtcItem(v.base_info.id));
            self.filter_etc_items();
        }

        self.holders
            .game_data_holder
            .item_holder
            .insert(v.base_info.id, (&v).into());
        self.holders
            .game_data_holder
            .armor_holder
            .insert(v.base_info.id, v);

        self.check_for_unwrote_changed();
        self.filter_armor();
    }
}

impl From<&Armor> for EntityInfo<Armor, ItemId> {
    fn from(value: &Armor) -> Self {
        EntityInfo::new(
            &format!(
                "ID: {}\n{} {}",
                value.base_info.id.0, value.base_info.name, value.base_info.additional_name
            ),
            value,
        )
    }
}
