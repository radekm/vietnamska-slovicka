//! Algoritmus rozloženého opakování FSRS-5 (Free Spaced Repetition Scheduler).
//!
//! Vlastní implementace podle referenční implementace py-fsrs v4.1.2 s výchozími parametry.
//! Plánuje se po celých dnech, bez učebních kroků a bez náhodného rozptylu intervalů,
//! takže výpočet je deterministický.
//!
//! Modul je čistá logika bez GUI a bez přístupu k hodinám. Aktuální studijní den mu dodává
//! volající (viz [`study_day`]).

use anyhow::{Result, ensure};
use time::{Date, Duration, OffsetDateTime};

/// Výchozí parametry FSRS-5 (w0 až w18).
pub const DEFAULT_PARAMETERS: [f64; 19] = [
    0.40255, 1.18385, 3.173, 15.69105, 7.1949, 0.5345, 1.4604, 0.0046, 1.54575, 0.1192, 1.01925,
    1.9395, 0.11, 0.29605, 2.2698, 0.2315, 2.9898, 0.51655, 0.6621,
];

/// Výchozí cílová pravděpodobnost zapamatování.
pub const DEFAULT_DESIRED_RETENTION: f64 = 0.9;

/// Nejmenší povolená cílová retence.
pub const MIN_DESIRED_RETENTION: f64 = 0.70;

/// Největší povolená cílová retence.
pub const MAX_DESIRED_RETENTION: f64 = 0.99;

/// Nejdelší možný interval ve dnech (100 let).
pub const MAX_INTERVAL_DAYS: u32 = 36_500;

/// Hodina místního času, ve které začíná nový studijní den.
pub const DAY_START_HOUR: i64 = 4;

/// Exponent křivky zapomínání ve FSRS-5.
const DECAY: f64 = -0.5;

/// Konstanta křivky zapomínání. Volí se tak, aby po uplynutí `stability` dnů
/// byla pravděpodobnost vybavení přesně 0,9.
fn factor() -> f64 {
    0.9_f64.powf(1.0 / DECAY) - 1.0
}

/// Hodnocení odpovědi uživatelem (klávesy 1–4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rating {
    /// Nevěděl jsem.
    Again = 1,
    /// Věděl jsem, ale s velkými obtížemi.
    Hard = 2,
    /// Věděl jsem.
    Good = 3,
    /// Věděl jsem hned a bez přemýšlení.
    Easy = 4,
}

impl Rating {
    /// Všechna hodnocení v pořadí tlačítek.
    pub const ALL: [Rating; 4] = [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy];

    /// Hodnocení podle čísla 1–4 (např. ze stisknuté klávesy).
    pub fn from_number(number: u8) -> Option<Rating> {
        match number {
            1 => Some(Rating::Again),
            2 => Some(Rating::Hard),
            3 => Some(Rating::Good),
            4 => Some(Rating::Easy),
            _ => None,
        }
    }

    /// Číslo hodnocení 1–4.
    pub fn number(self) -> u8 {
        self as u8
    }

    /// Popisek tlačítka v uživatelském rozhraní.
    pub fn label(self) -> &'static str {
        match self {
            Rating::Again => "Znovu",
            Rating::Hard => "Těžké",
            Rating::Good => "Dobré",
            Rating::Easy => "Snadné",
        }
    }

    /// Hodnocení jako číslo pro vzorce.
    fn value(self) -> f64 {
        f64::from(self.number())
    }
}

/// Paměťový stav kartičky podle FSRS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryState {
    /// Stabilita ve dnech: za kolik dní klesne pravděpodobnost vybavení na 0,9.
    pub stability: f64,
    /// Obtížnost v rozsahu 1 (snadná) až 10 (těžká).
    pub difficulty: f64,
}

/// Stav kartičky, která už byla aspoň jednou opakována.
///
/// Nová kartička stav nemá, v API se předává jako `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct CardState {
    /// Paměťový stav.
    pub memory: MemoryState,
    /// Studijní den posledního opakování.
    pub last_review: Date,
    /// Studijní den, kdy je kartička znovu na řadě.
    pub due: Date,
    /// Počet všech opakování.
    pub reps: u32,
    /// Počet zapomenutí: hodnocení „Znovu“ při opakování v jiný den než předchozí opakování.
    pub lapses: u32,
}

/// Plánovač opakování s parametry FSRS-5 a cílovou retencí.
#[derive(Debug, Clone, PartialEq)]
pub struct Fsrs {
    w: [f64; 19],
    desired_retention: f64,
}

impl Fsrs {
    /// Plánovač s výchozími parametry a danou cílovou retencí.
    ///
    /// Retence musí být v rozsahu [`MIN_DESIRED_RETENTION`] až [`MAX_DESIRED_RETENTION`].
    pub fn new(desired_retention: f64) -> Result<Self> {
        // Zápis přes `contains` odmítne i NaN.
        ensure!(
            (MIN_DESIRED_RETENTION..=MAX_DESIRED_RETENTION).contains(&desired_retention),
            "cílová retence {desired_retention} musí být v rozsahu \
             {MIN_DESIRED_RETENTION} až {MAX_DESIRED_RETENTION}"
        );
        Ok(Self {
            w: DEFAULT_PARAMETERS,
            desired_retention,
        })
    }

    /// Cílová retence.
    pub fn desired_retention(&self) -> f64 {
        self.desired_retention
    }

    /// Pravděpodobnost vybavení po `elapsed_days` dnech od posledního opakování.
    pub fn retrievability(&self, stability: f64, elapsed_days: u32) -> f64 {
        (1.0 + factor() * f64::from(elapsed_days) / stability).powf(DECAY)
    }

    /// Interval ve dnech, po kterém klesne pravděpodobnost vybavení na cílovou retenci.
    ///
    /// Zaokrouhluje se na celé dny (polovina k sudé, stejně jako `round` v Pythonu
    /// v referenční implementaci) a omezuje na 1 až [`MAX_INTERVAL_DAYS`].
    pub fn next_interval(&self, stability: f64) -> u32 {
        let interval = stability / factor() * (self.desired_retention.powf(1.0 / DECAY) - 1.0);
        let interval = interval
            .round_ties_even()
            .clamp(1.0, f64::from(MAX_INTERVAL_DAYS));
        // Hodnota je po omezení celé číslo v rozsahu 1..=36500, převod je bezztrátový.
        // NaN (nemožné při kladné stabilitě) by se převedlo na 0, proto ještě `max(1)`.
        (interval as u32).max(1)
    }

    /// Nový stav kartičky po hodnocení `rating` ve studijní den `today`.
    ///
    /// `card` je `None` u nové kartičky. Opakování ve stejný den jako předchozí
    /// (např. zopakování po „Znovu“ během sezení) používá krátkodobý vzorec FSRS-5.
    /// Kdyby byl `today` dřív než poslední opakování (posunuté hodiny), bere se jako
    /// opakování ve stejný den.
    pub fn review(&self, card: Option<&CardState>, rating: Rating, today: Date) -> CardState {
        let (memory, reps, lapses) = match card {
            None => (self.initial_memory(rating), 1, 0),
            Some(card) => {
                let elapsed_days = (today - card.last_review).whole_days();
                let memory = if elapsed_days < 1 {
                    self.short_term_memory(card.memory, rating)
                } else {
                    // Víc než u32::MAX dní uplynout nemůže, kdyby ano, stačí strop.
                    let elapsed_days = u32::try_from(elapsed_days).unwrap_or(u32::MAX);
                    self.long_term_memory(card.memory, rating, elapsed_days)
                };
                let lapse = rating == Rating::Again && elapsed_days >= 1;
                (
                    memory,
                    card.reps.saturating_add(1),
                    card.lapses.saturating_add(u32::from(lapse)),
                )
            }
        };
        let interval = self.next_interval(memory.stability);
        // Přetečení nastane jen u data blízko roku 9999, pak stačí nejzazší možné datum.
        let due = today
            .checked_add(Duration::days(i64::from(interval)))
            .unwrap_or(Date::MAX);
        CardState {
            memory,
            last_review: today,
            due,
            reps,
            lapses,
        }
    }

    /// Paměťový stav po prvním hodnocení nové kartičky.
    fn initial_memory(&self, rating: Rating) -> MemoryState {
        MemoryState {
            stability: self.initial_stability(rating),
            difficulty: self.initial_difficulty(rating),
        }
    }

    /// Paměťový stav po opakování ve stejný den.
    fn short_term_memory(&self, memory: MemoryState, rating: Rating) -> MemoryState {
        let w = &self.w;
        MemoryState {
            stability: memory.stability * (w[17] * (rating.value() - 3.0 + w[18])).exp(),
            difficulty: self.next_difficulty(memory.difficulty, rating),
        }
    }

    /// Paměťový stav po opakování s odstupem aspoň jednoho dne.
    fn long_term_memory(
        &self,
        memory: MemoryState,
        rating: Rating,
        elapsed_days: u32,
    ) -> MemoryState {
        let retrievability = self.retrievability(memory.stability, elapsed_days);
        let stability = if rating == Rating::Again {
            self.forget_stability(memory, retrievability)
        } else {
            self.recall_stability(memory, retrievability, rating)
        };
        MemoryState {
            stability,
            difficulty: self.next_difficulty(memory.difficulty, rating),
        }
    }

    /// Počáteční stabilita: přímo parametr w0 až w3 podle hodnocení.
    fn initial_stability(&self, rating: Rating) -> f64 {
        self.w[usize::from(rating.number() - 1)].max(0.1)
    }

    /// Počáteční obtížnost: D0(G) = w4 − e^(w5·(G − 1)) + 1, omezená na 1 až 10.
    fn initial_difficulty(&self, rating: Rating) -> f64 {
        let w = &self.w;
        (w[4] - (w[5] * (rating.value() - 1.0)).exp() + 1.0).clamp(1.0, 10.0)
    }

    /// Nová obtížnost: změna podle hodnocení s lineárním tlumením u horní meze
    /// a mírný návrat k počáteční obtížnosti hodnocení „Snadné“.
    fn next_difficulty(&self, difficulty: f64, rating: Rating) -> f64 {
        let w = &self.w;
        let delta = -w[6] * (rating.value() - 3.0);
        let damped = difficulty + (10.0 - difficulty) * delta / 9.0;
        let reverted = w[7] * self.initial_difficulty(Rating::Easy) + (1.0 - w[7]) * damped;
        reverted.clamp(1.0, 10.0)
    }

    /// Stabilita po úspěšném vybavení (Těžké, Dobré, Snadné).
    fn recall_stability(&self, memory: MemoryState, retrievability: f64, rating: Rating) -> f64 {
        let w = &self.w;
        let MemoryState {
            stability,
            difficulty,
        } = memory;
        let hard_penalty = if rating == Rating::Hard { w[15] } else { 1.0 };
        let easy_bonus = if rating == Rating::Easy { w[16] } else { 1.0 };
        stability
            * (1.0
                + w[8].exp()
                    * (11.0 - difficulty)
                    * stability.powf(-w[9])
                    * (((1.0 - retrievability) * w[10]).exp() - 1.0)
                    * hard_penalty
                    * easy_bonus)
    }

    /// Stabilita po zapomenutí (Znovu). Nesmí vyjít větší než krátkodobá stabilita
    /// po hodnocení „Znovu“.
    fn forget_stability(&self, memory: MemoryState, retrievability: f64) -> f64 {
        let w = &self.w;
        let MemoryState {
            stability,
            difficulty,
        } = memory;
        let long_term = w[11]
            * difficulty.powf(-w[12])
            * ((stability + 1.0).powf(w[13]) - 1.0)
            * ((1.0 - retrievability) * w[14]).exp();
        let short_term = stability / (w[17] * w[18]).exp();
        long_term.min(short_term)
    }
}

impl Default for Fsrs {
    /// Plánovač s výchozími parametry a výchozí cílovou retencí 0,9.
    fn default() -> Self {
        Self {
            w: DEFAULT_PARAMETERS,
            desired_retention: DEFAULT_DESIRED_RETENTION,
        }
    }
}

/// Studijní den pro daný místní čas.
///
/// Den začíná ve 4:00 ([`DAY_START_HOUR`]), takže učení po půlnoci patří
/// k předchozímu dni. Čas musí být už převedený na místní časové pásmo.
pub fn study_day(local_now: OffsetDateTime) -> Date {
    local_now
        .checked_sub(Duration::hours(DAY_START_HOUR))
        // Přetečení nastane jen v úplně prvních hodinách nejmenšího možného data.
        .map_or(local_now.date(), |shifted| shifted.date())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::{date, datetime};

    const START: Date = date!(2026 - 01 - 01);

    /// Porovnání s relativní tolerancí (referenční hodnoty jsou z Pythonu, `f64`).
    fn assert_close(actual: f64, expected: f64, what: &str) {
        let tolerance = 1e-9 * expected.abs().max(1.0);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{what}: vyšlo {actual}, očekáváno {expected}"
        );
    }

    /// Jeden krok scénáře: (den od začátku, hodnocení, stabilita, obtížnost, interval).
    type Step = (i64, u8, f64, f64, u32);

    /// Projde scénář a každý krok porovná s referenční hodnotou.
    fn check_scenario(retention: f64, steps: &[Step]) {
        let fsrs = Fsrs::new(retention).unwrap();
        let mut card: Option<CardState> = None;
        for (index, &(day, rating, stability, difficulty, interval)) in steps.iter().enumerate() {
            let today = START + Duration::days(day);
            let rating = Rating::from_number(rating).unwrap();
            let next = fsrs.review(card.as_ref(), rating, today);
            let what = format!("krok {index} (den {day}, {rating:?})");
            assert_close(
                next.memory.stability,
                stability,
                &format!("{what}: stabilita"),
            );
            assert_close(
                next.memory.difficulty,
                difficulty,
                &format!("{what}: obtížnost"),
            );
            assert_eq!(next.last_review, today, "{what}: poslední opakování");
            assert_eq!(
                (next.due - today).whole_days(),
                i64::from(interval),
                "{what}: interval"
            );
            assert_eq!(
                next.reps,
                u32::try_from(index + 1).unwrap(),
                "{what}: počet opakování"
            );
            card = Some(next);
        }
    }

    // Referenční hodnoty níže pocházejí z py-fsrs v4.1.2 (FSRS-5, výchozí parametry)
    // se Scheduler(learning_steps=(), relearning_steps=(), enable_fuzzing=False).
    // Opakování probíhá vždy v den splatnosti, pokud scénář neříká jinak.

    #[test]
    fn new_card_each_rating() {
        check_scenario(0.9, &[(0, 1, 0.40255, 7.1949, 1)]);
        check_scenario(0.9, &[(0, 2, 1.18385, 6.488305268471453, 1)]);
        check_scenario(0.9, &[(0, 3, 3.173, 5.282434422319005, 3)]);
        check_scenario(0.9, &[(0, 4, 15.69105, 3.2245015893713678, 16)]);
    }

    #[test]
    fn always_good_on_due_day() {
        check_scenario(
            0.9,
            &[
                (0, 3, 3.173, 5.282434422319005, 3),
                (3, 3, 10.73892584613159, 5.272967931287446, 11),
                (14, 3, 34.57762350060534, 5.263544986114632, 35),
                (49, 3, 100.74831285186508, 5.254165386489613, 101),
                (150, 3, 269.28383515131185, 5.244828933022869, 269),
                (419, 3, 669.309326136791, 5.2355354272420715, 669),
                (1088, 3, 1563.237654040241, 5.226284671587866, 1563),
            ],
        );
    }

    #[test]
    fn always_easy_reaches_max_interval() {
        check_scenario(
            0.9,
            &[
                (0, 4, 15.69105, 3.2245015893713678, 16),
                (16, 4, 150.2835611186658, 2.1301214599670124, 150),
                (166, 4, 1252.2233394307545, 1.0, 1252),
                (1418, 4, 9304.574709811886, 1.0, 9305),
                (10723, 4, 56424.00294926881, 1.0, 36500),
            ],
        );
    }

    #[test]
    fn always_hard_on_due_day() {
        check_scenario(
            0.9,
            &[
                (0, 2, 1.18385, 6.488305268471453, 1),
                (1, 2, 1.7073629306504086, 7.040501547390819, 2),
                (3, 2, 2.558557339977389, 7.500966847928992, 3),
                (6, 2, 3.6336340003780676, 7.884939542230092, 4),
                (10, 2, 4.866219628072568, 8.205126600400446, 5),
                (15, 2, 6.211094994271307, 8.472124107622648, 6),
            ],
        );
    }

    #[test]
    fn lapse_and_relearning() {
        // Ve dni 49 „Znovu“ a zopakování týž den s hodnocením „Dobré“.
        check_scenario(
            0.9,
            &[
                (0, 3, 3.173, 5.282434422319005, 3),
                (3, 3, 10.73892584613159, 5.272967931287446, 11),
                (14, 3, 34.57762350060534, 5.263544986114632, 35),
                (49, 1, 3.818174311547718, 6.784232087673549, 4),
                (49, 3, 5.375115888647206, 6.767857327381359, 5),
                (54, 3, 14.15413535588553, 6.751557890986513, 14),
                (68, 3, 36.00965296181716, 6.735333431999083, 36),
            ],
        );
    }

    #[test]
    fn again_repeated_same_day() {
        check_scenario(
            0.9,
            &[
                (0, 1, 0.40255, 7.1949, 1),
                (0, 1, 0.20168903241409677, 8.082797017759107, 1),
                (0, 3, 0.28393201416083236, 8.060448858788522, 1),
                (1, 3, 1.6644705994929343, 8.038203501349201, 2),
            ],
        );
    }

    #[test]
    fn late_reviews() {
        check_scenario(
            0.9,
            &[
                (0, 3, 3.173, 5.282434422319005, 3),
                (10, 3, 23.895875500557327, 5.272967931287446, 24),
                (40, 2, 37.268969540220596, 6.0270563403407795, 37),
                (140, 1, 5.053298775449699, 7.297586215315591, 5),
                (141, 4, 10.056615760980412, 6.842355502954714, 10),
            ],
        );
    }

    #[test]
    fn lower_desired_retention() {
        check_scenario(
            0.8,
            &[
                (0, 3, 3.173, 5.282434422319005, 8),
                (8, 3, 20.622575360233377, 5.272967931287446, 49),
                (57, 3, 107.32964892542199, 5.263544986114632, 257),
                (314, 1, 7.635708595027789, 6.784232087673549, 18),
                (332, 3, 34.07229866448929, 6.767857327381359, 82),
            ],
        );
    }

    #[test]
    fn higher_desired_retention() {
        check_scenario(
            0.97,
            &[
                (0, 3, 3.173, 5.282434422319005, 1),
                (1, 3, 5.869141822641378, 5.272967931287446, 2),
                (3, 4, 20.833738801375222, 4.500033631888484, 6),
                (9, 2, 24.24592291297469, 5.382522072262206, 6),
            ],
        );
    }

    #[test]
    fn retrievability_matches_reference() {
        let fsrs = Fsrs::default();
        let cases = [
            (1.0, 1, 0.9),
            (3.173, 3, 0.9046982108893272),
            (10.0, 0, 1.0),
            (10.0, 5, 0.946058996209746),
            (10.0, 30, 0.7661308776828738),
            (0.5, 7, 0.4831453143855171),
        ];
        for (stability, days, expected) in cases {
            let what = format!("R(S = {stability}, t = {days})");
            assert_close(fsrs.retrievability(stability, days), expected, &what);
        }
    }

    #[test]
    fn next_interval_matches_reference() {
        let fsrs = Fsrs::default();
        // 1,5 i 2,5 se zaokrouhlí k sudé, tedy na 2.
        let cases = [
            (0.01, 1),
            (0.40255, 1),
            (1.49, 1),
            (1.5, 2),
            (2.5, 2),
            (15.69105, 16),
            (100.4, 100),
            (50000.0, 36500),
        ];
        for (stability, expected) in cases {
            assert_eq!(fsrs.next_interval(stability), expected, "S = {stability}");
        }

        let fsrs = Fsrs::new(0.8).unwrap();
        assert_eq!(fsrs.next_interval(1.0), 2);
        assert_eq!(fsrs.next_interval(10.0), 24);
    }

    #[test]
    fn desired_retention_is_validated() {
        assert!(Fsrs::new(0.70).is_ok());
        assert!(Fsrs::new(0.99).is_ok());
        for bad in [0.69, 0.995, 0.0, 1.0, -0.9, f64::NAN, f64::INFINITY] {
            assert!(Fsrs::new(bad).is_err(), "retence {bad} měla být odmítnuta");
        }
        assert_eq!(Fsrs::default(), Fsrs::new(0.9).unwrap());
    }

    #[test]
    fn lapses_count_only_again_on_later_day() {
        let fsrs = Fsrs::default();
        let card = fsrs.review(None, Rating::Again, START);
        assert_eq!(
            card.lapses, 0,
            "první hodnocení nové kartičky není zapomenutí"
        );
        let card = fsrs.review(Some(&card), Rating::Again, START);
        assert_eq!(card.lapses, 0, "zopakování týž den není zapomenutí");
        let card = fsrs.review(Some(&card), Rating::Good, START + Duration::days(1));
        let card = fsrs.review(Some(&card), Rating::Again, START + Duration::days(3));
        assert_eq!(card.lapses, 1);
        assert_eq!(card.reps, 4);
    }

    #[test]
    fn review_before_last_review_counts_as_same_day() {
        let fsrs = Fsrs::default();
        let card = fsrs.review(None, Rating::Good, START);
        let earlier = START - Duration::days(2);
        let next = fsrs.review(Some(&card), Rating::Good, earlier);
        let same_day = fsrs.review(Some(&card), Rating::Good, START);
        assert_eq!(next.memory, same_day.memory);
        assert_eq!(next.last_review, earlier);
    }

    #[test]
    fn due_date_saturates_near_max_date() {
        let fsrs = Fsrs::default();
        let card = fsrs.review(None, Rating::Easy, date!(9999 - 12 - 30));
        assert_eq!(card.due, Date::MAX);
    }

    #[test]
    fn study_day_starts_at_four() {
        assert_eq!(
            study_day(datetime!(2026-11-02 03:59:59 +01:00)),
            date!(2026 - 11 - 01)
        );
        assert_eq!(
            study_day(datetime!(2026-11-02 04:00:00 +01:00)),
            date!(2026 - 11 - 02)
        );
        assert_eq!(
            study_day(datetime!(2026-11-02 23:59:59 +01:00)),
            date!(2026 - 11 - 02)
        );
        assert_eq!(
            study_day(datetime!(2026-11-03 00:30:00 +01:00)),
            date!(2026 - 11 - 02)
        );
        // Rozhoduje místní čas, ne UTC.
        assert_eq!(
            study_day(datetime!(2026-07-01 04:30:00 +02:00)),
            date!(2026 - 07 - 01)
        );
    }

    #[test]
    fn rating_numbers_and_labels() {
        for (index, rating) in Rating::ALL.into_iter().enumerate() {
            let number = u8::try_from(index + 1).unwrap();
            assert_eq!(rating.number(), number);
            assert_eq!(Rating::from_number(number), Some(rating));
        }
        assert_eq!(Rating::from_number(0), None);
        assert_eq!(Rating::from_number(5), None);
        assert_eq!(Rating::Again.label(), "Znovu");
        assert_eq!(Rating::Easy.label(), "Snadné");
    }
}
