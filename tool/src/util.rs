#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::needless_borrow)]
#![allow(dead_code)]

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::slice;
use std::slice::Iter;

use deunicode::deunicode;
use num_traits::{AsPrimitive, FromPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use yore::code_pages::CP1252;

pub trait DebugUtils {
    fn print_ordered(&self);
}

impl<K: Ord + Debug + Hash, V: Debug> DebugUtils for HashMap<K, V> {
    fn print_ordered(&self) {
        let mut keys: Vec<_> = self.keys().collect();
        keys.sort();

        for k in keys {
            println!("  {k:?} - {:?}", self.get(k).unwrap())
        }
    }
}

pub trait L2StringTable {
    fn keys(&self) -> Keys<u32, String>;
    fn get(&self, key: &u32) -> Option<&String>;
    fn get_o(&self, key: &u32) -> String;
    fn from_vec(values: Vec<String>) -> Self;
    fn get_index(&mut self, value: &str) -> u32;
    fn add(&mut self, value: String) -> u32;
}

pub trait StrUtils {
    fn to_ascii_snake_case(&self) -> String;
    fn to_ascii_camel_case(&self) -> String;
    fn de_unicode(&self) -> String;
}

impl StrUtils for str {
    fn to_ascii_snake_case(&self) -> String {
        let mut res = "".to_string();

        let mut first = true;
        for l in self.de_unicode().trim().chars() {
            if l == ' ' {
                res.push('_');
                first = true;

                continue;
            }

            if !l.is_alphanumeric() {
                continue;
            } else if l.is_lowercase() || !l.is_alphabetic() {
                res.push(l);
            } else {
                if !first {
                    res.push('_');
                }

                res.push_str(&l.to_lowercase().to_string());
            }

            first = false;
        }

        res
    }
    fn to_ascii_camel_case(&self) -> String {
        let mut res = "".to_string();

        let mut force_capital = true;
        for l in self.de_unicode().trim().chars() {
            if l == ' ' {
                force_capital = true;

                continue;
            }

            if !l.is_alphanumeric() {
                continue;
            } else if !l.is_alphabetic() {
                res.push(l);
            } else if force_capital {
                res.push_str(&l.to_uppercase().to_string());
            } else {
                res.push_str(&l.to_lowercase().to_string());
            }

            force_capital = false;
        }

        res
    }

    fn de_unicode(&self) -> String {
        deunicode(self).replace('\'', "")
    }
}

pub type BYTE = u8;
pub type WORD = u16;
pub type USHORT = u16;
pub type SHORT = i16;
pub type DWORD = u32;
pub type INT = i32;
pub type LONG = i64;
pub type FLOAT = f32;
pub type DOUBLE = f64;
pub type GUID = u128;
/** The import table, export table and many other places in a package file reference objects. Such references are stored as compact index value in UE1 and 2 and as DWORD value in UE3. Object references are resolved as follows:

If the reference is zero, no object is referenced, i.e. NULL/None.
If the reference is positive, the object must be looked up in the export table at index (-reference)-1.
If the reference is negative, the object must be looked up in the import table at index 1-reference.*/
pub type INDEX = CompactInt;
pub type STR = String;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CompactInt(i32);
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ASCF(pub(crate) String);

impl ToString for ASCF {
    fn to_string(&self) -> String {
        self.0.replace("\\n", "\n")
    }
}

impl From<&String> for ASCF {
    fn from(value: &String) -> Self {
        ASCF(value.replace('\n', "\\n"))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct UVEC<I, T> {
    pub(crate) _i: PhantomData<I>,
    pub(crate) inner: Vec<T>,
}

impl<I, T> From<Vec<T>> for UVEC<I, T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            _i: PhantomData,
            inner: value,
        }
    }
}

impl<'a, I, T> IntoIterator for &'a UVEC<I, T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct FLOC {
    pub(crate) x: FLOAT,
    pub(crate) y: FLOAT,
    pub(crate) z: FLOAT,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
pub struct Color {
    pub r: BYTE,
    pub g: BYTE,
    pub b: BYTE,
    pub a: BYTE,
}
#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
pub struct Collision {
    pub radius_1: FLOAT,
    pub radius_2: FLOAT,
    pub height_1: FLOAT,
    pub height_2: FLOAT,
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal, Default)]
pub struct MTX {
    pub vec_1: UVEC<BYTE, DWORD>,
    pub vec_2: UVEC<BYTE, DWORD>,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MTX3 {
    pub vec_1: Vec<DWORD>,
    pub vec_1_f: Vec<(BYTE, BYTE)>,
    pub vec_2: Vec<DWORD>,
    pub val: DWORD,
}

impl ReadUnreal for MTX3 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        let s1 = reader.read_u8().unwrap() as usize;

        let mut vec_1 = Vec::with_capacity(s1);
        let mut vec_1_f = Vec::with_capacity(s1);

        for _ in 0..s1 {
            vec_1.push(reader.read_u32::<LittleEndian>().unwrap())
        }

        for _ in 0..s1 {
            vec_1_f.push((reader.read_u8().unwrap(), reader.read_u8().unwrap()))
        }

        let s2 = reader.read_u8().unwrap() as usize;

        let mut vec_2 = Vec::with_capacity(s1);

        for _ in 0..s2 {
            vec_2.push(reader.read_u32::<LittleEndian>().unwrap())
        }

        let val = reader.read_u32::<LittleEndian>().unwrap();

        Self {
            vec_1,
            vec_1_f,
            vec_2,
            val,
        }
    }
}

impl WriteUnreal for MTX3 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_u8(self.vec_1.len() as u8)?;

        for v in &self.vec_1 {
            writer.write_u32::<LittleEndian>(*v)?;
        }

        for (v1, v2) in &self.vec_1_f {
            writer.write_u8(*v1)?;
            writer.write_u8(*v2)?;
        }

        writer.write_u8(self.vec_2.len() as u8)?;

        for v in &self.vec_2 {
            writer.write_u32::<LittleEndian>(*v)?;
        }

        writer.write_u32::<LittleEndian>(self.val)?;

        Ok(())
    }
}

pub trait WriteUnreal {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()>;
}

impl WriteUnreal for () {
    fn write_unreal<T: Write>(&self, _writer: &mut T) -> std::io::Result<()> {
        unreachable!()
    }
}

impl WriteUnreal for INDEX {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let negative = self.0 < 0;
        let v = self.0.abs();

        let mut bbytes = [
            v & 63,
            v >> 6 & 127,
            v >> 13 & 127,
            v >> 20 & 127,
            v >> 27 & 127,
        ];

        if negative {
            bbytes[0] |= 128;
        }

        let mut size = 5;

        let mut i = 4;
        while i > 0 && bbytes[i] == 0 {
            size -= 1;
            i -= 1;
        }

        let mut res = vec![0u8; size];

        let mut i = 0;
        while i < size {
            if i != size - 1 {
                bbytes[i] |= if i == 0 { 64 } else { 128 }
            }

            res[i] = bbytes[i] as u8;
            i += 1
        }

        writer.write_all(&res)?;

        Ok(())
    }
}

impl WriteUnreal for STR {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let c: Vec<_> = self.encode_utf16().collect();
        ((c.len() * 2) as u32).write_unreal(writer)?;

        for v in c {
            writer.write_all(&v.to_le_bytes())?;
        }

        Ok(())
    }
}

impl WriteUnreal for ASCF {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        if self.0.is_ascii() {
            let v = CP1252.encode(&self.0).unwrap();
            CompactInt(v.len() as i32).write_unreal(writer)?;
            writer.write_all(&v)?;
        } else {
            let c = self.0.encode_utf16();

            CompactInt(-(c.clone().count() as i32) - 1).write_unreal(writer)?;

            for v in c {
                writer.write_all(&v.to_le_bytes())?;
            }

            writer.write_all(&0u16.to_le_bytes())?;
        }

        Ok(())
    }
}

impl WriteUnreal for u8 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_u8(*self)?;

        Ok(())
    }
}

impl WriteUnreal for u16 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_u16::<LittleEndian>(*self)?;

        Ok(())
    }
}

impl WriteUnreal for i16 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_i16::<LittleEndian>(*self)?;

        Ok(())
    }
}

impl WriteUnreal for u32 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(*self)?;

        Ok(())
    }
}

impl WriteUnreal for i32 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_i32::<LittleEndian>(*self)?;

        Ok(())
    }
}

impl WriteUnreal for f32 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_f32::<LittleEndian>(*self)?;

        Ok(())
    }
}

impl WriteUnreal for f64 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_f64::<LittleEndian>(*self)?;

        Ok(())
    }
}

impl WriteUnreal for i64 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_i64::<LittleEndian>(*self)?;

        Ok(())
    }
}

impl WriteUnreal for u128 {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_u128::<LittleEndian>(*self)?;

        Ok(())
    }
}

impl<V: WriteUnreal> WriteUnreal for Vec<V> {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        CompactInt(self.len() as i32).write_unreal(writer)?;

        for v in self {
            v.write_unreal(writer)?;
        }

        Ok(())
    }
}
impl<I: WriteUnreal + FromPrimitive + AsPrimitive<usize>, V: WriteUnreal> WriteUnreal for UVEC<I, V>
where
    usize: AsPrimitive<I>,
{
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let v: I = self.inner.len().as_();
        v.write_unreal(writer)?;

        for v in &self.inner {
            v.write_unreal(writer)?;
        }

        Ok(())
    }
}

impl<V: WriteUnreal> WriteUnreal for &V {
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        (*self).write_unreal(writer)?;

        Ok(())
    }
}

pub trait UnrealWriter {
    fn write_unreal_value<V: WriteUnreal>(&mut self, val: V) -> std::io::Result<()>;
}

impl<T: Write> UnrealWriter for T {
    fn write_unreal_value<V: WriteUnreal>(&mut self, val: V) -> std::io::Result<()> {
        val.write_unreal(self)?;

        Ok(())
    }
}

pub trait ReadUnreal {
    fn read_unreal<T: Read>(reader: &mut T) -> Self;
}

impl ReadUnreal for INDEX {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        let mut output: i32 = 0;
        let mut signed = false;

        for i in 0..4 {
            let x = reader.read_u8().unwrap() as i32 & 255;

            if i == 0 {
                if x & 128 > 0 {
                    signed = true;
                }

                output |= x & 63;

                if x & 64 == 0 {
                    break;
                }
            } else if i == 4 {
                output |= (x & 31) << 27;
            } else {
                output |= (x & 127) << (6 + (i - 1) * 7);
                if x & 128 == 0 {
                    break;
                }
            }
        }

        if signed {
            output *= -1
        }

        Self(output)
    }
}

impl ReadUnreal for STR {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        let count = u32::read_unreal(reader);
        let mut bytes: Vec<u8> = vec![0u8; count as usize];
        reader.read_exact(&mut bytes).unwrap();
        let s: &[u16] =
            unsafe { slice::from_raw_parts(bytes.as_ptr() as *const _, bytes.len() / 2) };

        String::from_utf16(s).unwrap().replace('\0', "")
    }
}

impl ReadUnreal for ASCF {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        let mut count = reader.read_unreal_value::<INDEX>().0;
        let mut skip = 1;

        if count < 0 {
            count *= -2;
            skip = 2;
        }

        let mut bytes: Vec<u8> = vec![0u8; count as usize];
        reader.read_exact(&mut bytes).unwrap();

        if skip == 2 {
            let s: &[u16] = unsafe {
                slice::from_raw_parts(bytes.as_ptr() as *const _, count as usize / 2 - 1)
            };
            ASCF(String::from_utf16(s).unwrap())
        } else {
            ASCF(CP1252.decode(&bytes).to_string())
        }
    }
}

impl ReadUnreal for u8 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        reader.read_u8().unwrap()
    }
}

impl ReadUnreal for u16 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        reader.read_u16::<LittleEndian>().unwrap()
    }
}

impl ReadUnreal for i16 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        reader.read_i16::<LittleEndian>().unwrap()
    }
}

impl ReadUnreal for u32 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        reader.read_u32::<LittleEndian>().unwrap()
    }
}

impl ReadUnreal for f32 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        reader.read_f32::<LittleEndian>().unwrap()
    }
}

impl ReadUnreal for f64 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        reader.read_f64::<LittleEndian>().unwrap()
    }
}

impl ReadUnreal for i32 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        reader.read_i32::<LittleEndian>().unwrap()
    }
}

impl ReadUnreal for i64 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        reader.read_i64::<LittleEndian>().unwrap()
    }
}

impl ReadUnreal for u128 {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        reader.read_u128::<LittleEndian>().unwrap()
    }
}

impl<V: ReadUnreal> ReadUnreal for Vec<V> {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        let len = INDEX::read_unreal(reader).0;

        let mut res = Vec::with_capacity(len as usize);

        for _ in 0..len {
            res.push(V::read_unreal(reader))
        }

        res
    }
}

impl<I: ReadUnreal + AsPrimitive<usize>, V: ReadUnreal> ReadUnreal for UVEC<I, V> {
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        let len: usize = I::read_unreal(reader).as_();

        let mut res = Vec::with_capacity(len);

        for _ in 0..len {
            res.push(V::read_unreal(reader))
        }

        UVEC {
            _i: PhantomData,
            inner: res,
        }
    }
}

pub trait UnrealReader {
    fn read_unreal_value<V: ReadUnreal>(&mut self) -> V;
}

impl<T: Read> UnrealReader for T {
    fn read_unreal_value<Z: ReadUnreal>(&mut self) -> Z {
        Z::read_unreal(self)
    }
}

pub trait UnrealCasts {
    fn to_u32_bool(self) -> u32;
    fn to_u8_bool(self) -> u8;
}

impl UnrealCasts for bool {
    fn to_u32_bool(self) -> u32 {
        if self {
            1
        } else {
            0
        }
    }
    fn to_u8_bool(self) -> u8 {
        if self {
            1
        } else {
            0
        }
    }
}

pub mod l2_reader {
    use crate::util::{CompactInt, ReadUnreal, UnrealWriter, WriteUnreal, INDEX};
    use byteorder::WriteBytesExt;
    use inflate::inflate_bytes_zlib_no_checksum;
    use miniz_oxide::deflate::compress_to_vec_zlib;
    use openssl::bn::BigNum;
    use openssl::rsa::Padding;
    use std::fmt::Debug;
    use std::fs::File;
    use std::io::{BufReader, Cursor, Read, Seek, Write};
    use std::path::Path;

    pub const PACKAGE_FILE_TAG: u32 = 0x9E2A83C1;
    pub const LINEAGE_HEADER: &[u8; 22] =
        b"L\x00i\x00n\x00e\x00a\x00g\x00e\x002\x00V\x00e\x00r\x00";
    pub const END_BYTES: &[u8; 20] =
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100];
    pub const V111: &[u8; 6] = b"1\x001\x001\x00";
    pub const V121: &[u8; 6] = b"1\x002\x001\x00";
    pub const V411: &[u8; 6] = b"4\x001\x001\x00";
    pub const V412: &[u8; 6] = b"4\x001\x002\x00";
    pub const V413: &[u8; 6] = b"4\x001\x003\x00";
    pub const V414: &[u8; 6] = b"4\x001\x004\x00";

    pub enum EncVersion {
        V111,
        V121,
        V411,
        V412,
        V413,
        V414,
    }

    impl EncVersion {
        fn get_modulus(&self) -> Option<BigNum> {
            match self {
                EncVersion::V111 => {
                    None
                }
                EncVersion::V121 => {
                    None
                }

                EncVersion::V411 => {
                    Some(BigNum::from_hex_str(
                        "8c9d5da87b30f5d7cd9dc88c746eaac5bb180267fa11737358c4c95d9adf59dd37689f9befb251508759555d6fe0eca87bebe0a10712cf0ec245af84cd22eb4cb675e98eaf5799fca62a20a2baa4801d5d70718dcd43283b8428f1387aec6600f937bfc7bb72404d187d3a9c438f1ffce9ce365dccf754232ff6def038a41385",
                    ).unwrap())
                }
                EncVersion::V412 => {
                    Some(BigNum::from_hex_str(
                        "a465134799cf2c45087093e7d0f0f144e6d528110c08f674730d436e40827330eccea46e70acf10cdda7d8f710e3b44dcca931812d76cd7494289bca8b73823f57efc0515b97e4a2a02612ccfa719cf7885104b06f2e7e2cc967b62e3d3b1aadb925db94cbc8cd3070a4bb13f7e202c7733a67b1b94c1ebc0afcbe1a63b448cf",
                    ).unwrap())
                }
                EncVersion::V413 => {
                    Some(BigNum::from_hex_str(
                        // "97df398472ddf737ef0a0cd17e8d172f0fef1661a38a8ae1d6e829bc1c6e4c3cfc19292dda9ef90175e46e7394a18850b6417d03be6eea274d3ed1dde5b5d7bde72cc0a0b71d03608655633881793a02c9a67d9ef2b45eb7c08d4be329083ce450e68f7867b6749314d40511d09bc5744551baa86a89dc38123dc1668fd72d83",
                        "75b4d6de5c016544068a1acf125869f43d2e09fc55b8b1e289556daf9b8757635593446288b3653da1ce91c87bb1a5c18f16323495c55d7d72c0890a83f69bfd1fd9434eb1c02f3e4679edfa43309319070129c267c85604d87bb65bae205de3707af1d2108881abb567c3b3d069ae67c3a4c6a3aa93d26413d4c66094ae2039",
                    ).unwrap())
                }
                EncVersion::V414 => {
                    Some(BigNum::from_hex_str(
                        "ad70257b2316ce09dfaf2ebc3f63b3d673b0c98a403950e26bb87379b11e17aed0e45af23e7171e5ec1fbc8d1ae32ffb7801b31266eef9c334b53469d4b7cbe83284273d35a9aab49b453e7012f374496c65f8089f5d134b0eb3d1e3b22051ed5977a6dd68c4f85785dfcc9f4412c81681944fc4b8ce27caf0242deaa5762e8d"
                    ).unwrap())
                }
            }
        }

        fn get_exponent(&self) -> Option<BigNum> {
            match self {
                EncVersion::V111 => None,
                EncVersion::V121 => None,

                EncVersion::V411 => Some(BigNum::from_hex_str("1d").unwrap()),
                EncVersion::V412 => Some(BigNum::from_hex_str("25").unwrap()),
                EncVersion::V413 => {
                    Some(
                        BigNum::from_hex_str(
                            // "35",
                            "1d",
                        )
                        .unwrap(),
                    )
                }
                EncVersion::V414 => Some(BigNum::from_hex_str("25").unwrap()),
            }
        }
    }

    struct Decoder<'a, T: Read> {
        data: T,
        output: &'a mut Vec<u8>,
    }

    impl<'a, T: Read> Decoder<'a, T> {
        fn byte_to_int(b: u8) -> i32 {
            if b > 128 {
                b as i32 - 256
            } else {
                b as i32
            }
        }

        pub fn decode(&mut self, modulus: BigNum, exp: BigNum) {
            let mut buff = [0u8; 128];

            let mut ct = self.data.read(&mut buff).unwrap();

            let rsa = openssl::rsa::Rsa::from_public_components(modulus, exp).unwrap();

            let mut chunk = [0u8; 128];

            while ct != 0 {
                rsa.public_decrypt(&buff, &mut chunk, Padding::NONE)
                    .unwrap();

                let size = Self::byte_to_int(chunk[3]) & 0xFF;
                let pad = (-size & 0x1) + (-size & 0x2);
                let start = (128 - size - pad) as usize;
                let end = start + size as usize;

                self.output.extend_from_slice(&chunk[start..end]);

                ct = self.data.read(&mut buff).unwrap();
                chunk = [0u8; 128];
            }

            //INFO: stored size, for debug
            //
            // let mut size = self.output[0] as i32;
            //
            // size = size
            //     .overflowing_add(Self::byte_to_int(self.output[1]).overflowing_shl(8).0 & 0xFF00)
            //     .0;
            // size = size
            //     .overflowing_add(Self::byte_to_int(self.output[2]).overflowing_shl(16).0 & 0xFF0000)
            //     .0;
            // size = size
            //     .overflowing_add(
            //         Self::byte_to_int(self.output[3]).overflowing_shl(24).0 & -16777216i32,
            //     )
            //     .0;

            let res =
                inflate_bytes_zlib_no_checksum(&self.output[4..self.output.len() - 4]).unwrap();

            *self.output = res;
        }
    }

    pub fn deserialize_dat_with_string_dict<S: ReadUnreal + Debug, T: ReadUnreal + Debug>(
        file_path: &Path,
    ) -> Result<(Vec<S>, Vec<T>), ()> {
        println!("Loading {file_path:?}...");
        let Ok(bytes) = read_l2_file(file_path) else {
            return Err(());
        };

        //INFO: For debug
        // let mut t = File::create(format!(
        //     "./test/{}",
        //     file_path.file_name().unwrap().to_str().unwrap()
        // ))
        // .unwrap();
        // t.write_all(&bytes).unwrap();

        let mut reader = BufReader::new(Cursor::new(bytes));

        let count = INDEX::read_unreal(&mut reader);
        let mut string_dict = Vec::with_capacity(count.0 as usize);

        println!("\tDict elements count: {}", count.0);

        for _ in 0..count.0 {
            let t = S::read_unreal(&mut reader);
            string_dict.push(t);
        }

        let count = u32::read_unreal(&mut reader);

        println!("\tElements count: {count}");

        let mut res = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let t = T::read_unreal(&mut reader);
            res.push(t);
        }

        Ok((string_dict, res))
    }

    pub fn deserialize_dat<T: ReadUnreal + Debug>(file_path: &Path) -> Result<Vec<T>, ()> {
        println!("Loading {file_path:?}");
        let Ok(bytes) = read_l2_file(file_path) else {
            println!("Err!");
            return Err(());
        };

        let bugged = file_path
            .to_str()
            .unwrap()
            .to_lowercase()
            .ends_with("_baseinfo.dat");

        //INFO: For debug
        // let mut t = File::create(format!(
        //     "./test/{}",
        //     file_path.file_name().unwrap().to_str().unwrap()
        // ))
        // .unwrap();
        // t.write_all(&bytes).unwrap();
        let bytes_count = bytes.len();

        let mut reader = BufReader::new(Cursor::new(bytes));
        let count = u32::read_unreal(&mut reader);

        println!("\tElements count: {count}");

        let mut res = Vec::with_capacity(count as usize);

        for _ in 0..count {
            if bugged && bytes_count - (reader.stream_position().unwrap() as usize) < 16 {
                break;
            }

            let t = T::read_unreal(&mut reader);

            res.push(t);
        }

        println!("\tLoaded: {}", res.len());
        Ok(res)
    }

    pub enum DatVariant<S: WriteUnreal + Debug, T: WriteUnreal + Debug> {
        Array(Vec<T>),
        DoubleArray(Vec<S>, Vec<T>),
    }

    fn int_to_byte(i: i32) -> u8 {
        if i < 0 {
            (i + 256) as u8
        } else {
            i as u8
        }
    }

    fn encode<D: Write>(data: Vec<u8>, output: &mut D, modulus: BigNum, exp: BigNum) {
        let compressed = compress_to_vec_zlib(&data, 6);

        let res = Vec::with_capacity(compressed.len() + 10);
        let mut cursor = Cursor::new(res);

        cursor
            .write_u8(int_to_byte(data.len() as i32 & 0xFF))
            .unwrap();
        cursor
            .write_u8(int_to_byte(
                (data.len() as i32 & 0xFF00).overflowing_shr(8).0,
            ))
            .unwrap();
        cursor
            .write_u8(int_to_byte(
                (data.len() as i32 & 0xFF0000).overflowing_shr(16).0,
            ))
            .unwrap();
        cursor
            .write_u8(int_to_byte(
                (data.len() as i32 & -16777216i32).overflowing_shr(24).0,
            ))
            .unwrap();

        cursor.write_all(&compressed).unwrap();

        let mut cursor = Cursor::new(cursor.into_inner());

        let mut buff = [0u8; 124];
        let mut block = [0u8; 128];
        let mut chunk = [0u8; 128];

        let rsa = openssl::rsa::Rsa::from_public_components(modulus, exp).unwrap();

        while let Ok(len) = cursor.read(&mut buff) {
            if len == 0 {
                break;
            }

            block[3] = int_to_byte(len as i32 & 0xFF);

            let start = 128 - len - (124 - len) % 4;

            block[start..(len + start)].copy_from_slice(&buff[..len]);

            rsa.public_encrypt(&block, &mut chunk, Padding::NONE)
                .unwrap();

            output.write_all(&chunk).unwrap();

            block = [0u8; 128];
            chunk = [0u8; 128];
        }
    }

    pub fn save_dat<S: WriteUnreal + Debug, T: WriteUnreal + Debug>(
        file_path: &Path,
        data: DatVariant<S, T>,
    ) -> std::io::Result<usize> {
        let mut serialized_data = Vec::new();

        match data {
            DatVariant::Array(data) => {
                serialized_data.write_unreal_value(data.len() as u32)?;
                for v in data {
                    serialized_data.write_unreal_value(v)?;
                }
            }
            DatVariant::DoubleArray(table, data) => {
                serialized_data.write_unreal_value(CompactInt(table.len() as i32))?;
                for v in table {
                    serialized_data.write_unreal_value(v)?;
                }

                serialized_data.write_unreal_value(data.len() as u32)?;
                for v in data {
                    serialized_data.write_unreal_value(v)?;
                }
            }
        }

        serialized_data.write_all(&[12, 83, 97, 102, 101, 80, 97, 99, 107, 97, 103, 101, 0])?;

        let mut out = File::create(file_path)?;

        out.write_all(LINEAGE_HEADER)?;
        out.write_all(V413)?;

        encode(
            serialized_data,
            &mut out,
            BigNum::from_hex_str("75b4d6de5c016544068a1acf125869f43d2e09fc55b8b1e289556daf9b8757635593446288b3653da1ce91c87bb1a5c18f16323495c55d7d72c0890a83f69bfd1fd9434eb1c02f3e4679edfa43309319070129c267c85604d87bb65bae205de3707af1d2108881abb567c3b3d069ae67c3a4c6a3aa93d26413d4c66094ae2039",).unwrap(),
            BigNum::from_hex_str("30b4c2d798d47086145c75063c8e841e719776e400291d7838d3e6c4405b504c6a07f8fca27f32b86643d2649d1d5f124cdd0bf272f0909dd7352fe10a77b34d831043d9ae541f8263c6fe3d1c14c2f04e43a7253a6dda9a8c1562cbd493c1b631a1957618ad5dfe5ca28553f746e2fc6f2db816c7db223ec91e955081c1de65",).unwrap()
        );

        out.write_all(END_BYTES)?;

        Ok(0)
    }

    pub fn read_l2_file(path: &Path) -> Result<Vec<u8>, ()> {
        let mut file;

        let mut f_file = File::open(path).unwrap();
        file = Vec::with_capacity(f_file.metadata().unwrap().len() as usize);
        f_file.read_to_end(&mut file).unwrap();

        if &file[0..22] == LINEAGE_HEADER {
            let enc = &file[22..28];
            let enc_version = if enc == V111 {
                EncVersion::V111
            } else if enc == V121 {
                EncVersion::V121
            } else if enc == V413 {
                EncVersion::V413
            } else {
                unreachable!("Unknown enc version: {}", String::from_utf8_lossy(enc))
            };

            if let Some(modulus) = enc_version.get_modulus() {
                let exp = enc_version.get_exponent().unwrap();
                let mut output = vec![];
                let mut decoder = Decoder {
                    data: BufReader::new(Cursor::new(&file[28..file.len() - 20])),
                    output: &mut output,
                };

                decoder.decode(modulus, exp);

                file = output;
            } else {
                let xor = file[28] ^ (PACKAGE_FILE_TAG & 0xFF) as u8;
                file = file[28..].iter_mut().map(|b| *b ^ xor).collect();
            }
        }

        Ok(file)
    }
}

pub trait GetId {
    fn get_id(&self) -> u32;
}

pub fn wrap_into_id_map<T: GetId>(vec: Vec<T>) -> HashMap<u32, T> where {
    let mut res = HashMap::new();
    for v in vec {
        res.insert(v.get_id(), v);
    }

    res
}

pub fn wrap_into_id_vec_map<T: GetId>(vec: Vec<T>) -> HashMap<u32, Vec<T>> where {
    let mut res: HashMap<u32, Vec<T>> = HashMap::new();
    for v in vec {
        if let Some(vec) = res.get_mut(&v.get_id()) {
            vec.push(v)
        } else {
            res.insert(v.get_id(), vec![v]);
        }
    }

    res
}
