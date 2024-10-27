use eframe::egui;
use eframe::egui::{
    vec2, Color32, Id, Pos2, Rect, Sense, Stroke, Ui, UiBuilder, Vec2, ViewportBuilder, Window,
};

pub trait NodeEditorOps {
    fn connected_to(&self) -> Option<usize>;
    fn set_connected_to(&mut self, connected_to: Option<usize>);
    fn get_pos(&self) -> Pos2;
    fn add_to_pos(&mut self, pos: Vec2);
}

struct T {
    pos: Pos2,
}

impl NodeEditorOps for T {
    fn connected_to(&self) -> Option<usize> {
        None
    }

    fn set_connected_to(&mut self, connected_to: Option<usize>) {
        todo!()
    }

    fn get_pos(&self) -> Pos2 {
        self.pos
    }

    fn add_to_pos(&mut self, pos: Vec2) {
        self.pos += pos;
    }
}

struct F {
    t: Vec<T>,
}

impl eframe::App for F {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Window::new(format!("TEST"))
            .id(Id::new("test")) // required since we change the title
            .resizable(true)
            .constrain(true)
            .collapsible(true)
            .title_bar(true)
            .scroll(true)
            .enabled(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| draw_node_editor(ui, &mut self.t, 12));
            });
    }
}

pub fn draw_node_editor<T: NodeEditorOps + Sized>(ui: &mut Ui, nodes: &mut Vec<T>, id: u32) {
    let start = ui.clip_rect().min.to_vec2();
    for (i, node) in nodes[0..2].iter_mut().enumerate() {
        let rect = Rect::from_min_size(node.get_pos() + start, Vec2::new(400., 200.));

        ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
            let response = ui.interact(
                rect,
                Id::new(id * 10000 + i as u32),
                Sense::click_and_drag(),
            );

            if response.dragged() {
                // Update node position based on drag delta
                node.add_to_pos(response.drag_delta());
            }

            ui.painter().rect(
                rect,
                10.0,
                Color32::DARK_GREEN,
                Stroke::new(1.0, Color32::RED),
            );
        });
    }
}

fn main() {
    let viewport = ViewportBuilder::default().with_inner_size(vec2(1200., 540.));

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "La2World Editor",
        options,
        Box::new(|cc| {
            Ok(Box::new(F {
                t: vec![
                    T {
                        pos: Default::default(),
                    },
                    T {
                        pos: Default::default(),
                    },
                    T {
                        pos: Default::default(),
                    },
                ],
            }))
        }),
    );
}
