use crate::backend::holder::DataHolder;
use eframe::egui::{
    Color32, Id, Key, Modifiers, PointerButton, Pos2, Rect, Sense, Stroke, Ui, UiBuilder, Vec2,
};
use eframe::epaint::{CubicBezierShape, StrokeKind};
use serde::{Deserialize, Serialize};

pub trait NodeEditorOps {
    fn connected_to(&self) -> Vec<usize>;
    fn add_connection(&mut self, to: usize);
    fn get_pos(&self) -> Pos2;
    fn add_to_pos(&mut self, pos: Vec2);
    fn get_size(&self) -> Vec2;
    fn draw_border(&self) -> bool;
    fn remove_all_input_connection(&mut self);
    fn remove_input_connection(&mut self, index: usize);
}

#[derive(Default, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum NodeEditorConnectionState {
    #[default]
    None,
    CreatingFrom(usize),
    CreateFrom {
        from: usize,
        to: usize,
    },
    CreatingTo(usize),
    RemoveConnectionsTo(usize),
}

#[derive(Default, Serialize, Deserialize)]
pub struct NodeEditorConnectionInfo {
    state: NodeEditorConnectionState,
    pub show: bool,
}

pub trait DrawChild<A> {
    fn draw_tree_child(&mut self, ui: &mut Ui, holders: &DataHolder, action: A, idx: usize);
}

pub fn draw_node_editor<A: Copy, T: NodeEditorOps + Sized + DrawChild<A>>(
    ui: &mut Ui,
    nodes: &mut Vec<T>,
    holders: &DataHolder,
    id: u32,
    info: &mut NodeEditorConnectionInfo,
    action: A,
) {
    let start = ui.cursor().min.to_vec2();

    for node in nodes.iter() {
        let current_node_size = node.get_size();

        for prev in node.connected_to() {
            let Some(prev_node) = nodes.get(prev) else {
                continue;
            };

            let prev_node_size = prev_node.get_size();

            let src_pos = prev_node.get_pos()
                + start
                + Vec2::new(prev_node_size.x / 2., prev_node_size.y + 10.);

            let dst_pos = node.get_pos() + start + Vec2::new(current_node_size.x / 2., -10.);

            let control_scale = ((dst_pos.y - src_pos.y) / 2.0).max(30.0);
            let src_control = src_pos + Vec2::Y * control_scale;
            let dst_control = dst_pos - Vec2::Y * control_scale;

            let bezier = CubicBezierShape::from_points_stroke(
                [src_pos, src_control, dst_control, dst_pos],
                false,
                Color32::TRANSPARENT,
                Stroke {
                    width: 2.0,
                    color: Color32::WHITE,
                },
            );

            ui.painter().add(bezier);
        }
    }

    let mut max_pos = Pos2::ZERO;

    if info.state != NodeEditorConnectionState::None && ui
            .ctx()
            .input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
        info.state = NodeEditorConnectionState::None;
    }

    match &info.state {
        NodeEditorConnectionState::None => {}

        NodeEditorConnectionState::RemoveConnectionsTo(i) => {
            for node in nodes.iter_mut() {
                node.remove_input_connection(*i);
            }
        }

        NodeEditorConnectionState::CreatingFrom(i) => {
            if let Some(node) = nodes.get(*i) {
                let node_size = node.get_size();

                let src_pos =
                    node.get_pos() + start + Vec2::new(node_size.x / 2., node_size.y + 10.);

                if let Some(dst_pos) = ui.ctx().pointer_latest_pos() {
                    let control_scale = ((dst_pos.y - src_pos.y) / 2.0).max(30.0);
                    let src_control = src_pos + Vec2::Y * control_scale;
                    let dst_control = dst_pos - Vec2::Y * control_scale;

                    let bezier = CubicBezierShape::from_points_stroke(
                        [src_pos, src_control, dst_control, dst_pos],
                        false,
                        Color32::TRANSPARENT,
                        Stroke {
                            width: 2.0,
                            color: Color32::WHITE,
                        },
                    );

                    ui.painter().add(bezier);
                }
            }
        }

        NodeEditorConnectionState::CreatingTo(i) => {
            if let Some(node) = nodes.get(*i) {
                let node_size = node.get_size();

                let src_pos = node.get_pos() + start + Vec2::new(node_size.x / 2., -10.);

                if let Some(dst_pos) = ui.ctx().pointer_latest_pos() {
                    let control_scale = ((dst_pos.y - src_pos.y) / 2.0).max(30.0);
                    let src_control = src_pos + Vec2::Y * control_scale;
                    let dst_control = dst_pos - Vec2::Y * control_scale;

                    let bezier = CubicBezierShape::from_points_stroke(
                        [src_pos, src_control, dst_control, dst_pos],
                        false,
                        Color32::TRANSPARENT,
                        Stroke {
                            width: 2.0,
                            color: Color32::WHITE,
                        },
                    );

                    ui.painter().add(bezier);
                }
            }
        }
        NodeEditorConnectionState::CreateFrom { from, to } => {
            if let Some(node) = nodes.get_mut(*to) {
                node.add_connection(*from);
                info.state = NodeEditorConnectionState::None;
            }
        }
    }

    for (i, node) in nodes.iter_mut().enumerate() {
        let rect = Rect::from_min_size(node.get_pos() + start, node.get_size());

        let input_rect = Rect::from_min_size(
            node.get_pos() + start + Vec2::new(0.5, 0.) * node.get_size() - Vec2::new(10., 20.),
            Vec2::new(20., 20.),
        );
        let output_rect = Rect::from_min_size(
            node.get_pos() + start + Vec2::new(0.5, 1.) * node.get_size() + Vec2::new(-10., 0.),
            Vec2::new(20., 20.),
        );

        max_pos = max_pos.max(node.get_pos() + node.get_size());

        ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
            let response = ui.interact(
                rect,
                Id::new(id * 10_000 + i as u32),
                Sense::click_and_drag(),
            );

            if response.dragged() {
                node.add_to_pos(response.drag_delta());
            }

            ui.painter().rect(
                rect,
                10.0,
                ui.ctx().style().visuals.window_fill,
                if node.draw_border() {
                    Stroke::new(2.0, Color32::GRAY)
                } else {
                    Stroke::NONE
                },
                StrokeKind::Outside,
            );

            node.draw_tree_child(ui, holders, action, i);

            let input_response = ui.interact(
                input_rect,
                Id::new(id * 100_000 + i as u32),
                Sense::click_and_drag(),
            );

            if input_response.dragged() {
                info.state = NodeEditorConnectionState::CreatingTo(i);
            } else if input_response.clicked() {
                match info.state {
                    NodeEditorConnectionState::None
                    | NodeEditorConnectionState::RemoveConnectionsTo(_)
                    | NodeEditorConnectionState::CreateFrom { .. } => {}
                    NodeEditorConnectionState::CreatingFrom(ii) => {
                        if ii != i {
                            node.add_connection(ii);
                        }
                        info.state = NodeEditorConnectionState::None;
                    }
                    NodeEditorConnectionState::CreatingTo(_) => {
                        info.state = NodeEditorConnectionState::None;
                    }
                }
            } else if input_response.clicked_by(PointerButton::Secondary) {
                node.remove_all_input_connection();
            }

            ui.painter().rect(
                input_rect,
                10.0,
                if input_response.hovered() {
                    Color32::from_rgb(199, 211, 240)
                } else {
                    Color32::from_rgb(102, 121, 167)
                },
                Stroke::NONE,
                StrokeKind::Outside,
            );

            if node.draw_border() {
                let output_response = ui.interact(
                    output_rect,
                    Id::new(id * 110_000 + i as u32),
                    Sense::click_and_drag(),
                );

                if output_response.dragged() {
                    info.state = NodeEditorConnectionState::CreatingFrom(i);
                } else if output_response.clicked() {
                    match info.state {
                        NodeEditorConnectionState::None
                        | NodeEditorConnectionState::RemoveConnectionsTo(_)
                        | NodeEditorConnectionState::CreateFrom { .. } => {}
                        NodeEditorConnectionState::CreatingFrom(_) => {
                            info.state = NodeEditorConnectionState::None;
                        }
                        NodeEditorConnectionState::CreatingTo(ii) => {
                            if ii != i {
                                info.state =
                                    NodeEditorConnectionState::CreateFrom { from: i, to: ii };
                            } else {
                                info.state = NodeEditorConnectionState::None;
                            }
                        }
                    }
                } else if output_response.clicked_by(PointerButton::Secondary) {
                    info.state = NodeEditorConnectionState::RemoveConnectionsTo(i);
                }

                ui.painter().rect(
                    output_rect,
                    10.0,
                    if output_response.hovered() {
                        Color32::from_rgb(199, 211, 240)
                    } else {
                        Color32::from_rgb(102, 121, 167)
                    },
                    Stroke::NONE,
                    StrokeKind::Outside,
                );
            }
        });
    }

    let rect = Rect::from_min_size(
        ui.max_rect().max + Vec2::new(300.0, 300.0),
        Vec2::new(2., 2.),
    );

    ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
        ui.vertical_centered(|ui| ui.label(""))
    });
}
