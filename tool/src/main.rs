#![windows_subsystem = "console"]
use crate::frontend::Frontend;
use eframe::{egui, IconData, Theme};

mod backend;
mod data;
mod entity;
mod frontend;
mod holders;
mod server_side;
mod util;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 540.0)),
        icon_data: Some(
            IconData::try_from_png_bytes(include_bytes!("../../files/logo.png")).unwrap(),
        ),
        follow_system_theme: false,
        default_theme: Theme::Dark,
        ..Default::default()
    };

    eframe::run_native(
        "La2World Quest Editor",
        options,
        Box::new(|cc| Box::<Frontend>::new(Frontend::init(cc))),
    )
}
