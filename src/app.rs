//! Stav aplikace: načtení dat, přepínání obrazovek a chybová lišta.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use eframe::egui::{self, RichText};
use time::{Date, OffsetDateTime, UtcOffset};
use vietnamska_slovicka::fsrs::study_day;
use vietnamska_slovicka::lessons;
use vietnamska_slovicka::model::{Lesson, Word};
use vietnamska_slovicka::paths::AppDirs;
use vietnamska_slovicka::scheduler::{self, Queue};
use vietnamska_slovicka::session::{Session, SessionSummary, Study};
use vietnamska_slovicka::settings::Settings;

use crate::fonts;
use crate::ui;

/// Hlavní struktura aplikace.
pub struct App {
    dirs: AppDirs,
    local_offset: UtcOffset,
    state: State,
}

/// Stav podle toho, jestli se data podařilo načíst.
enum State {
    /// Načtení selhalo, zobrazí se hláška a tlačítko „Zkusit znovu“.
    /// Data se v tomto stavu nikdy nezapisují.
    Failed(String),
    /// Data jsou načtená.
    Ready(Box<Loaded>),
}

/// Načtená data a obrazovka, která se právě zobrazuje.
pub struct Loaded {
    /// Nastavení.
    pub settings: Settings,
    /// Složka, ze které se lekce načetly.
    pub lessons_dir: PathBuf,
    /// Lekce seřazené podle čísla.
    pub lessons: Vec<Lesson>,
    /// Slovíčka podle klíče, pro zobrazení kartiček.
    pub words: HashMap<String, Word>,
    /// Postup, historie a plánovač.
    pub study: Study,
    /// Aktuální obrazovka.
    pub screen: Screen,
    /// Chyba, která se zobrazí v liště nahoře (např. nepovedené uložení).
    pub error: Option<String>,
    /// Fronta na dnešek spočítaná pro úvodní obrazovku a den, pro který platí.
    queue_cache: Option<(Date, Queue)>,
}

/// Obrazovky.
pub enum Screen {
    /// Úvod s přehledem a výběrem lekcí.
    Home,
    /// Probíhající sezení.
    Review(ui::review::ReviewState),
    /// Shrnutí dokončeného sezení.
    Finished(SessionSummary),
}

/// Co má aplikace udělat po vykreslení obrazovky.
pub enum Action {
    /// Nic.
    None,
    /// Načíst data znovu.
    Reload,
}

impl App {
    /// Vytvoří aplikaci, nastaví fonty a načte data.
    pub fn new(cc: &eframe::CreationContext<'_>, dirs: AppDirs, local_offset: UtcOffset) -> Self {
        fonts::install(&cc.egui_ctx);
        let state = load_state(&dirs);
        App {
            dirs,
            local_offset,
            state,
        }
    }
}

/// Načte data a převede výsledek na stav aplikace.
fn load_state(dirs: &AppDirs) -> State {
    match Loaded::load(dirs) {
        Ok(loaded) => {
            tracing::info!(
                lessons_dir = %loaded.lessons_dir.display(),
                lessons = loaded.lessons.len(),
                words = loaded.words.len(),
                cards_with_progress = loaded.study.progress.len(),
                reviews_in_history = loaded.study.history.entries().len(),
                "data načtena"
            );
            State::Ready(Box::new(loaded))
        }
        Err(err) => {
            let message = format!("{err:#}");
            tracing::error!(error = %message, "data se nepodařilo načíst");
            State::Failed(message)
        }
    }
}

impl Loaded {
    /// Načte nastavení, lekce, postup a historii.
    fn load(dirs: &AppDirs) -> Result<Self> {
        let settings = Settings::load(&dirs.settings_file())?;
        let lessons_dir = settings
            .lessons_dir
            .clone()
            .unwrap_or_else(|| dirs.default_lessons.clone());
        let lessons = lessons::load_dir(&lessons_dir)
            .with_context(|| format!("složka se slovníčky {}", lessons_dir.display()))?;
        let words = lessons
            .iter()
            .flat_map(|lesson| &lesson.words)
            .map(|word| (word.key().to_owned(), word.clone()))
            .collect();
        let study = Study::load(dirs, settings.fsrs()?)?;
        Ok(Self {
            settings,
            lessons_dir,
            lessons,
            words,
            study,
            screen: Screen::Home,
            error: None,
            queue_cache: None,
        })
    }

    /// Fronta na studijní den `today`. Počítá se znovu jen při změně dne nebo dat.
    pub fn queue(&mut self, today: Date) -> &Queue {
        if !matches!(&self.queue_cache, Some((day, _)) if *day == today) {
            self.queue_cache = None;
        }
        let (lessons, settings, progress) = (&self.lessons, &self.settings, &self.study.progress);
        &self
            .queue_cache
            .get_or_insert_with(|| {
                let queue = scheduler::build_queue(lessons, settings, progress, today);
                (today, queue)
            })
            .1
    }

    /// Zahodí spočítanou frontu (po změně výběru lekcí nebo po sezení).
    pub fn invalidate_queue(&mut self) {
        self.queue_cache = None;
    }

    /// Uloží nastavení. Chybu zobrazí v liště.
    pub fn save_settings(&mut self, settings_file: &std::path::Path) {
        if let Err(err) = self.settings.save(settings_file) {
            self.report_error(format!("{err:#}"));
        }
    }

    /// Zobrazí chybu v liště a zaloguje ji.
    pub fn report_error(&mut self, message: String) {
        tracing::error!(error = %message, "chyba");
        self.error = Some(message);
    }

    /// Začne sezení z dnešní fronty.
    pub fn start_session(&mut self, today: Date) {
        let queue = self.queue(today).clone();
        tracing::info!(
            due = queue.due.len(),
            new = queue.new.len(),
            "sezení zahájeno"
        );
        self.screen = Screen::Review(ui::review::ReviewState::new(Session::new(queue)));
    }

    /// Ukončí sezení (dokončené i předčasně), zaloguje souhrn a přejde na další obrazovku.
    pub fn end_session(&mut self, session: &Session) {
        let summary = session.summary();
        tracing::info!(
            reviews = summary.total(),
            again = summary.by_rating[0],
            hard = summary.by_rating[1],
            good = summary.by_rating[2],
            easy = summary.by_rating[3],
            remaining = session.remaining(),
            "sezení ukončeno"
        );
        self.invalidate_queue();
        self.screen = if session.is_finished() {
            Screen::Finished(summary)
        } else {
            Screen::Home
        };
    }
}

impl App {
    /// Aktuální místní čas.
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc().to_offset(self.local_offset)
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = self.now();
        let action = match &mut self.state {
            State::Failed(message) => failed_ui(ui, message),
            State::Ready(loaded) => {
                error_bar(ui, loaded);
                let settings_file = self.dirs.settings_file();
                match &loaded.screen {
                    Screen::Home => ui::home::show(ui, loaded, study_day(now), &settings_file),
                    Screen::Review(_) => ui::review::show(ui, loaded, now),
                    Screen::Finished(summary) => {
                        let summary = *summary;
                        ui::finished::show(ui, loaded, summary)
                    }
                }
            }
        };
        if let Action::Reload = action {
            self.state = load_state(&self.dirs);
        }
        // Úvod ukazuje počty na dnešní studijní den, který se mění ve 4:00.
        ui.ctx().request_repaint_after(Duration::from_secs(60));
    }
}

/// Obrazovka, když se data nepodařilo načíst.
fn failed_ui(ui: &mut egui::Ui, message: &str) -> Action {
    let mut action = Action::None;
    egui::CentralPanel::default().show(ui, |ui| {
        ui.label(
            RichText::new("Data se nepodařilo načíst")
                .size(28.0)
                .family(fonts::bold()),
        );
        ui.add_space(8.0);
        ui.label("Opravte prosím problémy níže a zkuste to znovu. Postup učení se nezměnil.");
        ui.add_space(12.0);
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 60.0)
            .show(ui, |ui| {
                ui.label(RichText::new(message).size(15.0).monospace());
            });
        ui.add_space(12.0);
        if ui
            .button(RichText::new("Zkusit znovu").size(18.0))
            .clicked()
        {
            action = Action::Reload;
        }
    });
    action
}

/// Lišta s chybou nahoře, pokud nějaká je.
fn error_bar(ui: &mut egui::Ui, loaded: &mut Loaded) {
    let Some(message) = loaded.error.clone() else {
        return;
    };
    egui::Panel::top("error_bar").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&message).color(ui.visuals().error_fg_color));
            if ui.button("Zkusit uložit znovu").clicked() {
                match loaded.study.save() {
                    Ok(()) => {
                        tracing::info!("uložení se po chybě podařilo");
                        loaded.error = None;
                    }
                    Err(err) => loaded.report_error(format!("{err:#}")),
                }
            }
            if ui.button("Zavřít").clicked() {
                loaded.error = None;
            }
        });
    });
}
