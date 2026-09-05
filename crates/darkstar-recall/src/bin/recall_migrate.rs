// darkstar-header-v1
// po co: recall_migrate.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-04
//! Przeprowadzka pamieci ze starej Chromy do magazynu Darkstar (darkstar-recall).
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-04 23:20:00
//! REASON FOR CREATION: Cala pamiec - 4629 wspomnien i dziewiec i pol miesiaca historii -
//! zyje dzis wylacznie w Chromie na jednym Windowsie. Bez tego laptopa nie ma kontekstu.
//! To narzedzie przenosi ja na brame, ktora stoi.
//! MECHANICS: Rozmowa dwoch serwerow HTTP, zero Pythona. Czyta z Chromy przez jej wlasne API
//! (`/api/v1/collections/{id}/get` z embeddings, documents i metadatas), zapisuje do Qdranta
//! paczkami. ZRODLO JEST NIETYKALNE: uzywane sa wylacznie metody odczytu, w kodzie nie ma ani
//! jednej sciezki zapisujacej do Chromy. Domyslny tryb to proba na sucho - zeby zobaczyc, co
//! sie stanie, zanim cokolwiek sie stanie.
//! SYSTEM PART: Darkstar Recall / migracja pamieci.
//! ARCHITECTURE FUNCTION: Jednorazowa (powtarzalna) przeprowadzka. Po niej zrodlem prawdy dla
//! pamieci jest Darkstar, a Chroma na Windowsie zostaje jako kopia historyczna.
//! DEPENDENCIES/LINKS: Chroma v1 na 127.0.0.1:8000 (zmierzone: ta instancja mowi v1, nie v2),
//! magazyn Darkstar na 127.0.0.1:6333, deploy/.env (DARKSTAR_RECALL_API_KEY).
//! TECH STACK: Rust 2024 + serde_json, klient HTTP z darkstar-recall. Bez Pythona: zasada
//! projektu, a przy okazji jedyny sposob, zeby to samo narzedzie dzialalo pozniej na Redoxie.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use darkstar_recall::{HttpClient, read_env_value};
use serde_json::{Value, json};

const BATCH: usize = 200;

struct Args {
    chroma: String,
    recall: String,
    env_file: String,
    only: Option<String>,
    apply: bool,
}

fn parse_args() -> Args {
    let mut a = Args {
        chroma: "http://127.0.0.1:8000".into(),
        recall: "http://127.0.0.1:6333".into(),
        env_file: "/home/owner/polip-agi/deploy/.env".into(),
        only: None,
        // Domyslnie NIE zapisuje. Zeby cokolwiek zmienic, trzeba to powiedziec wprost.
        apply: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--chroma" => a.chroma = it.next().unwrap_or(a.chroma),
            "--recall" => a.recall = it.next().unwrap_or(a.recall),
            "--env-file" => a.env_file = it.next().unwrap_or(a.env_file),
            "--only" => a.only = it.next(),
            "--apply" => a.apply = true,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                eprintln!("nieznany argument: {other}");
                print_usage();
                std::process::exit(2);
            }
        }
    }
    a
}

fn print_usage() {
    println!(
        "recall-migrate - przeprowadzka pamieci z Chromy do magazynu Darkstar\n\
         \n\
           --chroma <url>     zrodlo, domyslnie http://127.0.0.1:8000\n\
           --recall <url>     cel,    domyslnie http://127.0.0.1:6333\n\
           --env-file <plik>  skad wziac DARKSTAR_RECALL_API_KEY\n\
           --only <nazwa>     przenies wylacznie te jedna kolekcje\n\
           --apply            NAPRAWDE zapisz (bez tego: proba na sucho)\n\
         \n\
         Zrodlo jest tylko czytane. Bez --apply nic nie zostaje zapisane."
    );
}

fn main() {
    let args = parse_args();
    let key = read_env_value(&args.env_file, "DARKSTAR_RECALL_API_KEY");
    // Klucz jest wymagany dopiero przy zapisie. Proba na sucho ma sie dac uruchomic takze
    // z maszyny, ktora nie ma dostepu do pliku z kluczem - inaczej "zobacz co sie stanie"
    // bylo by trudniejsze niz "zrob to", a to zla kolejnosc.
    if args.apply && key.is_none() {
        eprintln!(
            "FAIL: brak DARKSTAR_RECALL_API_KEY w {} - bez klucza magazyn odmowi zapisu",
            args.env_file
        );
        std::process::exit(1);
    }

    let src = HttpClient::new(&args.chroma, None);
    let dst = HttpClient::new(&args.recall, key);

    if !args.apply {
        println!("TRYB: proba na sucho. Zrodlo czytane, cel NIE dotykany. Dodaj --apply, zeby zapisac.");
    } else {
        println!("TRYB: ZAPIS. Zrodlo nadal wylacznie czytane.");
    }

    let collections = match list_collections(&src) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("FAIL: nie moge odczytac listy kolekcji ze zrodla: {e}");
            std::process::exit(1);
        }
    };
    println!("INFO: kolekcji w zrodle: {}", collections.len());

    let mut total_read = 0usize;
    let mut total_written = 0usize;
    let mut failures = 0usize;

    for (id, name) in &collections {
        if let Some(only) = &args.only
            && only != name
        {
            continue;
        }
        match migrate_one(&src, &dst, id, name, args.apply) {
            Ok((read, written)) => {
                total_read += read;
                total_written += written;
                println!("PASS: {name}: odczytano {read}, zapisano {written}");
            }
            Err(e) => {
                failures += 1;
                eprintln!("FAIL: {name}: {e}");
            }
        }
    }

    println!("\nPODSUMOWANIE: odczytano {total_read}, zapisano {total_written}, bledow {failures}");
    if failures > 0 {
        std::process::exit(1);
    }
}

fn list_collections(src: &HttpClient) -> Result<Vec<(String, String)>, String> {
    let body = src.get("/api/v1/collections").map_err(|e| e.to_string())?;
    let parsed: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let arr = parsed.as_array().ok_or("odpowiedz nie jest lista")?;
    Ok(arr
        .iter()
        .filter_map(|c| {
            Some((
                c.get("id")?.as_str()?.to_string(),
                c.get("name")?.as_str()?.to_string(),
            ))
        })
        .collect())
}

fn migrate_one(
    src: &HttpClient,
    dst: &HttpClient,
    id: &str,
    name: &str,
    apply: bool,
) -> Result<(usize, usize), String> {
    let mut offset = 0usize;
    let mut read_total = 0usize;
    let mut written_total = 0usize;
    let mut created = false;

    loop {
        let req = json!({
            "limit": BATCH,
            "offset": offset,
            "include": ["embeddings", "documents", "metadatas"]
        })
        .to_string();
        let body = src
            .post(&format!("/api/v1/collections/{id}/get"), &req)
            .map_err(|e| e.to_string())?;
        let page: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;

        let ids = page.get("ids").and_then(Value::as_array).cloned().unwrap_or_default();
        if ids.is_empty() {
            break;
        }
        let embeddings = page.get("embeddings").and_then(Value::as_array).cloned().unwrap_or_default();
        let documents = page.get("documents").and_then(Value::as_array).cloned().unwrap_or_default();
        let metadatas = page.get("metadatas").and_then(Value::as_array).cloned().unwrap_or_default();

        // Wymiar bierzemy z pierwszego wektora, nie z zalozenia - kolekcje moga sie roznic.
        let dim = embeddings
            .first()
            .and_then(Value::as_array)
            .map(|v| v.len())
            .ok_or("brak wektorow w odpowiedzi zrodla")?;

        if !created {
            if apply {
                let create = json!({ "vectors": { "size": dim, "distance": "Cosine" } }).to_string();
                // Kolekcja moze juz istniec z poprzedniego przebiegu - to nie jest blad.
                match dst.put(&format!("/collections/{name}"), &create) {
                    Ok(_) => {}
                    Err(e) => {
                        let msg = e.to_string();
                        if !msg.contains("already exists") {
                            return Err(format!("nie moge zalozyc kolekcji: {msg}"));
                        }
                    }
                }
            }
            created = true;
        }

        let mut points = Vec::with_capacity(ids.len());
        for (i, chroma_id) in ids.iter().enumerate() {
            let vector = embeddings.get(i).cloned().unwrap_or(Value::Null);
            if !vector.is_array() {
                continue;
            }
            // Qdrant przyjmuje jako identyfikator liczbe albo UUID; identyfikatory Chromy
            // bywaja dowolnym tekstem. Numerujemy po kolei, a oryginal zachowujemy w danych -
            // dzieki temu nic nie ginie i da sie wrocic do zrodla po tym samym kluczu.
            let mut payload = serde_json::Map::new();
            payload.insert("chroma_id".into(), chroma_id.clone());
            payload.insert("collection".into(), json!(name));
            if let Some(doc) = documents.get(i) {
                payload.insert("document".into(), doc.clone());
            }
            if let Some(Value::Object(meta)) = metadatas.get(i) {
                for (k, v) in meta {
                    payload.insert(k.clone(), v.clone());
                }
            }
            points.push(json!({
                "id": offset + i,
                "vector": vector,
                "payload": Value::Object(payload)
            }));
        }

        read_total += ids.len();

        if apply && !points.is_empty() {
            let upsert = json!({ "points": points }).to_string();
            dst.put(&format!("/collections/{name}/points?wait=true"), &upsert)
                .map_err(|e| format!("zapis paczki nie powiodl sie: {e}"))?;
            written_total += points.len();
        }

        if ids.len() < BATCH {
            break;
        }
        offset += BATCH;
    }

    Ok((read_total, written_total))
}
