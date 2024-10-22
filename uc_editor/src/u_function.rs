#![allow(dead_code)]

use crate::u_struct::UStruct;
use l2_rw::ue2_rw::{UnrealReader, INDEX};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
use std::io::{Read, Seek, SeekFrom};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Clone, Debug)]
pub struct UFunction {
    pub u_struct: UStruct,
    pub native_index: u16,
    pub operator_precedence: u8,
    pub flags: u32,
    pub replication_offset: Option<u16>,
}

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
enum FunctionFlags {
    ///Function is final (prebindable, non-overridable function).
    Final = 0x00000001,
    /// Function has been defined (not just declared). Not used in source code.
    Defined = 0x00000002,
    /// Function is an iterator.
    Iterator = 0x00000004,
    /// Function is a latent state function.
    Latent = 0x00000008,
    /// Unary operator is a prefix operator.
    PreOperator = 0x00000010,
    /// Function cannot be reentered.
    Singular = 0x00000020,
    /// Function is network-replicated. Not used in source code.
    Net = 0x00000040,
    /// Function should be sent reliably on the network. Not used in source code.
    NetReliable = 0x00000080,
    /// Function executed on the client side.
    Simulated = 0x00000100,
    /// Executable from command line.
    Exec = 0x00000200,
    /// Native function.
    Native = 0x00000400,
    /// Event function.
    Event = 0x00000800,
    /// Operator function.
    Operator = 0x00001000,
    /// Static function.
    Static = 0x00002000,
    /// Don't export intrinsic function to C++.
    NoExport = 0x00004000,
    /// Function doesn't modify this object.
    Const = 0x00008000,
    /// Return value is purely dependent on parameters;
    Invariant = 0x00010000,
}

impl FunctionFlags {
    fn is(&self, v: u32) -> bool {
        self.to_u32().unwrap() & v > 0
    }
}

impl UFunction {
    pub(crate) fn parse<T: Read + Seek, F1: Fn(INDEX) -> String, F2: Fn(INDEX) -> String>(
        reader: &mut T,
        object_name_resolver: &F1,
        name_resolver: &F2,
        data_size: usize,
    ) -> Self {
        const COMMON_L2_FUNCTION_META_SIZE: usize = 7;

        let u_struct = UStruct::parse(
            reader,
            object_name_resolver,
            name_resolver,
            data_size - COMMON_L2_FUNCTION_META_SIZE,
        );

        let left = data_size as u64 - reader.stream_position().unwrap();

        reader.seek(SeekFrom::Current(left as i64 - 7)).unwrap();

        let native_index = reader.read_unreal_value::<u16>();
        let operator_precedence = reader.read_unreal_value::<u8>();
        let flags = reader.read_unreal_value::<u32>();

        // for f in FunctionFlags::iter() {
        //     if f.is(flags) {
        //         println!("{f:?}")
        //     }
        // }

        let replication_offset = if FunctionFlags::Net.is(flags) {
            Some(reader.read_unreal_value::<u16>())
        } else {
            None
        };

        Self {
            u_struct,
            native_index,
            operator_precedence,
            flags,
            replication_offset,
        }
    }
}
