use crate::script_decompiler::CastToken;
use l2_rw::ue2_rw::INDEX;
use std::fmt::{Display};

pub enum Token {
    NativeFunction {
        index: u16,
    },

    // Casts
    PrimitiveCast {
        to: CastToken,
        from: Box<Token>,
    },
    DynamicCast {
        from: INDEX,
        to: Box<Token>,
    },
    MetaCast {
        from: INDEX,
        to: Box<Token>,
    },
    InterfaceCast,
    // --------------------------------------------

    // Context
    Context {
        start: Box<Token>,
        size: u16,
        property_type: u8,
        end: Box<Token>,
    },
    ClassContext {
        start: Box<Token>,
        size: u16,
        property_type: u8,
        end: Box<Token>,
    },
    InterfaceContext {
        start: Box<Token>,
    },
    StructMember {
        meta_super_field: INDEX,
        meta_next: INDEX,
        member: Box<Token>,
    },
    // --------------------------------------------

    // Assigns
    //A = B
    Let {
        left: Box<Token>,
        right: Box<Token>,
    },
    LetBool {
        left: Box<Token>,
        right: Box<Token>,
    },
    LetDelegate {
        left: Box<Token>,
        right: Box<Token>,
    },
    EndParmValue,
    Conditional {
        // Condition
        condition: Box<Token>,
        // Size. Used to skip ? if Condition is False.
        size_t: u16,
        // If TRUE expression
        if_true: Box<Token>,
        // Size. Used to skip : if Condition is True.
        size_f: u16,
        // If FALSE expression
        if_false: Box<Token>,
    },
    // --------------------------------------------


    // Jumps
    Return {
        value: Box<Token>,
    },
    ReturnVoid,
    GoToLabel {
        label: Box<Token>,
    },
    Jump {
        offset: u16,
    },
    JumpIfNot {
        offset: u16,
        condition: Box<Token>,
    },
    Switch {
        expression: Box<Token>,
    },
    Case {
        offset: u16,
        condition: Option<Box<Token>>,
    },
    Iterator {
        expression: Box<Token>,
        offset: u16,
    },
    ArrayIterator {
        expression: Box<Token>,
        item: Box<Token>,
        with_index_param: u8,
        index: Box<Token>,
    },
    IteratorNext,
    IteratorPop,
    
    // --------------------------------------------

    // Other
    BeginFunction {
        //TODO:
    },
    // --------------------------------------------

    // Context

    // --------------------------------------------

    Array(ArrayToken),

    EndFunctionParams,

    Dummy,
}

#[derive(Debug)]
pub enum ArrayToken {
    ArrayElementToken,
    DynamicArrayElementToken,
    DynamicArrayLengthToken,
    DynamicArrayMethodToken,
    DynamicArrayFindToken,
    DynamicArrayFindStructToken,
    DynamicArraySortToken,
    DynamicArrayAddToken,
    DynamicArrayAddItemToken,
    DynamicArrayInsertToken,
    DynamicArrayInsertItemToken,
    DynamicArrayRemoveToken,
    DynamicArrayRemoveItemToken,
}
