use crate::backend::Log;
use crate::dat_loader::grand_crusade_110::Loader110;
use crate::holder::{ChroniclesProtocol, GameDataHolder};
use std::collections::HashMap;
use walkdir::{DirEntry, WalkDir};

mod grand_crusade_110;

pub trait DatLoader {
    fn load(&mut self, dat_paths: HashMap<String, DirEntry>) -> Result<Vec<Log>, ()>;
    fn from_holder(game_data_holder: &GameDataHolder) -> Self;
    fn to_holder(self) -> GameDataHolder;
    fn serialize_to_binary(&mut self) -> std::io::Result<()>;
}

fn get_loader_for_protocol(protocol: ChroniclesProtocol) -> Result<impl DatLoader + Sized, ()> {
    Ok(match protocol {
        ChroniclesProtocol::GrandCrusade110 => Loader110::default(),
    })
}

pub fn get_loader_from_holder(holder: &GameDataHolder) -> impl DatLoader + Sized {
    match holder.protocol_version {
        ChroniclesProtocol::GrandCrusade110 => Loader110::from_holder(holder),
    }
}

pub fn load_game_data_holder(
    path: &str,
    protocol: ChroniclesProtocol,
) -> Result<(GameDataHolder, Vec<Log>), ()> {
    let mut dat_paths = HashMap::new();

    for path in WalkDir::new(path).into_iter().flatten() {
        if let Ok(meta) = path.metadata() {
            if meta.is_file() && path.file_name().to_str().unwrap().ends_with(".dat") {
                dat_paths.insert(path.file_name().to_str().unwrap().to_lowercase(), path);
            }
        }
    }

    let mut loader = get_loader_for_protocol(protocol)?;
    let warnings = loader.load(dat_paths)?;

    Ok((loader.to_holder(), warnings))
}
