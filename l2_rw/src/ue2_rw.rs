use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_traits::{AsPrimitive, FromPrimitive};
use r#macro::{ReadUnreal, WriteUnreal};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::slice;
use std::slice::Iter;
use yore::code_pages::CP1252;

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
pub struct CompactInt(pub(crate) i32);
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ASCF(String);

impl ASCF {
    pub fn inner(&self) -> &String {
        &self.0
    }

    pub fn empty() -> Self {
        Self("\0".to_string())
    }
}

impl ToString for ASCF {
    fn to_string(&self) -> String {
        self.0.replace('\0', "").replace("\\n", "\n")
    }
}

impl From<&String> for ASCF {
    fn from(value: &String) -> Self {
        ASCF(value.replace('\n', "\\n") + "\0")
    }
}

impl From<String> for ASCF {
    fn from(value: String) -> Self {
        ASCF(value.replace('\n', "\\n") + "\0")
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct UVEC<I, T> {
    pub _i: PhantomData<I>,
    pub inner: Vec<T>,
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

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct DVEC<I, T1, T2> {
    pub _i: PhantomData<I>,
    pub inner: Vec<(T1, T2)>,
}

impl<I, T1, T2> From<Vec<(T1, T2)>> for DVEC<I, T1, T2> {
    fn from(value: Vec<(T1, T2)>) -> Self {
        Self {
            _i: PhantomData,
            inner: value,
        }
    }
}

impl<'a, I, T1, T2> IntoIterator for &'a DVEC<I, T1, T2> {
    type Item = &'a (T1, T2);
    type IntoIter = Iter<'a, (T1, T2)>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
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

impl<I: WriteUnreal + FromPrimitive + AsPrimitive<usize>, V1: WriteUnreal, V2: WriteUnreal>
    WriteUnreal for DVEC<I, V1, V2>
where
    usize: AsPrimitive<I>,
{
    fn write_unreal<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let v: I = self.inner.len().as_();
        v.write_unreal(writer)?;

        for v in &self.inner {
            v.0.write_unreal(writer)?;
        }

        for v in &self.inner {
            v.1.write_unreal(writer)?;
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

        String::from_utf16(s).unwrap()
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
            let s: &[u16] =
                unsafe { slice::from_raw_parts(bytes.as_ptr() as *const _, count as usize / 2) };
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

impl<
        I: ReadUnreal + AsPrimitive<usize>,
        V1: ReadUnreal + Default + Clone,
        V2: ReadUnreal + Default + Clone,
    > ReadUnreal for DVEC<I, V1, V2>
{
    fn read_unreal<T: Read>(reader: &mut T) -> Self {
        let len: usize = I::read_unreal(reader).as_();

        let mut res = vec![(V1::default(), V2::default()); len];

        for v in res.iter_mut().take(len) {
            v.0 = V1::read_unreal(reader);
        }
        for v in res.iter_mut().take(len) {
            v.1 = V2::read_unreal(reader);
        }

        DVEC {
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
