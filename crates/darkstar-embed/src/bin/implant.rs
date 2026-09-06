// darkstar-header-v1
// po co: implant.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Wszczepia rdzeniowi zrozumienie znaczen - wektory znakow liczone modelem, ktory je zna.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 20:15:00
//! REASON FOR CREATION: Zmierzone: osadzenia rdzenia nie rozrozniaja znaczen. Wektory znakow
//! powstawaly z LOSOWEGO RZUTU sasiedztwa i przy 128 wymiarach na 33 tysiace znakow rzut jest
//! za ciasny - kazde zapytanie do slownika odruchow dawalo oceny miedzy 0.89 a 0.95, czyli
//! szum. Marcin zapytal, czy da sie wmontowac rdzeniowi baze wektorowa w leb. Da sie: to jest
//! to samo, co zrobilismy z tablica czestosci, tylko wpisywana wiedza jest inna.
//! MECHANICS: Dla kazdego wpisu ksiegi bierze SLOWO, ktore ten znak reprezentuje, liczy jego
//! osadzenie modelem all-MiniLM-L6-v2 (384 liczby), rzutuje na wymiar rdzenia i wpisuje jako
//! wiersz tablicy znakow. Slowa prowadzace na ten sam rdzen esperancki sa USREDNIANE przed
//! rzutem - dzieki temu znak dostaje opis wielojezyczny, czego sam MiniLM nie potrafi.
//! SYSTEM PART: Darkstar / wszczepianie znaczen.
//! ARCHITECTURE FUNCTION: Domyka droge, ktorej brakowalo: rdzen mysli w CBMS, gdzie jezyk nie
//! istnieje, ale nie wiedzial, ktore znaki znacza podobne rzeczy. Teraz to wie - nie z nauki,
//! tylko z wszczepienia.
//! DEPENDENCIES/LINKS: darkstar-embed (MiniLmEmbedder), ksiega CBMS, most esperanto, plik wag
//! w formacie NWRD.
//! TECH STACK: Rust 2024.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//! REVISION 2026-09-06: index*2 / index*2+1 was a historical wrong assumption that
//! line number equals symbol slot. Active CBMS (cbms-writing vocab.rs) uses
//! SYMBOL_BASE=272 with plain/spaced ids at base plus 2*slot (interleaved). Book
//! line number is not the symbol slot (headers, blanks, non-equals notes, and
//! duplicate symbols do not mint ids). Implant aligned to that real CBMS contract;
//! OOB vocab ids fail loud instead of silent skip.
//! REVISION 2026-09-06b: local helpers mirror the CBMS Book/Vocabulary contract; they are NOT an import/shared impl from cbms_writing (drift risk — shared parser = later refactor).
//! ==========================================
//!
//! Uzycie:
//! ```text
//! implant <wagi.nwrd> <ksiega.txt> <wynik.nwrd> [--bridge most.tsv] [--limit N]
//! ```

use std::collections::{HashMap, HashSet};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

use darkstar_embed::MiniLmEmbedder;
use darkstar_shadow::Embedder;

const MAGIC: &[u8; 4] = b"NWRD";
const EMBED_NAME: &str = "model.embeddings.token.weight";
const NORM_NAME: &str = "model.final_norm.weight";

/// Control block (16) + raw bytes (256). Matches cbms-writing `vocab.rs` SYMBOL_BASE.
const SYMBOL_BASE: usize = 272;

struct Tensor {
    name: String,
    values: Vec<f32>,
}

/// Plain and spaced embedding row ids for a CBMS symbol slot.
///
/// Contract (cbms-writing vocab.rs): `id = SYMBOL_BASE + 2 * slot`, spaced = plain + 1.
fn cbms_embedding_rows(symbol_slot: usize) -> Result<(usize, usize), String> {
    let plain_id = SYMBOL_BASE
        .checked_add(
            symbol_slot
                .checked_mul(2)
                .ok_or_else(|| format!("symbol_slot {symbol_slot} overflowed *2"))?,
        )
        .ok_or_else(|| format!("symbol_slot {symbol_slot} overflowed SYMBOL_BASE+2*slot"))?;
    let spaced_id = plain_id
        .checked_add(1)
        .ok_or_else(|| format!("plain_id {plain_id} overflowed +1"))?;
    Ok((plain_id, spaced_id))
}

/// Local mirror of cbms-writing `book.rs` entry minting: which lines become symbol slots.
/// Skips blanks, section headers, lines without `=`, empty sides, bad U+ values, and
/// duplicate symbols (first claimant wins — same as Book::parse_lenient + Vocabulary::new).
fn next_book_symbol_entry(
    line: &str,
    seen_symbols: &mut HashSet<String>,
) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    if line == "CODEBOOK_CBMS_ES"
        || line.starts_with("CBMS-Eo-v1.1")
        || line.starts_with("CBMS-CODES-v1")
    {
        return None;
    }
    let (raw_root, value) = split_on_separator(line)?;
    let root = unescape_root(raw_root.trim());
    let value = value.trim();
    if root.is_empty() || value.is_empty() {
        return None;
    }
    let symbol = decode_value(value)?;
    if !seen_symbols.insert(symbol.clone()) {
        return None;
    }
    Some((root, symbol))
}

/// Mirror of cbms-writing book.rs `split_on_separator` — `=` after `\` is literal.
fn split_on_separator(line: &str) -> Option<(&str, &str)> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'=' => return Some((&line[..i], &line[i + 1..])),
            _ => i += 1,
        }
    }
    None
}

fn unescape_root(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn decode_value(value: &str) -> Option<String> {
    if let Some(hex) = value.strip_prefix("U+") {
        let mut out = String::new();
        for part in hex.split("U+") {
            let cp = u32::from_str_radix(part, 16).ok()?;
            out.push(char::from_u32(cp)?);
        }
        Some(out)
    } else {
        Some(value.to_string())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!(
            "uzycie: implant <wagi.nwrd> <ksiega.txt> <wynik.nwrd> [--bridge most.tsv] [--limit N]"
        );
        eprintln!();
        eprintln!("Wymaga DARKSTAR_EMBED_MODEL_DIR i ORT_DYLIB_PATH.");
        std::process::exit(2);
    }
    let opt = |name: &str| -> Option<String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    // Ograniczenie liczby znakow: kazdy wymaga jednego przejscia przez model, wiec pelna
    // ksiega to dziesiatki tysiecy wywolan. Przy sprawdzaniu, czy pomysl w ogole dziala,
    // kilka tysiecy wystarcza i kosztuje minuty zamiast godzin.
    let limit: usize = opt("--limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);

    let mut tensors = match read_nwrd(&args[0]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: wagi: {e}");
            std::process::exit(1);
        }
    };
    let hidden = match tensors
        .iter()
        .find(|t| t.name == NORM_NAME)
        .map(|t| t.values.len())
    {
        Some(h) if h > 0 => h,
        _ => {
            eprintln!("FAIL: brak {NORM_NAME} - nie ustale wymiaru");
            std::process::exit(1);
        }
    };

    let book = match std::fs::read_to_string(&args[1]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: ksiega: {e}");
            std::process::exit(1);
        }
    };

    // Most: rdzen -> lista slow, ktore na niego prowadza. Slowa z roznych jezykow trafiaja
    // na ten sam rdzen i to jest dokladnie ta informacja, ktorej model angielski nie ma.
    let mut bridge: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(path) = opt("--bridge")
        && let Ok(text) = std::fs::read_to_string(&path)
    {
        for line in text.lines().skip(1) {
            let mut parts = line.split('\t');
            if let (Some(word), Some(root)) = (parts.next(), parts.next()) {
                bridge
                    .entry(root.trim().to_string())
                    .or_default()
                    .push(word.trim().to_string());
            }
        }
        let multi = bridge.values().filter(|v| v.len() >= 2).count();
        println!(
            "most: {} rdzeni, w tym {multi} laczacych jezyki",
            bridge.len()
        );
    }

    let engine = match MiniLmEmbedder::from_env() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("FAIL: model znaczen: {e}");
            std::process::exit(1);
        }
    };
    println!(
        "model znaczen: {} ({} liczb) -> rdzen ({hidden} liczb)",
        engine.name(),
        engine.dimensions()
    );

    // Slot follows CBMS Vocabulary::new order (unique symbols), NOT physical line index.
    // plain_id = SYMBOL_BASE + 2*slot, spaced_id = plain_id + 1 — both get the same vector.
    let mut done = 0usize;
    let mut multilingual = 0usize;
    let mut symbol_slot = 0usize;
    let mut seen_symbols: HashSet<String> = HashSet::new();
    let mut vectors: Vec<(usize, String, Vec<f32>)> = Vec::new();

    for line in book.lines() {
        if done >= limit {
            break;
        }
        let Some((word, symbol)) = next_book_symbol_entry(line, &mut seen_symbols) else {
            continue;
        };
        let slot = symbol_slot;
        symbol_slot += 1;

        if word.len() > 64 {
            continue;
        }

        // Jesli slowo jest rdzeniem majacym na moscie kilka form, usredniamy je wszystkie.
        // Znak dostaje wtedy opis wielojezyczny, a nie opis jednego jezyka.
        let subject: Vec<String> = match bridge.get(word.as_str()) {
            Some(forms) if forms.len() >= 2 => {
                multilingual += 1;
                forms.clone()
            }
            _ => vec![word.clone()],
        };

        let mut sum = vec![0.0f32; engine.dimensions()];
        let mut counted = 0.0f32;
        for form in &subject {
            if let Ok(v) = engine.embed(form) {
                for (a, b) in sum.iter_mut().zip(&v) {
                    *a += b
                }
                counted += 1.0;
            }
        }
        if counted == 0.0 {
            continue;
        }
        for v in sum.iter_mut() {
            *v /= counted
        }

        vectors.push((slot, symbol, project(&sum, hidden)));
        done += 1;
        if done.is_multiple_of(500) {
            println!("  ... {done} znakow");
        }
    }

    println!("osadzonych znakow: {done}, w tym wielojezycznych: {multilingual}");
    if vectors.is_empty() {
        eprintln!("FAIL: zaden znak nie dal sie osadzic");
        std::process::exit(1);
    }

    let Some(table) = tensors.iter_mut().find(|t| t.name == EMBED_NAME) else {
        eprintln!("FAIL: brak {EMBED_NAME}");
        std::process::exit(1);
    };
    let vocab = table.values.len() / hidden;
    let mut written = 0usize;
    for (slot, symbol, vector) in &vectors {
        if let Err(e) =
            implant_rows_or_fail(&mut table.values, hidden, vocab, symbol, *slot, vector)
        {
            eprintln!("{e}");
            std::process::exit(1);
        }
        written += 2;
    }
    println!("wpisanych wierszy: {written} z {vocab}");

    if let Err(e) = write_nwrd(&args[2], &tensors) {
        eprintln!("FAIL: zapis: {e}");
        std::process::exit(1);
    }
    println!("\nzapisano: {}", args[2]);
    println!("zrodlo NIETKNIETE: {}", args[0]);
}

/// Rzutuje wektor na mniejszy wymiar, zachowujac odleglosci.
///
/// Rzut losowy ze stalymi znakami: kazda pozycja wyjscia jest suma pozycji wejscia ze
/// znakami zaleznymi od pary indeksow. Lemat Johnsona-Lindenstraussa mowi, ze taki rzut
/// zachowuje odleglosci z bledem malejacym wraz z wymiarem docelowym - 384 na 128 jest
/// w zakresie, w ktorym blad jest maly.
///
/// Znaki licza sie z indeksow, nie losuja - ten sam wektor daje zawsze ten sam wynik,
/// wiec wszczepienie jest powtarzalne co do bitu.
fn project(source: &[f32], target: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; target];
    for (i, v) in source.iter().enumerate() {
        for (j, o) in out.iter_mut().enumerate() {
            *o += v * sign(i, j);
        }
    }
    // Normalizacja do dlugosci 1: magazyn i model porownuja kierunki, a dlugosc po sumowaniu
    // rosnie z wymiarem zrodla i niosla by tylko informacje o tym, ile bylo skladnikow.
    let norm = out.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in out.iter_mut() {
            *v /= norm
        }
    }
    out
}

fn sign(i: usize, j: usize) -> f32 {
    let mut h = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (j as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    if h & 1 == 0 { 1.0 } else { -1.0 }
}

fn read_nwrd(path: &str) -> std::io::Result<Vec<Tensor>> {
    let mut r = BufReader::new(File::open(path)?);
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "zly znacznik, oczekiwano NWRD",
        ));
    }
    let mut b4 = [0u8; 4];
    let mut b8 = [0u8; 8];
    r.read_exact(&mut b4)?;
    r.read_exact(&mut b4)?;
    let count = u32::from_le_bytes(b4);
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        r.read_exact(&mut b4)?;
        let n = u32::from_le_bytes(b4) as usize;
        let mut name = vec![0u8; n];
        r.read_exact(&mut name)?;
        r.read_exact(&mut b8)?;
        let vc = u64::from_le_bytes(b8) as usize;
        let mut values = Vec::with_capacity(vc);
        for _ in 0..vc {
            r.read_exact(&mut b4)?;
            values.push(f32::from_le_bytes(b4));
        }
        out.push(Tensor {
            name: String::from_utf8_lossy(&name).into_owned(),
            values,
        });
    }
    Ok(out)
}

fn write_nwrd(path: &str, tensors: &[Tensor]) -> std::io::Result<()> {
    let mut w = BufWriter::new(File::create(path)?);
    w.write_all(MAGIC)?;
    w.write_all(&1u32.to_le_bytes())?;
    w.write_all(&(tensors.len() as u32).to_le_bytes())?;
    for t in tensors {
        w.write_all(&(t.name.len() as u32).to_le_bytes())?;
        w.write_all(t.name.as_bytes())?;
        w.write_all(&(t.values.len() as u64).to_le_bytes())?;
        for v in &t.values {
            w.write_all(&v.to_le_bytes())?;
        }
    }
    w.flush()
}

#[cfg(test)]
/// Pure mirror of main slotting: Book entry → symbol_slot BEFORE optional embed/write.
/// `embed_ok(word)` stands in for MiniLM success without ONNX/weights.
fn plan_writes_after_embed_filter(
    book: &str,
    embed_ok: impl Fn(&str) -> bool,
) -> Vec<(usize, String, usize, usize)> {
    let mut seen = HashSet::new();
    let mut symbol_slot = 0usize;
    let mut planned = Vec::new();
    for line in book.lines() {
        let Some((word, symbol)) = next_book_symbol_entry(line, &mut seen) else {
            continue;
        };
        let slot = symbol_slot;
        symbol_slot += 1;
        if word.len() > 64 {
            continue;
        }
        if !embed_ok(word.as_str()) {
            continue;
        }
        let (plain_id, spaced_id) = cbms_embedding_rows(slot).expect("slot ids");
        planned.push((slot, symbol, plain_id, spaced_id));
    }
    planned
}

/// Apply CBMS row ids into an embedding table; loud error if out of vocab.
fn implant_rows_or_fail(
    table: &mut [f32],
    hidden: usize,
    vocab: usize,
    symbol: &str,
    symbol_slot: usize,
    vector: &[f32],
) -> Result<(), String> {
    let (plain_id, spaced_id) = cbms_embedding_rows(symbol_slot)?;
    if plain_id >= vocab || spaced_id >= vocab {
        return Err(format!(
            "FAIL: CBMS embedding id out of vocab — symbol={symbol:?} symbol_slot={symbol_slot} plain_id={plain_id} spaced_id={spaced_id} vocab={vocab}"
        ));
    }
    for id in [plain_id, spaced_id] {
        let start = id * hidden;
        let row = table
            .get_mut(start..start + hidden)
            .ok_or_else(|| {
                format!(
                    "FAIL: CBMS embedding row missing — symbol={symbol:?} symbol_slot={symbol_slot} plain_id={plain_id} spaced_id={spaced_id} vocab={vocab}"
                )
            })?;
        row.copy_from_slice(vector);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_keeps_similar_things_similar() {
        // Sedno rzutu: wektory bliskie w 384 wymiarach maja zostac bliskie w 128.
        // Gdyby rzut tego nie zachowywal, wszczepienie przenioslo by szum zamiast znaczenia.
        let a: Vec<f32> = (0..384).map(|i| (i as f32 * 0.01).sin()).collect();
        let mut b = a.clone();
        b[0] += 0.01; // prawie to samo
        let c: Vec<f32> = (0..384).map(|i| (i as f32 * 0.07).cos()).collect(); // co innego

        let (pa, pb, pc) = (project(&a, 128), project(&b, 128), project(&c, 128));
        let dot = |x: &[f32], y: &[f32]| x.iter().zip(y).map(|(p, q)| p * q).sum::<f32>();

        assert!(
            dot(&pa, &pb) > dot(&pa, &pc),
            "podobne musi zostac blizej niz rozne"
        );
    }

    #[test]
    fn projection_is_repeatable() {
        let v: Vec<f32> = (0..384).map(|i| i as f32).collect();
        assert_eq!(
            project(&v, 128),
            project(&v, 128),
            "ten sam wektor musi dac ten sam wynik"
        );
    }

    #[test]
    fn projection_returns_unit_length() {
        let v: Vec<f32> = (0..384).map(|i| (i % 7) as f32).collect();
        let p = project(&v, 128);
        let norm = p.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "dlugosc {norm}");
    }

    #[test]
    fn cbms_rows_slot0_and_slot1() {
        assert_eq!(cbms_embedding_rows(0).unwrap(), (272, 273));
        assert_eq!(cbms_embedding_rows(1).unwrap(), (274, 275));
    }

    #[test]
    fn cbms_rows_large_slot_exceeds_u16() {
        let (plain, spaced) = cbms_embedding_rows(32768).unwrap();
        assert!(plain > 65535, "plain={plain}");
        assert!(spaced > 65535, "spaced={spaced}");
        assert_eq!(plain, SYMBOL_BASE + 32768 * 2);
        assert_eq!(spaced, plain + 1);
    }

    #[test]
    fn book_fixture_non_slot_lines_do_not_advance_symbol_slot() {
        // symbol / non-= / symbol / non-= / non-= / symbol
        let book = "\
CODEBOOK_CBMS_ES
alpha=Ա
this line has no separator
beta=б
just a note
CBMS-Eo-v1.1-EXT
gamma=U+25B6
";
        let mut seen = HashSet::new();
        let mut slots = Vec::new();
        let mut symbol_slot = 0usize;
        for line in book.lines() {
            let Some((root, symbol)) = next_book_symbol_entry(line, &mut seen) else {
                continue;
            };
            let (plain, spaced) = cbms_embedding_rows(symbol_slot).unwrap();
            slots.push((root, symbol, symbol_slot, plain, spaced));
            symbol_slot += 1;
        }
        assert_eq!(slots.len(), 3, "exactly three symbol slots; got {slots:?}");
        assert_eq!(slots[0].2, 0);
        assert_eq!(slots[0].3, 272);
        assert_eq!(slots[0].4, 273);
        assert_eq!(slots[1].2, 1);
        assert_eq!(slots[1].3, 274);
        assert_eq!(slots[1].4, 275);
        assert_eq!(slots[2].2, 2);
        assert_eq!(slots[2].3, 276);
        assert_eq!(slots[2].4, 277);
        assert_eq!(slots[0].0, "alpha");
        assert_eq!(slots[1].0, "beta");
        assert_eq!(slots[2].0, "gamma");
    }

    #[test]
    fn several_skipped_lines_do_not_desync_ids() {
        let book = "\
CODEBOOK_CBMS_ES

# comment without equals
homo=Ա

blank above and below

viro=բ
not-an-entry
CBMS-CODES-v1
1 2 3 4
skribi=г
homo=Ա
";
        let mut seen = HashSet::new();
        let mut ids = Vec::new();
        let mut symbol_slot = 0usize;
        for line in book.lines() {
            if next_book_symbol_entry(line, &mut seen).is_none() {
                continue;
            }
            ids.push(cbms_embedding_rows(symbol_slot).unwrap());
            symbol_slot += 1;
        }
        // homo, viro, skribi — duplicate homo must not mint a fourth slot
        assert_eq!(ids, vec![(272, 273), (274, 275), (276, 277)]);
    }

    #[test]
    fn oob_vocab_fails_loud_not_silent_skip() {
        let hidden = 2;
        // plain_id=272 fits; spaced_id=273 must trip loud OOB (not silent skip)
        let vocab = 273;
        let mut table = vec![0.0f32; vocab * hidden];
        let vector = vec![1.0f32, 0.0];
        let err = implant_rows_or_fail(&mut table, hidden, vocab, "Ա", 0, &vector)
            .expect_err("must fail loud when spaced_id >= vocab");
        assert!(err.contains("symbol=\"Ա\""), "{err}");
        assert!(err.contains("symbol_slot=0"), "{err}");
        assert!(err.contains("plain_id=272"), "{err}");
        assert!(err.contains("spaced_id=273"), "{err}");
        assert!(err.contains("vocab=273"), "{err}");
        // table untouched on failure path after check — plain may not have been written
        assert!(
            table.iter().all(|&v| v == 0.0),
            "must not partially write on OOB"
        );
    }

    #[test]
    fn ids_do_not_depend_on_embed_success() {
        // #7: A embed OK, B embed skipped, C embed OK.
        // C must keep slot2 → 276/277 — must NOT compress onto B's 274/275.
        let book = "\
CODEBOOK_CBMS_ES
alpha=Ա
beta=բ
gamma=գ
";
        let planned = plan_writes_after_embed_filter(book, |word| word != "beta");
        assert_eq!(
            planned,
            vec![
                (0usize, "Ա".to_string(), 272, 273),
                (2usize, "գ".to_string(), 276, 277),
            ],
            "C must not inherit B's ids after embed skip"
        );

        let hidden = 2;
        let vocab = 280;
        let mut table = vec![0.0f32; vocab * hidden];
        let va = vec![1.0f32, 0.0];
        let vc = vec![0.0f32, 1.0];
        implant_rows_or_fail(&mut table, hidden, vocab, "Ա", 0, &va).unwrap();
        // B skipped — no write at slot1
        implant_rows_or_fail(&mut table, hidden, vocab, "գ", 2, &vc).unwrap();

        assert_eq!(&table[272 * hidden..272 * hidden + hidden], &[1.0, 0.0]);
        assert_eq!(&table[273 * hidden..273 * hidden + hidden], &[1.0, 0.0]);
        assert_eq!(&table[274 * hidden..274 * hidden + hidden], &[0.0, 0.0]);
        assert_eq!(&table[275 * hidden..275 * hidden + hidden], &[0.0, 0.0]);
        assert_eq!(&table[276 * hidden..276 * hidden + hidden], &[0.0, 1.0]);
        assert_eq!(&table[277 * hidden..277 * hidden + hidden], &[0.0, 1.0]);
    }

    #[test]
    fn implant_rows_writes_both_plain_and_spaced() {
        let hidden = 2;
        let vocab = 280;
        let mut table = vec![0.0f32; vocab * hidden];
        let vector = vec![0.5f32, -0.25];
        implant_rows_or_fail(&mut table, hidden, vocab, "Ա", 1, &vector).unwrap();
        assert_eq!(&table[274 * hidden..274 * hidden + hidden], &[0.5, -0.25]);
        assert_eq!(&table[275 * hidden..275 * hidden + hidden], &[0.5, -0.25]);
    }
}
