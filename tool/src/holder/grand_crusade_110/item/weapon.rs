use crate::backend::WindowParams;
use crate::entity::item::weapon::{
    CharacterAnimationType, RandomDamage, Weapon, WeaponEnchantInfo, WeaponEnchantParams,
    WeaponMeshInfo, WeaponMpConsume, WeaponType, WeaponVariationInfo,
};
use crate::entity::item::{
    BodyPart, CrystalType, DropAnimationType, DropType, InventoryType, ItemAdditionalInfo,
    ItemBaseInfo, ItemBattleStats, ItemDefaultAction, ItemDropInfo, ItemDropMeshInfo, ItemIcons,
    ItemMaterial, ItemNameColor, ItemQuality, KeepType,
};
use crate::holder::grand_crusade_110::item::{
    AdditionalItemGrpDat, DropDatInfo, ItemBaseInfoDat, ItemNameDat, ItemStatDataDat,
};
use crate::holder::grand_crusade_110::{CoordsXYZ, L2GeneralStringTable, Loader110};
use crate::util::l2_reader::{deserialize_dat, save_dat, DatVariant};
use crate::util::{
    GetId, L2StringTable, ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal, ASCF, BYTE, DWORD,
    FLOAT, SHORT, USHORT, UVEC,
};
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;

impl From<(&Weapon, &mut L2GeneralStringTable)> for ItemNameDat {
    fn from(value: (&Weapon, &mut L2GeneralStringTable)) -> Self {
        let (weapon, table) = value;

        ItemNameDat {
            id: weapon.base_info.id.0,
            name_link: table.get_index(&weapon.base_info.name),
            additional_name: (&weapon.base_info.additional_name).into(),
            description: (&weapon.base_info.desc).into(),
            popup: weapon.base_info.popup,
            default_action: ASCF(weapon.base_info.default_action.to_string()),
            use_order: weapon.base_info.use_order,
            set_id: weapon.base_info.set_id.0 as USHORT,
            color: weapon.base_info.color.to_u8().unwrap(),
            tooltip_texture_link: table.get_index(&weapon.base_info.tooltip_texture),
            is_trade: weapon.base_info.is_trade.into(),
            is_drop: weapon.base_info.is_drop.into(),
            is_destruct: weapon.base_info.is_destruct.into(),
            is_private_store: weapon.base_info.is_private_store.into(),
            keep_type: weapon.base_info.keep_type.to_u8().unwrap(),
            is_npc_trade: weapon.base_info.is_npc_trade.into(),
            is_commission_store: weapon.base_info.is_commission_store.into(),
        }
    }
}
impl From<(&Weapon, &mut L2GeneralStringTable)> for ItemBaseInfoDat {
    fn from(value: (&Weapon, &mut L2GeneralStringTable)) -> Self {
        let (weapon, _table) = value;

        ItemBaseInfoDat {
            id: weapon.base_info.id.0,
            default_price: weapon.base_info.default_price,
            is_premium: weapon.base_info.is_premium.into(),
        }
    }
}
impl From<(&Weapon, &mut L2GeneralStringTable)> for ItemStatDataDat {
    fn from(value: (&Weapon, &mut L2GeneralStringTable)) -> Self {
        let (weapon, _table) = value;

        ItemStatDataDat {
            id: weapon.base_info.id.0,
            p_defense: weapon.base_info.battle_stats.inner.p_defense,
            m_defense: weapon.base_info.battle_stats.inner.m_defense,
            p_attack: weapon.base_info.battle_stats.inner.p_attack,
            m_attack: weapon.base_info.battle_stats.inner.m_attack,
            p_attack_speed: weapon.base_info.battle_stats.inner.p_attack_speed,
            p_hit: weapon.base_info.battle_stats.inner.p_hit,
            m_hit: weapon.base_info.battle_stats.inner.m_hit,
            p_critical: weapon.base_info.battle_stats.inner.p_critical,
            m_critical: weapon.base_info.battle_stats.inner.m_critical,
            speed: weapon.base_info.battle_stats.inner.speed,
            shield_defense: weapon.base_info.battle_stats.inner.shield_defense,
            shield_defense_rate: weapon.base_info.battle_stats.inner.shield_defense_rate,
            p_avoid: weapon.base_info.battle_stats.inner.p_avoid,
            m_avoid: weapon.base_info.battle_stats.inner.m_avoid,
            property_params: weapon.base_info.battle_stats.inner.property_params,
        }
    }
}
impl From<(&Weapon, &mut L2GeneralStringTable)> for AdditionalItemGrpDat {
    fn from(value: (&Weapon, &mut L2GeneralStringTable)) -> Self {
        let (weapon, table) = value;

        AdditionalItemGrpDat {
            id: weapon.base_info.id.0,
            has_ani: weapon.base_info.additional_info.inner.has_animation.into(),
            included_items: weapon
                .base_info
                .additional_info
                .inner
                .include_items
                .iter()
                .map(|v| v.0)
                .collect(),
            max_energy: weapon.base_info.additional_info.inner.max_energy,
            look_change: table.get_index(&weapon.base_info.additional_info.inner.look_change),
            hide_cloak: weapon.base_info.additional_info.inner.hide_cloak.into(),
            unk1: weapon.base_info.additional_info.inner.unk.into(),
            hide_armor: weapon.base_info.additional_info.inner.hide_armor.into(),
        }
    }
}
impl From<(&Weapon, &mut L2GeneralStringTable)> for WeaponGrpDat {
    fn from(value: (&Weapon, &mut L2GeneralStringTable)) -> Self {
        let (weapon, table) = value;

        Self {
            tag: 0,
            id: weapon.base_info.id.0,
            drop_type: weapon.base_info.drop_info.inner.drop_type.to_u8().unwrap(),
            drop_animation_type: weapon
                .base_info
                .drop_info
                .inner
                .drop_animation_type
                .to_u8()
                .unwrap(),
            drop_radius: weapon
                .base_info
                .drop_info
                .inner
                .drop_radius
                .to_u8()
                .unwrap(),
            drop_height: weapon
                .base_info
                .drop_info
                .inner
                .drop_height
                .to_u8()
                .unwrap(),
            drop_info: weapon
                .base_info
                .drop_info
                .inner
                .drop_mesh_info
                .iter()
                .map(|v| DropDatInfo {
                    mesh: table.get_index(&v.mesh),
                    texture: v
                        .textures
                        .iter()
                        .map(|vv| table.get_index(vv))
                        .collect::<Vec<u32>>()
                        .into(),
                })
                .collect::<Vec<DropDatInfo>>()
                .into(),
            icon_1: table.get_index(&weapon.base_info.icons.inner.icon_1),
            icon_2: table.get_index(&weapon.base_info.icons.inner.icon_2),
            icon_3: table.get_index(&weapon.base_info.icons.inner.icon_3),
            icon_4: table.get_index(&weapon.base_info.icons.inner.icon_4),
            icon_5: table.get_index(&weapon.base_info.icons.inner.icon_5),
            durability: weapon.base_info.durability,
            weight: weapon.base_info.weight,
            material_type: weapon.base_info.material.to_u8().unwrap(),
            crystallizable: weapon.base_info.crystallizable.into(),
            related_quests_ids: weapon
                .base_info
                .related_quests
                .iter()
                .map(|v| v.0 as u16)
                .collect::<Vec<u16>>()
                .into(),
            color: weapon.base_info.color.to_u8().unwrap(),
            is_blessed: weapon.base_info.is_blessed.into(),
            property_params: weapon.base_info.property_params,
            icon_panel: table.get_index(&weapon.base_info.icons.inner.icon_panel),
            complete_item_drop_sound: table
                .get_index(&weapon.base_info.drop_info.inner.complete_item_drop_sound),
            inventory_type: weapon.base_info.inventory_type.to_u8().unwrap(),
            body_part: weapon.base_info.body_part.to_u8().unwrap(),
            hand_stance_type: weapon.character_animation_type.to_u8().unwrap(),
            mesh: weapon
                .mesh_info
                .iter()
                .map(|v| MeshDatInfo {
                    mesh: table.get_index(&v.mesh),
                    unk1: v.unk,
                })
                .collect::<Vec<MeshDatInfo>>()
                .into(),
            texture: weapon
                .mesh_info
                .iter()
                .map(|v| table.get_index(&v.texture))
                .collect::<Vec<u32>>()
                .into(),
            item_sound: weapon
                .sound
                .iter()
                .map(|v| table.get_index(v))
                .collect::<Vec<u32>>()
                .into(),
            drop_sound: table.get_index(&weapon.base_info.drop_info.inner.drop_sound),
            equip_sound: table.get_index(&weapon.base_info.equip_sound),
            effect: table.get_index(&weapon.effect),
            random_damage_type: weapon.random_damage.to_u8().unwrap(),
            weapon_type: weapon.weapon_type.to_u8().unwrap(),
            crystal_type: weapon.base_info.crystal_type.to_u8().unwrap(),
            mp_consume: weapon.mp_consume.to_u8().unwrap(),
            soulshot_count: weapon.soulshot_count,
            spiritshot_count: weapon.spiritshot_count,
            curvature: weapon.curvature,
            unk_1: weapon.unk.into(),
            can_equip_hero: weapon.can_equip_hero.into(),
            is_magic_weapon: weapon.is_magic_weapon.into(),
            ertheia_fist_scale: weapon.ertheia_fists_scale,
            junk: weapon.enchant_info.inner.junk,
            enchant_info: weapon
                .enchant_info
                .inner
                .params
                .iter()
                .map(|v| EnchantInfo {
                    effect: table.get_index(&v.effect),
                    effect_offset: v.effect_offset.into(),
                    mesh_offset: v.mesh_offset.into(),
                    mesh_scale: v.mesh_scale.into(),
                    effect_velocity: v.effect_velocity,
                    particle_scale: v.particle_scale,
                    effect_scale: v.effect_scale,
                    particle_offset: v.particle_offset.into(),
                    ring_offset: v.ring_offset.into(),
                    ring_scale: v.ring_scale.into(),
                })
                .collect::<Vec<EnchantInfo>>()
                .into(),
            variation_effect_1: weapon.variation_info.inner.effect_1,
            variation_effect_2: weapon.variation_info.inner.effect_2,
            variation_effect_3: weapon.variation_info.inner.effect_3,
            variation_effect_4: weapon.variation_info.inner.effect_4,
            variation_effect_5: weapon.variation_info.inner.effect_5,
            variation_effect_6: weapon.variation_info.inner.effect_6,
            variation_icon: weapon
                .variation_info
                .inner
                .icon
                .iter()
                .map(|v| table.get_index(v))
                .collect::<Vec<u32>>()
                .into(),
            ensoul_count: weapon.ensoul_count,
            is_ensoul: weapon.can_ensoul.into(),
        }
    }
}

impl Loader110 {
    pub fn serialize_weapons_to_binary(&mut self) -> JoinHandle<()> {
        let mut weapons: Vec<WeaponGrpDat> = vec![];

        for v in self.weapons.values() {
            weapons.push((v, &mut self.game_data_name).into())
        }

        let weapon_grp_path = self
            .dat_paths
            .get(&"weapongrp.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) = save_dat(
                weapon_grp_path.path(),
                DatVariant::<(), WeaponGrpDat>::Array(weapons),
            ) {
                println!("{e:?}");
            } else {
                println!("Weapon Grp saved");
            }
        })
    }

    pub fn fill_items_from_weapons(
        &mut self,
        additional_item_grp: &mut Vec<AdditionalItemGrpDat>,
        item_stat: &mut Vec<ItemStatDataDat>,
        item_base_info: &mut Vec<ItemBaseInfoDat>,
        item_name: &mut Vec<ItemNameDat>,
    ) {
        for (_, v) in &self.weapons.inner {
            additional_item_grp.push((v, &mut self.game_data_name).into());
            item_stat.push((v, &mut self.game_data_name).into());
            item_base_info.push((v, &mut self.game_data_name).into());
            item_name.push((v, &mut self.game_data_name).into());
        }
    }

    pub(crate) fn load_weapons(
        &mut self,
        additional_item_grp: &HashMap<u32, AdditionalItemGrpDat>,
        item_stat: &HashMap<u32, ItemStatDataDat>,
        item_base_info: &HashMap<u32, ItemBaseInfoDat>,
        item_name: &HashMap<u32, ItemNameDat>,
    ) -> Result<(), ()> {
        let weapon_grp = deserialize_dat::<WeaponGrpDat>(
            self.dat_paths
                .get(&"weapongrp.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let base_info_default = ItemBaseInfoDat::default();
        let base_stat_default = ItemStatDataDat::default();
        let additional_default = AdditionalItemGrpDat::default();
        let mut skipped = vec![];

        for weapon in weapon_grp {
            let Some(name_grp) = item_name.get(&weapon.id) else {
                skipped.push(weapon.id);

                continue;
            };

            let base_info_grp = item_base_info.get(&weapon.id).unwrap_or(&base_info_default);
            let add_info_grp = additional_item_grp
                .get(&weapon.id)
                .unwrap_or(&additional_default);
            let stats = item_stat.get(&weapon.id).unwrap_or(&base_stat_default);

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
                    textures: v
                        .texture
                        .inner
                        .iter()
                        .map(|vv| self.game_data_name.get_o(&vv))
                        .collect(),
                })
            }

            let drop_info = ItemDropInfo {
                drop_type: DropType::from_u8(weapon.drop_type).unwrap(),
                drop_animation_type: DropAnimationType::from_u8(weapon.drop_animation_type)
                    .unwrap(),
                drop_radius: weapon.drop_radius,
                drop_height: weapon.drop_height,
                drop_mesh_info,
                complete_item_drop_sound: self
                    .game_data_name
                    .get_o(&weapon.complete_item_drop_sound),
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
                        durability: weapon.durability,
                        weight: weapon.weight,
                        icons: WindowParams::new(ItemIcons {
                            icon_1: self.game_data_name.get_o(&weapon.icon_1),
                            icon_2: self.game_data_name.get_o(&weapon.icon_2),
                            icon_3: self.game_data_name.get_o(&weapon.icon_3),
                            icon_4: self.game_data_name.get_o(&weapon.icon_4),
                            icon_5: self.game_data_name.get_o(&weapon.icon_5),
                            icon_panel: self.game_data_name.get_o(&weapon.icon_panel),
                        }),
                        default_price: base_info_grp.default_price,
                        is_premium: base_info_grp.is_premium == 1,
                        is_blessed: weapon.is_blessed == 1,
                        property_params: weapon.property_params,
                        related_quests: weapon
                            .related_quests_ids
                            .inner
                            .iter()
                            .map(|v| (*v).into())
                            .collect(),
                        equip_sound: self.game_data_name.get_o(&weapon.equip_sound),
                        additional_info: WindowParams::new(ItemAdditionalInfo {
                            has_animation: add_info_grp.has_ani == 1,
                            include_items: add_info_grp
                                .included_items
                                .iter()
                                .map(|v| (*v).into())
                                .collect(),
                            max_energy: add_info_grp.max_energy,
                            look_change: self.game_data_name.get_o(&add_info_grp.look_change),
                            hide_cloak: add_info_grp.hide_cloak == 1,
                            unk: add_info_grp.unk1 == 1,
                            hide_armor: add_info_grp.hide_armor == 1,
                        }),
                        drop_info: WindowParams::new(drop_info),
                        battle_stats: WindowParams::new(ItemBattleStats {
                            p_defense: stats.p_defense,
                            m_defense: stats.m_defense,
                            p_attack: stats.p_attack,
                            m_attack: stats.m_attack,
                            p_attack_speed: stats.p_attack_speed,
                            p_hit: stats.p_hit,
                            m_hit: stats.m_hit,
                            p_critical: stats.p_critical,
                            m_critical: stats.m_critical,
                            speed: stats.speed,
                            shield_defense: stats.shield_defense,
                            shield_defense_rate: stats.shield_defense_rate,
                            p_avoid: stats.p_avoid,
                            m_avoid: stats.m_avoid,
                            property_params: stats.property_params,
                        }),
                    },
                    weapon_type: WeaponType::from_u8(weapon.weapon_type).unwrap(),
                    character_animation_type: CharacterAnimationType::from_u8(
                        weapon.hand_stance_type,
                    )
                    .unwrap(),
                    random_damage: RandomDamage::from_u8(weapon.random_damage_type).unwrap(),
                    ertheia_fists_scale: weapon.ertheia_fist_scale,
                    mesh_info,
                    sound: weapon
                        .item_sound
                        .inner
                        .iter()
                        .map(|v| self.game_data_name.get_o(v))
                        .collect(),
                    effect: self.game_data_name.get_o(&weapon.effect),
                    mp_consume: WeaponMpConsume::from_u8(weapon.mp_consume).unwrap(),
                    soulshot_count: weapon.soulshot_count,
                    spiritshot_count: weapon.spiritshot_count,
                    curvature: weapon.curvature,
                    unk: weapon.unk_1 == 1,
                    can_equip_hero: weapon.can_equip_hero == 1,
                    is_magic_weapon: weapon.is_magic_weapon == 1,
                    enchant_info: WindowParams::new(WeaponEnchantInfo {
                        junk: weapon.junk,
                        params: weapon
                            .enchant_info
                            .inner
                            .iter()
                            .map(|v| WeaponEnchantParams {
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
                            })
                            .collect(),
                    }),
                    variation_info: WindowParams::new(WeaponVariationInfo {
                        icon: weapon
                            .variation_icon
                            .inner
                            .iter()
                            .map(|v| self.game_data_name.get_o(v))
                            .collect(),
                        effect_1: weapon.variation_effect_1,
                        effect_2: weapon.variation_effect_2,
                        effect_3: weapon.variation_effect_3,
                        effect_4: weapon.variation_effect_4,
                        effect_5: weapon.variation_effect_5,
                        effect_6: weapon.variation_effect_6,
                    }),
                    can_ensoul: weapon.is_ensoul == 1,
                    ensoul_count: weapon.ensoul_count,
                },
            )
        }

        if !skipped.is_empty() {
            println!(
                "Skipped {} Weapons, because no record in ItemName was found:\n{:?}",
                skipped.len(),
                skipped
            );
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct WeaponGrpDat {
    tag: BYTE,
    id: DWORD, //+

    drop_type: BYTE,                    //+
    drop_animation_type: BYTE,          //+
    drop_radius: BYTE,                  //+
    drop_height: BYTE,                  //+
    drop_info: UVEC<BYTE, DropDatInfo>, //+

    icon_1: DWORD,                          //+
    icon_2: DWORD,                          //+
    icon_3: DWORD,                          //+
    icon_4: DWORD,                          //+
    icon_5: DWORD,                          //+
    durability: USHORT,                     //+
    weight: USHORT,                         //+
    material_type: BYTE,                    //+
    crystallizable: BYTE,                   //+
    related_quests_ids: UVEC<BYTE, USHORT>, //+
    color: BYTE,                            //+ Quality
    is_blessed: BYTE,                       //+
    property_params: SHORT,                 //+
    icon_panel: DWORD,                      //+

    complete_item_drop_sound: DWORD, //+

    inventory_type: BYTE,   //+
    body_part: BYTE,        //+
    hand_stance_type: BYTE, //+ character_animation_type

    mesh: UVEC<BYTE, MeshDatInfo>, //+
    texture: UVEC<BYTE, DWORD>,    //+

    item_sound: UVEC<BYTE, DWORD>, //+

    drop_sound: DWORD, //+

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
