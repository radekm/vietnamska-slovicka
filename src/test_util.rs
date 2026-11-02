//! Pomůcky sdílené testy více modulů.

use std::path::{Path, PathBuf};

/// Dočasná složka pro test, po skončení testu se smaže.
///
/// Název musí být v rámci všech testů unikátní, testy běží souběžně.
pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "vietnamska-slovicka-test-{}-{name}",
            std::process::id()
        ));
        // Pozůstatek po předchozím přerušeném běhu odstraníme.
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }

    /// Cesta k souboru uvnitř složky.
    pub fn join(&self, file_name: &str) -> PathBuf {
        self.0.join(file_name)
    }

    pub fn write(&self, file_name: &str, content: &str) {
        std::fs::write(self.0.join(file_name), content).unwrap();
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
