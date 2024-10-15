use rhai::plugin::*;
use rhai::{Engine};
use strum::IntoEnumIterator;
use crate::entity::item::{InventoryType, ItemAdditionalInfo, ItemBaseInfo, ItemBattleStats, ItemDropInfo, ItemDropMeshInfo, ItemIcons, ItemMaterial};
use crate::entity::item::armor::Armor;
use crate::entity::item::weapon::Weapon;

pub fn reg(engine: &mut Engine) {
    engine.build_type::<Armor>();
    engine.build_type::<ItemBaseInfo>();
    engine.build_type::<ItemAdditionalInfo>();
    engine.build_type::<ItemIcons>();
    engine.build_type::<ItemDropInfo>();
    engine.build_type::<ItemDropMeshInfo>();
    engine.build_type::<ItemBattleStats>();


    engine.build_type::<Weapon>();

    engine.register_type_with_name::<ItemMaterial>("ItemMaterial")
        .register_static_module("ItemMaterial", exported_module!(ItemMaterialModule).into());
    engine.register_type_with_name::<InventoryType>("InventoryType")
        .register_static_module("InventoryType", exported_module!(InventoryTypeModule).into());
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

    use strum::IntoEnumIterator;
    use crate::entity::item::InventoryType;
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
    pub const EnchtAttrAncientCrystalEnchantAm: ItemMaterial = ItemMaterial::EnchtAttrAncientCrystalEnchantAm;
    pub const EnchtAttrAncientCrystalEnchantWp: ItemMaterial = ItemMaterial::EnchtAttrAncientCrystalEnchantWp;
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
use crate::entity::item::ItemMaterial;}

