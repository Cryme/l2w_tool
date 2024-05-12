use crate::backend::entity_catalog::EntityInfo;
use eframe::egui::{Button, Color32, FontFamily, Response, RichText, Stroke, Ui};

pub mod hunting_zone;
pub mod item;
pub mod item_set;
pub mod npc;
pub mod quest;
pub mod raid_info;
pub mod recipe;
pub mod region;
pub mod skill;

enum EntityInfoState {
    Nothing,
    Opened,
    Current,
}

impl EntityInfoState {
    fn add_stroke_to_button(self, button: Button) -> Button {
        match self {
            EntityInfoState::Nothing => button,
            EntityInfoState::Opened => button.fill(Color32::from_rgb(37, 54, 71)),
            EntityInfoState::Current => button
                .fill(Color32::from_rgb(37, 54, 71))
                .stroke(Stroke::new(2.0, Color32::from_rgb(178, 178, 178))),
        }
    }
}

impl<T1, ID: Copy> EntityInfo<T1, ID> {
    fn draw_catalog_buttons(
        &self,
        ui: &mut Ui,
        id: &mut Option<ID>,
        info_state: EntityInfoState,
    ) -> Response {
        if if self.deleted {
            ui.add(
                Button::new(RichText::new("\u{f82a}").family(FontFamily::Name("icons".into())))
                    .min_size([5., 36.].into()),
            )
            .on_hover_text("Restore")
        } else {
            ui.add(
                Button::new(
                    RichText::new("\u{f2ed}")
                        .family(FontFamily::Name("icons".into()))
                        .color(Color32::from_rgb(221, 65, 65)),
                )
                .min_size([5., 36.].into()),
            )
            .on_hover_text("Delete")
        }
        .clicked()
        {
            *id = Some(self.id);
        }

        if self.deleted {
            ui.button(
                RichText::new(&self.label)
                    .color(Color32::from_rgb(221, 65, 65))
                    .strikethrough(),
            )
            .on_hover_text("DELETED")
        } else if self.changed {
            let button = info_state.add_stroke_to_button(Button::new(
                RichText::new(&self.label).color(Color32::from_rgb(242, 192, 124)),
            ));

            ui.add(button).on_hover_text("Changed")
        } else {
            ui.add(info_state.add_stroke_to_button(Button::new(&self.label)))
        }
    }
}
