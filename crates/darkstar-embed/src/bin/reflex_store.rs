// darkstar-header-v1
// po co: reflex_store.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Slownik odruchow: gotowe bloki kodu, ktore rdzen potrafi znalezc po opisie zadania.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 19:00:00
//! REASON FOR CREATION: Noworodek jest RDZENIEM KREGOWYM, nie rozmowca - Marcin postawil to
//! wprost. Mozg (Qwen 30B albo agent) mysli wolno i drogo; rdzen ma odpowiadac odruchowo,
//! a cialem sa narzedzia AIONS i Darkstar. Odruch nie polega na wymyslaniu kodu, tylko na
//! siegnieciu po WLASCIWY GOTOWY BLOK i podmianie w nim tego, co trzeba. Ten program buduje
//! slownik takich odruchow.
//! MECHANICS: Czyta bloki (opis + kod) z pliku JSON, liczy osadzenie OPISU silnikiem
//! Noworodka - czyli w przestrzeni CBMS, w ktorej rdzen mysli - i zapisuje je do magazynu
//! wektorow jako osobna kolekcje. Wyszukiwanie dziala potem tak samo jak dla wspomnien:
//! opis zadania staje sie wektorem, magazyn zwraca najblizsze bloki.
//! SYSTEM PART: Darkstar / slownik odruchow rdzenia.
//! ARCHITECTURE FUNCTION: Zamyka droge mozg -> rdzen -> cialo. Mozg mowi czego potrzebuje,
//! rdzen znajduje odruch, cialo go wykonuje. Kolekcja jest osobna od wspomnien, bo ma inny
//! rozmiar wektora (Noworodek liczy w swojej przestrzeni, nie w przestrzeni MiniLM) i inne
//! przeznaczenie - wspomnienia sie pamieta, odruchy sie wykonuje.
//! DEPENDENCIES/LINKS: darkstar-embed::noworodek (silnik rdzenia), darkstar-recall (magazyn),
//! plik JSON z blokami wyprodukowany przez wyciag z kodu zrodlowego.
//! TECH STACK: Rust 2024.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//!
//! Uzycie:
//! ```text
//! reflex-store <bloki.json> [--collection reflex_blocks] [--recall http://127.0.0.1:6333]
//!              [--env-file /home/owner/polip-agi/deploy/.env] [--dry-run]
//! ```
//! Bez `--dry-run` zapisuje do magazynu. Z nim tylko liczy i pokazuje, co by zapisal.

use darkstar_embed::noworodek::NoworodekEmbedder;
use darkstar_recall::{HttpClient, read_env_value};
use darkstar_shadow::Embedder;
use serde_json::{Value, json};

const DEFAULT_COLLECTION: &str = "reflex_blocks";
const DEFAULT_RECALL: &str = "http://127.0.0.1:6333";
const DEFAULT_ENV: &str = "/home/owner/polip-agi/deploy/.env";
/// Ile blokow idzie w jednej paczce. Wieksze paczki to mniej rozmow z magazynem, ale
/// dluzsza pojedyncza wiadomosc - przy tysiacach blokow i wektorach po 128 liczb
/// dwiescie jest rozsadnym srodkiem.
const BATCH: usize = 200;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!("uzycie: reflex-store <bloki.json> [--collection <nazwa>] [--recall <url>]");
        eprintln!("        [--env-file <plik>] [--dry-run]");
        eprintln!();
        eprintln!("Wymaga NOWORODEK_WEIGHTS, NOWORODEK_BOOK, NOWORODEK_CBMS.");
        std::process::exit(2);
    };
    let opt = |name: &str, default: &str| -> String {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| default.to_string())
    };
    let collection = opt("--collection", DEFAULT_COLLECTION);
    let recall_url = opt("--recall", DEFAULT_RECALL);
    let env_file = opt("--env-file", DEFAULT_ENV);
    let dry_run = args.iter().any(|a| a == "--dry-run");

    // Tryb szukania: pierwszy argument jest wtedy OPISEM ZADANIA, nie sciezka do pliku.
    // To jest wlasciwa miara tego narzedzia - nie ile blokow zapisano, tylko czy na opis
    // zadania wraca ten blok, ktory czlowiek by wybral.
    if args.iter().any(|a| a == "--find") {
        let core = match NoworodekEmbedder::from_env() {
            Ok(e) => e,
            Err(e) => {
                eprintln!("FAIL: nie moge wczytac rdzenia: {e}");
                std::process::exit(1);
            }
        };
        let vector = match core.embed(path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("FAIL: nie moge opisac zadania: {e}");
                std::process::exit(1);
            }
        };
        let Some(key) = read_env_value(&env_file, "DARKSTAR_RECALL_API_KEY") else {
            eprintln!("FAIL: brak DARKSTAR_RECALL_API_KEY w {env_file}");
            std::process::exit(1);
        };
        let store = HttpClient::new(&recall_url, Some(key));
        let limit: usize = args
            .iter()
            .position(|a| a == "--limit")
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        let query = json!({ "vector": vector, "limit": limit, "with_payload": true }).to_string();

        match store.post(&format!("/collections/{collection}/points/search"), &query) {
            Ok(body) => {
                let parsed: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
                let empty = vec![];
                let hits = parsed["result"].as_array().unwrap_or(&empty);
                println!("zadanie: {path}\nodruchow znalezionych: {}\n", hits.len());
                for (i, hit) in hits.iter().enumerate() {
                    let score = hit["score"].as_f64().unwrap_or(0.0);
                    let p = &hit["payload"];
                    println!(
                        "{}. [{score:.4}] {} ({})",
                        i + 1,
                        p["name"].as_str().unwrap_or("?"),
                        p["file"].as_str().unwrap_or("?")
                    );
                    let doc = p["doc"].as_str().unwrap_or("");
                    if !doc.is_empty() {
                        println!("   {}", doc.chars().take(100).collect::<String>());
                    }
                }
            }
            Err(e) => {
                eprintln!("FAIL: magazyn nie odpowiedzial: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    let raw = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: nie moge odczytac {path}: {e}");
            std::process::exit(1);
        }
    };
    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("FAIL: {path} nie jest poprawnym JSON-em: {e}");
            std::process::exit(1);
        }
    };
    // Jeden blok tez jest lista - inaczej plik z jednym wpisem wymagalby innego ksztaltu.
    let blocks: Vec<Value> = match parsed {
        Value::Array(a) => a,
        other => vec![other],
    };
    println!("blokow w pliku: {}", blocks.len());

    let core = match NoworodekEmbedder::from_env() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("FAIL: nie moge wczytac rdzenia: {e}");
            std::process::exit(1);
        }
    };
    println!(
        "rdzen: {} ({} liczb, zna {} znakow)",
        core.name(),
        core.dimensions(),
        core.known_symbols()
    );

    // Klucz wymagany dopiero przy zapisie: proba na sucho ma dzialac takze tam, gdzie
    // klucza nie ma - inaczej "zobacz co sie stanie" byloby trudniejsze niz "zrob to".
    let key = read_env_value(&env_file, "DARKSTAR_RECALL_API_KEY");
    if !dry_run && key.is_none() {
        eprintln!("FAIL: brak DARKSTAR_RECALL_API_KEY w {env_file}");
        std::process::exit(1);
    }
    let store = HttpClient::new(&recall_url, key);

    let mut points = Vec::new();
    let mut skipped = 0usize;

    for (index, block) in blocks.iter().enumerate() {
        let name = block.get("Name").and_then(Value::as_str).unwrap_or("");
        let doc = block.get("Doc").and_then(Value::as_str).unwrap_or("");
        let body = block.get("Body").and_then(Value::as_str).unwrap_or("");
        let file = block.get("File").and_then(Value::as_str).unwrap_or("");

        // Osadzamy OPIS, nie kod. Szukanie zaczyna sie od "czego potrzebuje", a to jest
        // zdanie o zadaniu, nie fragment skladni. Nazwa idzie razem z opisem, bo dobra
        // nazwa funkcji sama w sobie mowi, co ta funkcja robi.
        let subject = format!("{name} {doc}");
        if subject.trim().len() < 8 || body.is_empty() {
            skipped += 1;
            continue;
        }

        match core.embed(&subject) {
            Ok(vector) => {
                points.push(json!({
                    "id": index,
                    "vector": vector,
                    "payload": {
                        "name": name,
                        "doc": doc,
                        "file": file,
                        "body": body,
                        "lines": block.get("Lines").and_then(Value::as_u64).unwrap_or(0),
                    }
                }));
            }
            Err(e) => {
                // Blok, ktorego rdzen nie umie opisac, jest pomijany, a nie zapisywany
                // z pustym wektorem: pusty wektor pasowalby do wszystkiego jednakowo
                // i zasmiecalby kazde wyszukiwanie.
                eprintln!("pomijam {name}: {e}");
                skipped += 1;
            }
        }
    }

    println!("osadzonych: {} | pominietych: {skipped}", points.len());
    if points.is_empty() {
        eprintln!("FAIL: zaden blok nie dal sie osadzic");
        std::process::exit(1);
    }

    if dry_run {
        println!("\nTRYB: proba na sucho. Magazyn NIE dotykany.");
        for p in points.iter().take(3) {
            let name = p["payload"]["name"].as_str().unwrap_or("");
            let doc = p["payload"]["doc"].as_str().unwrap_or("");
            println!("  {name}: {}", doc.chars().take(70).collect::<String>());
        }
        return;
    }

    let dims = core.dimensions();
    let create = json!({ "vectors": { "size": dims, "distance": "Cosine" } }).to_string();
    match store.put(&format!("/collections/{collection}"), &create) {
        Ok(_) => println!("\nkolekcja {collection} zalozona ({dims} liczb na wektor)"),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") {
                println!("\nkolekcja {collection} juz istnieje - dopisuje");
            } else {
                eprintln!("FAIL: nie moge zalozyc kolekcji: {msg}");
                std::process::exit(1);
            }
        }
    }

    let mut written = 0usize;
    for chunk in points.chunks(BATCH) {
        let body = json!({ "points": chunk }).to_string();
        match store.put(&format!("/collections/{collection}/points?wait=true"), &body) {
            Ok(_) => written += chunk.len(),
            Err(e) => {
                eprintln!("FAIL: zapis paczki: {e}");
                std::process::exit(1);
            }
        }
    }
    println!("zapisano odruchow: {written}");
}
