//! Jednotlivé obrazovky aplikace.

pub mod finished;
pub mod home;
pub mod review;

use eframe::egui::{Color32, Key};
use vietnamska_slovicka::fsrs::Rating;

/// Barva tlačítka hodnocení. Sytá, aby se bílý text dal číst ve světlém i tmavém režimu.
pub fn rating_color(rating: Rating) -> Color32 {
    match rating {
        Rating::Again => Color32::from_rgb(185, 55, 55),
        Rating::Hard => Color32::from_rgb(190, 120, 30),
        Rating::Good => Color32::from_rgb(50, 140, 75),
        Rating::Easy => Color32::from_rgb(45, 105, 185),
    }
}

/// Klávesa hodnocení (1–4).
pub fn rating_key(rating: Rating) -> Key {
    match rating {
        Rating::Again => Key::Num1,
        Rating::Hard => Key::Num2,
        Rating::Good => Key::Num3,
        Rating::Easy => Key::Num4,
    }
}
