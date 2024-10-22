#![allow(dead_code)]

use l2_rw::ue2_rw::ReadUnreal;
use l2_rw::ue2_rw::UnrealReader;
use l2_rw::ue2_rw::{ASCF, DWORD, GUID, INDEX, INT, QWORD, WORD};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use r#macro::ReadUnreal;
use std::io::{Read, Seek, SeekFrom};
use strum::{Display, EnumIter};

#[derive(Debug, Clone, ReadUnreal)]
pub struct UClassB1 {
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
    line: INT,
    text_pos: INT,
    byte_script_size: INT,
}

#[derive(Debug, Clone, ReadUnreal)]
pub struct UClassDependency {
    ///Object Reference.
    class: INDEX,
    deep: DWORD,
    script_text_crc: DWORD,
}

#[derive(Debug, Clone, ReadUnreal)]
pub struct UClassB2 {
    probe_mask: QWORD,
    ignore_mask: QWORD,
    ///Offset of the Label Table into the script
    label_table_offset: WORD,
    state_flags: DWORD,
    class_flags: DWORD,
    guid: GUID,
    dependencies: Vec<UClassDependency>,
    ///Object References.
    imports: Vec<INDEX>,
    ///Object References.
    class_within: INDEX,
    ///Name Reference.
    class_config_name: INDEX,
    properties: Vec<INDEX>,
}

#[derive(Debug, Clone)]
pub struct UClass {
    super_class_name: String,
    next_field_name: String,
    script_text_name: String,
    child: String,
    ///Name of the struct. Name Reference.
    name: String,
    cpp_text: String,
    line: i32,
    text_pos: i32,
    byte_script_size: i32,

    probe_mask: u64,
    ignore_mask: u64,
    label_table_offset: u16,
    state_flags: u32,
    class_flags: u32,
    guid: u128,
    dependencies: Vec<UClassDependency>,
    imports: Vec<String>,
    class_within: String,
    class_config_name: String,
    properties: Vec<String>,
}

impl UClass {
    pub(crate) fn parse<T: Read + Seek, F1: Fn(INDEX) -> String, F2: Fn(INDEX) -> String>(
        reader: &mut T,
        object_name_resolver: &F1,
        name_resolver: &F2,
    ) -> Self {
        let bin_1 = reader.read_unreal_value::<UClassB1>();
        let bin_2 = reader.read_unreal_value::<UClassB2>();

        let class = UClass {
            super_class_name: object_name_resolver(bin_1.super_class),
            next_field_name: object_name_resolver(bin_1.next_field),
            script_text_name: object_name_resolver(bin_1.script_text),
            child: object_name_resolver(bin_1.children),
            name: name_resolver(bin_1.friendly_name),
            cpp_text: object_name_resolver(bin_1.cpp_text),

            line: bin_1.line,
            text_pos: bin_1.text_pos,
            byte_script_size: bin_1.byte_script_size,

            probe_mask: bin_2.probe_mask,
            ignore_mask: bin_2.ignore_mask,
            label_table_offset: bin_2.label_table_offset,
            state_flags: bin_2.state_flags,
            class_flags: bin_2.class_flags,
            guid: bin_2.guid,

            dependencies: bin_2.dependencies,
            imports: bin_2
                .imports
                .iter()
                .map(|v| object_name_resolver(*v))
                .collect(),
            class_within: object_name_resolver(bin_2.class_within),
            class_config_name: name_resolver(bin_2.class_config_name),
            properties: bin_2
                .properties
                .iter()
                .map(|v| object_name_resolver(*v))
                .collect(),
        };

        let mut props = vec![];

        while reader.stream_position().unwrap() < class.byte_script_size as u64 {
            props.push(PropertyRecord::parse(
                reader,
                object_name_resolver,
                name_resolver,
                0,
                0,
            ));
        }

        class
    }
}

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
enum PropertyType {
    None = 0,
    Byte,
    Int,
    Bool,
    Float,
    Object,
    Name,
    String,
    Class,
    Array,
    Struct,
    Vector,
    Rotator,
    Str,
    Map,
    FixedArray,
}

enum PropertyRecord {
    None,
    Byte(u8),
    Int(i32),
    Bool(bool),
    Float(f32),
    Object(INDEX),
    Name(String),
    String(String),
    Class(String),
    Array,
    Struct(Box<Struct>),
    Vector(f32, f32, f32),
    Rotator(i32, i32, i32),
    Str(String),
    Map,
    FixedArray,
}

struct RGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

enum Struct {
    Vector(f32, f32, f32),
    Color {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
    RangeVector {
        x_min: f32,
        x_max: f32,

        y_min: f32,
        y_max: f32,

        z_min: f32,
        z_max: f32,
    },
    Range {
        min: f32,
        max: f32,
    },
    Rotator(i32, i32, i32),
    Parse,
    TextureModify {
        use_modify: bool,
        two_side: bool,
        alpha_blend: bool,
        dummy: bool,
        color: RGBA,
        alpha_op: i32,
        color_op: i32,
    },
    Plane(
        PropertyRecord,
        PropertyRecord,
        PropertyRecord,
        PropertyRecord,
    ),
    EffectPawnLightParam(
        PropertyRecord,
        PropertyRecord,
        PropertyRecord,
        PropertyRecord,
        PropertyRecord,
        PropertyRecord,
    ),
    Scale(PropertyRecord),
    Region(
        PropertyRecord,
        PropertyRecord,
        PropertyRecord,
        PropertyRecord,
    ),
}

impl Struct {
    pub(crate) fn parse<T: Read + Seek, F1: Fn(INDEX) -> String, F2: Fn(INDEX) -> String>(
        reader: &mut T,
        object_name_resolver: &F1,
        name_resolver: &F2,
        data_offset: usize,
        data_size: usize,
        struct_type: StructType,
    ) -> Self {
        match struct_type {
            StructType::None => unreachable!(),

            StructType::Vector => Struct::Vector(
                reader.read_unreal_value::<f32>(),
                reader.read_unreal_value::<f32>(),
                reader.read_unreal_value::<f32>(),
            ),

            StructType::Color => Struct::Color {
                r: reader.read_unreal_value::<u8>(),
                g: reader.read_unreal_value::<u8>(),
                b: reader.read_unreal_value::<u8>(),
                a: reader.read_unreal_value::<u8>(),
            },

            StructType::RangeVector => {
                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(3)).unwrap();
                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(1)).unwrap();
                let x_min = reader.read_unreal_value::<f32>();

                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(1)).unwrap();
                let x_max = reader.read_unreal_value::<f32>();

                reader.seek(SeekFrom::Current(1)).unwrap();
                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(3)).unwrap();
                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(1)).unwrap();
                let y_min = reader.read_unreal_value::<f32>();

                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(1)).unwrap();
                let y_max = reader.read_unreal_value::<f32>();

                reader.seek(SeekFrom::Current(1)).unwrap();
                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(3)).unwrap();
                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(1)).unwrap();
                let z_min = reader.read_unreal_value::<f32>();

                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(1)).unwrap();
                let z_max = reader.read_unreal_value::<f32>();

                reader.seek(SeekFrom::Current(2)).unwrap();

                Struct::RangeVector {
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                    z_min,
                    z_max,
                }
            }

            StructType::Range => {
                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(1)).unwrap();

                let min = reader.read_unreal_value::<f32>();

                reader.read_unreal_value::<INDEX>();
                reader.seek(SeekFrom::Current(1)).unwrap();

                let max = reader.read_unreal_value::<f32>();

                reader.seek(SeekFrom::Current(1)).unwrap();

                Struct::Range { min, max }
            }

            StructType::Rotator => Struct::Rotator(
                reader.read_unreal_value::<i32>(),
                reader.read_unreal_value::<i32>(),
                reader.read_unreal_value::<i32>(),
            ),

            StructType::Parse => unimplemented!(),

            StructType::TextureModify => {
                reader.read_unreal_value::<INDEX>();
                reader.read_unreal_value::<u8>();
                let use_modify = reader.read_unreal_value::<u8>() != 0x0;

                reader.read_unreal_value::<INDEX>();
                reader.read_unreal_value::<u8>();
                let two_side = reader.read_unreal_value::<u8>() != 0x0;

                reader.read_unreal_value::<INDEX>();
                reader.read_unreal_value::<u8>();
                let alpha_blend = reader.read_unreal_value::<u8>() != 0x0;

                reader.read_unreal_value::<INDEX>();
                reader.read_unreal_value::<u8>();
                let dummy = reader.read_unreal_value::<u8>() != 0x0;

                reader.read_unreal_value::<INDEX>();
                reader.read_unreal_value::<u8>();
                reader.read_unreal_value::<u8>();

                let b = reader.read_unreal_value::<u8>();
                let g = reader.read_unreal_value::<u8>();
                let r = reader.read_unreal_value::<u8>();
                let a = reader.read_unreal_value::<u8>();

                reader.read_unreal_value::<INDEX>();
                reader.read_unreal_value::<u8>();
                let alpha_op = reader.read_unreal_value::<i32>();

                reader.read_unreal_value::<INDEX>();
                reader.read_unreal_value::<u8>();
                let color_op = reader.read_unreal_value::<i32>();

                Struct::TextureModify {
                    use_modify,
                    two_side,
                    alpha_blend,
                    dummy,
                    color: RGBA { r, g, b, a },
                    alpha_op,
                    color_op,
                }
            }

            StructType::Plane => Struct::Plane(
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
            ),

            StructType::EffectPawnLightParam => Struct::EffectPawnLightParam(
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
            ),

            StructType::Scale => Struct::Scale(PropertyRecord::parse(
                reader,
                object_name_resolver,
                name_resolver,
                data_offset,
                data_size,
            )),

            StructType::Region => Struct::Region(
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
                PropertyRecord::parse(
                    reader,
                    object_name_resolver,
                    name_resolver,
                    data_offset,
                    data_size,
                ),
            ),
        }
    }
}

#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
enum StructType {
    None,
    Vector,
    Color,
    RangeVector,
    Range,
    Rotator,
    Parse,
    TextureModify,
    Plane,
    EffectPawnLightParam,
    Scale,
    Region,
}

impl From<String> for StructType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Vector" => StructType::Vector,
            "Color" => StructType::Color,
            "RangeVector" => StructType::RangeVector,
            "Range" => StructType::Range,
            "Rotator" => StructType::Rotator,
            "PARSE" => StructType::Parse,
            "TextureModifyinfo" => StructType::TextureModify,
            "Plane" => StructType::Plane,
            "EffectPawnLightParam" => StructType::EffectPawnLightParam,
            "Scale" => StructType::Scale,
            "Region" => StructType::Region,

            _ => unreachable!(),
        }
    }
}

impl PropertyRecord {
    pub(crate) fn parse<T: Read + Seek, F1: Fn(INDEX) -> String, F2: Fn(INDEX) -> String>(
        reader: &mut T,
        object_name_resolver: &F1,
        name_resolver: &F2,
        data_offset: usize,
        data_size: usize,
    ) -> Self {
        let pos = reader.stream_position().unwrap();

        let name = reader.read_unreal_value::<INDEX>();

        if name.0 == 0 {
            return PropertyRecord::None;
        }

        let info_byte = reader.read_unreal_value::<u8>();

        let s_type: PropertyType = PropertyType::from_u8(info_byte & 0x0F).unwrap();

        let mut struct_type = StructType::None;

        if s_type == PropertyType::Struct {
            struct_type = StructType::from(name_resolver(reader.read_unreal_value::<INDEX>()));
        };

        let local_size = match (info_byte >> 4) & 7 {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 12,
            4 => 16,
            5 => reader.read_unreal_value::<u8>() as u64,
            6 => reader.read_unreal_value::<u16>() as u64,
            7 => reader.read_unreal_value::<u32>() as u64,

            _ => unreachable!(),
        };

        let mut _size = local_size + reader.stream_position().unwrap() - pos;

        let is_array = info_byte & 0x80 != 0;

        if is_array {}

        match s_type {
            PropertyType::None => PropertyRecord::None,

            PropertyType::Byte => PropertyRecord::Byte(reader.read_unreal_value::<u8>()),

            PropertyType::Bool => {
                if local_size == 1 {
                    _size += 1;
                    PropertyRecord::Bool(reader.read_unreal_value::<u8>() > 0)
                } else {
                    PropertyRecord::Bool(is_array)
                }
            }

            PropertyType::Float => PropertyRecord::Float(reader.read_unreal_value::<f32>()),

            PropertyType::Int => PropertyRecord::Int(reader.read_unreal_value::<i32>()),

            PropertyType::Str => {
                PropertyRecord::Str(reader.read_unreal_value::<ASCF>().to_string())
            }

            PropertyType::Struct => PropertyRecord::Struct(Box::new(Struct::parse(
                reader,
                object_name_resolver,
                name_resolver,
                data_offset,
                data_size,
                struct_type,
            ))),

            PropertyType::Object => PropertyRecord::Object(reader.read_unreal_value::<INDEX>()),

            PropertyType::String => {
                if data_size == 0 {
                    PropertyRecord::String("".to_string())
                } else {
                    unimplemented!()
                }
            }

            PropertyType::Rotator => PropertyRecord::Rotator(
                reader.read_unreal_value::<i32>(),
                reader.read_unreal_value::<i32>(),
                reader.read_unreal_value::<i32>(),
            ),

            PropertyType::Name => {
                PropertyRecord::Name(name_resolver(reader.read_unreal_value::<INDEX>()))
            }

            PropertyType::Class => {
                PropertyRecord::Class(name_resolver(reader.read_unreal_value::<INDEX>()))
            }

            PropertyType::Array => unimplemented!(),

            PropertyType::Vector => PropertyRecord::Vector(
                reader.read_unreal_value::<f32>(),
                reader.read_unreal_value::<f32>(),
                reader.read_unreal_value::<f32>(),
            ),

            PropertyType::Map => unimplemented!(),

            PropertyType::FixedArray => unimplemented!(),
        }
    }
}
