//! Legenda tónů: plovoucí okno, které se přepíná tlačítkem „?“ nebo klávesou F1.

use eframe::egui::{self, RichText};

use vietnamska_slovicka::phonetics::Tone;

use crate::fonts;

/// Příklady k tónům v pořadí [`Tone::ALL`]: slabika a její význam.
const EXAMPLES: [(&str, &str); 6] = [
    ("ma", "duch"),
    ("mà", "ale"),
    ("má", "tvář"),
    ("mả", "hrob"),
    ("mã", "kód"),
    ("mạ", "sazenice rýže"),
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
                    let open_syllables = Tone::ALL
                        .into_iter()
                        .zip(EXAMPLES)
                        .map(|(tone, (example, meaning))| (tone, false, example, meaning));
                    for (tone, checked, example, meaning) in open_syllables {
                        tone_row(ui, tone, checked, example, meaning);
                    }
                    // Jedna mřížka, aby se sloupce zarovnaly s tóny v otevřených slabikách.
                    ui.label(RichText::new("zavřené:").family(fonts::bold()));
                    ui.end_row();
                    tone_row(ui, Tone::Sac, true, "mát", "chladný");
                    tone_row(ui, Tone::Nang, true, "mạt", "veš");
                });
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Popis tónu je v závorce za každou slabikou výslovnosti, např. „kam (klesá hluboko)“.\nZavřená slabika končí na -p, -t, -c nebo -ch, jiné tóny v ní nejsou. Tóny podle Kirby (2011).\nOkno přepíná tlačítko „?“ nebo klávesa F1.",
                )
                .weak()
                .size(13.0),
            );
        });
}

/// Jeden řádek tabulky tónů.
fn tone_row(ui: &mut egui::Ui, tone: Tone, checked: bool, example: &str, meaning: &str) {
    ui.label(tone.name());
    ui.label(RichText::new(tone.mark()).size(20.0).family(fonts::bold()));
    ui.label(tone.description(checked));
    ui.label(RichText::new(tone.chao(checked)).size(18.0));
    ui.label(format!("{example} – {meaning}"));
    ui.end_row();
}
