//! Úvodní obrazovka: přehled na dnešek, výběr lekcí a začátek sezení.

use std::path::Path;

use eframe::egui::{self, Button, Key, Modifiers, RichText, vec2};
use time::Date;

use vietnamska_slovicka::session::czech_plural;

use crate::app::{Action, Loaded};
use crate::fonts;

/// Údaje o lekci pro seznam.
struct LessonRow {
    number: u32,
    title: String,
    description: Option<String>,
    words: usize,
    new_cards: usize,
    selected: bool,
}

/// Vykreslí úvod.
pub fn show(ui: &mut egui::Ui, loaded: &mut Loaded, today: Date, settings_file: &Path) -> Action {
    let (due, new) = {
        let queue = loaded.queue(today);
        (queue.due.len(), queue.new.len())
    };
    let mut rows = lesson_rows(loaded);
    let mut start = false;
    let enter = ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter));

    egui::Panel::bottom("home_footer").show(ui, |ui| {
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("Slovníčky: {}", loaded.lessons_dir.display()))
                .weak()
                .size(13.0),
        );
        ui.add_space(4.0);
    });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.label(
            RichText::new("Vietnamská slovíčka")
                .size(30.0)
                .family(fonts::bold()),
        );
        ui.add_space(12.0);
        ui.label(RichText::new(format!("Dnes k opakování: {due}   ·   nových: {new}")).size(20.0));
        ui.add_space(12.0);

        let can_start = due + new > 0;
        let button = Button::new(RichText::new("Začít").size(22.0)).min_size(vec2(160.0, 44.0));
        if ui.add_enabled(can_start, button).clicked() || (can_start && enter) {
            start = true;
        }
        // Řádek s nápovědou je tu vždy, aby seznam lekcí při změně výběru neposkakoval.
        let hint = if can_start {
            "Sezení spustí i klávesa Enter."
        } else if loaded.settings.selected_lessons.is_empty() {
            "Pro nové kartičky vyberte níže aspoň jednu lekci."
        } else {
            "Na dnešek je hotovo."
        };
        ui.label(RichText::new(hint).weak());

        ui.add_space(16.0);
        ui.separator();
        ui.label(RichText::new("Lekce").size(20.0).family(fonts::bold()));
        ui.label(RichText::new("Nové kartičky se berou jen z vybraných lekcí.").weak());
        ui.add_space(6.0);

        if rows.is_empty() {
            ui.label(format!(
                "Ve složce {} nejsou žádné lekce (soubory lekce01.toml, lekce02.toml, …).",
                loaded.lessons_dir.display()
            ));
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for row in &mut rows {
                ui.horizontal(|ui| {
                    let title = format!("{}. {}", row.number, row.title);
                    ui.checkbox(&mut row.selected, RichText::new(title).size(17.0));
                    ui.label(
                        RichText::new(format!(
                            "{} {} · nových kartiček: {}",
                            row.words,
                            czech_plural(
                                u32::try_from(row.words).unwrap_or(u32::MAX),
                                "slovíčko",
                                "slovíčka",
                                "slovíček"
                            ),
                            row.new_cards
                        ))
                        .weak(),
                    );
                });
                if let Some(description) = &row.description {
                    ui.indent(("lesson_description", row.number), |ui| {
                        ui.label(RichText::new(description).weak().size(13.0));
                    });
                }
                ui.add_space(4.0);
            }
        });
    });

    // Změna výběru lekcí se hned uloží.
    let selected: Vec<u32> = rows
        .iter()
        .filter(|row| row.selected)
        .map(|row| row.number)
        .collect();
    if selected != loaded.settings.selected_lessons {
        tracing::info!(selected_lessons = ?selected, "změna výběru lekcí");
        loaded.settings.selected_lessons = selected;
        loaded.save_settings(settings_file);
        loaded.invalidate_queue();
    }

    if start {
        loaded.start_session(today);
    }
    Action::None
}

/// Údaje o lekcích: počet slovíček a kolik zbývá nových kartiček v zapnutých směrech.
fn lesson_rows(loaded: &Loaded) -> Vec<LessonRow> {
    loaded
        .lessons
        .iter()
        .map(|lesson| {
            let new_cards = lesson
                .words
                .iter()
                .flat_map(|word| word.card_ids())
                .filter(|id| {
                    loaded.settings.direction_enabled(id.direction)
                        && loaded.study.progress.get(id).is_none()
                })
                .count();
            LessonRow {
                number: lesson.number,
                title: lesson.title.clone(),
                description: lesson.description.clone(),
                words: lesson.words.len(),
                new_cards,
                selected: loaded.settings.lesson_selected(lesson.number),
            }
        })
        .collect()
}
