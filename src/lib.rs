//! Logika aplikace Vietnamská slovíčka, oddělená od uživatelského rozhraní.
//!
//! Binárka (`main.rs`) obsahuje jen GUI a spuštění. Všechno, co jde testovat
//! bez okna, patří sem.

pub mod fsrs;
pub mod lessons;
pub mod model;
pub mod paths;
pub mod progress;
pub mod scheduler;
pub mod session;
pub mod settings;
pub mod storage;

#[cfg(test)]
mod test_util;
