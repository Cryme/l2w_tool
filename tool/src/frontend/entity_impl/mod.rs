use crate::backend::entity_catalog::EntityInfo;
use eframe::egui::{Button, Color32, FontFamily, Response, RichText, Ui};

pub mod hunting_zone;
pub mod item;
pub mod item_set;
pub mod npc;
pub mod quest;
pub mod recipe;
pub mod region;
pub mod skill;

impl<T1, ID: Copy> EntityInfo<T1, ID> {
    fn draw_select_button(&self, ui: &mut Ui, id: &mut Option<ID>) -> Response {
        if if self.deleted {
            ui.add(
                Button::new(RichText::new("\u{f82a}").family(FontFamily::Name("icons".into())))
                    .min_size([5., 36.].into()),
            )
            .on_hover_text("Restore")
        } else {
            ui.add(
                Button::new(
                    RichText::new("\u{f1f8}")
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
            ui.button(RichText::new(&self.label).color(Color32::from_rgb(242, 192, 124)))
                .on_hover_text("Changed")
        } else {
            ui.button(&self.label)
        }
    }
}
