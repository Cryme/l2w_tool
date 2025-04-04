use crate::backend::dat_loader::protocol_166::L2GeneralStringTable;
use crate::backend::dat_loader::protocol_166::item::{
    AdditionalItemGrpDat, DropDatInfo, ItemBaseInfoDat, ItemNameDat, ItemStatDataDat,
};
use crate::backend::editor::WindowParams;
use crate::entity::item::etc_item::{
    ConsumeType, EnsoulSlotType, EnsoulStone, EtcItem, EtcItemType, EtcMeshInfo,
};
use crate::entity::item::{
    BodyPart, CrystalType, DropAnimationType, DropType, InventoryType, ItemAdditionalInfo,
    ItemBaseInfo, ItemBattleStats, ItemDefaultAction, ItemDropInfo, ItemDropMeshInfo, ItemIcons,
    ItemMaterial, ItemNameColor, ItemQuality, KeepType,
};

use l2_rw::ue2_rw::{BYTE, DWORD, SHORT, USHORT, UVEC};
use l2_rw::{DatVariant, deserialize_dat, save_dat};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::Localization;
use crate::backend::dat_loader::{GetId, wrap_into_id_map};
use crate::backend::holder::{GameDataHolder, HolderMapOps};
use crate::backend::log_holder::{Log, LogLevel};
use crate::common::EnsoulOptionId;
use r#macro::{ReadUnreal, WriteUnreal};
use num_traits::{FromPrimitive, ToPrimitive};
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;

impl From<(&EtcItem, &mut L2GeneralStringTable, Localization)> for ItemNameDat {
    fn from(value: (&EtcItem, &mut L2GeneralStringTable, Localization)) -> Self {
        let (item, table, localization) = value;

        ItemNameDat {
            id: item.base_info.id.0,
            name_link: table.get_index(&item.base_info.name[localization]),
            additional_name: (&item.base_info.additional_name[localization]).into(),
            description: (&item.base_info.desc[localization]).into(),
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
impl From<(&EtcItem, &mut L2GeneralStringTable)> for ItemBaseInfoDat {
    fn from(value: (&EtcItem, &mut L2GeneralStringTable)) -> Self {
        let (item, _table) = value;

        ItemBaseInfoDat {
            id: item.base_info.id.0,
            default_price: item.base_info.default_price,
            is_premium: item.base_info.is_premium.into(),
        }
    }
}
impl From<&EtcItem> for Option<EnsoulStoneDat> {
    fn from(item: &EtcItem) -> Self {
        item.ensoul_stone.as_ref().map(|v| EnsoulStoneDat {
            id: item.base_info.id.0,
            slot_type: v.slot_type.to_u32().unwrap(),
            ensoul_options: v.options.iter().map(|v| v.0).collect(),
        })
    }
}
impl From<(&EtcItem, &mut L2GeneralStringTable)> for ItemStatDataDat {
    fn from(value: (&EtcItem, &mut L2GeneralStringTable)) -> Self {
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
impl From<(&EtcItem, &mut L2GeneralStringTable)> for AdditionalItemGrpDat {
    fn from(value: (&EtcItem, &mut L2GeneralStringTable)) -> Self {
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
impl From<(&EtcItem, &mut L2GeneralStringTable)> for EtcItemGrpDat {
    fn from(value: (&EtcItem, &mut L2GeneralStringTable)) -> Self {
        let (item, table) = value;

        Self {
            tag: 2,
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
            mesh: item
                .mesh_info
                .iter()
                .map(|v| table.get_index(&v.mesh))
                .collect::<Vec<u32>>()
                .into(),
            texture: item
                .mesh_info
                .iter()
                .map(|v| table.get_index(&v.texture))
                .collect::<Vec<u32>>()
                .into(),
            drop_sound: table.get_index(&item.base_info.drop_info.inner.drop_sound),
            equip_sound: table.get_index(&item.base_info.equip_sound),
            consume_type: item.consume_type.to_u8().unwrap(),
            crystal_type: item.base_info.crystal_type.to_u8().unwrap(),
            etc_item_type: item.etc_item_type.to_u32().unwrap(),
        }
    }
}

impl GameDataHolder {
    pub fn serialize_etc_items_to_binary(&mut self) -> JoinHandle<(Log, Log)> {
        let mut items: Vec<EtcItemGrpDat> = vec![];
        let mut ensoul_stones: Vec<EnsoulStoneDat> = vec![];

        for v in self.etc_item_holder.values().filter(|v| !v._deleted) {
            items.push((v, &mut self.game_string_table_ru).into());

            if let Some(s) = v.into() {
                ensoul_stones.push(s);
            }
        }

        let etc_item_grp_path = self
            .dat_paths
            .get(&"etcitemgrp.dat".to_string())
            .unwrap()
            .clone();
        let ensoul_stone_path = self
            .dat_paths
            .get(&"ensoul_stone_client.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            let l = if let Err(e) = save_dat(
                ensoul_stone_path.path(),
                DatVariant::<(), EnsoulStoneDat>::Array(ensoul_stones),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("Ensoul Stones Saved")
            };

            if let Err(e) = save_dat(
                etc_item_grp_path.path(),
                DatVariant::<(), EtcItemGrpDat>::Array(items),
            ) {
                (l, Log::from_loader_e(e))
            } else {
                (l, Log::from_loader_i("Etc Item Grp saved"))
            }
        })
    }

    pub fn fill_items_from_etc_items(
        &mut self,
        additional_item_grp: &mut Vec<AdditionalItemGrpDat>,
        item_stat: &mut Vec<ItemStatDataDat>,
        item_base_info: &mut Vec<ItemBaseInfoDat>,
        item_name_ru: &mut Vec<ItemNameDat>,
        item_name_eu: &mut Vec<ItemNameDat>,
    ) {
        for v in self.etc_item_holder.values() {
            additional_item_grp.push((v, &mut self.game_string_table_ru).into());
            item_stat.push((v, &mut self.game_string_table_ru).into());
            item_base_info.push((v, &mut self.game_string_table_ru).into());
            item_name_ru.push((v, &mut self.game_string_table_ru, Localization::RU).into());
            item_name_eu.push((v, &mut self.game_string_table_eu, Localization::EU).into());
        }
    }

    pub(crate) fn load_etc_items(
        &mut self,
        additional_item_grp: &HashMap<u32, AdditionalItemGrpDat>,
        item_stat: &HashMap<u32, ItemStatDataDat>,
        item_base_info: &HashMap<u32, ItemBaseInfoDat>,
        item_name_ru: &HashMap<u32, ItemNameDat>,
        item_name_eu: &HashMap<u32, ItemNameDat>,
    ) -> Result<Vec<Log>, ()> {
        let etc_grp = deserialize_dat::<EtcItemGrpDat>(
            self.dat_paths
                .get(&"etcitemgrp.dat".to_string())
                .unwrap()
                .path(),
        )?;

        let mut stones = wrap_into_id_map(deserialize_dat::<EnsoulStoneDat>(
            self.dat_paths
                .get(&"ensoul_stone_client.dat".to_string())
                .unwrap()
                .path(),
        )?);

        let base_info_default = ItemBaseInfoDat::default();
        let base_stat_default = ItemStatDataDat::default();
        let additional_default = AdditionalItemGrpDat::default();
        let mut warnings = vec![];

        let not_existing_name_eu = ItemNameDat::not_existing();

        for item in etc_grp {
            let Some(name_grp_ru) = item_name_ru.get(&item.id) else {
                warnings.push(Log {
                    level: LogLevel::Error,
                    producer: "Weapon Loader".to_string(),
                    log: format!("Item[{}]: No record in itemname found. Skipped", item.id),
                });

                continue;
            };

            let name_grp_eu = item_name_eu.get(&item.id).unwrap_or(&not_existing_name_eu);

            let base_info_grp = item_base_info.get(&item.id).unwrap_or(&base_info_default);
            let add_info_grp = additional_item_grp
                .get(&item.id)
                .unwrap_or(&additional_default);
            let stats = item_stat.get(&item.id).unwrap_or(&base_stat_default);

            let mut mesh_info = vec![];

            for (i, v) in item.mesh.inner.iter().enumerate() {
                let texture = item.texture.inner.get(i).unwrap();
                mesh_info.push(EtcMeshInfo {
                    mesh: self.game_string_table_ru.get_o(v),
                    texture: self.game_string_table_ru.get_o(texture),
                });
            }

            let mut drop_mesh_info = vec![];
            for v in &item.drop_info {
                drop_mesh_info.push(ItemDropMeshInfo {
                    mesh: self.game_string_table_ru.get_o(&v.mesh),
                    textures: v
                        .texture
                        .inner
                        .iter()
                        .map(|vv| self.game_string_table_ru.get_o(vv))
                        .collect(),
                })
            }

            let drop_info = ItemDropInfo {
                drop_type: DropType::from_u8(item.drop_type).unwrap(),
                drop_animation_type: DropAnimationType::from_u8(item.drop_animation_type).unwrap(),
                drop_radius: item.drop_radius,
                drop_height: item.drop_height,
                drop_mesh_info,
                complete_item_drop_sound: self
                    .game_string_table_ru
                    .get_o(&item.complete_item_drop_sound),
                drop_sound: self.game_string_table_ru.get_o(&item.drop_sound),
            };

            self.etc_item_holder.insert(
                item.id.into(),
                EtcItem {
                    base_info: ItemBaseInfo {
                        id: item.id.into(),
                        name: (
                            self.game_string_table_ru.get_o(&name_grp_ru.name_link),
                            self.game_string_table_eu.get_o(&name_grp_eu.name_link),
                        )
                            .into(),
                        additional_name: (
                            name_grp_ru.additional_name.to_string(),
                            name_grp_eu.additional_name.to_string(),
                        )
                            .into(),
                        popup: name_grp_ru.popup,
                        default_action: ItemDefaultAction::from_ascf(&name_grp_ru.default_action),
                        use_order: name_grp_ru.use_order,
                        set_id: name_grp_ru.set_id.into(),
                        color: ItemNameColor::from_u8(name_grp_ru.color).unwrap(),
                        tooltip_texture: self
                            .game_string_table_ru
                            .get_o(&name_grp_ru.tooltip_texture_link),
                        is_trade: name_grp_ru.is_trade == 1,
                        is_drop: name_grp_ru.is_drop == 1,
                        is_destruct: name_grp_ru.is_destruct == 1,
                        is_private_store: name_grp_ru.is_private_store == 1,
                        is_npc_trade: name_grp_ru.is_npc_trade == 1,
                        is_commission_store: name_grp_ru.is_commission_store == 1,
                        keep_type: KeepType::from_u8(name_grp_ru.keep_type).unwrap(),
                        desc: (
                            name_grp_ru.description.to_string(),
                            name_grp_eu.description.to_string(),
                        )
                            .into(),
                        inventory_type: InventoryType::from_u8(item.inventory_type).unwrap(),
                        material: ItemMaterial::from_u8(item.material_type).unwrap(),
                        body_part: BodyPart::None,
                        quality: ItemQuality::from_u8(item.color).unwrap(),
                        crystallizable: item.crystallizable == 1,
                        crystal_type: CrystalType::from_u8(item.crystal_type).unwrap(),
                        durability: item.durability,
                        weight: item.weight,
                        icons: WindowParams::new(ItemIcons {
                            icon_1: self.game_string_table_ru.get_o(&item.icon_1),
                            icon_2: self.game_string_table_ru.get_o(&item.icon_2),
                            icon_3: self.game_string_table_ru.get_o(&item.icon_3),
                            icon_4: self.game_string_table_ru.get_o(&item.icon_4),
                            icon_5: self.game_string_table_ru.get_o(&item.icon_5),
                            icon_panel: self.game_string_table_ru.get_o(&item.icon_panel),
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
                        equip_sound: self.game_string_table_ru.get_o(&item.equip_sound),
                        additional_info: WindowParams::new(ItemAdditionalInfo {
                            has_animation: add_info_grp.has_ani == 1,
                            include_items: add_info_grp
                                .included_items
                                .iter()
                                .map(|v| (*v).into())
                                .collect(),
                            max_energy: add_info_grp.max_energy,
                            look_change: self.game_string_table_ru.get_o(&add_info_grp.look_change),
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
                    etc_item_type: EtcItemType::from_u32(item.etc_item_type)
                        .unwrap_or_else(|| panic!("unknown etc item type {}", item.etc_item_type)),
                    consume_type: ConsumeType::from_u8(item.consume_type).unwrap(),
                    ensoul_stone: stones.remove(&item.id).map(|v| EnsoulStone {
                        slot_type: EnsoulSlotType::from_u32(v.slot_type).unwrap(),
                        options: v
                            .ensoul_options
                            .iter()
                            .map(|v| EnsoulOptionId(*v))
                            .collect(),
                    }),
                    mesh_info,
                    ..Default::default()
                },
            );
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct EtcItemGrpDat {
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
    mesh: UVEC<BYTE, DWORD>,                //+
    texture: UVEC<BYTE, DWORD>,             //+
    drop_sound: DWORD,                      //+
    equip_sound: DWORD,                     //+
    consume_type: BYTE,                     //+
    etc_item_type: DWORD,                   //+
    crystal_type: BYTE,                     //+
}
impl GetId for EtcItemGrpDat {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct EnsoulStoneDat {
    id: DWORD,
    slot_type: DWORD,
    ensoul_options: Vec<DWORD>,
}
impl GetId for EnsoulStoneDat {
    fn get_id(&self) -> u32 {
        self.id
    }
}
