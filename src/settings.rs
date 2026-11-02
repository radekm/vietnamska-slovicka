//! Nastavení aplikace a jeho ukládání do `data/nastaveni.toml`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

use crate::fsrs::{DEFAULT_DESIRED_RETENTION, Fsrs};
use crate::model::Direction;
use crate::storage;

/// Výchozí denní limit nových kartiček.
pub const DEFAULT_NEW_CARDS_PER_DAY: u32 = 10;

/// Největší povolený denní limit nových kartiček.
pub const MAX_NEW_CARDS_PER_DAY: u32 = 1000;

/// Nastavení aplikace.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// Složka se slovníčky (lekcemi). Dokud ji uživatel nevybere, není nastavená.
    pub lessons_dir: Option<PathBuf>,
    /// Denní limit nových kartiček.
    pub new_cards_per_day: u32,
    /// Cílová pravděpodobnost zapamatování pro FSRS.
    pub desired_retention: f64,
    /// Zapnuté směry procvičování, aspoň jeden, seřazené a bez opakování.
    pub directions: Vec<Direction>,
    /// Čísla lekcí, ze kterých se berou nové kartičky, seřazená a bez opakování.
    pub selected_lessons: Vec<u32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lessons_dir: None,
            new_cards_per_day: DEFAULT_NEW_CARDS_PER_DAY,
            desired_retention: DEFAULT_DESIRED_RETENTION,
            directions: Direction::ALL.to_vec(),
            selected_lessons: Vec::new(),
        }
    }
}

/// Podoba souboru `nastaveni.toml`. Názvy polí v souboru jsou česky.
///
/// Kromě verze jsou všechna pole nepovinná, chybějící dostane výchozí hodnotu.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsFile {
    #[serde(rename = "verze")]
    version: u32,
    #[serde(
        rename = "slozka_slovnicku",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    lessons_dir: Option<PathBuf>,
    #[serde(rename = "novych_denne", default = "default_new_cards_per_day")]
    new_cards_per_day: u32,
    #[serde(rename = "cilova_retence", default = "default_desired_retention")]
    desired_retention: f64,
    #[serde(rename = "smery", default = "default_directions")]
    directions: Vec<String>,
    #[serde(rename = "vybrane_lekce", default)]
    selected_lessons: Vec<u32>,
}

fn default_new_cards_per_day() -> u32 {
    DEFAULT_NEW_CARDS_PER_DAY
}

fn default_desired_retention() -> f64 {
    DEFAULT_DESIRED_RETENTION
}

fn default_directions() -> Vec<String> {
    Direction::ALL
        .iter()
        .map(|direction| direction.code().to_owned())
        .collect()
}

impl Settings {
    /// Načte nastavení ze souboru. Chybějící soubor znamená výchozí nastavení.
    pub fn load(path: &Path) -> Result<Self> {
        match storage::read_optional(path)? {
            None => Ok(Self::default()),
            Some(text) => Self::from_toml_str(&text)
                .with_context(|| format!("chyba v souboru s nastavením {}", path.display())),
        }
    }

    /// Bezpečně uloží nastavení do souboru.
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = self.to_toml_string()?;
        storage::write_atomic(path, &text)
            .with_context(|| format!("nelze uložit nastavení do {}", path.display()))
    }

    /// Nastavení z textu ve formátu TOML. Seřadí směry a lekce a odstraní opakování.
    pub fn from_toml_str(text: &str) -> Result<Self> {
        let file: SettingsFile = toml::from_str(text)?;
        storage::check_version(file.version)?;
        let mut directions = file
            .directions
            .iter()
            .map(|code| match Direction::from_code(code) {
                Some(direction) => Ok(direction),
                None => bail!("neznámý směr \"{code}\" (povolené jsou \"cs-vi\" a \"vi-cs\")"),
            })
            .collect::<Result<Vec<_>>>()?;
        directions.sort();
        directions.dedup();
        let mut selected_lessons = file.selected_lessons;
        selected_lessons.sort_unstable();
        selected_lessons.dedup();

        let settings = Self {
            lessons_dir: file.lessons_dir,
            new_cards_per_day: file.new_cards_per_day,
            desired_retention: file.desired_retention,
            directions,
            selected_lessons,
        };
        settings.validate()?;
        Ok(settings)
    }

    /// Nastavení jako text ve formátu TOML.
    pub fn to_toml_string(&self) -> Result<String> {
        self.validate()?;
        let file = SettingsFile {
            version: storage::FORMAT_VERSION,
            lessons_dir: self.lessons_dir.clone(),
            new_cards_per_day: self.new_cards_per_day,
            desired_retention: self.desired_retention,
            directions: self
                .directions
                .iter()
                .map(|direction| direction.code().to_owned())
                .collect(),
            selected_lessons: self.selected_lessons.clone(),
        };
        // Cesta, která není platné UTF-8, se do TOML zapsat nedá, to ohlásí serializace.
        toml::to_string(&file).context("nelze převést nastavení do TOML")
    }

    /// Zkontroluje, že hodnoty jsou v povoleném rozsahu.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.new_cards_per_day <= MAX_NEW_CARDS_PER_DAY,
            "denní limit nových kartiček {} je větší než povolených {MAX_NEW_CARDS_PER_DAY}",
            self.new_cards_per_day
        );
        ensure!(
            !self.directions.is_empty(),
            "musí být zapnutý aspoň jeden směr procvičování"
        );
        self.fsrs()?;
        Ok(())
    }

    /// Plánovač FSRS s cílovou retencí z nastavení.
    pub fn fsrs(&self) -> Result<Fsrs> {
        Fsrs::new(self.desired_retention)
    }

    /// Je směr zapnutý?
    pub fn direction_enabled(&self, direction: Direction) -> bool {
        self.directions.contains(&direction)
    }

    /// Je lekce vybraná pro nové kartičky?
    pub fn lesson_selected(&self, number: u32) -> bool {
        self.selected_lessons.binary_search(&number).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    #[test]
    fn missing_file_gives_defaults() {
        let dir = TempDir::new("settings-missing");
        let settings = Settings::load(&dir.join("nastaveni.toml")).unwrap();
        assert_eq!(settings, Settings::default());
        assert_eq!(settings.new_cards_per_day, 10);
        assert_eq!(settings.desired_retention, 0.9);
        assert_eq!(settings.directions, Direction::ALL);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = TempDir::new("settings-round-trip");
        let path = dir.join("data/nastaveni.toml");
        let settings = Settings {
            lessons_dir: Some(PathBuf::from("/home/radek/slovníčky")),
            new_cards_per_day: 15,
            desired_retention: 0.85,
            directions: vec![Direction::ViToCs],
            selected_lessons: vec![1, 3],
        };
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap(), settings);
    }

    #[test]
    fn file_format_uses_czech_names() {
        let text = Settings {
            selected_lessons: vec![2],
            ..Settings::default()
        }
        .to_toml_string()
        .unwrap();
        let expected = "verze = 1\nnovych_denne = 10\ncilova_retence = 0.9\n\
                        smery = [\"cs-vi\", \"vi-cs\"]\nvybrane_lekce = [2]\n";
        assert_eq!(text, expected);
    }

    #[test]
    fn missing_fields_get_defaults_and_lists_are_normalized() {
        let settings = Settings::from_toml_str(
            "verze = 1\nsmery = [\"vi-cs\", \"cs-vi\", \"vi-cs\"]\nvybrane_lekce = [3, 1, 3]\n",
        )
        .unwrap();
        assert_eq!(settings.new_cards_per_day, 10);
        assert_eq!(settings.directions, [Direction::CsToVi, Direction::ViToCs]);
        assert_eq!(settings.selected_lessons, [1, 3]);
        assert!(settings.lesson_selected(3));
        assert!(!settings.lesson_selected(2));
    }

    #[test]
    fn invalid_settings_are_rejected() {
        let cases = [
            "novych_denne = 10\n",                // chybí verze
            "verze = 2\n",                        // neznámá verze
            "verze = 1\nnovych_denne = 1001\n",   // přes limit
            "verze = 1\ncilova_retence = 0.5\n",  // retence mimo rozsah
            "verze = 1\nsmery = []\n",            // žádný směr
            "verze = 1\nsmery = [\"cs-en\"]\n",   // neznámý směr
            "verze = 1\nnovych_denné = 5\n",      // neznámé pole (překlep)
            "verze = 1\nnovych_denne = \"10\"\n", // špatný typ
        ];
        for text in cases {
            assert!(
                Settings::from_toml_str(text).is_err(),
                "mělo selhat: {text}"
            );
        }
    }

    #[test]
    fn corrupted_file_is_error_with_path() {
        let dir = TempDir::new("settings-corrupted");
        dir.write("nastaveni.toml", "verze = 1\nnovych_denne = \n");
        let err = Settings::load(&dir.join("nastaveni.toml")).unwrap_err();
        assert!(format!("{err:#}").contains("nastaveni.toml"), "{err:#}");
    }

    #[test]
    fn save_refuses_invalid_settings() {
        let dir = TempDir::new("settings-invalid-save");
        let path = dir.join("nastaveni.toml");
        let settings = Settings {
            directions: Vec::new(),
            ..Settings::default()
        };
        assert!(settings.save(&path).is_err());
        assert!(!path.exists());
    }
}
