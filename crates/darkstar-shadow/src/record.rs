// darkstar-header-v1
// po co: record.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-04
//! Zapis pojedynczej obserwacji - co uczen zobaczyl i co sam by zrobil.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-04 23:30:00
//! REASON FOR CREATION: Noworodek ma sie docelowo nauczyc obslugiwac AIONS i pisac kod, a nie
//! gadac. Do tego potrzebuje patrzec, jak robia to model osadzen i agenci - zanim dostanie
//! prawo glosu. Ten plik definiuje, co dokladnie zostaje zapisane z kazdej takiej obserwacji.
//! MECHANICS: Jeden rekord = jedno zdarzenie. Dwa rodzaje: ObservedEmbedding (to samo zdanie
//! policzone przez model zywy i przez ucznia, plus zmierzona zgodnosc) oraz ObservedAction
//! (agent dostal zadanie, zrobil X, wyszlo Y). Rekordy sa niezmienne i serializowane do JSONL -
//! formatu, ktory czytaja trenery bez zadnej konwersji.
//! SYSTEM PART: Darkstar Shadow / format zbioru uczacego.
//! ARCHITECTURE FUNCTION: Kontrakt miedzy tym, co system robi dzis, a tym, czego uczy sie
//! model, ktory ma to robic jutro. Zmiana tego formatu uniewaznia zebrany material, wiec
//! pola sa dodawane, nigdy przemianowywane.
//! DEPENDENCIES/LINKS: serde, serde_json; wzorowane na darkstar_core::audit::AuditRecord -
//! ten sam uklad pol (kto, co, wynik, metadane), inny cel: audyt sluzy sledzeniu, ten zbior
//! sluzy nauce, wiec moze byc probkowany i kasowany, a audyt nie.
//! TECH STACK: Rust 2024 + serde. Bez nowych zaleznosci - kazda to kolejna rzecz do
//! przeniesienia na Redoxa.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Co uczen zobaczyl. Jeden wariant na rodzaj zdarzenia.
///
/// `#[serde(tag = "kind")]` daje w pliku plaski obiekt z polem rozroznajacym zamiast
/// zagniezdzenia - trener czyta `kind` i wie, jak potraktowac reszte linii.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Observation {
    /// To samo zdanie policzone dwa razy: przez model, ktory dzis obsluguje pamiec,
    /// i przez ucznia. Material do odpowiedzi na pytanie "czy uczen juz dorownuje".
    Embedding {
        /// Tekst wejsciowy. To jest zadanie; wektory ponizej to dwie odpowiedzi na nie.
        text: String,
        /// Nazwa modelu, ktory dzis obsluguje system.
        live_model: String,
        /// Nazwa ucznia.
        shadow_model: String,
        live_dims: usize,
        shadow_dims: Option<usize>,
        /// Zgodnosc kierunku obu wektorow w zakresie -1..1. `None`, gdy uczen nie odpowiedzial
        /// albo gdy wymiary sie roznia i porownanie nie ma sensu bez rzutowania.
        cosine: Option<f32>,
        /// Ile milisekund zajelo to zywemu i uczniowi. Uczen wolniejszy o rzad wielkosci
        /// nie nadaje sie do zamiany, choc by nawet trafial lepiej.
        live_ms: u64,
        shadow_ms: Option<u64>,
        /// Wypelnione, gdy uczen sie wywrocil. To jest cenna obserwacja, nie awaria.
        shadow_error: Option<String>,
    },

    /// Agent dostal zadanie i cos zrobil. To jest material na cel wlasciwy: nauczyc sie
    /// obslugiwac system i pisac kod, a nie prowadzic rozmowe.
    Action {
        /// Kto dzialal - nazwa agenta albo narzedzia.
        actor: String,
        /// Co mial zrobic, wlasnymi slowami zlecajacego.
        task: String,
        /// Nazwa uzytego narzedzia, jesli dzialanie sprowadzalo sie do jego wywolania.
        tool: Option<String>,
        /// Argumenty wywolania. Dowolny JSON, bo kazde narzedzie ma inny ksztalt.
        input: Value,
        /// Co z tego wyszlo.
        outcome: ActionOutcome,
        /// Skrocony wynik. Pelnego nie zapisujemy: zbior uczacy ma byc czytelny, a nie
        /// archiwum. Pelny wynik i tak zyje w audycie.
        summary: String,
    },
}

/// Czy dzialanie sie udalo. Osobny typ zamiast `bool`, bo "nie wiadomo" jest trzecim
/// prawidlowym stanem, a `Option<bool>` nie mowi czytelnikowi, co znaczy `None`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcome {
    Success,
    Failure,
    /// Zdarzenie zapisane, zanim wynik byl znany. Uczen ma widziec takze to, ze system
    /// czasem nie wie - inaczej nauczy sie, ze pewnosc jest zawsze uzasadniona.
    Unknown,
}

/// Jedna linia w dzienniku: obserwacja plus kiedy i w jakiej sesji zaszla.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShadowRecord {
    pub timestamp_unix_ms: i64,
    /// Sesja, z ktorej pochodzi zdarzenie. Pozwala pozniej odtworzyc kolejnosc i kontekst -
    /// pojedyncze dzialanie w oderwaniu uczy mniej niz ciag dzialan.
    pub session: Option<String>,
    pub observation: Observation,
}

impl ShadowRecord {
    pub fn new(timestamp_unix_ms: i64, session: Option<String>, observation: Observation) -> Self {
        Self {
            timestamp_unix_ms,
            session,
            observation,
        }
    }
}

/// Zgodnosc kierunku dwoch wektorow, w zakresie -1..1.
///
/// Zwraca `None`, gdy dlugosci sie roznia albo ktorys wektor jest zerowy - w obu przypadkach
/// wynik nie mialby znaczenia, a zwrocenie 0.0 udawaloby zmierzona odpowiedz tam, gdzie
/// pomiaru nie bylo.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let mut dot = 0.0f64;
    let mut norm_a = 0.0f64;
    let mut norm_b = 0.0f64;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += f64::from(*x) * f64::from(*y);
        norm_a += f64::from(*x) * f64::from(*x);
        norm_b += f64::from(*y) * f64::from(*y);
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return None;
    }
    Some((dot / (norm_a.sqrt() * norm_b.sqrt())) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_vectors_are_fully_aligned() {
        let v = [1.0, 2.0, 3.0];
        let got = cosine_similarity(&v, &v).expect("ten sam wektor musi dac wynik");
        assert!((got - 1.0).abs() < 1e-6, "oczekiwano 1.0, jest {got}");
    }

    #[test]
    fn opposite_vectors_are_fully_opposed() {
        let got = cosine_similarity(&[1.0, 0.0], &[-1.0, 0.0]).expect("wynik");
        assert!((got + 1.0).abs() < 1e-6, "oczekiwano -1.0, jest {got}");
    }

    #[test]
    fn mismatched_dimensions_give_no_answer_instead_of_a_fake_one() {
        // Noworodek liczy 128 wymiarow, model zywy 384. Bez tego strazniku porownanie
        // zwrociloby liczbe, ktora wyglada jak pomiar, a nim nie jest.
        assert_eq!(cosine_similarity(&[1.0; 128], &[1.0; 384]), None);
        assert_eq!(cosine_similarity(&[], &[]), None);
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[1.0, 1.0]), None);
    }

    #[test]
    fn record_round_trips_through_jsonl() {
        // Format zapisu jest kontraktem ze zbiorem uczacym. Jesli rekord nie wraca z pliku
        // taki sam, jaki tam poszedl, material zbierany tygodniami jest bezwartosciowy.
        let record = ShadowRecord::new(
            1_757_030_000_000,
            Some("claude_marcin_main".into()),
            Observation::Embedding {
                text: "jak dziala magazyn pamieci".into(),
                live_model: "all-MiniLM-L6-v2".into(),
                shadow_model: "noworodek".into(),
                live_dims: 384,
                shadow_dims: Some(128),
                cosine: None,
                live_ms: 12,
                shadow_ms: Some(3),
                shadow_error: None,
            },
        );
        let line = serde_json::to_string(&record).expect("zapis");
        assert!(!line.contains('\n'), "linia JSONL nie moze zawierac znaku nowej linii");
        let back: ShadowRecord = serde_json::from_str(&line).expect("odczyt");
        assert_eq!(record, back);
    }

    #[test]
    fn action_round_trips_and_keeps_its_kind() {
        let record = ShadowRecord::new(
            1_757_030_000_001,
            None,
            Observation::Action {
                actor: "recall-migrate".into(),
                task: "przenies kolekcje pamieci".into(),
                tool: Some("http.put".into()),
                input: serde_json::json!({ "collection": "przyklad", "points": 200 }),
                outcome: ActionOutcome::Success,
                summary: "zapisano 200 punktow".into(),
            },
        );
        let line = serde_json::to_string(&record).expect("zapis");
        assert!(line.contains("\"kind\":\"action\""), "brak znacznika rodzaju: {line}");
        let back: ShadowRecord = serde_json::from_str(&line).expect("odczyt");
        assert_eq!(record, back);
    }
}
