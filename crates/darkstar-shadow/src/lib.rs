// darkstar-header-v1
// po co: lib.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-04
//! Tryb cienia: uczen patrzy, jak pracuje system, i nie ma jak w to ingerowac.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-04 23:40:00
//! REASON FOR CREATION: Noworodek ma sie docelowo nauczyc obslugiwac AIONS i pisac kod. Zanim
//! dostanie prawo glosu, ma patrzec: dostawac te same zadania co model dzialajacy, liczyc
//! swoja odpowiedz i oddawac ja do dziennika, nigdy do systemu.
//! MECHANICS: `ShadowedEmbedder` trzyma dwa silniki. Publiczna metoda `embed` czyta wylacznie
//! ten zywy - do ucznia nie ma z zewnatrz zadnej drogi, bo pole jest prywatne i nie ma
//! gettera. Odpowiedz ucznia liczona jest po fakcie, opakowana w `catch_unwind`, i idzie do
//! dziennika razem z czasem i zmierzona zgodnoscia.
//! SYSTEM PART: Darkstar Shadow / warstwa osadzen z wymiennymi silnikami.
//! ARCHITECTURE FUNCTION: Gniazdo, w ktore wpina sie dowolny model osadzen. Dzis zywy jest
//! all-MiniLM-L6-v2, uczniem Noworodek. Zamiana rol to podmiana dwoch argumentow konstruktora,
//! nie przepisanie systemu - dlatego warstwa powstaje teraz, a nie wtedy, gdy bedzie potrzebna.
//! DEPENDENCIES/LINKS: record.rs (format obserwacji), journal.rs (trwalosc). Konkretne silniki
//! zyja poza tym crate'em: ten definiuje wylacznie kontrakt i regule, ze cien nie wykonuje.
//! TECH STACK: Rust 2024 + serde. Prywatnosc pola jest tu mechanizmem bezpieczenstwa, nie
//! kwestia stylu: komentarz "nie wywoluj tego" mozna zignorowac, bledu kompilacji nie.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

pub mod journal;
pub mod record;

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    time::Instant,
};

pub use journal::{Counters, ShadowJournal};
pub use record::{ActionOutcome, Observation, ShadowRecord, cosine_similarity};

#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    #[error("model niedostepny: {0}")]
    Unavailable(String),
    #[error("blad liczenia: {0}")]
    Failed(String),
}

/// Cokolwiek, co zamienia zdanie na wektor.
///
/// Ten kontrakt jest celowo waski. Model osadzen ma robic jedna rzecz, a im mniej metod,
/// tym mniej trzeba zaimplementowac, zeby wpiac kolejny silnik.
pub trait Embedder: Send + Sync {
    fn name(&self) -> &str;
    /// Ile liczb ma zwracany wektor. Sluzy do wykrycia niezgodnosci z tym, czym policzono
    /// pamiec juz lezaca w magazynie - 384 wobec 128 znaczy, ze stare wspomnienia i nowe
    /// pytania nie trafiaja w ten sam swiat.
    fn dimensions(&self) -> usize;
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>;
}

/// Model dzialajacy plus opcjonalny uczen, ktory patrzy.
///
/// # Dlaczego uczen nie moze nic zepsuc
///
/// Pole `shadow` jest prywatne i nie ma metody, ktora by je zwracala. Jedyna publiczna
/// sciezka do wektora - [`ShadowedEmbedder::embed`] - czyta wylacznie `live`. Proba siegniecia
/// po ucznia z zewnatrz nie jest zla praktyka, tylko bledem kompilacji:
///
/// ```compile_fail
/// fn siegnij(e: &darkstar_shadow::ShadowedEmbedder) {
///     let _ = &e.shadow; // pole prywatne - kod sie nie zbuduje
/// }
/// ```
pub struct ShadowedEmbedder {
    live: Box<dyn Embedder>,
    shadow: Option<Box<dyn Embedder>>,
    journal: Option<Arc<ShadowJournal>>,
    session: Option<String>,
}

impl ShadowedEmbedder {
    /// Sam model dzialajacy, bez ucznia i bez dziennika.
    pub fn new(live: Box<dyn Embedder>) -> Self {
        Self {
            live,
            shadow: None,
            journal: None,
            session: None,
        }
    }

    /// Wpina ucznia. Od tej chwili dostaje te same zdania co model dzialajacy - i nadal
    /// nie ma jak wplynac na to, co system dostaje z powrotem.
    pub fn with_shadow(mut self, shadow: Box<dyn Embedder>) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub fn with_journal(mut self, journal: Arc<ShadowJournal>) -> Self {
        self.journal = Some(journal);
        self
    }

    pub fn with_session(mut self, session: impl Into<String>) -> Self {
        self.session = Some(session.into());
        self
    }

    pub fn live_name(&self) -> &str {
        self.live.name()
    }

    pub fn dimensions(&self) -> usize {
        self.live.dimensions()
    }

    /// Nazwa ucznia, jesli jakis patrzy. Zwraca sam tekst, nigdy silnika - podglad, kto
    /// obserwuje, nie moze byc furtka do uzycia go.
    pub fn shadow_name(&self) -> Option<&str> {
        self.shadow.as_ref().map(|s| s.name())
    }

    /// Jedyna droga do wektora. Wynik pochodzi wylacznie z modelu dzialajacego.
    ///
    /// Uczen liczy dopiero po tym, jak zywy skonczyl, i wylacznie wtedy, gdy zywy sie udal:
    /// obserwacja "co uczen powiedzialby zamiast bledu" nie uczy niczego, bo nie ma jej z czym
    /// porownac.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        let started = Instant::now();
        let live_vector = self.live.embed(text)?;
        let live_ms = started.elapsed().as_millis() as u64;

        self.observe(text, &live_vector, live_ms);

        Ok(live_vector)
    }

    /// Zapisuje, co uczen zrobilby z tym samym zdaniem. Nic nie zwraca - to jest cala regula.
    fn observe(&self, text: &str, live_vector: &[f32], live_ms: u64) {
        let (Some(shadow), Some(journal)) = (&self.shadow, &self.journal) else {
            return;
        };

        let started = Instant::now();
        // Uczen jest modelem w trakcie treningu: ma prawo sie wywrocic, a wywrotka w Rust
        // bywa panika, nie bledem. `catch_unwind` jest tu uczciwe, bo po panice nie dotykamy
        // zadnego stanu ucznia - wynik jest odrzucany, a dziennik ma wlasna obsluge zatrutego
        // zamka.
        //
        // CZEGO TO NIE OBEJMUJE, wprost: `catch_unwind` lapie panike i tylko panike. Uczen,
        // ktory sie ZAWIESI albo zje cala pamiec, zatrzyma tez watek wolajacego - bo obserwacja
        // jest tu synchroniczna. To znaczy, ze wpiecie ucznia o nieprzewidywalnym czasie
        // dzialania jest dzis decyzja o ryzyku, nie darmowym dodatkiem. Domkniecie tej dziury
        // wymaga osobnego procesu z wlasnym limitem czasu; do tego czasu wpinaj tylko ucznia,
        // ktoremu ufasz co do czasu odpowiedzi.
        let outcome = catch_unwind(AssertUnwindSafe(|| shadow.embed(text)));
        let shadow_ms = started.elapsed().as_millis() as u64;

        let (shadow_vector, shadow_error) = match outcome {
            Ok(Ok(v)) => (Some(v), None),
            Ok(Err(e)) => (None, Some(e.to_string())),
            Err(_) => (None, Some("uczen sie wywrocil (panika)".to_string())),
        };

        let record = ShadowRecord::new(
            now_unix_ms(),
            self.session.clone(),
            Observation::Embedding {
                text: text.to_string(),
                live_model: self.live.name().to_string(),
                shadow_model: shadow.name().to_string(),
                live_dims: live_vector.len(),
                shadow_dims: shadow_vector.as_ref().map(Vec::len),
                cosine: shadow_vector
                    .as_ref()
                    .and_then(|v| cosine_similarity(live_vector, v)),
                live_ms,
                shadow_ms: Some(shadow_ms),
                shadow_error,
            },
        );

        journal.append(&record);
    }

    /// Zapisuje, co zrobil agent. To jest material na cel wlasciwy - nauczyc sie obslugiwac
    /// system i pisac kod - i dziala niezaleznie od tego, czy jakis uczen jest wpiety.
    pub fn record_action(
        &self,
        actor: impl Into<String>,
        task: impl Into<String>,
        tool: Option<String>,
        input: serde_json::Value,
        outcome: ActionOutcome,
        summary: impl Into<String>,
    ) -> bool {
        let Some(journal) = &self.journal else {
            return false;
        };
        journal.append(&ShadowRecord::new(
            now_unix_ms(),
            self.session.clone(),
            Observation::Action {
                actor: actor.into(),
                task: task.into(),
                tool,
                input,
                outcome,
                summary: summary.into(),
            },
        ))
    }
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Model dzialajacy: zwraca staly, rozpoznawalny wektor.
    struct FakeLive;
    impl Embedder for FakeLive {
        fn name(&self) -> &str {
            "zywy"
        }
        fn dimensions(&self) -> usize {
            3
        }
        fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbedError> {
            Ok(vec![1.0, 0.0, 0.0])
        }
    }

    /// Uczen, ktory odpowiada calkiem czym innym. Gdyby jego glos gdziekolwiek przeciekal,
    /// zobaczylibysmy to w wyniku.
    struct LoudShadow;
    impl Embedder for LoudShadow {
        fn name(&self) -> &str {
            "uczen"
        }
        fn dimensions(&self) -> usize {
            3
        }
        fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbedError> {
            Ok(vec![-9.0, -9.0, -9.0])
        }
    }

    struct FailingShadow;
    impl Embedder for FailingShadow {
        fn name(&self) -> &str {
            "uczen-bledny"
        }
        fn dimensions(&self) -> usize {
            3
        }
        fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbedError> {
            Err(EmbedError::Failed("model jeszcze sie uczy".into()))
        }
    }

    struct PanickingShadow;
    impl Embedder for PanickingShadow {
        fn name(&self) -> &str {
            "uczen-wywrotka"
        }
        fn dimensions(&self) -> usize {
            3
        }
        fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbedError> {
            panic!("wagi jeszcze nie wczytane");
        }
    }

    /// Uczen o innej liczbie wymiarow - dokladnie przypadek Noworodka (128) wobec
    /// all-MiniLM-L6-v2 (384).
    struct NarrowShadow;
    impl Embedder for NarrowShadow {
        fn name(&self) -> &str {
            "uczen-waski"
        }
        fn dimensions(&self) -> usize {
            2
        }
        fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbedError> {
            Ok(vec![0.5, 0.5])
        }
    }

    fn temp_journal(name: &str) -> (std::path::PathBuf, Arc<ShadowJournal>) {
        let dir = std::env::temp_dir().join(format!("darkstar-shadow-lib-{name}-{}", std::process::id()));
        let path = dir.join("shadow.jsonl");
        let journal = Arc::new(ShadowJournal::open(&path).expect("dziennik"));
        (dir, journal)
    }

    fn read_records(path: &std::path::Path) -> Vec<ShadowRecord> {
        std::fs::read_to_string(path)
            .expect("odczyt dziennika")
            .lines()
            .map(|l| serde_json::from_str(l).expect("linia jest rekordem"))
            .collect()
    }

    #[test]
    fn the_answer_comes_from_the_live_model_only() {
        // Sedno calego rozwiazania: uczen krzyczy zupelnie co innego, a system tego nie slyszy.
        let (dir, journal) = temp_journal("isolation");
        let e = ShadowedEmbedder::new(Box::new(FakeLive))
            .with_shadow(Box::new(LoudShadow))
            .with_journal(Arc::clone(&journal));

        let got = e.embed("cokolwiek").expect("model zywy dziala");

        assert_eq!(got, vec![1.0, 0.0, 0.0], "odpowiedz musi pochodzic od modelu zywego");
        assert!(!got.contains(&-9.0), "glos ucznia nie moze przeciekac do wyniku");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_shadow_that_crashes_does_not_bring_the_system_down() {
        // Uczen w treningu ma prawo sie wywrocic. Jesli wywrotka przewraca system, cala
        // konstrukcja jest gorsza niz brak ucznia.
        let (dir, journal) = temp_journal("panic");
        let e = ShadowedEmbedder::new(Box::new(FakeLive))
            .with_shadow(Box::new(PanickingShadow))
            .with_journal(Arc::clone(&journal));

        let got = e.embed("zdanie").expect("wywrotka ucznia nie moze zepsuc odpowiedzi");
        assert_eq!(got, vec![1.0, 0.0, 0.0]);

        let records = read_records(journal.path());
        assert_eq!(records.len(), 1, "wywrotka tez jest obserwacja");
        match &records[0].observation {
            Observation::Embedding { shadow_error, cosine, .. } => {
                assert!(shadow_error.is_some(), "wywrotka musi zostac zapisana");
                assert_eq!(*cosine, None, "brak odpowiedzi to brak pomiaru, nie zero");
            }
            other => panic!("zly rodzaj obserwacji: {other:?}"),
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_failing_shadow_is_recorded_not_hidden() {
        let (dir, journal) = temp_journal("failure");
        let e = ShadowedEmbedder::new(Box::new(FakeLive))
            .with_shadow(Box::new(FailingShadow))
            .with_journal(Arc::clone(&journal));

        assert!(e.embed("zdanie").is_ok());

        let records = read_records(journal.path());
        match &records[0].observation {
            Observation::Embedding { shadow_error, .. } => {
                let msg = shadow_error.as_deref().unwrap_or_default();
                assert!(msg.contains("model jeszcze sie uczy"), "zapisano: {msg}");
            }
            other => panic!("zly rodzaj obserwacji: {other:?}"),
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn different_dimensions_are_recorded_without_a_fabricated_score() {
        // Noworodek liczy 128 wymiarow, model zywy 384. Obserwacja ma zostac zapisana,
        // ale bez liczby udajacej pomiar zgodnosci.
        let (dir, journal) = temp_journal("dims");
        let e = ShadowedEmbedder::new(Box::new(FakeLive))
            .with_shadow(Box::new(NarrowShadow))
            .with_journal(Arc::clone(&journal));

        e.embed("zdanie").expect("dziala");

        match &read_records(journal.path())[0].observation {
            Observation::Embedding { live_dims, shadow_dims, cosine, .. } => {
                assert_eq!(*live_dims, 3);
                assert_eq!(*shadow_dims, Some(2));
                assert_eq!(*cosine, None, "roznych wymiarow nie wolno porownywac");
            }
            other => panic!("zly rodzaj obserwacji: {other:?}"),
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn without_a_shadow_nothing_is_observed() {
        // Wpiecie ucznia jest decyzja, nie stanem domyslnym.
        let (dir, journal) = temp_journal("noshadow");
        let e = ShadowedEmbedder::new(Box::new(FakeLive)).with_journal(Arc::clone(&journal));

        e.embed("zdanie").expect("dziala");

        assert_eq!(journal.counters().written, 0, "bez ucznia nie ma czego obserwowac");
        assert_eq!(e.shadow_name(), None);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn agent_actions_are_recorded_even_without_a_shadow() {
        // Material na cel wlasciwy - obsluga systemu i pisanie kodu - zbiera sie niezaleznie
        // od tego, czy jakis model osadzen patrzy.
        let (dir, journal) = temp_journal("action");
        let e = ShadowedEmbedder::new(Box::new(FakeLive))
            .with_journal(Arc::clone(&journal))
            .with_session("claude_marcin_main");

        assert!(e.record_action(
            "recall-migrate",
            "przenies pamiec z Chromy",
            Some("http.put".into()),
            serde_json::json!({ "batch": 200 }),
            ActionOutcome::Success,
            "zapisano 200 punktow",
        ));

        let records = read_records(journal.path());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].session.as_deref(), Some("claude_marcin_main"));
        match &records[0].observation {
            Observation::Action { actor, outcome, .. } => {
                assert_eq!(actor, "recall-migrate");
                assert_eq!(*outcome, ActionOutcome::Success);
            }
            other => panic!("zly rodzaj obserwacji: {other:?}"),
        }

        std::fs::remove_dir_all(&dir).ok();
    }
}
