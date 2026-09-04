// darkstar-header-v1
// po co: journal.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-04
//! Dziennik obserwacji - miejsce, gdzie odklada sie material do nauki.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-04 23:35:00
//! REASON FOR CREATION: Obserwacje sa warte tyle, ile ich trwalosc. Bez zapisu na dysk uczen
//! widzi tylko biezaca sesje, a material zbiera sie tygodniami.
//! MECHANICS: Plik JSONL otwierany w trybie dopisywania - jedna obserwacja na linie, nigdy
//! nadpisania. Zapis jest za Mutexem, bo dziennik trafi pod wspoldzielony uchwyt w serwerze.
//! Bledy zapisu sa liczone, nie zglaszane w gore: cien nie moze przewrocic systemu, ale cicha
//! awaria musi byc widoczna, inaczej zbieramy zero i uwazamy, ze zbieramy.
//! SYSTEM PART: Darkstar Shadow / trwalosc zbioru uczacego.
//! ARCHITECTURE FUNCTION: Jedyne miejsce, ktore wie, gdzie i w jakim formacie leza obserwacje.
//! Trener czyta ten plik wprost, bez posrednika.
//! DEPENDENCIES/LINKS: record.rs (format linii), std::fs. Docelowa sciezka na Darkstarze:
//! /var/lib/darkstar/shadow/ - ext4, nigdy /mnt/aionslab (exFAT nie ma rzetelnych blokad pliku).
//! TECH STACK: Rust 2024 + std. Bez biblioteki do logow: format jest nasz, a kazda zaleznosc
//! to kolejna rzecz do przeniesienia na Redoxa.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::record::ShadowRecord;

/// Dziennik dopisujacy obserwacje do pliku JSONL.
pub struct ShadowJournal {
    path: PathBuf,
    file: Mutex<File>,
    /// Ile linii udalo sie zapisac i ile prob padlo. Za tym samym zamkiem co plik, wiec
    /// licznik nie moze sie rozjechac z rzeczywistoscia.
    counters: Mutex<Counters>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counters {
    pub written: u64,
    pub failed: u64,
}

impl ShadowJournal {
    /// Otwiera dziennik, zakladajac katalog nadrzedny, jesli go nie ma.
    ///
    /// Tryb `append` jest tu istotny: przy `write` kazdy restart serwera kasowalby material
    /// zebrany do tej pory, a to jest dokladnie ta awaria, ktorej nikt nie zauwaza od razu.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            file: Mutex::new(file),
            counters: Mutex::new(Counters::default()),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Dopisuje jedna obserwacje.
    ///
    /// Zwraca `false`, gdy zapis sie nie powiodl. Wolajacy moze to zignorowac - i zwykle
    /// powinien, bo obserwowanie nie jest zadaniem systemu - ale licznik `counters().failed`
    /// zostaje podniesiony, wiec cisza jest sprawdzalna.
    pub fn append(&self, record: &ShadowRecord) -> bool {
        let line = match serde_json::to_string(record) {
            Ok(l) => l,
            Err(_) => {
                self.bump(false);
                return false;
            }
        };
        // Zamek na czas samego zapisu: dwa watki dopisujace jednoczesnie moglyby przeplesc
        // polowki linii, a polowka linii psuje kazdy rekord po niej przy czytaniu.
        let ok = match self.file.lock() {
            Ok(mut f) => f.write_all(line.as_bytes()).and_then(|_| f.write_all(b"\n")).is_ok(),
            Err(_) => false,
        };
        self.bump(ok);
        ok
    }

    pub fn counters(&self) -> Counters {
        self.counters.lock().map(|c| *c).unwrap_or_default()
    }

    fn bump(&self, ok: bool) {
        if let Ok(mut c) = self.counters.lock() {
            if ok {
                c.written += 1;
            } else {
                c.failed += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{ActionOutcome, Observation};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("darkstar-shadow-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("katalog testowy");
        dir
    }

    fn sample(n: u64) -> ShadowRecord {
        ShadowRecord::new(
            1_757_030_000_000 + n as i64,
            Some("test".into()),
            Observation::Action {
                actor: "test".into(),
                task: format!("zadanie {n}"),
                tool: None,
                input: serde_json::json!({}),
                outcome: ActionOutcome::Success,
                summary: "ok".into(),
            },
        )
    }

    #[test]
    fn each_observation_is_exactly_one_line() {
        let dir = temp_dir("lines");
        let path = dir.join("shadow.jsonl");
        let journal = ShadowJournal::open(&path).expect("otwarcie");

        for n in 0..3 {
            assert!(journal.append(&sample(n)), "zapis {n}");
        }

        let content = std::fs::read_to_string(&path).expect("odczyt");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3, "trzy obserwacje = trzy linie");
        for line in lines {
            serde_json::from_str::<ShadowRecord>(line).expect("kazda linia jest osobnym rekordem");
        }
        assert_eq!(journal.counters(), Counters { written: 3, failed: 0 });

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reopening_appends_instead_of_erasing() {
        // Ta wlasnosc jest cala wartoscia dziennika: restart serwera nie moze skasowac
        // materialu zbieranego tygodniami.
        let dir = temp_dir("append");
        let path = dir.join("shadow.jsonl");

        {
            let first = ShadowJournal::open(&path).expect("pierwsze otwarcie");
            assert!(first.append(&sample(1)));
        }
        {
            let second = ShadowJournal::open(&path).expect("drugie otwarcie");
            assert!(second.append(&sample(2)));
        }

        let content = std::fs::read_to_string(&path).expect("odczyt");
        assert_eq!(content.lines().count(), 2, "drugie otwarcie dopisalo, nie nadpisalo");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_parent_directory_is_created() {
        let dir = temp_dir("nested");
        let path = dir.join("a").join("b").join("shadow.jsonl");
        let journal = ShadowJournal::open(&path).expect("otwarcie z zakladaniem katalogow");
        assert!(journal.append(&sample(0)));
        assert!(path.exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
