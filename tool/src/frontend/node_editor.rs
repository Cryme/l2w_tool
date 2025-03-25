use crate::backend::entity_impl::quest::QuestAction;
use crate::backend::holder::DataHolder;
use eframe::egui::{
    Color32, Context, CursorIcon, Id, Key, Modifiers, PointerButton, Pos2, Rect, Sense, Stroke, Ui,
    UiBuilder, Vec2,
};
use eframe::epaint::{CubicBezierShape, StrokeKind};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

pub trait NodeEditorOps {
    fn connected_to(&self) -> Vec<usize>;
    fn add_connection(&mut self, to: usize);
    fn get_pos(&self) -> Pos2;
    fn add_to_pos(&mut self, pos: Vec2);
    fn get_size(&self) -> Vec2;
    fn is_not_finish(&self) -> bool;
    fn remove_all_input_connection(&mut self);
    fn remove_input_connection(&mut self, index: usize);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
enum GlobalPointerState {
    #[default]
    Any,
    Blocked,
    Drag,
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
enum NodeAction {
    #[default]
    None,
    MoveToStackTop(usize),
    DeleteNode {
        stack_index: usize,
        node_index: usize,
    },
}

#[derive(Default, Serialize, Deserialize)]
pub struct NodeEditorParams {
    action: NodeAction,
    state: NodeEditorConnectionState,
    pointer: GlobalPointerState,
    pub show: bool,
    selected_node: Option<usize>,
    position: Vec2,
    stack: Vec<usize>,
}

impl NodeEditorParams {
    fn handle_key_presses(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.key_pressed(Key::Delete) {
                if let Some(target_node_index) = self.selected_node {
                    for (stack_index, node_index) in self.stack.iter().enumerate() {
                        if target_node_index == *node_index {
                            self.action = NodeAction::DeleteNode {
                                stack_index,
                                node_index: *node_index,
                            }
                        }
                    }
                };
            }
        });
    }

    fn handle_node_action(&mut self, quest_action: &RwLock<QuestAction>) {
        match self.action {
            NodeAction::MoveToStackTop(idx) => {
                let t = self.stack.remove(idx);
                self.stack.push(t);
            }
            NodeAction::DeleteNode {
                stack_index,
                node_index,
            } => {
                if let Some(n) = self.selected_node {
                    if n == node_index {
                        self.selected_node = None;
                    }
                }

                self.stack.remove(stack_index);
                {
                    let mut c = quest_action.write().unwrap();
                    *c = QuestAction::RemoveStep(node_index);
                }

                for v in self.stack.iter_mut() {
                    if *v > node_index {
                        *v -= 1;
                    }
                }
            }

            NodeAction::None => {}
        }

        self.action = NodeAction::None;
    }
}

pub trait DrawChild<A> {
    fn draw_tree_child(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<QuestAction>,
        idx: usize,
    );
}

pub fn draw_node_editor<A: Copy, T: NodeEditorOps + Sized + DrawChild<A>>(
    ui: &mut Ui,
    nodes: &mut Vec<T>,
    holders: &DataHolder,
    id: u32,
    params: &mut NodeEditorParams,
    action: &RwLock<QuestAction>,
) {
    if params.stack.len() != nodes.len() {
        params.selected_node = None;
        params.stack = nodes.iter().enumerate().map(|(i, _)| i).collect();
    }

    let response = ui.interact(ui.cursor(), Id::new("CORE_SCREEN"), Sense::click_and_drag());

    if response.dragged_by(PointerButton::Middle) {
        params.position += response.drag_delta();
    }

    let start = ui.cursor().min.to_vec2() + params.position;

    for i in &params.stack {
        let node = &nodes[*i];

        let current_node_size = node.get_size();

        for prev in node.connected_to() {
            let Some(prev_node) = nodes.get(prev) else {
                continue;
            };

            let prev_node_size = prev_node.get_size();

            let src_pos = prev_node.get_pos()
                + start
                + Vec2::new(prev_node_size.x / 2., prev_node_size.y + 5.);

            let dst_pos = node.get_pos() + start + Vec2::new(current_node_size.x / 2., -5.);

            let control_scale = ((dst_pos.y - src_pos.y) / 2.0).max(30.0);
            let src_control = src_pos + Vec2::Y * control_scale;
            let dst_control = dst_pos - Vec2::Y * control_scale;

            let bezier = CubicBezierShape::from_points_stroke(
                [src_pos, src_control, dst_control, dst_pos],
                false,
                Color32::TRANSPARENT,
                Stroke {
                    width: 2.0,
                    color: Color32::GRAY,
                },
            );

            ui.painter().add(bezier);
        }
    }

    if params.state != NodeEditorConnectionState::None
        && ui
            .ctx()
            .input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape))
    {
        params.state = NodeEditorConnectionState::None;
    }

    match &params.state {
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
                params.state = NodeEditorConnectionState::None;
            }
        }
    }

    for (stack_index, node_index) in params.stack.iter().enumerate() {
        let node = &mut nodes[*node_index];

        let selected = params.selected_node.is_some_and(|v| v == *node_index);

        let rect = Rect::from_min_size(node.get_pos() + start, node.get_size());

        let input_rect = Rect::from_min_size(
            node.get_pos() + start + Vec2::new(0.5, 0.) * node.get_size() - Vec2::new(10., 5.),
            Vec2::new(20., 10.),
        );
        let output_rect = Rect::from_min_size(
            node.get_pos() + start + Vec2::new(0.5, 1.) * node.get_size() + Vec2::new(-10., -5.),
            Vec2::new(20., 10.),
        );

        ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
            let response = ui.interact(
                rect,
                Id::new(id * 10_000 + *node_index as u32),
                Sense::click_and_drag(),
            );

            if response.contains_pointer() && response.ctx.input(|i| i.pointer.any_down()) {
                params.pointer = GlobalPointerState::Blocked;
            }

            if response.dragged() {
                node.add_to_pos(response.drag_delta());
                params.selected_node = Some(*node_index);
                params.pointer = GlobalPointerState::Drag;
            }

            if response.clicked() || response.dragged() {
                params.action = NodeAction::MoveToStackTop(stack_index);
                params.selected_node = Some(*node_index);
            }

            ui.painter().rect(
                rect,
                6.0,
                ui.ctx().style().visuals.window_fill,
                if node.is_not_finish() {
                    Stroke::new(
                        2.0,
                        if selected {
                            Color32::from_rgb(72, 112, 184)
                        } else {
                            Color32::GRAY
                        },
                    )
                } else {
                    Stroke::NONE
                },
                StrokeKind::Outside,
            );

            node.draw_tree_child(ui, holders, action, *node_index);

            let input_response = ui.interact(
                input_rect,
                Id::new(id * 100_000 + *node_index as u32),
                Sense::click_and_drag(),
            );

            if input_response.dragged() {
                params.state = NodeEditorConnectionState::CreatingTo(*node_index);
            } else if input_response.clicked() {
                match params.state {
                    NodeEditorConnectionState::None
                    | NodeEditorConnectionState::RemoveConnectionsTo(_)
                    | NodeEditorConnectionState::CreateFrom { .. } => {}
                    NodeEditorConnectionState::CreatingFrom(ii) => {
                        if ii != *node_index {
                            node.add_connection(ii);
                        }
                        params.state = NodeEditorConnectionState::None;
                    }
                    NodeEditorConnectionState::CreatingTo(_) => {
                        params.state = NodeEditorConnectionState::None;
                    }
                }
            } else if input_response.clicked_by(PointerButton::Secondary) {
                node.remove_all_input_connection();
            }

            ui.painter().rect(
                input_rect,
                10.0,
                if input_response.hovered() {
                    Color32::LIGHT_GRAY
                } else {
                    Color32::GRAY
                },
                Stroke::NONE,
                StrokeKind::Outside,
            );

            if node.is_not_finish() {
                let output_response = ui.interact(
                    output_rect,
                    Id::new(id * 110_000 + *node_index as u32),
                    Sense::click_and_drag(),
                );

                if output_response.dragged() {
                    params.state = NodeEditorConnectionState::CreatingFrom(*node_index);
                } else if output_response.clicked() {
                    match params.state {
                        NodeEditorConnectionState::None
                        | NodeEditorConnectionState::RemoveConnectionsTo(_)
                        | NodeEditorConnectionState::CreateFrom { .. } => {}
                        NodeEditorConnectionState::CreatingFrom(_) => {
                            params.state = NodeEditorConnectionState::None;
                        }
                        NodeEditorConnectionState::CreatingTo(ii) => {
                            if ii != *node_index {
                                params.state = NodeEditorConnectionState::CreateFrom {
                                    from: *node_index,
                                    to: ii,
                                };
                            } else {
                                params.state = NodeEditorConnectionState::None;
                            }
                        }
                    }
                } else if output_response.clicked_by(PointerButton::Secondary) {
                    params.state = NodeEditorConnectionState::RemoveConnectionsTo(*node_index);
                }

                ui.painter().rect(
                    output_rect,
                    10.0,
                    if output_response.hovered() {
                        Color32::LIGHT_GRAY
                    } else {
                        Color32::GRAY
                    },
                    Stroke::NONE,
                    StrokeKind::Outside,
                );
            }
        });
    }

    if ui.ctx().input(|s| s.pointer.any_released()) {
        params.pointer = GlobalPointerState::Any;
    }

    if params.pointer == GlobalPointerState::Any
        && response.contains_pointer()
        && response.ctx.input(|s| s.pointer.any_down())
    {
        params.pointer = GlobalPointerState::Drag;
    }

    match params.pointer {
        GlobalPointerState::Drag => {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }

        GlobalPointerState::Any | GlobalPointerState::Blocked => {}
    }

    params.handle_key_presses(ui.ctx());
    params.handle_node_action(action);
}
