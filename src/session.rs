//! Průběh jednoho sezení opakování a ukládání hodnocení.
//!
//! [`Session`] řeší jen pořadí kartiček (rovnoměrné rozložení nových, oddělení kartiček
//! téhož slovíčka, návrat po „Znovu“). [`Study`] drží postup a historii, počítá FSRS
//! a ukládá každé hodnocení na disk. Pravidla popisuje `PLAN.md`, oddíl „Průběh opakování“.

use std::collections::VecDeque;
use std::path::PathBuf;

use anyhow::{Context, Result};
use time::{Date, OffsetDateTime};

use crate::fsrs::{Fsrs, Rating, study_day};
use crate::model::CardId;
use crate::paths::AppDirs;
use crate::progress::{History, Progress, ReviewEntry};
use crate::scheduler::Queue;

/// O kolik kartiček dál se vrací kartička ohodnocená „Znovu“.
pub const AGAIN_GAP: usize = 5;

/// Jedno sezení opakování.
#[derive(Debug, Clone)]
pub struct Session {
    queue: VecDeque<CardId>,
    summary: SessionSummary,
}

/// Souhrn hodnocení v sezení.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SessionSummary {
    /// Počet hodnocení podle hodnocení, v pořadí Znovu, Těžké, Dobré, Snadné.
    pub by_rating: [usize; 4],
}

impl SessionSummary {
    /// Počet hodnocení `rating`.
    pub fn count(&self, rating: Rating) -> usize {
        self.by_rating[rating_index(rating)]
    }

    /// Celkový počet hodnocení.
    pub fn total(&self) -> usize {
        self.by_rating.iter().sum()
    }
}

fn rating_index(rating: Rating) -> usize {
    usize::from(rating.number() - 1)
}

impl Session {
    /// Sezení z fronty: nové kartičky rovnoměrně mezi opakováními a kartičky téhož
    /// slovíčka ne hned po sobě.
    pub fn new(queue: Queue) -> Self {
        let mut cards = interleave(queue.due, queue.new);
        separate_siblings(&mut cards);
        Self {
            queue: cards.into(),
            summary: SessionSummary::default(),
        }
    }

    /// Kartička na řadě, nebo `None`, když je sezení hotové.
    pub fn current(&self) -> Option<&CardId> {
        self.queue.front()
    }

    /// Zaznamená hodnocení aktuální kartičky a posune se na další.
    ///
    /// Kartička ohodnocená „Znovu“ se vrátí o [`AGAIN_GAP`] kartiček dál (nebo na konec)
    /// a v sezení zůstává, dokud nedostane jiné hodnocení.
    pub fn answer(&mut self, rating: Rating) {
        let Some(card) = self.queue.pop_front() else {
            return;
        };
        self.summary.by_rating[rating_index(rating)] += 1;
        if rating == Rating::Again {
            let position = again_position(&self.queue, &card);
            self.queue.insert(position, card);
        }
    }

    /// Počet hodnocení v sezení (včetně opakovaných kartiček).
    pub fn done(&self) -> usize {
        self.summary.total()
    }

    /// Kolik kartiček ještě zbývá.
    pub fn remaining(&self) -> usize {
        self.queue.len()
    }

    /// Je sezení hotové?
    pub fn is_finished(&self) -> bool {
        self.queue.is_empty()
    }

    /// Souhrn hodnocení.
    pub fn summary(&self) -> SessionSummary {
        self.summary
    }
}

/// Rozprostře nové kartičky rovnoměrně mezi kartičky k opakování.
///
/// Po každé pozici má být zařazeno tolik nových, kolik odpovídá jejich podílu v celé
/// frontě, zaokrouhleno k nejbližšímu. Nové tak vyjdou doprostřed svých úseků
/// (např. 2 opakování a 1 nová: o n o; 4 opakování a 2 nové: o n o o n o).
fn interleave(due: Vec<CardId>, new: Vec<CardId>) -> Vec<CardId> {
    let total = due.len() + new.len();
    let new_count = new.len();
    let mut due = due.into_iter();
    let mut new = new.into_iter();
    let mut result = Vec::with_capacity(total);
    let mut new_taken = 0;
    for position in 0..total {
        // Kolik nových má být zařazeno po této pozici: round((pozice + 1) · N / T)
        // v celých číslech.
        let new_wanted = (2 * (position + 1) * new_count + total) / (2 * total);
        let next = if new_wanted > new_taken {
            new_taken += 1;
            new.next()
        } else {
            due.next()
        };
        // Obě části mají dohromady přesně `total` kartiček, `next` je vždy `Some`.
        // Kdyby nebyl, vezmeme kartičku z druhé části.
        if let Some(card) = next.or_else(|| due.next()).or_else(|| new.next()) {
            result.push(card);
        }
    }
    result
}

/// Zajistí, aby dvě kartičky téhož slovíčka nešly hned po sobě, pokud je mezi ně
/// co dát. Druhou kartičku odsune tím, že před ni přesune nejbližší jinou kartičku.
fn separate_siblings(cards: &mut Vec<CardId>) {
    let mut index = 0;
    while index + 1 < cards.len() {
        if cards[index].key == cards[index + 1].key {
            let other = (index + 2..cards.len()).find(|&j| cards[j].key != cards[index].key);
            if let Some(other) = other {
                let card = cards.remove(other);
                cards.insert(index + 1, card);
            }
        }
        index += 1;
    }
}

/// Kam vrátit kartičku ohodnocenou „Znovu“: o [`AGAIN_GAP`] dál nebo na konec,
/// a pokud by tam sousedila s kartičkou téhož slovíčka, ještě o kus dál.
fn again_position(queue: &VecDeque<CardId>, card: &CardId) -> usize {
    let len = queue.len();
    let sibling_at = |index: usize| queue.get(index).is_some_and(|c| c.key == card.key);
    let mut position = AGAIN_GAP.min(len);
    while position < len && ((position > 0 && sibling_at(position - 1)) || sibling_at(position)) {
        position += 1;
    }
    position
}

/// Postup a historie s plánovačem: počítá a ukládá hodnocení.
#[derive(Debug, Clone)]
pub struct Study {
    /// Postup učení.
    pub progress: Progress,
    /// Historie hodnocení.
    pub history: History,
    fsrs: Fsrs,
    progress_file: PathBuf,
    history_file: PathBuf,
}

impl Study {
    /// Načte postup a historii z datové složky.
    pub fn load(dirs: &AppDirs, fsrs: Fsrs) -> Result<Self> {
        let progress_file = dirs.progress_file();
        let history_file = dirs.history_file();
        Ok(Self {
            progress: Progress::load(&progress_file)?,
            history: History::load(&history_file)?,
            fsrs,
            progress_file,
            history_file,
        })
    }

    /// Plánovač FSRS.
    pub fn fsrs(&self) -> &Fsrs {
        &self.fsrs
    }

    /// Vymění plánovač (po změně cílové retence). Nová retence platí jen pro další
    /// hodnocení, už naplánovaná opakování v postupu se nemění (viz `PLAN.md`).
    pub fn set_fsrs(&mut self, fsrs: Fsrs) {
        self.fsrs = fsrs;
    }

    /// Ohodnotí kartičku v čase `now` (místní čas) a uloží postup i historii.
    ///
    /// Hodnocení se zapíše do paměti vždy. Když uložení selže, vrátí chybu a příští
    /// uložení zapíše všechno znovu (soubory se zapisují celé).
    pub fn rate(&mut self, card: &CardId, rating: Rating, now: OffsetDateTime) -> Result<()> {
        let today = study_day(now);
        let previous = self.progress.get(card).map(|progress| &progress.state);
        let state = self.fsrs.review(previous, rating, today);
        self.progress.record(card.clone(), state);
        self.history.push(ReviewEntry {
            time: now,
            card: card.clone(),
            rating,
        });
        self.save()
    }

    /// Uloží postup i historii. Zkusí obojí, i když první uložení selže.
    pub fn save(&self) -> Result<()> {
        let progress = self.progress.save(&self.progress_file);
        let history = self.history.save(&self.history_file);
        progress
            .and(history)
            .context("hodnocení je v paměti, ale neuložilo se")
    }

    /// Intervaly ve dnech, které by kartička dostala za jednotlivá hodnocení
    /// (v pořadí Znovu, Těžké, Dobré, Snadné). Pro popisky tlačítek.
    pub fn preview(&self, card: &CardId, now: OffsetDateTime) -> [u32; 4] {
        let today = study_day(now);
        let previous = self.progress.get(card).map(|progress| &progress.state);
        Rating::ALL.map(|rating| {
            let state = self.fsrs.review(previous, rating, today);
            days_between(today, state.due)
        })
    }
}

/// Počet dní mezi dvěma daty (nezáporný, s horní mezí `u32::MAX`).
fn days_between(from: Date, to: Date) -> u32 {
    u32::try_from((to - from).whole_days().max(0)).unwrap_or(u32::MAX)
}

/// Popisek intervalu pro tlačítko, např. „za 3 dny“, „za 2 měsíce“, „za 1,5 roku“.
pub fn interval_label(days: u32) -> String {
    if days < 31 {
        return format!("za {days} {}", czech_plural(days, "den", "dny", "dní"));
    }
    if days < 365 {
        // Průměrná délka měsíce, nejméně 1 a nejvýš 12.
        let months = (f64::from(days) / 30.44).round().clamp(1.0, 12.0) as u32;
        return format!(
            "za {months} {}",
            czech_plural(months, "měsíc", "měsíce", "měsíců")
        );
    }
    // Roky s přesností na desetiny.
    let tenths = (f64::from(days) / 365.25 * 10.0).round() as u32;
    let (whole, fraction) = (tenths / 10, tenths % 10);
    if fraction == 0 {
        format!("za {whole} {}", czech_plural(whole, "rok", "roky", "let"))
    } else {
        format!("za {whole},{fraction} roku")
    }
}

/// Český tvar podstatného jména podle počtu: 1 den, 2–4 dny, 0 a 5+ dní.
pub fn czech_plural<'a>(count: u32, one: &'a str, few: &'a str, many: &'a str) -> &'a str {
    match count {
        1 => one,
        2..=4 => few,
        _ => many,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Direction;
    use crate::test_util::TempDir;
    use time::macros::datetime;

    fn vi(key: &str) -> CardId {
        CardId {
            key: key.to_owned(),
            direction: Direction::ViToCs,
        }
    }

    fn cs(key: &str) -> CardId {
        CardId {
            key: key.to_owned(),
            direction: Direction::CsToVi,
        }
    }

    fn keys(cards: &[CardId]) -> Vec<String> {
        cards
            .iter()
            .map(|card| format!("{}:{}", card.key, card.direction.code()))
            .collect()
    }

    /// Projde sezení se zadanými hodnoceními a vrátí pořadí kartiček, jak přišly na řadu.
    fn run(session: &mut Session, ratings: &[Rating]) -> Vec<CardId> {
        let mut seen = Vec::new();
        for &rating in ratings {
            seen.push(session.current().unwrap().clone());
            session.answer(rating);
        }
        seen
    }

    #[test]
    fn new_cards_are_spread_evenly() {
        let due = vec![vi("o1"), vi("o2"), vi("o3"), vi("o4")];
        let new = vec![vi("n1"), vi("n2")];
        let cards = interleave(due, new);
        assert_eq!(
            keys(&cards),
            [
                "o1:vi-cs", "n1:vi-cs", "o2:vi-cs", "o3:vi-cs", "n2:vi-cs", "o4:vi-cs"
            ]
        );
    }

    #[test]
    fn interleave_edge_cases() {
        assert!(interleave(Vec::new(), Vec::new()).is_empty());
        assert_eq!(
            keys(&interleave(Vec::new(), vec![vi("n1"), vi("n2")])).len(),
            2
        );
        assert_eq!(keys(&interleave(vec![vi("o1")], Vec::new())), ["o1:vi-cs"]);
        // Jediná nová mezi mnoha opakováními vyjde doprostřed.
        let due = (0..100).map(|n| vi(&format!("o{n}"))).collect();
        let cards = interleave(due, vec![vi("n")]);
        assert_eq!(cards.iter().position(|c| c.key == "n"), Some(50));
        // Víc nových než opakování.
        let cards = interleave(vec![vi("o1")], vec![vi("n1"), vi("n2"), vi("n3")]);
        assert_eq!(
            keys(&cards),
            ["n1:vi-cs", "n2:vi-cs", "o1:vi-cs", "n3:vi-cs"]
        );
    }

    #[test]
    fn siblings_are_separated_when_possible() {
        let mut cards = vec![cs("a"), vi("a"), vi("b"), vi("c")];
        separate_siblings(&mut cards);
        assert_eq!(keys(&cards), ["a:cs-vi", "b:vi-cs", "a:vi-cs", "c:vi-cs"]);

        // Není co dát mezi ně, pořadí zůstane.
        let mut cards = vec![vi("x"), cs("a"), vi("a")];
        separate_siblings(&mut cards);
        assert_eq!(keys(&cards), ["x:vi-cs", "a:cs-vi", "a:vi-cs"]);

        // Víc dvojic za sebou.
        let mut cards = vec![cs("a"), vi("a"), cs("b"), vi("b"), vi("c")];
        separate_siblings(&mut cards);
        for pair in cards.windows(2) {
            assert_ne!(pair[0].key, pair[1].key, "{:?}", keys(&cards));
        }
    }

    #[test]
    fn session_without_again_goes_through_queue() {
        let queue = Queue {
            due: vec![vi("a"), vi("b")],
            new: vec![vi("c")],
        };
        let mut session = Session::new(queue);
        assert_eq!(session.remaining(), 3);
        let seen = run(&mut session, &[Rating::Good, Rating::Easy, Rating::Hard]);
        assert_eq!(keys(&seen), ["a:vi-cs", "c:vi-cs", "b:vi-cs"]);
        assert!(session.is_finished());
        assert_eq!(session.current(), None);
        assert_eq!(session.done(), 3);
        let summary = session.summary();
        assert_eq!(summary.count(Rating::Good), 1);
        assert_eq!(summary.count(Rating::Again), 0);
        assert_eq!(summary.total(), 3);
    }

    #[test]
    fn again_returns_card_five_later_until_other_rating() {
        let due = (1..=8).map(|n| vi(&format!("k{n}"))).collect();
        let mut session = Session::new(Queue {
            due,
            new: Vec::new(),
        });
        session.answer(Rating::Again); // k1 se vrátí za k6
        let next: Vec<_> = session.queue.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(next, ["k2", "k3", "k4", "k5", "k6", "k1", "k7", "k8"]);

        // Projdeme k2–k6, pak k1 znovu „Znovu“, pak „Dobré“.
        run(&mut session, &[Rating::Good; 5]);
        assert_eq!(session.current().unwrap().key, "k1");
        session.answer(Rating::Again);
        // Zbývají k7 a k8, k1 jde na konec.
        let next: Vec<_> = session.queue.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(next, ["k7", "k8", "k1"]);
        run(&mut session, &[Rating::Good; 3]);
        assert!(session.is_finished());
        assert_eq!(session.summary().count(Rating::Again), 2);
        assert_eq!(session.done(), 10);
    }

    #[test]
    fn again_on_last_card_repeats_it() {
        let mut session = Session::new(Queue {
            due: vec![vi("a")],
            new: Vec::new(),
        });
        session.answer(Rating::Again);
        assert_eq!(session.current(), Some(&vi("a")));
        session.answer(Rating::Good);
        assert!(session.is_finished());
        // Hodnocení prázdného sezení nic nedělá.
        session.answer(Rating::Good);
        assert_eq!(session.done(), 2);
    }

    #[test]
    fn again_avoids_landing_next_to_sibling() {
        let mut session = Session::new(Queue {
            due: vec![
                vi("x"),
                vi("1"),
                vi("2"),
                vi("3"),
                vi("4"),
                cs("x"),
                vi("5"),
            ],
            new: Vec::new(),
        });
        session.answer(Rating::Again);
        let next: Vec<_> = keys(&session.queue.iter().cloned().collect::<Vec<_>>());
        assert_eq!(
            next,
            [
                "1:vi-cs", "2:vi-cs", "3:vi-cs", "4:vi-cs", "x:cs-vi", "5:vi-cs", "x:vi-cs"
            ]
        );
    }

    #[test]
    fn study_rates_saves_and_reloads() {
        let dir = TempDir::new("session-study");
        let dirs = AppDirs::in_dir(dir.path());
        let mut study = Study::load(&dirs, Fsrs::default()).unwrap();
        let card = vi("cảm ơn");

        // Nová kartička: Znovu 1 den, Těžké 1, Dobré 3, Snadné 16 (viz testy FSRS).
        let now = datetime!(2026-11-02 20:00 +01:00);
        assert_eq!(study.preview(&card, now), [1, 1, 3, 16]);

        study.rate(&card, Rating::Good, now).unwrap();
        // Po půlnoci, ale před 4:00 je to stále tentýž studijní den.
        let night = datetime!(2026-11-03 01:30 +01:00);
        study.rate(&card, Rating::Good, night).unwrap();
        let state = &study.progress.get(&card).unwrap().state;
        assert_eq!(state.last_review, time::macros::date!(2026 - 11 - 02));
        assert_eq!(state.reps, 2);

        let reloaded = Study::load(&dirs, Fsrs::default()).unwrap();
        assert_eq!(reloaded.progress, study.progress);
        assert_eq!(reloaded.history.entries().len(), 2);
        assert_eq!(reloaded.history.entries()[1].time, night);
    }

    #[test]
    fn new_retention_applies_only_to_further_reviews() {
        let dir = TempDir::new("session-retention");
        let dirs = AppDirs::in_dir(dir.path());
        let mut study = Study::load(&dirs, Fsrs::default()).unwrap();
        let card = vi("cảm ơn");
        study
            .rate(&card, Rating::Easy, datetime!(2026-11-02 20:00 +01:00))
            .unwrap();
        let scheduled = study.progress.get(&card).unwrap().clone();

        // Nižší retence znamená delší intervaly, naplánované opakování se ale nemění.
        let later = datetime!(2026-11-18 20:00 +01:00);
        let before = study.preview(&card, later);
        study.set_fsrs(Fsrs::new(0.8).unwrap());
        assert_eq!(study.fsrs().desired_retention(), 0.8);
        assert_eq!(study.progress.get(&card), Some(&scheduled));
        let after = study.preview(&card, later);
        assert!(after[2] > before[2], "{before:?} → {after:?}");
    }

    #[test]
    fn study_keeps_rating_in_memory_when_saving_fails() {
        let dir = TempDir::new("session-save-fails");
        let dirs = AppDirs::in_dir(dir.path());
        let mut study = Study::load(&dirs, Fsrs::default()).unwrap();
        // Místo datové složky soubor, takže zápis do ní selže.
        dir.write("data", "není složka");
        let card = vi("a");
        let result = study.rate(&card, Rating::Good, datetime!(2026-11-02 20:00 +01:00));
        assert!(result.is_err());
        assert!(study.progress.get(&card).is_some());
        assert_eq!(study.history.entries().len(), 1);

        // Po nápravě se uloží všechno najednou.
        std::fs::remove_file(dir.join("data")).unwrap();
        study.save().unwrap();
        let reloaded = Study::load(&dirs, Fsrs::default()).unwrap();
        assert_eq!(reloaded.progress, study.progress);
        assert_eq!(reloaded.history, study.history);
    }

    #[test]
    fn interval_labels() {
        let cases = [
            (0, "za 0 dní"),
            (1, "za 1 den"),
            (2, "za 2 dny"),
            (4, "za 4 dny"),
            (5, "za 5 dní"),
            (30, "za 30 dní"),
            (31, "za 1 měsíc"),
            (61, "za 2 měsíce"),
            (152, "za 5 měsíců"),
            (364, "za 12 měsíců"),
            (365, "za 1 rok"),
            (548, "za 1,5 roku"),
            (730, "za 2 roky"),
            (1826, "za 5 let"),
            (36500, "za 99,9 roku"),
        ];
        for (days, expected) in cases {
            assert_eq!(interval_label(days), expected, "{days} dní");
        }
    }
}
