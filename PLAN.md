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

### Klíč slovíčka a proč existuje pole `klic`

Aplikace si u každé kartičky pamatuje postup učení (kdy se opakovala, jak dobře ji umíš).
Aby ho mohla přiřadit ke správnému slovíčku, potřebuje každé slovíčko jednoznačný
identifikátor, tedy **klíč**. Klíčem je běžně samotné vietnamské slovo (`vi`).

**Pole `klic` existuje kvůli homonymům.** Homonyma jsou stejně psaná slova s různým
významem, např. *đường* = cesta i cukr, *bàn* = stůl i diskutovat. Kdyby klíčem bylo jen
`vi`, obě slovíčka *đường* by sdílela jeden postup učení a aplikace by je nerozlišila.
Proto se u homonym vyplní `klic`, který je odliší (např. `"đường (cesta)"`
a `"đường (cukr)"`). U ostatních slovíček se `klic` nevyplňuje.

Pravidla:

- Klíčem je `klic`, pokud je vyplněný, jinak `vi`.
- Klíč musí být unikátní napříč všemi lekcemi. Při načtení se to kontroluje
  a duplicita je chyba se srozumitelnou hláškou, která radí doplnit `klic`.
- Když se změní `vi` (např. oprava překlepu) nebo `klic`, slovíčko přijde o postup.
  Aplikace takové osiřelé záznamy v postupu ponechá, takže omyl jde napravit vrácením
  původního klíče.

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

Cílem je **přesný** fonetický přepis hanojské výslovnosti, bez zjednodušování kvůli fontu.

- Slabiky se oddělují mezerou.
- Tóny se zapisují Chaovými tónovými písmeny za slabikou: ngang ˧˧, huyền ˨˩, sắc ˧˥,
  hỏi ˧˩˧, ngã ˧ˀ˥, nặng ˨˩ˀ.
- *ch* a *tr*, které v Hanoji splývají, se zapisují jako **t͡ɕ** (např. *chào* → `t͡ɕaːw˨˩`).
- Nevypuštěné koncové souhlásky se značí ̚ (např. *một* → `mot̚˨˩ˀ`).
- Při přípravě první lekce (etapa 8) se konvence ověří podle odborných zdrojů
  (např. Kirby 2011, *Vietnamese (Hanoi Vietnamese)*, Journal of the IPA) a případně
  upřesní. Hlavně jde o přesné hodnoty tónů.

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

# Homonyma: stejné vi, různý význam, proto má každé svůj klic.
[[slovicka]]
vi = "đường"
klic = "đường (cesta)"
cs = "cesta, silnice"
vyslovnost_cs = "dɨəng (nízko klesá)"
ipa = "ɗɨəŋ˨˩"

[[slovicka]]
vi = "đường"
klic = "đường (cukr)"
cs = "cukr"
vyslovnost_cs = "dɨəng (nízko klesá)"
ipa = "ɗɨəŋ˨˩"
```

### Kontrola lekcí

Při načtení složky se slovníčky se kontroluje a **všechny problémy se ohlásí najednou**,
každý s názvem souboru a pořadím slovíčka:

- Soubory se jmenují `lekce<číslo>.toml`. Jiné soubory se ignorují (a zalogují).
  Dva soubory se stejným číslem (např. `lekce1.toml` a `lekce01.toml`) jsou chyba.
- Zápis TOML musí být platný. Neznámé pole (např. překlep `poznamka` → `poznámka`) je chyba,
  aby se tiše neztratilo.
- Lekce musí mít `nazev` a aspoň jedno slovíčko.
- Pole `vi`, `cs`, `vyslovnost_cs` a `ipa` nesmí být prázdná. Pokud je uvedený `klic`,
  nesmí být prázdný.
- `vi` a `klic` nesmí mít mezeru na začátku ani na konci. Vznikl by jiný klíč, který vypadá
  stejně.
- `vi`, `klic`, `cs` a `vyslovnost_cs` musí používat složené znaky (Unicode NFC), ne písmeno
  + kombinující diakritiku. Na pohled se neliší, ale klíče by se lišily. V `ipa` jsou
  kombinující znaménka (t͡ɕ, ̚) v pořádku.
- Klíče musí být unikátní napříč všemi lekcemi. Hláška u duplicity radí doplnit `klic`.

## Datová složka a logy

Obě složky jsou v **pracovním adresáři**, ze kterého se aplikace spouští, mimo složku
se slovníčky:

- `data/` je datová složka. Obsahuje:
  - `postup.toml` se stavem FSRS pro každou kartičku (klíč + směr): stabilitu, obtížnost,
    datum posledního a příštího opakování, počet opakování a počet zapomenutí. Kartičky,
    které v souboru nejsou, jsou nové.
  - `historie.toml` s historií hodnocení (čas, kartička, hodnocení). Slouží pro statistiky
    a pro budoucí doladění parametrů FSRS.
  - `nastaveni.toml` s cestou ke slovníčkům, denním limitem nových kartiček, cílovou retencí
    a směry procvičování.

  Vytvoří se při prvním ukládání.
- `logs/` obsahuje JSON logy, jeden soubor na den, drží se posledních 7.
  Vytváří se při startu aplikace.

Obě složky jsou v `.gitignore`. Zápis dat je vždy bezpečný: nejdřív do dočasného souboru,
pak přejmenování.

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

Všude se používá **Noto Sans** (licence OFL), statické řezy `NotoSans-Regular.ttf`
a `NotoSans-Bold.ttf` ve složce `assets/fonts/`, přibalené do binárky (`include_bytes!`).
Variabilní verze se nepoužívá.

Ověřené pokrytí: celá vietnamská abeceda (malá i velká písmena), čeština a všechny znaky
IPA potřebné pro vietnamštinu, včetně ɕ, ̚, ̰, spojovacího obloučku ͡ a tónových písmen ˥˦˧˨˩.

- Noto Sans nemá šipku → a další symboly. Výchozí fonty egui proto zůstanou v seznamu
  jako záložní, až za Noto Sans.
- Vykreslení v egui je ověřené (etapa 1): vietnamská diakritika, čeština, šipka → (ze
  záložního fontu), spojovací oblouček v t͡ɕ, ̚ i tónová písmena se zobrazují správně.

## Logování

`tracing` + `tracing-subscriber` (feature `json`) + `tracing-appender`
(denní rotace, `max_log_files(7)`), zápis do složky `logs/` v pracovním adresáři. Loguje se start aplikace, načtení lekcí, chyby
při načítání a ukládání a souhrn sezení.

## Struktura projektu (návrh)

```
Cargo.toml
rust-toolchain.toml
README.md
src/
  main.rs        # binárka: spuštění eframe, inicializace logování, GUI
  lib.rs         # knihovna: logika oddělená od GUI (testovatelná bez okna)
  fonts.rs       # nastavení fontů
  app.rs         # stav aplikace, přepínání obrazovek
  ui/            # jednotlivé obrazovky
  model.rs       # slovíčko, kartička, směr
  lessons.rs     # načítání lekcí z TOML + kontrola (duplicitní klíče, prázdná pole)
  progress.rs    # načítání/ukládání postupu a historie
  settings.rs    # nastavení
  paths.rs       # umístění složek data/ a logs/
  logging.rs     # nastavení tracing
  fsrs.rs        # algoritmus FSRS
  scheduler.rs   # sestavení fronty na dnešek
assets/
  fonts/
    NotoSans-Regular.ttf
    NotoSans-Bold.ttf
    OFL.txt
slovnicky/
  lekce01.toml
  ...
```

## Etapy

| # | Etapa | Stav |
|---|-------|------|
| 0 | Pravidla (`AGENTS.md`), plán (`PLAN.md`), zafixovaný toolchain | hotovo |
| 1 | Kostra projektu: `Cargo.toml`, prázdné okno eframe, logování, font Noto Sans a vizuální ověření IPA | hotovo |
| 2 | Datový model a načítání lekcí z TOML, validace, testy | hotovo |
| 3 | FSRS + testy | čeká |
| 4 | Postup učení a nastavení: ukládání, načítání, sestavení fronty, testy | čeká |
| 5 | GUI: úvod a opakování kartiček (to hlavní, s čím už se dá učit) | čeká |
| 6 | GUI: seznam lekcí, nastavení, legenda tónů | čeká |
| 7 | Statistiky | čeká |
| 8 | Lekce: první sada (pozdravy, čísla, základní fráze, jídlo…) | čeká |

## Otevřené otázky

- Přepis hlásek, které čeština nemá, do českého popisu výslovnosti (etapa 8).
