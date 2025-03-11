use crate::ue2_rw::{CompactInt, ReadUnreal, UnrealWriter, WriteUnreal, INDEX};
use byteorder::WriteBytesExt;
use inflate::inflate_bytes_zlib_no_checksum;
use miniz_oxide::deflate::compress_to_vec_zlib;
use openssl::bn::BigNum;
use openssl::rsa::Padding;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, Write};
use std::path::Path;

pub mod ue2_rw;

pub const PACKAGE_FILE_TAG: u32 = 0x9E2A83C1;
pub const LINEAGE_HEADER: &[u8; 22] = b"L\x00i\x00n\x00e\x00a\x00g\x00e\x002\x00V\x00e\x00r\x00";
pub const END_BYTES: &[u8; 20] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100];
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
            EncVersion::V413 => Some(BigNum::from_hex_str("1d").unwrap()),
            EncVersion::V414 => Some(BigNum::from_hex_str("25").unwrap()),
        }
    }
}

struct Decoder<'a, T: Read> {
    data: T,
    output: &'a mut Vec<u8>,
}

impl<T: Read> Decoder<'_, T> {
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

        let res = inflate_bytes_zlib_no_checksum(&self.output[4..self.output.len() - 4]).unwrap();

        *self.output = res;
    }
}

pub fn deserialize_dat_with_string_dict<S: ReadUnreal + Debug, T: ReadUnreal + Debug>(
    file_path: &Path,
) -> Result<(Vec<S>, Vec<T>), ()> {
    println!("Loading {file_path:?}...");
    let Ok(bytes) = read_encoded_file(file_path) else {
        return Err(());
    };

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
    let Ok(bytes) = read_encoded_file(file_path) else {
        println!("Err!");
        return Err(());
    };

    let bugged = file_path
        .to_str()
        .unwrap()
        .to_lowercase()
        .ends_with("_baseinfo.dat");

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

pub fn read_encoded_file(path: &Path) -> Result<Vec<u8>, ()> {
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
