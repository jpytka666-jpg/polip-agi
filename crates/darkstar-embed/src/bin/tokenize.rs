// darkstar-header-v1
// po co: tokenize.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Tokenizer-tlumacz: dowolny jezyk -> znaki CBMS. Wyszukiwanie zamiast tabelki.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 22:10:00
//! REASON FOR CREATION: Marcin: "RAG MA BYC TLUMACZEM ... TO ROBI NOWORODKOWI ZA TOKENIZER".
//! To nie jest pomocnik przed tokenizerem - to JEST tokenizer. Wejscie w dowolnym jezyku
//! zamienia sie na znaki CBMS, i dopiero to widzi rdzen. W druga strone ta sama warstwa
//! obrabia odpowiedz na angielski, i tylko na angielski.
//! MECHANICS: Ksiega daje slowom znaki. Slowo, ktorego w ksiedze nie ma, nie jest zgadywane
//! ani stemplowane - jest WYSZUKIWANE: jego wektor porownuje sie z wektorami slow, ktore
//! znaki maja, i bierze najblizsze, o ile jest dosc blisko. Ponizej progu slowo zostaje
//! nieznane i jest oznaczone, bo brak odpowiedzi da sie uzupelnic, a cicha pomylka wchodzi
//! do materialu na stale.
//! SYSTEM PART: Darkstar / warstwa jezyka - tokenizer.
//! ARCHITECTURE FUNCTION: Znaczek jest NUMEREM, nie napisem - rdzeniowi jest obojetne, czy
//! pojecie nazywa sie `memoro` czy `memory`. Wiec nie trzeba najpierw zbudowac esperanta,
//! zeby miec jeden znak na pojecie; wystarczy, zeby wszystkie jezyki trafialy w ten sam
//! numer. Esperancka nazwe dokleja sie do tego samego numeru pozniej, gdy most ja pozna.
//! To jest ta sama mysl, ktora Marcin podal dla drogi wyjsciowej - wspolny, plastyczny
//! srodek - uzyta takze na wejsciu.
//! DEPENDENCIES/LINKS: darkstar-embed (MiniLmEmbedder), darkstar-recall (magazyn wektorow),
//! ksiega CBMS (slowo=znak). Wyjscie to ciag znakow CBMS, czyli dokladnie to, co przyjmuje
//! `cbms <ksiega> ids` - warstwa wpina sie w istniejacy lancuch bez przerabiania go.
//! TECH STACK: Rust 2024.
//! LOCAL WORKSPACE: budowane i uruchamiane na Darkstarze (owner@100.64.0.2)
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//! REVISION 2026-09-06: kanoniczny EO dla Noworodka idzie przez darkstar_embed::frontend_to_cbms
//! (cbms-writing Vocabulary), NIE przez glify ani most glyph->u32. Ten bin zostaje sciezka RAG/glif.
//! REVISION 2026-09-06 (Codex; dokladny model niedostepny): osobny tryb canonical
//! przyjmuje gotowy EO i wypisuje Vec<u32> przez frontend_to_cbms_path -> frontend_to_cbms.
//! Bez MiniLM/sieci; istniejace run/index zachowuja swoj kontrakt.
//!
//!
//! Uzycie:
//! ```text
//! tokenize index --book ksiega.txt [--collection cbms_concepts] [--limit N]
//! tokenize run   --book ksiega.txt --text "dowolne zdanie" [--threshold 0.55]
//! tokenize canonical --book ksiega.txt --text "kanoniczny tekst EO"
//! ```

use std::collections::HashMap;
use std::fs;

use darkstar_embed::{MiniLmEmbedder, frontend_to_cbms_path};
use darkstar_recall::{HttpClient, read_env_value};
use darkstar_shadow::Embedder;
use serde_json::{Value, json};

const DEFAULT_RECALL: &str = "http://127.0.0.1:6333";
const DEFAULT_ENV: &str = "/home/owner/polip-agi/deploy/.env";
const DEFAULT_COLLECTION: &str = "cbms_concepts";

/// Jak blisko musi byc najblizsze pojecie, zeby uznac je za trafienie.
///
/// TYMCZASOWY i celowo ostrozny. Zmierzone dzis na slowniku odruchow: trafienie trafne dalo
/// 0.4702 przy drugim 0.2265, a zgadywanie 0.5997 przy drugim 0.5916 - czyli sama wysokosc
/// oceny nie rozstrzyga. Dlatego oprocz progu sprawdzamy takze ODSTEP do drugiego wyniku,
/// tak samo jak robi to rdzen przy decyzji. Obie liczby nalezy przeliczyc, gdy bedzie na czym.
const NEAR_ENOUGH: f64 = 0.55;

/// Najmniejszy odstep do drugiego pojecia. Bez niego slowo, ktore pasuje jednakowo do pieciu
/// roznych rzeczy, dostaloby znak pierwszej z brzegu - i nikt by sie nie dowiedzial.
const MIN_MARGIN: f64 = 0.05;

/// Ile punktow wysylamy do magazynu naraz. Cala ksiega w jednym zadaniu to kilkadziesiat
/// megabajtow JSON-a i magazyn to odrzuca.
const BATCH: usize = 500;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("");
    let opt = |name: &str, default: &str| -> String {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| default.to_string())
    };

    let book_path = opt("--book", "");
    if book_path.is_empty() {
        eprintln!("uzycie:");
        eprintln!("  tokenize index --book ksiega.txt [--collection {DEFAULT_COLLECTION}]");
        eprintln!("  tokenize run   --book ksiega.txt --text \"zdanie\" [--threshold 0.55]");
        eprintln!("  tokenize canonical --book ksiega.txt --text \"kanoniczny tekst EO\"");
        std::process::exit(2);
    }
    if mode == "canonical" {
        std::process::exit(do_canonical(
            &book_path,
            &opt("--text", ""),
            &mut std::io::stdout().lock(),
        ));
    }
    let book = match load_book(&book_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("FAIL: ksiega {book_path}: {e}");
            std::process::exit(1);
        }
    };
    println!("ksiega: {} pojec", book.len());

    let collection = opt("--collection", DEFAULT_COLLECTION);
    let env_file = opt("--env-file", DEFAULT_ENV);
    let Some(key) = read_env_value(&env_file, "DARKSTAR_RECALL_API_KEY") else {
        eprintln!("FAIL: brak DARKSTAR_RECALL_API_KEY w {env_file}");
        std::process::exit(1);
    };
    let store = HttpClient::new(opt("--recall", DEFAULT_RECALL), Some(key));
    let engine = match MiniLmEmbedder::from_env() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("FAIL: silnik osadzen: {e}");
            std::process::exit(1);
        }
    };

    let code = match mode {
        "index" => {
            let limit: usize = opt("--limit", "0").parse().unwrap_or(0);
            do_index(&store, &engine, &collection, &book, limit)
        }
        "run" => {
            let text = opt("--text", "");
            let threshold: f64 = opt("--threshold", "0.55").parse().unwrap_or(NEAR_ENOUGH);
            do_run(&store, &engine, &collection, &book, &text, threshold)
        }
        _ => {
            eprintln!("FAIL: tryb to 'index', 'run' albo 'canonical'");
            2
        }
    };
    std::process::exit(code);
}

/// Gotowy EO bez normalizacji lub wyboru znaczenia; stdout zawiera tylko liste u32.
fn do_canonical(book_path: &str, text: &str, output: &mut impl std::io::Write) -> i32 {
    if text.trim().is_empty() {
        eprintln!("FAIL: --text jest puste");
        return 2;
    }
    match frontend_to_cbms_path(book_path, text) {
        Ok(ids) => match writeln!(output, "{ids:?}") {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("FAIL: zapis CBMS IDs: {e}");
                1
            }
        },
        Err(e) => {
            eprintln!("FAIL: canonical: {e}");
            1
        }
    }
}

/// Wczytuje ksiege: `slowo=znak` w kazdej linii, naglowek pomijany.
///
/// Slowo zapisujemy malymi literami, bo wejscie od czlowieka nie trzyma sie wielkosci liter,
/// a znak ma byc ten sam niezaleznie od tego, jak ktos je napisal.
fn load_book(path: &str) -> std::io::Result<HashMap<String, String>> {
    let text = fs::read_to_string(path)?;
    Ok(text
        .lines()
        .skip(1)
        .filter_map(|l| {
            let (word, symbol) = l.split_once('=')?;
            let (word, symbol) = (word.trim(), symbol.trim());
            if word.is_empty() || symbol.is_empty() {
                return None;
            }
            Some((word.to_lowercase(), symbol.to_string()))
        })
        .collect())
}

/// Buduje wyszukiwarke pojec: kazde slowo z ksiegi dostaje wektor obok swojego znaku.
fn do_index(
    store: &HttpClient,
    engine: &MiniLmEmbedder,
    collection: &str,
    book: &HashMap<String, String>,
    limit: usize,
) -> i32 {
    // Kolekcje zakladamy jawnie, z wymiarem odczytanym z silnika. Wymiar wpisany na sztywno
    // rozjechalby sie po pierwszej zmianie modelu, i to po cichu - magazyn przyjalby zapis.
    let create = json!({
        "vectors": { "size": engine.dimensions(), "distance": "Cosine" }
    })
    .to_string();
    match store.put(&format!("/collections/{collection}"), &create) {
        Ok(_) => println!(
            "kolekcja {collection} gotowa ({} liczb)",
            engine.dimensions()
        ),
        Err(e) => println!("kolekcja {collection}: {e} (zakladam, ze juz jest)"),
    }

    let mut words: Vec<(&String, &String)> = book.iter().collect();
    // Kolejnosc musi byc powtarzalna, inaczej numer punktu oznaczalby co innego przy
    // kazdym przebiegu i dopisanie jednego slowa przemieszaloby cala kolekcje.
    words.sort_by(|a, b| a.0.cmp(b.0));
    if limit > 0 {
        words.truncate(limit);
    }

    let mut points = Vec::new();
    let mut sent = 0usize;
    let mut skipped = 0usize;
    for (index, (word, symbol)) in words.iter().enumerate() {
        match engine.embed(word) {
            Ok(vector) => points.push(json!({
                "id": index,
                "vector": vector,
                "payload": { "word": word, "symbol": symbol }
            })),
            Err(_) => skipped += 1,
        }
        if points.len() >= BATCH {
            sent += flush(store, collection, &mut points);
            if sent % 5000 < BATCH {
                println!("  ... {sent} pojec");
            }
        }
    }
    sent += flush(store, collection, &mut points);
    println!("zapisanych pojec: {sent} | pominietych: {skipped}");
    if sent == 0 { 1 } else { 0 }
}

fn flush(store: &HttpClient, collection: &str, points: &mut Vec<Value>) -> usize {
    if points.is_empty() {
        return 0;
    }
    let n = points.len();
    let body = json!({ "points": points }).to_string();
    points.clear();
    match store.put(&format!("/collections/{collection}/points"), &body) {
        Ok(_) => n,
        Err(e) => {
            eprintln!("partia odrzucona ({n} pojec): {e}");
            0
        }
    }
}

/// Zamienia tekst na znaki CBMS: najpierw ksiega, potem wyszukiwanie, na koncu przyznanie
/// sie do niewiedzy.
fn do_run(
    store: &HttpClient,
    engine: &MiniLmEmbedder,
    collection: &str,
    book: &HashMap<String, String>,
    text: &str,
    threshold: f64,
) -> i32 {
    if text.trim().is_empty() {
        eprintln!("FAIL: --text jest puste");
        return 2;
    }
    let mut symbols = String::new();
    let mut rows = Vec::new();
    let (mut exact, mut found, mut unknown) = (0usize, 0usize, 0usize);

    for raw in text.split_whitespace() {
        let word: String = raw
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        if word.is_empty() {
            continue;
        }
        if let Some(sym) = book.get(&word) {
            symbols.push_str(sym);
            exact += 1;
            rows.push(format!("  {word:<16} {sym}   ksiega"));
            continue;
        }
        match nearest(store, engine, collection, &word, threshold) {
            Ok(Some((sym, near, score, margin))) => {
                symbols.push_str(&sym);
                found += 1;
                rows.push(format!(
                    "  {word:<16} {sym}   ~ {near} ({score:.4}, odstep {margin:.4})"
                ));
            }
            Ok(None) => {
                unknown += 1;
                rows.push(format!("  {word:<16} -    NIEZNANE"));
            }
            Err(e) => {
                unknown += 1;
                rows.push(format!("  {word:<16} -    BLAD: {e}"));
            }
        }
    }

    for r in &rows {
        println!("{r}");
    }
    println!("\nz ksiegi: {exact} | dobrane: {found} | nieznane: {unknown}");
    println!("znaki CBMS: {symbols}");
    0
}

/// Najblizsze pojecie, o ile jest dosc blisko I wyraznie blizsze od nastepnego.
///
/// Dwa warunki, nie jeden. Sama wysokosc oceny nie wystarcza: pomiar z 2026-09-05 pokazal
/// przypadek, w ktorym zla odpowiedz miala 0.5997, a dobra 0.4702 - roznil je dopiero odstep
/// do drugiego wyniku (0.0081 wobec 0.2437). Slowo pasujace jednakowo do pieciu rzeczy nie
/// ma jednego znaczenia i musi zostac nieznane.
fn nearest(
    store: &HttpClient,
    engine: &MiniLmEmbedder,
    collection: &str,
    word: &str,
    threshold: f64,
) -> Result<Option<(String, String, f64, f64)>, String> {
    let vector = engine.embed(word).map_err(|e| e.to_string())?;
    let body = json!({ "vector": vector, "limit": 2, "with_payload": true }).to_string();
    let text = store
        .post(&format!("/collections/{collection}/points/search"), &body)
        .map_err(|e| e.to_string())?;
    let parsed: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let empty = vec![];
    let hits = parsed["result"].as_array().unwrap_or(&empty);
    let Some(best) = hits.first() else {
        return Ok(None);
    };
    let score = best["score"].as_f64().unwrap_or(0.0);
    let second = hits.get(1).and_then(|h| h["score"].as_f64()).unwrap_or(0.0);
    let margin = score - second;
    if score < threshold || margin < MIN_MARGIN {
        return Ok(None);
    }
    let sym = best["payload"]["symbol"].as_str().unwrap_or("").to_string();
    let near = best["payload"]["word"].as_str().unwrap_or("").to_string();
    if sym.is_empty() {
        return Ok(None);
    }
    Ok(Some((sym, near, score, margin)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_output_matches_frontend_u32() {
        let path = std::env::var("NOWORODEK_BOOK")
            .or_else(|_| std::env::var("CBMS_BOOK"))
            .expect("Ustaw NOWORODEK_BOOK lub CBMS_BOOK na ksiege baseline.");
        let book = darkstar_embed::load_cbms_book(&path).unwrap();
        let mut results = Vec::new();
        for text in ["kodo", "ĉifro", "memoro", "memorilo", "cifro"] {
            let mut output = Vec::new();
            assert_eq!(do_canonical(&path, text, &mut output), 0);
            let ids: Vec<u32> = serde_json::from_slice(&output).unwrap();
            assert_eq!(ids, darkstar_embed::frontend_to_cbms(&book, text).unwrap());
            println!("canonical {text}: {ids:?}");
            results.push(ids);
        }
        assert_ne!(results[1], results[4], "ĉifro != cifro");
        assert!(results.iter().flatten().any(|id| *id > 65535));
    }

    #[test]
    fn canonical_empty_text_rejected_before_book_load() {
        let mut output = Vec::new();
        assert_eq!(do_canonical("", " \t", &mut output), 2);
        assert!(output.is_empty());
    }

    #[test]
    fn canonical_book_error_does_not_emit_ids() {
        let mut output = Vec::new();
        assert_eq!(do_canonical("", "kodo", &mut output), 1);
        assert!(output.is_empty());
    }

    #[test]
    fn czyta_ksiege_pomijajac_naglowek() {
        let tmp = std::env::temp_dir().join("tokenize-test-book.txt");
        fs::write(&tmp, "CODEBOOK_CBMS_ES\nnulo=\u{96f6}\nMemory=\u{4e00}\n").unwrap();
        let b = load_book(tmp.to_str().unwrap()).unwrap();
        assert_eq!(b.len(), 2, "naglowek nie moze byc pojeciem");
        assert_eq!(b.get("nulo").map(String::as_str), Some("\u{96f6}"));
        assert_eq!(
            b.get("memory").map(String::as_str),
            Some("\u{4e00}"),
            "wielkosc liter nie moze zmieniac znaku"
        );
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn linia_bez_znaku_rownosci_jest_pomijana() {
        let tmp = std::env::temp_dir().join("tokenize-test-bad.txt");
        fs::write(&tmp, "CODEBOOK_CBMS_ES\nsmiec bez rownosci\nok=\u{4e8c}\n").unwrap();
        let b = load_book(tmp.to_str().unwrap()).unwrap();
        assert_eq!(b.len(), 1);
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn pusty_znak_nie_tworzy_pojecia() {
        let tmp = std::env::temp_dir().join("tokenize-test-empty.txt");
        fs::write(&tmp, "CODEBOOK_CBMS_ES\nslowo=\n").unwrap();
        assert!(load_book(tmp.to_str().unwrap()).unwrap().is_empty());
        let _ = fs::remove_file(&tmp);
    }
}
