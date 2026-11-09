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

- Nahoře jsou záložky **Úvod**, **Lekce**, **Statistiky** a **Nastavení**. Tlačítko **?** (nebo klávesa
  **F1**) otevře legendu tónů, i během opakování.
- Na úvodu zaškrtni lekce, ze kterých se mají brát nové kartičky, a spusť sezení
  tlačítkem **Začít** (nebo Enterem).
- Kartičku otočíš **mezerníkem** a ohodnotíš klávesami **1–4**
  (Znovu / Těžké / Dobré / Snadné). U každého tlačítka je vidět, kdy kartička přijde znovu.
- **Esc** sezení předčasně ukončí. Postup se ukládá po každém hodnocení.
- V **Lekcích** se dají procházet všechna slovíčka a u každého je vidět, kdy přijdou
  na řadu jeho kartičky. Hledání prohledává všechny lekce a na diakritice nezáleží
  („cam on“ najde „cảm ơn“).
- Ve **Statistikách** je souhrn (dnes ohodnoceno, zavedené a naučené kartičky, úspěšnost
  za 30 dní) a tři grafy: opakování za posledních 30 dní podle hodnocení, předpověď opakování
  na dalších 30 dní a vývoj počtu zavedených a naučených kartiček. Naučená je kartička se
  stabilitou FSRS aspoň 21 dní. Po najetí myší na sloupec nebo bod grafu se ukáže datum
  a hodnota.
- V **Nastavení** jde změnit složku se slovníčky, denní limit nových kartiček, cílovou
  retenci a směry procvičování. Změny se projeví až po stisknutí **Uložit**. Nová cílová
  retence platí jen pro další hodnocení, už naplánovaná opakování se nemění.

## Kde jsou lekce

Lekce (`lekce01.toml`, `lekce02.toml`, …) aplikace hledá ve složce `slovnicky/`
v pracovním adresáři, ze kterého ji spustíš. V repozitáři je to složka `slovnicky/`,
takže stačí spouštět aplikaci z kořene repozitáře. Jinou složku jde vybrat v Nastavení.
Po ruční úpravě lekcí je načte znovu tlačítko **Načíst lekce znovu** v Nastavení.

V repozitáři je 23 lekcí (589 slovíček): pozdravy, zájmena a oslovení, čísla,
základní fráze a otázky, jídlo a pití, objednávání v restauraci, ptaní se na cestu,
základní slovesa, stavba věty (čas a zápor), čas a kalendář, přídavná jména a barvy,
nakupování, doprava a ubytování, o sobě a rodina, zdraví a nouze, vietnamská kuchyně
a chutě, ovoce, pití a kavárna, kde co je, nápisy a cedule, památky a výlety, počasí
a roční období, praktické služby a problémy a povídání s místními.
Výslovnost odpovídá hanojskému dialektu, IPA se řídí článkem Kirby (2011). Pravidla
českého popisu výslovnosti a zápisu IPA jsou v [PLAN.md](PLAN.md). Po úpravě lekcí v repozitáři
spusť `cargo test`: zkontroluje, že sedí počet slabik a tóny ve výslovnosti a v IPA.

> **Pomocník pro nová slovíčka:** `python3 tools/pronunciation.py "cảm ơn"` vypíše hotové
> řádky `vyslovnost_cs` a `ipa` podle pravidel z PLAN.md. `python3 tools/pronunciation.py --check`
> porovná s pravidly všechna slovíčka ve `slovnicky/` (na rozdíl od `cargo test` hlídá
> i hlásky, nejen tóny). Stačí Python 3.11+, bez dalších balíčků. Výstup je návrh,
> přejímky a vzácné zápisy je potřeba zkontrolovat ručně.

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
