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
dialogem pro výběr složky (`rfd`). Dokud není nastavená, použije se `slovnicky/`
v pracovním adresáři. Lekce se řadí podle čísla v názvu souboru.

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
    datum prvního, posledního a příštího opakování, počet opakování a počet zapomenutí.
    Kartičky, které v souboru nejsou, jsou nové.
  - `historie.toml` s historií hodnocení (čas, kartička, hodnocení). Slouží pro statistiky
    a pro budoucí doladění parametrů FSRS.
  - `nastaveni.toml` s cestou ke slovníčkům, denním limitem nových kartiček, cílovou retencí,
    směry procvičování a vybranými lekcemi.

  Vytvoří se při prvním ukládání.
- `logs/` obsahuje JSON logy, jeden soubor na den, drží se posledních 7.
  Vytváří se při startu aplikace.

Obě složky jsou v `.gitignore`. Zápis dat je vždy bezpečný: nejdřív do dočasného souboru
ve stejné složce, `sync_all` a pak přejmenování.

### Formát datových souborů

Data se zapisují v nativních typech TOML (datum, čas s časovým posunem). Každý soubor má
`verze = 1` kvůli budoucím úpravám formátu. Neznámé pole je chyba, aby starší verze
aplikace tiše nesmazala, čemu nerozumí.

```toml
# data/postup.toml
verze = 1

[[karticky]]
klic = "cảm ơn"
smer = "vi-cs"                 # "cs-vi" nebo "vi-cs"
stabilita = 10.73892584613159
obtiznost = 5.272967931287446
prvni_opakovani = 2026-11-02   # pro denní limit nových a statistiku naučených
posledni_opakovani = 2026-11-05
pristi_opakovani = 2026-11-16
opakovani = 2
zapomenuti = 0
```

```toml
# data/historie.toml
verze = 1

[[hodnoceni]]
cas = 2026-11-02T16:53:12+01:00
klic = "cảm ơn"
smer = "vi-cs"
hodnoceni = 3                  # 1 Znovu, 2 Těžké, 3 Dobré, 4 Snadné
```

```toml
# data/nastaveni.toml
verze = 1
slozka_slovnicku = "/home/radek/vietnamska-slovicka/slovnicky"   # volitelné
novych_denne = 10
cilova_retence = 0.9
smery = ["cs-vi", "vi-cs"]
vybrane_lekce = [1, 2]         # čísla lekcí zapamatovaná z úvodní obrazovky
```

Pravidla:

- Chybějící soubor znamená prázdný postup, prázdnou historii nebo výchozí nastavení.
- **Poškozený soubor je chyba, nikdy prázdný stav.** Jinak by ho první uložení přepsalo
  a postup by se ztratil.
- Osiřelé záznamy (klíč, který v lekcích už není) se zachovávají.
- Historie se po každém hodnocení zapisuje celá znovu (bezpečně, viz výše).

### Místní čas

Místní časový posun se zjistí **jednou na začátku `main`**, dokud program běží v jednom
vlákně (`time` ho na Linuxu ve vícevláknovém programu nevrátí). Když se to nepovede, použije
se UTC a zaloguje se varování. Pokud aplikace běží přes přechod letního času, může se začátek
studijního dne posunout o hodinu až do restartu, což je zanedbatelné.

## Algoritmus: FSRS

- Implementace vlastní, bez externí crate (vzorce mají pár desítek řádků).
- Verze FSRS-5 s **výchozími parametry**.
- Cílová pravděpodobnost zapamatování (desired retention): **0,9**, nastavitelná.
- Hodnocení čtyřmi tlačítky: **Znovu / Těžké / Dobré / Snadné** (klávesy 1–4).
- Denní limit nových kartiček (výchozí 10), nastavitelný.
- Plánování po dnech. Den začíná ve 4:00 místního času, aby učení po půlnoci patřilo k předchozímu dni.
- Modul s algoritmem je čistá logika bez GUI, pokrytá unit testy (známé hodnoty z referenční implementace).

Upřesnění implementace (`src/fsrs.rs`, etapa 3):

- Referenční implementace je **py-fsrs v4.1.2** (FSRS-5) nastavená bez učebních kroků
  a bez fuzzu (`learning_steps=()`, `relearning_steps=()`, `enable_fuzzing=False`).
  Testy porovnávají stabilitu, obtížnost a interval s hodnotami z ní.
- **Bez náhodného rozptylu intervalů (fuzz)**, plánování je deterministické.
- **Bez učebních kroků**, nejkratší interval je 1 den, nejdelší 36 500 dní.
- Interval se zaokrouhluje na celé dny, polovina k sudé (jako `round` v Pythonu).
- Opakování ve **stejný studijní den** jako předchozí (např. zopakování po „Znovu“ během
  sezení) používá krátkodobý vzorec FSRS-5 (parametry w17, w18).
- **Zapomenutí** (`lapses`) je hodnocení „Znovu“ při opakování v jiný den než předchozí
  opakování. První hodnocení nové kartičky ani zopakování týž den se nepočítá.
- Cílová retence musí být v rozsahu 0,70 až 0,99.
- Studijní den spočítá `study_day` z místního času (posun o 4 hodiny). Zjištění místního
  časového posunu popisuje oddíl „Místní čas“.

## Průběh opakování

1. Aplikace sestaví frontu: kartičky k opakování dnes + nové kartičky do denního limitu.
   - K opakování jdou kartičky ze **všech** lekcí (výběr lekcí je neomezuje), jen v zapnutých
     směrech. Řadí se od nejvíc prošlých, při shodě podle pořadí lekcí a slovíček.
   - Nové kartičky jen z **vybraných** lekcí, v pořadí lekcí a slovíček. Limit se počítá
     v kartičkách a odečítají se nové, které už dnes začaly (podle `prvni_opakovani`).
   - Nové slovíčko se zavádí nejdřív jako **VI → CS**. Kartička **CS → VI** je nová k zavedení
     až v pozdější den, než se poprvé opakovala VI → CS (jinak by byla zadarmo, protože
     odpověď byla právě vidět). Platí jen, když jsou zapnuté oba směry.
   - Když jsou obě kartičky téhož slovíčka k opakování týž den, ukážou se obě.
2. Zobrazí se **líc** kartičky:
   - **CS → VI:** česky (+ poznámka).
   - **VI → CS:** vietnamsky + český popis výslovnosti + IPA.
3. Uživatel přemýšlí a pak kartičku otočí (tlačítko nebo mezerník).
4. Zobrazí se **rub**: vždy všechna pole, tedy vietnamsky, česky, výslovnost česky, IPA a poznámka.
5. Uživatel se ohodnotí. FSRS spočítá příští opakování. Při hodnocení „Znovu“ se kartička
   vrátí do fronty o 5 kartiček dál (nebo na konec, když jich tolik nezbývá) a opakuje se,
   dokud nedostane jiné hodnocení než „Znovu“.
6. Postup se ukládá průběžně po každém hodnocení. Když se uložení nepovede, hodnocení
   zůstane v paměti, zobrazí se chyba a další hodnocení zkusí uložit znovu. Studijní den
   se bere z času každého hodnocení, takže sezení přes 4:00 se zapíše správně.

Pořadí kartiček v sezení:

- Nové kartičky jsou **rovnoměrně rozprostřené** mezi kartičky k opakování.
- Dvě kartičky téhož slovíčka nejdou hned po sobě, pokud je mezi ně co dát.
- Na konci sezení (i předčasném) se zaloguje souhrn hodnocení.

## Obrazovky (egui)

- **Úvod:** kolik je dnes k opakování a kolik nových, výběr lekcí (u každé počet slovíček
  a zbývajících nových kartiček, výběr se hned ukládá) a tlačítko Začít. Chyby v lekcích
  nebo poškozená data se zobrazí jako seznam hlášek s tlačítkem „Zkusit znovu“.
- **Opakování:** ukazatel postupu sezení, směr, tlačítko Ukončit; kartička a otočení
  (mezerník); hodnocení tlačítky nebo klávesami 1–4, u každého tlačítka příští interval
  (např. „za 3 dny“). Na konci shrnutí a návrat na úvod.
- **Lekce:** seznam lekcí a slovíček, aby šlo procházet a hledat.
- **Statistiky** (`egui_plot`): opakování za den, předpověď počtu opakování na další dny,
  počet naučených kartiček.
- **Nastavení:** složka se slovníčky, denní limit nových kartiček, cílová retence, směry procvičování.
- **Legenda tónů:** vždy po ruce (např. tlačítko „?“), obsahuje tabulku tónů z tohoto plánu.

Upřesnění (etapa 6):

- **Navigace:** nahoře lišta se záložkami Úvod · Lekce · Statistiky · Nastavení a vpravo tlačítko „?“. Během opakování se záložky schovají, aby se sezení
  neopustilo omylem, tlačítko „?“ zůstane.
- **Legenda tónů** je plovoucí okno, které nic neblokuje, přepíná ho tlačítko „?“ nebo F1.
  Kromě tabulky tónů ukazuje i Chaova tónová písmena (viz „Konvence IPA“) a příklad
  *ma, mà, má, mả, mã, mạ* s významy. Po ověření hodnot tónů v etapě 8 se upraví i tady.
- **Lekce:** vlevo seznam lekcí a pole pro hledání, vpravo tabulka slovíček (VI, CS,
  výslovnost, IPA, poznámka a štítky) se stavem obou kartiček („nová“, „dnes“ nebo datum
  příštího opakování). Vyplněné hledání prohledává všechny lekce (VI, CS, výslovnost,
  poznámku a štítky), nerozlišuje velikost písmen a **ignoruje diakritiku** („cam on“
  najde „cảm ơn“, „dekuji“ najde „děkuji“).
- **Nastavení** se ukládá tlačítkem Uložit, ne hned. Před uložením nové složky se slovníčky
  se lekce z ní zkusmo načtou, a pokud jsou v nich chyby, ukážou se a nic se neuloží.
  Po změně složky se lekce načtou znovu.
- **Změna cílové retence platí jen pro další hodnocení.** Už naplánovaná data příštího
  opakování v `postup.toml` se nepřepočítávají, data se tím nemění (stejně jako v Anki).
- **Chybová obrazovka** (data se nepodařilo načíst) nabízí kromě „Zkusit znovu“ i změnu
  složky se slovníčky.
- V Nastavení je i tlačítko „Načíst lekce znovu“ pro načtení ručně upravených lekcí.

Upřesnění (etapa 7, schváleno autorem):

- **Statistiky** jsou záložka mezi Lekce a Nastavení. Výpočty jsou v `src/stats.rs` (čistá
  logika s testy), GUI je jen vykresluje. Počítá se po **studijních dnech** (od 4:00),
  **oba směry dohromady**, pevný rozsah **30 dní** (bez přepínače).
- **Souhrn nahoře:** dnes ohodnoceno, zavedeno kartiček, naučeno kartiček a úspěšnost za
  30 dní (podíl opakování v jiný den než předchozí opakování, která nebyla „Znovu“).
- **Opakování za den:** posledních 30 dní včetně dneška, skládaný sloupcový graf podle
  hodnocení Znovu / Těžké / Dobré / Snadné. Počítá se každé hodnocení z `historie.toml`,
  i zopakování po „Znovu“ týž den.
- **Předpověď opakování:** dnešek a dalších 29 dní, sloupcový graf počtu kartiček podle
  `pristi_opakovani`. Prošlé se připočtou k dnešku, nové kartičky se nepočítají. Jen kartičky,
  které jsou v lekcích a v zapnutých směrech (jako skutečná fronta).
- **Naučené kartičky:** čárový graf za posledních 30 dní se dvěma čarami:
  - **zavedené** = kartička byla aspoň jednou ohodnocena,
  - **naučené** = stabilita FSRS **≥ 21 dní** (obdoba „zralých“ kartiček v Anki).
  Stav v minulých dnech se zjistí přehráním historie hodnocení přes FSRS (stabilita nezávisí
  na cílové retenci). Počítají se jen kartičky, které jsou v lekcích (bez osiřelých).
- Osa X ukazuje datum („2. 11.“), při najetí myší se ukáže den a hodnota.

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
  ui/            # obrazovky: úvod, opakování, shrnutí, lekce, statistiky, nastavení, legenda tónů
  model.rs       # slovíčko, kartička, směr
  browse.rs      # hledání slovíček bez diakritiky, stav kartičky pro seznam lekcí
  lessons.rs     # načítání lekcí z TOML + kontrola (duplicitní klíče, prázdná pole)
  progress.rs    # načítání/ukládání postupu a historie
  settings.rs    # nastavení
  storage.rs     # bezpečný zápis souborů, převody data a času pro TOML
  paths.rs       # umístění složek data/ a logs/
  logging.rs     # nastavení tracing
  fsrs.rs        # algoritmus FSRS-5, studijní den
  scheduler.rs   # sestavení fronty na dnešek
  session.rs     # průběh sezení, uložení hodnocení
  stats.rs       # výpočty pro statistiky (opakování za den, předpověď, naučené)
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
| 3 | FSRS + testy | hotovo |
| 4 | Postup učení a nastavení: ukládání, načítání, sestavení fronty, testy | hotovo |
| 5 | GUI: úvod a opakování kartiček (to hlavní, s čím už se dá učit) | hotovo |
| 6 | GUI: seznam lekcí, nastavení, legenda tónů | hotovo |
| 7 | Statistiky | rozpracováno |
| 8 | Lekce: první sada (pozdravy, čísla, základní fráze, jídlo…) | čeká |

## Otevřené otázky

- Přepis hlásek, které čeština nemá (ơ, ư, â, ă, koncové -ng/-nh/-ch…), do českého popisu
  výslovnosti (etapa 8).
- Ověřit konvenci IPA, hlavně přesné hodnoty tónů, podle odborných zdrojů (např. Kirby 2011).
  Po ověření upravit i tabulku tónů v `src/ui/legend.rs` (etapa 8).
- Neuložené změny v Nastavení se odchodem na jinou záložku zahodí (na obrazovce je jen
  upozornění). Případně doplnit potvrzovací dotaz.
- V logu jsou i záznamy knihoven (eframe, wgpu, rfd). Případné filtrování zatím neřešíme.
