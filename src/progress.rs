//! Postup učení (`data/postup.toml`) a historie hodnocení (`data/historie.toml`).

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::Path;

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use toml::value::Datetime;

use crate::fsrs::{CardState, MemoryState, Rating};
use crate::model::{CardId, Direction};
use crate::storage;

/// Postup učení jedné kartičky.
#[derive(Debug, Clone, PartialEq)]
pub struct CardProgress {
    /// Stav podle FSRS.
    pub state: CardState,
    /// Studijní den prvního opakování (kdy kartička přestala být nová).
    pub first_review: Date,
}

/// Postup učení všech kartiček, které už byly aspoň jednou opakovány.
///
/// Kartičky, které tu nejsou, jsou nové. Záznamy kartiček, které v lekcích už nejsou,
/// se zachovávají (viz `PLAN.md`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Progress {
    cards: BTreeMap<CardId, CardProgress>,
}

/// Podoba souboru `postup.toml`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgressFile {
    #[serde(rename = "verze")]
    version: u32,
    #[serde(rename = "karticky", default)]
    cards: Vec<CardRecord>,
}

/// Jedna kartička v souboru `postup.toml`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CardRecord {
    #[serde(rename = "klic")]
    key: String,
    #[serde(rename = "smer")]
    direction: String,
    #[serde(rename = "stabilita")]
    stability: f64,
    #[serde(rename = "obtiznost")]
    difficulty: f64,
    #[serde(rename = "prvni_opakovani")]
    first_review: Datetime,
    #[serde(rename = "posledni_opakovani")]
    last_review: Datetime,
    #[serde(rename = "pristi_opakovani")]
    due: Datetime,
    #[serde(rename = "opakovani")]
    reps: u32,
    #[serde(rename = "zapomenuti")]
    lapses: u32,
}

impl Progress {
    /// Načte postup ze souboru. Chybějící soubor znamená prázdný postup.
    ///
    /// Poškozený soubor je chyba, aby ho další uložení nepřepsalo prázdným postupem.
    pub fn load(path: &Path) -> Result<Self> {
        match storage::read_optional(path)? {
            None => Ok(Self::default()),
            Some(text) => Self::from_toml_str(&text)
                .with_context(|| format!("chyba v souboru s postupem {}", path.display())),
        }
    }

    /// Bezpečně uloží postup do souboru.
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = self.to_toml_string()?;
        storage::write_atomic(path, &text)
            .with_context(|| format!("nelze uložit postup do {}", path.display()))
    }

    /// Postup z textu ve formátu TOML.
    pub fn from_toml_str(text: &str) -> Result<Self> {
        let file: ProgressFile = toml::from_str(text)?;
        storage::check_version(file.version)?;
        let mut cards = BTreeMap::new();
        for (index, record) in file.cards.into_iter().enumerate() {
            let (id, progress) = card_from_record(record)
                .with_context(|| format!("kartička č. {} v seznamu", index + 1))?;
            match cards.entry(id) {
                Entry::Vacant(entry) => {
                    entry.insert(progress);
                }
                Entry::Occupied(entry) => bail!(
                    "kartička č. {}: kartička \"{}\" ({}) je v souboru víckrát",
                    index + 1,
                    entry.key().key,
                    entry.key().direction.code()
                ),
            }
        }
        Ok(Self { cards })
    }

    /// Postup jako text ve formátu TOML, kartičky seřazené podle klíče a směru.
    pub fn to_toml_string(&self) -> Result<String> {
        let file = ProgressFile {
            version: storage::FORMAT_VERSION,
            cards: self
                .cards
                .iter()
                .map(|(id, progress)| CardRecord {
                    key: id.key.clone(),
                    direction: id.direction.code().to_owned(),
                    stability: progress.state.memory.stability,
                    difficulty: progress.state.memory.difficulty,
                    first_review: storage::date_to_toml(progress.first_review),
                    last_review: storage::date_to_toml(progress.state.last_review),
                    due: storage::date_to_toml(progress.state.due),
                    reps: progress.state.reps,
                    lapses: progress.state.lapses,
                })
                .collect(),
        };
        toml::to_string(&file).context("nelze převést postup do TOML")
    }

    /// Postup kartičky, nebo `None`, pokud je nová.
    pub fn get(&self, id: &CardId) -> Option<&CardProgress> {
        self.cards.get(id)
    }

    /// Zaznamená nový stav kartičky po hodnocení.
    ///
    /// U nové kartičky se jako první opakování zapíše den jejího posledního opakování.
    pub fn record(&mut self, id: CardId, state: CardState) {
        match self.cards.entry(id) {
            Entry::Occupied(mut entry) => entry.get_mut().state = state,
            Entry::Vacant(entry) => {
                let first_review = state.last_review;
                entry.insert(CardProgress {
                    state,
                    first_review,
                });
            }
        }
    }

    /// Všechny kartičky s postupem, seřazené podle klíče a směru.
    pub fn iter(&self) -> impl Iterator<Item = (&CardId, &CardProgress)> {
        self.cards.iter()
    }

    /// Počet kartiček s postupem.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Nemá ještě žádná kartička postup?
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Kolik kartiček poprvé opakovaných v daný studijní den (pro denní limit nových).
    pub fn first_reviewed_on(&self, day: Date) -> usize {
        self.cards
            .values()
            .filter(|progress| progress.first_review == day)
            .count()
    }
}

/// Převede záznam ze souboru na kartičku a zkontroluje, že hodnoty dávají smysl.
fn card_from_record(record: CardRecord) -> Result<(CardId, CardProgress)> {
    ensure!(!record.key.is_empty(), "klíč nesmí být prázdný");
    let direction = parse_direction(&record.direction)?;
    let what = |field: &str| format!("pole {field} kartičky \"{}\"", record.key);
    ensure!(
        record.stability.is_finite() && record.stability > 0.0,
        "{}: stabilita {} musí být kladné číslo",
        what("stabilita"),
        record.stability
    );
    ensure!(
        (1.0..=10.0).contains(&record.difficulty),
        "{}: obtížnost {} musí být v rozsahu 1 až 10",
        what("obtiznost"),
        record.difficulty
    );
    let first_review =
        storage::date_from_toml(&record.first_review).with_context(|| what("prvni_opakovani"))?;
    let last_review =
        storage::date_from_toml(&record.last_review).with_context(|| what("posledni_opakovani"))?;
    let due = storage::date_from_toml(&record.due).with_context(|| what("pristi_opakovani"))?;
    ensure!(
        first_review <= last_review && last_review <= due,
        "kartička \"{}\": data musí jít po sobě (první {first_review} ≤ poslední \
         {last_review} ≤ příští {due})",
        record.key
    );
    ensure!(
        record.reps >= 1,
        "kartička \"{}\": počet opakování musí být aspoň 1",
        record.key
    );

    let id = CardId {
        key: record.key,
        direction,
    };
    let progress = CardProgress {
        state: CardState {
            memory: MemoryState {
                stability: record.stability,
                difficulty: record.difficulty,
            },
            last_review,
            due,
            reps: record.reps,
            lapses: record.lapses,
        },
        first_review,
    };
    Ok((id, progress))
}

fn parse_direction(code: &str) -> Result<Direction> {
    match Direction::from_code(code) {
        Some(direction) => Ok(direction),
        None => bail!("neznámý směr \"{code}\" (povolené jsou \"cs-vi\" a \"vi-cs\")"),
    }
}

/// Jedno hodnocení v historii.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewEntry {
    /// Kdy hodnocení proběhlo (místní čas s časovým posunem).
    pub time: OffsetDateTime,
    /// Hodnocená kartička.
    pub card: CardId,
    /// Hodnocení.
    pub rating: Rating,
}

/// Historie všech hodnocení v pořadí, jak proběhla.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct History {
    entries: Vec<ReviewEntry>,
}

/// Podoba souboru `historie.toml`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoryFile {
    #[serde(rename = "verze")]
    version: u32,
    #[serde(rename = "hodnoceni", default)]
    entries: Vec<ReviewRecord>,
}

/// Jedno hodnocení v souboru `historie.toml`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewRecord {
    #[serde(rename = "cas")]
    time: Datetime,
    #[serde(rename = "klic")]
    key: String,
    #[serde(rename = "smer")]
    direction: String,
    #[serde(rename = "hodnoceni")]
    rating: u8,
}

impl History {
    /// Načte historii ze souboru. Chybějící soubor znamená prázdnou historii.
    ///
    /// Poškozený soubor je chyba, aby ho další uložení nepřepsalo.
    pub fn load(path: &Path) -> Result<Self> {
        match storage::read_optional(path)? {
            None => Ok(Self::default()),
            Some(text) => Self::from_toml_str(&text)
                .with_context(|| format!("chyba v souboru s historií {}", path.display())),
        }
    }

    /// Bezpečně uloží celou historii do souboru.
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = self.to_toml_string()?;
        storage::write_atomic(path, &text)
            .with_context(|| format!("nelze uložit historii do {}", path.display()))
    }

    /// Historie z textu ve formátu TOML.
    pub fn from_toml_str(text: &str) -> Result<Self> {
        let file: HistoryFile = toml::from_str(text)?;
        storage::check_version(file.version)?;
        let entries = file
            .entries
            .into_iter()
            .enumerate()
            .map(|(index, record)| {
                entry_from_record(record)
                    .with_context(|| format!("hodnocení č. {} v seznamu", index + 1))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { entries })
    }

    /// Historie jako text ve formátu TOML.
    pub fn to_toml_string(&self) -> Result<String> {
        let file = HistoryFile {
            version: storage::FORMAT_VERSION,
            entries: self
                .entries
                .iter()
                .map(|entry| ReviewRecord {
                    time: storage::datetime_to_toml(entry.time),
                    key: entry.card.key.clone(),
                    direction: entry.card.direction.code().to_owned(),
                    rating: entry.rating.number(),
                })
                .collect(),
        };
        toml::to_string(&file).context("nelze převést historii do TOML")
    }

    /// Přidá hodnocení na konec historie.
    pub fn push(&mut self, entry: ReviewEntry) {
        self.entries.push(entry);
    }

    /// Všechna hodnocení v pořadí, jak proběhla.
    pub fn entries(&self) -> &[ReviewEntry] {
        &self.entries
    }
}

fn entry_from_record(record: ReviewRecord) -> Result<ReviewEntry> {
    ensure!(!record.key.is_empty(), "klíč nesmí být prázdný");
    let time = storage::datetime_from_toml(&record.time).context("pole cas")?;
    let direction = parse_direction(&record.direction)?;
    let Some(rating) = Rating::from_number(record.rating) else {
        bail!("hodnocení {} musí být 1 až 4", record.rating);
    };
    Ok(ReviewEntry {
        time,
        card: CardId {
            key: record.key,
            direction,
        },
        rating,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsrs::Fsrs;
    use crate::test_util::TempDir;
    use time::macros::{date, datetime};

    fn card(key: &str, direction: Direction) -> CardId {
        CardId {
            key: key.to_owned(),
            direction,
        }
    }

    const SAMPLE_PROGRESS: &str = r#"verze = 1

[[karticky]]
klic = "cảm ơn"
smer = "vi-cs"
stabilita = 10.73892584613159
obtiznost = 5.272967931287446
prvni_opakovani = 2026-11-02
posledni_opakovani = 2026-11-05
pristi_opakovani = 2026-11-16
opakovani = 2
zapomenuti = 0
"#;

    #[test]
    fn progress_file_format() {
        let progress = Progress::from_toml_str(SAMPLE_PROGRESS).unwrap();
        let state = &progress
            .get(&card("cảm ơn", Direction::ViToCs))
            .unwrap()
            .state;
        assert_eq!(state.memory.stability, 10.73892584613159);
        assert_eq!(state.due, date!(2026 - 11 - 16));
        // Zápis dá přesně stejný text, čísla se neztrácejí ani nezaokrouhlují.
        assert_eq!(progress.to_toml_string().unwrap(), SAMPLE_PROGRESS);
    }

    #[test]
    fn missing_progress_file_is_empty() {
        let dir = TempDir::new("progress-missing");
        let progress = Progress::load(&dir.join("postup.toml")).unwrap();
        assert!(progress.is_empty());
    }

    #[test]
    fn record_save_and_load_round_trip() {
        let dir = TempDir::new("progress-round-trip");
        let path = dir.join("data/postup.toml");
        let fsrs = Fsrs::default();
        let day1 = date!(2026 - 11 - 02);
        let day2 = date!(2026 - 11 - 05);
        let id = card("xin chào", Direction::CsToVi);

        let mut progress = Progress::default();
        progress.record(id.clone(), fsrs.review(None, Rating::Good, day1));
        let state = fsrs.review(Some(&progress.get(&id).unwrap().state), Rating::Hard, day2);
        progress.record(id.clone(), state.clone());
        progress.record(
            card("đường (cukr)", Direction::ViToCs),
            fsrs.review(None, Rating::Again, day2),
        );

        let saved = progress.get(&id).unwrap();
        assert_eq!(saved.first_review, day1, "první opakování se nepřepíše");
        assert_eq!(saved.state, state);

        progress.save(&path).unwrap();
        let loaded = Progress::load(&path).unwrap();
        assert_eq!(loaded, progress);
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.first_reviewed_on(day1), 1);
        assert_eq!(loaded.first_reviewed_on(day2), 1);
    }

    #[test]
    fn invalid_progress_is_rejected() {
        let replace = |from: &str, to: &str| SAMPLE_PROGRESS.replace(from, to);
        let cases = [
            replace("verze = 1", "verze = 2"),
            replace("smer = \"vi-cs\"", "smer = \"vi\""),
            replace("stabilita = 10.73892584613159", "stabilita = 0.0"),
            replace("stabilita = 10.73892584613159", "stabilita = nan"),
            replace("obtiznost = 5.272967931287446", "obtiznost = 11.0"),
            replace(
                "pristi_opakovani = 2026-11-16",
                "pristi_opakovani = 2026-11-01",
            ),
            replace(
                "prvni_opakovani = 2026-11-02",
                "prvni_opakovani = 2026-11-02T10:00:00Z",
            ),
            replace("opakovani = 2", "opakovani = 0"),
            replace("zapomenuti = 0", "zapomenuti = 0\npoznamka = \"x\""),
            replace("klic = \"cảm ơn\"", "klic = \"\""),
            // Stejná kartička dvakrát.
            format!(
                "{SAMPLE_PROGRESS}{}",
                SAMPLE_PROGRESS.replace("verze = 1\n", "")
            ),
        ];
        for text in cases {
            assert!(
                Progress::from_toml_str(&text).is_err(),
                "mělo selhat:\n{text}"
            );
        }
    }

    #[test]
    fn same_key_in_both_directions_is_fine() {
        let other = SAMPLE_PROGRESS
            .replace("verze = 1\n", "")
            .replace("vi-cs", "cs-vi");
        let progress = Progress::from_toml_str(&format!("{SAMPLE_PROGRESS}{other}")).unwrap();
        assert_eq!(progress.len(), 2);
    }

    #[test]
    fn corrupted_progress_file_is_error_and_not_overwritten() {
        let dir = TempDir::new("progress-corrupted");
        dir.write("postup.toml", "verze = 1\n[[karticky]\n");
        let err = Progress::load(&dir.join("postup.toml")).unwrap_err();
        assert!(format!("{err:#}").contains("postup.toml"), "{err:#}");
        assert_eq!(
            std::fs::read_to_string(dir.join("postup.toml")).unwrap(),
            "verze = 1\n[[karticky]\n"
        );
    }

    #[test]
    fn history_round_trip() {
        let dir = TempDir::new("history-round-trip");
        let path = dir.join("data/historie.toml");
        assert!(History::load(&path).unwrap().entries().is_empty());

        let mut history = History::default();
        history.push(ReviewEntry {
            time: datetime!(2026-11-02 16:53:12 +01:00),
            card: card("cảm ơn", Direction::ViToCs),
            rating: Rating::Good,
        });
        history.push(ReviewEntry {
            time: datetime!(2026-11-03 01:10:00 +01:00),
            card: card("cảm ơn", Direction::CsToVi),
            rating: Rating::Again,
        });
        history.save(&path).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let expected = "verze = 1\n\n[[hodnoceni]]\ncas = 2026-11-02T16:53:12+01:00\n\
                        klic = \"cảm ơn\"\nsmer = \"vi-cs\"\nhodnoceni = 3\n\n\
                        [[hodnoceni]]\ncas = 2026-11-03T01:10:00+01:00\n\
                        klic = \"cảm ơn\"\nsmer = \"cs-vi\"\nhodnoceni = 1\n";
        assert_eq!(text, expected);
        assert_eq!(History::load(&path).unwrap(), history);
    }

    #[test]
    fn invalid_history_is_rejected() {
        let entry = "[[hodnoceni]]\ncas = 2026-11-02T16:53:12+01:00\nklic = \"a\"\n\
                     smer = \"vi-cs\"\nhodnoceni = 3\n";
        assert!(History::from_toml_str(&format!("verze = 1\n{entry}")).is_ok());
        let cases = [
            format!("verze = 2\n{entry}"),
            format!(
                "verze = 1\n{}",
                entry.replace("hodnoceni = 3", "hodnoceni = 5")
            ),
            format!("verze = 1\n{}", entry.replace("+01:00", "")),
            format!("verze = 1\n{}", entry.replace("vi-cs", "x")),
            format!(
                "verze = 1\n{}",
                entry.replace("klic = \"a\"", "klic = \"\"")
            ),
            format!("verze = 1\n{entry}trvani = 5\n"),
        ];
        for text in cases {
            assert!(
                History::from_toml_str(&text).is_err(),
                "mělo selhat:\n{text}"
            );
        }
    }
}
