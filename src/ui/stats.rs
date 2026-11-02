//! Obrazovka Statistiky: souhrn a tři grafy (opakování za den, předpověď, naučené kartičky).
//!
//! Všechno se počítá v knihovně (`stats.rs`), tady se jen vykresluje.

use eframe::egui::{self, Color32, RichText};
use egui_plot::{
    Bar, BarChart, GridInput, GridMark, HoverPosition, Line, Plot, uniform_grid_spacer,
};
use time::{Date, Weekday};
use vietnamska_slovicka::fsrs::Rating;
use vietnamska_slovicka::stats::{self, DAYS, LEARNED_STABILITY, Stats};

use crate::app::Loaded;
use crate::fonts;
use crate::ui::rating_color;

/// Výška jednoho grafu.
const PLOT_HEIGHT: f32 = 190.0;

/// Barva čáry zavedených kartiček.
const INTRODUCED_COLOR: Color32 = Color32::from_rgb(120, 130, 150);

/// Barva sloupců předpovědi.
const FORECAST_COLOR: Color32 = Color32::from_rgb(45, 105, 185);

/// Stav obrazovky: spočítané statistiky.
pub struct StatsState {
    stats: Stats,
}

impl StatsState {
    /// Spočítá statistiky pro studijní den `today`.
    pub fn new(loaded: &Loaded, today: Date) -> Self {
        Self {
            stats: compute(loaded, today),
        }
    }
}

/// Spočítá statistiky a zaloguje, když historie nesedí s postupem.
fn compute(loaded: &Loaded, today: Date) -> Stats {
    let stats = stats::compute(
        &loaded.lessons,
        &loaded.settings,
        &loaded.study.progress,
        &loaded.study.history,
        today,
    );
    if stats.mismatched > 0 {
        tracing::warn!(
            cards = stats.mismatched,
            "historie hodnocení nesedí s postupem, statistiky naučených mohou být nepřesné"
        );
    }
    stats
}

/// Vykreslí statistiky.
pub fn show(ui: &mut egui::Ui, loaded: &mut Loaded, today: Date) {
    // Statistiky se počítají při otevření obrazovky a znovu jen při změně studijního dne
    // (data se mezitím změnit nemohou, opakuje se na jiné obrazovce).
    let stale =
        matches!(&loaded.screen, crate::app::Screen::Stats(state) if state.stats.today != today);
    if stale {
        loaded.screen = crate::app::Screen::Stats(StatsState::new(loaded, today));
    }
    let crate::app::Screen::Stats(state) = &loaded.screen else {
        return;
    };
    let stats = &state.stats;

    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            summary(ui, stats);
            ui.add_space(12.0);
            let rating_legend: Vec<(Color32, &str)> = Rating::ALL
                .into_iter()
                .map(|rating| (rating_color(rating), rating.label()))
                .collect();
            section_title(ui, "Opakování za den", "posledních 30 dní", &rating_legend);
            reviews_plot(ui, stats);
            ui.add_space(10.0);
            section_title(
                ui,
                "Předpověď opakování",
                "kolik kartiček bude na řadě v dalších 30 dnech, prošlé jsou u dneška",
                &[],
            );
            forecast_plot(ui, stats);
            ui.add_space(10.0);
            section_title(
                ui,
                "Naučené kartičky",
                &format!(
                    "zavedené = aspoň jednou opakované, naučené = stabilita aspoň {LEARNED_STABILITY:.0} dní"
                ),
                &[
                    (INTRODUCED_COLOR, "Zavedené"),
                    (rating_color(Rating::Good), "Naučené"),
                ],
            );
            learned_plot(ui, stats);
        });
    });
}

/// Řádek se souhrnnými čísly.
fn summary(ui: &mut egui::Ui, stats: &Stats) {
    let learned = stats.learned_today();
    let success = match stats.success.ratio() {
        Some(ratio) => format!("{:.0} %", ratio * 100.0),
        None => "–".to_owned(),
    };
    let success_hint = format!(
        "Podíl opakování za posledních 30 dní, která nebyla „Znovu“ ({} z {}).\n\
         Nepočítá se první hodnocení nové kartičky ani zopakování týž den.",
        stats.success.passed, stats.success.reviews
    );
    ui.horizontal(|ui| {
        tile(
            ui,
            &stats.reviewed_today().to_string(),
            "dnes ohodnoceno",
            None,
        );
        tile(
            ui,
            &learned.introduced.to_string(),
            "zavedeno kartiček",
            None,
        );
        tile(
            ui,
            &learned.learned.to_string(),
            "naučeno kartiček",
            Some("Kartičky se stabilitou FSRS aspoň 21 dní."),
        );
        tile(ui, &success, "úspěšnost za 30 dní", Some(&success_hint));
    });
}

/// Jedno souhrnné číslo s popiskem.
fn tile(ui: &mut egui::Ui, value: &str, label: &str, hint: Option<&str>) {
    let response = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.set_min_width(170.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(value).size(26.0).family(fonts::bold()));
                ui.label(RichText::new(label).size(14.0).weak());
            });
        })
        .response;
    if let Some(hint) = hint {
        response.on_hover_text(hint);
    }
}

/// Nadpis grafu s krátkým vysvětlením a legendou vpravo.
///
/// Legenda je vlastní, protože legenda `egui_plot` řadí položky abecedně a překrývá graf.
fn section_title(ui: &mut egui::Ui, title: &str, note: &str, legend: &[(Color32, &str)]) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).size(18.0).family(fonts::bold()));
        ui.label(RichText::new(note).size(13.0).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Zprava doleva, proto pozpátku, aby pořadí odpovídalo čtení.
            ui.add_space(12.0);
            for (color, label) in legend.iter().rev() {
                ui.label(RichText::new(*label).size(14.0));
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, *color);
                ui.add_space(6.0);
            }
        });
    });
}

/// Graf bez posouvání a přibližování, osa X ve dnech (0 až DAYS - 1).
///
/// `first_day` je den na pozici 0, `anchor` pozice, od které se počítají týdenní značky
/// (dnešek), aby popisky dat vycházely na dnešek.
fn base_plot(id: &'static str, first_day: Date, anchor: usize) -> Plot<'static> {
    Plot::new(id)
        .height(PLOT_HEIGHT)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .allow_double_click_reset(false)
        .show_crosshair(false)
        // Popisek jen u dat (sloupce mají vlastní `element_formatter`), ne souřadnice kurzoru.
        .label_formatter(|_position| None)
        .include_x(-0.6)
        .include_x(DAYS as f64 - 0.4)
        .include_y(0.0)
        .x_grid_spacer(move |input| day_marks(input, anchor))
        .x_axis_formatter(move |mark, _range| match day_at(first_day, mark.value) {
            Some(day) => short_date(day),
            None => String::new(),
        })
        .y_grid_spacer(uniform_grid_spacer(count_steps))
        .y_axis_formatter(|mark, _range| integer_label(mark.value))
}

/// Kroky mřížky na ose Y: celá „hezká“ čísla (1, 2, 5, 10, 20…), asi šest popisků na výšku.
fn count_steps(input: GridInput) -> [f64; 3] {
    let (min, max) = input.bounds;
    let wanted = ((max - min) / 6.0).max(1.0);
    let magnitude = 10_f64.powf(wanted.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|factor| factor * magnitude)
        .find(|step| *step >= wanted)
        .unwrap_or(10.0 * magnitude);
    [step, step * 5.0, step * 10.0]
}

/// Značky na ose X: každý den a silnější každý týden od pozice `anchor`.
/// Popisky se ukážou jen u značek, které jsou dost daleko od sebe.
fn day_marks(input: GridInput, anchor: usize) -> Vec<GridMark> {
    let (min, max) = input.bounds;
    let mut marks = Vec::new();
    for index in 0..DAYS {
        let value = index as f64;
        if value < min || value > max {
            continue;
        }
        let weekly = index.abs_diff(anchor) % 7 == 0;
        marks.push(GridMark {
            value,
            step_size: if weekly { 7.0 } else { 1.0 },
        });
    }
    marks
}

/// Den na pozici `value` osy X, pokud je to celé číslo v rozsahu grafu.
fn day_at(first_day: Date, value: f64) -> Option<Date> {
    let index = value.round();
    if (value - index).abs() > 1e-6 || !(0.0..DAYS as f64).contains(&index) {
        return None;
    }
    // Hodnota je ověřené celé číslo 0..DAYS, převod je přesný.
    first_day.checked_add(time::Duration::days(index as i64))
}

/// Popisek osy Y: jen celá čísla (počty kartiček).
fn integer_label(value: f64) -> String {
    let rounded = value.round();
    if (value - rounded).abs() > 1e-6 || rounded < 0.0 {
        String::new()
    } else {
        format!("{rounded:.0}")
    }
}

/// Datum ve tvaru „2. 11.“.
fn short_date(day: Date) -> String {
    format!("{}. {}.", day.day(), u8::from(day.month()))
}

/// Datum s dnem v týdnu ve tvaru „po 2. 11.“.
fn long_date(day: Date) -> String {
    let weekday = match day.weekday() {
        Weekday::Monday => "po",
        Weekday::Tuesday => "út",
        Weekday::Wednesday => "st",
        Weekday::Thursday => "čt",
        Weekday::Friday => "pá",
        Weekday::Saturday => "so",
        Weekday::Sunday => "ne",
    };
    format!("{weekday} {}", short_date(day))
}

/// Horní mez osy Y s rezervou nad nejvyšším sloupcem, aby se do grafu vešel popisek
/// při najetí myší.
fn headroom(max: usize) -> f64 {
    (max as f64 * 1.35).max(4.0)
}

/// Skládaný sloupcový graf hodnocení za den.
fn reviews_plot(ui: &mut egui::Ui, stats: &Stats) {
    let first_day = stats.reviews.first().map_or(stats.today, |d| d.day);
    let mut charts: Vec<BarChart> = Vec::new();
    for rating in Rating::ALL {
        let bars = stats
            .reviews
            .iter()
            .enumerate()
            .map(|(index, day)| {
                Bar::new(index as f64, day.count(rating) as f64)
                    .width(0.7)
                    .fill(rating_color(rating))
            })
            .collect();
        let totals: Vec<usize> = stats.reviews.iter().map(|day| day.total()).collect();
        let mut chart = BarChart::new(rating.label(), bars)
            .color(rating_color(rating))
            .element_formatter(Box::new(move |bar, _chart| {
                let day = day_at(first_day, bar.argument).map_or_else(String::new, long_date);
                let total = totals.get(bar.argument as usize).copied().unwrap_or(0);
                format!(
                    "{day}: celkem {total}\n{}: {:.0}",
                    rating.label(),
                    bar.value
                )
            }));
        let below: Vec<&BarChart> = charts.iter().collect();
        chart = chart.stack_on(&below);
        charts.push(chart);
    }
    let max = stats
        .reviews
        .iter()
        .map(|day| day.total())
        .max()
        .unwrap_or(0);
    base_plot("stats_reviews", first_day, DAYS - 1)
        .include_y(headroom(max))
        .show(ui, |plot_ui| {
            for chart in charts {
                plot_ui.bar_chart(chart);
            }
        });
}

/// Sloupcový graf předpovědi opakování.
fn forecast_plot(ui: &mut egui::Ui, stats: &Stats) {
    let first_day = stats.today;
    let bars = stats
        .forecast
        .iter()
        .enumerate()
        .map(|(index, day)| {
            Bar::new(index as f64, day.count as f64)
                .width(0.7)
                .fill(FORECAST_COLOR)
        })
        .collect();
    let chart = BarChart::new("K opakování", bars)
        .color(FORECAST_COLOR)
        .element_formatter(Box::new(move |bar, _chart| {
            let day = day_at(first_day, bar.argument).map_or_else(String::new, long_date);
            let when = if bar.argument < 0.5 { " (dnes)" } else { "" };
            format!("{day}{when}\nk opakování: {:.0}", bar.value)
        }));
    let max = stats
        .forecast
        .iter()
        .map(|day| day.count)
        .max()
        .unwrap_or(0);
    base_plot("stats_forecast", first_day, 0)
        .include_y(headroom(max))
        .show(ui, |plot_ui| plot_ui.bar_chart(chart));
}

/// Čárový graf zavedených a naučených kartiček.
fn learned_plot(ui: &mut egui::Ui, stats: &Stats) {
    let first_day = stats.learned.first().map_or(stats.today, |d| d.day);
    let points = |value: fn(&stats::DayLearned) -> usize| -> Vec<[f64; 2]> {
        stats
            .learned
            .iter()
            .enumerate()
            .map(|(index, day)| [index as f64, value(day) as f64])
            .collect()
    };
    let introduced = Line::new("Zavedené", points(|day| day.introduced))
        .color(INTRODUCED_COLOR)
        .width(2.0);
    let learned = Line::new("Naučené", points(|day| day.learned))
        .color(rating_color(Rating::Good))
        .width(2.5);
    let max = stats
        .learned
        .iter()
        .map(|day| day.introduced)
        .max()
        .unwrap_or(0);
    base_plot("stats_learned", first_day, DAYS - 1)
        .include_y((max as f64 * 1.1).max(4.0))
        .label_formatter(move |position| match position {
            HoverPosition::NearDataPoint {
                plot_name,
                position,
                ..
            } => {
                let day = day_at(first_day, position.x).map_or_else(String::new, long_date);
                Some(format!("{day}\n{plot_name}: {:.0}", position.y))
            }
            HoverPosition::Elsewhere { .. } => None,
        })
        .show(ui, |plot_ui| {
            plot_ui.line(introduced);
            plot_ui.line(learned);
        });
}
