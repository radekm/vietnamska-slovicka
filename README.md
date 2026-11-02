# Vietnamská slovíčka

Desktopová aplikace pro učení vietnamských slovíček a frází metodou kartiček
s rozloženým opakováním (algoritmus FSRS). U vietnamštiny vždy ukazuje, jak se čte:
český popis výslovnosti a přesný přepis IPA (severní, hanojská výslovnost).

Projekt je zatím ve vývoji. Stav a další kroky popisuje [PLAN.md](PLAN.md),
pravidla pro vývoj [AGENTS.md](AGENTS.md).

## Sestavení a spuštění

Potřebuješ jen [rustup](https://rustup.rs/). Správnou verzi Rustu (1.99.1) si stáhne
sám podle souboru `rust-toolchain.toml`.

```sh
cargo run --release
```

## Kde jsou data

Postup učení, nastavení a logy se ukládají do datové složky aplikace:

| Systém  | Složka                                                   |
|---------|----------------------------------------------------------|
| Linux   | `~/.local/share/vietnamska-slovicka/` (nebo `$XDG_DATA_HOME`) |
| macOS   | `~/Library/Application Support/vietnamska-slovicka/`     |
| Windows | `%APPDATA%\vietnamska-slovicka\`                          |

Logy jsou ve formátu JSON v podsložce `logy/`, jeden soubor na den, drží se posledních 7.

## Licence fontu

Aplikace obsahuje font Noto Sans pod licencí SIL Open Font License 1.1,
viz [assets/fonts/OFL.txt](assets/fonts/OFL.txt).
