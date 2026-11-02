//! Vietnamská slovíčka: učení slovíček metodou kartiček s rozloženým opakováním.

mod app;
mod fonts;
mod logging;
mod paths;

use anyhow::Result;
use eframe::egui;

fn main() -> Result<()> {
    let data_dir = paths::data_dir()?;
    // Strážce logování musí žít až do konce `main`, jinak by se ztratily poslední záznamy.
    let _log_guard = logging::init(&paths::log_dir(&data_dir))?;

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        data_dir = %data_dir.display(),
        "aplikace spuštěna"
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vietnamská slovíčka")
            .with_inner_size([900.0, 640.0])
            .with_min_inner_size([480.0, 360.0]),
        ..Default::default()
    };

    let result = eframe::run_native(
        "vietnamska-slovicka",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    );

    match &result {
        Ok(()) => tracing::info!("aplikace ukončena"),
        Err(err) => tracing::error!(error = %err, "aplikace skončila chybou"),
    }
    // Chyba z eframe není `Send + Sync`, proto ji převádíme na text.
    result.map_err(|err| anyhow::anyhow!("chyba okna aplikace: {err}"))
}
