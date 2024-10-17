use crate::frontend::Frontend;
use eframe::egui;
use eframe::egui::{Context, ScrollArea, Ui};

pub struct ScriptRunner {
    pub(crate) script: String,
    pub(crate) output: String,
    pub opened: bool,
    pub execute_requested: bool,
}

impl ScriptRunner {
    pub fn new() -> Self {
        Self {
            script: "".to_string(),
            output: "".to_string(),
            opened: false,
            execute_requested: false,
        }
    }

    pub fn draw(&mut self, ui: &mut Ui, ctx: &Context) {
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(
            ui.ctx(),
            &ui.ctx().theme().default_style(),
        );

        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                &ui.ctx().theme().default_style(),
                &theme,
                string,
                "java",
            );

            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::Window::new("Script Runner")
            .id(egui::Id::new("script_runner"))
            .open(&mut self.opened)
            .show(ctx, |ui| {
                if ui.button("Execute").clicked() {
                    self.execute_requested = true;
                }

                ui.scope(|ui| {
                    ui.set_max_height(ctx.screen_rect().height() - 300.);
                    ScrollArea::vertical()
                        .id_salt(ui.next_auto_id())
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.script)
                                    .font(egui::TextStyle::Monospace) // for cursor height
                                    .code_editor()
                                    .desired_rows(10)
                                    .lock_focus(true)
                                    .desired_width(f32::INFINITY)
                                    .layouter(&mut layouter),
                            );
                        });
                });

                ui.scope(|ui| {
                    ui.set_min_height(30.0);
                    ui.set_max_height(200.0);
                    ui.separator();
                    ui.vertical_centered(|ui| {
                        ui.label("Output");
                    });
                    ScrollArea::vertical()
                        .id_salt(ui.next_auto_id())
                        .show(ui, |ui| {
                            ui.label(&self.output);
                            ui.separator();
                        });
                });
            });
    }
}

impl Frontend {
    pub fn draw_script_runner(&mut self, ui: &mut Ui, ctx: &Context) {
        if self.script_runner.opened {
            self.script_runner.draw(ui, ctx);
        }
    }
}
