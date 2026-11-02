//! Legenda tónů: plovoucí okno, které se přepíná tlačítkem „?“ nebo klávesou F1.

use eframe::egui::{self, RichText};

use crate::fonts;

/// Jeden tón v legendě.
struct Tone {
    /// Vietnamský název tónu.
    name: &'static str,
    /// Značka nad samohláskou.
    mark: &'static str,
    /// Popis v závorce v českém popisu výslovnosti.
    description: &'static str,
    /// Chaova tónová písmena (viz PLAN.md, „Konvence IPA“).
    chao: &'static str,
    /// Příkladová slabika s tímto tónem.
    example: &'static str,
    /// Význam příkladu.
    meaning: &'static str,
}

/// Tóny podle PLAN.md. Hodnoty tónů se ještě ověří v etapě 8, pak je potřeba
/// upravit i tuto tabulku.
const TONES: [Tone; 6] = [
    Tone {
        name: "ngang",
        mark: "a",
        description: "rovně",
        chao: "˧˧",
        example: "ma",
        meaning: "duch",
    },
    Tone {
        name: "huyền",
        mark: "à",
        description: "nízko klesá",
        chao: "˨˩",
        example: "mà",
        meaning: "ale",
    },
    Tone {
        name: "sắc",
        mark: "á",
        description: "stoupá",
        chao: "˧˥",
        example: "má",
        meaning: "tvář",
    },
    Tone {
        name: "hỏi",
        mark: "ả",
        description: "klesá a stoupá",
        chao: "˧˩˧",
        example: "mả",
        meaning: "hrob",
    },
    Tone {
        name: "ngã",
        mark: "ã",
        description: "stoupá se zlomem",
        chao: "˧ˀ˥",
        example: "mã",
        meaning: "kůň",
    },
    Tone {
        name: "nặng",
        mark: "ạ",
        description: "krátce klesá, ráz",
        chao: "˨˩ˀ",
        example: "mạ",
        meaning: "sazenice rýže",
    },
];

/// Vykreslí okno s legendou, pokud je otevřené. Křížek okno zavře.
pub fn show(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Legenda tónů")
        .open(open)
        .collapsible(false)
        .resizable(false)
        // Na začátku vpravo nahoře pod lištou, dá se přesunout jinam.
        .pivot(egui::Align2::RIGHT_TOP)
        .default_pos([ctx.content_rect().right() - 16.0, 48.0])
        .show(ctx, |ui| {
            egui::Grid::new("tones")
                .striped(true)
                .spacing([18.0, 6.0])
                .show(ui, |ui| {
                    for header in ["Tón", "Značka", "Popis", "IPA", "Příklad"] {
                        ui.label(RichText::new(header).family(fonts::bold()));
                    }
                    ui.end_row();
                    for tone in &TONES {
                        ui.label(tone.name);
                        ui.label(RichText::new(tone.mark).size(20.0).family(fonts::bold()));
                        ui.label(tone.description);
                        ui.label(RichText::new(tone.chao).size(18.0));
                        ui.label(format!("{} – {}", tone.example, tone.meaning));
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Popis tónu je v závorce za každou slabikou výslovnosti, např. „kam (klesá a stoupá)“.\nOkno přepíná tlačítko „?“ nebo klávesa F1.",
                )
                .weak()
                .size(13.0),
            );
        });
}
