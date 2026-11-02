//! Nastavení fontů: Noto Sans jako hlavní font, výchozí fonty egui jako záložní.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

/// Název fontu pro běžný text.
const REGULAR: &str = "NotoSans-Regular";
/// Název fontu pro tučný text.
const BOLD: &str = "NotoSans-Bold";

/// Rodina fontů pro tučný text. Použití: `RichText::new(..).family(fonts::bold())`.
pub fn bold() -> FontFamily {
    FontFamily::Name(BOLD.into())
}

/// Sestaví definice fontů aplikace.
///
/// Noto Sans pokrývá vietnamštinu, češtinu i znaky IPA. Výchozí fonty egui
/// zůstávají v seznamu za ním jako záložní, pro symboly, které Noto Sans nemá
/// (např. šipka →, ikonky ovládacích prvků).
pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        REGULAR.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/NotoSans-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        BOLD.to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/NotoSans-Bold.ttf"
        ))),
    );

    // Záložní fonty pro proporcionální písmo (výchozí seznam egui).
    let fallbacks = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    // Běžný text: Noto Sans na první místo, výchozí fonty za ním.
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, REGULAR.to_owned());

    // Neproporcionální písmo: necháme výchozí, Noto Sans přidáme jako záložní
    // pro znaky, které výchozí neproporcionální font nemá.
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(REGULAR.to_owned());

    // Tučný text: Noto Sans Bold, pak stejné záložní fonty jako u běžného textu.
    let mut bold_family = vec![BOLD.to_owned()];
    bold_family.extend(fallbacks);
    fonts.families.insert(bold(), bold_family);

    fonts
}

/// Nastaví fonty aplikace do kontextu egui.
pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(definitions());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noto_sans_is_primary_font() {
        let fonts = definitions();
        assert_eq!(fonts.families[&FontFamily::Proportional][0], REGULAR);
        assert_eq!(fonts.families[&bold()][0], BOLD);
    }

    #[test]
    fn default_fonts_remain_as_fallback() {
        let fonts = definitions();
        let proportional = &fonts.families[&FontFamily::Proportional];
        assert!(proportional.len() > 1, "chybí záložní fonty");
        assert_eq!(proportional[1..], fonts.families[&bold()][1..]);
    }
}
