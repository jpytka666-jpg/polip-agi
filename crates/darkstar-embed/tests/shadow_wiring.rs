// darkstar-header-v1
// po co: shadow_wiring.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Wpiecie ucznia: obaj w gniezdzie, glos ma tylko jeden.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 14:45:00
//! REASON FOR CREATION: Gniazdo, model zywy i uczen powstawaly osobno i kazde dziala we
//! wlasnych testach. Ten sprawdza jedyna rzecz, ktorej zaden z nich sprawdzic nie moze:
//! czy dzialaja RAZEM - i czy obecnosc ucznia niczego nie zmienia w odpowiedzi.
//! MECHANICS: Wczytuje oba silniki, wpina Noworodka jako ucznia do ShadowedEmbedder, liczy
//! to samo zdanie przed wpieciem i po, porownuje co do bitu. Sprawdza takze dziennik: czy
//! uczen zostal odpytany i czy jego rozmiar wektora zostal zanotowany jako inny niz zywego.
//! SYSTEM PART: Darkstar Embed / test wpiecia.
//! ARCHITECTURE FUNCTION: Bramka. Kazda zmiana w gniezdzie albo w ktorymkolwiek silniku musi
//! przez nia przejsc, zanim uczen zostanie wpiety na stale.
//! DEPENDENCIES/LINKS: darkstar-shadow, darkstar-embed. Wymaga zmiennych srodowiskowych
//! obu silnikow; bez nich test POMIJA SIE glosno, nie przechodzi po cichu.
//! TECH STACK: Rust 2024, zwykly test integracyjny.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//!
//! Uruchomienie:
//! ```text
//! ORT_DYLIB_PATH=...  DARKSTAR_EMBED_MODEL_DIR=...
//! NOWORODEK_WEIGHTS=...nwrd  NOWORODEK_BOOK=...txt  NOWORODEK_CBMS=...cbms
//! cargo test -p darkstar-embed --test shadow_wiring -- --nocapture
//! ```

use std::sync::Arc;

use darkstar_embed::{MiniLmEmbedder, noworodek::NoworodekEmbedder};
use darkstar_shadow::{Embedder, Observation, ShadowJournal, ShadowRecord, ShadowedEmbedder};

const NEEDED: [&str; 4] = [
    "DARKSTAR_EMBED_MODEL_DIR",
    "NOWORODEK_WEIGHTS",
    "NOWORODEK_BOOK",
    "NOWORODEK_CBMS",
];

fn missing() -> bool {
    let absent: Vec<&str> = NEEDED
        .iter()
        .copied()
        .filter(|v| std::env::var(v).is_err())
        .collect();
    if absent.is_empty() {
        return false;
    }
    eprintln!("POMINIETO: brak {}", absent.join(", "));
    true
}

#[test]
fn the_student_watches_and_the_answer_stays_the_master_s() {
    if missing() {
        return;
    }

    let dir = std::env::temp_dir().join(format!("darkstar-wiring-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("katalog testowy");
    let journal_path = dir.join("shadow.jsonl");
    let journal = Arc::new(ShadowJournal::open(&journal_path).expect("dziennik"));

    // Wynik SAMEGO modelu zywego - punkt odniesienia sprzed wpiecia ucznia.
    let master = MiniLmEmbedder::from_env().expect("model zywy");
    let expected = master.embed("pamiec systemu").expect("liczenie przed wpieciem");
    let master_dims = master.dimensions();

    let student = NoworodekEmbedder::from_env().expect("uczen");
    let student_dims = student.dimensions();
    eprintln!(
        "zywy: {} ({master_dims} liczb) | uczen: {} ({student_dims} liczb, zna {} znakow)",
        master.name(),
        student.name(),
        student.known_symbols()
    );

    let socket = ShadowedEmbedder::new(Box::new(master))
        .with_shadow(Box::new(student))
        .with_journal(Arc::clone(&journal))
        .with_session("test-wpiecia");

    assert_eq!(socket.shadow_name(), Some("noworodek-cbms"), "uczen ma byc wpiety");
    assert_eq!(socket.dimensions(), master_dims, "gniazdo podaje rozmiar ZYWEGO");

    let got = socket.embed("pamiec systemu").expect("liczenie przez gniazdo");

    // To jest cala teza tego testu: obecnosc ucznia nie zmienia odpowiedzi ani o bit.
    assert_eq!(
        got, expected,
        "odpowiedz musi byc identyczna jak bez ucznia - gniazdo tylko przepuszcza"
    );

    // A tu druga polowa: uczen naprawde zostal odpytany, nie zignorowany.
    let lines = std::fs::read_to_string(&journal_path).expect("odczyt dziennika");
    let records: Vec<ShadowRecord> = lines
        .lines()
        .map(|l| serde_json::from_str(l).expect("linia dziennika"))
        .collect();
    assert_eq!(records.len(), 1, "jedno pytanie = jedna obserwacja");

    match &records[0].observation {
        Observation::Embedding {
            live_model,
            shadow_model,
            live_dims,
            shadow_dims,
            cosine,
            live_ms,
            shadow_ms,
            shadow_error,
            // Tresci pytania nie sprawdzamy tutaj: zapisuje ja gniazdo, a ten test dotyczy
            // wpiecia, nie formatu dziennika - ten ma wlasne testy w darkstar-shadow.
            ..
        } => {
            assert_eq!(live_model, "all-MiniLM-L6-v2");
            assert_eq!(shadow_model, "noworodek-cbms");
            assert_eq!(*live_dims, master_dims);
            eprintln!(
                "zanotowano: zywy {live_dims} liczb w {live_ms} ms | uczen {shadow_dims:?} w {shadow_ms:?} ms"
            );
            if let Some(err) = shadow_error {
                eprintln!("uczen zglosil blad (to tez jest obserwacja): {err}");
            } else {
                assert_eq!(
                    *shadow_dims,
                    Some(student_dims),
                    "rozmiar wektora ucznia ma trafic do dziennika"
                );
                // Rozne rozmiary - zgodnosci nie wolno udawac liczba.
                assert_eq!(
                    *cosine, None,
                    "przy roznych rozmiarach zgodnosc musi zostac niepoliczona, nie zerowa"
                );
            }
        }
        other => panic!("zly rodzaj obserwacji: {other:?}"),
    }

    eprintln!("ZDANE: uczen wpiety, patrzy, i nie ma wplywu na odpowiedz");
    std::fs::remove_dir_all(&dir).ok();
}
