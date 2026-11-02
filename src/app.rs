//! Stav a vykreslování aplikace.
//!
//! V etapě 1 je to jen ukázková obrazovka, na které jde ověřit,
//! že se správně vykresluje vietnamština, čeština a IPA.

use eframe::egui::{self, RichText};

use crate::fonts;

/// Ukázky textu: (popisek, text).
const SAMPLES: &[(&str, &str)] = &[
    ("Vietnamsky", "Xin chào! Cảm ơn. Một, hai, ba."),
    ("IPA", "sin˧˧ t͡ɕaːw˨˩ · kaːm˧˩˧ ʔəːn˧˧ · mot̚˨˩ˀ"),
    ("IPA (homonymum)", "ɗɨəŋ˨˩ (đường)"),
    ("Tóny", "a à á ả ã ạ · ˧˧ ˨˩ ˧˥ ˧˩˧ ˧ˀ˥ ˨˩ˀ"),
    ("Česky", "Příliš žluťoučký kůň úpěl ďábelské ódy."),
    ("Směr", "CS → VI · VI → CS"),
    ("Abeceda", "ăắằẳẵặ âấầẩẫậ đ êếềểễệ ôốồổỗộ ơớờởỡợ ưứừửữự"),
    (
        "Velká písmena",
        "ĂẮẰẲẴẶ ÂẤẦẨẪẬ Đ ÊẾỀỂỄỆ ÔỐỒỔỖỘ ƠỚỜỞỠỢ ƯỨỪỬỮỰ",
    ),
];

/// Hlavní struktura aplikace.
pub struct App;

impl App {
    /// Vytvoří aplikaci a nastaví fonty.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts::install(&cc.egui_ctx);
        App
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label(
                RichText::new("Vietnamská slovíčka")
                    .size(32.0)
                    .family(fonts::bold()),
            );
            ui.label("Ukázka vykreslení fontu Noto Sans");
            ui.add_space(16.0);

            egui::Grid::new("samples")
                .num_columns(2)
                .spacing([24.0, 12.0])
                .show(ui, |ui| {
                    for (caption, text) in SAMPLES {
                        ui.label(RichText::new(*caption).weak());
                        ui.label(RichText::new(*text).size(24.0));
                        ui.end_row();
                    }
                });
        });
    }
}
