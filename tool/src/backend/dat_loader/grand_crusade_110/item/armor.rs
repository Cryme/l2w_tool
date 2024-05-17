use crate::backend::dat_loader::grand_crusade_110::item::{
    AdditionalItemGrpDat, DropDatInfo, ItemBaseInfoDat, ItemNameDat, ItemStatDataDat,
};
use crate::backend::dat_loader::grand_crusade_110::{L2GeneralStringTable, Loader110};
use crate::backend::dat_loader::{GetId, L2StringTable};
use crate::backend::entity_editor::WindowParams;
use crate::backend::log_holder::{Log, LogLevel};
use crate::entity::item::armor::{
    Armor, ArmorMeshAdditional, ArmorMeshAdditionalF, ArmorMeshBase, ArmorMeshInfo, ArmorMeshes,
    ArmorType, UnderwaterBodyType1, UnderwaterBodyType2,
};
use crate::entity::item::{
    BodyPart, CrystalType, DropAnimationType, DropType, InventoryType, ItemAdditionalInfo,
    ItemBaseInfo, ItemBattleStats, ItemDefaultAction, ItemDropInfo, ItemDropMeshInfo, ItemIcons,
    ItemMaterial, ItemNameColor, ItemQuality, KeepType,
};
use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};
use l2_rw::ue2_rw::{BYTE, DWORD, MTX, MTX3, SHORT, USHORT, UVEC};
use l2_rw::{deserialize_dat, save_dat, DatVariant};
use num_traits::{FromPrimitive, ToPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use crate::backend::holder::HolderMapOps;

impl From<(&Armor, &mut L2GeneralStringTable)> for ItemNameDat {
    fn from(value: (&Armor, &mut L2GeneralStringTable)) -> Self {
        let (item, table) = value;

        ItemNameDat {
            id: item.base_info.id.0,
            name_link: table.get_index(&item.base_info.name),
            additional_name: (&item.base_info.additional_name).into(),
            description: (&item.base_info.desc).into(),
            popup: item.base_info.popup,
            default_action: item.base_info.default_action.to_string().into(),
            use_order: item.base_info.use_order,
            set_id: item.base_info.set_id.0 as USHORT,
            color: item.base_info.color.to_u8().unwrap(),
            tooltip_texture_link: table.get_index(&item.base_info.tooltip_texture),
            is_trade: item.base_info.is_trade.into(),
            is_drop: item.base_info.is_drop.into(),
            is_destruct: item.base_info.is_destruct.into(),
            is_private_store: item.base_info.is_private_store.into(),
            keep_type: item.base_info.keep_type.to_u8().unwrap(),
            is_npc_trade: item.base_info.is_npc_trade.into(),
            is_commission_store: item.base_info.is_commission_store.into(),
        }
    }
}
impl From<(&Armor, &mut L2GeneralStringTable)> for ItemBaseInfoDat {
    fn from(value: (&Armor, &mut L2GeneralStringTable)) -> Self {
        let (item, _table) = value;

        ItemBaseInfoDat {
            id: item.base_info.id.0,
            default_price: item.base_info.default_price,
            is_premium: item.base_info.is_premium.into(),
        }
    }
}
impl From<(&Armor, &mut L2GeneralStringTable)> for ItemStatDataDat {
    fn from(value: (&Armor, &mut L2GeneralStringTable)) -> Self {
        let (item, _table) = value;

        ItemStatDataDat {
            id: item.base_info.id.0,
            p_defense: item.base_info.battle_stats.inner.p_defense,
            m_defense: item.base_info.battle_stats.inner.m_defense,
            p_attack: item.base_info.battle_stats.inner.p_attack,
            m_attack: item.base_info.battle_stats.inner.m_attack,
            p_attack_speed: item.base_info.battle_stats.inner.p_attack_speed,
            p_hit: item.base_info.battle_stats.inner.p_hit,
            m_hit: item.base_info.battle_stats.inner.m_hit,
            p_critical: item.base_info.battle_stats.inner.p_critical,
            m_critical: item.base_info.battle_stats.inner.m_critical,
            speed: item.base_info.battle_stats.inner.speed,
            shield_defense: item.base_info.battle_stats.inner.shield_defense,
            shield_defense_rate: item.base_info.battle_stats.inner.shield_defense_rate,
            p_avoid: item.base_info.battle_stats.inner.p_avoid,
            m_avoid: item.base_info.battle_stats.inner.m_avoid,
            property_params: item.base_info.battle_stats.inner.property_params,
        }
    }
}
impl From<(&Armor, &mut L2GeneralStringTable)> for AdditionalItemGrpDat {
    fn from(value: (&Armor, &mut L2GeneralStringTable)) -> Self {
        let (item, table) = value;

        AdditionalItemGrpDat {
            id: item.base_info.id.0,
            has_ani: item.base_info.additional_info.inner.has_animation.into(),
            included_items: item
                .base_info
                .additional_info
                .inner
                .include_items
                .iter()
                .map(|v| v.0)
                .collect(),
            max_energy: item.base_info.additional_info.inner.max_energy,
            look_change: table.get_index(&item.base_info.additional_info.inner.look_change),
            hide_cloak: item.base_info.additional_info.inner.hide_cloak.into(),
            unk1: item.base_info.additional_info.inner.unk.into(),
            hide_armor: item.base_info.additional_info.inner.hide_armor.into(),
        }
    }
}

impl ArmorMeshInfo {
    fn as_dat_data(&self, table: &mut L2GeneralStringTable) -> ArmorDatMeshInfo {
        ArmorDatMeshInfo {
            base: MTX {
                vec_1: self
                    .base
                    .unk1
                    .iter()
                    .map(|v| table.get_index(v))
                    .collect::<Vec<u32>>()
                    .into(),
                vec_2: self
                    .base
                    .unk2
                    .iter()
                    .map(|v| table.get_index(v))
                    .collect::<Vec<u32>>()
                    .into(),
            },
            additional: MTX3 {
                vec_1: self
                    .additional
                    .unk1
                    .iter()
                    .map(|v| table.get_index(&v.unk2))
                    .collect::<Vec<u32>>(),
                vec_1_f: self
                    .additional
                    .unk1
                    .iter()
                    .map(|v| (v.unk3, v.unk4))
                    .collect::<Vec<(u8, u8)>>(),
                vec_2: self
                    .additional
                    .unk5
                    .iter()
                    .map(|v| table.get_index(v))
                    .collect::<Vec<u32>>(),
                val: table.get_index(&self.additional.unk6),
            },
        }
    }
}

impl From<(&Armor, &mut L2GeneralStringTable)> for ArmorGrpDat {
    fn from(value: (&Armor, &mut L2GeneralStringTable)) -> Self {
        let (item, table) = value;

        Self {
            tag: 1,
            id: item.base_info.id.0,
            drop_type: item.base_info.drop_info.inner.drop_type.to_u8().unwrap(),
            drop_animation_type: item
                .base_info
                .drop_info
                .inner
                .drop_animation_type
                .to_u8()
                .unwrap(),
            drop_radius: item.base_info.drop_info.inner.drop_radius.to_u8().unwrap(),
            drop_height: item.base_info.drop_info.inner.drop_height.to_u8().unwrap(),
            drop_info: item
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
            icon_1: table.get_index(&item.base_info.icons.inner.icon_1),
            icon_2: table.get_index(&item.base_info.icons.inner.icon_2),
            icon_3: table.get_index(&item.base_info.icons.inner.icon_3),
            icon_4: table.get_index(&item.base_info.icons.inner.icon_4),
            icon_5: table.get_index(&item.base_info.icons.inner.icon_5),
            durability: item.base_info.durability,
            weight: item.base_info.weight,
            material_type: item.base_info.material.to_u8().unwrap(),
            crystallizable: item.base_info.crystallizable.into(),
            related_quests_ids: item
                .base_info
                .related_quests
                .iter()
                .map(|v| v.0 as u16)
                .collect::<Vec<u16>>()
                .into(),
            color: item.base_info.color.to_u8().unwrap(),
            is_blessed: item.base_info.is_blessed.into(),
            property_params: item.base_info.property_params,
            icon_panel: table.get_index(&item.base_info.icons.inner.icon_panel),
            complete_item_drop_sound: table
                .get_index(&item.base_info.drop_info.inner.complete_item_drop_sound),
            inventory_type: item.base_info.inventory_type.to_u8().unwrap(),
            body_part: item.base_info.body_part.to_u8().unwrap(),
            m_human_fighter: item.mesh_info.inner.m_human_fighter.as_dat_data(table),
            f_human_fighter: item.mesh_info.inner.f_human_fighter.as_dat_data(table),
            m_dark_elf: item.mesh_info.inner.m_dark_elf.as_dat_data(table),
            f_dark_elf: item.mesh_info.inner.f_dark_elf.as_dat_data(table),
            m_dwarf: item.mesh_info.inner.m_dwarf.as_dat_data(table),
            f_dwarf: item.mesh_info.inner.f_dwarf.as_dat_data(table),
            m_elf: item.mesh_info.inner.m_elf.as_dat_data(table),
            f_elf: item.mesh_info.inner.f_elf.as_dat_data(table),
            m_human_mystic: item.mesh_info.inner.m_human_mystic.as_dat_data(table),
            f_human_mystic: item.mesh_info.inner.f_human_mystic.as_dat_data(table),
            m_orc_fighter: item.mesh_info.inner.m_orc_fighter.as_dat_data(table),
            f_orc_fighter: item.mesh_info.inner.f_orc_fighter.as_dat_data(table),
            m_orc_mystic: item.mesh_info.inner.m_orc_mystic.as_dat_data(table),
            f_orc_mystic: item.mesh_info.inner.f_orc_mystic.as_dat_data(table),
            m_kamael: item.mesh_info.inner.m_kamael.as_dat_data(table),
            f_kamael: item.mesh_info.inner.f_kamael.as_dat_data(table),
            m_ertheia: item.mesh_info.inner.m_ertheia.as_dat_data(table),
            f_ertheia: item.mesh_info.inner.f_ertheia.as_dat_data(table),
            npc: item.mesh_info.inner.npc.as_dat_data(table),
            attack_effect: table.get_index(&item.attack_effect),
            item_sound: item
                .item_sound
                .iter()
                .map(|v| table.get_index(v))
                .collect::<Vec<u32>>()
                .into(),
            drop_sound: table.get_index(&item.base_info.drop_info.inner.drop_sound),
            equip_sound: table.get_index(&item.base_info.equip_sound),
            unk1: item.unk1,
            unk2: item.unk2.into(),
            armor_type: item.armor_type.to_u8().unwrap(),
            crystal_type: item.base_info.crystal_type.to_u8().unwrap(),
            mp_bonus: item.mp_bonus,
            hide_mask: item.hide_mask,
            underwear_body_part1: item.underwater_body_type1.to_u8().unwrap(),
            underwear_body_part2: item.underwater_body_type2.to_u8().unwrap(),
            full_armor_enchant_effect_type: item.set_enchant_effect_id.0,
        }
    }
}

impl Loader110 {
    pub fn serialize_armor_to_binary(&mut self) -> JoinHandle<Log> {
        let mut items: Vec<ArmorGrpDat> = vec![];

        for v in self.armor.values().filter(|v| !v._deleted) {
            items.push((v, &mut self.game_data_name).into())
        }

        let armor_grp_path = self
            .dat_paths
            .get(&"armorgrp.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) = save_dat(
                armor_grp_path.path(),
                DatVariant::<(), ArmorGrpDat>::Array(items),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("Armor Grp saved")
            }
        })
    }

    pub fn fill_items_from_armor(
        &mut self,
        additional_item_grp: &mut Vec<AdditionalItemGrpDat>,
        item_stat: &mut Vec<ItemStatDataDat>,
        item_base_info: &mut Vec<ItemBaseInfoDat>,
        item_name: &mut Vec<ItemNameDat>,
    ) {
        for v in self.armor.values() {
            additional_item_grp.push((v, &mut self.game_data_name).into());
            item_stat.push((v, &mut self.game_data_name).into());
            item_base_info.push((v, &mut self.game_data_name).into());
            item_name.push((v, &mut self.game_data_name).into());
        }
    }

    pub(crate) fn load_armor(
        &mut self,
        additional_item_grp: &HashMap<u32, AdditionalItemGrpDat>,
        item_stat: &HashMap<u32, ItemStatDataDat>,
        item_base_info: &HashMap<u32, ItemBaseInfoDat>,
        item_name: &HashMap<u32, ItemNameDat>,
    ) -> Result<Vec<Log>, ()> {
        let armor_grp = deserialize_dat::<ArmorGrpDat>(
            self.dat_paths
                .get(&"armorgrp.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let base_info_default = ItemBaseInfoDat::default();
        let base_stat_default = ItemStatDataDat::default();
        let additional_default = AdditionalItemGrpDat::default();
        let mut warnings = vec![];

        for item in armor_grp {
            let Some(name_grp) = item_name.get(&item.id) else {
                warnings.push(Log {
                    level: LogLevel::Error,
                    producer: "Armor Loader".to_string(),
                    log: format!("Item[{}]: No record in itemname found. Skipped", item.id),
                });

                continue;
            };

            let base_info_grp = item_base_info.get(&item.id).unwrap_or(&base_info_default);
            let add_info_grp = additional_item_grp
                .get(&item.id)
                .unwrap_or(&additional_default);
            let stats = item_stat.get(&item.id).unwrap_or(&base_stat_default);

            let mut drop_mesh_info = vec![];
            for v in &item.drop_info {
                drop_mesh_info.push(ItemDropMeshInfo {
                    mesh: self.game_data_name.get_o(&v.mesh),
                    textures: v
                        .texture
                        .inner
                        .iter()
                        .map(|vv| self.game_data_name.get_o(vv))
                        .collect(),
                })
            }

            let drop_info = ItemDropInfo {
                drop_type: DropType::from_u8(item.drop_type).unwrap(),
                drop_animation_type: DropAnimationType::from_u8(item.drop_animation_type).unwrap(),
                drop_radius: item.drop_radius,
                drop_height: item.drop_height,
                drop_mesh_info,
                complete_item_drop_sound: self.game_data_name.get_o(&item.complete_item_drop_sound),
                drop_sound: self.game_data_name.get_o(&item.drop_sound),
            };

            self.armor.insert(
                item.id.into(),
                Armor {
                    base_info: ItemBaseInfo {
                        id: item.id.into(),
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
                        inventory_type: InventoryType::from_u8(item.inventory_type).unwrap(),
                        material: ItemMaterial::from_u8(item.material_type).unwrap(),
                        body_part: BodyPart::from_u8(item.body_part).unwrap(),
                        quality: ItemQuality::from_u8(item.color).unwrap(),
                        crystallizable: item.crystallizable == 1,
                        crystal_type: CrystalType::from_u8(item.crystal_type).unwrap(),
                        durability: item.durability,
                        weight: item.weight,
                        icons: WindowParams::new(ItemIcons {
                            icon_1: self.game_data_name.get_o(&item.icon_1),
                            icon_2: self.game_data_name.get_o(&item.icon_2),
                            icon_3: self.game_data_name.get_o(&item.icon_3),
                            icon_4: self.game_data_name.get_o(&item.icon_4),
                            icon_5: self.game_data_name.get_o(&item.icon_5),
                            icon_panel: self.game_data_name.get_o(&item.icon_panel),
                        }),
                        default_price: base_info_grp.default_price,
                        is_premium: base_info_grp.is_premium == 1,
                        is_blessed: item.is_blessed == 1,
                        property_params: item.property_params,
                        related_quests: item
                            .related_quests_ids
                            .inner
                            .iter()
                            .map(|v| (*v).into())
                            .collect(),
                        equip_sound: self.game_data_name.get_o(&item.equip_sound),
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
                    armor_type: ArmorType::from_u8(item.armor_type).unwrap(),
                    attack_effect: self.game_data_name.get_o(&item.attack_effect),
                    item_sound: item
                        .item_sound
                        .inner
                        .iter()
                        .map(|v| self.game_data_name.get_o(v))
                        .collect(),
                    unk1: item.unk1,
                    unk2: item.unk2 == 1,
                    mp_bonus: item.mp_bonus,
                    hide_mask: item.hide_mask,
                    underwater_body_type1: UnderwaterBodyType1::from_u8(item.unk2).unwrap(),
                    underwater_body_type2: UnderwaterBodyType2::from_u8(item.unk2).unwrap(),
                    set_enchant_effect_id: item.full_armor_enchant_effect_type.into(),
                    mesh_info: WindowParams::new(ArmorMeshes {
                        m_human_fighter: (&item.m_human_fighter, &self.game_data_name).into(),
                        f_human_fighter: (&item.f_human_fighter, &self.game_data_name).into(),
                        m_dark_elf: (&item.m_dark_elf, &self.game_data_name).into(),
                        f_dark_elf: (&item.f_dark_elf, &self.game_data_name).into(),
                        m_dwarf: (&item.m_dwarf, &self.game_data_name).into(),
                        f_dwarf: (&item.f_dwarf, &self.game_data_name).into(),
                        m_elf: (&item.m_elf, &self.game_data_name).into(),
                        f_elf: (&item.f_elf, &self.game_data_name).into(),
                        m_human_mystic: (&item.m_human_mystic, &self.game_data_name).into(),
                        f_human_mystic: (&item.f_human_mystic, &self.game_data_name).into(),
                        m_orc_fighter: (&item.m_orc_fighter, &self.game_data_name).into(),
                        f_orc_fighter: (&item.f_orc_fighter, &self.game_data_name).into(),
                        m_orc_mystic: (&item.m_orc_mystic, &self.game_data_name).into(),
                        f_orc_mystic: (&item.f_orc_mystic, &self.game_data_name).into(),
                        m_kamael: (&item.m_kamael, &self.game_data_name).into(),
                        f_kamael: (&item.f_kamael, &self.game_data_name).into(),
                        m_ertheia: (&item.m_ertheia, &self.game_data_name).into(),
                        f_ertheia: (&item.f_ertheia, &self.game_data_name).into(),
                        npc: (&item.npc, &self.game_data_name).into(),
                    }),
                    ..Default::default()
                },
            );
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct ArmorGrpDat {
    tag: BYTE,
    id: DWORD,                              //+
    drop_type: BYTE,                        //+
    drop_animation_type: BYTE,              //+
    drop_radius: BYTE,                      //+
    drop_height: BYTE,                      //+
    drop_info: UVEC<BYTE, DropDatInfo>,     //+
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
    complete_item_drop_sound: DWORD,        //+
    inventory_type: BYTE,                   //+
    body_part: BYTE,                        //+
    //============================================================//
    m_human_fighter: ArmorDatMeshInfo,
    f_human_fighter: ArmorDatMeshInfo,

    m_dark_elf: ArmorDatMeshInfo,
    f_dark_elf: ArmorDatMeshInfo,

    m_dwarf: ArmorDatMeshInfo,
    f_dwarf: ArmorDatMeshInfo,

    m_elf: ArmorDatMeshInfo,
    f_elf: ArmorDatMeshInfo,

    m_human_mystic: ArmorDatMeshInfo,
    f_human_mystic: ArmorDatMeshInfo,

    m_orc_fighter: ArmorDatMeshInfo,
    f_orc_fighter: ArmorDatMeshInfo,

    m_orc_mystic: ArmorDatMeshInfo,
    f_orc_mystic: ArmorDatMeshInfo,

    m_kamael: ArmorDatMeshInfo,
    f_kamael: ArmorDatMeshInfo,

    m_ertheia: ArmorDatMeshInfo,
    f_ertheia: ArmorDatMeshInfo,

    npc: ArmorDatMeshInfo,
    //============================================================//
    attack_effect: DWORD,          //+
    item_sound: UVEC<BYTE, DWORD>, //+
    drop_sound: DWORD,             //+
    equip_sound: DWORD,            //+
    unk1: DWORD,                   //+
    unk2: BYTE,                    //+
    armor_type: BYTE,              //+
    crystal_type: BYTE,            //+
    mp_bonus: USHORT,
    hide_mask: USHORT,
    underwear_body_part1: BYTE,
    underwear_body_part2: BYTE,
    full_armor_enchant_effect_type: BYTE,
}

impl GetId for ArmorGrpDat {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct ArmorDatMeshInfo {
    base: MTX,
    additional: MTX3,
}

impl From<(&ArmorDatMeshInfo, &L2GeneralStringTable)> for ArmorMeshInfo {
    fn from(value: (&ArmorDatMeshInfo, &L2GeneralStringTable)) -> Self {
        let (data, table) = value;

        let mut unk1 = Vec::with_capacity(data.additional.vec_1.len());

        for (i, v) in data.additional.vec_1.iter().enumerate() {
            unk1.push(ArmorMeshAdditionalF {
                unk2: table.get_o(v),
                unk3: data.additional.vec_1_f[i].0,
                unk4: data.additional.vec_1_f[i].1,
            })
        }

        Self {
            base: ArmorMeshBase {
                unk1: data
                    .base
                    .vec_1
                    .inner
                    .iter()
                    .map(|v| table.get_o(v))
                    .collect(),
                unk2: data
                    .base
                    .vec_2
                    .inner
                    .iter()
                    .map(|v| table.get_o(v))
                    .collect(),
            },
            additional: ArmorMeshAdditional {
                unk1,
                unk5: data
                    .additional
                    .vec_2
                    .iter()
                    .map(|v| table.get_o(v))
                    .collect(),
                unk6: table.get_o(&data.additional.val),
            },
        }
    }
}
