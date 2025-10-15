use crate::backend::entity_impl::quest::QuestAction;
use crate::backend::holder::DataHolder;
use crate::entity::quest::{QuestStep, UnkQLevel};
use crate::frontend::DELETE_ICON;
use crate::frontend::util::{
    Draw, DrawUtils, combo_box_row, num_row, polylines_intersect, text_row, text_row_multiline,
};
use eframe::egui::{
    Align, Color32, Context, CursorIcon, Direction, FontFamily, Frame, Id, Key, Label, Layout,
    Modifiers, PointerButton, Pos2, Rect, Response, RichText, ScrollArea, Sense, Shape, Stroke, Ui,
    UiBuilder, Vec2, vec2,
};
use eframe::epaint;
use eframe::epaint::{CubicBezierShape, StrokeKind};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use strum::IntoEnumIterator;

const EXPANDED_WIDTH: f32 = 750.;

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
    RemoveConnections(Vec<(usize, usize)>),
}

#[derive(Default, Serialize, Deserialize)]
enum NodeAction {
    #[default]
    None,
    MoveToStackTop(usize),
    DeleteNodes,
    AddNode,
}

#[derive(Default, Serialize, Deserialize)]
pub struct QuestStepsEditorParams {
    action: NodeAction,
    state: NodeEditorConnectionState,
    pointer: GlobalPointerState,
    pub show: bool,
    selected_nodes: Vec<usize>,
    position: Vec2,
    stack: Vec<usize>,
    last_click_pos_for_select: Option<Pos2>,

    disconnect_line: Vec<Pos2>,
    nodes_connections: Vec<(usize, usize, Vec<Pos2>)>,
}

impl QuestStepsEditorParams {
    fn handle_key_presses(&mut self, ctx: &Context, nodes: &mut Vec<QuestStep>) {
        if ctx.wants_keyboard_input() {
            return;
        }

        ctx.input_mut(|i| {
            if i.consume_key(Modifiers::NONE, Key::Delete) {
                self.action = NodeAction::DeleteNodes;
            } else if i.consume_key(Modifiers::CTRL, Key::A) {
                self.selected_nodes = (0..nodes.len()).collect();
            } else if i.consume_key(Modifiers::NONE, Key::A) {
                self.action = NodeAction::AddNode;
            } else if i.consume_key(Modifiers::NONE, Key::E) {
                for selected_node_index in self.selected_nodes.iter() {
                    nodes[*selected_node_index].collapsed = false;
                }
            } else if i.consume_key(Modifiers::NONE, Key::C) {
                for selected_node_index in self.selected_nodes.iter() {
                    nodes[*selected_node_index].collapsed = true;
                }
            } else if i.consume_key(Modifiers::NONE, Key::Escape) {
                self.selected_nodes.clear();
            }
        });
    }

    fn handle_node_action(
        &mut self,
        quest_action: &RwLock<QuestAction>,
        nodes: &mut Vec<QuestStep>,
    ) {
        match self.action {
            NodeAction::MoveToStackTop(idx) => {
                let t = self.stack.remove(idx);
                self.stack.push(t);
            }

            NodeAction::DeleteNodes => {
                self.selected_nodes.sort();
                let mut stack_indexes = Vec::with_capacity(self.selected_nodes.len());

                'outer: for node_index in &self.selected_nodes {
                    for (i, idx) in self.stack.iter().enumerate() {
                        if idx == node_index {
                            stack_indexes.push(i);

                            continue 'outer;
                        }
                    }
                }

                stack_indexes.sort();

                for idx in stack_indexes.into_iter().rev() {
                    let n_idx = self.stack[idx];

                    self.stack.remove(idx);

                    for v in self.stack.iter_mut() {
                        if *v > n_idx {
                            *v -= 1;
                        }
                    }
                }

                {
                    let mut c = quest_action.write().unwrap();
                    *c = QuestAction::RemoveSteps(self.selected_nodes.clone());
                }

                self.selected_nodes.clear();
            }

            NodeAction::AddNode => {
                self.stack.push(nodes.len());
                nodes.push(QuestStep::default());
            }

            NodeAction::None => {}
        }

        self.action = NodeAction::None;
    }
}

pub fn disconnector(ui: &mut Ui, params: &mut QuestStepsEditorParams, response: Response) {
    if ui.ctx().wants_keyboard_input() {
        return;
    }

    if ui.ctx().input(|i| i.key_released(Key::Y)) {
        let mut disconnected = vec![];
        for (node_idx, prev_idx, line) in params.nodes_connections.clone() {
            if polylines_intersect(&params.disconnect_line, &line) {
                disconnected.push((node_idx, prev_idx));
            }
        }

        if !disconnected.is_empty() {
            params.state = NodeEditorConnectionState::RemoveConnections(disconnected);
        }

        params.disconnect_line.clear();
        params.nodes_connections.clear();
    }

    if !ui.ctx().input(|i| i.keys_down.contains(&Key::Y)) {
        return;
    }

    if let Some(pos) = response.hover_pos()
        && (params.disconnect_line.is_empty()
            || params.disconnect_line[params.disconnect_line.len() - 1].distance(pos) > 2.)
        {
            params.disconnect_line.push(pos);
        };

    ui.painter().add(Shape::line(
        params.disconnect_line.clone(),
        Stroke::new(1.0, Color32::DARK_RED),
    ));
}

pub fn draw_node_editor(
    ui: &mut Ui,
    nodes: &mut Vec<QuestStep>,
    holders: &DataHolder,
    id: u32,
    params: &mut QuestStepsEditorParams,
    action: &RwLock<QuestAction>,
) {
    let shift_pressed = ui.ctx().input(|i| i.modifiers.shift);
    let alt_pressed = ui.ctx().input(|i| i.modifiers.alt);
    let space_pressed =
        ui.ctx().input(|i| i.keys_down.contains(&Key::Space)) && !ui.ctx().wants_keyboard_input();

    let y_pressed =
        ui.ctx().input(|i| i.keys_down.contains(&Key::Y)) && !ui.ctx().wants_keyboard_input();

    let y_released = ui.ctx().input(|i| i.key_released(Key::Y)) && !ui.ctx().wants_keyboard_input();

    if params.stack.len() != nodes.len() {
        params.selected_nodes.clear();
        params.stack = nodes.iter().enumerate().map(|(i, _)| i).collect();
    }

    let response = ui.interact(ui.cursor(), Id::new("CORE_SCREEN"), Sense::click_and_drag());

    let is_grid_drag = response.dragged_by(PointerButton::Middle)
        || (response.dragged_by(PointerButton::Primary)
            && space_pressed
            && !y_pressed
            && !shift_pressed
            && !alt_pressed);

    if response.clicked() && !shift_pressed && !alt_pressed && !space_pressed {
        params.selected_nodes.clear();
    }

    if is_grid_drag {
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

            if y_released {
                params
                    .nodes_connections
                    .push((*i, prev, bezier.flatten(Some(1.0))))
            }

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

        NodeEditorConnectionState::RemoveConnections(vec) => {
            for (from, prev) in vec {
                nodes[*from].remove_input_connection(*prev);
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

    let mut drag = None;

    for (stack_index, node_index) in params.stack.iter().enumerate() {
        let node = &mut nodes[*node_index];

        let selected = params.selected_nodes.contains(node_index);

        let rect = Rect::from_min_size(node.get_pos() + start, node.get_size());

        let input_rect = Rect::from_min_size(
            node.get_pos() + start + Vec2::new(0.5, 0.) * node.get_size() - Vec2::new(10., 6.),
            Vec2::new(20., 6.),
        );
        let output_rect = Rect::from_min_size(
            node.get_pos() + start + Vec2::new(0.5, 1.) * node.get_size() + Vec2::new(-10., 0.),
            Vec2::new(20., 6.),
        );

        ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
            let response = ui.interact(
                rect,
                Id::new(id as u64 * 10_000 + *node_index as u64),
                Sense::click_and_drag(),
            );

            if response.contains_pointer() && response.ctx.input(|i| i.pointer.any_down()) {
                params.pointer = GlobalPointerState::Blocked;
            }

            if response.dragged() {
                if alt_pressed {
                    node.add_to_pos(response.drag_delta());
                } else {
                    if !params.selected_nodes.contains(node_index) {
                        if !shift_pressed {
                            params.selected_nodes.clear();
                        }

                        params.selected_nodes.push(*node_index);
                    }

                    drag = Some(response.drag_delta());
                }

                params.pointer = GlobalPointerState::Drag;
            }

            if response.clicked() {
                params.action = NodeAction::MoveToStackTop(stack_index);

                if alt_pressed {
                    params.selected_nodes.retain(|v| *v != *node_index);
                } else if shift_pressed && !params.selected_nodes.contains(node_index) {
                    params.selected_nodes.push(*node_index);
                } else {
                    params.selected_nodes.clear();
                    params.selected_nodes.push(*node_index);
                }
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
                Id::new(id as u64 * 100_000 + *node_index as u64),
                Sense::click_and_drag(),
            );

            if input_response.dragged() {
                params.state = NodeEditorConnectionState::CreatingTo(*node_index);
            } else if input_response.clicked() {
                match params.state {
                    NodeEditorConnectionState::None => {
                        params.state = NodeEditorConnectionState::CreatingTo(*node_index);
                    }

                    NodeEditorConnectionState::RemoveConnectionsTo(_)
                    | NodeEditorConnectionState::RemoveConnections(_)
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
                    Id::new(id as u64 * 110_000 + *node_index as u64),
                    Sense::click_and_drag(),
                );

                if output_response.dragged() {
                    params.state = NodeEditorConnectionState::CreatingFrom(*node_index);
                } else if output_response.clicked() {
                    match params.state {
                        NodeEditorConnectionState::None => {
                            params.state = NodeEditorConnectionState::CreatingFrom(*node_index);
                        }

                        NodeEditorConnectionState::RemoveConnectionsTo(_)
                        | NodeEditorConnectionState::RemoveConnections(_)
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

    if let Some(drag) = drag {
        for node_index in params.selected_nodes.iter() {
            nodes[*node_index].add_to_pos(drag)
        }
    }

    if ui.ctx().input(|s| s.pointer.any_released()) {
        params.pointer = GlobalPointerState::Any;
    }

    if params.pointer == GlobalPointerState::Any
        && response.contains_pointer()
        && response.ctx.input(|s| s.pointer.middle_down())
    {
        params.pointer = GlobalPointerState::Drag;
    }

    if params.pointer == GlobalPointerState::Drag || space_pressed {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }

    if !is_grid_drag && !y_pressed {
        select_box(ui, nodes, params, &response, start);
    } else if params.last_click_pos_for_select.is_some() && !y_pressed {
        params.last_click_pos_for_select = None;
    }

    draw_legend(ui);
    disconnector(ui, params, response);

    params.handle_key_presses(ui.ctx(), nodes);
    params.handle_node_action(action, nodes);
}

fn select_box(
    ui: &mut Ui,
    nodes: &mut Vec<QuestStep>,
    params: &mut QuestStepsEditorParams,
    response: &Response,
    start: Vec2,
) {
    let mut boxed_zoom_rect = None;
    let shift_pressed = ui.ctx().input(|i| i.modifiers.shift);
    let alt_pressed = ui.ctx().input(|i| i.modifiers.alt);

    if response.drag_started() && response.dragged_by(PointerButton::Primary) {
        params.last_click_pos_for_select = response.hover_pos();
    }

    if !response.dragged_by(PointerButton::Primary) {
        return;
    }

    let box_start_pos = params.last_click_pos_for_select;
    let box_end_pos = response.hover_pos();

    if let (Some(box_start_pos), Some(box_end_pos)) = (box_start_pos, box_end_pos) {
        let sx = box_start_pos.x.min(box_end_pos.x);
        let ex = box_start_pos.x.max(box_end_pos.x);

        let sy = box_start_pos.y.min(box_end_pos.y);
        let ey = box_start_pos.y.max(box_end_pos.y);

        let rect = Rect::from([Pos2::new(sx, sy), Pos2::new(ex, ey)]);

        if !shift_pressed && !alt_pressed {
            params.selected_nodes.clear();
        }

        for (i, node) in nodes.iter().enumerate() {
            if rect.intersects(Rect::from([
                node.pos + start,
                node.pos + node.get_size() + start,
            ])) {
                if alt_pressed {
                    params.selected_nodes.retain(|v| *v != i)
                } else if !shift_pressed || !params.selected_nodes.contains(&i) {
                    params.selected_nodes.push(i);
                }
            }
        }

        let rect = Rect::from_two_pos(box_start_pos, box_end_pos);

        boxed_zoom_rect = Some(epaint::RectShape::stroke(
            rect,
            0.0,
            Stroke::new(1., Color32::from_rgb(254, 236, 183)),
            StrokeKind::Outside,
        ));

        if response.drag_stopped() {
            params.last_click_pos_for_select = None;
        }
    }

    if let Some(boxed_zoom_rect) = boxed_zoom_rect {
        ui.painter().add(boxed_zoom_rect);
    }
}

fn draw_legend(ui: &mut Ui) {
    ui.add(|ui: &mut Ui| {
        let layout = Layout::from_main_dir_and_cross_align(Direction::TopDown, Align::RIGHT);

        let mut legend_ui = ui.child_ui(ui.clip_rect().shrink(4.0), layout, None);

        legend_ui
            .scope(|ui| {
                Frame {
                    inner_margin: vec2(8.0, 4.0).into(),
                    corner_radius: ui.style().visuals.window_corner_radius,
                    shadow: epaint::Shadow::NONE,
                    fill: Color32::from_rgba_premultiplied(0, 0, 0, 125),
                    stroke: ui.style().visuals.window_stroke(),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_width(145.);

                    let mut resp = ui.add(
                        Label::new(RichText::new("E: expand").color(Color32::WHITE).size(10.))
                            .selectable(false),
                    );
                    resp = ui.add(
                        Label::new(RichText::new("C: collapse").color(Color32::WHITE).size(10.))
                            .selectable(false),
                    );
                    resp = ui.add(
                        Label::new(RichText::new("A: add step").color(Color32::WHITE).size(10.))
                            .selectable(false),
                    );
                    resp = ui.add(
                        Label::new(
                            RichText::new("Shift + L Mouse: select multiple")
                                .color(Color32::WHITE)
                                .size(10.),
                        )
                        .selectable(false),
                    );
                    resp = ui.add(
                        Label::new(
                            RichText::new("Alt + L Mouse: deselect multiple")
                                .color(Color32::WHITE)
                                .size(10.),
                        )
                        .selectable(false),
                    );
                    resp = ui.add(
                        Label::new(
                            RichText::new("Ctrl + A: select all")
                                .color(Color32::WHITE)
                                .size(10.),
                        )
                        .selectable(false),
                    );
                    resp = ui.add(
                        Label::new(
                            RichText::new("Del: delete selected")
                                .color(Color32::WHITE)
                                .size(10.),
                        )
                        .selectable(false),
                    );
                    resp = ui.add(
                        Label::new(
                            RichText::new("Middle mouse / Space: drag")
                                .color(Color32::WHITE)
                                .size(10.),
                        )
                        .selectable(false),
                    );
                    resp = ui.add(
                        Label::new(
                            RichText::new("Y: draw to disconnect")
                                .color(Color32::WHITE)
                                .size(10.),
                        )
                        .selectable(false),
                    );

                    resp
                })
                .inner
            })
            .inner
    });
}

impl QuestStep {
    fn connected_to(&self) -> Vec<usize> {
        self.prev_steps.to_vec()
    }

    fn add_connection(&mut self, connected_to: usize) {
        if !self.prev_steps.contains(&connected_to) {
            self.prev_steps.push(connected_to)
        }
    }

    fn get_pos(&self) -> Pos2 {
        self.pos
    }

    fn add_to_pos(&mut self, pos: Vec2) {
        self.pos += pos;
    }

    fn get_size(&self) -> Vec2 {
        if self.is_finish_step() {
            Vec2::new(100., 100.)
        } else if self.collapsed {
            Vec2::new(200., 50.)
        } else {
            Vec2::new(EXPANDED_WIDTH, 300.)
        }
    }

    fn is_not_finish(&self) -> bool {
        !self.is_finish_step()
    }

    fn remove_all_input_connection(&mut self) {
        self.prev_steps.clear();
    }

    fn remove_input_connection(&mut self, index: usize) {
        if let Some((i, _)) = self
            .prev_steps
            .iter()
            .enumerate()
            .find(|(_, v)| **v == index)
        {
            self.prev_steps.remove(i);
        }
    }

    fn draw_tree_child(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<QuestAction>,
        idx: usize,
    ) {
        if self.is_finish_step() {
            ui.vertical_centered(|ui| {
                ui.set_width(100.);
                ui.set_height(100.);

                ui.add(
                    Label::new(
                        RichText::new("\u{f11e}")
                            .family(FontFamily::Name("icons".into()))
                            .size(60.),
                    )
                    .selectable(false),
                );
            });
        } else if self.collapsed {
            ui.scope(|ui| {
                ui.set_width(200.);
                ui.set_height(50.);

                ui.put(
                    ui.min_rect(),
                    Label::new(format!(
                        "{}\nStage: {}",
                        &self.title[holders.localization], self.stage
                    ))
                    .selectable(false),
                );
            });
        } else {
            // -------------------------------------------------------------------------------------
            //                                       Expanded
            // -------------------------------------------------------------------------------------

            ui.horizontal(|ui| {
                ui.set_width(EXPANDED_WIDTH);
                ui.set_height(300.);

                let min = ui.cursor().min;

                ui.add_space(5.0);
                ui.vertical(|ui| {
                    ui.add_space(5.0);

                    ui.vertical(|ui| {
                        text_row(ui, &mut self.title[holders.localization], "Title");
                        text_row(ui, &mut self.label[holders.localization], "Label");
                        num_row(ui, &mut self.stage, "Stage");

                        text_row_multiline(ui, &mut self.desc[holders.localization], "Description");
                    });
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_height(100.);

                            self.goals.draw_vertical(
                                ui,
                                &format!("Goals: {}", self.goals.len()),
                                |v| {
                                    *action.write().unwrap() = QuestAction::RemoveStepGoal {
                                        step_index: idx,
                                        goal_index: v,
                                    }
                                },
                                holders,
                                true,
                                false,
                            );
                        });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.label("Base Location");
                        self.location.draw(ui, holders);

                        self.additional_locations.draw_vertical(
                            ui,
                            &format!("Additional: {}", self.additional_locations.len()),
                            |v| {
                                *action.write().unwrap() =
                                    QuestAction::RemoveStepAdditionalLocation {
                                        step_index: idx,
                                        location_index: v,
                                    }
                            },
                            holders,
                            true,
                            false,
                        );
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.separator();

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Unknown 3");
                                ui.menu_button("+", |ui| {
                                    for v in UnkQLevel::iter() {
                                        if ui.button(format!("{v}")).clicked() {
                                            self.unk_q_level.push(v);
                                            ui.close_menu();
                                        }
                                    }
                                });
                            });

                            ui.push_id(ui.next_auto_id(), |ui| {
                                ScrollArea::vertical().show(ui, |ui| {
                                    for (i, v) in self.unk_q_level.clone().iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("{v}"));
                                            if ui.button(DELETE_ICON.to_string()).clicked() {
                                                self.unk_q_level.remove(i);
                                            }
                                        });
                                    }
                                });
                            });
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            combo_box_row(ui, &mut self.unk_1, "Unknown 1");
                            combo_box_row(ui, &mut self.unk_2, "Unknown 2");
                        });
                    });
                });
            });
        }
    }
}
