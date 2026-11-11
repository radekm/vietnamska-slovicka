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
| huyền            | à      | klesá                  |
| sắc              | á      | stoupá                 |
| hỏi              | ả      | klesá hluboko          |
| ngã              | ã      | stoupá se zlomem       |
| nặng             | ạ      | krátce klesá, ráz      |

V zavřené slabice (končí na -p, -t, -c, -ch) má nặng popis **„krátce klesá“** (bez rázu,
viz IPA). Přepis hlásek (schváleno v etapě 8):

| Pravopis | Popis | Pravopis | Popis |
|----------|-------|----------|-------|
| ơ, â | ə | ư | ɨ |
| ê, e | e | ô, o | o |
| a, ă | a | i, y | i |
| iê, ia, yê, ya | iə | ươ, ưa | ɨə |
| uô, ua | uə | koncové -i, -y | j (*ai* → aj, *ơi* → əj) |
| koncové -o, -u | u (*ao* → au, *êu* → eu) | vložené o, u (*hoa*, *quá*) | u (hua, kua) |
| ch, tr | č | d, gi, r | z |
| x, s | s | kh | ch |
| nh | ň | ng, ngh | ng |
| ph | f | th | th (t s přídechem) |
| đ | d | g, gh | g |
| c, k, q | k | -anh, -ách | ajng, ajk |

Ostatní písmena se čtou jako v češtině. Ráz na začátku slabiky (*ơn* = [ʔɤn]) se v popisu
nepíše, rozdíl ê/e a ô/o a sevření rtů v koncovém -ng/-c po u, ô, o je vidět jen v IPA.

### Konvence IPA

Přepis hanojské výslovnosti podle **Kirby (2011)**, *Vietnamese (Hanoi Vietnamese)*,
Journal of the IPA 41/3 (schváleno v etapě 8). Hodnoty tónů v číslech potvrzuje i nástroj
vPhon téhož autora.

- Slabiky se oddělují mezerou, tón se píše Chaovými tónovými písmeny za slabikou.
- **Tóny:**

  | Tón | Otevřená slabika | Zavřená slabika (-p -t -c -ch) | Příklad |
  |-----|------------------|--------------------------------|---------|
  | ngang | ˧˧ | – | *ma* → `ma˧˧` |
  | huyền | ˧˨ | – | *mà* → `ma˧˨` |
  | sắc | ˨˦ | ˦˥ | *má* → `ma˨˦`, *mát* → `mat˦˥` |
  | hỏi | ˧˩˨ | – | *mả* → `ma˧˩˨` |
  | ngã | ˧ˀ˥ | – | *mã* → `ma˧ˀ˥` |
  | nặng | ˨˩ˀ | ˨˩ | *mạ* → `ma˨˩ˀ`, *mạt* → `mat˨˩` |

  Kirby píše každý tón jedním obrysovým znakem, my stejný obrys rozepisujeme do posloupnosti
  tónových písmen. Jiná kvalita hlasu než ráz (ˀ) se nepřepisuje.
- **Samohlásky:** i/y `i`, ê `e`, e `ɛ`, ư `ɯ`, ơ `ɤ`, â `ɤ̆`, a `a`, ă `ă`, u `u`, ô `o`,
  o `ɔ`. Délka se jinak neznačí. Dvojhlásky `iə`, `ɯə`, `uə`. Koncové -i/-y `j`, -o/-u `w`
  (*ai* `aj`, *ay* `ăj`, *ao* `aw`, *au* `ăw`, *ây* `ɤ̆j`, *âu* `ɤ̆w`).
- **Počáteční souhlásky:** b `ɓ`, đ `ɗ`, t `t`, th `tʰ`, ch/tr `t͡ɕ`, c/k/q `k`, ph `f`,
  v `v`, x/s `s`, d/gi/r `z`, kh `x`, g/gh `ɣ`, h `h`, m `m`, n `n`, nh `ɲ`, ng/ngh `ŋ`,
  l `l`. Slabika bez souhlásky začíná rázem `ʔ` (*ăn* → `ʔăn˧˧`). Vložené o/u se píše jako
  `ʷ` za souhláskou (*hoa* → `hʷa˧˧`, *quá* → `kʷa˨˦`), bez souhlásky jako `w`
  (*oan* → `wan˧˧`).
- **Koncové souhlásky:** `p t k m n ŋ` bez značky ̚ (koncové p, t, k jsou vždy
  nevypuštěné). Po i, ê, e se k a ŋ posouvají dopředu: *-inh* `iŋ̟`, *-ich* `ik̟`,
  *-ênh* `eŋ̟`, *-êch* `ek̟`, *-anh* `ɛŋ̟`, *-ach* `ɛk̟`. Po u, ô, o jsou retně-zadopatrové:
  *-ung* `uŋ͡m`, *-ông* `oŋ͡m`, *-ong* `ɔŋ͡m`, *-uc* `uk͡p`, *-ôc* `ok͡p`, *-oc* `ɔk͡p`
  (po dvojhlásce *uô* ne: *xuống* → `suəŋ˨˦`).

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
vyslovnost_cs = "sin (rovně) čau (klesá)"
ipa = "sin˧˧ t͡ɕaw˧˨"
stitky = ["pozdrav"]

[[slovicka]]
vi = "cảm ơn"
cs = "děkuji"
vyslovnost_cs = "kam (klesá hluboko) ən (rovně)"
ipa = "kam˧˩˨ ʔɤn˧˧"
poznamka = "běžné poděkování"

# Homonyma: stejné vi, různý význam, proto má každé svůj klic.
[[slovicka]]
vi = "đường"
klic = "đường (cesta)"
cs = "cesta, silnice"
vyslovnost_cs = "dɨəng (klesá)"
ipa = "ɗɯəŋ˧˨"

[[slovicka]]
vi = "đường"
klic = "đường (cukr)"
cs = "cukr"
vyslovnost_cs = "dɨəng (klesá)"
ipa = "ɗɯəŋ˧˨"
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

Lekce v repozitáři navíc kontroluje test (`cargo test`, modul `src/phonetics.rs`, etapa 8):
počet slabik ve `vi`, `ipa` a `vyslovnost_cs` musí souhlasit, tónová písmena v IPA a popis
tónu v závorce musí odpovídat diakritice ve `vi` (včetně zavřených slabik). Interpunkce
ve `vi` (např. otazník u frází) se přitom ignoruje. Aplikace tuto kontrolu při načítání
nedělá, ručně upravená lekce s jiným popisem tónu se tedy načte.

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
  *ma, mà, má, mả, mã, mạ* s významy. Od etapy 8 bere tóny z `src/phonetics.rs` (Kirby 2011)
  a ukazuje i tóny v zavřených slabikách.
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
- Statistiky se počítají při otevření záložky (přehrání historie je při velké historii
  pomalejší) a znovu jen při změně studijního dne.
- Úspěšnost se počítá z přehrané historie, stejně jako „zapomenutí“ v FSRS: hodnocení
  v jiný studijní den než předchozí hodnocení téže kartičky.
- Když přehraná historie nesedí s `postup.toml` (jiná stabilita nebo poslední opakování,
  např. po ruční úpravě dat), zaloguje se varování s počtem takových kartiček. Graf
  naučených pak může být nepřesný, ostatní grafy ne.
- Legenda grafů je vlastní, v řádku s nadpisem (legenda `egui_plot` řadí položky abecedně
  a překrývá graf).

## První sada lekcí (etapa 8)

Schválená témata, každá lekce zhruba 20 slovíček:

1. Pozdravy a zdvořilost
2. Zájmena a oslovení (anh, chị, em, ông, bà, tôi…)
3. Čísla
4. Základní fráze a otázky (có, không, bao nhiêu, ở đâu…)
5. Jídlo a pití
6. Objednávání v restauraci
7. Ptaní se na cestu ve městě

Upřesnění (etapa 8):

- IPA a český popis se připravily pomocným skriptem podle pravidel z oddílů „Český popis
  výslovnosti“ a „Konvence IPA“ a všechny slabiky (163) se porovnaly s nástrojem vPhon
  (Kirby). Shodují se v souhláskách, samohláskách, délce i tónech. Pak prošlo vše ruční
  kontrolou. vPhon (licence GPL) v repozitáři není.
- **Pomocný skript `tools/pronunciation.py`** (Python 3.11+, jen standardní knihovna,
  schváleno autorem) je v repozitáři. Vypíše `vyslovnost_cs` a `ipa` pro nové slovíčko
  (`python3 tools/pronunciation.py "cảm ơn"`) a s volbou `--check` porovná všechny lekce
  s pravidly. Hlídá i hlásky, test v Rustu (`src/phonetics.rs`) jen tóny a počet slabik.
  Výstup je návrh, přejímky a vzácné zápisy se kontrolují ručně. Při změně pravidel v tomto
  plánu je potřeba upravit skript i `src/phonetics.rs`.
- Slovní zásoba je severní (hanojská), u slov, která se na jihu říkají jinak, je to
  v poznámce (např. *bát* / jih *chén*).
- Homonyma *không* (ne / nula) mají `klic`. *đường* má `klic = "đường (cesta)"` už teď,
  aby později přidané *đường* (cukr) nezměnilo klíč a nepřipravilo cestu o postup.
- Fráze mají ve `vi` interpunkci (otazník, vykřičník), kontrola výslovnosti ji ignoruje.
- Noto Sans spojuje posloupnost tónových písmen do jednoho obrysu (např. ˧˩˨), jen ˧˧ se
  zobrazí jako dva stejné znaky.

## Druhá sada lekcí (etapa 9)

Schváleno autorem. První sada je situační a mnoho základních slov (slovesa, *là, người,
rất*…) se v ní objevuje jen uvnitř frází. Druhá sada doplňuje slovní druhy a gramatiku,
na které stojí stavba vlastních vět, a pak pokračuje dalšími situacemi.

Témata v tomto pořadí, každá lekce **20–30 slovíček**:

8. **Základní slovesa** (là, đi, đến, về, ăn, uống, nói, biết, hiểu, muốn, thích, cần,
   làm, học, mua, ở…). Odemknou skládání vět, řadu z nich žák zná z frází.
9. **Stavba věty: čas a zápor** (đã, đang, sẽ, rồi, chưa, cũng, rất, lắm, và, nhưng, với,
   của…). Vietnamština nečasuje, tyto částice časování nahrazují.
10. **Čas a kalendář** (hodiny, minuty, dnes, zítra, včera, části dne, dny v týdnu, měsíc,
    rok). Využije čísla z lekce 3 a částice z lekce 9.
11. **Přídavná jména a barvy** (velký, malý, hezký, drahý, levný, horký, studený…).
    Potřebná pro nakupování.
12. **Nakupování a smlouvání**, včetně počítacích slov (cái, con, chiếc, quả).
13. **Doprava a ubytování** (taxi, jízdenka, pokoj, rezervace…). Navazuje na lekci 7.
14. **O sobě a rodina** (manžel, manželka, sourozenci, věk, povolání). Rozšiřuje lekci 2.
15. **Zdraví a nouze** (bolí, lékař, lék, pomoc!, policie, části těla).

Pravidla pro obsah:

- Zhruba **2/3 jednotlivých slov a 1/3 krátkých frází** (nejvýš asi 6 slabik). Fráze jen
  tehdy, když jsou užitečné samy o sobě (*đắt quá!*), nebo ukazují gramatiku, kterou samo
  slovo nepředá (*ăn cơm chưa?*). Skládají se hlavně ze slov, která už v lekcích jsou.
- **Všechno jsou klasické kartičky**, i gramatické částice. `cs` je krátký význam
  (např. *đã* = „už (minulost)“), vysvětlení gramatiky a příkladové věty jsou
  v `poznamka`, ne na kartičce.
- Štítky navazují na první sadu (`sloveso`, `částice`, `čas`, `přídavné jméno`, `barva`,
  `nakupování`, `počítací slovo`, `doprava`, `ubytování`, `rodina`, `zdraví`, `nouze`,
  `fráze`…).

Homonyma a slova, která už v lekcích jsou:

- **Existující klíče se nikdy nemění.** Když přibude homonymum ke slovu bez `klic`,
  nové slovíčko dostane `klic` a staré si ponechá klíč `vi` (upravit se smí jen jeho
  `poznamka`, to postup nemaže). Pro *đường* (cukr) je připravený klíč `"đường (cukr)"`.
- **Samostatnou kartičku dostane jen opravdu jiný význam**, který stojí za to učit se
  zvlášť: *năm* (rok), *đường* (cukr), *nước* (země, stát), *con* (počítací slovo pro
  zvířata). Příbuzné významy jednoho slova (*bạn* = ty / kamarád, *có* = ano / mít)
  zůstávají na jedné kartičce, další význam je v poznámce.
- Slovo, které už je **uvnitř fráze**, smí dostat samostatnou kartičku (*ăn* vedle
  *tôi ăn chay*), klíče se liší.
- Stejné `vi` (bez nového významu) se podruhé nepřidává. Před psaním lekce se projdou
  klíče všech lekcí, duplicitu by odhalilo i načtení a `cargo test`.

Upřesnění (etapa 9, hotovo):

- Lekce 8–15 mají 231 slovíček (27–30 v lekci), v repozitáři je jich teď 375. Výslovnost
  vznikla skriptem `tools/pronunciation.py` a prošla ruční kontrolou, `--check` je bez
  rozdílů. Přejímky jsou ve vietnamském pravopisu (*tắc xi*, *ô tô*), pravidla je pokrývají.
- Lekce 10 (čas) má víc jednotlivých slov (dny v týdnu) a jen tři fráze, poměr 2/3 : 1/3
  je orientační.
- Nové klíče s `klic`: `"phải (muset)"`, `"năm (rok)"`, `"con (počítací slovo)"`,
  `"nước (země)"`. Starší *năm* (pět), *con* (dítě) a *nước* (voda) si ponechaly klíč `vi`.
  *phải* dostalo `klic` hned, aby pozdější *phải* (vpravo) nezměnilo klíč.
- Pravopis tónů zůstává jako v první sadě, tedy po staru (*hóa đơn*, *khỏe*, *chìa khóa*,
  *điều hòa*, *tàu hỏa*).
- Tísňové linky v poznámkách lekce 15: policie 113, hasiči 114, záchranka 115.

## Třetí sada lekcí (etapa 10)

Schváleno autorem. Třetí sada je pro turistu: jídlo a pití podrobněji, orientace, nápisy,
výlety, počasí, služby a povídání s místními. Platí pravidla pro obsah, klíče a homonyma
z oddílu „Druhá sada lekcí (etapa 9)“.

Témata v tomto pořadí, každá lekce **20–30 slovíček**:

16. **Vietnamská kuchyně a chutě** (bún chả, nem rán, chả cá, xôi, chutě, úprava, přání
    k objednávce, alergie). Navazuje na lekce 5 a 6.
17. **Ovoce, pití a kavárna** (ovoce z trhu, nước mía, sinh tố, bia hơi, cà phê trứng,
    přípitek). Hned po kuchyni, téma jídla zůstane pohromadě.
18. **Kde co je** (trên, dưới, trong, ngoài, bên cạnh, đối diện, tầng, vzdálenost). Hlavně
    předložky, tedy gramatika. Rozšiřuje lekci 7 a nápisy z ní berou *tầng*.
19. **Nápisy a cedule** (lối vào, lối ra, mở cửa, đóng cửa, cấm, đẩy, kéo, hết hàng…).
    Před památkami, aby z *mở cửa* a *đóng cửa* šlo v lekci 20 skládat otevírací dobu.
20. **Památky a výlety** (chùa, đền, bảo tàng, hồ, núi, vịnh, hang, vstupenka, fotit).
21. **Počasí a roční období** (mưa, nắng, gió, rét, ô, áo mưa, mùa…).
22. **Praktické služby a problémy** (výběr a výměna peněz, SIM, nabíječka, prádelna,
    kadeřník, masáž, pošta, problémy na pokoji).
23. **Povídání s místními** (poprvé ve Vietnamu, jak dlouho tu jsi, reakce, poděkování
    za pohostinnost). Poslední, protože používá slova ze všech předchozích lekcí, a proto
    má víc frází než ostatní.

Rozhodnutí:

- **Nové klíče s `klic`:** `"đường (cukr)"` (připravený v etapě 8), `"phải (vpravo)"`
  (vedle `"phải (muset)"`), a předem kvůli očekávaným homonymům `"lạc (arašídy)"`
  (*lạc* = zabloudit), `"nam (muži)"` (*nam* = jih) a `"trái (vlevo)"` (na jihu *trái* =
  ovoce).
- ***chè*** je jedna kartička. Oba významy (dezert, na severu i čaj) mají stejný původ,
  v poznámce je *chè xanh* a *trà đá*. Znění `cs` upravilo review, viz níže. *bơ* (avokádo / máslo) se nepřidává, *sinh tố
  bơ* je v poznámce u *sinh tố*.
- **Nápisy se procvičují oběma směry** jako ostatní lekce, formát ani aplikace se nemění.
  Lekce obsahuje jen nápisy složené z užitečných slov (*cấm*, *thuê*, *cửa*, *hết*), aby měl
  smysl i směr CS → VI. Mají štítek `nápis`.
- **Hovorové ceny** nejsou samostatné kartičky (slang, zdroje se u *xị*, *chai* a *vé*
  rozcházejí). Shodné údaje jsou v poznámce u *đổi tiền* a u *đồng* v lekci 12 (změna jen
  `poznamka`): na cedulkách se píše 50k = 50 000, hovorově *lít* = 100 000 a *củ* =
  1 000 000.
- **Latinkové zkratky** (*wifi*, *ATM*, *Zalo*) nejsou na kartičkách, neprošly by kontrolou
  výslovnosti. Jsou v poznámkách u *mạng*, *rút tiền* a *số điện thoại*.
- **Slova se spojovníkem** se zapisují bez něj (*mát xa*), pravopisná varianta je
  v poznámce.

Upřesnění (etapa 10, hotovo):

- Lekce 16–23 mají 214 slovíček (26–29 v lekci), v repozitáři je jich teď 589. Výslovnost
  vznikla skriptem `tools/pronunciation.py` a prošla ruční kontrolou (přejímky *sim*, *pin*,
  *mét*, *sạc*, *mát xa* i názvy jídel), `--check` je bez rozdílů.
- Lekce 23 má víc frází (11 z 27), protože je konverzační.
- Nové štítky: `chuť`, `vaření`, `ovoce`, `vzdálenost`, `nápis`, `výlet`, `počasí`,
  `služby`, `konverzace`. *cấm chụp ảnh* v lekci 20 má štítek `nápis`.
- U *đồng* v lekci 12 se doplnila jen `poznamka` (hovorové ceny), klíč zůstal.

## Review lekcí (po etapě 10)

Review správnosti všech 23 lekcí (589 slovíček), kód aplikace se neměnil.

Co review potvrdilo:

- Pravopis je jednotný: tóny po staru (*hòa, hóa, hỏa, khóa, khỏe*), *kỳ, tỷ, lý* s y,
  varianty *kì, tỉ* jen v poznámkách.
- `tools/pronunciation.py --check` je bez rozdílů, přejímky a vlastní jména (*sim, pin, mét,
  sạc, mát xa, Việt Nam, Hạ Long, buýt, Séc*) prošly ruční kontrolou.
- Klíče jsou unikátní, odkazy na jiné lekce v poznámkách sedí.

Co se opravilo:

- **Jediná změna `vi`:** *cà phê sữa đá* (lekce 5) je jižní název, kartička je teď hanojské
  ***nâu đá*** (schváleno autorem, kartička přišla o postup). Jižní název je v poznámce.
- **Zpřesnění `cs` a poznámek** (klíče beze změny): *tôi ở đây hai tuần* (bez *rồi* = „na dva
  týdny“), *bạn chụp giúp tôi nhé?* (tykání, zdvořilejší varianta v poznámce), *khuyến mãi*
  (akce, ne výprodej; i *khuyến mại*), *tháng* (duben je vždy *tháng tư*), *bao giờ*,
  *tạm biệt* (formální), *tính tiền* (i *thanh toán*), *xanh* (*xanh da trời*).
- ***chè*** má `cs` jen „sladký dezert“, čaj je v poznámce, štítek `jídlo` (nahrazuje
  rozhodnutí z etapy 10).
- **Kartičky, které se pletly ve směru CS → VI,** dostaly rozlišující `cs`: *sắp* / *sớm*,
  *thật à?* / *thế à?*, *hết* / *hết hàng*, *cửa* / *lối vào*, *cẩn thận!* / *chú ý*.
- **Jižní varianty** se v poznámkách píšou jednotně „jih: …“ a doplnily se chybějící
  (*xe lửa, máy lạnh, hẻm, lầu, bóp, ngò, má, coi, dĩa, gởi*). Učí se severní vietnamština,
  jižní varianty jsou jen pro porozumění.
- **Doslovné významy** v poznámkách jsou v uvozovkách („doslova „odnést domů““).
- **Štítek `fráze`** mají navíc i fráze v lekcích 6, 7, 19 a 20, které měly jen tematický
  štítek.

Co zůstalo:

- *bạn* se v `cs` překládá někde tykáním a někde vykáním (lekce 8, 13).
- Český popis výslovnosti nerozliší *uân* a *uôn* (obojí „uən“), rozdíl je jen v IPA.
- Hovorové ceny *lít* a *củ* se nepodařilo ověřit ve zdroji, odpovídají ale běžnému úzu.

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
tools/
  pronunciation.py  # pomocník pro lekce: IPA a český popis podle pravidel, kontrola lekcí
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
| 7 | Statistiky | hotovo |
| 8 | Lekce: první sada (pozdravy, čísla, základní fráze, jídlo, restaurace, cesta) | hotovo |
| 9 | Lekce: druhá sada (slovesa, stavba věty, čas, přídavná jména, nakupování, doprava, rodina, zdraví) | hotovo |
| 10 | Lekce: třetí sada pro turistu (kuchyně, ovoce a pití, kde co je, nápisy, památky, počasí, služby, povídání) | hotovo |
| – | Review správnosti lekcí 1–23 | hotovo |

## Otevřené otázky

- Ve směru CS → VI se na líci ukazuje i poznámka a ta často prozrazuje odpověď (u 111
  slovíček obsahuje přímo hledané slovo, např. u *đi* „đi bộ = jít pěšky“). U názvů jídel (*phở, bún chả, chả cá, xôi, lẩu, bia hơi*) je
  odpověď už v `cs`. Řešením by bylo ukazovat poznámku až na rubu, to je změna kódu.
- Neuložené změny v Nastavení se odchodem na jinou záložku zahodí (na obrazovce je jen
  upozornění). Případně doplnit potvrzovací dotaz.
- V logu jsou i záznamy knihoven (eframe, wgpu, rfd). Případné filtrování zatím neřešíme.
