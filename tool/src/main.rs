#![windows_subsystem = "console"]

use crate::frontend::Frontend;
use eframe::{egui, IconData};

mod backend;
mod data;
mod entity;
mod frontend;
mod holders;
mod server_side;
mod util;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1060.0, 540.0)),
        icon_data: Some(IconData::try_from_png_bytes(include_bytes!("../../logo.png")).unwrap()),
        ..Default::default()
    };

    eframe::run_native(
        "La2World Quest Editor",
        options,
        Box::new(|cc| Box::<Frontend>::new(Frontend::init(cc))),
    )
}
