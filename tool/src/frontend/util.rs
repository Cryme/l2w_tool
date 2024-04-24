#![allow(unused)]

use crate::backend::{Holders, WindowParams};
use crate::frontend::{ADD_ICON, DELETE_ICON};
use eframe::egui::{Response, ScrollArea, Ui};
use eframe::{egui, emath};
use std::fmt::Display;
use std::sync::RwLock;

impl<Inner: Build<Action>, T1, Action, T3> WindowParams<Inner, T1, Action, T3> {
    pub(crate) fn build(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        holders: &Holders,
        button_title: &str,
        window_title: &str,
        window_id_source: &str,
    ) {
        if ui.button(button_title).clicked() {
            self.opened = true;
        }

        if self.opened {
            egui::Window::new(window_title)
                .id(egui::Id::new(window_id_source))
                .open(&mut self.opened)
                .show(ctx, |ui| {
                    self.inner.build(ui, holders, &mut self.action);
                });
        }
    }
}

pub trait Build<T> {
    fn build(&mut self, ui: &mut Ui, holders: &Holders, action: &RwLock<T>);
}

pub trait Draw {
    fn draw(&mut self, ui: &mut Ui, holders: &Holders) -> Response;
}

pub trait DrawUtils {
    fn draw_vertical(
        &mut self,
        ui: &mut Ui,
        label: &str,
        action_callback: impl Fn(usize),
        holders: &Holders,
        with_scroll: bool,
    );
    fn draw_horizontal(
        &mut self,
        ui: &mut Ui,
        label: &str,
        action_callback: impl Fn(usize),
        holders: &Holders,
        with_scroll: bool,
    );
}

impl<T: Draw + Default> DrawUtils for Vec<T> {
    fn draw_vertical(
        &mut self,
        ui: &mut Ui,
        label: &str,
        action_callback: impl Fn(usize),
        holders: &Holders,
        with_scroll: bool,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(label));

                if ui.button(ADD_ICON).clicked() {
                    self.push(T::default());
                }
            });

            if with_scroll {
                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for (i, v) in self.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                v.draw(ui, holders);
                                if ui.button(DELETE_ICON).clicked() {
                                    action_callback(i);
                                }
                                ui.add_space(6.0);
                            });
                        }
                    });
                });
            } else {
                for (i, v) in self.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        v.draw(ui, holders);
                        if ui.button(DELETE_ICON).clicked() {
                            action_callback(i);
                        }
                    });
                }
            }
        });
    }

    fn draw_horizontal(
        &mut self,
        ui: &mut Ui,
        label: &str,
        action_callback: impl Fn(usize),
        holders: &Holders,
        with_scroll: bool,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(label));

                if ui.button(ADD_ICON).clicked() {
                    self.push(T::default());
                }
            });

            if with_scroll {
                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (i, v) in self.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    v.draw(ui, holders);
                                    if ui.button(DELETE_ICON).clicked() {
                                        action_callback(i);
                                    }
                                });

                                ui.separator();
                            }
                        });
                        ui.add_space(6.0);
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    for (i, v) in self.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            v.draw(ui, holders);
                            if ui.button(DELETE_ICON).clicked() {
                                action_callback(i);
                            }
                        });

                        ui.separator();
                    }
                });
            }
        });
    }
}

impl Draw for String {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        ui.add(egui::TextEdit::singleline(self))
    }
}

impl Draw for u32 {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        ui.add(egui::DragValue::new(self))
    }
}

impl Draw for u16 {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        ui.add(egui::DragValue::new(self))
    }
}

impl Draw for u8 {
    fn draw(&mut self, ui: &mut Ui, _holders: &Holders) -> Response {
        ui.add(egui::DragValue::new(self))
    }
}

pub fn text_row_multiline(ui: &mut Ui, val: &mut String, label: &str) -> Response {
    ui.vertical(|ui| {
        let mut r = ui.label(label);
        r = r.union(ui.text_edit_multiline(val));

        r
    })
    .inner
}

pub fn text_row(ui: &mut Ui, val: &mut String, label: &str) -> Response {
    ui.horizontal(|ui| {
        let mut r = ui.label(label);
        r = r.union(ui.text_edit_singleline(val));

        r
    })
    .inner
}

pub fn text_tooltip_row(ui: &mut Ui, val: &mut String, label: &str, tooltip: &str) {
    ui.horizontal(|ui| {
        let mut r = ui.label(label);
        r = r.union(ui.text_edit_singleline(val));

        r
    })
    .inner
    .on_hover_text(tooltip);
}

pub fn bool_row(ui: &mut Ui, val: &mut bool, label: &str) -> Response {
    ui.horizontal(|ui| {
        let mut r = ui.label(label);
        r = r.union(ui.add(egui::Checkbox::new(val, "")));

        r
    })
    .inner
}

pub fn bool_tooltip_row(ui: &mut Ui, val: &mut bool, label: &str, tooltip: &str) {
    ui.horizontal(|ui| {
        let mut r = ui.label(label);
        r = r.union(ui.add(egui::Checkbox::new(val, "")));

        r
    })
    .inner
    .on_hover_text(tooltip);
}

pub fn num_row<Num: emath::Numeric>(ui: &mut Ui, val: &mut Num, label: &str) -> Response {
    ui.horizontal(|ui| {
        let mut r = ui.label(label);
        r = r.union(ui.add(egui::DragValue::new(val)));

        r
    })
    .inner
}
pub fn num_tooltip_row<Num: emath::Numeric>(
    ui: &mut Ui,
    val: &mut Num,
    label: &str,
    tooltip: &str,
) {
    ui.horizontal(|ui| {
        let mut r = ui.label(label);
        r = r.union(ui.add(egui::DragValue::new(val)));

        r
    })
    .inner
    .on_hover_text(tooltip);
}

pub fn combo_box_row<T: Display + PartialEq + Copy, I: Iterator<Item = T>>(
    ui: &mut Ui,
    val: &mut T,
    iter: I,
    label: &str,
) {
    ui.horizontal(|ui| {
        if !label.is_empty() {
            ui.add(egui::Label::new(label));
        }
        egui::ComboBox::from_id_source(ui.next_auto_id())
            .selected_text(format!("{}", val))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(20.0);

                for t in iter {
                    ui.selectable_value(val, t, format!("{t}"));
                }
            });
    });
}

pub trait DrawAsTooltip {
    fn build_as_tooltip(&self, ui: &mut Ui);
}

impl<T: DrawAsTooltip> DrawAsTooltip for Option<T> {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        if let Some(v) = self {
            v.build_as_tooltip(ui);
        } else {
            ui.label("Not Exists");
        }
    }
}

impl<T: DrawAsTooltip> DrawAsTooltip for &T {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        (*self).build_as_tooltip(ui);
    }
}

impl DrawAsTooltip for String {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.label(self);
    }
}
