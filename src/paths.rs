//! Umístění datové složky a složky s logy.
//!
//! Obě složky jsou v pracovním adresáři, ze kterého se aplikace spouští:
//! - `data/` obsahuje postup učení, historii a nastavení,
//! - `logs/` obsahuje JSON logy.
//!
//! Složka se slovníčky (lekcemi) je jinde a vybírá ji uživatel.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Název datové složky v pracovním adresáři.
const DATA_DIR_NAME: &str = "data";
/// Název složky s logy v pracovním adresáři.
const LOG_DIR_NAME: &str = "logs";

/// Složky, se kterými aplikace pracuje.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDirs {
    /// Datová složka (postup učení, historie, nastavení).
    pub data: PathBuf,
    /// Složka s logy.
    pub logs: PathBuf,
}

impl AppDirs {
    /// Složky v aktuálním pracovním adresáři. Nic nevytváří.
    pub fn in_working_dir() -> Result<Self> {
        let cwd = std::env::current_dir().context("nelze zjistit pracovní adresář")?;
        Ok(Self::in_dir(&cwd))
    }

    /// Složky uvnitř zadaného adresáře. Nic nevytváří.
    pub fn in_dir(base: &Path) -> Self {
        Self {
            data: base.join(DATA_DIR_NAME),
            logs: base.join(LOG_DIR_NAME),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirs_are_inside_base() {
        let dirs = AppDirs::in_dir(Path::new("/home/radek/slovicka"));
        assert_eq!(dirs.data, PathBuf::from("/home/radek/slovicka/data"));
        assert_eq!(dirs.logs, PathBuf::from("/home/radek/slovicka/logs"));
    }

    #[test]
    fn working_dir_is_absolute() {
        let dirs = AppDirs::in_working_dir().unwrap();
        assert!(dirs.data.is_absolute());
        assert!(dirs.logs.is_absolute());
    }
}
