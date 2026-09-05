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
//!
//! Uzycie:
//! ```text
//! implant <wagi.nwrd> <ksiega.txt> <wynik.nwrd> [--bridge most.tsv] [--limit N]
//! ```

use std::collections::HashMap;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

use darkstar_embed::MiniLmEmbedder;
use darkstar_shadow::Embedder;

const MAGIC: &[u8; 4] = b"NWRD";
const EMBED_NAME: &str = "model.embeddings.token.weight";
const NORM_NAME: &str = "model.final_norm.weight";

struct Tensor {
    name: String,
    values: Vec<f32>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("uzycie: implant <wagi.nwrd> <ksiega.txt> <wynik.nwrd> [--bridge most.tsv] [--limit N]");
        eprintln!();
        eprintln!("Wymaga DARKSTAR_EMBED_MODEL_DIR i ORT_DYLIB_PATH.");
        std::process::exit(2);
    }
    let opt = |name: &str| -> Option<String> {
        args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
    };
    // Ograniczenie liczby znakow: kazdy wymaga jednego przejscia przez model, wiec pelna
    // ksiega to dziesiatki tysiecy wywolan. Przy sprawdzaniu, czy pomysl w ogole dziala,
    // kilka tysiecy wystarcza i kosztuje minuty zamiast godzin.
    let limit: usize = opt("--limit").and_then(|v| v.parse().ok()).unwrap_or(usize::MAX);

    let mut tensors = match read_nwrd(&args[0]) {
        Ok(t) => t,
        Err(e) => { eprintln!("FAIL: wagi: {e}"); std::process::exit(1); }
    };
    let hidden = match tensors.iter().find(|t| t.name == NORM_NAME).map(|t| t.values.len()) {
        Some(h) if h > 0 => h,
        _ => { eprintln!("FAIL: brak {NORM_NAME} - nie ustale wymiaru"); std::process::exit(1); }
    };

    let book = match std::fs::read_to_string(&args[1]) {
        Ok(t) => t,
        Err(e) => { eprintln!("FAIL: ksiega: {e}"); std::process::exit(1); }
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
                bridge.entry(root.trim().to_string()).or_default().push(word.trim().to_string());
            }
        }
        let multi = bridge.values().filter(|v| v.len() >= 2).count();
        println!("most: {} rdzeni, w tym {multi} laczacych jezyki", bridge.len());
    }

    let engine = match MiniLmEmbedder::from_env() {
        Ok(e) => e,
        Err(e) => { eprintln!("FAIL: model znaczen: {e}"); std::process::exit(1); }
    };
    println!("model znaczen: {} ({} liczb) -> rdzen ({hidden} liczb)", engine.name(), engine.dimensions());

    // Kolejnosc wpisow ksiegi wyznacza numery znakow, tak samo jak przy kodowaniu:
    // pierwszy wpis to numery 0 i 1, drugi 2 i 3, i tak dalej. Wersja ze spacja dostaje
    // ten sam wektor co bez - to jest to samo slowo, tylko poprzedzone odstepem.
    let mut done = 0usize;
    let mut multilingual = 0usize;
    let mut vectors: Vec<(usize, Vec<f32>)> = Vec::new();

    for (index, line) in book.lines().enumerate() {
        if done >= limit { break }
        let Some((word, _)) = line.split_once('=') else { continue };
        let word = word.trim_end_matches('\\').trim();
        if word.is_empty() || word.len() > 64 { continue }

        // Jesli slowo jest rdzeniem majacym na moscie kilka form, usredniamy je wszystkie.
        // Znak dostaje wtedy opis wielojezyczny, a nie opis jednego jezyka.
        let subject: Vec<String> = match bridge.get(word) {
            Some(forms) if forms.len() >= 2 => { multilingual += 1; forms.clone() }
            _ => vec![word.to_string()],
        };

        let mut sum = vec![0.0f32; engine.dimensions()];
        let mut counted = 0.0f32;
        for form in &subject {
            if let Ok(v) = engine.embed(form) {
                for (a, b) in sum.iter_mut().zip(&v) { *a += b }
                counted += 1.0;
            }
        }
        if counted == 0.0 { continue }
        for v in sum.iter_mut() { *v /= counted }

        vectors.push((index, project(&sum, hidden)));
        done += 1;
        if done.is_multiple_of(500) { println!("  ... {done} znakow"); }
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
    for (index, vector) in &vectors {
        // plain na 2i, spaced na 2i+1 - obie wersje tego samego slowa dostaja ten sam opis.
        for id in [index * 2, index * 2 + 1] {
            if id >= vocab { continue }
            let start = id * hidden;
            if let Some(slot) = table.values.get_mut(start..start + hidden) {
                slot.copy_from_slice(vector);
                written += 1;
            }
        }
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
    if norm > 0.0 { for v in out.iter_mut() { *v /= norm } }
    out
}

fn sign(i: usize, j: usize) -> f32 {
    let mut h = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (j as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
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
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "zly znacznik, oczekiwano NWRD"));
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
        out.push(Tensor { name: String::from_utf8_lossy(&name).into_owned(), values });
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
        for v in &t.values { w.write_all(&v.to_le_bytes())?; }
    }
    w.flush()
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

        assert!(dot(&pa, &pb) > dot(&pa, &pc), "podobne musi zostac blizej niz rozne");
    }

    #[test]
    fn projection_is_repeatable() {
        let v: Vec<f32> = (0..384).map(|i| i as f32).collect();
        assert_eq!(project(&v, 128), project(&v, 128), "ten sam wektor musi dac ten sam wynik");
    }

    #[test]
    fn projection_returns_unit_length() {
        let v: Vec<f32> = (0..384).map(|i| (i % 7) as f32).collect();
        let p = project(&v, 128);
        let norm = p.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "dlugosc {norm}");
    }
}
