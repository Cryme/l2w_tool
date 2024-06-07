#![windows_subsystem = "console"]

use crate::frontend::{Frontend, INGAME_WORLD_MAP, NOT_FOUND, WORLD_MAP};
use backend::log_holder::{Log, LogHolder};
use eframe::egui::{vec2, IconData, ImageSource, SizeHint, TextureOptions, ViewportBuilder};
use eframe::{egui, Theme};
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use eframe::emath::Float;

const VERSION: f32 = 1.02;

mod backend;
mod data;
mod entity;
mod frontend;

static APP_LOGS: OnceLock<RwLock<LogHolder>> = OnceLock::new();

fn logs() -> RwLockReadGuard<'static, LogHolder> {
    APP_LOGS.get().unwrap().read().unwrap()
}

fn logs_mut() -> RwLockWriteGuard<'static, LogHolder> {
    APP_LOGS.get().unwrap().write().unwrap()
}

#[allow(unused)]
fn log(log: Log) {
    APP_LOGS.get().unwrap().write().unwrap().add(log);
}

fn log_multiple(logs: Vec<Log>) {
    let mut c = APP_LOGS.get().unwrap().write().unwrap();

    for l in logs {
        c.add(l);
    }
}

fn main() -> Result<(), eframe::Error> {
    APP_LOGS.set(RwLock::new(LogHolder::new())).unwrap();

    let icon = image::load_from_memory(include_bytes!("../../files/logo.png"))
        .expect("Failed to open icon path")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    let viewport = ViewportBuilder::default()
        .with_icon(IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        })
        .with_inner_size(vec2(1200., 540.));

    let options = eframe::NativeOptions {
        viewport,
        follow_system_theme: false,
        default_theme: Theme::Dark,
        ..Default::default()
    };

    eframe::run_native(
        "La2World Editor",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let world_map_source = ImageSource::Bytes {
                uri: "bytes://world_map.png".into(),
                bytes: WORLD_MAP.into(),
            };

            let world_map_id = world_map_source
                .load(
                    &cc.egui_ctx,
                    TextureOptions::default(),
                    SizeHint::Scale(1.0.ord()),
                )
                .unwrap()
                .texture_id()
                .unwrap();

            let ingame_world_map_source = ImageSource::Bytes {
                uri: "bytes://ingame_world_map.png".into(),
                bytes: INGAME_WORLD_MAP.into(),
            };

            let ingame_world_map_id = ingame_world_map_source
                .load(
                    &cc.egui_ctx,
                    TextureOptions::default(),
                    SizeHint::Scale(1.0.ord()),
                )
                .unwrap()
                .texture_id()
                .unwrap();

            let not_found = ImageSource::Bytes {
                uri: "bytes://not_found.png".into(),
                bytes: NOT_FOUND.into(),
            };

            let not_found_texture_id = not_found
                .load(
                    &cc.egui_ctx,
                    TextureOptions::default(),
                    SizeHint::Scale(1.0.ord()),
                )
                .unwrap()
                .texture_id()
                .unwrap();

            Ok(Box::<Frontend>::new(Frontend::init(
                world_map_id,
                ingame_world_map_id,
                not_found_texture_id,
            )))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../files/Nunito-Black.ttf")),
    );
    fonts.font_data.insert(
        "my_icons".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../files/Font Awesome 6 Free-Regular-400.otf"
        )),
    );
    fonts.font_data.insert(
        "my_icons2".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../files/Font Awesome 6 Free-Solid-900.otf"
        )),
    );
    fonts.font_data.insert(
        "my_brands".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../files/Font Awesome 6 Brands-Regular-400.otf"
        )),
    );

    fonts
        .families
        .entry(egui::FontFamily::Name("icons".into()))
        .or_default()
        .push("my_icons".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Name("icons".into()))
        .or_default()
        .push("my_icons2".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Name("icons".into()))
        .or_default()
        .push("my_brands".to_owned());

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
