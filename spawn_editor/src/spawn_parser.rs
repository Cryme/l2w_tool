use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::Arc;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};
use xml::ParserConfig;

use crate::backend::{
    MapPoint, NpcInfo, PointSpawn, Region, Spawn, SpawnInfo, SpawnPeriodOfDay, TerritoryInfo,
    TerritoryInfoRegion, TerritorySpawn,
};

pub(crate) trait ParseElement {
    fn parse_element<R: Read>(
        reader: &mut EventReader<R>,
        attributes: &Vec<OwnedAttribute>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl ParseElement for TerritorySpawn {
    fn parse_element<R: Read>(
        reader: &mut EventReader<R>,
        attributes: &Vec<OwnedAttribute>,
    ) -> anyhow::Result<Self> {
        let mut name = None;
        let mut super_point = None;

        for v in attributes {
            match v.name.local_name.as_ref() {
                "name" => {
                    name = Some(v.value.clone());
                }
                "superPoint" => {
                    super_point = Some(v.value.clone());
                }
                _ => {}
            }
        }

        let territory;
        if let Some(name) = name {
            territory = TerritoryInfoRegion::Named(name);
            reader.skip().unwrap();
        } else {
            territory =
                TerritoryInfoRegion::Inlined(TerritoryInfo::parse_element(reader, attributes)?);
        }

        Ok(TerritorySpawn {
            territory,
            super_point,
        })
    }
}

impl ParseElement for PointSpawn {
    fn parse_element<R: Read>(
        reader: &mut EventReader<R>,
        attributes: &Vec<OwnedAttribute>,
    ) -> anyhow::Result<Self> {
        let mut x = None;
        let mut y = None;
        let mut z = None;
        let mut h = None;
        let mut super_point = None;

        for v in attributes {
            match v.name.local_name.as_ref() {
                "x" => {
                    x = Some(v.value.parse::<i32>()?);
                }
                "y" => {
                    y = Some(v.value.parse::<i32>()?);
                }
                "z" => {
                    z = Some(v.value.parse::<i32>()?);
                }
                "h" => {
                    h = Some(v.value.parse::<i32>()?);
                }
                "superPoint" => {
                    super_point = Some(v.value.clone());
                }
                _ => {
                    println!("Unknown Point attribute: {} {}", v.name, v.value);
                }
            }
        }

        reader.skip().unwrap();

        Ok(PointSpawn {
            x: x.unwrap(),
            y: y.unwrap(),
            z: z.unwrap(),
            heading: if let Some(h) = h { h } else { 0 },
            super_point,
        })
    }
}

impl ParseElement for Region {
    fn parse_element<R: Read>(
        reader: &mut EventReader<R>,
        _attributes: &Vec<OwnedAttribute>,
    ) -> anyhow::Result<Self> {
        let mut res = Vec::new();

        loop {
            match reader.next() {
                Ok(e) => match e {
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => {
                        if name.local_name == "add" {
                            res.push(MapPoint::parse_element(reader, &attributes)?);
                        }
                    }
                    XmlEvent::EndElement { .. } => {
                        break;
                    }
                    _ => {}
                },

                Err(e) => {
                    eprintln!("Error at {}: {e}", reader.position());
                    break;
                }
            }
        }

        Ok(res)
    }
}

impl ParseElement for TerritoryInfo {
    fn parse_element<R: Read>(
        reader: &mut EventReader<R>,
        attributes: &Vec<OwnedAttribute>,
    ) -> anyhow::Result<Self> {
        let mut region = Vec::new();
        let mut banned_regions = Vec::new();
        let mut name = None;

        loop {
            match reader.next() {
                Ok(e) => match e {
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => {
                        if name.local_name == "add" {
                            region.push(MapPoint::parse_element(reader, &attributes)?);
                        } else if name.local_name == "banned_territory" {
                            banned_regions.push(Region::parse_element(reader, &attributes)?);
                        }
                    }
                    XmlEvent::EndElement { .. } => {
                        break;
                    }
                    _ => {}
                },

                Err(e) => {
                    eprintln!("Error at {}: {e}", reader.position());
                    break;
                }
            }
        }

        for att in attributes {
            if att.name.local_name == "name" {
                name = Some(att.value.clone());

                break;
            }
        }

        let mut z_min = i32::MAX;
        let mut z_max = i32::MIN;

        for point in &region {
            z_min = point.z_min.min(z_min);
            z_max = point.z_max.max(z_max);
        }

        Ok(TerritoryInfo {
            region,
            banned_regions,
            name,
            z_min,
            z_max,
        })
    }
}

impl ParseElement for SpawnInfo {
    fn parse_element<R: Read>(
        reader: &mut EventReader<R>,
        attributes: &Vec<OwnedAttribute>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut group = None;
        let mut count = 1;
        let mut respawn_sec = 60;
        let mut respawn_random_sec = 0;
        let mut period_of_day = SpawnPeriodOfDay::NONE;

        for v in attributes {
            match v.name.local_name.as_ref() {
                "group" => {
                    group = Some(v.value.clone());
                }
                "respawn" => {
                    respawn_sec = v.value.parse::<u32>()?;
                }
                "respawn_random" => {
                    respawn_random_sec = v.value.parse::<u32>()?;
                }
                "count" => {
                    count = v.value.parse::<u32>()?;
                }
                "period_of_day" => match v.value.to_lowercase().as_ref() {
                    "day" => period_of_day = SpawnPeriodOfDay::DAY,
                    "night" => period_of_day = SpawnPeriodOfDay::NIGHT,
                    _ => {}
                },
                _ => {}
            }
        }

        let mut npc = Vec::new();
        let mut spawn = Vec::new();

        loop {
            match reader.next() {
                Ok(e) => match e {
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => match name.local_name.as_ref() {
                        "point" => {
                            spawn.push(Spawn::Point(PointSpawn::parse_element(
                                reader,
                                &attributes,
                            )?));
                        }
                        "territory" => {
                            spawn.push(Spawn::Territory(TerritorySpawn::parse_element(
                                reader,
                                &attributes,
                            )?));
                        }
                        "territoryName" => {
                            let mut ters = Vec::new();
                            for att in attributes {
                                if att.name.local_name == "name" {
                                    ters.extend(att.value.split(';').map(|v| v.to_string()))
                                }
                            }
                            spawn.push(Spawn::RandomTerritory(ters));

                            reader.skip().unwrap();
                        }
                        "npc" => {
                            let mut id = None;
                            let mut max = 0;

                            for att in attributes {
                                if att.name.local_name == "id" {
                                    id = Some(att.value.parse::<u32>()?);
                                } else if att.name.local_name == "max" {
                                    max = att.value.parse::<u32>()?;
                                }
                            }

                            npc.push(NpcInfo {
                                id: id.unwrap(),
                                max,
                            });

                            reader.skip().unwrap();
                        }
                        "debug" => {
                            reader.skip().unwrap();
                        }
                        _ => {
                            println!(
                                "Unknown spawn info element {} {:#?}",
                                name.local_name, attributes
                            );
                            reader.skip().unwrap();
                        }
                    },
                    XmlEvent::EndElement { .. } => {
                        break;
                    }
                    _ => {}
                },

                Err(e) => {
                    eprintln!("Error at {}: {e}", reader.position());
                    break;
                }
            }
        }

        Ok(SpawnInfo {
            group: if let Some(group) = group {
                group
            } else {
                period_of_day.name()
            },
            count,
            respawn_sec,
            respawn_random_sec,
            period_of_day,
            npc,
            spawns: spawn,
            map_squares: HashSet::new(),
            file_name: None,
        })
    }
}

impl ParseElement for MapPoint {
    fn parse_element<R: Read>(
        reader: &mut EventReader<R>,
        attributes: &Vec<OwnedAttribute>,
    ) -> anyhow::Result<Self> {
        let mut x = None;
        let mut y = None;
        let mut z_min = None;
        let mut z_max = None;

        for v in attributes {
            match v.name.local_name.as_ref() {
                "x" => {
                    x = Some(v.value.parse::<i32>()?);
                }
                "y" => {
                    y = Some(v.value.parse::<i32>()?);
                }
                "zmin" => {
                    z_min = Some(v.value.parse::<i32>()?);
                }
                "zmax" => {
                    z_max = Some(v.value.parse::<i32>()?);
                }
                _ => {
                    println!("Unknown Point attribute: {} {}", v.name, v.value);
                }
            }
        }

        reader.skip().unwrap();

        Ok(MapPoint {
            x: x.unwrap(),
            y: y.unwrap(),
            z_min: z_min.unwrap(),
            z_max: z_max.unwrap(),
        })
    }
}

pub(crate) fn parse_file<P: AsRef<Path>>(
    file_path: P,
    spawns: &mut Vec<SpawnInfo>,
    territories: &mut HashMap<String, TerritoryInfo>,
) -> anyhow::Result<()> {
    let path = Arc::new(file_path.as_ref().to_str().unwrap().to_string());

    let file = File::open(file_path)?;
    let file = BufReader::new(file); // Buffering is important for performance

    let mut reader = ParserConfig::default()
        .ignore_root_level_whitespace(false)
        .create_reader(BufReader::new(file));

    loop {
        match reader.next() {
            Ok(e) => match e {
                XmlEvent::EndDocument => {
                    break;
                }
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    if name.local_name == "spawn" {
                        let mut spawn = SpawnInfo::parse_element(&mut reader, &attributes)?;

                        spawn.file_name = Some(path.clone());
                        spawns.push(spawn);
                    } else if name.local_name == "territory" {
                        let tet = TerritoryInfo::parse_element(&mut reader, &attributes)?;

                        if let Some(name) = tet.name.clone() {
                            if territories.insert(name.clone(), tet).is_some() {
                                println!("Duplicated territory: {}", name);
                            }
                        }
                    }
                }
                _ => {}
            },
            Err(e) => {
                eprintln!("Error at {}: {e}", reader.position());
                break;
            }
        }
    }

    Ok(())
}

pub const L2_SERVER_ROOT_SPAWN_FOLDER: &str = "/data/spawn/";
