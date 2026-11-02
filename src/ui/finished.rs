//! Shrnutí dokončeného sezení.

use eframe::egui::{self, Button, Key, Modifiers, RichText, vec2};
use vietnamska_slovicka::fsrs::Rating;
use vietnamska_slovicka::session::SessionSummary;

use crate::app::{Action, Loaded, Screen};
use crate::fonts;
use crate::ui::rating_color;

/// Vykreslí shrnutí sezení.
pub fn show(ui: &mut egui::Ui, loaded: &mut Loaded, summary: SessionSummary) -> Action {
    let back_key = ui.input_mut(|input| {
        input.consume_key(Modifiers::NONE, Key::Enter)
            || input.consume_key(Modifiers::NONE, Key::Space)
            || input.consume_key(Modifiers::NONE, Key::Escape)
    });
    let mut back = back_key;

    egui::CentralPanel::default().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.label(
                RichText::new("Sezení dokončeno")
                    .size(32.0)
                    .family(fonts::bold()),
            );
            ui.add_space(12.0);
            ui.label(RichText::new(format!("Hodnocení celkem: {}", summary.total())).size(20.0));
            ui.add_space(8.0);
            for rating in Rating::ALL {
                ui.label(
                    RichText::new(format!("{}: {}", rating.label(), summary.count(rating)))
                        .size(18.0)
                        .color(rating_color(rating)),
                );
            }
            ui.add_space(24.0);
            let button =
                Button::new(RichText::new("Zpět na úvod").size(20.0)).min_size(vec2(200.0, 44.0));
            if ui.add(button).clicked() {
                back = true;
            }
        });
    });

    if back {
        loaded.screen = Screen::Home;
    }
    Action::None
}
