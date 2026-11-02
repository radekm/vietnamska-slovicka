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
                    for (tone, (example, meaning)) in Tone::ALL.into_iter().zip(EXAMPLES) {
                        ui.label(tone.name());
                        ui.label(RichText::new(tone.mark()).size(20.0).family(fonts::bold()));
                        ui.label(tone.description(false));
                        ui.label(RichText::new(tone.chao(false)).size(18.0));
                        ui.label(format!("{example} – {meaning}"));
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
            ui.label(RichText::new("Zavřené slabiky (končí na -p, -t, -c, -ch)").family(fonts::bold()));
            egui::Grid::new("checked_tones")
                .striped(true)
                .spacing([18.0, 6.0])
                .show(ui, |ui| {
                    for (tone, example, meaning) in
                        [(Tone::Sac, "mát", "chladný"), (Tone::Nang, "mạt", "veš")]
                    {
                        ui.label(tone.name());
                        ui.label(RichText::new(tone.mark()).size(20.0).family(fonts::bold()));
                        ui.label(tone.description(true));
                        ui.label(RichText::new(tone.chao(true)).size(18.0));
                        ui.label(format!("{example} – {meaning}"));
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Popis tónu je v závorce za každou slabikou výslovnosti, např. „kam (klesá hluboko)“.\nOstatní tóny se v zavřených slabikách nevyskytují. Tóny podle Kirby (2011).\nOkno přepíná tlačítko „?“ nebo klávesa F1.",
                )
                .weak()
                .size(13.0),
            );
        });
}
