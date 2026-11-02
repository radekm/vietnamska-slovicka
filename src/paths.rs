//! Umístění datové složky aplikace podle operačního systému.
//!
//! Do datové složky se ukládá postup učení, nastavení a logy.
//! Složka se slovníčky (lekcemi) je jinde a vybírá ji uživatel.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};

/// Název podsložky aplikace uvnitř systémové složky pro data.
const APP_DIR_NAME: &str = "vietnamska-slovicka";

/// Operační systémy, které rozlišujeme při hledání datové složky.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    Macos,
    Windows,
}

impl Os {
    /// Systém, pro který je program přeložený.
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            // Ostatní unixové systémy se chovají jako Linux (konvence XDG).
            Os::Linux
        }
    }
}

/// Vrátí datovou složku aplikace pro aktuální systém. Složku nevytváří.
pub fn data_dir() -> Result<PathBuf> {
    data_dir_for(Os::current(), |name| {
        std::env::var_os(name).map(PathBuf::from)
    })
}

/// Podsložka datové složky, do které se ukládají logy.
pub fn log_dir(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("logy")
}

/// Vrátí datovou složku pro zadaný systém.
///
/// Proměnné prostředí se čtou přes `env`, aby šla funkce otestovat
/// bez změny skutečného prostředí procesu.
pub fn data_dir_for(os: Os, env: impl Fn(&str) -> Option<PathBuf>) -> Result<PathBuf> {
    // Prázdná proměnná se bere, jako by nebyla nastavená.
    let var = |name: &str| env(name).filter(|path| !path.as_os_str().is_empty());

    let base = match os {
        Os::Linux => match var("XDG_DATA_HOME") {
            // Podle specifikace XDG se relativní cesta ignoruje.
            Some(path) if path.is_absolute() => path,
            _ => var("HOME")
                .context("není nastavena proměnná HOME")?
                .join(".local")
                .join("share"),
        },
        Os::Macos => var("HOME")
            .context("není nastavena proměnná HOME")?
            .join("Library")
            .join("Application Support"),
        Os::Windows => var("APPDATA").context("není nastavena proměnná APPDATA")?,
    };

    if !base.is_absolute() {
        bail!("cesta k datové složce není absolutní: {}", base.display());
    }
    Ok(base.join(APP_DIR_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vyrobí funkci prostředí z pevného seznamu proměnných.
    fn env_from(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<PathBuf> + use<> {
        let vars: Vec<(String, String)> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |name| {
            vars.iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| PathBuf::from(v))
        }
    }

    #[test]
    fn linux_uses_xdg_data_home() {
        let env = env_from(&[("XDG_DATA_HOME", "/data"), ("HOME", "/home/radek")]);
        assert_eq!(
            data_dir_for(Os::Linux, env).unwrap(),
            PathBuf::from("/data/vietnamska-slovicka")
        );
    }

    #[test]
    fn linux_falls_back_to_home() {
        let env = env_from(&[("HOME", "/home/radek")]);
        assert_eq!(
            data_dir_for(Os::Linux, env).unwrap(),
            PathBuf::from("/home/radek/.local/share/vietnamska-slovicka")
        );
    }

    #[test]
    fn linux_ignores_empty_and_relative_xdg_data_home() {
        for xdg in ["", "relativni/cesta"] {
            let env = env_from(&[("XDG_DATA_HOME", xdg), ("HOME", "/home/radek")]);
            assert_eq!(
                data_dir_for(Os::Linux, env).unwrap(),
                PathBuf::from("/home/radek/.local/share/vietnamska-slovicka")
            );
        }
    }

    #[test]
    fn macos_uses_application_support() {
        let env = env_from(&[("HOME", "/Users/radek")]);
        assert_eq!(
            data_dir_for(Os::Macos, env).unwrap(),
            PathBuf::from("/Users/radek/Library/Application Support/vietnamska-slovicka")
        );
    }

    #[test]
    fn missing_home_is_error() {
        assert!(data_dir_for(Os::Linux, env_from(&[])).is_err());
        assert!(data_dir_for(Os::Macos, env_from(&[])).is_err());
    }

    #[test]
    fn missing_appdata_is_error() {
        assert!(data_dir_for(Os::Windows, env_from(&[])).is_err());
    }
}
