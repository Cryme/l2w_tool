use crate::backend::holder::FHashMap;
use crate::backend::item::{ItemAdditionalInfoAction, ItemDropInfoAction};
use crate::backend::{
    Backend, CommonEditorOps, CurrentEntity, EditParams, EntityEditParams, HandleAction,
    WindowParams,
};
use crate::data::ItemId;
use crate::entity::item::weapon::Weapon;
use crate::entity::CommonEntity;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub enum WeaponAction {
    #[default]
    None,
    RemoveMesh(usize),
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
            self.current_entity = CurrentEntity::Weapon(self.weapons.add(q.clone(), q.id()));
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
        let s = self.filter_params.weapon_filter_string.to_lowercase();

        let fun: Box<dyn Fn(&&Weapon) -> bool> = if s.is_empty() {
            Box::new(|_: &&Weapon| true)
        } else if let Ok(id) = u32::from_str(&s) {
            Box::new(move |v: &&Weapon| v.base_info.id == ItemId(id))
        } else if s.starts_with("texture:"){
            let c = s.replace("texture:", "");
            Box::new(move |v: &&Weapon| {
                v.mesh_info.iter().any(|v| v.texture.to_lowercase().contains(&c))
            })
        } else if s.starts_with("mesh:"){
            let c = s.replace("mesh:", "");
            Box::new(move |v: &&Weapon| {
                v.mesh_info.iter().any(|v| v.mesh.to_lowercase().contains(&c))
            })
        } else {
            Box::new(move |v: &&Weapon| {
                v.base_info.name.to_lowercase().contains(&s)
                    || v.base_info.additional_name.to_lowercase().contains(&s)
            })
        };

        self.filter_params.weapon_catalog = self
            .holders
            .game_data_holder
            .weapon_holder
            .values()
            .filter(fun)
            .map(WeaponInfo::from)
            .collect();

        self.filter_params
            .weapon_catalog
            .sort_by(|a, b| a.id.cmp(&b.id))
    }

    pub fn save_weapon_from_dlg(&mut self, id: ItemId) {
        if let CurrentEntity::Weapon(index) = self.edit_params.current_entity {
            let new_entity = self.edit_params.weapons.opened.get_mut(index).unwrap();

            if new_entity.inner.inner.id() != id {
                return;
            }

            new_entity.inner.initial_id = new_entity.inner.inner.id();

            let entity = new_entity.inner.inner.clone();

            self.save_weapon_force(entity);
        }
    }

    pub(crate) fn save_weapon_force(&mut self, v: Weapon) {
        if let Some(vv) = self.holders.game_data_holder.weapon_holder.get(&v.id()) {
            if *vv == v {
                return;
            }
        }
        self.set_changed();

        self.holders
            .game_data_holder
            .item_holder
            .insert(v.base_info.id, (&v).into());

        self.holders
            .game_data_holder
            .weapon_holder
            .insert(v.base_info.id, v);

        self.filter_weapons();
    }
}

pub struct WeaponInfo {
    pub(crate) id: ItemId,
    pub(crate) name: String,
}

impl From<&Weapon> for WeaponInfo {
    fn from(value: &Weapon) -> Self {
        WeaponInfo {
            id: value.base_info.id,
            name: format!(
                "{} {}",
                value.base_info.name, value.base_info.additional_name
            ),
        }
    }
}
