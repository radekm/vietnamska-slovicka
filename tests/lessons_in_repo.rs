//! Kontrola lekcí ve složce `slovnicky/` v repozitáři: načtení bez chyb a výslovnost
//! (počet slabik, tóny v IPA a v českém popisu, viz `PLAN.md`).

use std::path::Path;

use vietnamska_slovicka::lessons;
use vietnamska_slovicka::phonetics;

#[test]
fn lessons_in_repository_are_valid() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("slovnicky");
    let lessons = lessons::load_dir(&dir).unwrap_or_else(|err| panic!("{err:#}"));
    assert!(
        !lessons.is_empty(),
        "ve složce {} nejsou lekce",
        dir.display()
    );

    let mut problems = Vec::new();
    for lesson in &lessons {
        for (index, word) in lesson.words.iter().enumerate() {
            for problem in phonetics::check_word(word) {
                problems.push(format!(
                    "{}, slovíčko č. {} („{}“): {problem}",
                    lesson.file_name,
                    index + 1,
                    word.vi
                ));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "problémy ve výslovnosti:\n{}",
        problems.join("\n")
    );
}
