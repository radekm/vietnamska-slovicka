//! Načítání lekcí ze složky se slovníčky a jejich kontrola.
//!
//! Lekce jsou v souborech `lekce01.toml`, `lekce02.toml`, … a řadí se podle čísla v názvu.
//! Při načítání se sbírají **všechny** nalezené problémy najednou, aby šlo opravit
//! celou složku na jeden zátah, a ne chybu po chybě.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::model::{Lesson, Word};

/// Předpona názvu souboru s lekcí.
const FILE_PREFIX: &str = "lekce";
/// Přípona názvu souboru s lekcí.
const FILE_SUFFIX: &str = ".toml";

/// Obsah souboru s lekcí tak, jak je zapsaný v TOML.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LessonFile {
    nazev: String,
    #[serde(default)]
    popis: Option<String>,
    #[serde(default)]
    slovicka: Vec<Word>,
}

/// Chyby nalezené ve slovníčcích. Obsahuje všechny problémy najednou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonErrors {
    /// Popisy jednotlivých problémů, každý se srozumitelnou hláškou pro uživatele.
    pub problems: Vec<String>,
}

impl fmt::Display for LessonErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ve slovníčcích jsou problémy ({}):", self.problems.len())?;
        for problem in &self.problems {
            write!(f, "\n- {problem}")?;
        }
        Ok(())
    }
}

impl std::error::Error for LessonErrors {}

/// Zjistí číslo lekce z názvu souboru (`lekce07.toml` → 7).
///
/// Vrací `None`, pokud název neodpovídá vzoru `lekce<číslo>.toml`.
pub fn lesson_number(file_name: &str) -> Option<u32> {
    let digits = file_name
        .strip_prefix(FILE_PREFIX)?
        .strip_suffix(FILE_SUFFIX)?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Načte všechny lekce ze složky se slovníčky.
///
/// Soubory, které neodpovídají vzoru `lekce<číslo>.toml`, se ignorují (a zalogují).
/// Pokud je ve slovníčcích jakýkoliv problém, vrátí chybu [`LessonErrors`]
/// se seznamem všech problémů. Chyba čtení samotné složky je obyčejná chyba.
pub fn load_dir(dir: &Path) -> Result<Vec<Lesson>> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("nelze číst složku se slovníčky {}", dir.display()))?;

    // Nalezené soubory s lekcemi: (číslo, název souboru).
    let mut files = Vec::new();
    for entry in entries {
        let entry =
            entry.with_context(|| format!("nelze číst složku se slovníčky {}", dir.display()))?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            tracing::warn!(path = %entry.path().display(), "soubor s ne-UTF-8 názvem ignorován");
            continue;
        };
        match lesson_number(&file_name) {
            Some(number) if entry.path().is_file() => files.push((number, file_name)),
            _ => {
                if file_name.ends_with(FILE_SUFFIX) {
                    tracing::warn!(file = %file_name, "soubor neodpovídá vzoru lekceNN.toml, ignorován");
                }
            }
        }
    }
    files.sort();

    let mut problems = Vec::new();

    // Dvě lekce se stejným číslem (např. lekce1.toml a lekce01.toml) by se nedaly seřadit.
    for pair in files.windows(2) {
        if pair[0].0 == pair[1].0 {
            problems.push(format!(
                "Soubory {} a {} mají stejné číslo lekce {}.",
                pair[0].1, pair[1].1, pair[0].0
            ));
        }
    }

    let mut lessons = Vec::new();
    for (number, file_name) in files {
        let text = match std::fs::read_to_string(dir.join(&file_name)) {
            Ok(text) => text,
            Err(err) => {
                problems.push(format!("{file_name}: soubor nelze přečíst: {err}"));
                continue;
            }
        };
        let (lesson, mut lesson_problems) = parse_lesson_with_problems(&file_name, number, &text);
        problems.append(&mut lesson_problems);
        // I lekce s problémy se zařadí do kontroly duplicit, aby se všechno ohlásilo najednou.
        lessons.extend(lesson);
    }

    problems.extend(check_duplicate_keys(&lessons));

    if !problems.is_empty() {
        tracing::warn!(count = problems.len(), dir = %dir.display(), "problémy ve slovníčcích");
        return Err(LessonErrors { problems }.into());
    }

    tracing::info!(
        dir = %dir.display(),
        lessons = lessons.len(),
        words = lessons.iter().map(|l| l.words.len()).sum::<usize>(),
        "slovníčky načteny"
    );
    Ok(lessons)
}

/// Načte jednu lekci z textu souboru a zkontroluje ji.
///
/// Kontroluje jen samotnou lekci. Duplicitní klíče napříč lekcemi
/// kontroluje [`check_duplicate_keys`].
pub fn parse_lesson(file_name: &str, number: u32, text: &str) -> Result<Lesson, Vec<String>> {
    match parse_lesson_with_problems(file_name, number, text) {
        (Some(lesson), problems) if problems.is_empty() => Ok(lesson),
        (_, problems) => Err(problems),
    }
}

/// Načte lekci a vrátí ji spolu se seznamem problémů.
///
/// Lekce je `None` jen tehdy, když nejde přečíst samotný zápis TOML.
fn parse_lesson_with_problems(
    file_name: &str,
    number: u32,
    text: &str,
) -> (Option<Lesson>, Vec<String>) {
    let file: LessonFile = match toml::from_str(text) {
        Ok(file) => file,
        Err(err) => {
            let message = err.to_string();
            return (
                None,
                vec![format!(
                    "{file_name}: chyba v zápisu TOML:\n{}",
                    message.trim_end()
                )],
            );
        }
    };

    let mut problems = Vec::new();

    if file.nazev.trim().is_empty() {
        problems.push(format!("{file_name}: lekce nemá vyplněný název (nazev)."));
    }
    if file.slovicka.is_empty() {
        problems.push(format!("{file_name}: lekce neobsahuje žádné slovíčko."));
    }
    for (index, word) in file.slovicka.iter().enumerate() {
        let location = word_location(file_name, index, word);
        problems.extend(
            check_word(word)
                .into_iter()
                .map(|problem| format!("{location}: {problem}")),
        );
    }

    let lesson = Lesson {
        number,
        file_name: file_name.to_owned(),
        title: file.nazev,
        description: file.popis.filter(|popis| !popis.trim().is_empty()),
        words: file.slovicka,
    };
    (Some(lesson), problems)
}

/// Najde klíče, které se opakují (v jedné lekci nebo napříč lekcemi).
pub fn check_duplicate_keys(lessons: &[Lesson]) -> Vec<String> {
    // Klíč → místa, kde se vyskytuje, v pořadí lekcí a slovíček.
    let mut occurrences: HashMap<&str, Vec<String>> = HashMap::new();
    let mut order = Vec::new();
    for lesson in lessons {
        for (index, word) in lesson.words.iter().enumerate() {
            let places = occurrences.entry(word.key()).or_insert_with(|| {
                order.push(word.key());
                Vec::new()
            });
            places.push(format!("{}, slovíčko č. {}", lesson.file_name, index + 1));
        }
    }

    order
        .into_iter()
        .filter_map(|key| {
            let places = &occurrences[key];
            (places.len() > 1).then(|| {
                format!(
                    "Klíč „{key}“ je použitý víckrát: {}. Pokud jde o homonyma \
                     (stejné slovo, jiný význam), doplň každému pole klic, \
                     např. klic = \"{key} (význam)\".",
                    places.join("; ")
                )
            })
        })
        .collect()
}

/// Popis místa slovíčka pro chybové hlášky.
fn word_location(file_name: &str, index: usize, word: &Word) -> String {
    if word.vi.trim().is_empty() {
        format!("{file_name}, slovíčko č. {}", index + 1)
    } else {
        format!("{file_name}, slovíčko č. {} ({})", index + 1, word.vi)
    }
}

/// Zkontroluje jedno slovíčko a vrátí seznam problémů.
fn check_word(word: &Word) -> Vec<String> {
    let mut problems = Vec::new();

    let required = [
        ("vi", &word.vi),
        ("cs", &word.cs),
        ("vyslovnost_cs", &word.pronunciation_cs),
        ("ipa", &word.ipa),
    ];
    for (field, value) in required {
        if value.trim().is_empty() {
            problems.push(format!("pole {field} je prázdné"));
        }
    }

    if let Some(key) = &word.key_override
        && key.trim().is_empty()
    {
        problems.push("pole klic je prázdné (pokud ho nepotřebuješ, smaž ho)".to_owned());
    }

    // Mezery na začátku nebo na konci klíče by vytvořily jiný klíč, který na pohled
    // vypadá stejně, a slovíčko by přišlo o postup učení.
    let mut key_fields = vec![("vi", &word.vi)];
    if let Some(key) = &word.key_override {
        key_fields.push(("klic", key));
    }
    for (field, value) in &key_fields {
        if !value.is_empty() && value.trim() != value.as_str() {
            problems.push(format!("pole {field} má mezeru na začátku nebo na konci"));
        }
    }

    // Stejné písmeno jde v Unicode zapsat jako jeden znak (ờ) nebo jako písmeno
    // + kombinující znaménka (o + ̛ + ̀). Na pohled se neliší, ale klíče by se lišily.
    // Proto u textů, které nejsou IPA, vyžadujeme složené znaky (NFC).
    // U IPA jsou kombinující znaménka (t͡ɕ, ̚) v pořádku.
    let mut text_fields = key_fields;
    text_fields.push(("cs", &word.cs));
    text_fields.push(("vyslovnost_cs", &word.pronunciation_cs));
    for (field, value) in text_fields {
        if value.chars().any(is_combining_mark) {
            problems.push(format!(
                "pole {field} obsahuje rozložené znaky s kombinující diakritikou; \
                 použij složené znaky (Unicode NFC)"
            ));
        }
    }

    problems
}

/// Kombinující diakritická znaménka (blok U+0300–U+036F).
fn is_combining_mark(c: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    const LESSON_1: &str = r#"
nazev = "Pozdravy"
popis = "Základní pozdravy"

[[slovicka]]
vi = "xin chào"
cs = "dobrý den / ahoj"
vyslovnost_cs = "sin (rovně) čao (nízko klesá)"
ipa = "sin˧˧ t͡ɕaːw˨˩"
stitky = ["pozdrav"]

[[slovicka]]
vi = "cảm ơn"
cs = "děkuji"
vyslovnost_cs = "kam (klesá a stoupá) ən (rovně)"
ipa = "kaːm˧˩˧ ʔəːn˧˧"
poznamka = "běžné poděkování"
"#;

    const LESSON_2: &str = r#"
nazev = "Jídlo"

[[slovicka]]
vi = "đường"
klic = "đường (cukr)"
cs = "cukr"
vyslovnost_cs = "dɨəng (nízko klesá)"
ipa = "ɗɨəŋ˨˩"
"#;

    /// Jedno slovíčko s doplněnými poli, která test nastavuje.
    fn lesson_with_word(fields: &str) -> String {
        format!("nazev = \"Test\"\n\n[[slovicka]]\n{fields}\n")
    }

    /// Problémy z chyby [`load_dir`].
    fn problems_of(result: Result<Vec<Lesson>>) -> Vec<String> {
        result
            .unwrap_err()
            .downcast::<LessonErrors>()
            .expect("čekali jsme LessonErrors")
            .problems
    }

    #[test]
    fn lesson_number_from_file_name() {
        assert_eq!(lesson_number("lekce01.toml"), Some(1));
        assert_eq!(lesson_number("lekce12.toml"), Some(12));
        assert_eq!(lesson_number("lekce7.toml"), Some(7));
        assert_eq!(lesson_number("lekce.toml"), None);
        assert_eq!(lesson_number("lekce01.txt"), None);
        assert_eq!(lesson_number("lekce-1.toml"), None);
        assert_eq!(lesson_number("lekce+1.toml"), None);
        assert_eq!(lesson_number("Lekce01.toml"), None);
        assert_eq!(lesson_number("poznamky.toml"), None);
    }

    #[test]
    fn parses_valid_lesson() {
        let lesson = parse_lesson("lekce01.toml", 1, LESSON_1).unwrap();
        assert_eq!(lesson.number, 1);
        assert_eq!(lesson.title, "Pozdravy");
        assert_eq!(lesson.description.as_deref(), Some("Základní pozdravy"));
        assert_eq!(lesson.words.len(), 2);

        let thanks = &lesson.words[1];
        assert_eq!(thanks.vi, "cảm ơn");
        assert_eq!(thanks.key(), "cảm ơn");
        assert_eq!(thanks.ipa, "kaːm˧˩˧ ʔəːn˧˧");
        assert_eq!(thanks.note.as_deref(), Some("běžné poděkování"));
        assert_eq!(lesson.words[0].tags, ["pozdrav"]);
    }

    #[test]
    fn key_override_is_read() {
        let lesson = parse_lesson("lekce02.toml", 2, LESSON_2).unwrap();
        assert_eq!(lesson.words[0].key(), "đường (cukr)");
        assert_eq!(lesson.description, None);
    }

    #[test]
    fn invalid_toml_reports_file_name() {
        let problems = parse_lesson("lekce01.toml", 1, "nazev = ").unwrap_err();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].starts_with("lekce01.toml: chyba v zápisu TOML"));
    }

    #[test]
    fn unknown_field_is_error() {
        // Překlep v názvu pole (poznámka s diakritikou) se nesmí tiše ignorovat.
        let text = lesson_with_word(
            r#"vi = "a"
cs = "b"
vyslovnost_cs = "c"
ipa = "d"
poznámka = "e""#,
        );
        let problems = parse_lesson("lekce01.toml", 1, &text).unwrap_err();
        assert!(problems[0].contains("poznámka"), "{problems:?}");
    }

    #[test]
    fn missing_required_field_is_error() {
        let text = lesson_with_word(
            r#"vi = "a"
cs = "b"
vyslovnost_cs = "c""#,
        );
        let problems = parse_lesson("lekce01.toml", 1, &text).unwrap_err();
        assert!(problems[0].contains("ipa"), "{problems:?}");
    }

    #[test]
    fn empty_fields_are_reported_all_at_once() {
        let text = lesson_with_word(
            r#"vi = "chó"
cs = " "
vyslovnost_cs = ""
ipa = "t͡ɕɔ˧˥"
klic = """#,
        );
        let problems = parse_lesson("lekce01.toml", 1, &text).unwrap_err();
        assert_eq!(problems.len(), 3, "{problems:?}");
        assert!(
            problems
                .iter()
                .all(|p| p.starts_with("lekce01.toml, slovíčko č. 1 (chó): "))
        );
        assert!(problems[0].ends_with("pole cs je prázdné"));
        assert!(problems[1].ends_with("pole vyslovnost_cs je prázdné"));
        assert!(problems[2].contains("pole klic je prázdné"));
    }

    #[test]
    fn lesson_without_title_or_words_is_error() {
        let problems = parse_lesson("lekce01.toml", 1, "nazev = \"\"").unwrap_err();
        assert_eq!(problems.len(), 2, "{problems:?}");
    }

    #[test]
    fn whitespace_around_key_is_error() {
        let text = lesson_with_word(
            r#"vi = "chó "
cs = "pes"
vyslovnost_cs = "čo (stoupá)"
ipa = "t͡ɕɔ˧˥""#,
        );
        let problems = parse_lesson("lekce01.toml", 1, &text).unwrap_err();
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("mezeru"));
    }

    #[test]
    fn decomposed_vietnamese_is_error() {
        // „chó“ zapsané rozloženě: o + kombinující čárka (U+0301).
        let text = lesson_with_word(
            "vi = \"cho\u{0301}\"\ncs = \"pes\"\nvyslovnost_cs = \"čo (stoupá)\"\nipa = \"t͡ɕɔ˧˥\"",
        );
        let problems = parse_lesson("lekce01.toml", 1, &text).unwrap_err();
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("pole vi obsahuje rozložené znaky"));
    }

    #[test]
    fn combining_marks_in_ipa_are_allowed() {
        // t͡ɕ (U+0361) a ̚ (U+031A) jsou v IPA správně.
        let text = lesson_with_word(
            r#"vi = "một"
cs = "jedna"
vyslovnost_cs = "mot (krátce klesá, ráz)"
ipa = "mot̚˨˩ˀ""#,
        );
        assert!(parse_lesson("lekce01.toml", 1, &text).is_ok());
    }

    #[test]
    fn duplicate_keys_across_lessons_are_reported() {
        let first = parse_lesson("lekce01.toml", 1, LESSON_1).unwrap();
        let mut second = parse_lesson("lekce02.toml", 2, LESSON_2).unwrap();
        // Druhé slovíčko „cảm ơn“ bez vlastního klíče koliduje s lekcí 1.
        let mut duplicate = first.words[1].clone();
        duplicate.cs = "díky".to_owned();
        second.words.push(duplicate);

        let problems = check_duplicate_keys(&[first, second]);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("„cảm ơn“"));
        assert!(problems[0].contains("lekce01.toml, slovíčko č. 2; lekce02.toml, slovíčko č. 2"));
        assert!(problems[0].contains("klic"));
    }

    #[test]
    fn homonyms_with_key_override_are_fine() {
        let mut lesson = parse_lesson("lekce02.toml", 2, LESSON_2).unwrap();
        let mut road = lesson.words[0].clone();
        road.key_override = Some("đường (cesta)".to_owned());
        road.cs = "cesta".to_owned();
        lesson.words.push(road);
        assert!(check_duplicate_keys(&[lesson]).is_empty());
    }

    #[test]
    fn loads_directory_sorted_by_number() {
        let dir = TempDir::new("sorted");
        dir.write(
            "lekce10.toml",
            &LESSON_2.replace("đường (cukr)", "đường (cukr 10)"),
        );
        dir.write("lekce02.toml", LESSON_2);
        dir.write("lekce01.toml", LESSON_1);
        // Jiné soubory se ignorují.
        dir.write("poznamky.txt", "nic");
        dir.write("navrh.toml", "neplatný = obsah = ");
        std::fs::create_dir(dir.0.join("lekce03.toml")).unwrap();

        let lessons = load_dir(&dir.0).unwrap();
        let numbers: Vec<u32> = lessons.iter().map(|l| l.number).collect();
        assert_eq!(numbers, [1, 2, 10]);
        assert_eq!(lessons[2].file_name, "lekce10.toml");
    }

    #[test]
    fn empty_directory_has_no_lessons() {
        let dir = TempDir::new("empty");
        assert!(load_dir(&dir.0).unwrap().is_empty());
    }

    #[test]
    fn missing_directory_is_error() {
        let dir = TempDir::new("missing");
        assert!(load_dir(&dir.0.join("neexistuje")).is_err());
    }

    #[test]
    fn directory_problems_are_collected_from_all_files() {
        let dir = TempDir::new("problems");
        dir.write("lekce01.toml", LESSON_1);
        dir.write("lekce1.toml", LESSON_2);
        dir.write("lekce02.toml", "nazev = ");
        dir.write("lekce03.toml", LESSON_1);

        let problems = problems_of(load_dir(&dir.0));
        // Stejné číslo lekce, chyba TOML, dva duplicitní klíče z lekce03.
        assert_eq!(problems.len(), 4, "{problems:#?}");
        assert!(problems[0].contains("stejné číslo lekce 1"));
        assert!(problems[1].starts_with("lekce02.toml: chyba v zápisu TOML"));
        assert!(problems[2].contains("„xin chào“"));
        assert!(problems[3].contains("„cảm ơn“"));
    }

    #[test]
    fn duplicates_are_reported_even_in_lesson_with_other_problems() {
        let dir = TempDir::new("duplicates-with-problems");
        let word = "[[slovicka]]\nvi = \"mèo\"\ncs = \"kočka\"\n\
                    vyslovnost_cs = \"meo (nízko klesá)\"\nipa = \"mɛw˨˩\"\n";
        dir.write("lekce01.toml", &format!("nazev = \"\"\n{word}{word}"));

        let problems = problems_of(load_dir(&dir.0));
        assert_eq!(problems.len(), 2, "{problems:#?}");
        assert!(problems[0].contains("nemá vyplněný název"));
        assert!(problems[1].contains("„mèo“"));
    }

    #[test]
    fn errors_display_lists_all_problems() {
        let errors = LessonErrors {
            problems: vec!["první".to_owned(), "druhý".to_owned()],
        };
        assert_eq!(
            errors.to_string(),
            "Ve slovníčcích jsou problémy (2):\n- první\n- druhý"
        );
    }
}
