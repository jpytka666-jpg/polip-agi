// darkstar-header-v1
// po co: recall_ask.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Zamienia pytanie zadane po ludzku na zapytanie do magazynu pamieci.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 01:50:00
//! REASON FOR CREATION: Magazyn pamieci przyjmuje wektor, nie zdanie. Do dzis oznaczalo to,
//! ze 4631 wspomnien bylo osiagalnych tylko dla kogos, kto ma juz gotowy wektor - czyli
//! praktycznie dla nikogo. To narzedzie domyka ostatni odcinek: czlowiek pisze zdanie.
//! MECHANICS: Liczy osadzenie pytania modelem all-MiniLM-L6-v2 i wypisuje na standardowe
//! wyjscie gotowe cialo zapytania w formacie magazynu. NIE laczy sie z siecia - swiadomie.
//! Dzieki temu wynik mozna przepuscic przez ssh na maszyne, na ktorej mieszka klucz dostepu,
//! i klucz ani razu nie musi opuscic tamtej maszyny ani przejsc przez historie polecen.
//! SYSTEM PART: Darkstar Embed / narzedzia.
//! ARCHITECTURE FUNCTION: Most miedzy jezykiem czlowieka a magazynem wektorow. Celowo robi
//! jedna rzecz i konczy - laczenie, uwierzytelnianie i odczyt wynikow naleza gdzie indziej.
//! DEPENDENCIES/LINKS: darkstar-embed (silnik), ORT_DYLIB_PATH i DARKSTAR_EMBED_MODEL_DIR.
//! TECH STACK: Rust 2024. Bez klienta HTTP: narzedzie, ktore sie nie laczy, nie potrzebuje
//! klucza - a narzedzie, ktore nie potrzebuje klucza, nie moze go wyciec.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//!
//! Uzycie:
//! ```text
//! recall-ask "jak dziala magazyn pamieci" [ile-wynikow]
//! ```
//! Typowo w potoku, zeby klucz zostal na swojej maszynie:
//! ```text
//! recall-ask "pytanie" | ssh owner@host 'curl -s -X POST \
//!   http://127.0.0.1:6333/collections/<zbior>/points/search \
//!   -H "api-key: $(grep ^DARKSTAR_RECALL_API_KEY= deploy/.env | cut -d= -f2-)" \
//!   -H "content-type: application/json" --data-binary @-'
//! ```

use darkstar_embed::MiniLmEmbedder;
use darkstar_shadow::Embedder;

const DEFAULT_LIMIT: usize = 5;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(question) = args.next() else {
        eprintln!("uzycie: recall-ask \"<pytanie>\" [ile-wynikow]");
        eprintln!();
        eprintln!("Wymaga DARKSTAR_EMBED_MODEL_DIR i ORT_DYLIB_PATH.");
        eprintln!("Wypisuje na standardowe wyjscie cialo zapytania do magazynu pamieci.");
        std::process::exit(2);
    };
    if question.trim().is_empty() {
        eprintln!("FAIL: puste pytanie");
        std::process::exit(2);
    }

    let limit = args
        .next()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_LIMIT);

    let embedder = match MiniLmEmbedder::from_env() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("FAIL: nie moge wczytac modelu: {e}");
            std::process::exit(1);
        }
    };

    let vector = match embedder.embed(&question) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("FAIL: nie moge policzyc osadzenia: {e}");
            std::process::exit(1);
        }
    };

    // Diagnostyka idzie na wyjscie bledow, nie na standardowe - inaczej zepsulaby JSON
    // w potoku. Samo pytanie tez tu nie trafia: moze byc prywatne, a to wyjscie bywa logowane.
    eprintln!(
        "policzono {} wymiarow modelem {} dla pytania o {} znakach",
        vector.len(),
        embedder.name(),
        question.chars().count()
    );

    let body = serde_json::json!({
        "vector": vector,
        "limit": limit,
        "with_payload": true,
    });

    match serde_json::to_string(&body) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("FAIL: nie moge zlozyc zapytania: {e}");
            std::process::exit(1);
        }
    }
}
