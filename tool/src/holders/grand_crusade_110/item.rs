use crate::data::ItemId;
use crate::entity::item::Item;
use crate::holders::grand_crusade_110::{CoordsXYZ, Loader110};
use crate::util::l2_reader::deserialize_dat;
use crate::util::{
    DebugUtils, ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal, ASCF, LONG, USHORT,
};
use crate::util::{L2StringTable, BYTE, DWORD, FLOAT, SHORT, UVEC};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::HashMap;

impl Loader110 {
    pub fn load_items(&mut self) -> Result<(), ()> {
        let weapon_grp = deserialize_dat::<WeaponGrpDat>(
            self.dat_paths
                .get(&"weapongrp.dat".to_string())
                .unwrap()
                .path(),
        )?;
        let _item_stat = deserialize_dat::<ItemStatDataDat>(
            self.dat_paths
                .get(&"itemstatdata.dat".to_string())
                .unwrap()
                .path(),
        )?;
        let _item_base_info = deserialize_dat::<ItemBaseInfoDat>(
            self.dat_paths
                .get(&"item_baseinfo.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let item_name = deserialize_dat::<ItemNameDat>(
            self.dat_paths
                .get(&"itemname-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        {
            let mut weapon_types = HashMap::new();
            for v in weapon_grp {
                if let std::collections::hash_map::Entry::Vacant(e) =
                    weapon_types.entry(v.weapon_type)
                {
                    e.insert(v.id);
                }
            }

            println!("weapon_types");
            weapon_types.print_ordered();
            println!("\n");
        }

        for v in item_name {
            let x = Item {
                id: ItemId(v.id),
                name: if let Some(name) = self.game_data_name.get(&v.name_link) {
                    name.clone()
                } else {
                    format!("NameNotFound[{}]", v.name_link)
                },
                desc: v.description.0,
            };

            self.items.insert(x.id, x);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct WeaponGrpDat {
    tag: BYTE,                 //+
    id: DWORD,                 //+
    drop_type: BYTE,           //+
    drop_animation_type: BYTE, //+
    drop_radius: BYTE,
    drop_height: BYTE,
    drop_info: UVEC<BYTE, DropDatInfo>,
    icon_1: DWORD,
    icon_2: DWORD,
    icon_3: DWORD,
    icon_4: DWORD,
    icon_5: DWORD,
    durability: SHORT,
    weight: SHORT,
    material_type: BYTE,
    crystallizable: BYTE,
    related_quests_ids: UVEC<BYTE, SHORT>,
    color: BYTE,
    is_attribution: BYTE,
    property_params: SHORT,
    icon_panel: DWORD,
    complete_item_drop_sound_type: DWORD,
    inventory_type: BYTE,
    body_part: BYTE,
    hand_stance_type: BYTE,
    mesh: UVEC<BYTE, MeshDatInfo>,
    texture: UVEC<BYTE, DWORD>,
    item_sound: UVEC<BYTE, DWORD>,
    drop_sound: DWORD,
    equip_sound: DWORD,
    effect: DWORD,
    random_damage_type: BYTE,
    weapon_type: BYTE,
    crystal_type: BYTE,
    mp_consume: BYTE,
    soulshot_count: BYTE,
    spiritshot_count: BYTE,
    curvature: SHORT,
    unk_1: BYTE,
    can_equip_hero: BYTE,
    is_magic_weapon: BYTE,
    ertheia_fist_scale: FLOAT,
    junk: SHORT,
    enchant_info: UVEC<BYTE, EnchantInfo>,
    variation_effect_1: BYTE,
    variation_effect_2: BYTE,
    variation_effect_3: BYTE,
    variation_effect_4: BYTE,
    variation_effect_5: BYTE,
    variation_effect_6: BYTE,
    variation_icon: UVEC<BYTE, DWORD>,
    ensoul_count: BYTE,
    is_ensoul: BYTE,
}
#[derive(Debug, Copy, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct EnchantInfo {
    effect: DWORD,
    effect_offset: CoordsXYZ,
    mesh_offset: CoordsXYZ,
    mesh_scale: CoordsXYZ,
    effect_velocity: FLOAT,
    particle_scale: FLOAT,
    effect_scale: FLOAT,
    particle_offset: CoordsXYZ,
    ring_offset: CoordsXYZ,
    ring_scale: CoordsXYZ,
}
#[derive(Debug, Copy, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct MeshDatInfo {
    mesh: DWORD,
    unk1: BYTE,
}
#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct DropDatInfo {
    mesh: DWORD,
    texture: UVEC<BYTE, DWORD>,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct ItemBaseInfoDat {
    id: DWORD,
    default_price: LONG,
    is_premium: DWORD,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct ItemStatDataDat {
    object_id: DWORD,
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

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct ItemNameDat {
    id: DWORD,
    name_link: DWORD,
    additional_name: ASCF,
    description: ASCF,
    popup: SHORT,
    default_action: ASCF,
    use_order: DWORD,
    set_id: SHORT,
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
