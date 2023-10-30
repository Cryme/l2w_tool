#![allow(clippy::upper_case_acronyms)]
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;
use std::slice;

use deunicode::deunicode;
use yore::code_pages::CP1252;
use r#macro::FromReader;

pub trait StrUtils {
    fn to_ascii_snake_case(&self) -> String;
    fn deunicode(&self) -> String;
}

impl StrUtils for str {
    fn to_ascii_snake_case(&self) -> String {
        let mut res = "".to_string();

        let mut first = true;
        for l in self.deunicode().trim().chars() {
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

    fn deunicode(&self) -> String {
        deunicode(self)
    }
}

pub type BYTE = u8;
pub type WORD = u16;
pub type SHORT = i16;
pub type DWORD = u32;
pub type LONG = i64;
pub type FLOAT = f32;
pub type INT = i32;
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

impl FromReader for INDEX {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
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

impl FromReader for STR {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        let count = u32::from_reader(reader);
        let mut bytes: Vec<u8> = vec![0u8; count as usize];
        reader.read_exact(&mut bytes).unwrap();
        let s: &[u16] = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const _, bytes.len()/2) };

        String::from_utf16(s).unwrap().replace('\0', "")
    }
}

impl FromReader for ASCF {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        let mut count = reader.read_unreal_value::<INDEX>().0;
        let mut skip = 1;

        if count < 0 {
            count *= -2;
            skip = 2;
        }

        let mut bytes: Vec<u8> = vec![0u8; count as usize];
        reader.read_exact(&mut bytes).unwrap();

        if skip == 2 {
            let s: &[u16] = unsafe { slice::from_raw_parts(bytes.as_ptr() as *const _, count as usize /2-1) };
            ASCF(String::from_utf16(s).unwrap().replace('\0', ""))
        } else {
            ASCF(CP1252.decode(&bytes).to_string())
        }
    }
}

pub trait UnrealValueFromReader {
    fn read_unreal_value<V: FromReader>(&mut self) -> V;
}

impl<T: Read> UnrealValueFromReader for T {
    fn read_unreal_value<Z: FromReader>(&mut self) -> Z {
        Z::from_reader(self)
    }
}

pub trait FromReader {
    fn from_reader<T: Read>(reader: &mut T) -> Self;
}

impl FromReader for u8 {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        reader.read_u8().unwrap()
    }
}

impl FromReader for u16 {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        reader.read_u16::<LittleEndian>().unwrap()
    }
}

impl FromReader for i16 {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        reader.read_i16::<LittleEndian>().unwrap()
    }
}

impl FromReader for u32 {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        reader.read_u32::<LittleEndian>().unwrap()
    }
}

impl FromReader for f32 {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        reader.read_f32::<LittleEndian>().unwrap()
    }
}

impl FromReader for i32 {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        reader.read_i32::<LittleEndian>().unwrap()
    }
}

impl FromReader for i64 {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        reader.read_i64::<LittleEndian>().unwrap()
    }
}

impl FromReader for u128 {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        reader.read_u128::<LittleEndian>().unwrap()
    }
}

impl<V: FromReader> FromReader for Vec<V> {
    fn from_reader<T: Read>(reader: &mut T) -> Self {
        let len = INDEX::from_reader(reader).0;

        let mut res = Vec::with_capacity(len as usize);

        for _ in 0..len {
            res.push(V::from_reader(reader))
        }

        res
    }
}

#[derive(Debug, Clone, PartialEq, FromReader)]
pub struct Color {
    pub r: BYTE,
    pub g: BYTE,
    pub b: BYTE,
    pub a: BYTE,
}

pub mod l2_reader {
    use inflate::inflate_bytes_zlib_no_checksum;
    use num_bigint::BigUint;
    use num_traits::Num;
    use std::fs::File;
    use std::io::{BufReader, Cursor, Read};
    use std::path::{Path};

    pub const PACKAGE_FILE_TAG: u32 = 0x9E2A83C1;
    pub const LINEAGE_HEADER: &[u8; 22] =
        b"L\x00i\x00n\x00e\x00a\x00g\x00e\x002\x00V\x00e\x00r\x00";
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
        fn get_modulus(&self) -> Option<BigUint> {
            match self {
                EncVersion::V111 => {
                    None
                }
                EncVersion::V121 => {
                    None
                }

                EncVersion::V411 => {
                    Some(BigUint::from_str_radix(
                        "8c9d5da87b30f5d7cd9dc88c746eaac5bb180267fa11737358c4c95d9adf59dd37689f9befb251508759555d6fe0eca87bebe0a10712cf0ec245af84cd22eb4cb675e98eaf5799fca62a20a2baa4801d5d70718dcd43283b8428f1387aec6600f937bfc7bb72404d187d3a9c438f1ffce9ce365dccf754232ff6def038a41385",
                        16
                    ).unwrap())
                }
                EncVersion::V412 => {
                    Some(BigUint::from_str_radix(
                        "a465134799cf2c45087093e7d0f0f144e6d528110c08f674730d436e40827330eccea46e70acf10cdda7d8f710e3b44dcca931812d76cd7494289bca8b73823f57efc0515b97e4a2a02612ccfa719cf7885104b06f2e7e2cc967b62e3d3b1aadb925db94cbc8cd3070a4bb13f7e202c7733a67b1b94c1ebc0afcbe1a63b448cf",
                        16
                    ).unwrap())
                }
                EncVersion::V413 => {
                    Some(BigUint::from_str_radix(
                        // "97df398472ddf737ef0a0cd17e8d172f0fef1661a38a8ae1d6e829bc1c6e4c3cfc19292dda9ef90175e46e7394a18850b6417d03be6eea274d3ed1dde5b5d7bde72cc0a0b71d03608655633881793a02c9a67d9ef2b45eb7c08d4be329083ce450e68f7867b6749314d40511d09bc5744551baa86a89dc38123dc1668fd72d83",
                        "75b4d6de5c016544068a1acf125869f43d2e09fc55b8b1e289556daf9b8757635593446288b3653da1ce91c87bb1a5c18f16323495c55d7d72c0890a83f69bfd1fd9434eb1c02f3e4679edfa43309319070129c267c85604d87bb65bae205de3707af1d2108881abb567c3b3d069ae67c3a4c6a3aa93d26413d4c66094ae2039",
                        16
                    ).unwrap())
                }
                EncVersion::V414 => {
                    Some(BigUint::from_str_radix(
                        "ad70257b2316ce09dfaf2ebc3f63b3d673b0c98a403950e26bb87379b11e17aed0e45af23e7171e5ec1fbc8d1ae32ffb7801b31266eef9c334b53469d4b7cbe83284273d35a9aab49b453e7012f374496c65f8089f5d134b0eb3d1e3b22051ed5977a6dd68c4f85785dfcc9f4412c81681944fc4b8ce27caf0242deaa5762e8d",
                        16
                    ).unwrap())
                }
            }
        }

        fn get_exponent(&self) -> Option<BigUint> {
            match self {
                EncVersion::V111 => None,
                EncVersion::V121 => None,

                EncVersion::V411 => Some(BigUint::from_str_radix("1d", 16).unwrap()),
                EncVersion::V412 => Some(BigUint::from_str_radix("25", 16).unwrap()),
                EncVersion::V413 => {
                    Some(
                        BigUint::from_str_radix(
                            // "35",
                            "1d", 16,
                        )
                        .unwrap(),
                    )
                }
                EncVersion::V414 => Some(BigUint::from_str_radix("25", 16).unwrap()),
            }
        }
    }

    struct Decoder<'a, T: Read> {
        modulus: BigUint,
        exponent: BigUint,
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
        pub fn decode(&mut self) {
            let mut buff = [0u8; 128];

            let mut ct = self.data.read(&mut buff).unwrap();

            while ct != 0 {
                let m = BigUint::from_bytes_be(&buff);
                let c = m.modpow(&self.exponent, &self.modulus);
                let mut chunk = c.to_bytes_be();

                let diff = 128 - chunk.len();
                for _ in 0..diff {
                    chunk.insert(0, 0);
                }

                let mut size = Self::byte_to_int(chunk[3]);

                size += Self::byte_to_int(chunk[2]).overflowing_shl(8).0 & 0xFF00;
                size += Self::byte_to_int(chunk[1]).overflowing_shl(16).0 & 0xFF0000;
                size += Self::byte_to_int(chunk[0]).overflowing_shl(24).0 & -16777216i32;

                let pad = (-size & 0x1) + (-size & 0x2);

                let start = (128 - size - pad) as usize;
                let end = start + size as usize;

                self.output.extend_from_slice(&chunk[start..end]);

                ct = self.data.read(&mut buff).unwrap();
            }

            // stored size, for debug
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

    pub fn load_dat_file(path: &Path) -> Result<Vec<u8>, ()> {
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
                let exponent = enc_version.get_exponent().unwrap();
                let mut output = vec![];
                let mut decoder = Decoder {
                    modulus,
                    exponent,
                    data: BufReader::new(Cursor::new(&file[28..file.len() - 20])),
                    output: &mut output,
                };

                decoder.decode();

                file = output;
            } else {
                let xor = file[28] ^ (PACKAGE_FILE_TAG & 0xFF) as u8;
                file = file[28..].iter_mut().map(|b| *b ^ xor).collect();
            }
        }

        Ok(file)
    }
}
