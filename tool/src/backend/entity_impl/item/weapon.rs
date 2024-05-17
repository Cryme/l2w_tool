use crate::backend::entity_catalog::EntityInfo;
use crate::backend::entity_editor::{
    CommonEditorOps, CurrentEntity, EditParams, EditParamsCommonOps, EntityEditParams, WindowParams,
};
use crate::backend::entity_impl::item::{ItemAdditionalInfoAction, ItemDropInfoAction};
use crate::backend::holder::{FHashMap, HolderMapOps};
use crate::backend::{Backend, HandleAction};
use crate::data::ItemId;
use crate::entity::item::weapon::Weapon;
use crate::entity::{CommonEntity, EntityT};
use serde::{Deserialize, Serialize};

pub type WeaponEditor = EntityEditParams<Weapon, ItemId, WeaponAction, ()>;

impl HandleAction for WindowParams<Weapon, ItemId, WeaponAction, ()> {
    fn handle_action(&mut self) {
        let weapon = self;

        let mut action = weapon.action.write().unwrap();

        match *action {
            WeaponAction::RemoveMesh(i) => {
                weapon.inner.mesh_info.remove(i);
            }

            WeaponAction::None => {}
        }

        *action = WeaponAction::None;

        {
            let mut action = weapon
                .inner
                .base_info
                .additional_info
                .action
                .write()
                .unwrap();
            match *action {
                ItemAdditionalInfoAction::RemoveItem(v) => {
                    weapon
                        .inner
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
            let mut action = weapon.inner.base_info.drop_info.action.write().unwrap();
            match *action {
                ItemDropInfoAction::RemoveMesh(v) => {
                    weapon
                        .inner
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

        {
            let mut action = weapon.inner.enchant_info.action.write().unwrap();
            match *action {
                WeaponEnchantAction::RemoveEnchant(v) => {
                    weapon.inner.enchant_info.inner.params.remove(v);
                }

                WeaponEnchantAction::None => {}
            }

            *action = WeaponEnchantAction::None;
        }

        {
            let mut action = weapon.inner.variation_info.action.write().unwrap();
            match *action {
                WeaponVariationAction::RemoveIcon(v) => {
                    weapon.inner.variation_info.inner.icon.remove(v);
                }

                WeaponVariationAction::None => {}
            }

            *action = WeaponVariationAction::None;
        }

        {
            let mut action = weapon.inner.sound.action.write().unwrap();
            match *action {
                WeaponSoundAction::RemoveSound(v) => {
                    weapon.inner.sound.inner.0.remove(v);
                }

                WeaponSoundAction::None => {}
            }

            *action = WeaponSoundAction::None;
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum WeaponAction {
    #[default]
    None,
    RemoveMesh(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum WeaponSoundAction {
    #[default]
    None,
    RemoveSound(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum WeaponEnchantAction {
    #[default]
    None,
    RemoveEnchant(usize),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum WeaponVariationAction {
    #[default]
    None,
    RemoveIcon(usize),
}

impl EditParams {
    pub fn get_opened_weapons_info(&self) -> Vec<(String, ItemId, bool)> {
        self.weapons.get_opened_info()
    }

    pub fn open_weapon(&mut self, id: ItemId, holder: &mut FHashMap<ItemId, Weapon>) {
        for (i, q) in self.weapons.opened.iter().enumerate() {
            if q.inner.initial_id == id {
                self.current_entity = CurrentEntity::Weapon(i);

                return;
            }
        }

        if let Some(q) = holder.get(&id) {
            self.current_entity = CurrentEntity::Weapon(self.weapons.add(q.clone(), q.id(), false));
        }
    }

    pub fn set_current_weapon(&mut self, index: usize) {
        if index < self.weapons.opened.len() {
            self.current_entity = CurrentEntity::Weapon(index);
        }
    }

    pub fn create_new_weapon(&mut self) {
        self.current_entity = CurrentEntity::Weapon(self.weapons.add_new());
    }
}

impl Backend {
    pub fn filter_weapons(&mut self) {
        self.entity_catalogs.weapon.filter(
            &self.holders.game_data_holder.weapon_holder,
            self.entity_catalogs.filter_mode,
        );
    }

    pub fn save_weapon_from_dlg(&mut self, id: ItemId) {
        if let CurrentEntity::Weapon(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.weapons.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id();

            let entity = new_entity.inner.inner.clone();

            new_entity.on_save();

            self.save_weapon_force(entity);
        }
    }

    pub(crate) fn save_weapon_force(&mut self, mut v: Weapon) {
        if let Some(vv) = self.holders.game_data_holder.weapon_holder.get(&v.id()) {
            if *vv == v {
                return;
            }
        }
        v._changed = true;

        if self
            .holders
            .game_data_holder
            .armor_holder
            .remove(&v.base_info.id)
            .is_some()
        {
            self.edit_params
                .close_if_opened(EntityT::Armor(v.base_info.id));
            self.filter_armor();
        }
        if self
            .holders
            .game_data_holder
            .etc_item_holder
            .remove(&v.base_info.id)
            .is_some()
        {
            self.edit_params
                .close_if_opened(EntityT::EtcItem(v.base_info.id));
            self.filter_etc_items();
        }

        self.holders
            .game_data_holder
            .item_holder
            .insert(v.base_info.id, (&v).into());
        self.holders
            .game_data_holder
            .weapon_holder
            .insert(v.base_info.id, v);

        self.filter_weapons();
        self.check_for_unwrote_changed();
    }
}

impl From<&Weapon> for EntityInfo<Weapon, ItemId> {
    fn from(value: &Weapon) -> EntityInfo<Weapon, ItemId> {
        EntityInfo::new(
            &format!(
                "ID: {}\n{} {}",
                value.base_info.id.0, value.base_info.name, value.base_info.additional_name
            ),
            value,
        )
    }
}
