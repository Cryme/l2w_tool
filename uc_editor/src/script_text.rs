#![allow(dead_code)]

use std::io::{Read, Seek};
use l2_rw::ue2_rw::{UnrealReader, INDEX};

#[derive(Clone, Debug)]
pub struct UScriptText {
    pos: u32,
    top: u32,
    size: u32,
    text: String,
}


impl UScriptText {
    pub(crate) fn parse<
        T: Read+Seek,
        F1: Fn(INDEX) -> String,
        F2: Fn(INDEX) -> String,
    >(reader: &mut T, _object_name_resolver: &F1, _name_resolver: &F2) -> Self {
        let pos = reader.read_unreal_value::<u32>();
        let top = reader.read_unreal_value::<u32>();
        let size = reader.read_unreal_value::<u16>();

        let mut text_bytes = vec![];

        reader.read_to_end(&mut text_bytes).unwrap();

        let text = String::from_utf8_lossy(&text_bytes).to_string();

        Self {
            pos,
            top,
            size: size as u32,
            text
        }

    }
}
