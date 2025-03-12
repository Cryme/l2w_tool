#![allow(unused)]

use crate::backend::editor::{CurrentEntity, WindowParams};
use crate::backend::holder::{DataHolder, DictItem};
use crate::backend::util::StringCow;
use crate::backend::Backend;
use crate::frontend::util::num_value::NumberValue;
use crate::frontend::{ADD_ICON, DELETE_ICON};
use eframe::egui::{
    Align2, Color32, Response, RichText, ScrollArea, TextWrapMode, Ui, Vec2, WidgetText,
};
use eframe::{egui, emath};
use std::fmt::Display;
use std::sync::RwLock;
use strum::IntoEnumIterator;

pub mod num_value;

impl<Inner: DrawActioned<Action, Params>, T1, Action, Params>
    WindowParams<Inner, T1, Action, Params>
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_as_button(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        holders: &DataHolder,
        button_title: impl Into<WidgetText>,
        window_title: &str,
        window_id_source: &str,
        rect: Vec2,
    ) {
        if ui.button(button_title).clicked() {
            self.opened = true;
        }

        if self.opened {
            egui::Window::new(window_title)
                .id(egui::Id::new(window_id_source))
                .open(&mut self.opened)
                .pivot(Align2::CENTER_CENTER)
                .default_pos([rect.x / 2.0, rect.y / 2.0])
                .show(ctx, |ui| {
                    self.inner
                        .draw_with_action(ui, holders, &self.action, &mut self.params);
                });
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_as_button_tooltip(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        holders: &DataHolder,
        button_title: impl Into<WidgetText>,
        window_title: &str,
        window_id_source: &str,
        tooltip: &str,
    ) {
        if ui.button(button_title).on_hover_text(tooltip).clicked() {
            self.opened = true;
        }

        if self.opened {
            egui::Window::new(window_title)
                .id(egui::Id::new(window_id_source))
                .open(&mut self.opened)
                .show(ctx, |ui| {
                    self.inner
                        .draw_with_action(ui, holders, &self.action, &mut self.params);
                });
        }
    }
}

pub trait DrawActioned<T, P> {
    fn draw_with_action(
        &mut self,
        ui: &mut Ui,
        holders: &DataHolder,
        action: &RwLock<T>,
        params: &mut P,
    );
}

pub trait Draw {
    fn draw(&mut self, ui: &mut Ui, holders: &DataHolder) -> Response;
}

pub trait DrawCtx {
    fn draw_ctx(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        holders: &mut DataHolder,
        init_rect: Vec2,
    ) -> Response;
}

pub trait DrawUtils {
    fn draw_vertical(
        &mut self,
        ui: &mut Ui,
        label: &str,
        action_callback: impl Fn(usize),
        holders: &DataHolder,
        with_scroll: bool,
        with_space: bool,
    );
    fn draw_vertical_nc(&mut self, ui: &mut Ui, label: &str, holders: &DataHolder);
    fn draw_horizontal(
        &mut self,
        ui: &mut Ui,
        label: &str,
        action_callback: impl Fn(usize),
        holders: &DataHolder,
        with_scroll: bool,
    );
}

impl<T: Draw + Default + Clone> DrawUtils for Vec<T> {
    fn draw_vertical(
        &mut self,
        ui: &mut Ui,
        label: &str,
        action_callback: impl Fn(usize),
        holders: &DataHolder,
        with_scroll: bool,
        accent: bool,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(format!("{label} [{}]", self.len())));

                if ui.button(ADD_ICON).clicked() {
                    self.push(T::default());
                }
            });

            if accent {
                ui.add_space(6.);
            }

            if with_scroll {
                ui.push_id(ui.next_auto_id(), |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        let len = self.len();
                        for (i, v) in self.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                if accent {
                                    ui.add_space(10.0);
                                }

                                if ui.button(DELETE_ICON).clicked() {
                                    action_callback(i);
                                }

                                v.draw(ui, holders);
                            });

                            if accent && i < len - 1 {
                                ui.separator();
                            }
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

    fn draw_vertical_nc(&mut self, ui: &mut Ui, label: &str, holders: &DataHolder) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(format!("{label} [{}]", self.len())));

                if ui.button(ADD_ICON).clicked() {
                    self.push(T::default());
                }
                if ui.button("-").clicked() {
                    self.pop();
                }
            });

            ui.add_space(6.);

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for (i, v) in self.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);

                            v.draw(ui, holders);
                            ui.add_space(6.0);
                        });

                        ui.separator();
                    }
                });
            });
        });
    }

    fn draw_horizontal(
        &mut self,
        ui: &mut Ui,
        label: &str,
        action_callback: impl Fn(usize),
        holders: &DataHolder,
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
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.add(egui::TextEdit::singleline(self))
    }
}

impl Draw for StringCow {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.add(egui::TextEdit::singleline(self.as_mut_string()))
    }
}

impl Draw for u32 {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.add(NumberValue::new(self))
    }
}

impl Draw for u16 {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.add(NumberValue::new(self))
    }
}

impl Draw for u8 {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.add(NumberValue::new(self))
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

pub fn text_row_c(ui: &mut Ui, val: &mut StringCow, label: &str) -> Response {
    ui.horizontal(|ui| {
        let mut r = ui.label(label);
        r = r.union(ui.text_edit_singleline(val.as_mut_string()));

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

pub fn num_row_optional<Num: emath::Numeric>(
    ui: &mut Ui,
    val: &mut Num,
    label: &str,
    val_label: &str,
    disable_value: Num,
) -> Response {
    ui.horizontal(|ui| {
        let disabled = *val == disable_value;
        let mut r = bool_row(ui, &mut !disabled, label);

        if r.changed() {
            *val = if disabled {
                Num::from_f64(1.0)
            } else {
                disable_value
            }
        }

        if !disabled {
            //TODO: Add Sets
            r = r.union(num_row(ui, val, val_label));
        }

        r
    })
    .inner
}

pub fn num_row<Num: emath::Numeric>(ui: &mut Ui, val: &mut Num, label: &str) -> Response {
    ui.horizontal(|ui| {
        let mut r = ui.label(label);
        r = r.union(ui.add(NumberValue::new(val)));

        r
    })
    .inner
}

pub fn num_row_2d<Num: emath::Numeric>(ui: &mut Ui, val: &mut [Num; 2], label: &str) -> Response {
    ui.horizontal(|ui| {
        let mut r = ui.label(label);

        r = r.union(ui.label("X"));
        r = r.union(ui.add(NumberValue::new(&mut val[0])));
        r = r.union(ui.label("Y"));
        r = r.union(ui.add(NumberValue::new(&mut val[1])));

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
        r = r.union(ui.add(NumberValue::new(val)));

        r
    })
    .inner
    .on_hover_text(tooltip);
}

pub fn combo_box_row<T: Display + PartialEq + Copy + IntoEnumIterator>(
    ui: &mut Ui,
    val: &mut T,
    label: &str,
) {
    ui.horizontal(|ui| {
        if !label.is_empty() {
            ui.add(egui::Label::new(label));
        }
        egui::ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(format!("{}", val))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                ui.set_min_width(20.0);

                for t in T::iter() {
                    ui.selectable_value(val, t, format!("{t}"));
                }
            });
    });
}

pub trait DrawAsTooltip {
    fn draw_as_tooltip(&self, ui: &mut Ui);
}

impl DrawAsTooltip for DictItem<u32, String> {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        self.item.draw_as_tooltip(ui)
    }
}

impl<T: DrawAsTooltip> DrawAsTooltip for Option<T> {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        if let Some(v) = self {
            v.draw_as_tooltip(ui);
        } else {
            ui.label("Not Exists");
        }
    }
}

impl<T: DrawAsTooltip> DrawAsTooltip for &T {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        (*self).draw_as_tooltip(ui);
    }
}

impl DrawAsTooltip for String {
    fn draw_as_tooltip(&self, ui: &mut Ui) {
        ui.label(self);
    }
}

pub fn format_button_text(mut val: &str) -> RichText {
    if val.chars().count() > 19 {
        RichText::new(format!("{}..", val.chars().take(18).collect::<String>()))
    } else {
        RichText::new(val)
    }
    .monospace()
    .strong()
    .color(Color32::LIGHT_GRAY)
    .size(11.)
}

impl Draw for i32 {
    fn draw(&mut self, ui: &mut Ui, _holders: &DataHolder) -> Response {
        ui.add(NumberValue::new(self))
    }
}

pub fn close_entity_button(
    ui: &mut Ui,
    entity: CurrentEntity,
    backend: &mut Backend,
    is_changed: bool,
) {
    let mut v = ui.button("❌");

    if is_changed {
        v = v.on_hover_text("Close\n(Ctrl+W)\nCtrl click to force close");
    } else {
        v = v.on_hover_text("Close\n(Ctrl+W)");
    }

    if v.clicked() && backend.no_dialog() {
        backend.close_entity(entity, ui.ctx().input(|i| i.modifiers.ctrl));
    }
}
