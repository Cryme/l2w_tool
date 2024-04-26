mod armor;
mod etc_item;
mod weapon;

use crate::holders::grand_crusade_110::Loader110;
use crate::util::l2_reader::{deserialize_dat, save_dat, DatVariant};
use crate::util::{
    wrap_into_id_map, GetId, ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal, ASCF, LONG,
    USHORT, UVEC,
};
use crate::util::{BYTE, DWORD, FLOAT, SHORT};
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl Loader110 {
    pub fn serialize_items_to_binary(&mut self) -> JoinHandle<()> {
        let mut additional_item_grp = vec![];
        let mut item_stat = vec![];
        let mut item_base_info = vec![];
        let mut item_name = vec![];

        let weapon_handle = if self.weapons.was_changed {
            Some(self.serialize_weapons_to_binary())
        } else {
            println!("Weapons are unchanged");
            None
        };

        let etc_item_handle = if self.etc_items.was_changed {
            Some(self.serialize_etc_items_to_binary())
        } else {
            println!("Etc Items are unchanged");
            None
        };

        let armor_handle = if self.armor.was_changed {
            Some(self.serialize_armor_to_binary())
        } else {
            println!("Armor are unchanged");
            None
        };

        self.fill_items_from_weapons(
            &mut additional_item_grp,
            &mut item_stat,
            &mut item_base_info,
            &mut item_name,
        );

        self.fill_items_from_armor(
            &mut additional_item_grp,
            &mut item_stat,
            &mut item_base_info,
            &mut item_name,
        );

        self.fill_items_from_etc_items(
            &mut additional_item_grp,
            &mut item_stat,
            &mut item_base_info,
            &mut item_name,
        );

        let additional_item_grp_path = self
            .dat_paths
            .get(&"additionalitemgrp.dat".to_string())
            .unwrap()
            .clone();

        let item_stat_path = self
            .dat_paths
            .get(&"itemstatdata.dat".to_string())
            .unwrap()
            .clone();

        let item_base_info_path = self
            .dat_paths
            .get(&"item_baseinfo.dat".to_string())
            .unwrap()
            .clone();

        let item_name_path = self
            .dat_paths
            .get(&"itemname-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            let additional_item_grp_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    additional_item_grp_path.path(),
                    DatVariant::<(), AdditionalItemGrpDat>::Array(additional_item_grp),
                ) {
                    println!("{e:?}");
                } else {
                    println!("Additional Item Grp saved");
                }
            });

            let item_stat_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    item_stat_path.path(),
                    DatVariant::<(), ItemStatDataDat>::Array(item_stat),
                ) {
                    println!("{e:?}");
                } else {
                    println!("Item Stat saved");
                }
            });

            let item_base_info_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    item_base_info_path.path(),
                    DatVariant::<(), ItemBaseInfoDat>::Array(item_base_info),
                ) {
                    println!("{e:?}");
                } else {
                    println!("Item Base Info saved");
                }
            });

            let item_name_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    item_name_path.path(),
                    DatVariant::<(), ItemNameDat>::Array(item_name),
                ) {
                    println!("{e:?}");
                } else {
                    println!("Item Name saved");
                }
            });

            let _ = additional_item_grp_handle.join();
            let _ = item_stat_handle.join();
            let _ = item_base_info_handle.join();
            let _ = item_name_handle.join();

            if let Some(h) = weapon_handle {
                let _ = h.join();
            }

            if let Some(h) = armor_handle {
                let _ = h.join();
            }

            if let Some(h) = etc_item_handle {
                let _ = h.join();
            }
        })
    }

    pub fn load_items(&mut self) -> Result<(), ()> {
        let additional_item_grp = wrap_into_id_map(deserialize_dat::<AdditionalItemGrpDat>(
            self.dat_paths
                .get(&"additionalitemgrp.dat".to_string())
                .unwrap()
                .path(),
        )?);

        let item_stat = wrap_into_id_map(deserialize_dat::<ItemStatDataDat>(
            self.dat_paths
                .get(&"itemstatdata.dat".to_string())
                .unwrap()
                .path(),
        )?);

        let item_base_info = wrap_into_id_map(deserialize_dat::<ItemBaseInfoDat>(
            self.dat_paths
                .get(&"item_baseinfo.dat".to_string())
                .unwrap()
                .path(),
        )?);

        let item_name = wrap_into_id_map(deserialize_dat::<ItemNameDat>(
            self.dat_paths
                .get(&"itemname-ru.dat".to_string())
                .unwrap()
                .path(),
        )?);

        self.load_weapons(
            &additional_item_grp,
            &item_stat,
            &item_base_info,
            &item_name,
        )?;

        self.load_etc_items(
            &additional_item_grp,
            &item_stat,
            &item_base_info,
            &item_name,
        )?;

        self.load_armor(
            &additional_item_grp,
            &item_stat,
            &item_base_info,
            &item_name,
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub(crate) struct ItemBaseInfoDat {
    id: DWORD,
    default_price: LONG,
    is_premium: DWORD,
}
impl GetId for ItemBaseInfoDat {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
pub(crate) struct ItemStatDataDat {
    id: DWORD,
    p_defense: USHORT,
    m_defense: USHORT,
    p_attack: USHORT,
    m_attack: USHORT,
    p_attack_speed: USHORT,
    p_hit: FLOAT,
    m_hit: FLOAT,
    p_critical: FLOAT,
    m_critical: FLOAT,
    speed: BYTE,
    shield_defense: USHORT,
    shield_defense_rate: BYTE,
    p_avoid: FLOAT,
    m_avoid: FLOAT,
    property_params: USHORT,
}
impl GetId for ItemStatDataDat {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
pub(crate) struct ItemNameDat {
    id: DWORD,
    name_link: DWORD,
    additional_name: ASCF,
    description: ASCF,
    popup: SHORT,
    default_action: ASCF,
    use_order: DWORD,
    set_id: USHORT,
    color: BYTE,
    tooltip_texture_link: DWORD,
    is_trade: BYTE,
    is_drop: BYTE,
    is_destruct: BYTE,
    is_private_store: BYTE,
    keep_type: BYTE,
    is_npc_trade: BYTE,
    is_commission_store: BYTE,
}

impl GetId for ItemNameDat {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
pub(crate) struct AdditionalItemGrpDat {
    id: DWORD,
    has_ani: BYTE,
    included_items: Vec<DWORD>,
    max_energy: DWORD,
    look_change: DWORD,
    hide_cloak: BYTE,
    unk1: BYTE,
    hide_armor: BYTE,
}

impl GetId for AdditionalItemGrpDat {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct DropDatInfo {
    mesh: DWORD,
    texture: UVEC<BYTE, DWORD>,
}
