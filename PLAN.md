# Plán: Vietnamská slovíčka

Desktopová aplikace (Rust + egui) pro učení vietnamských slovíček a frází metodou kartiček
s rozloženým opakováním (spaced repetition).

## Cíle

- Učit slovíčka a fráze oběma směry: čeština → vietnamština a vietnamština → čeština.
- U vietnamštiny vždy zobrazit, jak se čte: český popis výslovnosti a IPA.
- Výslovnost odpovídá **severnímu (hanojskému)** dialektu.
- Plánovat opakování algoritmem **FSRS** tak, aby se učilo co nejméně opakováními.
- Lekce připravuje AI agent, ukládají se v TOML a dají se snadno upravovat ručně.

## Jedno slovíčko

| Pole            | Povinné | Význam                                         | Příklad                             |
|-----------------|---------|------------------------------------------------|-------------------------------------|
| `vi`            | ano     | vietnamsky (s diakritikou a tóny), zároveň klíč | `"cảm ơn"`                          |
| `klic`          | ne      | náhradní klíč pro homonyma (viz níže)          | `"đường (cukr)"`                    |
| `cs`            | ano     | česky                                          | `"děkuji"`                          |
| `vyslovnost_cs` | ano     | výslovnost popsaná česky včetně tónů           | `"kam (klesá a stoupá) ən (rovně)"` |
| `ipa`           | ano     | výslovnost v IPA (hanojská)                    | `"kaːm˧˩˧ ʔəːn˧˧"`                  |
| `poznamka`      | ne      | volitelná poznámka                             | `"běžné poděkování"`                |
| `stitky`        | ne      | volitelné štítky                               | `["zdvořilost"]`                    |

Ze slovíčka vzniknou **dvě kartičky**, jedna pro každý směr. Každá se plánuje samostatně.

### Klíč slovíčka

Postup učení se k slovíčku váže přes jeho **klíč**. Klíčem je `klic`, pokud je vyplněný,
jinak `vi`. Klíč musí být unikátní napříč všemi lekcemi. Při načtení se to kontroluje
a duplicita je chyba se srozumitelnou hláškou.

`klic` je potřeba jen u homonym, tedy stejně psaných slov s jiným významem
(např. *đường* = cesta / cukr, *bàn* = stůl / diskutovat). Pokud se opraví překlep ve `vi`,
slovíčko přijde o postup. Aplikace takové osiřelé záznamy v postupu ponechá, takže omyl
jde napravit vrácením původního klíče.

### Český popis výslovnosti

Slabiky se oddělují mezerou a za každou slabikou je v závorce slovní popis tónu:

| Tón (vietnamsky) | Značka | Popis v závorce        |
|------------------|--------|------------------------|
| ngang            | a      | rovně                  |
| huyền            | à      | nízko klesá            |
| sắc              | á      | stoupá                 |
| hỏi              | ả      | klesá a stoupá         |
| ngã              | ã      | stoupá se zlomem       |
| nặng             | ạ      | krátce klesá, ráz      |

Přesný přepis hlásek, které čeština nemá (ơ, ư, â, ă, koncové -ng/-nh/-ch…), se ustálí
při přípravě první lekce (etapa 8) a doplní se sem.

### Konvence IPA

Hanojská výslovnost, tóny zapsané Chaovými tónovými písmeny
(ngang ˧˧, huyền ˨˩, sắc ˧˥, hỏi ˧˩˧, ngã ˧ˀ˥, nặng ˨˩ˀ), slabiky oddělené mezerou.

## Složka se slovníčky

Lekce jsou ve složce se slovníčky jako `lekce01.toml`, `lekce02.toml`, … V repozitáři je
to složka `slovnicky/`. Aplikace si cestu ke složce pamatuje v nastavení a jde ji změnit
dialogem pro výběr složky (`rfd`). Lekce se řadí podle čísla v názvu souboru.

```toml
# slovnicky/lekce01.toml
nazev = "Pozdravy"
popis = "Základní pozdravy a zdvořilostní fráze"

[[slovicka]]
vi = "xin chào"
cs = "dobrý den / ahoj"
vyslovnost_cs = "sin (rovně) čao (nízko klesá)"
ipa = "sin˧˧ t͡ɕaːw˨˩"
stitky = ["pozdrav"]

[[slovicka]]
vi = "cảm ơn"
cs = "děkuji"
vyslovnost_cs = "kam (klesá a stoupá) ən (rovně)"
ipa = "kaːm˧˩˧ ʔəːn˧˧"
poznamka = "běžné poděkování"
```

## Datová složka aplikace

Postup učení, nastavení a logy jsou **mimo složku se slovníčky**, v datové složce aplikace:

- Linux: `$XDG_DATA_HOME/vietnamska-slovicka/`, výchozí `~/.local/share/vietnamska-slovicka/`
- macOS: `~/Library/Application Support/vietnamska-slovicka/`
- Windows: `%APPDATA%\vietnamska-slovicka\`

Obsah:

- `postup.toml` obsahuje pro každou kartičku (klíč + směr) stav FSRS: stabilitu, obtížnost,
  datum posledního a příštího opakování, počet opakování a počet zapomenutí. Kartičky, které
  v souboru nejsou, jsou nové.
- `historie.toml` obsahuje historii hodnocení (čas, kartička, hodnocení). Slouží pro statistiky
  a pro budoucí doladění parametrů FSRS.
- `nastaveni.toml` obsahuje cestu ke slovníčkům, denní limit nových kartiček, cílovou retenci
  a směry procvičování.
- `logy/` obsahuje JSON logy, drží se posledních 7.

Zápis je vždy bezpečný: nejdřív do dočasného souboru, pak přejmenování.

## Algoritmus: FSRS

- Implementace vlastní, bez externí crate (vzorce mají pár desítek řádků).
- Verze FSRS-5 s **výchozími parametry**.
- Cílová pravděpodobnost zapamatování (desired retention): **0,9**, nastavitelná.
- Hodnocení čtyřmi tlačítky: **Znovu / Těžké / Dobré / Snadné** (klávesy 1–4).
- Denní limit nových kartiček (výchozí 10), nastavitelný.
- Plánování po dnech. Den začíná ve 4:00 místního času, aby učení po půlnoci patřilo k předchozímu dni.
- Modul s algoritmem je čistá logika bez GUI, pokrytá unit testy (známé hodnoty z referenční implementace).

## Průběh opakování

1. Aplikace sestaví frontu: kartičky k opakování dnes + nové kartičky do denního limitu
   (z vybraných lekcí, v pořadí lekcí a slovíček).
2. Zobrazí se **líc** kartičky:
   - **CS → VI:** česky (+ poznámka).
   - **VI → CS:** vietnamsky + český popis výslovnosti + IPA.
3. Uživatel přemýšlí a pak kartičku otočí (tlačítko nebo mezerník).
4. Zobrazí se **rub**: vždy všechna pole, tedy vietnamsky, česky, výslovnost česky, IPA a poznámka.
5. Uživatel se ohodnotí. FSRS spočítá příští opakování. Při hodnocení „Znovu“ se kartička
   během sezení zopakuje ještě jednou.
6. Postup se ukládá průběžně po každém hodnocení.

## Obrazovky (egui)

- **Úvod:** kolik je dnes k opakování a kolik nových, výběr lekcí a tlačítko Začít.
- **Opakování:** kartička, otočení, hodnocení, ukazatel postupu sezení.
- **Lekce:** seznam lekcí a slovíček, aby šlo procházet a hledat.
- **Statistiky** (`egui_plot`): opakování za den, předpověď počtu opakování na další dny,
  počet naučených kartiček.
- **Nastavení:** složka se slovníčky, denní limit nových kartiček, cílová retence, směry procvičování.
- **Legenda tónů:** vždy po ruce (např. tlačítko „?“), obsahuje tabulku tónů z tohoto plánu.

## Font

Použije se **Inter**, soubor fontu dodá autor a bude přibalen do binárky.
Inter pokrývá vietnamskou diakritiku. Pokud v něm budou chybět znaky IPA
(ɗ, ɲ, ŋ, ʔ, ˀ, tónová písmena ˧˨˩˥), domluvíme záložní font pro tyto znaky.
Pokrytí se ověří v etapě 1.

## Logování

`tracing` + `tracing-subscriber` (JSON výstup) + `tracing-appender`
(rotace souborů, `max_log_files(7)`). Loguje se start aplikace, načtení lekcí, chyby
při načítání a ukládání a souhrn sezení.

## Struktura projektu (návrh)

```
Cargo.toml
rust-toolchain.toml
src/
  main.rs        # spuštění eframe, inicializace logování
  app.rs         # stav aplikace, přepínání obrazovek
  ui/            # jednotlivé obrazovky
  model.rs       # slovíčko, kartička, směr
  lessons.rs     # načítání lekcí z TOML + kontrola (duplicitní klíče, prázdná pole)
  progress.rs    # načítání/ukládání postupu a historie
  settings.rs    # nastavení
  paths.rs       # datová složka aplikace podle OS
  logging.rs     # nastavení tracing
  fsrs.rs        # algoritmus FSRS
  scheduler.rs   # sestavení fronty na dnešek
assets/
  Inter-*.ttf
slovnicky/
  lekce01.toml
  ...
```

## Etapy

| # | Etapa | Stav |
|---|-------|------|
| 0 | Pravidla (`AGENTS.md`), plán (`PLAN.md`), zafixovaný toolchain | rozpracováno |
| 1 | Kostra projektu: `Cargo.toml`, prázdné okno eframe, logování, font Inter a ověření IPA | čeká |
| 2 | Datový model a načítání lekcí z TOML, validace, testy | čeká |
| 3 | FSRS + testy | čeká |
| 4 | Postup učení a nastavení: ukládání, načítání, sestavení fronty, testy | čeká |
| 5 | GUI: úvod a opakování kartiček (to hlavní, s čím už se dá učit) | čeká |
| 6 | GUI: seznam lekcí, nastavení, legenda tónů | čeká |
| 7 | Statistiky | čeká |
| 8 | Lekce: první sada (pozdravy, čísla, základní fráze, jídlo…) | čeká |

## Otevřené otázky

- Feature `json` u `tracing-subscriber` (potřebná pro JSON logy) čeká na schválení.
- Rotace logů: denně, nebo jeden soubor na každé spuštění?
- Přepis hlásek, které čeština nemá, do českého popisu výslovnosti (etapa 8).
