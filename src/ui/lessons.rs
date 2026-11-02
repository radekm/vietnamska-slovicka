//! Obrazovka Lekce: procházení lekcí a slovíček a hledání.

use eframe::egui::{
    self, Align, Color32, Label, Layout, RichText, TextEdit, TextWrapMode, WidgetText,
};
use egui_extras::{Column, TableBuilder};
use time::Date;
use vietnamska_slovicka::browse::{CardStatus, Query};
use vietnamska_slovicka::model::{Direction, Lesson, Word};
use vietnamska_slovicka::progress::Progress;
use vietnamska_slovicka::session::czech_plural;
use vietnamska_slovicka::settings::Settings;

use crate::app::Loaded;
use crate::fonts;

/// Stav obrazovky Lekce. Pamatuje si ho aplikace, aby se po návratu na obrazovku
/// neztratilo hledání ani vybraná lekce.
#[derive(Default)]
pub struct LessonsView {
    /// Vybraná lekce, `None` znamená všechny lekce.
    lesson: Option<u32>,
    /// Hledaný text. Když je vyplněný, hledá se ve všech lekcích.
    query: String,
}

/// Velikosti písma v tabulce.
const VI_SIZE: f32 = 17.0;
const TEXT_SIZE: f32 = 15.0;
const EXTRA_SIZE: f32 = 13.0;
/// Svislý okraj řádku tabulky (nahoře a dole dohromady).
const ROW_PADDING: f32 = 8.0;
/// Šířka sloupce s číslem lekce a sloupců se stavem kartiček.
const LESSON_WIDTH: f32 = 48.0;
const STATUS_WIDTH: f32 = 92.0;
/// Podíl textových sloupců (VI, CS, výslovnost, IPA) na zbylé šířce a jejich nejmenší šířka.
const TEXT_SHARES: [f32; 4] = [0.22, 0.30, 0.28, 0.20];
const TEXT_MIN_WIDTH: f32 = 90.0;

/// Jeden řádek tabulky.
struct Row<'a> {
    lesson: u32,
    word: &'a Word,
    /// Stav kartiček ve směrech [`Direction::ALL`], `None` u vypnutého směru.
    statuses: [Option<CardStatus>; 2],
}

/// Vykreslí obrazovku Lekce.
pub fn show(ui: &mut egui::Ui, loaded: &mut Loaded, today: Date) {
    let Loaded {
        lessons,
        study,
        settings,
        lessons_view: view,
        lessons_dir,
        ..
    } = loaded;

    egui::Panel::left("lessons_list")
        .resizable(true)
        .default_size(260.0)
        .min_size(180.0)
        .show(ui, |ui| lesson_list(ui, lessons, view));

    let query = Query::new(&view.query);
    let searching = !query.is_empty();
    let scope = if searching { None } else { view.lesson };
    let rows: Vec<Row<'_>> = lessons
        .iter()
        .filter(|lesson| scope.is_none_or(|number| number == lesson.number))
        .flat_map(|lesson| lesson.words.iter().map(move |word| (lesson.number, word)))
        .filter(|(_, word)| query.matches(word))
        .map(|(lesson, word)| Row {
            lesson,
            word,
            statuses: statuses(word, settings, &study.progress, today),
        })
        .collect();

    egui::CentralPanel::default().show(ui, |ui| {
        heading(ui, lessons, scope, searching, rows.len());
        ui.add_space(8.0);
        if lessons.is_empty() {
            ui.label(format!(
                "Ve složce {} nejsou žádné lekce (soubory lekce01.toml, lekce02.toml, …).",
                lessons_dir.display()
            ));
        } else if rows.is_empty() {
            ui.label(RichText::new("Nic nenalezeno.").weak());
        } else {
            words_table(ui, &rows, scope.is_none());
        }
    });
}

/// Levý panel: pole pro hledání a seznam lekcí.
fn lesson_list(ui: &mut egui::Ui, lessons: &[Lesson], view: &mut LessonsView) {
    ui.add_space(8.0);
    ui.label(RichText::new("Hledat").family(fonts::bold()));
    // Zprava doleva: nejdřív tlačítko, pole pak vyplní přesně zbytek šířky. Kdyby bylo
    // pole širší než panel, panel by se s každým snímkem rozšiřoval.
    let row = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(row, Layout::right_to_left(Align::Center), |ui| {
        if ui
            .add_enabled(!view.query.is_empty(), egui::Button::new("✖"))
            .on_hover_text("Smazat hledání")
            .clicked()
        {
            view.query.clear();
        }
        let edit = TextEdit::singleline(&mut view.query)
            .hint_text("např. cam on, dekuji")
            .desired_width(ui.available_width());
        ui.add(edit);
    });
    ui.label(
        RichText::new("Hledá ve všech lekcích, na diakritice nezáleží.")
            .weak()
            .size(12.0),
    );
    ui.add_space(6.0);
    ui.separator();

    let searching = !view.query.trim().is_empty();
    egui::ScrollArea::vertical().show(ui, |ui| {
        let total: usize = lessons.iter().map(|lesson| lesson.words.len()).sum();
        let all = format!("Všechny lekce ({total})");
        if ui
            .selectable_label(!searching && view.lesson.is_none(), all)
            .clicked()
        {
            view.lesson = None;
            view.query.clear();
        }
        for lesson in lessons {
            let text = format!(
                "{}. {} ({})",
                lesson.number,
                lesson.title,
                lesson.words.len()
            );
            let selected = !searching && view.lesson == Some(lesson.number);
            let response = ui.selectable_label(selected, text);
            let response = match &lesson.description {
                Some(description) => response.on_hover_text(description),
                None => response,
            };
            // Výběr lekce zruší hledání, jinak by se vybraná lekce neprojevila.
            if response.clicked() {
                view.lesson = Some(lesson.number);
                view.query.clear();
            }
        }
    });
}

/// Nadpis nad tabulkou podle toho, co se zobrazuje.
fn heading(ui: &mut egui::Ui, lessons: &[Lesson], scope: Option<u32>, searching: bool, n: usize) {
    let count = u32::try_from(n).unwrap_or(u32::MAX);
    let words = czech_plural(count, "slovíčko", "slovíčka", "slovíček");
    let lesson = scope.and_then(|number| lessons.iter().find(|l| l.number == number));
    let (title, subtitle) = match lesson {
        _ if searching => (
            "Výsledky hledání".to_owned(),
            Some(format!("{count} {words}")),
        ),
        Some(lesson) => (
            format!("{}. {}", lesson.number, lesson.title),
            Some(match &lesson.description {
                Some(description) => format!("{description} · {count} {words}"),
                None => format!("{count} {words}"),
            }),
        ),
        None => ("Všechny lekce".to_owned(), Some(format!("{count} {words}"))),
    };
    ui.label(RichText::new(title).size(24.0).family(fonts::bold()));
    if let Some(subtitle) = subtitle {
        ui.label(RichText::new(subtitle).weak());
    }
}

/// Stav obou kartiček slovíčka. U vypnutého směru `None`.
fn statuses(
    word: &Word,
    settings: &Settings,
    progress: &Progress,
    today: Date,
) -> [Option<CardStatus>; 2] {
    word.card_ids().map(|card| {
        settings
            .direction_enabled(card.direction)
            .then(|| CardStatus::of(progress, &card, today))
    })
}

/// Text slovíčka pro jednotlivé sloupce.
fn vi_text(word: &Word) -> RichText {
    RichText::new(&word.vi).size(VI_SIZE).family(fonts::bold())
}

fn cs_text(word: &Word) -> RichText {
    RichText::new(&word.cs).size(TEXT_SIZE)
}

/// Poznámka a štítky na řádek pod češtinou, pokud nějaké jsou.
fn extra_text(word: &Word) -> Option<RichText> {
    let tags = word
        .tags
        .iter()
        .map(|tag| format!("#{tag}"))
        .collect::<Vec<_>>()
        .join(" ");
    let text = match (&word.note, tags.is_empty()) {
        (None, true) => return None,
        (Some(note), true) => note.clone(),
        (None, false) => tags,
        (Some(note), false) => format!("{note} · {tags}"),
    };
    Some(RichText::new(text).size(EXTRA_SIZE).italics().weak())
}

fn pronunciation_text(word: &Word) -> RichText {
    RichText::new(&word.pronunciation_cs).size(TEXT_SIZE)
}

fn ipa_text(word: &Word) -> RichText {
    RichText::new(&word.ipa).size(TEXT_SIZE)
}

/// Výška textu zalomeného do šířky `width`.
fn text_height(ui: &egui::Ui, text: RichText, width: f32) -> f32 {
    WidgetText::from(text)
        .into_galley(ui, Some(TextWrapMode::Wrap), width, egui::TextStyle::Body)
        .size()
        .y
}

/// Popisek se zalamováním textu.
fn wrapped(ui: &mut egui::Ui, text: RichText) {
    ui.add(Label::new(text).wrap());
}

/// Šířky textových sloupců tak, aby tabulka vyplnila šířku `available`.
/// V úzkém okně nejsou sloupce užší než [`TEXT_MIN_WIDTH`] a tabulka se posouvá do strany.
fn text_widths(ui: &egui::Ui, available: f32, show_lesson: bool) -> [f32; 4] {
    let fixed_columns = Direction::ALL.len() + usize::from(show_lesson);
    let fixed =
        STATUS_WIDTH * Direction::ALL.len() as f32 + if show_lesson { LESSON_WIDTH } else { 0.0 };
    // Mezery mezi všemi sloupci.
    let spacing = ui.spacing().item_spacing.x * (fixed_columns + TEXT_SHARES.len()) as f32;
    let free = (available - fixed - spacing).max(0.0);
    TEXT_SHARES.map(|share| (free * share).max(TEXT_MIN_WIDTH).floor())
}

/// Tabulka slovíček. Výška řádku se počítá podle zalomení textu v šířce sloupců.
fn words_table(ui: &mut egui::Ui, rows: &[Row<'_>], show_lesson: bool) {
    let widths = text_widths(ui, ui.available_width(), show_lesson);
    egui::ScrollArea::horizontal()
        .auto_shrink(false)
        .show(ui, |ui| table(ui, rows, show_lesson, widths));
}

fn table(ui: &mut egui::Ui, rows: &[Row<'_>], show_lesson: bool, widths: [f32; 4]) {
    let mut table = TableBuilder::new(ui)
        .striped(true)
        .auto_shrink(false)
        .cell_layout(Layout::top_down(Align::LEFT));
    if show_lesson {
        table = table.column(Column::exact(LESSON_WIDTH));
    }
    for width in widths {
        table = table.column(Column::exact(width).clip(true));
    }
    table = table.columns(Column::exact(STATUS_WIDTH), Direction::ALL.len());

    table
        .header(28.0, |mut header| {
            let mut titles = vec!["Lekce", "Vietnamsky", "Česky", "Výslovnost", "IPA"];
            if !show_lesson {
                titles.remove(0);
            }
            titles.extend(Direction::ALL.map(Direction::label));
            for title in titles {
                header.col(|ui| {
                    ui.label(RichText::new(title).family(fonts::bold()));
                });
            }
        })
        .body(|mut body| {
            let heights: Vec<f32> = rows
                .iter()
                .map(|row| row_height(body.ui_mut(), row.word, &widths))
                .collect();
            body.heterogeneous_rows(heights.into_iter(), |mut table_row| {
                let row = &rows[table_row.index()];
                let word = row.word;
                if show_lesson {
                    table_row.col(|ui| {
                        ui.label(RichText::new(row.lesson.to_string()).weak());
                    });
                }
                table_row.col(|ui| wrapped(ui, vi_text(word)));
                table_row.col(|ui| {
                    wrapped(ui, cs_text(word));
                    if let Some(extra) = extra_text(word) {
                        wrapped(ui, extra);
                    }
                });
                table_row.col(|ui| wrapped(ui, pronunciation_text(word)));
                table_row.col(|ui| wrapped(ui, ipa_text(word)));
                for status in row.statuses {
                    table_row.col(|ui| {
                        status_label(ui, status);
                    });
                }
            });
        });
}

/// Výška řádku: nejvyšší z buněk s textem plus okraj.
fn row_height(ui: &egui::Ui, word: &Word, widths: &[f32]) -> f32 {
    let spacing = ui.spacing().item_spacing.y;
    let cs = text_height(ui, cs_text(word), widths[1])
        + extra_text(word).map_or(0.0, |extra| spacing + text_height(ui, extra, widths[1]));
    [
        text_height(ui, vi_text(word), widths[0]),
        cs,
        text_height(ui, pronunciation_text(word), widths[2]),
        text_height(ui, ipa_text(word), widths[3]),
    ]
    .into_iter()
    .fold(0.0, f32::max)
        + ROW_PADDING
}

/// Stav kartičky v tabulce.
fn status_label(ui: &mut egui::Ui, status: Option<CardStatus>) {
    let text = match status {
        None => RichText::new("vypnuto").weak(),
        Some(CardStatus::New) => RichText::new("nová").weak(),
        Some(status @ CardStatus::Due) => RichText::new(status.label())
            .family(fonts::bold())
            .color(Color32::from_rgb(50, 140, 75)),
        Some(status) => RichText::new(status.label()),
    };
    ui.label(text);
}
