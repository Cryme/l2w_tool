use crate::data::ItemId;
use crate::entity::item::weapon::{CharacterAnimationType, RandomDamage, Weapon, WeaponEnchantInfo, WeaponMeshInfo, WeaponMpConsume, WeaponType, WeaponVariationInfo};
use crate::entity::item::{BodyPart, InventoryType, Item, ItemBaseInfo, ItemDefaultAction, ItemMaterial, ItemNameColor, KeepType, ItemQuality, CrystalType, ItemIcons, ItemAdditionalInfo, ItemBattleStats, ItemDropInfo, DropType, DropAnimationType, ItemDropMeshInfo};
use crate::holders::grand_crusade_110::{CoordsXYZ, Loader110};
use crate::util::l2_reader::deserialize_dat;
use crate::util::{
    wrap_into_id_map, DebugUtils, GetId, ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal, ASCF,
    LONG, USHORT,
};
use crate::util::{L2StringTable, BYTE, DWORD, FLOAT, SHORT, UVEC};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::HashMap;
use num_traits::FromPrimitive;

impl Loader110 {
    pub fn load_items(&mut self) -> Result<(), ()> {
        let weapon_grp = deserialize_dat::<WeaponGrpDat>(
            self.dat_paths
                .get(&"weapongrp.dat".to_string())
                .unwrap()
                .path(),
        )?;

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

        {
            let mut types = HashMap::new();
            for v in &weapon_grp {
                if let std::collections::hash_map::Entry::Vacant(e) = types.entry(v.hand_stance_type) {
                    e.insert(v.id);
                }
            }

            println!("\nTypes:");
            types.print_ordered();
            println!("\n");
        }

        for v in item_name.values() {
            let x = Item {
                id: ItemId(v.id),
                name: if let Some(name) = self.game_data_name.get(&v.name_link) {
                    name.clone()
                } else {
                    format!("NameNotFound[{}]", v.name_link)
                },
                desc: v.description.0.clone(),
            };

            self.items.insert(x.id, x);
        }

        for weapon in weapon_grp {
            let name_grp = item_name.get(&weapon.id).unwrap();
            let base_info_grp = item_base_info.get(&weapon.id).unwrap();
            let add_info_grp = additional_item_grp.get(&weapon.id).unwrap();
            let stats = item_stat.get(&weapon.id).unwrap();

            let mut mesh_info = vec![];

            for (i, v) in weapon.mesh.inner.iter().enumerate() {
                let texture = weapon.texture.inner.get(i).unwrap();
                mesh_info.push(WeaponMeshInfo {
                    mesh: self.game_data_name.get_o(&v.mesh),
                    unk: v.unk1,
                    texture: self.game_data_name.get_o(texture),
                });
            }

            let mut drop_mesh_info = vec![];
            for v in &weapon.drop_info {
                drop_mesh_info.push(ItemDropMeshInfo {
                    mesh: self.game_data_name.get_o(&v.mesh),
                    textures: v.texture.inner.iter().map(|vv| self.game_data_name.get_o(&vv)).collect(),
                })
            }

            let drop_info = ItemDropInfo {
                drop_type: DropType::from_u8(weapon.drop_type).unwrap(),
                drop_animation_type: DropAnimationType::from_u8(weapon.drop_animation_type).unwrap(),
                drop_radius: weapon.drop_radius,
                drop_height: weapon.drop_height,
                drop_mesh_info,
                complete_item_drop_sound: self.game_data_name.get_o(&weapon.complete_item_drop_sound),
                drop_sound: self.game_data_name.get_o(&weapon.drop_sound),
            };

            self.weapons.insert(
                weapon.id.into(),
                Weapon {
                    base_info: ItemBaseInfo {
                        id: weapon.id.into(),
                        name: self.game_data_name.get_o(&name_grp.name_link),
                        additional_name: name_grp.additional_name.to_string(),
                        popup: name_grp.popup,
                        default_action: ItemDefaultAction::from_ascf(&name_grp.default_action),
                        use_order: name_grp.use_order,
                        set_id: name_grp.set_id.into(),
                        color: ItemNameColor::from_u8(name_grp.color).unwrap(),
                        tooltip_texture: self.game_data_name.get_o(&name_grp.tooltip_texture_link),
                        is_trade: name_grp.is_trade == 1,
                        is_drop: name_grp.is_drop == 1,
                        is_destruct: name_grp.is_destruct == 1,
                        is_private_store: name_grp.is_private_store == 1,
                        is_npc_trade: name_grp.is_npc_trade == 1,
                        is_commission_store: name_grp.is_commission_store == 1,
                        keep_type: KeepType::from_u8(name_grp.keep_type).unwrap(),
                        desc: name_grp.description.to_string(),
                        inventory_type: InventoryType::from_u8(weapon.inventory_type).unwrap(),
                        material: ItemMaterial::from_u8(weapon.material_type).unwrap(),
                        body_part: BodyPart::from_u8(weapon.body_part).unwrap(),
                        quality: ItemQuality::from_u8(weapon.color).unwrap(),
                        crystallizable: weapon.crystallizable == 1,
                        crystal_type: CrystalType::from_u8(weapon.crystal_type).unwrap(),
                        durability: weapon.durability as u16,
                        weight: weapon.weight as u16,
                        icons: ItemIcons {
                            icon_1: self.game_data_name.get_o(&weapon.icon_1),
                            icon_2: self.game_data_name.get_o(&weapon.icon_2),
                            icon_3: self.game_data_name.get_o(&weapon.icon_3),
                            icon_4: self.game_data_name.get_o(&weapon.icon_4),
                            icon_5: self.game_data_name.get_o(&weapon.icon_5),
                            icon_panel: self.game_data_name.get_o(&weapon.icon_panel),
                        },
                        default_price: base_info_grp.default_price,
                        is_premium: base_info_grp.is_premium == 1,
                        is_blessed: weapon.is_blessed == 1,
                        property_params: weapon.property_params,
                        related_quests: weapon.related_quests_ids.inner.iter().map(|v| (*v).into()).collect(),
                        equip_sound: self.game_data_name.get_o(&weapon.equip_sound),
                        additional_info: ItemAdditionalInfo {
                            has_animation: add_info_grp.has_ani == 1,
                            include_items: add_info_grp.included_items.iter().map(|v| (*v).into()).collect(),
                            max_energy: add_info_grp.max_energy,
                            look_change: self.game_data_name.get_o(&add_info_grp.look_change),
                            hide_cloak: add_info_grp.hide_cloak == 1,
                            unk: add_info_grp.unk1 == 1,
                            hide_armor: add_info_grp.hide_armor == 1,
                        },
                        drop_info,
                    },
                    weapon_type: WeaponType::from_u8(weapon.weapon_type).unwrap(),
                    character_animation_type: CharacterAnimationType::from_u8(weapon.hand_stance_type).unwrap(),
                    battle_stats: ItemBattleStats {
                        p_defense: stats.p_defense,
                        m_defense: stats.m_defense,
                        p_attack: stats.p_attack,
                        m_attack: stats.m_attack,
                        p_attack_speed: stats.p_attack_speed,
                        p_hit:  stats.p_hit,
                        m_hit:  stats.m_hit,
                        p_critical:  stats.p_critical,
                        m_critical:  stats.m_critical,
                        speed: stats.speed,
                        shield_defense: stats.shield_defense,
                        shield_defense_rate: stats.shield_defense_rate,
                        p_avoid:  stats.p_avoid,
                        m_avoid:  stats.m_avoid,
                        property_params: stats.property_params,
                    },
                    random_damage: RandomDamage::from_u8(weapon.random_damage_type).unwrap(),
                    ertheia_fists_scale: weapon.ertheia_fist_scale,
                    mesh_info,
                    sound: weapon.item_sound.inner.iter().map(|v| self.game_data_name.get_o(v)).collect(),
                    effect: self.game_data_name.get_o(&weapon.effect),
                    mp_consume: WeaponMpConsume::from_u8(weapon.mp_consume).unwrap(),
                    soulshot_count: weapon.soulshot_count,
                    spiritshot_count: weapon.spiritshot_count,
                    curvature: weapon.curvature,
                    unk: weapon.unk_1 == 1,
                    can_equip_hero: weapon.can_equip_hero == 1,
                    is_magic_weapon: weapon.is_magic_weapon == 1,
                    enchant_junk: weapon.junk,
                    enchant_info: weapon.enchant_info.inner.iter().map(|v| {
                        WeaponEnchantInfo {
                            effect: self.game_data_name.get_o(&v.effect),
                            effect_offset: v.effect_offset.into(),
                            effect_scale: v.effect_scale,
                            effect_velocity: v.effect_velocity,
                            mesh_offset: v.mesh_offset.into(),
                            mesh_scale: v.mesh_scale.into(),
                            particle_offset: v.particle_offset.into(),
                            particle_scale: v.particle_scale,
                            ring_offset: v.ring_offset.into(),
                            ring_scale: v.ring_scale.into(),
                        }
                    }).collect(),
                    variation_info: WeaponVariationInfo {
                        icon: weapon.variation_icon.inner.iter().map(|v| self.game_data_name.get_o(v)).collect(),
                        effect_1: weapon.variation_effect_1,
                        effect_2: weapon.variation_effect_2,
                        effect_3: weapon.variation_effect_3,
                        effect_4: weapon.variation_effect_4,
                        effect_5: weapon.variation_effect_5,
                        effect_6: weapon.variation_effect_6,
                    },
                    can_ensoul: weapon.is_ensoul == 1,
                    ensoul_count: weapon.ensoul_count,
                },
            )
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct WeaponGrpDat {
    tag: BYTE,
    id: DWORD,                             //+

    drop_type: BYTE,                       //+
    drop_animation_type: BYTE,             //+
    drop_radius: BYTE,                     //+
    drop_height: BYTE,                     //+
    drop_info: UVEC<BYTE, DropDatInfo>,    //+

    icon_1: DWORD,                         //+
    icon_2: DWORD,                         //+
    icon_3: DWORD,                         //+
    icon_4: DWORD,                         //+
    icon_5: DWORD,                         //+
    durability: SHORT,                     //+
    weight: SHORT,                         //+
    material_type: BYTE,                   //+
    crystallizable: BYTE,                  //+
    related_quests_ids: UVEC<BYTE, USHORT>, //+
    color: BYTE,                           //+ Quality
    is_blessed: BYTE,                      //+
    property_params: SHORT,                //+
    icon_panel: DWORD,                     //+

    complete_item_drop_sound: DWORD,       //+

    inventory_type: BYTE,                  //+
    body_part: BYTE,                       //+
    hand_stance_type: BYTE,                //+ character_animation_type

    mesh: UVEC<BYTE, MeshDatInfo>,         //+
    texture: UVEC<BYTE, DWORD>,            //+

    item_sound: UVEC<BYTE, DWORD>,         //+

    drop_sound: DWORD,                     //+

    equip_sound: DWORD,                    //+
    effect: DWORD,                         //+
    random_damage_type: BYTE,              //+
    weapon_type: BYTE,                     //+
    crystal_type: BYTE,                    //+
    mp_consume: BYTE,                      //+
    soulshot_count: BYTE,                  //+
    spiritshot_count: BYTE,                //+
    curvature: SHORT,                      //+
    unk_1: BYTE,                           //+
    can_equip_hero: BYTE,                  //+
    is_magic_weapon: BYTE,                 //+
    ertheia_fist_scale: FLOAT,             //+
    junk: SHORT,                           //+
    enchant_info: UVEC<BYTE, EnchantInfo>, //+
    variation_effect_1: BYTE,              //+
    variation_effect_2: BYTE,              //+
    variation_effect_3: BYTE,              //+
    variation_effect_4: BYTE,              //+
    variation_effect_5: BYTE,              //+
    variation_effect_6: BYTE,              //+
    variation_icon: UVEC<BYTE, DWORD>,     //+
    ensoul_count: BYTE,                    //+
    is_ensoul: BYTE,                       //+
}
impl GetId for WeaponGrpDat {
    fn get_id(&self) -> u32 {
        self.id
    }
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
impl GetId for ItemBaseInfoDat {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct ItemStatDataDat {
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
impl GetId for ItemNameDat {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct AdditionalItemGrpDat {
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
