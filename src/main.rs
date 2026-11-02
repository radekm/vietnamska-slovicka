//! Vietnamská slovíčka: učení slovíček metodou kartiček s rozloženým opakováním.

mod app;
mod fonts;
mod logging;

use anyhow::Result;
use eframe::egui;
use time::{OffsetDateTime, UtcOffset};
use vietnamska_slovicka::{fsrs, paths};

fn main() -> Result<()> {
    // Místní časový posun musí být zjištěn jako úplně první věc, dokud program běží
    // v jednom vlákně. Ve vícevláknovém programu ho `time` na Linuxu nevrátí.
    // Logování i eframe spouštějí další vlákna. Viz PLAN.md, oddíl „Místní čas“.
    let local_offset = UtcOffset::current_local_offset();

    let dirs = paths::AppDirs::in_working_dir()?;
    // Strážce logování musí žít až do konce `main`, jinak by se ztratily poslední záznamy.
    // Složku `logs/` vytvoří `logging::init`.
    let _log_guard = logging::init(&dirs.logs)?;

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        data_dir = %dirs.data.display(),
        log_dir = %dirs.logs.display(),
        "aplikace spuštěna"
    );

    let local_offset = match local_offset {
        Ok(offset) => offset,
        Err(err) => {
            tracing::warn!(error = %err, "nelze zjistit místní časový posun, použije se UTC");
            UtcOffset::UTC
        }
    };
    let local_now = OffsetDateTime::now_utc().to_offset(local_offset);
    tracing::info!(
        local_offset = %local_offset,
        local_time = %local_now,
        study_day = %fsrs::study_day(local_now),
        "místní čas"
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
