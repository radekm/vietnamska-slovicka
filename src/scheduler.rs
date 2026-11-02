//! Sestavení fronty kartiček na dnešní studijní den.
//!
//! Pravidla popisuje `PLAN.md`, oddíl „Průběh opakování“:
//! - k opakování jdou kartičky ze všech lekcí, jen v zapnutých směrech,
//! - nové kartičky jen z vybraných lekcí, do denního limitu,
//! - nové slovíčko se zavádí nejdřív jako VI → CS, CS → VI až v pozdější den.

use time::Date;

use crate::model::{CardId, Direction, Lesson};
use crate::progress::Progress;
use crate::settings::Settings;

/// Fronta kartiček na dnešek.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Queue {
    /// Kartičky k opakování, od nejvíc prošlých.
    pub due: Vec<CardId>,
    /// Nové kartičky v pořadí lekcí a slovíček.
    pub new: Vec<CardId>,
}

impl Queue {
    /// Celkový počet kartiček ve frontě.
    pub fn len(&self) -> usize {
        self.due.len() + self.new.len()
    }

    /// Je fronta prázdná?
    pub fn is_empty(&self) -> bool {
        self.due.is_empty() && self.new.is_empty()
    }
}

/// Sestaví frontu na studijní den `today`.
///
/// Lekce musí být seřazené podle čísla (tak je vrací `lessons::load_dir`). Kartičky
/// s postupem, které v lekcích nejsou (osiřelé), se do fronty nedostanou.
pub fn build_queue(
    lessons: &[Lesson],
    settings: &Settings,
    progress: &Progress,
    today: Date,
) -> Queue {
    Queue {
        due: due_cards(lessons, settings, progress, today),
        new: new_cards(lessons, settings, progress, today),
    }
}

/// Kartičky k opakování ze všech lekcí, seřazené podle data splatnosti
/// a při shodě podle pořadí lekcí, slovíček a směrů.
fn due_cards(
    lessons: &[Lesson],
    settings: &Settings,
    progress: &Progress,
    today: Date,
) -> Vec<CardId> {
    let mut due: Vec<(Date, usize, CardId)> = Vec::new();
    let cards = lessons
        .iter()
        .flat_map(|lesson| &lesson.words)
        .flat_map(|word| word.card_ids());
    for (order, id) in cards.enumerate() {
        if !settings.direction_enabled(id.direction) {
            continue;
        }
        if let Some(card) = progress.get(&id)
            && card.state.due <= today
        {
            due.push((card.state.due, order, id));
        }
    }
    due.sort_by_key(|(due_date, order, _)| (*due_date, *order));
    due.into_iter().map(|(_, _, id)| id).collect()
}

/// Nové kartičky z vybraných lekcí do zbývajícího denního limitu.
fn new_cards(
    lessons: &[Lesson],
    settings: &Settings,
    progress: &Progress,
    today: Date,
) -> Vec<CardId> {
    // Kolik nových kartiček už dnes začalo, se od limitu odečte.
    let started_today = progress.first_reviewed_on(today);
    let limit = usize::try_from(settings.new_cards_per_day)
        .unwrap_or(usize::MAX)
        .saturating_sub(started_today);
    let vi_cs_enabled = settings.direction_enabled(Direction::ViToCs);
    let cs_vi_enabled = settings.direction_enabled(Direction::CsToVi);

    let mut new = Vec::new();
    let words = lessons
        .iter()
        .filter(|lesson| settings.lesson_selected(lesson.number))
        .flat_map(|lesson| &lesson.words);
    for word in words {
        let [cs_vi, vi_cs] = word.card_ids();
        let vi_cs_progress = progress.get(&vi_cs);
        if vi_cs_enabled && vi_cs_progress.is_none() {
            new.push(vi_cs);
        }
        if cs_vi_enabled && progress.get(&cs_vi).is_none() {
            // Při obou směrech čeká CS → VI, až se VI → CS poprvé opakuje v dřívější den.
            let ready =
                !vi_cs_enabled || vi_cs_progress.is_some_and(|card| card.first_review < today);
            if ready {
                new.push(cs_vi);
            }
        }
        if new.len() >= limit {
            break;
        }
    }
    new.truncate(limit);
    new
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsrs::{Fsrs, Rating};
    use crate::model::Word;
    use time::Duration;
    use time::macros::date;

    const TODAY: Date = date!(2026 - 11 - 10);

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

    fn lesson(number: u32, words: &[&str]) -> Lesson {
        Lesson {
            number,
            file_name: format!("lekce{number:02}.toml"),
            title: format!("Lekce {number}"),
            description: None,
            words: words.iter().map(|vi| word(vi)).collect(),
        }
    }

    fn id(key: &str, direction: Direction) -> CardId {
        CardId {
            key: key.to_owned(),
            direction,
        }
    }

    fn vi_cs(key: &str) -> CardId {
        id(key, Direction::ViToCs)
    }

    fn cs_vi(key: &str) -> CardId {
        id(key, Direction::CsToVi)
    }

    /// Nastavení s vybranými lekcemi a limitem.
    fn settings(selected: &[u32], limit: u32) -> Settings {
        Settings {
            selected_lessons: selected.to_vec(),
            new_cards_per_day: limit,
            ..Settings::default()
        }
    }

    /// Zaznamená kartičku poprvé opakovanou v den `first` s příštím opakováním v den `due`.
    fn learned(progress: &mut Progress, card: CardId, first: Date, due: Date) {
        let mut state = Fsrs::default().review(None, Rating::Good, first);
        state.due = due;
        progress.record(card, state);
    }

    fn days(n: i64) -> Duration {
        Duration::days(n)
    }

    #[test]
    fn fresh_start_introduces_vi_cs_only() {
        let lessons = [lesson(1, &["a", "b", "c"])];
        let queue = build_queue(&lessons, &settings(&[1], 10), &Progress::default(), TODAY);
        assert!(queue.due.is_empty());
        assert_eq!(queue.new, [vi_cs("a"), vi_cs("b"), vi_cs("c")]);
        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn no_selected_lessons_means_no_new_cards() {
        let lessons = [lesson(1, &["a"])];
        let queue = build_queue(&lessons, &settings(&[], 10), &Progress::default(), TODAY);
        assert!(queue.is_empty());
    }

    #[test]
    fn new_cards_only_from_selected_lessons_in_order() {
        let lessons = [lesson(1, &["a"]), lesson(2, &["b"]), lesson(3, &["c"])];
        let queue = build_queue(
            &lessons,
            &settings(&[1, 3], 10),
            &Progress::default(),
            TODAY,
        );
        assert_eq!(queue.new, [vi_cs("a"), vi_cs("c")]);
    }

    #[test]
    fn cs_vi_waits_until_a_later_day() {
        let lessons = [lesson(1, &["a", "b"])];
        let mut progress = Progress::default();
        // „a“ začalo VI → CS včera, „b“ dnes.
        learned(&mut progress, vi_cs("a"), TODAY - days(1), TODAY + days(5));
        learned(&mut progress, vi_cs("b"), TODAY, TODAY + days(3));
        let queue = build_queue(&lessons, &settings(&[1], 10), &progress, TODAY);
        assert_eq!(queue.new, [cs_vi("a")]);
    }

    #[test]
    fn single_direction_does_not_wait() {
        let lessons = [lesson(1, &["a", "b"])];
        let only_cs_vi = Settings {
            directions: vec![Direction::CsToVi],
            ..settings(&[1], 10)
        };
        let queue = build_queue(&lessons, &only_cs_vi, &Progress::default(), TODAY);
        assert_eq!(queue.new, [cs_vi("a"), cs_vi("b")]);

        let only_vi_cs = Settings {
            directions: vec![Direction::ViToCs],
            ..settings(&[1], 10)
        };
        let queue = build_queue(&lessons, &only_vi_cs, &Progress::default(), TODAY);
        assert_eq!(queue.new, [vi_cs("a"), vi_cs("b")]);
    }

    #[test]
    fn limit_counts_cards_and_subtracts_started_today() {
        let lessons = [lesson(1, &["a", "b", "c", "d", "e"])];
        let queue = build_queue(&lessons, &settings(&[1], 2), &Progress::default(), TODAY);
        assert_eq!(queue.new, [vi_cs("a"), vi_cs("b")]);

        let mut progress = Progress::default();
        learned(&mut progress, vi_cs("a"), TODAY, TODAY + days(3));
        learned(&mut progress, vi_cs("b"), TODAY, TODAY + days(3));
        let queue = build_queue(&lessons, &settings(&[1], 3), &progress, TODAY);
        assert_eq!(queue.new, [vi_cs("c")], "dvě z limitu tří už dnes začaly");

        let queue = build_queue(&lessons, &settings(&[1], 2), &progress, TODAY);
        assert!(queue.new.is_empty(), "limit je vyčerpaný");

        let queue = build_queue(&lessons, &settings(&[1], 0), &Progress::default(), TODAY);
        assert!(queue.new.is_empty());
    }

    #[test]
    fn limit_cuts_between_directions_of_one_word() {
        let lessons = [lesson(1, &["a", "b"])];
        let mut progress = Progress::default();
        learned(&mut progress, vi_cs("b"), TODAY - days(2), TODAY + days(5));
        // „a“ dá VI → CS, „b“ dá CS → VI, limit 1 pustí jen první.
        let queue = build_queue(&lessons, &settings(&[1], 1), &progress, TODAY);
        assert_eq!(queue.new, [vi_cs("a")]);
    }

    #[test]
    fn due_cards_from_all_lessons_most_overdue_first() {
        let lessons = [lesson(1, &["a", "b"]), lesson(2, &["c"])];
        let mut progress = Progress::default();
        let start = TODAY - days(30);
        learned(&mut progress, vi_cs("a"), start, TODAY);
        learned(&mut progress, cs_vi("a"), start, TODAY);
        learned(&mut progress, vi_cs("b"), start, TODAY - days(3));
        learned(&mut progress, vi_cs("c"), start, TODAY - days(1));
        learned(&mut progress, cs_vi("c"), start, TODAY + days(1)); // ještě není na řadě
        // Lekce 2 není vybraná, přesto se z ní opakuje.
        let queue = build_queue(&lessons, &settings(&[1], 0), &progress, TODAY);
        assert_eq!(
            queue.due,
            [vi_cs("b"), vi_cs("c"), cs_vi("a"), vi_cs("a")],
            "prošlé dřív, při shodě pořadí slovíček a směrů"
        );
    }

    #[test]
    fn due_cards_respect_enabled_directions() {
        let lessons = [lesson(1, &["a"])];
        let mut progress = Progress::default();
        let start = TODAY - days(10);
        learned(&mut progress, vi_cs("a"), start, TODAY);
        learned(&mut progress, cs_vi("a"), start, TODAY);
        let only_vi_cs = Settings {
            directions: vec![Direction::ViToCs],
            ..settings(&[1], 10)
        };
        let queue = build_queue(&lessons, &only_vi_cs, &progress, TODAY);
        assert_eq!(queue.due, [vi_cs("a")]);
        assert!(queue.new.is_empty());
    }

    #[test]
    fn orphaned_progress_is_ignored() {
        let lessons = [lesson(1, &["a"])];
        let mut progress = Progress::default();
        learned(
            &mut progress,
            vi_cs("smazané"),
            TODAY - days(5),
            TODAY - days(1),
        );
        let queue = build_queue(&lessons, &settings(&[], 10), &progress, TODAY);
        assert!(queue.is_empty());
    }

    #[test]
    fn homonyms_use_their_own_keys() {
        let mut first = word("đường");
        first.key_override = Some("đường (cesta)".to_owned());
        let mut second = word("đường");
        second.key_override = Some("đường (cukr)".to_owned());
        let lessons = [Lesson {
            words: vec![first, second],
            ..lesson(1, &[])
        }];
        let queue = build_queue(&lessons, &settings(&[1], 10), &Progress::default(), TODAY);
        assert_eq!(queue.new, [vi_cs("đường (cesta)"), vi_cs("đường (cukr)")]);
    }
}
