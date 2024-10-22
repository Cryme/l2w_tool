#![allow(dead_code)]

use crate::script_decompiler::BytecodeDecompiler;
use l2_rw::ue2_rw::ReadUnreal;
use l2_rw::ue2_rw::{UnrealReader, DWORD, INDEX};
use r#macro::ReadUnreal;
use std::io::{Read, Seek};

#[derive(Debug, Clone, ReadUnreal)]
struct UStructB {
    ///Parent object. Object Reference.
    super_class: INDEX,
    ///Next object in list. Object Reference.
    next_field: INDEX,

    ///Object Reference.
    script_text: INDEX,
    ///First object inside the struct. Object Reference.
    children: INDEX,
    ///Name of the struct. Name Reference.
    friendly_name: INDEX,
    cpp_text: INDEX,

    line: DWORD,
    text_pos: DWORD,
    script_size: DWORD,
}

#[derive(Clone, Debug)]
pub struct UStruct {
    ///Parent object. Object Reference.
    pub super_class: INDEX,
    ///Next object in list. Object Reference.
    pub next_field: INDEX,
    ///Object Reference.
    pub script_text: INDEX,
    ///First object inside the struct. Object Reference.
    pub children: INDEX,
    ///Name of the struct. Name Reference.
    pub friendly_name: INDEX,

    pub line: u32,
    pub text_pos: u32,
    pub script_size: u32,

    pub script_bytes: Vec<u8>,
}

impl UStruct {
    pub(crate) fn parse<T: Read + Seek, F1: Fn(INDEX) -> String, F2: Fn(INDEX) -> String>(
        reader: &mut T,
        object_name_resolver: &F1,
        name_resolver: &F2,
        data_size: usize,
    ) -> Self {
        let _props_count = reader.read_unreal_value::<INDEX>();

        let bin = reader.read_unreal_value::<UStructB>();

        let mut script_decompiler = BytecodeDecompiler::new(reader, bin.script_size as usize);

        let script = script_decompiler.decompile();

        //По какой-то причине script_size превышает реальный размер, так что тут костыль.
        //До конца не считываем, потому что дальше могут быть данные верхней структуры, много кто наследует от Struct
        let mut script = vec![
            0u8;
            (bin.script_size as usize)
                .min(data_size - reader.stream_position().unwrap() as usize)
        ];

        reader.read_exact(&mut script).unwrap();

        Self {
            super_class: bin.super_class,
            next_field: bin.next_field,
            script_text: bin.script_text,
            children: bin.children,
            friendly_name: bin.friendly_name,
            line: bin.line,
            text_pos: bin.text_pos,
            script_size: bin.script_size,
            script_bytes: script,
        }
    }
}
