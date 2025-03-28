use crate::backend::holder::GameDataHolder;
use crate::backend::log_holder::{Log, LogLevel};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use walkdir::{DirEntry, WalkDir};

mod protocol_166;

pub const NOT_EXIST: &str = "NOT_EXIST";

impl Log {
    fn from_loader_i(val: &str) -> Self {
        Log {
            level: LogLevel::Info,
            producer: "Dat Loader".to_string(),
            log: val.to_string(),
        }
    }

    fn from_loader_e(val: impl Debug) -> Self {
        Log {
            level: LogLevel::Error,
            producer: "Dat Loader".to_string(),
            log: format!("{val:#?}"),
        }
    }
}

pub trait DatLoader {
    fn load_from_binary(&mut self, dat_paths: HashMap<String, DirEntry>) -> Result<Vec<Log>, ()>;
    fn save_to_binary(&mut self, ron_path: &Option<String>) -> std::io::Result<()>;
    fn save_to_ron(&self, folder_path: &str, all: bool) -> std::io::Result<()>;
}

pub fn load_game_data_holder(path: &str) -> Result<(GameDataHolder, Vec<Log>), ()> {
    let mut dat_paths = HashMap::new();

    for path in WalkDir::new(path).into_iter().flatten() {
        if let Ok(meta) = path.metadata() {
            if meta.is_file() && path.file_name().to_str().unwrap().ends_with(".dat") {
                dat_paths.insert(path.file_name().to_str().unwrap().to_lowercase(), path);
            }
        }
    }

    let mut holder = GameDataHolder::default();

    let warnings = holder.load_from_binary(dat_paths).unwrap();

    holder.set_all_holders_unchanged();

    Ok((holder, warnings))
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
        deunicode::deunicode(self).replace('\'', "")
    }
}
