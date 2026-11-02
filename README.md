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

## Ovládání

- Na úvodu zaškrtni lekce, ze kterých se mají brát nové kartičky, a spusť sezení
  tlačítkem **Začít** (nebo Enterem).
- Kartičku otočíš **mezerníkem** a ohodnotíš klávesami **1–4**
  (Znovu / Těžké / Dobré / Snadné). U každého tlačítka je vidět, kdy kartička přijde znovu.
- **Esc** sezení předčasně ukončí. Postup se ukládá po každém hodnocení.

## Kde jsou lekce

Lekce (`lekce01.toml`, `lekce02.toml`, …) aplikace hledá ve složce `slovnicky/`
v pracovním adresáři, ze kterého ji spustíš. V repozitáři je to složka `slovnicky/`,
takže stačí spouštět aplikaci z kořene repozitáře. (První lekce přibudou v etapě 8,
do té doby aplikace ohlásí, že složka chybí.)

## Kde jsou data

Aplikace si vytváří dvě složky v pracovním adresáři, ze kterého ji spustíš:

- `data/` obsahuje postup učení (`postup.toml`), historii hodnocení (`historie.toml`)
  a nastavení (`nastaveni.toml`). Jsou to čitelné TOML soubory, formát popisuje
  [PLAN.md](PLAN.md). Když je některý poškozený, aplikace ho nepřepíše a ohlásí chybu,
  aby nepřišel postup učení,
- `logs/` obsahuje logy ve formátu JSON, jeden soubor na den, drží se posledních 7.

Spouštěj ji proto vždy ze stejného adresáře, jinak nenajde svůj postup učení.

## Licence fontu

Aplikace obsahuje font Noto Sans pod licencí SIL Open Font License 1.1,
viz [assets/fonts/OFL.txt](assets/fonts/OFL.txt).
