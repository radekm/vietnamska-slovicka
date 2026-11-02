//! Datový model: slovíčko, lekce, směr procvičování a identifikátor kartičky.

use serde::Deserialize;

/// Jedno slovíčko nebo fráze tak, jak je zapsané v souboru s lekcí.
///
/// Názvy polí v TOML jsou česky, aby se lekce daly snadno upravovat ručně.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Word {
    /// Vietnamsky, s diakritikou a tóny.
    pub vi: String,

    /// Náhradní klíč pro homonyma (stejně psaná slova s jiným významem).
    ///
    /// Bez něj by homonyma sdílela jeden postup učení. U ostatních slovíček se nevyplňuje.
    #[serde(rename = "klic", default)]
    pub key_override: Option<String>,

    /// Česky.
    pub cs: String,

    /// Výslovnost popsaná česky, za každou slabikou slovní popis tónu.
    #[serde(rename = "vyslovnost_cs")]
    pub pronunciation_cs: String,

    /// Výslovnost v IPA (hanojská).
    pub ipa: String,

    /// Volitelná poznámka.
    #[serde(rename = "poznamka", default)]
    pub note: Option<String>,

    /// Volitelné štítky.
    #[serde(rename = "stitky", default)]
    pub tags: Vec<String>,
}

impl Word {
    /// Klíč slovíčka, přes který se k němu váže postup učení.
    ///
    /// Je to `klic`, pokud je vyplněný, jinak vietnamské slovo.
    pub fn key(&self) -> &str {
        self.key_override.as_deref().unwrap_or(&self.vi)
    }

    /// Identifikátory obou kartiček, které ze slovíčka vzniknou.
    pub fn card_ids(&self) -> [CardId; 2] {
        Direction::ALL.map(|direction| CardId {
            key: self.key().to_owned(),
            direction,
        })
    }
}

/// Směr procvičování.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Direction {
    /// Z češtiny do vietnamštiny.
    CsToVi,
    /// Z vietnamštiny do češtiny.
    ViToCs,
}

impl Direction {
    /// Oba směry v pevném pořadí.
    pub const ALL: [Direction; 2] = [Direction::CsToVi, Direction::ViToCs];

    /// Krátký popisek pro uživatelské rozhraní.
    pub fn label(self) -> &'static str {
        match self {
            Direction::CsToVi => "CS → VI",
            Direction::ViToCs => "VI → CS",
        }
    }

    /// Kód směru v datových souborech.
    pub fn code(self) -> &'static str {
        match self {
            Direction::CsToVi => "cs-vi",
            Direction::ViToCs => "vi-cs",
        }
    }

    /// Směr podle kódu z datového souboru, viz [`Direction::code`].
    pub fn from_code(code: &str) -> Option<Direction> {
        Direction::ALL
            .into_iter()
            .find(|direction| direction.code() == code)
    }
}

/// Identifikátor kartičky: klíč slovíčka a směr.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId {
    /// Klíč slovíčka, viz [`Word::key`].
    pub key: String,
    /// Směr procvičování.
    pub direction: Direction,
}

/// Načtená lekce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lesson {
    /// Číslo lekce z názvu souboru (`lekce03.toml` → 3). Podle něj se lekce řadí.
    pub number: u32,
    /// Název souboru, ze kterého lekce pochází (pro chybové hlášky).
    pub file_name: String,
    /// Název lekce.
    pub title: String,
    /// Volitelný popis lekce.
    pub description: Option<String>,
    /// Slovíčka v pořadí, v jakém jsou v souboru.
    pub words: Vec<Word>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(vi: &str, key: Option<&str>) -> Word {
        Word {
            vi: vi.to_owned(),
            key_override: key.map(str::to_owned),
            cs: "význam".to_owned(),
            pronunciation_cs: "výslovnost".to_owned(),
            ipa: "ipa".to_owned(),
            note: None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn key_defaults_to_vietnamese_word() {
        assert_eq!(word("cảm ơn", None).key(), "cảm ơn");
    }

    #[test]
    fn key_override_wins() {
        assert_eq!(word("đường", Some("đường (cukr)")).key(), "đường (cukr)");
    }

    #[test]
    fn direction_codes_round_trip() {
        for direction in Direction::ALL {
            assert_eq!(Direction::from_code(direction.code()), Some(direction));
        }
        assert_eq!(Direction::from_code("CS-VI"), None);
        assert_eq!(Direction::from_code(""), None);
    }

    #[test]
    fn word_yields_card_for_each_direction() {
        let [first, second] = word("đường", Some("đường (cukr)")).card_ids();
        assert_eq!(first.key, "đường (cukr)");
        assert_eq!(first.direction, Direction::CsToVi);
        assert_eq!(second.key, "đường (cukr)");
        assert_eq!(second.direction, Direction::ViToCs);
    }
}
