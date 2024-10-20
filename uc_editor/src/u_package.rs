#![allow(dead_code)]

use l2_rw::ue2_rw::{CompactInt, ReadUnreal, UnrealReader, ASCF, DWORD, GUID, INDEX, INT, WORD};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use r#macro::ReadUnreal;
use crate::script_text::UScriptText;
use crate::un_class::UClass;

#[derive(Debug)]
enum UnrealEntity {
    Class(UClass),
    ScriptText(UScriptText),
    Unsupported
}


#[derive(Debug, Copy, Clone, ReadUnreal)]
struct PackageHeader {
    pub tag: DWORD,
    pub version: WORD,
    pub license: WORD,
    pub unknown1: DWORD,
    pub name_count: DWORD,
    pub name_offset: DWORD,
    pub export_count: DWORD,
    pub export_offset: DWORD,
    pub import_count: DWORD,
    pub import_offset: DWORD,
    pub guid: GUID,
    pub generation_count: DWORD,
}

#[derive(Debug, Clone, ReadUnreal)]
struct NameRecord {
    name: ASCF,
    flags: DWORD,
}

#[derive(Debug, Copy, Clone, ReadUnreal)]
struct GenerationInfo {
    p1: DWORD,
    p2: DWORD
}

#[derive(Debug, Clone, ReadUnreal)]
struct ImportRecord {
    package: INDEX,
    class: INDEX,
    outer: INT,
    name: INDEX,
}

impl ImportRecord {
    fn get_full_path(&self, names: &[NameRecord], imports: &[ImportRecord]) -> String {
        let outer_name = if self.outer != 0 {
            format!("{}.", imports[(-self.outer - 1) as usize].get_full_path(names, imports))
        } else { "".to_string() };

        let self_name = &names[self.name.0 as usize].name;

        format!("{}{}", outer_name, self_name.to_string())
    }
}

#[derive(Debug, Clone)]
struct ExportRecord {
    ///The class of this object. None means this is a class itself.
    class: INDEX,
    ///The object this object inherits from. Only used for struct, states, classes and functions.
    s_super: INDEX,
    ///The object containing this object. Resources may be contained in groups (which are subpackages), functions may be contained in states or classes, and so on.
    outer: INT,
    ///	The index of this object's name in the name table.
    name: INDEX,
    ///This object's flags.
    flags: DWORD,
    data_size: INDEX,
    data_offset: INDEX,
    data: Option<Vec<u8>>,
    children_indexes: Vec<usize>,
}

impl ExportRecord {
    fn is_class(&self) -> bool {
        self.class.0 == 0
    }

    fn parse<
        F1: Fn(INDEX) -> String,
        F2: Fn(INDEX) -> String,
    >(&self, object_name_resolver: &F1, name_resolver: &F2) -> UnrealEntity {
        let Some(data) = &self.data else {
            return UnrealEntity::Unsupported;
        };
        let mut reader = Cursor::new(data);

        if self.is_class() {
            let class = UClass::parse(&mut reader, object_name_resolver, name_resolver);

            return UnrealEntity::Class(class);
        }

        let name = name_resolver(self.name);

        if name == "ScriptText" {
            return UnrealEntity::ScriptText(UScriptText::parse(&mut reader, object_name_resolver, name_resolver));
        }

        UnrealEntity::Unsupported
    }

    fn save_payload<T: Write>(&self, out: &mut T) -> std::io::Result<()> {
        if let Some(data) = &self.data {
            out.write_all(data)
        } else {
            Ok(())
        }
    }
}

impl ReadUnreal for ExportRecord {
    fn read_unreal<T: Read+Seek>(reader: &mut T) -> Self {
        #[derive(ReadUnreal)]
        struct ExportRecordI {
            class: INDEX,
            s_super: INDEX,
            outer: INT,
            name: INDEX,
            flags: DWORD,
            data_size: INDEX,
        }

        let ExportRecordI {
            class,
            s_super,
            outer,
            name,
            flags,
            data_size,
        } = reader.read_unreal_value::<ExportRecordI>();

        let data_offset = if data_size.0 > 0 {
            reader.read_unreal_value::<INDEX>()
        } else {
            CompactInt(-1)
        };

        if data_size.0 == 0 {
            ExportRecord {
                class,
                s_super,
                outer,
                name,
                flags,
                data_size,
                data_offset,
                data: None,
                children_indexes: vec![],
            }
        } else {
            let current_pos = reader.stream_position().unwrap();

            let mut data: Vec<u8> = vec![0u8; data_size.0 as usize];

            reader.seek(SeekFrom::Start(data_offset.0 as u64)).unwrap();
            reader.read_exact(&mut data).unwrap();

            reader.seek(SeekFrom::Start(current_pos)).unwrap();

            ExportRecord {
                class,
                s_super,
                outer,
                name,
                flags,
                data_size,
                data_offset,
                data: Some(data),
                children_indexes: vec![],
            }
        }
    }
}

struct Package {
    header: PackageHeader,
    generations: Vec<GenerationInfo>,
    names: Vec<NameRecord>,
    exports: Vec<ExportRecord>,
    imports: Vec<ImportRecord>,
}

impl Package {
    fn print_info(&self) {
        println!("{:?}", self.header);
    }
    fn load(file_path: &str) -> Result<Package, std::io::Error> {
        let mut file = File::open(file_path)?;

        Ok(Package::read_unreal(&mut file))
    }

    fn print_export_names(&self) {
        for v in &self.exports {
            println!("{}", self.names[v.name.0 as usize].name.to_string());
        }
    }

    fn print_exported_classes(&self) {
        for v in &self.exports {
            if !v.is_class() {
                continue;
            }

            println!("{}", self.names[v.name.0 as usize].name.to_string());
        }
    }

    fn fill_children_indexes(&mut self) {
        for (i, v) in self.exports.clone().iter().enumerate() {
            if v.outer == 0 {
                continue;
            }

            else {
                self.exports[(v.outer - 1) as usize].children_indexes.push(i);
            }
        }
    }

    fn get_name(&self, index: &INDEX) -> String {
        if index.0 > 0 {
            let idx = self.exports[(index.0 - 1) as usize].name;

            self.names[idx.0 as usize].name.to_string()
        } else if index.0 < 0 {
            self.imports[(-index.0 -1) as usize].get_full_path(&self.names, &self.imports)
        } else {
            return "None".to_string();
        }

    }
}

impl ReadUnreal for Package {
    fn read_unreal<T: Read+Seek>(reader: &mut T) -> Self {
        let header = reader.read_unreal_value::<PackageHeader>();
        let generations = reader.read_unreal_value::<Vec<GenerationInfo>>();

        reader
            .seek(SeekFrom::Start(header.name_offset as u64))
            .unwrap();
        let mut names = Vec::with_capacity(header.name_count as usize);
        for _ in 0..header.name_count {
            names.push(reader.read_unreal_value::<NameRecord>());
        }


        reader
            .seek(SeekFrom::Start(header.import_offset as u64))
            .unwrap();
        let mut imports = Vec::with_capacity(header.import_count as usize);
        for _ in 0..header.import_count {
            imports.push(reader.read_unreal_value::<ImportRecord>());
        }

        reader
            .seek(SeekFrom::Start(header.export_offset as u64))
            .unwrap();
        let mut exports = Vec::with_capacity(header.export_count as usize);
        for _ in 0..(header.export_count - 2) {
            exports.push(reader.read_unreal_value::<ExportRecord>());
        }

        Package {
            header,
            generations,
            names,
            exports,
            imports,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f() {
        let package = Package::load("/home/cryme/RustroverProjects/l2w_tool/uc_editor/dec.u").unwrap();

        for export in &package.exports {
            let f1 = |v: INDEX| -> String {
                package.get_name(&v)
            };
            let f2 = |v: INDEX| -> String {
                package.names[v.0 as usize].name.to_string()
            };

            println!("Parsing: {}", f1(export.class));

            let parsed = export.parse(&f1, &f2);

            match parsed {
                UnrealEntity::Class(_) => {}
                UnrealEntity::ScriptText(v) => {
                    println!("{:#?}", v);
                }
                UnrealEntity::Unsupported => {}
            }

            println!("_________________________________________________________________________");

            // if package.names[export.name.0 as usize].name.0 == "guard_naia_b3t\0" {
            //     println!("found");
            //
            //     // let mut f = File::create("./test.uc").unwrap();
            //     //
            //     // export.save_payload(&mut f).unwrap()
            //
            // }
        }
    }
}