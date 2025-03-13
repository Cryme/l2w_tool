use l2_rw::ue2_rw::{ASCF, DWORD, USHORT};
use l2_rw::{deserialize_dat, save_dat, DatVariant};

use l2_rw::ue2_rw::{ReadUnreal, UnrealReader, UnrealWriter, WriteUnreal};

use crate::backend::holder::{GameDataHolder, HolderMapOps, L2GeneralStringTable};
use crate::backend::log_holder::Log;
use crate::entity::residence::Residence;
use r#macro::{ReadUnreal, WriteUnreal};
use std::thread;
use std::thread::JoinHandle;

impl From<(&Residence, &mut L2GeneralStringTable)> for CastleNameDat {
    fn from(value: (&Residence, &mut L2GeneralStringTable)) -> Self {
        let (zone, table) = value;

        CastleNameDat {
            number: 0,
            tag: 1,
            id: zone.id.0,
            name: (&zone.name).into(),
            loc: (&zone.territory).into(),
            desc: (&zone.desc).into(),
            mark: table.get_index(&zone.mark),
            mark_grey: table.get_index(&zone.mark_grey),
            flag_icon: table.get_index(&zone.flag_icon),
            merc_name: (&zone.merc_name).into(),
            region_id: zone.region_id,
        }
    }
}

impl GameDataHolder {
    pub fn serialize_residence_to_binary(&mut self) -> JoinHandle<Log> {
        let residences = self
            .residence_holder
            .values()
            .map(|v| (v, &mut self.game_string_table).into())
            .collect();

        let residence_path = self
            .dat_paths
            .get(&"castlename-ru.dat".to_string())
            .unwrap()
            .clone();

        thread::spawn(move || {
            if let Err(e) = save_dat(
                residence_path.path(),
                DatVariant::<(), CastleNameDat>::Array(residences),
            ) {
                Log::from_loader_e(e)
            } else {
                Log::from_loader_i("Residences saved")
            }
        })
    }

    pub fn load_residences(&mut self) -> Result<Vec<Log>, ()> {
        let warnings = vec![];

        let residences = deserialize_dat::<CastleNameDat>(
            self.dat_paths
                .get(&"castlename-ru.dat".to_string())
                .unwrap()
                .path(),
        )?;

        for v in residences {
            self.residence_holder.insert(
                v.id.into(),
                Residence {
                    id: v.id.into(),
                    name: v.name.to_string(),
                    desc: v.desc.to_string(),
                    territory: v.loc.to_string(),
                    mark: self.game_string_table.get_o(&v.mark).into(),
                    mark_grey: self.game_string_table.get_o(&v.mark_grey).into(),
                    flag_icon: self.game_string_table.get_o(&v.flag_icon).into(),
                    merc_name: v.merc_name.to_string(),
                    region_id: v.region_id,
                    ..Default::default()
                },
            );
        }

        Ok(warnings)
    }
}

#[derive(Debug, Clone, PartialEq, ReadUnreal, WriteUnreal)]
struct CastleNameDat {
    number: DWORD,
    tag: DWORD,
    id: DWORD,
    name: ASCF,
    loc: ASCF,
    desc: ASCF,
    mark: DWORD,
    mark_grey: DWORD,
    flag_icon: DWORD,
    merc_name: ASCF,
    region_id: USHORT,
}
