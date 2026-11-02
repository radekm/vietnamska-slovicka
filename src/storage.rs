//! Bezpečné čtení a zápis datových souborů a převody data a času mezi `time` a TOML.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use time::{Date, Month, OffsetDateTime, Time, UtcOffset};
use toml::value::Datetime;

/// Verze formátu datových souborů, kterou aplikace zapisuje a umí číst.
pub const FORMAT_VERSION: u32 = 1;

/// Zkontroluje verzi formátu načteného souboru.
pub fn check_version(version: u32) -> Result<()> {
    ensure!(
        version == FORMAT_VERSION,
        "neznámá verze formátu {version} (aplikace umí verzi {FORMAT_VERSION}), \
         soubor možná vytvořila novější verze aplikace"
    );
    Ok(())
}

/// Bezpečně zapíše `contents` do souboru `path`.
///
/// Zapisuje se do dočasného souboru ve stejné složce, ten se uloží na disk (`sync_all`)
/// a pak se přejmenuje na cílový název. Při pádu uprostřed zápisu tak zůstane buď starý,
/// nebo nový obsah, nikdy poloviční. Chybějící složka se vytvoří.
pub fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let dir = path
        .parent()
        .with_context(|| format!("soubor {} nemá nadřazenou složku", path.display()))?;
    std::fs::create_dir_all(dir)
        .with_context(|| format!("nelze vytvořit složku {}", dir.display()))?;

    let tmp = temp_path(path)?;
    let result = write_and_sync(&tmp, contents).and_then(|()| {
        std::fs::rename(&tmp, path)
            .with_context(|| format!("nelze přejmenovat {} na {}", tmp.display(), path.display()))
    });
    if result.is_err() {
        // Nepovedený pokus po sobě uklidíme, chyba úklidu už nic nemění.
        let _ = std::fs::remove_file(&tmp);
        return result;
    }

    // Uložení složky zajistí, že přejmenování přežije výpadek proudu. Na některých
    // systémech složku jako soubor otevřít nejde, proto je to jen pokus navíc.
    if let Ok(dir_file) = std::fs::File::open(dir) {
        let _ = dir_file.sync_all();
    }
    Ok(())
}

/// Název dočasného souboru vedle cílového, např. `.postup.toml.tmp`.
fn temp_path(path: &Path) -> Result<PathBuf> {
    let name = path
        .file_name()
        .with_context(|| format!("cesta {} nemá název souboru", path.display()))?;
    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(name);
    tmp_name.push(".tmp");
    Ok(path.with_file_name(tmp_name))
}

fn write_and_sync(path: &Path, contents: &str) -> Result<()> {
    let mut file = std::fs::File::create(path)
        .with_context(|| format!("nelze vytvořit soubor {}", path.display()))?;
    file.write_all(contents.as_bytes())
        .with_context(|| format!("nelze zapsat do souboru {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("nelze uložit soubor {} na disk", path.display()))
}

/// Přečte textový soubor. Chybějící soubor není chyba, vrací `None`.
pub fn read_optional(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("nelze přečíst soubor {}", path.display())),
    }
}

/// Datum jako hodnota TOML (např. `2026-11-02`).
pub fn date_to_toml(date: Date) -> Datetime {
    Datetime {
        date: Some(toml_date(date)),
        time: None,
        offset: None,
    }
}

/// Datum z hodnoty TOML. Hodnota musí být čisté datum bez času.
pub fn date_from_toml(value: &Datetime) -> Result<Date> {
    match value {
        Datetime {
            date: Some(date),
            time: None,
            offset: None,
        } => date_from_parts(date),
        _ => bail!("očekáváno datum (např. 2026-11-02), nalezeno {value}"),
    }
}

/// Čas s časovým posunem jako hodnota TOML (např. `2026-11-02T16:53:12+01:00`).
///
/// Ukládá se s přesností na sekundy. Posun se zaokrouhlí dolů na celé minuty, jemnější
/// TOML neumí (skutečná časová pásma ho nemají).
pub fn datetime_to_toml(datetime: OffsetDateTime) -> Datetime {
    let offset_minutes = datetime.offset().whole_minutes();
    Datetime {
        date: Some(toml_date(datetime.date())),
        time: Some(toml::value::Time {
            hour: datetime.hour(),
            minute: datetime.minute(),
            second: Some(datetime.second()),
            nanosecond: None,
        }),
        offset: Some(toml::value::Offset::Custom {
            minutes: offset_minutes,
        }),
    }
}

/// Čas s časovým posunem z hodnoty TOML. Hodnota musí mít datum, čas i posun.
pub fn datetime_from_toml(value: &Datetime) -> Result<OffsetDateTime> {
    let (Some(date), Some(time), Some(offset)) = (&value.date, &value.time, &value.offset) else {
        bail!("očekáván čas s časovým posunem (např. 2026-11-02T16:53:12+01:00), nalezeno {value}");
    };
    let date = date_from_parts(date)?;
    let time = Time::from_hms_nano(
        time.hour,
        time.minute,
        time.second.unwrap_or(0),
        time.nanosecond.unwrap_or(0),
    )
    .with_context(|| format!("neplatný čas {value}"))?;
    let offset_seconds = match offset {
        toml::value::Offset::Z => 0,
        toml::value::Offset::Custom { minutes } => i32::from(*minutes) * 60,
    };
    let offset = UtcOffset::from_whole_seconds(offset_seconds)
        .with_context(|| format!("neplatný časový posun {value}"))?;
    Ok(date.with_time(time).assume_offset(offset))
}

fn toml_date(date: Date) -> toml::value::Date {
    toml::value::Date {
        // Rok v TOML musí mít čtyři číslice. Mimo rozsah 0–9999 by `time` bez feature
        // `large-dates` datum ani nevytvořil, ale převod zůstává bez paniky.
        year: u16::try_from(date.year()).unwrap_or(0),
        month: u8::from(date.month()),
        day: date.day(),
    }
}

fn date_from_parts(date: &toml::value::Date) -> Result<Date> {
    let month =
        Month::try_from(date.month).with_context(|| format!("neplatný měsíc v datu {date}"))?;
    Date::from_calendar_date(i32::from(date.year), month, date.day)
        .with_context(|| format!("neplatné datum {date}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;
    use time::macros::{date, datetime};

    #[test]
    fn write_atomic_creates_dir_and_replaces_file() {
        let dir = TempDir::new("storage-write");
        let path = dir.join("data/postup.toml");
        write_atomic(&path, "první").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "první");
        write_atomic(&path, "druhý").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "druhý");
        // Dočasný soubor po sobě nenechá.
        let names: Vec<_> = std::fs::read_dir(dir.join("data"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["postup.toml"]);
    }

    #[test]
    fn read_optional_missing_file_is_none() {
        let dir = TempDir::new("storage-read");
        assert_eq!(read_optional(&dir.join("neni.toml")).unwrap(), None);
        dir.write("je.toml", "obsah");
        assert_eq!(
            read_optional(&dir.join("je.toml")).unwrap().as_deref(),
            Some("obsah")
        );
        // Složka místo souboru je skutečná chyba.
        assert!(read_optional(dir.path()).is_err());
    }

    #[test]
    fn date_round_trip() {
        let value = date_to_toml(date!(2026 - 11 - 02));
        assert_eq!(value.to_string(), "2026-11-02");
        assert_eq!(date_from_toml(&value).unwrap(), date!(2026 - 11 - 02));
    }

    #[test]
    fn date_rejects_datetime_and_invalid_date() {
        let datetime: Datetime = "2026-11-02T10:00:00Z".parse().unwrap();
        assert!(date_from_toml(&datetime).is_err());
        let time_only: Datetime = "10:00:00".parse().unwrap();
        assert!(date_from_toml(&time_only).is_err());
    }

    #[test]
    fn datetime_round_trip() {
        let original = datetime!(2026-11-02 16:53:12 +01:00);
        let value = datetime_to_toml(original);
        assert_eq!(value.to_string(), "2026-11-02T16:53:12+01:00");
        assert_eq!(datetime_from_toml(&value).unwrap(), original);

        let negative = datetime!(2026-07-01 08:00:00 -05:30);
        assert_eq!(
            datetime_from_toml(&datetime_to_toml(negative)).unwrap(),
            negative
        );
    }

    #[test]
    fn datetime_drops_subseconds() {
        let value = datetime_to_toml(datetime!(2026-11-02 16:53:12.75 +01:00));
        assert_eq!(value.to_string(), "2026-11-02T16:53:12+01:00");
    }

    #[test]
    fn datetime_accepts_zulu_and_requires_offset() {
        let zulu: Datetime = "2026-11-02T15:53:12Z".parse().unwrap();
        assert_eq!(
            datetime_from_toml(&zulu).unwrap(),
            datetime!(2026-11-02 16:53:12 +01:00)
        );
        let local: Datetime = "2026-11-02T15:53:12".parse().unwrap();
        assert!(datetime_from_toml(&local).is_err());
        let date_only: Datetime = "2026-11-02".parse().unwrap();
        assert!(datetime_from_toml(&date_only).is_err());
    }

    #[test]
    fn version_check() {
        assert!(check_version(1).is_ok());
        assert!(check_version(2).is_err());
        assert!(check_version(0).is_err());
    }
}
