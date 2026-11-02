//! Obrazovka opakování: kartička, otočení a hodnocení.

use eframe::egui::{self, Button, Key, Modifiers, RichText, vec2};
use time::OffsetDateTime;
use vietnamska_slovicka::fsrs::Rating;
use vietnamska_slovicka::model::{Direction, Word};
use vietnamska_slovicka::session::{Session, interval_label};

use crate::app::{Action, Loaded, Screen};
use crate::fonts;
use crate::ui::{rating_color, rating_key};

/// Stav obrazovky opakování.
pub struct ReviewState {
    session: Session,
    /// Je kartička otočená na rub?
    flipped: bool,
    /// Intervaly pro tlačítka hodnocení aktuální kartičky (spočítané při otočení).
    preview: Option<[u32; 4]>,
}

impl ReviewState {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            flipped: false,
            preview: None,
        }
    }
}

/// Co uživatel na obrazovce udělal.
enum Outcome {
    None,
    Rate(Rating),
    Quit,
}

/// Klávesy stisknuté v tomto snímku.
struct Keys {
    flip: bool,
    rating: Option<Rating>,
    quit: bool,
}

/// Přečte klávesy a „spotřebuje“ je, aby mezerník nebo Enter nestiskly tlačítko,
/// které má zrovna fokus (jinak by šlo omylem ohodnotit kartičku dvakrát).
fn read_keys(ui: &mut egui::Ui) -> Keys {
    ui.input_mut(|input| {
        let space = input.consume_key(Modifiers::NONE, Key::Space);
        let enter = input.consume_key(Modifiers::NONE, Key::Enter);
        let rating = Rating::ALL
            .into_iter()
            .find(|&rating| input.consume_key(Modifiers::NONE, rating_key(rating)));
        Keys {
            flip: space || enter,
            rating,
            quit: input.consume_key(Modifiers::NONE, Key::Escape),
        }
    })
}

/// Vykreslí obrazovku opakování.
pub fn show(ui: &mut egui::Ui, loaded: &mut Loaded, now: OffsetDateTime) -> Action {
    let keys = read_keys(ui);
    let Screen::Review(state) = &mut loaded.screen else {
        return Action::None;
    };
    let Some(card) = state.session.current().cloned() else {
        // Prázdné sezení (nemělo by nastat, Začít je u prázdné fronty neaktivní).
        finish(loaded);
        return Action::None;
    };

    let mut outcome = Outcome::None;
    if keys.quit {
        outcome = Outcome::Quit;
    }
    if keys.flip && !state.flipped {
        state.flipped = true;
    } else if let Some(rating) = keys.rating
        && state.flipped
    {
        outcome = Outcome::Rate(rating);
    }
    if state.flipped && state.preview.is_none() {
        state.preview = Some(loaded.study.preview(&card, now));
    }

    egui::Panel::top("review_top").show(ui, |ui| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button("Ukončit (Esc)").clicked() {
                outcome = Outcome::Quit;
            }
            ui.separator();
            ui.label(format!(
                "Hotovo: {}   ·   zbývá: {}",
                state.session.done(),
                state.session.remaining()
            ));
            ui.separator();
            ui.label(RichText::new(card.direction.label()).family(fonts::bold()));
        });
        ui.add_space(4.0);
    });

    egui::Panel::bottom("review_buttons").show(ui, |ui| {
        ui.add_space(8.0);
        if !state.flipped {
            ui.vertical_centered(|ui| {
                let button = Button::new(RichText::new("Otočit (mezerník)").size(20.0))
                    .min_size(vec2(240.0, 48.0));
                if ui.add(button).clicked() {
                    state.flipped = true;
                }
            });
        } else if let Some(rating) = rating_buttons(ui, state.preview) {
            outcome = Outcome::Rate(rating);
        }
        ui.add_space(8.0);
    });

    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(24.0);
            ui.vertical_centered(|ui| match loaded.words.get(&card.key) {
                Some(word) if state.flipped => card_back(ui, word),
                Some(word) => card_front(ui, word, card.direction),
                None => {
                    ui.label(format!(
                        "Slovíčko „{}“ v lekcích chybí. Ukončete prosím sezení.",
                        card.key
                    ));
                }
            });
        });
    });

    match outcome {
        Outcome::None => {}
        Outcome::Quit => finish(loaded),
        Outcome::Rate(rating) => {
            if let Err(err) = loaded.study.rate(&card, rating, now) {
                loaded.report_error(format!("{err:#}"));
            }
            if let Screen::Review(state) = &mut loaded.screen {
                state.session.answer(rating);
                state.flipped = false;
                state.preview = None;
                if state.session.is_finished() {
                    finish(loaded);
                }
            }
        }
    }
    Action::None
}

/// Ukončí sezení a přejde na shrnutí nebo na úvod.
fn finish(loaded: &mut Loaded) {
    if let Screen::Review(state) = std::mem::replace(&mut loaded.screen, Screen::Home) {
        loaded.end_session(&state.session);
    }
}

/// Čtyři tlačítka hodnocení s intervaly. Vrátí zvolené hodnocení.
fn rating_buttons(ui: &mut egui::Ui, preview: Option<[u32; 4]>) -> Option<Rating> {
    let mut chosen = None;
    let spacing = ui.spacing().item_spacing.x;
    let width = ((ui.available_width() - 3.0 * spacing) / 4.0).max(60.0);
    ui.horizontal(|ui| {
        for (index, rating) in Rating::ALL.into_iter().enumerate() {
            let interval = preview
                .map(|days| interval_label(days[index]))
                .unwrap_or_default();
            let text = RichText::new(format!(
                "{} {}\n{interval}",
                rating.number(),
                rating.label()
            ))
            .size(17.0)
            .color(egui::Color32::WHITE);
            let button = Button::new(text)
                .fill(rating_color(rating))
                .min_size(vec2(width, 56.0));
            if ui.add(button).clicked() {
                chosen = Some(rating);
            }
        }
    });
    chosen
}

/// Líc kartičky: CS → VI česky a poznámka, VI → CS vietnamsky s výslovností.
fn card_front(ui: &mut egui::Ui, word: &Word, direction: Direction) {
    match direction {
        Direction::CsToVi => {
            ui.label(
                RichText::new(&word.cs)
                    .size(36.0)
                    .family(fonts::bold())
                    .color(ui.visuals().strong_text_color()),
            );
            note(ui, word);
        }
        Direction::ViToCs => {
            vietnamese(ui, word);
            pronunciation(ui, word);
        }
    }
}

/// Rub kartičky: vždy všechna pole.
fn card_back(ui: &mut egui::Ui, word: &Word) {
    vietnamese(ui, word);
    ui.add_space(4.0);
    ui.label(
        RichText::new(&word.cs)
            .size(28.0)
            .color(ui.visuals().strong_text_color()),
    );
    ui.add_space(12.0);
    pronunciation(ui, word);
    note(ui, word);
}

fn vietnamese(ui: &mut egui::Ui, word: &Word) {
    ui.label(
        RichText::new(&word.vi)
            .size(44.0)
            .family(fonts::bold())
            .color(ui.visuals().strong_text_color()),
    );
}

fn pronunciation(ui: &mut egui::Ui, word: &Word) {
    ui.add_space(8.0);
    ui.label(RichText::new(&word.pronunciation_cs).size(20.0));
    ui.label(RichText::new(&word.ipa).size(22.0).weak());
}

fn note(ui: &mut egui::Ui, word: &Word) {
    if let Some(note) = &word.note {
        ui.add_space(12.0);
        ui.label(RichText::new(note).size(17.0).italics().weak());
    }
}
