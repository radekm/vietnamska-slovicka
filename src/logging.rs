//! Strukturované logování do JSON souborů s denní rotací.

use std::path::Path;

use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// Kolik denních log souborů se drží. Starší se mažou automaticky.
const MAX_LOG_FILES: usize = 7;

/// Zapne logování do složky `log_dir`.
///
/// Vrácený [`WorkerGuard`] musí žít až do konce programu. Při jeho zrušení
/// se zapíšou zbylé záznamy z vyrovnávací paměti.
pub fn init(log_dir: &Path) -> Result<WorkerGuard> {
    std::fs::create_dir_all(log_dir)
        .with_context(|| format!("nelze vytvořit složku pro logy {}", log_dir.display()))?;

    // Soubory se jmenují např. `vietnamska-slovicka.2026-11-02.json`.
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("vietnamska-slovicka")
        .filename_suffix("json")
        .max_log_files(MAX_LOG_FILES)
        .build(log_dir)
        .context("nelze otevřít log soubor")?;

    // Zápis běží ve vlastním vlákně, aby logování nebrzdilo vykreslování.
    let (writer, guard) = tracing_appender::non_blocking(appender);

    tracing_subscriber::fmt()
        .json()
        .with_writer(writer)
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .try_init()
        // Chyba z `try_init` není `Send + Sync`, proto ji převádíme na text.
        .map_err(|err| anyhow::anyhow!("nelze zapnout logování: {err}"))?;

    Ok(guard)
}
