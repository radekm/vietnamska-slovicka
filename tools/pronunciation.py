#!/usr/bin/env python3
"""Výslovnost vietnamských slovíček pro lekce: IPA a český popis.

Převádí vietnamský pravopis do IPA (hanojská výslovnost, konvence Kirby 2011) a do českého
popisu výslovnosti podle pravidel v PLAN.md (oddíly „Český popis výslovnosti“ a „Konvence
IPA“). Potřebuje jen Python 3.11 nebo novější, bez dalších balíčků.

Použití:

    python3 tools/pronunciation.py "cảm ơn" "bạn khỏe không?"
        Vypíše řádky vyslovnost_cs a ipa pro každé slovíčko, připravené pro lekci.

    python3 tools/pronunciation.py --check [složka]
        Porovná vyslovnost_cs a ipa všech slovíček v lekcích (výchozí složka slovnicky/)
        s tím, co vyjde z pravidel, a vypíše rozdíly. Při rozdílech skončí s kódem 1.

Výstup je návrh, ne poslední slovo: přejímky a vzácné zápisy (např. *xẻng* s pravým
zadopatrovým -ng po e, *boong*) pravidla nepokrývají a je potřeba je opravit ručně. Ručně
opravené slovíčko pak `--check` ohlásí jako rozdíl, to je v pořádku.
"""

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path

# Samohlásky se všemi tóny v pořadí ngang, huyền, sắc, hỏi, ngã, nặng.
TONED_VOWELS = ["aàáảãạ", "ăằắẳẵặ", "âầấẩẫậ", "eèéẻẽẹ", "êềếểễệ", "iìíỉĩị", "oòóỏõọ",
                "ôồốổỗộ", "ơờớởỡợ", "uùúủũụ", "ưừứửữự", "yỳýỷỹỵ"]
VOWELS = set("aăâeêioôơuưy")

# Tón podle (číslo tónu, zavřená slabika). Musí odpovídat src/phonetics.rs.
CHAO = {(0, False): "˧˧", (1, False): "˧˨", (2, False): "˨˦", (2, True): "˦˥",
        (3, False): "˧˩˨", (4, False): "˧ˀ˥", (5, False): "˨˩ˀ", (5, True): "˨˩"}
TONE_DESCRIPTION = {(0, False): "rovně", (1, False): "klesá", (2, False): "stoupá",
                    (2, True): "stoupá", (3, False): "klesá hluboko",
                    (4, False): "stoupá se zlomem", (5, False): "krátce klesá, ráz",
                    (5, True): "krátce klesá"}

# Počáteční souhlásky, delší zápisy první (ngh před ng, ng před n…).
ONSETS = ["ngh", "ng", "nh", "ch", "tr", "th", "ph", "kh", "gh", "gi", "qu",
          "b", "c", "d", "đ", "g", "h", "k", "l", "m", "n", "p", "r", "s", "t", "v", "x"]
ONSET_IPA = {"b": "ɓ", "c": "k", "k": "k", "d": "z", "đ": "ɗ", "g": "ɣ", "gh": "ɣ",
             "gi": "z", "h": "h", "kh": "x", "l": "l", "m": "m", "n": "n", "ng": "ŋ",
             "ngh": "ŋ", "nh": "ɲ", "p": "p", "ph": "f", "r": "z", "s": "s", "t": "t",
             "th": "tʰ", "tr": "t͡ɕ", "ch": "t͡ɕ", "v": "v", "x": "s", "qu": "k", "": "ʔ"}
ONSET_CS = {"b": "b", "c": "k", "k": "k", "d": "z", "đ": "d", "g": "g", "gh": "g",
            "gi": "z", "h": "h", "kh": "ch", "l": "l", "m": "m", "n": "n", "ng": "ng",
            "ngh": "ng", "nh": "ň", "p": "p", "ph": "f", "r": "z", "s": "s", "t": "t",
            "th": "th", "tr": "č", "ch": "č", "v": "v", "x": "s", "qu": "k", "": ""}

# Jádro slabiky: pravopis → IPA. Dvojhlásky se zkoušejí dřív než jednotlivé samohlásky.
NUCLEUS_IPA = {"i": "i", "y": "i", "ê": "e", "e": "ɛ", "ư": "ɯ", "ơ": "ɤ", "â": "ɤ̆",
               "a": "a", "ă": "ă", "u": "u", "ô": "o", "o": "ɔ", "iê": "iə", "yê": "iə",
               "ia": "iə", "ya": "iə", "ươ": "ɯə", "ưa": "ɯə", "uô": "uə", "ua": "uə"}
NUCLEUS_PATTERN = re.compile(r"(iê|yê|ươ|uô|ia|ya|ưa|ua|[aăâeêioôơuưy])(.*)$")
# Dvojhlásky zapsané s -a jsou jen na konci slabiky (bia, mưa, mua).
OPEN_ONLY = {"ia", "ya", "ưa", "ua"}

# IPA → český popis.
NUCLEUS_CS = {"i": "i", "e": "e", "ɛ": "e", "ɯ": "ɨ", "ɤ": "ə", "ɤ̆": "ə", "a": "a",
              "ă": "a", "u": "u", "o": "o", "ɔ": "o", "iə": "iə", "ɯə": "ɨə", "uə": "uə"}
CODA_IPA = {"": "", "i": "j", "y": "j", "o": "w", "u": "w", "c": "k", "ch": "k̟",
            "nh": "ŋ̟", "ng": "ŋ", "m": "m", "n": "n", "p": "p", "t": "t"}
CODA_CS = {"": "", "j": "j", "w": "u", "k": "k", "k̟": "k", "k͡p": "k", "ŋ": "ng",
           "ŋ̟": "ng", "ŋ͡m": "ng", "m": "m", "n": "n", "p": "p", "t": "t"}


def strip_tone(syllable):
    """Vrátí slabiku bez značky tónu a číslo tónu (0 = ngang … 5 = nặng)."""
    base, tone = "", 0
    for char in syllable:
        for vowels in TONED_VOWELS:
            if char in vowels:
                index = vowels.index(char)
                if index:
                    if tone:
                        raise ValueError(f"slabika „{syllable}“ má víc značek tónu")
                    tone = index
                char = vowels[0]
                break
        base += char
    return base, tone


def parse(syllable):
    """Rozloží slabiku (malými písmeny) na části potřebné pro přepis."""
    base, tone = strip_tone(syllable)
    onset = next((o for o in ONSETS if base.startswith(o)), "")
    rest = base[len(onset):]
    # gì, gìn: i patří k jádru, gi je jen souhláska z.
    if onset == "gi" and not any(c in VOWELS for c in rest):
        rest = "i" + rest

    # Vložené o/u (hoa, khỏe, quá, thuý, xuân).
    medial = onset == "qu"
    if not medial and (rest.startswith(("oa", "oă", "oe"))
                       or rest.startswith(("uê", "uơ", "uâ", "uy"))):
        medial, rest = True, rest[1:]

    match = NUCLEUS_PATTERN.match(rest)
    if not match:
        raise ValueError(f"slabika „{syllable}“ nemá samohlásku")
    nucleus, coda = match.groups()
    if coda not in CODA_IPA or (nucleus in OPEN_ONLY and coda):
        raise ValueError(f"slabiku „{syllable}“ pravidla neumí přepsat")

    nucleus_ipa = NUCLEUS_IPA[nucleus]
    coda_ipa = CODA_IPA[coda]
    if nucleus == "a" and coda in ("y", "u"):
        nucleus_ipa = "ă"  # ay = ăj, au = ăw
    if nucleus == "a" and coda in ("nh", "ch"):
        nucleus_ipa = "ɛ"  # anh = ɛŋ̟, ach = ɛk̟
    if nucleus in ("u", "ô", "o") and coda in ("c", "ng") and not medial:
        # Po zaokrouhlené samohlásce retně-zadopatrové (Kirby 2011).
        coda_ipa = {"c": "k͡p", "ng": "ŋ͡m"}[coda]
    checked = coda in ("p", "t", "c", "ch")
    if checked and tone not in (2, 5):
        raise ValueError(f"slabika „{syllable}“ končí na p, t, c nebo ch, "
                         "a může mít jen tón sắc nebo nặng")
    return {"onset": onset, "medial": medial, "nucleus": nucleus,
            "nucleus_ipa": nucleus_ipa, "coda_ipa": coda_ipa,
            "tone": (tone, checked)}


def syllable_ipa(syllable):
    p = parse(syllable)
    if p["onset"] == "" and p["medial"]:
        head = "w"  # oan → wan
    else:
        head = ONSET_IPA[p["onset"]] + ("ʷ" if p["medial"] else "")
    return head + p["nucleus_ipa"] + p["coda_ipa"] + CHAO[p["tone"]]


def syllable_cs(syllable):
    p = parse(syllable)
    head = ONSET_CS[p["onset"]] + ("u" if p["medial"] else "")
    if p["nucleus"] == "a" and p["nucleus_ipa"] == "ɛ":
        body = "aj" + CODA_CS[p["coda_ipa"]]  # anh → ajng, ách → ajk
    else:
        body = NUCLEUS_CS[p["nucleus_ipa"]] + CODA_CS[p["coda_ipa"]]
    return f"{head}{body} ({TONE_DESCRIPTION[p['tone']]})"


def syllables(text):
    """Slabiky textu malými písmeny, bez interpunkce na okrajích (jako src/phonetics.rs)."""
    result = []
    for token in text.split():
        token = re.sub(r"^\W+|\W+$", "", token)
        if token:
            result.append(token.lower())
    return result


def pronunciation(text):
    """Vrátí (vyslovnost_cs, ipa) pro vietnamský text."""
    parts = syllables(text)
    return (" ".join(syllable_cs(s) for s in parts),
            " ".join(syllable_ipa(s) for s in parts))


def print_entries(texts):
    for text in texts:
        try:
            cs, ipa = pronunciation(text)
        except ValueError as error:
            print(f"# {text}: {error}", file=sys.stderr)
            continue
        print(f"vi = {json.dumps(text, ensure_ascii=False)}")
        print(f"vyslovnost_cs = {json.dumps(cs, ensure_ascii=False)}")
        print(f"ipa = {json.dumps(ipa, ensure_ascii=False)}")
        print()


def check(directory):
    files = sorted(Path(directory).glob("lekce*.toml"))
    if not files:
        print(f"ve složce {directory} nejsou lekce", file=sys.stderr)
        return 1
    problems = 0
    for path in files:
        lesson = tomllib.loads(path.read_text(encoding="utf-8"))
        for index, word in enumerate(lesson.get("slovicka", []), start=1):
            where = f"{path.name}, slovíčko č. {index} („{word['vi']}“)"
            try:
                cs, ipa = pronunciation(word["vi"])
            except ValueError as error:
                print(f"{where}: {error}")
                problems += 1
                continue
            if word.get("vyslovnost_cs") != cs:
                print(f"{where}: vyslovnost_cs\n    je:      {word.get('vyslovnost_cs')}"
                      f"\n    pravidla: {cs}")
                problems += 1
            if word.get("ipa") != ipa:
                print(f"{where}: ipa\n    je:      {word.get('ipa')}\n    pravidla: {ipa}")
                problems += 1
    print(f"rozdílů: {problems}")
    return 1 if problems else 0


def main():
    parser = argparse.ArgumentParser(
        description="IPA a český popis výslovnosti vietnamských slovíček (viz PLAN.md).")
    parser.add_argument("texts", nargs="*", help="vietnamská slovíčka nebo fráze")
    parser.add_argument("--check", nargs="?", const="slovnicky", metavar="SLOŽKA",
                        help="zkontrolovat lekce ve složce (výchozí slovnicky/)")
    args = parser.parse_args()
    if args.check is not None:
        sys.exit(check(args.check))
    if not args.texts:
        parser.print_help()
        sys.exit(2)
    print_entries(args.texts)


if __name__ == "__main__":
    main()
