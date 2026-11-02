# Pravidla pro práci na projektu

Tento soubor popisuje závazná pravidla pro každého, kdo na projektu pracuje (člověka i AI agenta).
Před začátkem práce si vždy přečti tento soubor a `PLAN.md`.

## Jazyky

- **Komunikace s autorem:** pouze česky.
- **Kód** (názvy proměnných, funkcí, typů, modulů, souborů se zdrojáky): anglicky.
- **Komentáře v kódu:** česky.
- **Dokumentace** (`README.md`, `PLAN.md`, tento soubor, dokumentační komentáře `///`): česky.
- **Texty v uživatelském rozhraní:** česky.
- **Obsah lekcí:** vietnamština (severní, hanojská výslovnost), čeština, přesný přepis IPA.

## Když si nejsi jistý

Raději se zeptej autora, než abys hádal. Platí to hlavně pro:

- přidání nové crate nebo nové feature existující crate,
- přidání nového flagu (kompilátoru, Cargo, `RUSTFLAGS`, profilů v `Cargo.toml`),
- změny datového formátu lekcí nebo souboru s postupem učení (hrozí ztráta dat),
- odchylky od `PLAN.md`.

## Jazyk Rust

- Edice Rust **2024**. Toolchain je zafixovaný v `rust-toolchain.toml` na verzi **1.99.1**.
  Změnu verze schvaluje autor.
- Žádný `unsafe` kód.
- Chyby se propagují přes `anyhow::Result`. `unwrap()` a `expect()` jsou povoleny jen v testech
  a tam, kde je chyba prokazatelně nemožná (s komentářem proč).
- Před commitem musí projít:
  - `cargo fmt --check`
  - `cargo clippy --all-targets` bez varování
  - `cargo test`
- Logika (algoritmus opakování, načítání a ukládání dat) musí být oddělená od GUI a pokrytá testy.

## Povolené crates

Používat se smí **pouze** následující crates, **přesně** v uvedených verzích a s uvedenými features.
Cokoliv dalšího (i nová feature) vyžaduje předchozí souhlas autora.

```toml
anyhow = "=1.0.104"
time = { version = "=0.3.55", default-features = false, features = ["std", "macros", "formatting", "parsing"] }
serde = { version = "=1.0.229", default-features = false, features = ["std", "derive"] }

tokio = { version = "=1.54.1", default-features = false, features = ["rt", "macros", "time"] }
tracing = "=0.1.44"
tracing-subscriber = { version = "=0.3.23", features = ["json"] }
tracing-appender = "=0.2.5"

eframe = "=0.37.0"
egui_extras = { version = "=0.37.0", default-features = false }
egui_plot = "=0.38.0"
rfd = "=0.17.2"

toml = "=1.1.8"
```

Povolené crates se nemusí používat všechny. Co není potřeba, do `Cargo.toml` nepřidávej.

## Povolené flagy

Zatím žádné. V `Cargo.toml` se nepřidávají vlastní profily ani `[lints]` a nepoužívá se
`.cargo/config.toml` ani `RUSTFLAGS`, dokud je autor nepovolí.

## Data

- Lekce jsou v TOML souborech `lekce01.toml`, `lekce02.toml`, … ve složce se slovníčky
  (v repozitáři `slovnicky/`). Formát popisuje `PLAN.md`.
- Klíčem slovíčka je vietnamské slovo (`vi`). Volitelné pole `klic` slouží **jen pro
  homonyma** (stejně psaná slova s jiným významem, např. *đường* = cesta / cukr), aby každé
  mělo vlastní postup učení. Klíč musí být unikátní napříč všemi lekcemi. **Změna `vi` nebo `klic` u existujícího slovíčka
  znamená ztrátu jeho postupu**, proto se dělá jen vědomě a po dohodě s autorem.
- Postup učení, nastavení a logy se ukládají mimo složku se slovníčky, do datové složky
  aplikace (viz `PLAN.md`).
- Ukládání musí být bezpečné: zápis do dočasného souboru a pak přejmenování.

## Logování

- Strukturované logování přes `tracing` (pole místo skládání textu, např.
  `info!(lesson = %name, count = n, "lekce načtena")`). Názvy polí anglicky, zprávy česky.
- Logy se ukládají jako JSON do datové složky aplikace, rotují se po dnech a drží se posledních 7.

## Git

- Malé, samostatné commity, které se dají přeložit.
- Commit zprávy česky, první řádek stručné shrnutí.
- Na začátku každé etapy z `PLAN.md` a po jejím dokončení aktualizuj stav v `PLAN.md`.
