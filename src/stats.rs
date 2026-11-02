//! Výpočty pro obrazovku Statistiky: opakování za den, předpověď opakování
//! a počet zavedených a naučených kartiček v čase.
//!
//! Pravidla popisuje `PLAN.md` (upřesnění etapy 7). Všechno se počítá po studijních
//! dnech a za oba směry dohromady.

use std::collections::{HashMap, HashSet};

use time::{Date, Duration};

use crate::fsrs::{CardState, Fsrs, Rating, study_day};
use crate::model::{CardId, Lesson};
use crate::progress::{History, Progress};
use crate::settings::Settings;

/// Kolik dní ukazují grafy.
pub const DAYS: usize = 30;

/// Od jaké stability (ve dnech) je kartička naučená.
pub const LEARNED_STABILITY: f64 = 21.0;

/// Spočítané statistiky.
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    /// Studijní den, pro který se statistiky spočítaly.
    pub today: Date,
    /// Hodnocení za posledních [`DAYS`] dní, od nejstaršího po dnešek.
    pub reviews: Vec<DayReviews>,
    /// Předpověď opakování na dnešek a dalších `DAYS - 1` dní.
    pub forecast: Vec<DayCount>,
    /// Zavedené a naučené kartičky na konci každého z posledních [`DAYS`] dní,
    /// od nejstaršího po dnešek.
    pub learned: Vec<DayLearned>,
    /// Úspěšnost opakování za posledních [`DAYS`] dní.
    pub success: SuccessRate,
    /// Kolik kartiček má po přehrání historie jinou stabilitu nebo datum příštího opakování
    /// než v postupu. Mělo by být 0, jinak historie nesedí s postupem (jen pro log).
    pub mismatched: usize,
}

/// Počty hodnocení v jednom dni.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayReviews {
    /// Studijní den.
    pub day: Date,
    /// Počty v pořadí Znovu, Těžké, Dobré, Snadné.
    pub by_rating: [usize; 4],
}

impl DayReviews {
    /// Počet hodnocení `rating`.
    pub fn count(&self, rating: Rating) -> usize {
        self.by_rating[rating_index(rating)]
    }

    /// Celkový počet hodnocení.
    pub fn total(&self) -> usize {
        self.by_rating.iter().sum()
    }
}

/// Počet kartiček v jednom dni.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayCount {
    /// Studijní den.
    pub day: Date,
    /// Počet kartiček.
    pub count: usize,
}

/// Zavedené a naučené kartičky na konci jednoho dne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayLearned {
    /// Studijní den.
    pub day: Date,
    /// Kartičky, které už byly aspoň jednou ohodnoceny.
    pub introduced: usize,
    /// Kartičky se stabilitou aspoň [`LEARNED_STABILITY`] dní.
    pub learned: usize,
}

/// Úspěšnost opakování: podíl opakování v jiný den než předchozí opakování,
/// která nebyla „Znovu“.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SuccessRate {
    /// Počet započítaných opakování.
    pub reviews: usize,
    /// Kolik z nich nebylo „Znovu“.
    pub passed: usize,
}

impl SuccessRate {
    /// Úspěšnost v rozsahu 0 až 1, nebo `None`, když žádné opakování nebylo.
    pub fn ratio(&self) -> Option<f64> {
        // Počty jsou malé, převod na f64 je přesný.
        (self.reviews > 0).then(|| self.passed as f64 / self.reviews as f64)
    }
}

impl Stats {
    /// Kolik hodnocení bylo dnes.
    pub fn reviewed_today(&self) -> usize {
        self.reviews.last().map_or(0, DayReviews::total)
    }

    /// Zavedené a naučené kartičky dnes.
    pub fn learned_today(&self) -> DayLearned {
        self.learned.last().copied().unwrap_or(DayLearned {
            day: self.today,
            introduced: 0,
            learned: 0,
        })
    }
}

fn rating_index(rating: Rating) -> usize {
    usize::from(rating.number() - 1)
}

/// Den posunutý o `days` dní. Mimo rozsah dat (rok 9999) zůstane na mezi.
fn add_days(day: Date, days: i64) -> Date {
    day.checked_add(Duration::days(days))
        .unwrap_or(if days < 0 { Date::MIN } else { Date::MAX })
}

/// Spočítá statistiky pro studijní den `today`.
///
/// Lekce určují, které kartičky se počítají do předpovědi a do zavedených a naučených
/// (osiřelé ne). Opakování za den a úspěšnost berou celou historii.
pub fn compute(
    lessons: &[Lesson],
    settings: &Settings,
    progress: &Progress,
    history: &History,
    today: Date,
) -> Stats {
    // Rozsah grafů do minulosti: dnešek je poslední z DAYS dní.
    let first_day = add_days(today, -(DAYS as i64 - 1));
    let days: Vec<Date> = (0..DAYS).map(|i| add_days(first_day, i as i64)).collect();

    let lesson_cards: HashSet<CardId> = lessons
        .iter()
        .flat_map(|lesson| &lesson.words)
        .flat_map(|word| word.card_ids())
        .collect();

    let replay = replay_history(history);

    Stats {
        today,
        reviews: reviews_per_day(history, &days),
        forecast: forecast(&lesson_cards, settings, progress, today),
        learned: learned_per_day(&replay.timelines, &lesson_cards, &days),
        success: success_rate(&replay.day_reviews, first_day, today),
        mismatched: count_mismatched(&replay.final_states, progress),
    }
}

/// Počty hodnocení po dnech.
fn reviews_per_day(history: &History, days: &[Date]) -> Vec<DayReviews> {
    let mut result: Vec<DayReviews> = days
        .iter()
        .map(|&day| DayReviews {
            day,
            by_rating: [0; 4],
        })
        .collect();
    let Some(first_day) = days.first().copied() else {
        return result;
    };
    for entry in history.entries() {
        let day = study_day(entry.time);
        let offset = (day - first_day).whole_days();
        if let Ok(index) = usize::try_from(offset)
            && let Some(slot) = result.get_mut(index)
        {
            slot.by_rating[rating_index(entry.rating)] += 1;
        }
    }
    result
}

/// Předpověď: kolik kartiček je na řadě v jednotlivé dny od dneška.
/// Prošlé kartičky se připočtou k dnešku.
fn forecast(
    lesson_cards: &HashSet<CardId>,
    settings: &Settings,
    progress: &Progress,
    today: Date,
) -> Vec<DayCount> {
    let mut result: Vec<DayCount> = (0..DAYS)
        .map(|i| DayCount {
            day: add_days(today, i as i64),
            count: 0,
        })
        .collect();
    for (id, card) in progress.iter() {
        if !lesson_cards.contains(id) || !settings.direction_enabled(id.direction) {
            continue;
        }
        let offset = (card.state.due - today).whole_days().max(0);
        if let Ok(index) = usize::try_from(offset)
            && let Some(slot) = result.get_mut(index)
        {
            slot.count += 1;
        }
    }
    result
}

/// Výsledek přehrání historie přes FSRS.
struct Replay {
    /// Pro každou kartičku stabilita na konci dnů, kdy byla hodnocena, v pořadí dnů.
    timelines: HashMap<CardId, Vec<(Date, f64)>>,
    /// Hodnocení v jiný den než předchozí hodnocení téže kartičky: (den, hodnocení).
    day_reviews: Vec<(Date, Rating)>,
    /// Stav kartiček po posledním hodnocení.
    final_states: HashMap<CardId, CardState>,
}

/// Přehraje historii hodnocení přes FSRS a zjistí stabilitu kartiček v čase.
///
/// Stabilita ani obtížnost nezávisí na cílové retenci, proto stačí výchozí plánovač.
/// Hodnocení se berou v pořadí, v jakém jsou v historii, stejně jako když vznikala.
fn replay_history(history: &History) -> Replay {
    let fsrs = Fsrs::default();
    let mut timelines: HashMap<CardId, Vec<(Date, f64)>> = HashMap::new();
    let mut day_reviews = Vec::new();
    let mut final_states: HashMap<CardId, CardState> = HashMap::new();
    for entry in history.entries() {
        let day = study_day(entry.time);
        let previous = final_states.get(&entry.card);
        if let Some(previous) = previous
            && day > previous.last_review
        {
            day_reviews.push((day, entry.rating));
        }
        let state = fsrs.review(previous, entry.rating, day);
        let timeline = timelines.entry(entry.card.clone()).or_default();
        // Za den se pamatuje jen stav po posledním hodnocení. FSRS bere opakování dřív
        // než poslední (posunuté hodiny) jako stejný den, proto se porovnává s ním.
        match timeline.last_mut() {
            Some((last_day, stability)) if *last_day >= day => {
                *stability = state.memory.stability;
            }
            _ => timeline.push((day, state.memory.stability)),
        }
        final_states.insert(entry.card.clone(), state);
    }
    Replay {
        timelines,
        day_reviews,
        final_states,
    }
}

/// Zavedené a naučené kartičky na konci každého dne.
fn learned_per_day(
    timelines: &HashMap<CardId, Vec<(Date, f64)>>,
    lesson_cards: &HashSet<CardId>,
    days: &[Date],
) -> Vec<DayLearned> {
    let mut result: Vec<DayLearned> = days
        .iter()
        .map(|&day| DayLearned {
            day,
            introduced: 0,
            learned: 0,
        })
        .collect();
    for (id, timeline) in timelines {
        if !lesson_cards.contains(id) {
            continue;
        }
        for slot in &mut result {
            // Poslední hodnocený den nejpozději v tento den.
            let known = timeline.partition_point(|(day, _)| *day <= slot.day);
            if let Some((_, stability)) = known.checked_sub(1).and_then(|i| timeline.get(i)) {
                slot.introduced += 1;
                if *stability >= LEARNED_STABILITY {
                    slot.learned += 1;
                }
            }
        }
    }
    result
}

/// Úspěšnost opakování mezi dny `from` a `to` (včetně).
fn success_rate(day_reviews: &[(Date, Rating)], from: Date, to: Date) -> SuccessRate {
    let mut rate = SuccessRate::default();
    for (day, rating) in day_reviews {
        if (from..=to).contains(day) {
            rate.reviews += 1;
            if *rating != Rating::Again {
                rate.passed += 1;
            }
        }
    }
    rate
}

/// Kolik kartiček z postupu nemá po přehrání historie stejnou stabilitu a příští opakování.
fn count_mismatched(final_states: &HashMap<CardId, CardState>, progress: &Progress) -> usize {
    progress
        .iter()
        .filter(|(id, card)| match final_states.get(*id) {
            None => true,
            Some(state) => {
                let stability = state.memory.stability;
                (stability - card.state.memory.stability).abs() > 1e-9 * stability.max(1.0)
                    // Příští opakování se liší, když se po hodnocení změnila cílová retence.
                    // To je v pořádku, proto se porovnává jen poslední opakování.
                    || state.last_review != card.state.last_review
            }
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Direction, Word};
    use crate::progress::ReviewEntry;
    use time::OffsetDateTime;
    use time::macros::{date, datetime};

    const TODAY: Date = date!(2026 - 11 - 30);

    fn word(vi: &str) -> Word {
        Word {
            vi: vi.to_owned(),
            key_override: None,
            cs: "význam".to_owned(),
            pronunciation_cs: "výslovnost".to_owned(),
            ipa: "ipa".to_owned(),
            note: None,
            tags: Vec::new(),
        }
    }

    fn lessons(words: &[&str]) -> Vec<Lesson> {
        vec![Lesson {
            number: 1,
            file_name: "lekce01.toml".to_owned(),
            title: "Test".to_owned(),
            description: None,
            words: words.iter().map(|vi| word(vi)).collect(),
        }]
    }

    fn card(key: &str, direction: Direction) -> CardId {
        CardId {
            key: key.to_owned(),
            direction,
        }
    }

    /// Ohodnotí kartičku stejně jako aplikace: zapíše postup i historii.
    fn rate(
        progress: &mut Progress,
        history: &mut History,
        id: &CardId,
        rating: Rating,
        time: OffsetDateTime,
    ) {
        let fsrs = Fsrs::default();
        let previous = progress.get(id).map(|card| card.state.clone());
        let state = fsrs.review(previous.as_ref(), rating, study_day(time));
        progress.record(id.clone(), state);
        history.push(ReviewEntry {
            time,
            card: id.clone(),
            rating,
        });
    }

    #[test]
    fn empty_data() {
        let stats = compute(
            &lessons(&["a"]),
            &Settings::default(),
            &Progress::default(),
            &History::default(),
            TODAY,
        );
        assert_eq!(stats.reviews.len(), DAYS);
        assert_eq!(stats.reviews[0].day, date!(2026 - 11 - 01));
        assert_eq!(stats.reviews[DAYS - 1].day, TODAY);
        assert_eq!(stats.forecast.len(), DAYS);
        assert_eq!(stats.forecast[0].day, TODAY);
        assert_eq!(stats.forecast[DAYS - 1].day, date!(2026 - 12 - 29));
        assert_eq!(stats.learned.len(), DAYS);
        assert_eq!(stats.reviewed_today(), 0);
        assert_eq!(stats.learned_today().introduced, 0);
        assert_eq!(stats.success.ratio(), None);
        assert_eq!(stats.mismatched, 0);
    }

    #[test]
    fn reviews_are_counted_per_study_day_and_rating() {
        let mut progress = Progress::default();
        let mut history = History::default();
        let a = card("a", Direction::ViToCs);
        let b = card("b", Direction::ViToCs);
        // Mimo rozsah (31 dní zpět).
        rate(
            &mut progress,
            &mut history,
            &a,
            Rating::Good,
            datetime!(2026-10-31 12:00 +01:00),
        );
        // První den rozsahu.
        rate(
            &mut progress,
            &mut history,
            &b,
            Rating::Again,
            datetime!(2026-11-01 12:00 +01:00),
        );
        rate(
            &mut progress,
            &mut history,
            &b,
            Rating::Good,
            datetime!(2026-11-01 12:05 +01:00),
        );
        // Ve 3:59 to ještě patří k předchozímu studijnímu dni.
        rate(
            &mut progress,
            &mut history,
            &a,
            Rating::Easy,
            datetime!(2026-11-30 03:59 +01:00),
        );
        rate(
            &mut progress,
            &mut history,
            &b,
            Rating::Hard,
            datetime!(2026-11-30 04:00 +01:00),
        );

        let stats = compute(
            &lessons(&["a", "b"]),
            &Settings::default(),
            &progress,
            &history,
            TODAY,
        );
        assert_eq!(stats.reviews[0].by_rating, [1, 0, 1, 0]);
        assert_eq!(stats.reviews[DAYS - 2].count(Rating::Easy), 1);
        assert_eq!(stats.reviews[DAYS - 1].by_rating, [0, 1, 0, 0]);
        assert_eq!(stats.reviewed_today(), 1);
        let total: usize = stats.reviews.iter().map(DayReviews::total).sum();
        assert_eq!(total, 4);
        assert_eq!(stats.mismatched, 0);
    }

    #[test]
    fn forecast_puts_overdue_today_and_skips_orphans_and_disabled_directions() {
        let mut progress = Progress::default();
        let mut history = History::default();
        // Hodnocení „Znovu“ dává interval 1 den, „Dobré“ u nové kartičky 3 dny.
        let time = datetime!(2026-11-20 10:00 +01:00);
        rate(
            &mut progress,
            &mut history,
            &card("a", Direction::ViToCs),
            Rating::Again,
            time,
        );
        rate(
            &mut progress,
            &mut history,
            &card("b", Direction::ViToCs),
            Rating::Again,
            time,
        );
        rate(
            &mut progress,
            &mut history,
            &card("b", Direction::CsToVi),
            Rating::Again,
            time,
        );
        rate(
            &mut progress,
            &mut history,
            &card("orphan", Direction::ViToCs),
            Rating::Again,
            time,
        );
        let time = datetime!(2026-11-30 10:00 +01:00);
        rate(
            &mut progress,
            &mut history,
            &card("c", Direction::ViToCs),
            Rating::Good,
            time,
        );

        let settings = Settings {
            directions: vec![Direction::ViToCs],
            ..Settings::default()
        };
        let stats = compute(
            &lessons(&["a", "b", "c"]),
            &settings,
            &progress,
            &history,
            TODAY,
        );
        let due_on = |day: Date| stats.forecast.iter().find(|d| d.day == day).unwrap().count;
        assert_eq!(due_on(TODAY), 2, "prošlé a, b (VI → CS)");
        let c_due = progress
            .get(&card("c", Direction::ViToCs))
            .unwrap()
            .state
            .due;
        assert!(c_due > TODAY);
        assert_eq!(due_on(c_due), 1);
        let total: usize = stats.forecast.iter().map(|d| d.count).sum();
        assert_eq!(total, 3);
    }

    #[test]
    fn learned_cards_follow_replayed_stability() {
        let mut progress = Progress::default();
        let mut history = History::default();
        let a = card("a", Direction::ViToCs);
        let orphan = card("orphan", Direction::ViToCs);
        // Kartička „a“ hodnocená „Snadné“ v den, kdy je na řadě, dokud nemá stabilitu
        // aspoň 21 dní.
        let mut day = date!(2026 - 11 - 02);
        let mut learned_on = None;
        while learned_on.is_none() {
            let time = day
                .with_hms(10, 0, 0)
                .unwrap()
                .assume_offset(time::UtcOffset::UTC);
            rate(&mut progress, &mut history, &a, Rating::Easy, time);
            rate(&mut progress, &mut history, &orphan, Rating::Easy, time);
            let state = &progress.get(&a).unwrap().state;
            if state.memory.stability >= LEARNED_STABILITY {
                learned_on = Some(day);
            }
            day = state.due;
        }
        let learned_on = learned_on.unwrap();
        assert!(
            learned_on <= TODAY,
            "test potřebuje naučení v rozsahu grafu"
        );

        let stats = compute(
            &lessons(&["a", "b"]),
            &Settings::default(),
            &progress,
            &history,
            TODAY,
        );
        for point in &stats.learned {
            let introduced = usize::from(point.day >= date!(2026 - 11 - 02));
            let learned = usize::from(point.day >= learned_on);
            assert_eq!(point.introduced, introduced, "zavedené {}", point.day);
            assert_eq!(point.learned, learned, "naučené {}", point.day);
        }
        assert_eq!(stats.learned_today().learned, 1);
        assert_eq!(stats.mismatched, 0);
    }

    #[test]
    fn success_rate_counts_only_reviews_on_later_days() {
        let mut progress = Progress::default();
        let mut history = History::default();
        let a = card("a", Direction::ViToCs);
        // První hodnocení a zopakování týž den se nepočítají.
        rate(
            &mut progress,
            &mut history,
            &a,
            Rating::Again,
            datetime!(2026-11-10 10:00 +01:00),
        );
        rate(
            &mut progress,
            &mut history,
            &a,
            Rating::Good,
            datetime!(2026-11-10 10:05 +01:00),
        );
        rate(
            &mut progress,
            &mut history,
            &a,
            Rating::Again,
            datetime!(2026-11-13 10:00 +01:00),
        );
        rate(
            &mut progress,
            &mut history,
            &a,
            Rating::Good,
            datetime!(2026-11-13 10:05 +01:00),
        );
        rate(
            &mut progress,
            &mut history,
            &a,
            Rating::Hard,
            datetime!(2026-11-14 10:00 +01:00),
        );
        rate(
            &mut progress,
            &mut history,
            &a,
            Rating::Good,
            datetime!(2026-11-20 10:00 +01:00),
        );

        let stats = compute(
            &lessons(&["a"]),
            &Settings::default(),
            &progress,
            &history,
            TODAY,
        );
        assert_eq!(
            stats.success,
            SuccessRate {
                reviews: 3,
                passed: 2
            }
        );
        assert!((stats.success.ratio().unwrap() - 2.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn mismatch_is_detected_when_history_is_missing() {
        let mut progress = Progress::default();
        let mut history = History::default();
        let time = datetime!(2026-11-20 10:00 +01:00);
        rate(
            &mut progress,
            &mut history,
            &card("a", Direction::ViToCs),
            Rating::Good,
            time,
        );
        let stats = compute(
            &lessons(&["a"]),
            &Settings::default(),
            &progress,
            &History::default(),
            TODAY,
        );
        assert_eq!(stats.mismatched, 1);
        assert_eq!(stats.learned_today().introduced, 0);
    }
}
