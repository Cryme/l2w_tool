mod quest;
mod skill;

use crate::backend::{Backend, CurrentOpenedEntity, Dialog, DialogAnswer};
use crate::data::Location;
use crate::entity::hunting_zone::HuntingZone;
use crate::entity::item::Item;
use crate::entity::npc::Npc;
use crate::entity::Entity;
use eframe::egui;
use eframe::egui::{Button, Color32, ScrollArea, Ui};
use strum::IntoEnumIterator;
use egui::special_emojis::GITHUB;

struct GlobalSearchParams {
    pub search_showing: bool,
    pub current_entity: Entity,
}

pub struct Frontend {
    backend: Backend,
    search_params: GlobalSearchParams,
}

pub trait BuildAsTooltip {
    fn build_as_tooltip(&self, ui: &mut Ui);
}

impl<T: BuildAsTooltip> BuildAsTooltip for Option<T> {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        if let Some(v) = self {
            v.build_as_tooltip(ui);
        } else {
            ui.label("Not Exists");
        }
    }
}

impl<T: BuildAsTooltip> BuildAsTooltip for &T {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        (*self).build_as_tooltip(ui);
    }
}

impl BuildAsTooltip for Item {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.label(format!("{} [{}]", self.name, self.id.0));

        if !self.desc.is_empty() {
            ui.label(self.desc.to_string());
        };
    }
}

impl BuildAsTooltip for Npc {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            if !self.title.is_empty() {
                ui.colored_label(self.title_color, self.title.to_string());
            };

            ui.label(format!("{} [{}]", self.name, self.id.0));
        });
    }
}

impl BuildAsTooltip for HuntingZone {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(format!("[{}]\n {}", self.id.0, self.name));

            if !self.desc.is_empty() {
                ui.label(self.desc.to_string());
            }
        });
    }
}

impl BuildAsTooltip for String {
    fn build_as_tooltip(&self, ui: &mut Ui) {
        ui.label(self);
    }
}

impl Location {
    fn build(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add(egui::DragValue::new(&mut self.x));

            ui.label("Y");
            ui.add(egui::DragValue::new(&mut self.y));

            ui.label("Z");
            ui.add(egui::DragValue::new(&mut self.z));
        });
    }
}

impl Frontend {
    fn build_editor(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        match self.backend.edit_params.current_opened_entity {
            CurrentOpenedEntity::Quest(index) => {
                let quest = &mut self.backend.edit_params.quests.opened[index];

                quest
                    .inner
                    .build(ui, ctx, &mut quest.action, &mut self.backend.holders);
            }
            CurrentOpenedEntity::Skill(index) => {
                let e = &mut self.backend.edit_params.skills.opened[index];

                e.inner.build(
                    ui,
                    ctx,
                    &mut e.action,
                    &mut self.backend.holders,
                    &mut e.params,
                );
            }
            CurrentOpenedEntity::None => {}
        }
    }

    fn build_top_menu(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.menu_button(" ⚙ ", |ui| {
                if ui.button("Set L2 System Folder").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.backend.update_system_path(path)
                    }
                }
                if ui.button("Set Quest Java Classes Folder").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.backend.update_quests_java_path(path)
                    }
                }
                if ui.button("Save to .dat").clicked() {
                    self.backend.save_to_dat();
                    ui.close_menu();
                }
            })
            .response
            .on_hover_text("Settings");

            ui.add_space(5.);

            ui.menu_button("ℹ", |ui| {
                ui.set_width(10.);
                ui.hyperlink_to(
                    format!("{GITHUB}"),
                    "https://github.com/La2world-ru/l2_quest_editor",
                );
                ui.hyperlink_to(
                    format!("✉"),
                    "https://t.me/CrymeAriven",
                );
                ui.hyperlink_to(
                    format!("🎮"),
                    "https://la2world.ru",
                );
            });

            ui.add_space(5.);

            if ui
                .button(" 📚 ")
                .on_hover_text("Search/Edit/Create")
                .clicked()
            {
                self.search_params.search_showing = true;
            }

            egui::Window::new("📚")
                .id(egui::Id::new("_search_"))
                .open(&mut self.search_params.search_showing)
                .show(ctx, |ui| {
                    ui.set_width(150.);

                    egui::ComboBox::from_id_source(ui.next_auto_id())
                        .selected_text(format!("{}", self.search_params.current_entity))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(20.0);

                            for t in Entity::iter() {
                                ui.selectable_value(
                                    &mut self.search_params.current_entity,
                                    t,
                                    format!("{t}"),
                                );
                            }
                        });

                    ui.separator();

                    match self.search_params.current_entity {
                        Entity::Quest => {
                            Self::build_quest_selector(
                                &mut self.backend,
                                ui,
                                ctx.screen_rect().height() - 130.,
                            );
                        }
                        Entity::Skill => Self::build_skill_selector(
                            &mut self.backend,
                            ui,
                            ctx.screen_rect().height() - 130.,
                        ),
                    }
                });

            ui.separator();

            ui.push_id(ui.next_auto_id(), |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    for (i, (title, id)) in self
                        .backend
                        .edit_params
                        .get_opened_quests_info()
                        .iter()
                        .enumerate()
                    {
                        if i > 0 {
                            ui.separator();
                        }
                        let mut button = Button::new(format!("{} [{}]", title, id.0));

                        if CurrentOpenedEntity::Quest(i)
                            == self.backend.edit_params.current_opened_entity
                        {
                            button = button.fill(Color32::from_rgb(42, 70, 83));
                        }

                        if ui.add(button).clicked() && !self.backend.dialog_showing {
                            self.backend.edit_params.set_current_quest(i);
                        }
                        if ui.button("❌").clicked() {
                            self.backend.edit_params.close_quest(i);
                        }
                    }

                    for (i, (title, id)) in self
                        .backend
                        .edit_params
                        .get_opened_skills_info()
                        .iter()
                        .enumerate()
                    {
                        if i > 0 {
                            ui.separator();
                        }
                        let mut button = Button::new(format!("{} [{}]", title, id.0));

                        if CurrentOpenedEntity::Skill(i)
                            == self.backend.edit_params.current_opened_entity
                        {
                            button = button.fill(Color32::from_rgb(42, 70, 83));
                        }

                        if ui.add(button).clicked() && !self.backend.dialog_showing {
                            self.backend.edit_params.set_current_skill(i);
                        }

                        if ui.button("❌").clicked() {
                            self.backend.edit_params.close_skill(i);
                        }
                    }
                });
            });

            if self.backend.edit_params.current_opened_entity.is_some() {
                ui.separator();

                if ui.button("Save").clicked() {
                    self.backend.save_current_entity();
                }
            }
        });
    }

    pub fn init(ctx: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&ctx.egui_ctx);

        Self {
            backend: Backend::init(),
            search_params: GlobalSearchParams {
                search_showing: false,
                current_entity: Entity::Quest,
            },
        }
    }

    fn show_dialog(&mut self, ctx: &egui::Context) {
        match &self.backend.dialog {
            Dialog::ConfirmQuestSave { message, .. } | Dialog::ConfirmSkillSave { message, .. } => {
                let m = message.clone();

                egui::Window::new("Confirm")
                    .id(egui::Id::new("_confirm_"))
                    .movable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(m);

                            ui.horizontal_centered(|ui| {
                                if ui.button("Confirm").clicked() {
                                    self.backend.answer(DialogAnswer::Confirm);
                                }
                                if ui.button("Abort").clicked() {
                                    self.backend.answer(DialogAnswer::Abort);
                                }
                            });
                        })
                    });
            }
            Dialog::ShowWarning(warn) => {
                let m = warn.clone();

                egui::Window::new("Warning!")
                    .id(egui::Id::new("_warn_"))
                    .resizable(false)
                    .movable(false)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(m);

                            if ui.button("   Ok   ").clicked() {
                                self.backend.answer(DialogAnswer::Confirm);
                            }
                        })
                    });
            }

            Dialog::None => {}
        }
    }
}

impl eframe::App for Frontend {
    fn on_close_event(&mut self) -> bool {
        self.backend.auto_save(true);

        true
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.backend.on_update();

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_dialog(ctx);

            self.build_top_menu(ui, ctx);

            ui.separator();

            self.build_editor(ui, ctx);
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../../Nunito-Black.ttf")),
    );

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
