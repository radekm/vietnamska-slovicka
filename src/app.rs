//! Stav aplikace: načtení dat, přepínání obrazovek a chybová lišta.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use eframe::egui::{self, Key, Layout, Modifiers, RichText};
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
    /// Postup a historie se v tomto stavu nikdy nezapisují, změnit jde jen složku
    /// se slovníčky v nastavení.
    Failed {
        /// Proč se data nenačetla.
        message: String,
        /// Chyba při změně složky se slovníčky, pokud nastala.
        folder_error: Option<String>,
    },
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
    /// Stav obrazovky Lekce (vybraná lekce a hledání), zachová se i po odchodu z ní.
    pub lessons_view: ui::lessons::LessonsView,
    /// Je otevřená legenda tónů?
    pub legend_open: bool,
    /// Fronta na dnešek spočítaná pro úvodní obrazovku a den, pro který platí.
    queue_cache: Option<(Date, Queue)>,
}

/// Obrazovky.
pub enum Screen {
    /// Úvod s přehledem a výběrem lekcí.
    Home,
    /// Procházení lekcí a hledání.
    Lessons,
    /// Statistiky.
    Stats(ui::stats::StatsState),
    /// Nastavení.
    Settings(ui::settings::SettingsState),
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
            State::Failed {
                message,
                folder_error: None,
            }
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
        let words = words_by_key(&lessons);
        let study = Study::load(dirs, settings.fsrs()?)?;
        Ok(Self {
            settings,
            lessons_dir,
            lessons,
            words,
            study,
            screen: Screen::Home,
            error: None,
            lessons_view: ui::lessons::LessonsView::default(),
            legend_open: false,
            queue_cache: None,
        })
    }

    /// Uloží nové nastavení a použije ho.
    ///
    /// Když se mění složka se slovníčky, lekce z ní se nejdřív zkusmo načtou. Pokud se to
    /// nepovede, nastavení se neuloží a nic se nezmění. Nová cílová retence platí jen pro
    /// další hodnocení.
    pub fn apply_settings(&mut self, settings: Settings, dirs: &AppDirs) -> Result<()> {
        let fsrs = settings.fsrs()?;
        let lessons_dir = settings
            .lessons_dir
            .clone()
            .unwrap_or_else(|| dirs.default_lessons.clone());
        let new_lessons = if lessons_dir == self.lessons_dir {
            None
        } else {
            let lessons = lessons::load_dir(&lessons_dir)
                .with_context(|| format!("složka se slovníčky {}", lessons_dir.display()))?;
            Some(lessons)
        };
        settings.save(&dirs.settings_file())?;
        tracing::info!(
            lessons_dir = %lessons_dir.display(),
            new_cards_per_day = settings.new_cards_per_day,
            desired_retention = settings.desired_retention,
            directions = ?settings.directions,
            "nastavení uloženo"
        );
        if let Some(lessons) = new_lessons {
            tracing::info!(
                lessons_dir = %lessons_dir.display(),
                lessons = lessons.len(),
                "lekce načteny z nové složky"
            );
            // Lekce se stejným číslem v nové složce nemusí existovat.
            self.lessons_view = ui::lessons::LessonsView::default();
            self.words = words_by_key(&lessons);
            self.lessons = lessons;
            self.lessons_dir = lessons_dir;
        }
        self.study.set_fsrs(fsrs);
        self.settings = settings;
        self.invalidate_queue();
        Ok(())
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
    pub fn save_settings(&mut self, settings_file: &Path) {
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
            State::Failed {
                message,
                folder_error,
            } => failed_ui(ui, message, folder_error, &self.dirs),
            State::Ready(loaded) => {
                if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::F1)) {
                    loaded.legend_open = !loaded.legend_open;
                }
                // Během opakování se záložky schovají, aby se sezení neopustilo omylem.
                // Tlačítko legendy má obrazovka opakování ve své liště.
                let today = study_day(now);
                if !matches!(loaded.screen, Screen::Review(_)) {
                    nav_bar(ui, loaded, today);
                }
                error_bar(ui, loaded);
                let settings_file = self.dirs.settings_file();
                let action = match &loaded.screen {
                    Screen::Home => ui::home::show(ui, loaded, today, &settings_file),
                    Screen::Lessons => {
                        ui::lessons::show(ui, loaded, today);
                        Action::None
                    }
                    Screen::Stats(_) => {
                        ui::stats::show(ui, loaded, today);
                        Action::None
                    }
                    Screen::Settings(_) => ui::settings::show(ui, loaded, &self.dirs),
                    Screen::Review(_) => ui::review::show(ui, loaded, now),
                    Screen::Finished(summary) => {
                        let summary = *summary;
                        ui::finished::show(ui, loaded, summary)
                    }
                };
                ui::legend::show(ui.ctx(), &mut loaded.legend_open);
                action
            }
        };
        if let Action::Reload = action {
            self.state = load_state(&self.dirs);
        }
        // Úvod ukazuje počty na dnešní studijní den, který se mění ve 4:00.
        ui.ctx().request_repaint_after(Duration::from_secs(60));
    }
}

/// Záložky obrazovek.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Home,
    Lessons,
    Stats,
    Settings,
}

/// Horní lišta se záložkami a tlačítkem legendy tónů.
fn nav_bar(ui: &mut egui::Ui, loaded: &mut Loaded, today: Date) {
    let active = match loaded.screen {
        Screen::Lessons => Some(Tab::Lessons),
        Screen::Stats(_) => Some(Tab::Stats),
        Screen::Settings(_) => Some(Tab::Settings),
        Screen::Home | Screen::Finished(_) => Some(Tab::Home),
        Screen::Review(_) => None,
    };
    let mut chosen = None;
    egui::Panel::top("nav_bar").show(ui, |ui| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            for (tab, title) in [
                (Tab::Home, "Úvod"),
                (Tab::Lessons, "Lekce"),
                (Tab::Stats, "Statistiky"),
                (Tab::Settings, "Nastavení"),
            ] {
                let text = RichText::new(title).size(17.0);
                if ui.selectable_label(active == Some(tab), text).clicked() {
                    chosen = Some(tab);
                }
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                legend_button(ui, &mut loaded.legend_open);
            });
        });
        ui.add_space(4.0);
    });
    match chosen {
        Some(tab) if Some(tab) != active => {
            // Odchodem z Nastavení se neuložené změny zahodí.
            loaded.screen = match tab {
                Tab::Home => Screen::Home,
                Tab::Lessons => Screen::Lessons,
                Tab::Stats => Screen::Stats(ui::stats::StatsState::new(loaded, today)),
                Tab::Settings => {
                    Screen::Settings(ui::settings::SettingsState::new(&loaded.settings))
                }
            };
        }
        _ => {}
    }
}

/// Tlačítko „?“, které přepíná legendu tónů.
pub fn legend_button(ui: &mut egui::Ui, open: &mut bool) {
    let button =
        egui::Button::new(RichText::new(" ? ").size(17.0).family(fonts::bold())).selected(*open);
    if ui.add(button).on_hover_text("Legenda tónů (F1)").clicked() {
        *open = !*open;
    }
}

/// Slovíčka podle klíče, pro zobrazení kartiček.
fn words_by_key(lessons: &[Lesson]) -> HashMap<String, Word> {
    lessons
        .iter()
        .flat_map(|lesson| &lesson.words)
        .map(|word| (word.key().to_owned(), word.clone()))
        .collect()
}

/// Změní složku se slovníčky přímo v souboru s nastavením (z chybové obrazovky).
///
/// Ostatní nastavení zůstane, jak bylo. Když je poškozený samotný soubor s nastavením,
/// nic se nezapíše.
fn change_lessons_dir(dirs: &AppDirs, lessons_dir: PathBuf) -> Result<()> {
    let settings_file = dirs.settings_file();
    let mut settings = Settings::load(&settings_file)
        .context("složku nelze změnit, protože se nepodařilo načíst nastavení")?;
    settings.lessons_dir = Some(lessons_dir);
    settings.save(&settings_file)?;
    tracing::info!(
        lessons_dir = ?settings.lessons_dir,
        "složka se slovníčky změněna z chybové obrazovky"
    );
    Ok(())
}

/// Obrazovka, když se data nepodařilo načíst.
fn failed_ui(
    ui: &mut egui::Ui,
    message: &str,
    folder_error: &mut Option<String>,
    dirs: &AppDirs,
) -> Action {
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
            .max_height(ui.available_height() - 90.0)
            .show(ui, |ui| {
                ui.label(RichText::new(message).size(15.0).monospace());
            });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("Zkusit znovu").size(18.0))
                .clicked()
            {
                action = Action::Reload;
            }
            if ui
                .button(RichText::new("Změnit složku se slovníčky…").size(18.0))
                .clicked()
                && let Some(path) = ui::settings::pick_folder(&dirs.default_lessons)
            {
                match change_lessons_dir(dirs, path) {
                    Ok(()) => action = Action::Reload,
                    Err(err) => *folder_error = Some(format!("{err:#}")),
                }
            }
        });
        if let Some(error) = folder_error {
            ui.label(RichText::new(error.as_str()).color(ui.visuals().error_fg_color));
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
