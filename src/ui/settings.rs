//! Obrazovka Nastavení: složka se slovníčky, denní limit, cílová retence a směry.
//!
//! Změny se upravují v konceptu a ukládají se až tlačítkem Uložit, protože změna
//! složky nebo retence znamená načíst nebo přepočítat data.

use std::path::PathBuf;

use eframe::egui::{self, Align, Button, Color32, DragValue, Label, RichText, Slider, vec2};
use vietnamska_slovicka::fsrs::{MAX_DESIRED_RETENTION, MIN_DESIRED_RETENTION};
use vietnamska_slovicka::model::Direction;
use vietnamska_slovicka::paths::AppDirs;
use vietnamska_slovicka::session::czech_plural;
use vietnamska_slovicka::settings::{MAX_NEW_CARDS_PER_DAY, Settings};

use crate::app::{Action, Loaded, Screen};
use crate::fonts;

/// Stav obrazovky Nastavení.
pub struct SettingsState {
    /// Rozpracované nastavení, které se ještě neuložilo.
    draft: Settings,
    /// Výsledek posledního ukládání.
    outcome: Option<SaveOutcome>,
    /// Posunout obrazovku k výsledku ukládání (jednou, hned po uložení).
    scroll_to_outcome: bool,
}

/// Výsledek ukládání, který se zobrazí pod tlačítky.
enum SaveOutcome {
    Saved,
    Failed(String),
}

impl SettingsState {
    /// Koncept podle aktuálního nastavení.
    pub fn new(settings: &Settings) -> Self {
        Self {
            draft: settings.clone(),
            outcome: None,
            scroll_to_outcome: false,
        }
    }
}

/// Co uživatel na obrazovce udělal.
enum Request {
    None,
    Save,
    Discard,
    ReloadLessons,
}

/// Vykreslí obrazovku Nastavení.
pub fn show(ui: &mut egui::Ui, loaded: &mut Loaded, dirs: &AppDirs) -> Action {
    let current = loaded.settings.clone();
    let lessons_info = format!(
        "Načteno {} {} ze složky {}",
        loaded.lessons.len(),
        czech_plural(
            u32::try_from(loaded.lessons.len()).unwrap_or(u32::MAX),
            "lekce",
            "lekce",
            "lekcí"
        ),
        loaded.lessons_dir.display()
    );
    let Screen::Settings(state) = &mut loaded.screen else {
        return Action::None;
    };
    // Výběr lekcí se mění na úvodu, koncept ho jen přebírá.
    state.draft.selected_lessons = current.selected_lessons.clone();
    state.draft.desired_retention = Settings::round_retention(state.draft.desired_retention);
    let changed = state.draft != current;
    let mut request = Request::None;

    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(RichText::new("Nastavení").size(28.0).family(fonts::bold()));
            ui.add_space(12.0);

            section(ui, "Složka se slovníčky");
            lessons_dir(ui, &mut state.draft, dirs, changed, &mut request);
            ui.label(RichText::new(lessons_info).weak().size(13.0));
            ui.add_space(16.0);

            section(ui, "Nových kartiček denně");
            ui.add(
                DragValue::new(&mut state.draft.new_cards_per_day)
                    .range(0..=MAX_NEW_CARDS_PER_DAY)
                    .speed(0.2),
            );
            hint(ui, "Kolik nových kartiček se nejvýš zavede za jeden den.");
            ui.add_space(16.0);

            section(ui, "Cílová retence");
            ui.add(
                Slider::new(
                    &mut state.draft.desired_retention,
                    MIN_DESIRED_RETENTION..=MAX_DESIRED_RETENTION,
                )
                .step_by(0.01)
                .custom_formatter(|value, _| format!("{value:.2}").replace('.', ","))
                .custom_parser(|text| text.trim().replace(',', ".").parse().ok()),
            );
            hint(
                ui,
                "Pravděpodobnost, že si kartičku při opakování vybavíte. Vyšší hodnota znamená \
                 častější opakování, výchozí je 0,90.\nZměna platí pro další hodnocení, \
                 už naplánovaná opakování se nemění.",
            );
            ui.add_space(16.0);

            section(ui, "Směry procvičování");
            directions(ui, &mut state.draft);
            hint(
                ui,
                "Aspoň jeden směr musí zůstat zapnutý. Při obou směrech se nové slovíčko \
                 zavede nejdřív jako VI → CS a CS → VI přijde na řadu až další den.",
            );
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                let save =
                    Button::new(RichText::new("Uložit").size(18.0)).min_size(vec2(120.0, 36.0));
                if ui.add_enabled(changed, save).clicked() {
                    request = Request::Save;
                }
                let discard = Button::new(RichText::new("Zahodit změny").size(18.0))
                    .min_size(vec2(120.0, 36.0));
                if ui.add_enabled(changed, discard).clicked() {
                    request = Request::Discard;
                }
                if changed {
                    ui.label(RichText::new("Máte neuložené změny.").weak());
                }
            });
            ui.add_space(8.0);
            let response = match &state.outcome {
                Some(SaveOutcome::Saved) if !changed => Some(ui.label(
                    RichText::new("Nastavení uloženo.").color(Color32::from_rgb(50, 140, 75)),
                )),
                Some(SaveOutcome::Failed(message)) => {
                    ui.label(
                        RichText::new("Nastavení se neuložilo:")
                            .family(fonts::bold())
                            .color(ui.visuals().error_fg_color),
                    );
                    Some(ui.add(Label::new(RichText::new(message).monospace()).wrap()))
                }
                _ => None,
            };
            // V malém okně by výsledek mohl být až pod okrajem.
            if let Some(response) = response
                && std::mem::take(&mut state.scroll_to_outcome)
            {
                response.scroll_to_me(Some(Align::BOTTOM));
            }
        });
    });

    match request {
        Request::None => Action::None,
        Request::Discard => {
            *state = SettingsState::new(&current);
            Action::None
        }
        Request::ReloadLessons => Action::Reload,
        Request::Save => {
            // Posuvník mohl hodnotu změnit až po zaokrouhlení na začátku snímku.
            let mut draft = state.draft.clone();
            draft.desired_retention = Settings::round_retention(draft.desired_retention);
            let outcome = match loaded.apply_settings(draft, dirs) {
                Ok(()) => SaveOutcome::Saved,
                Err(err) => {
                    let message = format!("{err:#}");
                    tracing::warn!(error = %message, "nastavení se neuložilo");
                    SaveOutcome::Failed(message)
                }
            };
            if let Screen::Settings(state) = &mut loaded.screen {
                state.outcome = Some(outcome);
                state.scroll_to_outcome = true;
            }
            Action::None
        }
    }
}

/// Nadpis oddílu.
fn section(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).size(18.0).family(fonts::bold()));
    ui.add_space(2.0);
}

/// Vysvětlující text pod prvkem.
fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).weak().size(13.0));
}

/// Výběr složky se slovníčky.
fn lessons_dir(
    ui: &mut egui::Ui,
    draft: &mut Settings,
    dirs: &AppDirs,
    changed: bool,
    request: &mut Request,
) {
    let path = match &draft.lessons_dir {
        Some(path) => path.display().to_string(),
        None => format!("{} (výchozí)", dirs.default_lessons.display()),
    };
    ui.label(RichText::new(path).monospace());
    ui.horizontal(|ui| {
        if ui.button("Vybrat…").clicked()
            && let Some(path) =
                pick_folder(draft.lessons_dir.as_ref().unwrap_or(&dirs.default_lessons))
        {
            draft.lessons_dir = Some(path);
        }
        if ui
            .add_enabled(draft.lessons_dir.is_some(), Button::new("Použít výchozí"))
            .on_hover_text("Složka slovnicky/ v pracovním adresáři")
            .clicked()
        {
            draft.lessons_dir = None;
        }
        if ui
            .add_enabled(!changed, Button::new("Načíst lekce znovu"))
            .on_hover_text("Po ruční úpravě souborů s lekcemi")
            .on_disabled_hover_text("Nejdřív uložte nebo zahoďte změny")
            .clicked()
        {
            *request = Request::ReloadLessons;
        }
    });
    hint(
        ui,
        "Před uložením nové složky se lekce z ní zkusmo načtou. Když jsou v nich chyby, \
         nic se neuloží.",
    );
}

/// Zaškrtávátka směrů. Poslední zapnutý směr nejde vypnout.
fn directions(ui: &mut egui::Ui, draft: &mut Settings) {
    for direction in Direction::ALL {
        let mut enabled = draft.direction_enabled(direction);
        let only_one = enabled && draft.directions.len() == 1;
        let checkbox = egui::Checkbox::new(&mut enabled, direction.label());
        if ui.add_enabled(!only_one, checkbox).changed() {
            if enabled {
                draft.directions.push(direction);
                draft.directions.sort();
            } else {
                draft.directions.retain(|&d| d != direction);
            }
        }
    }
}

/// Otevře dialog pro výběr složky. Vrátí `None`, když uživatel dialog zavřel.
pub fn pick_folder(start: &std::path::Path) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_title("Vyberte složku se slovníčky");
    if start.is_dir() {
        dialog = dialog.set_directory(start);
    }
    // Záložní dialog (zenity) umí vrátit prázdnou cestu, bereme ji jako zrušený výběr.
    let picked = dialog
        .pick_folder()
        .filter(|path| !path.as_os_str().is_empty());
    match &picked {
        Some(path) => tracing::info!(path = %path.display(), "vybrána složka se slovníčky"),
        None => tracing::info!("výběr složky zrušen"),
    }
    picked
}
