mod armor;
mod etc_item;
mod weapon;

use crate::backend::dat_loader::grand_crusade_110::Loader110;
use crate::backend::log_holder::Log;

use l2_rw::ue2_rw::{ASCF, BYTE, DWORD, FLOAT, LONG, SHORT, USHORT, UVEC};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::dat_loader::{wrap_into_id_map, GetId};
use crate::entity::item::ItemDefaultAction;
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl Loader110 {
    pub fn serialize_items_to_binary(&mut self) -> JoinHandle<Vec<Log>> {
        let mut logs = vec![];

        let mut additional_item_grp = vec![];
        let mut item_stat = vec![];
        let mut item_base_info = vec![];
        let mut item_name = vec![];

        let weapon_handle = if self.weapons.was_changed() {
            Some(self.serialize_weapons_to_binary())
        } else {
            None
        };

        let etc_item_handle = if self.etc_items.was_changed() {
            Some(self.serialize_etc_items_to_binary())
        } else {
            None
        };

        let armor_handle = if self.armor.was_changed() {
            Some(self.serialize_armor_to_binary())
        } else {
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
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("Additional Item Grp saved")
                }
            });

            let item_stat_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    item_stat_path.path(),
                    DatVariant::<(), ItemStatDataDat>::Array(item_stat),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("Item Stat saved")
                }
            });

            let item_base_info_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    item_base_info_path.path(),
                    DatVariant::<(), ItemBaseInfoDat>::Array(item_base_info),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("Item Base Info saved")
                }
            });

            let item_name_handle = thread::spawn(move || {
                if let Err(e) = save_dat(
                    item_name_path.path(),
                    DatVariant::<(), ItemNameDat>::Array(item_name),
                ) {
                    Log::from_loader_e(e)
                } else {
                    Log::from_loader_i("Item Name saved")
                }
            });

            logs.push(additional_item_grp_handle.join().unwrap());
            logs.push(item_stat_handle.join().unwrap());
            logs.push(item_base_info_handle.join().unwrap());
            logs.push(item_name_handle.join().unwrap());

            if let Some(h) = weapon_handle {
                logs.push(h.join().unwrap());
            }

            if let Some(h) = armor_handle {
                logs.push(h.join().unwrap());
            }

            if let Some(h) = etc_item_handle {
                logs.push(h.join().unwrap());
            }

            logs
        })
    }

    pub fn load_items(&mut self) -> Result<Vec<Log>, ()> {
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

        let mut logs = self.load_weapons(
            &additional_item_grp,
            &item_stat,
            &item_base_info,
            &item_name,
        )?;

        logs.extend(self.load_etc_items(
            &additional_item_grp,
            &item_stat,
            &item_base_info,
            &item_name,
        )?);

        logs.extend(self.load_armor(
            &additional_item_grp,
            &item_stat,
            &item_base_info,
            &item_name,
        )?);

        Ok(logs)
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

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
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

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
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

impl ItemDefaultAction {
    pub fn from_ascf(value: &ASCF) -> Self {
        match &**value.inner() {
            "action_bless_spiritshot\0" => Self::action_bless_spiritshot,
            "action_calc\0" => Self::action_calc,
            "action_call_skill\0" => Self::action_call_skill,
            "action_capsule\0" => Self::action_capsule,
            "action_create_mpcc\0" => Self::action_create_mpcc,
            "action_dice\0" => Self::action_dice,
            "action_equip\0" => Self::action_equip,
            "action_fishingshot\0" => Self::action_fishingshot,
            "action_harvest\0" => Self::action_harvest,
            "action_hide_name\0" => Self::action_hide_name,
            "action_keep_exp\0" => Self::action_keep_exp,
            "action_nick_color\0" => Self::action_nick_color,
            "action_none\0" => Self::action_none,
            "action_peel\0" => Self::action_peel,
            "action_recipe\0" => Self::action_recipe,
            "action_seed\0" => Self::action_seed,
            "action_show_adventurer_guide_book\0" => Self::action_show_adventurer_guide_book,
            "action_show_html\0" => Self::action_show_html,
            "action_show_ssq_status\0" => Self::action_show_ssq_status,
            "action_show_tutorial\0" => Self::action_show_tutorial,
            "action_skill_maintain\0" => Self::action_skill_maintain,
            "action_skill_reduce\0" => Self::action_skill_reduce,
            "action_skill_reduce_on_skill_success\0" => Self::action_skill_reduce_on_skill_success,
            "action_soulshot\0" => Self::action_soulshot,
            "action_spiritshot\0" => Self::action_spiritshot,
            "action_start_quest\0" => Self::action_start_quest,
            "action_summon_soulshot\0" => Self::action_summon_soulshot,
            "action_summon_spiritshot\0" => Self::action_summon_spiritshot,
            "action_xmas_open\0" => Self::action_xmas_open,

            _ => unreachable!(),
        }
    }
}
