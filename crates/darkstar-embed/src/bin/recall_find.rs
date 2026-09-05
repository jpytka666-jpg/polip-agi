// darkstar-header-v1
// po co: recall_find.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Jednorazowe polecenie: pytanie -> embedding -> search -> wyniki.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Haiku 4.5
//! TIMESTAMP: 2026-09-05 14:30:00
//! REASON FOR CREATION: recall-ask zwraca tylko wektor, recall_migrate pracuje ze stalem
//! zrodlem i celem. recall-find to trzecie ogniwo: przyjmuje pytanie po ludzku, liczy wektor,
//! wysyla do magazynu, odbiera wyniki, wypisuje je czytelnie. Mozna to uzyc bez skryptu
//! powlokowego - w trybie interaktywnym.
//! MECHANICS: Wczytuje model z DARKSTAR_EMBED_MODEL_DIR (jak recall-ask), oblicza osadzenie
//! pytania, wysyla POST na /collections/<nazwa>/points/search z wektorem. Odpowiedz Qdranta
//! zawiera punkty z payload; dla kazdego punktu wypisujemy numer, score i pierwsze 200
//! znakow pola payload.document. Klucz API czytany z pliku, nigdy z argumentow.
//! SYSTEM PART: Darkstar Embed / narzedzia / szukanie.
//! ARCHITECTURE FUNCTION: Most miedzy jezykiem czlowieka a magazynem - calosciowy: od pytania
//! do przejrzystych wynikow. Laczenie sie z siecia, uwierzytelnianie i wyswietlanie wszystko
//! tutaj, przeciwnie niz recall-ask.
//! DEPENDENCIES/LINKS: darkstar-embed (MiniLmEmbedder), darkstar-recall (HttpClient,
//! read_env_value), ORT_DYLIB_PATH, DARKSTAR_EMBED_MODEL_DIR, magazyn na /collections/search.
//! TECH STACK: Rust 2024. Zalezy od darkstar-recall dla HTTP bez dodatkowych bibliotek,
//! od darkstar-embed dla embedding.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: 9a4cff7
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan, commit 9a4cff7
//! ==========================================
//!
//! Uzycie:
//! ```text
//! recall-find "<pytanie>" [--collection <nazwa>] [--limit N] [--recall <url>] [--env-file <plik>]
//! ```
//!
//! Przyklad:
//! ```text
//! recall-find "jak dziala pamiec systemu"
//! recall-find "Rust programowanie" --limit 10 --collection inne_wspomnienia
//! ```
//!
//! Domyslne wartosci:
//! - kolekcja: session_claude_marcin_main
//! - limit: 5
//! - recall: http://127.0.0.1:6333
//! - env-file: /home/owner/polip-agi/deploy/.env
//!
//! Klucz API czytany z pliku podanego jako --env-file, szukany w linii DARKSTAR_RECALL_API_KEY=.
//! Klucz nigdy nie trafia do argumentow ani do komunikatow diagnostycznych.

use darkstar_embed::MiniLmEmbedder;
use darkstar_recall::{HttpClient, read_env_value};
use darkstar_shadow::Embedder;
use serde_json::{json, Value};

struct Args {
    question: String,
    collection: String,
    limit: usize,
    recall_url: String,
    env_file: String,
}

fn parse_args() -> Option<Args> {
    let mut args = std::env::args().skip(1);
    let question = args.next()?;

    if question.trim().is_empty() {
        return None;
    }

    let mut collection = "session_claude_marcin_main".to_string();
    let mut limit = 5usize;
    let mut recall_url = "http://127.0.0.1:6333".to_string();
    let mut env_file = "/home/owner/polip-agi/deploy/.env".to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--collection" => {
                if let Some(v) = args.next() {
                    collection = v;
                }
            }
            "--limit" => {
                if let Some(v) = args.next() {
                    if let Ok(n) = v.parse::<usize>() {
                        if n > 0 {
                            limit = n;
                        }
                    }
                }
            }
            "--recall" => {
                if let Some(v) = args.next() {
                    recall_url = v;
                }
            }
            "--env-file" => {
                if let Some(v) = args.next() {
                    env_file = v;
                }
            }
            _ => {
                // Nieznane opcje ignorujemy - mogą być dla przyszłych wersji
            }
        }
    }

    Some(Args {
        question,
        collection,
        limit,
        recall_url,
        env_file,
    })
}

fn print_usage() {
    eprintln!("uzycie: recall-find \"<pytanie>\" [--collection <nazwa>] [--limit N] [--recall <url>] [--env-file <plik>]");
    eprintln!();
    eprintln!("Wymaga DARKSTAR_EMBED_MODEL_DIR i ORT_DYLIB_PATH.");
    eprintln!("Klucz API czytany z pliku podanego jako --env-file.");
    eprintln!();
    eprintln!("Domyslne wartosci:");
    eprintln!("  kolekcja:  session_claude_marcin_main");
    eprintln!("  limit:     5");
    eprintln!("  recall:    http://127.0.0.1:6333");
    eprintln!("  env-file:  /home/owner/polip-agi/deploy/.env");
}

fn main() {
    let args = match parse_args() {
        Some(a) => a,
        None => {
            print_usage();
            std::process::exit(2);
        }
    };

    // Load embedder
    let embedder = match MiniLmEmbedder::from_env() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("FAIL: nie moge wczytac modelu: {e}");
            eprintln!("Ustaw DARKSTAR_EMBED_MODEL_DIR i ORT_DYLIB_PATH.");
            std::process::exit(1);
        }
    };

    // Compute embedding
    let vector = match embedder.embed(&args.question) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("FAIL: nie moge policzyc osadzenia: {e}");
            std::process::exit(1);
        }
    };

    eprintln!(
        "policzono {} wymiarow dla pytania o {} znakach",
        vector.len(),
        args.question.chars().count()
    );

    // Read API key from file
    let api_key = read_env_value(&args.env_file, "DARKSTAR_RECALL_API_KEY");
    if api_key.is_none() {
        eprintln!("FAIL: brak DARKSTAR_RECALL_API_KEY w {}", args.env_file);
        eprintln!("Plausible paths:");
        eprintln!("  /home/owner/polip-agi/deploy/.env");
        eprintln!("  ~/.config/polip-agi/.env");
        std::process::exit(1);
    }

    // Create HTTP client
    let client = HttpClient::new(&args.recall_url, api_key);

    // Search
    let search_body = json!({
        "vector": vector,
        "limit": args.limit,
        "with_payload": true,
    })
    .to_string();

    let response = match client.post(
        &format!("/collections/{}/points/search", args.collection),
        &search_body,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FAIL: magazyn nieosiagalny lub beda w kolekcji: {e}");
            eprintln!("Sprawdz:");
            eprintln!("  1. czy adres {} dostepny", args.recall_url);
            eprintln!("  2. czy kolekcja {} istnieje", args.collection);
            eprintln!("  3. czy klucz API jest poprawny");
            std::process::exit(1);
        }
    };

    // Parse response
    let parsed: Value = match serde_json::from_str(&response) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("FAIL: magazyn wysLal zla odpowiedz: {e}");
            eprintln!("Raw response:\n{}", response);
            std::process::exit(1);
        }
    };

    // Extract results
    let empty = vec![];
    let results = parsed
        .get("result")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    if results.is_empty() {
        println!("brak wynikow");
        std::process::exit(0);
    }

    println!(
        "znaleziono {} wynikow dla: \"{}\" (kolekcja: {})\n",
        results.len(),
        args.question.chars().take(60).collect::<String>(),
        args.collection
    );

    for (idx, point) in results.iter().enumerate() {
        let score = point
            .get("score")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);

        let payload = point.get("payload").and_then(Value::as_object);
        let document = payload
            .and_then(|p| p.get("document"))
            .map(|d| d.as_str().unwrap_or(""))
            .unwrap_or("");

        let preview = if document.len() > 200 {
            format!("{}...", &document[..200])
        } else {
            document.to_string()
        };

        println!("{}. [score: {:.4}]", idx + 1, score);
        println!("   {}", preview);
        println!();
    }
}
