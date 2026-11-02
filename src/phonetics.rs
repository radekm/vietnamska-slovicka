//! Tóny a kontrola výslovnosti slovíček.
//!
//! Tabulka tónů (Chaova tónová písmena podle Kirby 2011 a slovní popisy) je tu jediným
//! zdrojem pro legendu tónů i pro kontrolu lekcí. Kontrola ověřuje, že počet slabik
//! ve vietnamštině, v IPA a v českém popisu souhlasí a že tóny v IPA a v popisu odpovídají
//! diakritice. Pravidla popisuje `PLAN.md`, oddíly „Český popis výslovnosti“
//! a „Konvence IPA“.
//!
//! Stejná pravidla (a navíc přepis hlásek) má pomocný skript `tools/pronunciation.py`.
//! Při změně tónů nebo popisů je potřeba upravit obojí.

use crate::model::Word;

/// Vietnamský tón.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Bez značky.
    Ngang,
    /// Značka à.
    Huyen,
    /// Značka á.
    Sac,
    /// Značka ả.
    Hoi,
    /// Značka ã.
    Nga,
    /// Značka ạ.
    Nang,
}

impl Tone {
    /// Všechny tóny v tradičním pořadí.
    pub const ALL: [Tone; 6] = [
        Tone::Ngang,
        Tone::Huyen,
        Tone::Sac,
        Tone::Hoi,
        Tone::Nga,
        Tone::Nang,
    ];

    /// Vietnamský název tónu.
    pub fn name(self) -> &'static str {
        match self {
            Tone::Ngang => "ngang",
            Tone::Huyen => "huyền",
            Tone::Sac => "sắc",
            Tone::Hoi => "hỏi",
            Tone::Nga => "ngã",
            Tone::Nang => "nặng",
        }
    }

    /// Tón jako značka nad písmenem a.
    pub fn mark(self) -> &'static str {
        match self {
            Tone::Ngang => "a",
            Tone::Huyen => "à",
            Tone::Sac => "á",
            Tone::Hoi => "ả",
            Tone::Nga => "ã",
            Tone::Nang => "ạ",
        }
    }

    /// Chaova tónová písmena v IPA. `checked` je zavřená slabika (končí na p, t, c, ch).
    pub fn chao(self, checked: bool) -> &'static str {
        match (self, checked) {
            (Tone::Ngang, _) => "˧˧",
            (Tone::Huyen, _) => "˧˨",
            (Tone::Sac, false) => "˨˦",
            (Tone::Sac, true) => "˦˥",
            (Tone::Hoi, _) => "˧˩˨",
            (Tone::Nga, _) => "˧ˀ˥",
            (Tone::Nang, false) => "˨˩ˀ",
            (Tone::Nang, true) => "˨˩",
        }
    }

    /// Slovní popis tónu v závorce v českém popisu výslovnosti.
    pub fn description(self, checked: bool) -> &'static str {
        match (self, checked) {
            (Tone::Ngang, _) => "rovně",
            (Tone::Huyen, _) => "klesá",
            (Tone::Sac, _) => "stoupá",
            (Tone::Hoi, _) => "klesá hluboko",
            (Tone::Nga, _) => "stoupá se zlomem",
            (Tone::Nang, false) => "krátce klesá, ráz",
            (Tone::Nang, true) => "krátce klesá",
        }
    }

    /// Tóny v zavřených slabikách. Jiné se v nich nevyskytují.
    pub fn allowed_in_checked(self) -> bool {
        matches!(self, Tone::Sac | Tone::Nang)
    }
}

/// Samohlásky se všemi tóny, v pořadí podle [`Tone::ALL`]. Malá písmena, složené znaky (NFC).
const TONED_VOWELS: [&str; 12] = [
    "aàáảãạ",
    "ăằắẳẵặ",
    "âầấẩẫậ",
    "eèéẻẽẹ",
    "êềếểễệ",
    "iìíỉĩị",
    "oòóỏõọ",
    "ôồốổỗộ",
    "ơờớởỡợ",
    "uùúủũụ",
    "ưừứửữự",
    "yỳýỷỹỵ",
];

/// Znaky, ze kterých se skládá zápis tónu za slabikou v IPA.
const CHAO_CHARS: &str = "˥˦˧˨˩ˀ";

/// Tón a typ jedné vietnamské slabiky.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyllableTone {
    /// Tón podle diakritiky.
    pub tone: Tone,
    /// Je slabika zavřená (končí na p, t, c, ch)?
    pub checked: bool,
}

/// Slabiky vietnamského textu: malými písmeny, bez interpunkce na okrajích.
///
/// Slova, ze kterých po odstranění interpunkce nic nezbude (např. pomlčka), se vynechají.
pub fn vietnamese_syllables(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_lowercase()
        })
        .filter(|syllable| !syllable.is_empty())
        .collect()
}

/// Tón slabiky (malými písmeny) podle diakritiky.
pub fn syllable_tone(syllable: &str) -> Result<SyllableTone, String> {
    let mut tone = None;
    let mut has_vowel = false;
    for c in syllable.chars() {
        let Some(index) = TONED_VOWELS
            .iter()
            .find_map(|vowels| vowels.chars().position(|v| v == c))
        else {
            continue;
        };
        has_vowel = true;
        if index == 0 {
            continue;
        }
        // Index je 1 až 5, `Tone::ALL` má 6 prvků.
        let found = Tone::ALL[index];
        if tone.is_some() {
            return Err(format!("slabika „{syllable}“ má víc značek tónu"));
        }
        tone = Some(found);
    }
    if !has_vowel {
        return Err(format!("slabika „{syllable}“ nemá samohlásku"));
    }
    let tone = tone.unwrap_or(Tone::Ngang);
    let checked = ["p", "t", "c", "ch"]
        .iter()
        .any(|ending| syllable.ends_with(ending));
    if checked && !tone.allowed_in_checked() {
        return Err(format!(
            "slabika „{syllable}“ končí na p, t, c nebo ch a nemůže mít tón {}",
            tone.name()
        ));
    }
    Ok(SyllableTone { tone, checked })
}

/// Zápis tónu na konci slabiky v IPA (např. `˧˩˨` z `kam˧˩˨`).
fn ipa_tone(syllable: &str) -> &str {
    let start = syllable
        .char_indices()
        .rev()
        .take_while(|(_, c)| CHAO_CHARS.contains(*c))
        .last()
        .map_or(syllable.len(), |(index, _)| index);
    &syllable[start..]
}

/// Rozdělí český popis výslovnosti na dvojice (slabika, popis tónu).
///
/// Očekávaný tvar: `kam (klesá hluboko) ən (rovně)`.
pub fn parse_pronunciation_cs(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut result = Vec::new();
    let mut rest = text.trim();
    while !rest.is_empty() {
        let Some(open) = rest.find('(') else {
            return Err(format!("za „{rest}“ chybí popis tónu v závorce"));
        };
        let Some(close) = rest[open..].find(')').map(|i| open + i) else {
            return Err(format!("v „{rest}“ chybí zavírací závorka"));
        };
        let syllable = rest[..open].trim();
        if syllable.is_empty() || syllable.contains(char::is_whitespace) {
            return Err(format!(
                "před každou závorkou má být právě jedna slabika, ne „{syllable}“"
            ));
        }
        let description = rest[open + 1..close].trim();
        result.push((syllable.to_owned(), description.to_owned()));
        rest = rest[close + 1..].trim_start();
    }
    Ok(result)
}

/// Zkontroluje výslovnost slovíčka. Vrátí seznam problémů (prázdný, když je vše v pořádku).
pub fn check_word(word: &Word) -> Vec<String> {
    let mut problems = Vec::new();
    let syllables = vietnamese_syllables(&word.vi);
    let tones: Vec<Option<SyllableTone>> = syllables
        .iter()
        .map(|syllable| match syllable_tone(syllable) {
            Ok(tone) => Some(tone),
            Err(problem) => {
                problems.push(problem);
                None
            }
        })
        .collect();

    let ipa: Vec<&str> = word.ipa.split_whitespace().collect();
    if ipa.len() != syllables.len() {
        problems.push(format!(
            "IPA má {} slabik, vietnamsky je jich {}",
            ipa.len(),
            syllables.len()
        ));
    } else {
        for ((syllable, tone), ipa_syllable) in syllables.iter().zip(&tones).zip(&ipa) {
            let Some(tone) = tone else { continue };
            let expected = tone.tone.chao(tone.checked);
            let actual = ipa_tone(ipa_syllable);
            if actual != expected {
                problems.push(format!(
                    "slabika „{syllable}“ má tón {} a v IPA má končit „{expected}“, \
                     ne „{ipa_syllable}“",
                    tone.tone.name()
                ));
            }
        }
    }

    match parse_pronunciation_cs(&word.pronunciation_cs) {
        Err(problem) => problems.push(format!("český popis výslovnosti: {problem}")),
        Ok(pairs) if pairs.len() != syllables.len() => problems.push(format!(
            "český popis výslovnosti má {} slabik, vietnamsky je jich {}",
            pairs.len(),
            syllables.len()
        )),
        Ok(pairs) => {
            for ((syllable, tone), (cs_syllable, description)) in
                syllables.iter().zip(&tones).zip(&pairs)
            {
                let Some(tone) = tone else { continue };
                let expected = tone.tone.description(tone.checked);
                if description != expected {
                    problems.push(format!(
                        "slabika „{syllable}“ má tón {}, v českém popisu má být \
                         „{cs_syllable} ({expected})“, ne „({description})“",
                        tone.tone.name()
                    ));
                }
            }
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(vi: &str, pronunciation_cs: &str, ipa: &str) -> Word {
        Word {
            vi: vi.to_owned(),
            key_override: None,
            cs: "význam".to_owned(),
            pronunciation_cs: pronunciation_cs.to_owned(),
            ipa: ipa.to_owned(),
            note: None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn tones_of_ma() {
        let tones: Vec<Tone> = ["ma", "mà", "má", "mả", "mã", "mạ"]
            .iter()
            .map(|s| syllable_tone(s).unwrap().tone)
            .collect();
        assert_eq!(tones, Tone::ALL);
    }

    #[test]
    fn checked_syllables() {
        let tone = syllable_tone("mát").unwrap();
        assert_eq!((tone.tone, tone.checked), (Tone::Sac, true));
        assert_eq!(tone.tone.chao(true), "˦˥");
        let tone = syllable_tone("sách").unwrap();
        assert!(tone.checked);
        assert!(!syllable_tone("anh").unwrap().checked);
        assert!(!syllable_tone("không").unwrap().checked);
        assert!(syllable_tone("mat").is_err(), "ngang v zavřené slabice");
        assert!(syllable_tone("mạt").unwrap().checked);
    }

    #[test]
    fn tone_on_any_vowel_letter() {
        assert_eq!(syllable_tone("người").unwrap().tone, Tone::Huyen);
        assert_eq!(syllable_tone("khỏe").unwrap().tone, Tone::Hoi);
        assert_eq!(syllable_tone("quý").unwrap().tone, Tone::Sac);
        assert_eq!(syllable_tone("đường").unwrap().tone, Tone::Huyen);
    }

    #[test]
    fn invalid_syllables() {
        assert!(syllable_tone("mấá").is_err());
        assert!(syllable_tone("ng").is_err());
    }

    #[test]
    fn syllables_ignore_punctuation_and_case() {
        assert_eq!(
            vietnamese_syllables("Bạn khỏe không? – Việt Nam,"),
            ["bạn", "khỏe", "không", "việt", "nam"]
        );
    }

    #[test]
    fn ipa_tone_suffix() {
        assert_eq!(ipa_tone("kam˧˩˨"), "˧˩˨");
        assert_eq!(ipa_tone("ma˧ˀ˥"), "˧ˀ˥");
        assert_eq!(ipa_tone("ɗɯəŋ˧˨"), "˧˨");
        assert_eq!(ipa_tone("kam"), "");
    }

    #[test]
    fn parse_czech_pronunciation() {
        assert_eq!(
            parse_pronunciation_cs("kam (klesá hluboko) ən (rovně)").unwrap(),
            [
                ("kam".to_owned(), "klesá hluboko".to_owned()),
                ("ən".to_owned(), "rovně".to_owned())
            ]
        );
        assert!(parse_pronunciation_cs("kam ən (rovně)").is_err());
        assert!(parse_pronunciation_cs("kam (rovně").is_err());
        assert!(parse_pronunciation_cs("kam (rovně) ən").is_err());
    }

    #[test]
    fn valid_words_pass() {
        let words = [
            word("xin chào", "sin (rovně) čau (klesá)", "sin˧˧ t͡ɕaw˧˨"),
            word("cảm ơn", "kam (klesá hluboko) ən (rovně)", "kam˧˩˨ ʔɤn˧˧"),
            word("một", "mot (krátce klesá)", "mot˨˩"),
            word("bạn", "ban (krátce klesá, ráz)", "ɓan˨˩ˀ"),
            word(
                "Bạn khỏe không?",
                "ban (krátce klesá, ráz) chue (klesá hluboko) chong (rovně)",
                "ɓan˨˩ˀ xʷɛ˧˩˨ xoŋ͡m˧˧",
            ),
        ];
        for word in &words {
            assert_eq!(check_word(word), Vec::<String>::new(), "{}", word.vi);
        }
    }

    #[test]
    fn wrong_tones_and_counts_are_reported() {
        let problems = check_word(&word(
            "cảm ơn",
            "kam (klesá a stoupá) ən (rovně)",
            "kam˧˩˧ ʔɤn˧˧",
        ));
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(problems[0].contains("˧˩˨"), "{problems:?}");
        assert!(problems[1].contains("klesá hluboko"), "{problems:?}");

        let problems = check_word(&word("mát", "mat (stoupá)", "mat˨˦"));
        assert_eq!(problems.len(), 1, "{problems:?}");

        let problems = check_word(&word("xin chào", "sin (rovně)", "sin˧˧"));
        assert_eq!(problems.len(), 2, "{problems:?}");
    }
}
