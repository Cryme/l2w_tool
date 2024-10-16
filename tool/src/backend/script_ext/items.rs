#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]
use crate::entity::item::armor::{
    Armor, ArmorMeshAdditional, ArmorMeshAdditionalF, ArmorMeshBase, ArmorMeshInfo, ArmorMeshes,
    ArmorType, UnderwaterBodyType1, UnderwaterBodyType2,
};
use crate::entity::item::weapon::Weapon;
use crate::entity::item::{
    BodyPart, CrystalType, DropAnimationType, DropType, InventoryType, ItemAdditionalInfo,
    ItemBaseInfo, ItemBattleStats, ItemDefaultAction, ItemDropInfo, ItemDropMeshInfo, ItemIcons,
    ItemMaterial, ItemNameColor, ItemQuality, KeepType,
};
use rhai::plugin::*;
use rhai::Engine;
use strum::IntoEnumIterator;
use crate::common::ItemId;

pub fn reg(engine: &mut Engine) {
    engine.build_type::<Armor>();
    engine.build_type::<ArmorMeshes>();
    engine.build_type::<ArmorMeshBase>();
    engine.build_type::<ArmorMeshAdditionalF>();
    engine.build_type::<ArmorMeshAdditional>();
    engine.build_type::<ArmorMeshInfo>();

    engine.build_type::<ItemId>();
    engine.build_type::<ItemBaseInfo>();
    engine.build_type::<ItemAdditionalInfo>();
    engine.build_type::<ItemIcons>();
    engine.build_type::<ItemDropInfo>();
    engine.build_type::<ItemDropMeshInfo>();
    engine.build_type::<ItemBattleStats>();

    engine.build_type::<Weapon>();

    engine
        .register_type_with_name::<ItemMaterial>("ItemMaterial")
        .register_static_module("ItemMaterial", exported_module!(ItemMaterialModule).into());
    engine
        .register_type_with_name::<InventoryType>("InventoryType")
        .register_static_module(
            "InventoryType",
            exported_module!(InventoryTypeModule).into(),
        );
    engine
        .register_type_with_name::<BodyPart>("BodyPart")
        .register_static_module("BodyPart", exported_module!(BodyPartModule).into());
    engine
        .register_type_with_name::<CrystalType>("CrystalType")
        .register_static_module("CrystalType", exported_module!(CrystalTypeModule).into());
    engine
        .register_type_with_name::<DropType>("DropType")
        .register_static_module("DropType", exported_module!(DropTypeModule).into());
    engine
        .register_type_with_name::<KeepType>("KeepType")
        .register_static_module("KeepType", exported_module!(KeepTypeModule).into());
    engine
        .register_type_with_name::<DropAnimationType>("DropAnimationType")
        .register_static_module(
            "DropAnimationType",
            exported_module!(DropAnimationTypeModule).into(),
        );
    engine
        .register_type_with_name::<ItemNameColor>("ItemNameColor")
        .register_static_module(
            "ItemNameColor",
            exported_module!(ItemNameColorModule).into(),
        );
    engine
        .register_type_with_name::<ItemQuality>("ItemQuality")
        .register_static_module("ItemQuality", exported_module!(ItemQualityModule).into());
    engine
        .register_type_with_name::<ItemDefaultAction>("ItemDefaultAction")
        .register_static_module(
            "ItemDefaultAction",
            exported_module!(ItemDefaultActionModule).into(),
        );
    engine
        .register_type_with_name::<ArmorMeshBase>("ArmorMeshBase")
        .register_static_module(
            "ArmorMeshBase",
            exported_module!(ArmorMeshBaseModule).into(),
        );
    engine
        .register_type_with_name::<ArmorMeshAdditionalF>("ArmorMeshAdditionalF")
        .register_static_module(
            "ArmorMeshAdditionalF",
            exported_module!(ArmorMeshAdditionalFModule).into(),
        );
    engine
        .register_type_with_name::<ArmorMeshAdditional>("ArmorMeshAdditional")
        .register_static_module(
            "ArmorMeshAdditional",
            exported_module!(ArmorMeshAdditionalModule).into(),
        );
    engine
        .register_type_with_name::<ArmorMeshInfo>("ArmorMeshInfo")
        .register_static_module(
            "ArmorMeshInfo",
            exported_module!(ArmorMeshInfoModule).into(),
        );
    engine
        .register_type_with_name::<ArmorType>("ArmorType")
        .register_static_module("ArmorType", exported_module!(ArmorTypeModule).into());
    engine
        .register_type_with_name::<UnderwaterBodyType1>("UnderwaterBodyType1")
        .register_static_module(
            "UnderwaterBodyType1",
            exported_module!(UnderwaterBodyType1Module).into(),
        );
    engine
        .register_type_with_name::<UnderwaterBodyType2>("UnderwaterBodyType2")
        .register_static_module(
            "UnderwaterBodyType2",
            exported_module!(UnderwaterBodyType2Module).into(),
        );
}

#[export_module]
mod UnderwaterBodyType2Module {
    pub const Unk0: UnderwaterBodyType2 = UnderwaterBodyType2::Unk0;
    pub const Unk1: UnderwaterBodyType2 = UnderwaterBodyType2::Unk1;
    pub const Unk2: UnderwaterBodyType2 = UnderwaterBodyType2::Unk2;
    pub const Unk3: UnderwaterBodyType2 = UnderwaterBodyType2::Unk3;
    pub const Unk4: UnderwaterBodyType2 = UnderwaterBodyType2::Unk4;
    pub const Unk5: UnderwaterBodyType2 = UnderwaterBodyType2::Unk5;

    #[rhai_fn(global, get = "underwater_body_type_2", pure)]
    pub fn get_type(underwater_body_type_2: &mut UnderwaterBodyType2) -> String {
        underwater_body_type_2.to_string()
    }

    pub fn all_variants() -> Vec<UnderwaterBodyType2> {
        UnderwaterBodyType2::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(underwater_body_type_2: &mut UnderwaterBodyType2) -> String {
        format!("{underwater_body_type_2:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(
        underwater_body_type_2: &mut UnderwaterBodyType2,
        other: UnderwaterBodyType2,
    ) -> bool {
        underwater_body_type_2 == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(
        underwater_body_type_2: &mut UnderwaterBodyType2,
        other: UnderwaterBodyType2,
    ) -> bool {
        underwater_body_type_2 != &other
    }

    use crate::entity::item::armor::UnderwaterBodyType2;
}

#[export_module]
mod UnderwaterBodyType1Module {
    pub const Unk0: UnderwaterBodyType1 = UnderwaterBodyType1::Unk0;
    pub const Unk1: UnderwaterBodyType1 = UnderwaterBodyType1::Unk1;
    pub const Unk2: UnderwaterBodyType1 = UnderwaterBodyType1::Unk2;
    pub const Unk3: UnderwaterBodyType1 = UnderwaterBodyType1::Unk3;
    pub const Unk4: UnderwaterBodyType1 = UnderwaterBodyType1::Unk4;
    pub const Unk5: UnderwaterBodyType1 = UnderwaterBodyType1::Unk5;

    #[rhai_fn(global, get = "underwater_body_type_1", pure)]
    pub fn get_type(underwater_body_type_1: &mut UnderwaterBodyType1) -> String {
        underwater_body_type_1.to_string()
    }

    pub fn all_variants() -> Vec<UnderwaterBodyType1> {
        UnderwaterBodyType1::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(underwater_body_type_1: &mut UnderwaterBodyType1) -> String {
        format!("{underwater_body_type_1:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(
        underwater_body_type_1: &mut UnderwaterBodyType1,
        other: UnderwaterBodyType1,
    ) -> bool {
        underwater_body_type_1 == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(
        underwater_body_type_1: &mut UnderwaterBodyType1,
        other: UnderwaterBodyType1,
    ) -> bool {
        underwater_body_type_1 != &other
    }
    use crate::entity::item::armor::UnderwaterBodyType1;
}

#[export_module]
mod ArmorTypeModule {
    pub const Unk0: ArmorType = ArmorType::Unk0;
    pub const Unk1: ArmorType = ArmorType::Unk1;
    pub const Unk2: ArmorType = ArmorType::Unk2;
    pub const Unk3: ArmorType = ArmorType::Unk3;
    pub const Unk4: ArmorType = ArmorType::Unk4;

    #[rhai_fn(global, get = "armor_type", pure)]
    pub fn get_type(armor_type: &mut ArmorType) -> String {
        armor_type.to_string()
    }

    pub fn all_variants() -> Vec<ArmorType> {
        ArmorType::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(armor_type: &mut ArmorType) -> String {
        format!("{armor_type:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(armor_type: &mut ArmorType, other: ArmorType) -> bool {
        armor_type == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(armor_type: &mut ArmorType, other: ArmorType) -> bool {
        armor_type != &other
    }

    use crate::entity::item::armor::ArmorType;
}

#[export_module]
mod ArmorMeshInfoModule {
    pub fn new_armor_mesh_info(
        base: ArmorMeshBase,
        additional: ArmorMeshAdditional,
    ) -> ArmorMeshInfo {
        ArmorMeshInfo { base, additional }
    }

    #[rhai_fn(global, get = "base")]
    pub fn get_base(armor_mesh_info: &mut ArmorMeshInfo) -> ArmorMeshBase {
        armor_mesh_info.base.clone()
    }

    #[rhai_fn(global, set = "base")]
    pub fn set_base(armor_mesh_info: &mut ArmorMeshInfo, value: ArmorMeshBase) {
        armor_mesh_info.base = value;
    }

    #[rhai_fn(global, get = "additional")]
    pub fn get_additional(armor_mesh_info: &mut ArmorMeshInfo) -> ArmorMeshAdditional {
        armor_mesh_info.additional.clone()
    }

    #[rhai_fn(global, set = "additional")]
    pub fn set_additional(armor_mesh_info: &mut ArmorMeshInfo, value: ArmorMeshAdditional) {
        armor_mesh_info.additional = value;
    }

    #[rhai_fn(global, name = "to_string", pure)]
    pub fn to_string(armor_mesh_info: &mut ArmorMeshInfo) -> String {
        format!("{armor_mesh_info:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(armor_mesh_info: &mut ArmorMeshInfo, other: ArmorMeshInfo) -> bool {
        armor_mesh_info == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(armor_mesh_info: &mut ArmorMeshInfo, other: ArmorMeshInfo) -> bool {
        armor_mesh_info != &other
    }
}

#[export_module]
mod ArmorMeshAdditionalModule {
    pub fn new_armor_mesh_additional(
        unk1: Vec<ArmorMeshAdditionalF>,
        unk5: Vec<String>,
        unk6: String,
    ) -> ArmorMeshAdditional {
        ArmorMeshAdditional { unk1, unk5, unk6 }
    }

    #[rhai_fn(global, get = "unk1")]
    pub fn get_unk1(armor_mesh_additional: &mut ArmorMeshAdditional) -> Vec<ArmorMeshAdditionalF> {
        armor_mesh_additional.unk1.clone()
    }

    #[rhai_fn(global, set = "unk1")]
    pub fn set_unk1(
        armor_mesh_additional: &mut ArmorMeshAdditional,
        value: Vec<ArmorMeshAdditionalF>,
    ) {
        armor_mesh_additional.unk1 = value;
    }

    #[rhai_fn(global, get = "unk5")]
    pub fn get_unk5(armor_mesh_additional: &mut ArmorMeshAdditional) -> Vec<String> {
        armor_mesh_additional.unk5.clone()
    }

    #[rhai_fn(global, set = "unk5")]
    pub fn set_unk5(armor_mesh_additional: &mut ArmorMeshAdditional, value: Vec<String>) {
        armor_mesh_additional.unk5 = value;
    }

    #[rhai_fn(global, get = "unk6")]
    pub fn get_unk6(armor_mesh_additional: &mut ArmorMeshAdditional) -> String {
        armor_mesh_additional.unk6.clone()
    }

    #[rhai_fn(global, set = "unk6")]
    pub fn set_unk6(armor_mesh_additional: &mut ArmorMeshAdditional, value: String) {
        armor_mesh_additional.unk6 = value;
    }

    #[rhai_fn(global, name = "to_string", pure)]
    pub fn to_string(armor_mesh_additional: &mut ArmorMeshAdditional) -> String {
        format!("{armor_mesh_additional:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(armor_mesh_additional: &mut ArmorMeshAdditional, other: ArmorMeshAdditional) -> bool {
        armor_mesh_additional == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(
        armor_mesh_additional: &mut ArmorMeshAdditional,
        other: ArmorMeshAdditional,
    ) -> bool {
        armor_mesh_additional != &other
    }
}

#[export_module]
mod ArmorMeshAdditionalFModule {
    pub fn new_armor_mesh_additional_f(unk2: String, unk3: u8, unk4: u8) -> ArmorMeshAdditionalF {
        ArmorMeshAdditionalF { unk2, unk3, unk4 }
    }

    #[rhai_fn(global, get = "unk2")]
    pub fn get_unk2(armor_mesh_additional_f: &mut ArmorMeshAdditionalF) -> String {
        armor_mesh_additional_f.unk2.clone()
    }

    #[rhai_fn(global, set = "unk2")]
    pub fn set_unk2(armor_mesh_additional_f: &mut ArmorMeshAdditionalF, value: String) {
        armor_mesh_additional_f.unk2 = value;
    }

    #[rhai_fn(global, get = "unk3")]
    pub fn get_unk3(armor_mesh_additional_f: &mut ArmorMeshAdditionalF) -> u8 {
        armor_mesh_additional_f.unk3
    }

    #[rhai_fn(global, set = "unk3")]
    pub fn set_unk3(armor_mesh_additional_f: &mut ArmorMeshAdditionalF, value: u8) {
        armor_mesh_additional_f.unk3 = value;
    }

    #[rhai_fn(global, get = "unk4")]
    pub fn get_unk4(armor_mesh_additional_f: &mut ArmorMeshAdditionalF) -> u8 {
        armor_mesh_additional_f.unk4
    }

    #[rhai_fn(global, set = "unk4")]
    pub fn set_unk4(armor_mesh_additional_f: &mut ArmorMeshAdditionalF, value: u8) {
        armor_mesh_additional_f.unk4 = value;
    }

    #[rhai_fn(global, name = "to_string", pure)]
    pub fn to_string(armor_mesh_additional_f: &mut ArmorMeshAdditionalF) -> String {
        format!("{armor_mesh_additional_f:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(
        armor_mesh_additional_f: &mut ArmorMeshAdditionalF,
        other: ArmorMeshAdditionalF,
    ) -> bool {
        armor_mesh_additional_f == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(
        armor_mesh_additional_f: &mut ArmorMeshAdditionalF,
        other: ArmorMeshAdditionalF,
    ) -> bool {
        armor_mesh_additional_f != &other
    }
}

#[export_module]
mod ArmorMeshBaseModule {
    pub fn new_armor_mesh_base(unk1: Vec<String>, unk2: Vec<String>) -> ArmorMeshBase {
        ArmorMeshBase { unk1, unk2 }
    }

    #[rhai_fn(global, get = "unk1")]
    pub fn get_unk1(armor_mesh_base: &mut ArmorMeshBase) -> Vec<String> {
        armor_mesh_base.unk1.clone()
    }

    #[rhai_fn(global, set = "unk1")]
    pub fn set_unk1(armor_mesh_base: &mut ArmorMeshBase, value: Vec<String>) {
        armor_mesh_base.unk1 = value;
    }

    #[rhai_fn(global, get = "unk2")]
    pub fn get_unk2(armor_mesh_base: &mut ArmorMeshBase) -> Vec<String> {
        armor_mesh_base.unk2.clone()
    }

    #[rhai_fn(global, set = "unk2")]
    pub fn set_unk2(armor_mesh_base: &mut ArmorMeshBase, value: Vec<String>) {
        armor_mesh_base.unk2 = value;
    }

    #[rhai_fn(global, name = "to_string", pure)]
    pub fn to_string(armor_mesh_base: &mut ArmorMeshBase) -> String {
        format!("{armor_mesh_base:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(armor_mesh_base: &mut ArmorMeshBase, other: ArmorMeshBase) -> bool {
        armor_mesh_base == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(armor_mesh_base: &mut ArmorMeshBase, other: ArmorMeshBase) -> bool {
        armor_mesh_base != &other
    }
}

#[export_module]
mod InventoryTypeModule {
    // Constructors for 'InventoryType' variants

    /// `InventoryType::None`
    pub const None: InventoryType = InventoryType::None;

    /// `InventoryType::Equipment`
    pub const Equipment: InventoryType = InventoryType::Equipment;

    /// `InventoryType::Consumable`
    pub const Consumable: InventoryType = InventoryType::Consumable;

    /// `InventoryType::Material`
    pub const Material: InventoryType = InventoryType::Material;

    /// `InventoryType::Etc`
    pub const Etc: InventoryType = InventoryType::Etc;

    /// `InventoryType::Quest`
    pub const Quest: InventoryType = InventoryType::Quest;

    // Get the current variant of `InventoryType`.
    #[rhai_fn(global, get = "inventory_type", pure)]
    pub fn get_type(inventory: &mut InventoryType) -> String {
        inventory.to_string()
    }

    // Return all variants of `InventoryType`.
    pub fn all_variants() -> Vec<InventoryType> {
        InventoryType::iter().collect()
    }

    // Printing
    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(inventory: &mut InventoryType) -> String {
        format!("{inventory:?}")
    }

    // '==' and '!=' operators
    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(inventory: &mut InventoryType, other: InventoryType) -> bool {
        inventory == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(inventory: &mut InventoryType, other: InventoryType) -> bool {
        inventory != &other
    }

    use crate::entity::item::InventoryType;
    use strum::IntoEnumIterator;
}

#[export_module]
mod ItemMaterialModule {
    pub const None: ItemMaterial = ItemMaterial::None;
    pub const Scroll: ItemMaterial = ItemMaterial::Scroll;
    pub const Arrow: ItemMaterial = ItemMaterial::Arrow;
    pub const Potion: ItemMaterial = ItemMaterial::Potion;
    pub const Spellbook: ItemMaterial = ItemMaterial::Spellbook;
    pub const Recipe: ItemMaterial = ItemMaterial::Recipe;
    pub const Material: ItemMaterial = ItemMaterial::Material;
    pub const PetCollar: ItemMaterial = ItemMaterial::PetCollar;
    pub const CastleGuard: ItemMaterial = ItemMaterial::CastleGuard;
    pub const Dye: ItemMaterial = ItemMaterial::Dye;
    pub const Seed: ItemMaterial = ItemMaterial::Seed;
    pub const Seed2: ItemMaterial = ItemMaterial::Seed2;
    pub const Harvest: ItemMaterial = ItemMaterial::Harvest;
    pub const Lotto: ItemMaterial = ItemMaterial::Lotto;
    pub const RaceTicket: ItemMaterial = ItemMaterial::RaceTicket;
    pub const TicketOfLord: ItemMaterial = ItemMaterial::TicketOfLord;
    pub const Lure: ItemMaterial = ItemMaterial::Lure;
    pub const Crop: ItemMaterial = ItemMaterial::Crop;
    pub const Maturecrop: ItemMaterial = ItemMaterial::Maturecrop;
    pub const EnchtWp: ItemMaterial = ItemMaterial::EnchtWp;
    pub const EnchtAm: ItemMaterial = ItemMaterial::EnchtAm;
    pub const BlessEnchtWp: ItemMaterial = ItemMaterial::BlessEnchtWp;
    pub const BlessEnchtAm: ItemMaterial = ItemMaterial::BlessEnchtAm;
    pub const Coupon: ItemMaterial = ItemMaterial::Coupon;
    pub const Elixir: ItemMaterial = ItemMaterial::Elixir;
    pub const EnchtAttr: ItemMaterial = ItemMaterial::EnchtAttr;
    pub const EnchtAttrCursed: ItemMaterial = ItemMaterial::EnchtAttrCursed;
    pub const Bolt: ItemMaterial = ItemMaterial::Bolt;
    pub const EnchtAttrIncPropEnchtWp: ItemMaterial = ItemMaterial::EnchtAttrIncPropEnchtWp;
    pub const EnchtAttrIncPropEnchtAm: ItemMaterial = ItemMaterial::EnchtAttrIncPropEnchtAm;
    pub const EnchtAttrCrystalEnchantAm: ItemMaterial = ItemMaterial::EnchtAttrCrystalEnchantAm;
    pub const EnchtAttrCrystalEnchantWp: ItemMaterial = ItemMaterial::EnchtAttrCrystalEnchantWp;
    pub const EnchtAttrAncientCrystalEnchantAm: ItemMaterial =
        ItemMaterial::EnchtAttrAncientCrystalEnchantAm;
    pub const EnchtAttrAncientCrystalEnchantWp: ItemMaterial =
        ItemMaterial::EnchtAttrAncientCrystalEnchantWp;
    pub const EnchtAttrRune: ItemMaterial = ItemMaterial::EnchtAttrRune;
    pub const EnchtAttrtRuneSelect: ItemMaterial = ItemMaterial::EnchtAttrtRuneSelect;
    pub const Teleportbookmark: ItemMaterial = ItemMaterial::Teleportbookmark;
    pub const ChangeAttr: ItemMaterial = ItemMaterial::ChangeAttr;
    pub const Soulshot: ItemMaterial = ItemMaterial::Soulshot;
    pub const ShapeShiftingWp: ItemMaterial = ItemMaterial::ShapeShiftingWp;
    pub const BlessShapeShiftingWp: ItemMaterial = ItemMaterial::BlessShapeShiftingWp;
    pub const ShapeShiftingWpFixed: ItemMaterial = ItemMaterial::ShapeShiftingWpFixed;
    pub const ShapeShiftingAm: ItemMaterial = ItemMaterial::ShapeShiftingAm;
    pub const BlessShapeShiftingAm: ItemMaterial = ItemMaterial::BlessShapeShiftingAm;
    pub const ShapeShiftingAmFixed: ItemMaterial = ItemMaterial::ShapeShiftingAmFixed;
    pub const ShapeShiftingHairacc: ItemMaterial = ItemMaterial::ShapeShiftingHairacc;
    pub const BlessShapeShiftingHairacc: ItemMaterial = ItemMaterial::BlessShapeShiftingHairacc;
    pub const ShapeShiftingHairaccFixed: ItemMaterial = ItemMaterial::ShapeShiftingHairaccFixed;
    pub const RestoreShapeShiftingWp: ItemMaterial = ItemMaterial::RestoreShapeShiftingWp;
    pub const RestoreShapeShiftingAm: ItemMaterial = ItemMaterial::RestoreShapeShiftingAm;
    pub const RestoreShapeShiftingHairacc: ItemMaterial = ItemMaterial::RestoreShapeShiftingHairacc;
    pub const RestoreShapeShiftingAllitem: ItemMaterial = ItemMaterial::RestoreShapeShiftingAllitem;
    pub const BlessIncPropEnchtWp: ItemMaterial = ItemMaterial::BlessIncPropEnchtWp;
    pub const BlessIncPropEnchtAm: ItemMaterial = ItemMaterial::BlessIncPropEnchtAm;
    pub const CardEvent: ItemMaterial = ItemMaterial::CardEvent;
    pub const ShapeShiftingAllitemFixed: ItemMaterial = ItemMaterial::ShapeShiftingAllitemFixed;
    pub const MultiEnchtWp: ItemMaterial = ItemMaterial::MultiEnchtWp;
    pub const MultiEnchtAm: ItemMaterial = ItemMaterial::MultiEnchtAm;
    pub const MultiIncProbEnchtWp: ItemMaterial = ItemMaterial::MultiIncProbEnchtWp;
    pub const MultiIncProbEnchtAm: ItemMaterial = ItemMaterial::MultiIncProbEnchtAm;
    pub const EnsoulStone: ItemMaterial = ItemMaterial::EnsoulStone;

    #[rhai_fn(global, get = "item_material", pure)]
    pub fn get_type(material: &mut ItemMaterial) -> String {
        material.to_string()
    }

    pub fn all_variants() -> Vec<ItemMaterial> {
        ItemMaterial::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(material: &mut ItemMaterial) -> String {
        format!("{material:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(material: &mut ItemMaterial, other: ItemMaterial) -> bool {
        material == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(material: &mut ItemMaterial, other: ItemMaterial) -> bool {
        material != &other
    }
    use crate::entity::item::ItemMaterial;
}

#[export_module]
mod BodyPartModule {
    pub const WolfWeapon: BodyPart = BodyPart::WolfWeapon;
    pub const Unk1: BodyPart = BodyPart::Unk1;
    pub const Unk2: BodyPart = BodyPart::Unk2;
    pub const Unk3: BodyPart = BodyPart::Unk3;
    pub const Unk4: BodyPart = BodyPart::Unk4;
    pub const Unk5: BodyPart = BodyPart::Unk5;
    pub const Unk6: BodyPart = BodyPart::Unk6;
    pub const TwoHandedWeapon: BodyPart = BodyPart::TwoHandedWeapon;
    pub const Unk8: BodyPart = BodyPart::Unk8;
    pub const Unk9: BodyPart = BodyPart::Unk9;
    pub const Unk10: BodyPart = BodyPart::Unk10;
    pub const Unk11: BodyPart = BodyPart::Unk11;
    pub const Unk12: BodyPart = BodyPart::Unk12;
    pub const Unk13: BodyPart = BodyPart::Unk13;
    pub const Unk14: BodyPart = BodyPart::Unk14;
    pub const Unk15: BodyPart = BodyPart::Unk15;
    pub const Unk16: BodyPart = BodyPart::Unk16;
    pub const Unk17: BodyPart = BodyPart::Unk17;
    pub const Unk18: BodyPart = BodyPart::Unk18;
    pub const Unk19: BodyPart = BodyPart::Unk19;
    pub const Unk20: BodyPart = BodyPart::Unk20;
    pub const Unk21: BodyPart = BodyPart::Unk21;
    pub const Unk22: BodyPart = BodyPart::Unk22;
    pub const Unk23: BodyPart = BodyPart::Unk23;
    pub const Unk24: BodyPart = BodyPart::Unk24;
    pub const Unk25: BodyPart = BodyPart::Unk25;
    pub const Unk26: BodyPart = BodyPart::Unk26;
    pub const Unk27: BodyPart = BodyPart::Unk27;
    pub const Unk28: BodyPart = BodyPart::Unk28;
    pub const Unk29: BodyPart = BodyPart::Unk29;
    pub const Unk30: BodyPart = BodyPart::Unk30;
    pub const Unk31: BodyPart = BodyPart::Unk31;
    pub const Unk32: BodyPart = BodyPart::Unk32;
    pub const Unk33: BodyPart = BodyPart::Unk33;
    pub const OneHandedWeapon: BodyPart = BodyPart::OneHandedWeapon;
    pub const Shield: BodyPart = BodyPart::Shield;
    pub const None: BodyPart = BodyPart::None;

    #[rhai_fn(global, get = "body_part", pure)]
    pub fn get_type(body_part: &mut BodyPart) -> String {
        body_part.to_string()
    }

    pub fn all_variants() -> Vec<BodyPart> {
        BodyPart::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(body_part: &mut BodyPart) -> String {
        format!("{body_part:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(body_part: &mut BodyPart, other: BodyPart) -> bool {
        body_part == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(body_part: &mut BodyPart, other: BodyPart) -> bool {
        body_part != &other
    }
    use crate::entity::item::BodyPart;
}

#[export_module]
mod CrystalTypeModule {
    pub const NG: CrystalType = CrystalType::NG;
    pub const D: CrystalType = CrystalType::D;
    pub const C: CrystalType = CrystalType::C;
    pub const B: CrystalType = CrystalType::B;
    pub const A: CrystalType = CrystalType::A;
    pub const S: CrystalType = CrystalType::S;
    pub const S80: CrystalType = CrystalType::S80;
    pub const S84: CrystalType = CrystalType::S84;
    pub const R: CrystalType = CrystalType::R;
    pub const R95: CrystalType = CrystalType::R95;
    pub const R99: CrystalType = CrystalType::R99;
    pub const NoRang: CrystalType = CrystalType::NoRang;

    #[rhai_fn(global, get = "crystal_type", pure)]
    pub fn get_type(crystal_type: &mut CrystalType) -> String {
        crystal_type.to_string()
    }

    pub fn all_variants() -> Vec<CrystalType> {
        CrystalType::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(crystal_type: &mut CrystalType) -> String {
        format!("{crystal_type:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(crystal_type: &mut CrystalType, other: CrystalType) -> bool {
        crystal_type == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(crystal_type: &mut CrystalType, other: CrystalType) -> bool {
        crystal_type != &other
    }
    use crate::entity::item::CrystalType;
}

#[export_module]
mod DropTypeModule {
    pub const Unk0: DropType = DropType::Unk0;
    pub const Unk1: DropType = DropType::Unk1;
    pub const Unk2: DropType = DropType::Unk2;
    pub const Unk3: DropType = DropType::Unk3;
    pub const Unk4: DropType = DropType::Unk4;
    pub const Unk5: DropType = DropType::Unk5;

    #[rhai_fn(global, get = "drop_type", pure)]
    pub fn get_type(drop_type: &mut DropType) -> String {
        drop_type.to_string()
    }

    pub fn all_variants() -> Vec<DropType> {
        DropType::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(drop_type: &mut DropType) -> String {
        format!("{drop_type:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(drop_type: &mut DropType, other: DropType) -> bool {
        drop_type == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(drop_type: &mut DropType, other: DropType) -> bool {
        drop_type != &other
    }

    use crate::entity::item::DropType;
}

#[export_module]
mod KeepTypeModule {
    pub const Unk0: KeepType = KeepType::Unk0;
    pub const Unk1: KeepType = KeepType::Unk1;
    pub const Unk2: KeepType = KeepType::Unk2;
    pub const Unk3: KeepType = KeepType::Unk3;
    pub const Unk4: KeepType = KeepType::Unk4;
    pub const Unk5: KeepType = KeepType::Unk5;
    pub const Unk6: KeepType = KeepType::Unk6;
    pub const Unk7: KeepType = KeepType::Unk7;
    pub const Unk8: KeepType = KeepType::Unk8;
    pub const Unk9: KeepType = KeepType::Unk9;
    pub const Unk10: KeepType = KeepType::Unk10;
    pub const Unk11: KeepType = KeepType::Unk11;
    pub const Unk12: KeepType = KeepType::Unk12;
    pub const Unk13: KeepType = KeepType::Unk13;
    pub const Unk14: KeepType = KeepType::Unk14;
    pub const Unk15: KeepType = KeepType::Unk15;

    #[rhai_fn(global, get = "keep_type", pure)]
    pub fn get_type(keep_type: &mut KeepType) -> String {
        keep_type.to_string()
    }

    pub fn all_variants() -> Vec<KeepType> {
        KeepType::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(keep_type: &mut KeepType) -> String {
        format!("{keep_type:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(keep_type: &mut KeepType, other: KeepType) -> bool {
        keep_type == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(keep_type: &mut KeepType, other: KeepType) -> bool {
        keep_type != &other
    }
    use crate::entity::item::KeepType;
}

#[export_module]
mod DropAnimationTypeModule {
    pub const Unk0: DropAnimationType = DropAnimationType::Unk0;
    pub const Unk1: DropAnimationType = DropAnimationType::Unk1;
    pub const Unk2: DropAnimationType = DropAnimationType::Unk2;
    pub const Unk3: DropAnimationType = DropAnimationType::Unk3;
    pub const Unk4: DropAnimationType = DropAnimationType::Unk4;
    pub const Unk5: DropAnimationType = DropAnimationType::Unk5;

    #[rhai_fn(global, get = "drop_animation_type", pure)]
    pub fn get_type(drop_animation_type: &mut DropAnimationType) -> String {
        drop_animation_type.to_string()
    }

    pub fn all_variants() -> Vec<DropAnimationType> {
        DropAnimationType::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(drop_animation_type: &mut DropAnimationType) -> String {
        format!("{drop_animation_type:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(drop_animation_type: &mut DropAnimationType, other: DropAnimationType) -> bool {
        drop_animation_type == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(drop_animation_type: &mut DropAnimationType, other: DropAnimationType) -> bool {
        drop_animation_type != &other
    }

    use crate::entity::item::DropAnimationType;
}

#[export_module]
mod ItemNameColorModule {
    pub const Common: ItemNameColor = ItemNameColor::Common;
    pub const Normal: ItemNameColor = ItemNameColor::Normal;
    pub const Rare: ItemNameColor = ItemNameColor::Rare;
    pub const Epic: ItemNameColor = ItemNameColor::Epic;
    pub const Blessed: ItemNameColor = ItemNameColor::Blessed;
    pub const Dragon: ItemNameColor = ItemNameColor::Dragon;

    #[rhai_fn(global, get = "item_name_color", pure)]
    pub fn get_type(item_name_color: &mut ItemNameColor) -> String {
        item_name_color.to_string()
    }

    pub fn all_variants() -> Vec<ItemNameColor> {
        ItemNameColor::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(item_name_color: &mut ItemNameColor) -> String {
        format!("{item_name_color:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(item_name_color: &mut ItemNameColor, other: ItemNameColor) -> bool {
        item_name_color == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(item_name_color: &mut ItemNameColor, other: ItemNameColor) -> bool {
        item_name_color != &other
    }

    use crate::entity::item::ItemNameColor;
}

#[export_module]
mod ItemQualityModule {
    pub const Common: ItemQuality = ItemQuality::Common;
    pub const Normal: ItemQuality = ItemQuality::Normal;
    pub const Rare: ItemQuality = ItemQuality::Rare;
    pub const Epic: ItemQuality = ItemQuality::Epic;
    pub const Blessed: ItemQuality = ItemQuality::Blessed;
    pub const Dragon: ItemQuality = ItemQuality::Dragon;

    #[rhai_fn(global, get = "item_quality", pure)]
    pub fn get_type(item_quality: &mut ItemQuality) -> String {
        item_quality.to_string()
    }

    pub fn all_variants() -> Vec<ItemQuality> {
        ItemQuality::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(item_quality: &mut ItemQuality) -> String {
        format!("{item_quality:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(item_quality: &mut ItemQuality, other: ItemQuality) -> bool {
        item_quality == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(item_quality: &mut ItemQuality, other: ItemQuality) -> bool {
        item_quality != &other
    }

    use crate::entity::item::ItemQuality;
}

#[export_module]
mod ItemDefaultActionModule {
    pub const action_bless_spiritshot: ItemDefaultAction =
        ItemDefaultAction::action_bless_spiritshot;
    pub const action_calc: ItemDefaultAction = ItemDefaultAction::action_calc;
    pub const action_call_skill: ItemDefaultAction = ItemDefaultAction::action_call_skill;
    pub const action_capsule: ItemDefaultAction = ItemDefaultAction::action_capsule;
    pub const action_create_mpcc: ItemDefaultAction = ItemDefaultAction::action_create_mpcc;
    pub const action_dice: ItemDefaultAction = ItemDefaultAction::action_dice;
    pub const action_equip: ItemDefaultAction = ItemDefaultAction::action_equip;
    pub const action_fishingshot: ItemDefaultAction = ItemDefaultAction::action_fishingshot;
    pub const action_harvest: ItemDefaultAction = ItemDefaultAction::action_harvest;
    pub const action_hide_name: ItemDefaultAction = ItemDefaultAction::action_hide_name;
    pub const action_keep_exp: ItemDefaultAction = ItemDefaultAction::action_keep_exp;
    pub const action_nick_color: ItemDefaultAction = ItemDefaultAction::action_nick_color;
    pub const action_none: ItemDefaultAction = ItemDefaultAction::action_none;
    pub const action_peel: ItemDefaultAction = ItemDefaultAction::action_peel;
    pub const action_recipe: ItemDefaultAction = ItemDefaultAction::action_recipe;
    pub const action_seed: ItemDefaultAction = ItemDefaultAction::action_seed;
    pub const action_show_adventurer_guide_book: ItemDefaultAction =
        ItemDefaultAction::action_show_adventurer_guide_book;
    pub const action_show_html: ItemDefaultAction = ItemDefaultAction::action_show_html;
    pub const action_show_ssq_status: ItemDefaultAction = ItemDefaultAction::action_show_ssq_status;
    pub const action_show_tutorial: ItemDefaultAction = ItemDefaultAction::action_show_tutorial;
    pub const action_skill_maintain: ItemDefaultAction = ItemDefaultAction::action_skill_maintain;
    pub const action_skill_reduce: ItemDefaultAction = ItemDefaultAction::action_skill_reduce;
    pub const action_skill_reduce_on_skill_success: ItemDefaultAction =
        ItemDefaultAction::action_skill_reduce_on_skill_success;
    pub const action_soulshot: ItemDefaultAction = ItemDefaultAction::action_soulshot;
    pub const action_spiritshot: ItemDefaultAction = ItemDefaultAction::action_spiritshot;
    pub const action_start_quest: ItemDefaultAction = ItemDefaultAction::action_start_quest;
    pub const action_summon_soulshot: ItemDefaultAction = ItemDefaultAction::action_summon_soulshot;
    pub const action_summon_spiritshot: ItemDefaultAction =
        ItemDefaultAction::action_summon_spiritshot;
    pub const action_xmas_open: ItemDefaultAction = ItemDefaultAction::action_xmas_open;

    #[rhai_fn(global, get = "item_default_action", pure)]
    pub fn get_type(item_default_action: &mut ItemDefaultAction) -> String {
        item_default_action.to_string()
    }

    pub fn all_variants() -> Vec<ItemDefaultAction> {
        ItemDefaultAction::iter().collect()
    }

    #[rhai_fn(global, name = "to_string", name = "to_debug", pure)]
    pub fn to_string(item_default_action: &mut ItemDefaultAction) -> String {
        format!("{item_default_action:?}")
    }

    #[rhai_fn(global, name = "==", pure)]
    pub fn eq(item_default_action: &mut ItemDefaultAction, other: ItemDefaultAction) -> bool {
        item_default_action == &other
    }

    #[rhai_fn(global, name = "!=", pure)]
    pub fn neq(item_default_action: &mut ItemDefaultAction, other: ItemDefaultAction) -> bool {
        item_default_action != &other
    }

    use crate::entity::item::ItemDefaultAction;
}
