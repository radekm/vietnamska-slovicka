//! Procházení lekcí: hledání slovíček bez ohledu na diakritiku a stav kartiček.

use time::Date;

use crate::model::{CardId, Word};
use crate::progress::Progress;

/// Písmena s diakritikou a jejich základ. Pokrývá vietnamskou abecedu se všemi tóny
/// a češtinu. Velká písmena se před převodem zmenší, proto tu jsou jen malá.
const FOLDS: &[(char, &str)] = &[
    ('a', "àáảãạăằắẳẵặâầấẩẫậ"),
    ('e', "èéẻẽẹêềếểễệě"),
    ('i', "ìíỉĩị"),
    ('o', "òóỏõọôồốổỗộơờớởỡợ"),
    ('u', "ùúủũụưừứửữựů"),
    ('y', "ỳýỷỹỵ"),
    ('d', "đď"),
    ('c', "č"),
    ('n', "ň"),
    ('r', "ř"),
    ('s', "š"),
    ('t', "ť"),
    ('z', "ž"),
];

/// Je znak kombinující diakritické znaménko (rozložený zápis, např. „a“ + háček zvlášť)?
fn is_combining_mark(c: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&c)
}

/// Základ písmene bez diakritiky (už zmenšeného).
fn fold_char(c: char) -> char {
    FOLDS
        .iter()
        .find(|(_, variants)| variants.contains(c))
        .map_or(c, |&(base, _)| base)
}

/// Text pro porovnávání při hledání: malá písmena, bez diakritiky a s jednou mezerou
/// mezi slovy. Např. „Cảm  Ơn“ → „cam on“, „Děkuji“ → „dekuji“.
pub fn fold(text: &str) -> String {
    let folded: String = text
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|&c| !is_combining_mark(c))
        .map(fold_char)
        .collect();
    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Hledaný výraz připravený pro porovnávání se slovíčky.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    folded: String,
}

impl Query {
    /// Připraví hledaný výraz tak, jak ho uživatel napsal.
    pub fn new(text: &str) -> Self {
        Self { folded: fold(text) }
    }

    /// Je výraz prázdný (nic se nehledá)?
    pub fn is_empty(&self) -> bool {
        self.folded.is_empty()
    }

    /// Odpovídá slovíčko výrazu? Prohledává vietnamštinu, češtinu, český popis
    /// výslovnosti, poznámku a štítky. Prázdnému výrazu odpovídá každé slovíčko.
    pub fn matches(&self, word: &Word) -> bool {
        if self.is_empty() {
            return true;
        }
        let fields = [&word.vi, &word.cs, &word.pronunciation_cs]
            .into_iter()
            .chain(&word.note)
            .chain(&word.tags);
        fields
            .into_iter()
            .any(|field| fold(field).contains(&self.folded))
    }
}

/// Stav kartičky z pohledu dneška.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardStatus {
    /// Kartička se ještě neopakovala.
    New,
    /// Kartička je dnes na řadě (nebo už prošlá).
    Due,
    /// Kartička přijde na řadu v uvedený den.
    Scheduled(Date),
}

impl CardStatus {
    /// Stav kartičky podle postupu na studijní den `today`.
    pub fn of(progress: &Progress, card: &CardId, today: Date) -> Self {
        match progress.get(card) {
            None => Self::New,
            Some(card) if card.state.due <= today => Self::Due,
            Some(card) => Self::Scheduled(card.state.due),
        }
    }

    /// Popisek pro tabulku: „nová“, „dnes“ nebo datum, např. „16. 11. 2026“.
    pub fn label(self) -> String {
        match self {
            Self::New => "nová".to_owned(),
            Self::Due => "dnes".to_owned(),
            Self::Scheduled(date) => {
                format!(
                    "{}. {}. {}",
                    date.day(),
                    u8::from(date.month()),
                    date.year()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsrs::{CardState, MemoryState};
    use crate::model::Direction;
    use time::macros::date;

    fn word(vi: &str, cs: &str) -> Word {
        Word {
            vi: vi.to_owned(),
            key_override: None,
            cs: cs.to_owned(),
            pronunciation_cs: "kam (klesá a stoupá) ən (rovně)".to_owned(),
            ipa: "kaːm˧˩˧ ʔəːn˧˧".to_owned(),
            note: Some("běžné poděkování".to_owned()),
            tags: vec!["zdvořilost".to_owned()],
        }
    }

    #[test]
    fn fold_removes_vietnamese_and_czech_diacritics() {
        assert_eq!(fold("cảm ơn"), "cam on");
        assert_eq!(fold("Đường"), "duong");
        assert_eq!(fold("Người Việt"), "nguoi viet");
        assert_eq!(
            fold("Příliš žluťoučký kůň úpěl ďábelské ódy"),
            "prilis zlutoucky kun upel dabelske ody"
        );
        assert_eq!(fold("  xin   chào  "), "xin chao");
    }

    #[test]
    fn fold_covers_all_vietnamese_vowels_with_tones() {
        let vowels = "aàáảãạăằắẳẵặâầấẩẫậeèéẻẽẹêềếểễệiìíỉĩịoòóỏõọôồốổỗộơờớởỡợuùúủũụưừứửữựyỳýỷỹỵ";
        let folded = fold(vowels);
        assert!(folded.chars().all(|c| "aeiouy".contains(c)), "{folded}");
        // Velká písmena dopadnou stejně.
        assert_eq!(fold(&vowels.to_uppercase()), folded);
    }

    #[test]
    fn fold_handles_decomposed_text() {
        // „ả“ zapsané jako „a“ + kombinující háček nahoře (U+0309).
        assert_eq!(fold("ca\u{309}m"), "cam");
    }

    #[test]
    fn query_matches_any_text_field() {
        let word = word("cảm ơn", "děkuji");
        for text in [
            "cam on",
            "CẢM",
            "dekuji",
            "Děk",
            "klesa a stoupa",
            "podekovani",
            "zdvoril",
            "",
        ] {
            assert!(Query::new(text).matches(&word), "mělo najít: {text}");
        }
        for text in ["xin chao", "camon", "kaːm"] {
            assert!(!Query::new(text).matches(&word), "nemělo najít: {text}");
        }
    }

    #[test]
    fn card_status_and_label() {
        let today = date!(2026 - 11 - 02);
        let card = CardId {
            key: "cảm ơn".to_owned(),
            direction: Direction::ViToCs,
        };
        let mut progress = Progress::default();
        assert_eq!(CardStatus::of(&progress, &card, today), CardStatus::New);
        assert_eq!(CardStatus::New.label(), "nová");

        let state = |due| CardState {
            memory: MemoryState {
                stability: 3.0,
                difficulty: 5.0,
            },
            last_review: date!(2026 - 10 - 30),
            due,
            reps: 1,
            lapses: 0,
        };
        progress.record(card.clone(), state(date!(2026 - 11 - 01)));
        assert_eq!(CardStatus::of(&progress, &card, today), CardStatus::Due);
        assert_eq!(CardStatus::Due.label(), "dnes");

        progress.record(card.clone(), state(date!(2026 - 11 - 16)));
        let status = CardStatus::of(&progress, &card, today);
        assert_eq!(status, CardStatus::Scheduled(date!(2026 - 11 - 16)));
        assert_eq!(status.label(), "16. 11. 2026");
    }
}
