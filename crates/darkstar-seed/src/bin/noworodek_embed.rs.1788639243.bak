// darkstar-header-v1
// po co: noworodek_embed.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Zamienia zdanie zapisane w CBMS na JEDEN wektor - opis calego zdania, nie pojedynczych
//! znakow.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 14:00:00
//! REASON FOR CREATION: Noworodek umie przewidziec nastepny znak, ale nie umie powiedziec,
//! o czym jest cale zdanie - a bez tego nie wepnie sie w gniazdo ucznia (darkstar-shadow),
//! ktore wymaga `embed(&str) -> Vec<f32>`. Model ma wektor NA ZNAK; brakuje zwijania po
//! znakach. To narzedzie sprawdza, czy zwiniecie w ogole niesie sens, ZANIM powstanie
//! pelna implementacja.
//! MECHANICS: Czyta wagi w formacie NWRD, bierze tablice `model.embeddings.token.weight`,
//! dla kazdego znaku zdania odczytuje jego wiersz, usrednia i normalizuje do dlugosci 1 -
//! tak samo jak robi to all-MiniLM-L6-v2, zeby oba silniki dawaly wektory tej samej natury.
//! Zdanie przyjmuje jako plik .u16 (wyjscie `cbms ids`), bo tokenizacja CBMS zyje w osobnym
//! narzedziu i powielanie jej tutaj byloby czwarta kopia tej samej logiki.
//! SYSTEM PART: Darkstar / zasiew wag, sprawdzenie sensu osadzen.
//! ARCHITECTURE FUNCTION: Krok przed implementacja cechy Embedder dla Noworodka. Jesli
//! zdania o podobnym znaczeniu dostana tu podobne wektory, wpiecie ucznia ma sens; jesli
//! nie - lepiej wiedziec to teraz niz po napisaniu calej warstwy.
//! DEPENDENCIES/LINKS: format NWRD (ten sam czytnik co darkstar-seed), pliki .u16
//! z `cbms <ksiega> ids <tekst> <wyjscie>`.
//! TECH STACK: Rust 2024, zero zaleznosci.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//!
//! Uzycie:
//! ```text
//! noworodek-embed <wagi.nwrd> <zdanie-a.u16> [zdanie-b.u16 ...]
//! ```
//! Przy dwoch i wiecej zdaniach wypisuje takze ich wzajemne podobienstwo - bo pojedynczy
//! wektor nic nie mowi, a dopiero porownanie pokazuje, czy cokolwiek zostalo uchwycone.

use std::{
    fs::File,
    io::{BufReader, Read},
};

const MAGIC: &[u8; 4] = b"NWRD";
const EMBED_NAME: &str = "model.embeddings.token.weight";
/// Warstwa o dlugosci rownej wymiarowi ukrytemu - stad go odczytujemy, zamiast zgadywac.
const NORM_NAME: &str = "model.final_norm.weight";

struct Tensor {
    name: String,
    values: Vec<f32>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("uzycie: noworodek-embed <wagi.nwrd> <zdanie-a.u16> [zdanie-b.u16 ...]");
        eprintln!();
        eprintln!("Pliki .u16 powstaja przez: cbms <ksiega> ids <plik-tekstowy> <wyjscie.u16>");
        std::process::exit(2);
    }

    let tensors = match read_nwrd(&args[0]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: nie moge odczytac wag: {e}");
            std::process::exit(1);
        }
    };

    let Some(table) = tensors.iter().find(|t| t.name == EMBED_NAME) else {
        eprintln!("FAIL: brak tablicy {EMBED_NAME} w pliku wag");
        std::process::exit(1);
    };

    // Wymiar odczytujemy, nie zgadujemy - patrz hidden_size.
    let Some(hidden) = hidden_size(&tensors) else {
        eprintln!("FAIL: brak {NORM_NAME} - nie moge ustalic wymiaru bez zgadywania");
        std::process::exit(1);
    };
    let vocab = table.values.len() / hidden;
    println!("tablica znakow: {vocab} znakow x {hidden} liczb");

    let mut vectors = Vec::new();
    for path in &args[1..] {
        let ids = match read_ids(path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("FAIL: nie moge odczytac {path}: {e}");
                std::process::exit(1);
            }
        };
        let (vector, used) = sentence_vector(&table.values, &ids, hidden, vocab);
        println!(
            "\n{path}\n  znakow w zdaniu : {}\n  znakow znanych  : {used}",
            ids.len()
        );
        if used == 0 {
            println!("  UWAGA: zaden znak nie ma wektora - zdanie nie da sie opisac");
        } else {
            println!("  pierwsze 4 liczby: {:?}", &vector[..4.min(vector.len())]);
        }
        vectors.push((path.clone(), vector, used));
    }

    // Pojedynczy wektor nic nie mowi. Sens widac dopiero w porownaniu.
    if vectors.len() >= 2 {
        println!("\n=== podobienstwo miedzy zdaniami ===");
        for i in 0..vectors.len() {
            for j in (i + 1)..vectors.len() {
                let (name_a, a, used_a) = &vectors[i];
                let (name_b, b, used_b) = &vectors[j];
                match (used_a, used_b) {
                    (0, _) | (_, 0) => println!("  {name_a} vs {name_b}: brak porownania"),
                    _ => {
                        let sim = cosine(a, b).unwrap_or(0.0);
                        println!("  {sim:+.4}  {name_a} vs {name_b}");
                    }
                }
            }
        }
    }
}

/// Srednia wektorow znakow zdania, znormalizowana do dlugosci 1.
///
/// Znaki spoza slownika sa pomijane, a nie zerowane: zero jest wektorem o kierunku, ktorego
/// nie ma, i wciagnieta do sredniej przesunelaby wynik ku poczatkowi ukladu. Zwracana liczba
/// mowi, ile znakow faktycznie wzieto - bez tego nie dalo by sie odroznic zdania dobrze
/// opisanego od takiego, z ktorego zrozumiano jedno slowo.
///
/// Normalizacja jest tu po to, zeby wektory byly tej samej natury co te z all-MiniLM-L6-v2 -
/// tamten model tez konczy normalizacja, a magazyn pamieci porownuje kierunki.
fn sentence_vector(table: &[f32], ids: &[usize], hidden: usize, vocab: usize) -> (Vec<f32>, usize) {
    let mut sum = vec![0.0f32; hidden];
    let mut used = 0usize;

    for &id in ids {
        if id >= vocab {
            continue;
        }
        let start = id * hidden;
        let Some(row) = table.get(start..start + hidden) else {
            continue;
        };
        if row.iter().all(|v| *v == 0.0) {
            continue;
        }
        for (acc, v) in sum.iter_mut().zip(row) {
            *acc += v;
        }
        used += 1;
    }

    if used > 0 {
        let n = used as f32;
        for v in sum.iter_mut() {
            *v /= n;
        }
        let norm = sum.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in sum.iter_mut() {
                *v /= norm;
            }
        }
    }
    (sum, used)
}

/// Odczytuje wymiar ukryty z warstwy, ktora ma go DOKLADNIE tyle, ile wynosi.
///
/// Pierwsza wersja zgadywala go z rozmiaru tablicy znakow, sprawdzajac potegi dwojki od
/// najwiekszej. Test to wywrocil i mial racje: 53746 x 128 dzieli sie rowniez przez 256,
/// wiec zgadywanie zwracalo 256, a przy zlym wymiarze KAZDY wektor jest bez sensu - i to
/// po cichu, bo liczby nadal wygladaja jak liczby.
///
/// `model.final_norm.weight` ma po jednej liczbie na wymiar ukryty, wiec jego dlugosc jest
/// odpowiedzia, a nie poszlaka. Gdyby go zabraklo, lepiej odmowic niz zgadnac.
fn hidden_size(tensors: &[Tensor]) -> Option<usize> {
    tensors
        .iter()
        .find(|t| t.name == NORM_NAME)
        .map(|t| t.values.len())
        .filter(|len| *len > 0)
}

fn cosine(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let (mut dot, mut na, mut nb) = (0.0f64, 0.0f64, 0.0f64);
    for (x, y) in a.iter().zip(b) {
        dot += f64::from(*x) * f64::from(*y);
        na += f64::from(*x) * f64::from(*x);
        nb += f64::from(*y) * f64::from(*y);
    }
    if na == 0.0 || nb == 0.0 {
        return None;
    }
    Some((dot / (na.sqrt() * nb.sqrt())) as f32)
}

fn read_ids(path: &str) -> std::io::Result<Vec<usize>> {
    let bytes = std::fs::read(path)?;
    Ok(bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]) as usize)
        .collect())
}

fn read_nwrd(path: &str) -> std::io::Result<Vec<Tensor>> {
    let mut r = BufReader::new(File::open(path)?);
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("zly znacznik pliku: {magic:?}, oczekiwano NWRD"),
        ));
    }
    let _version = read_u32(&mut r)?;
    let count = read_u32(&mut r)?;

    let mut tensors = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let name_len = read_u32(&mut r)? as usize;
        let mut name_bytes = vec![0u8; name_len];
        r.read_exact(&mut name_bytes)?;
        let name = String::from_utf8_lossy(&name_bytes).into_owned();
        let value_count = read_u64(&mut r)? as usize;
        let mut values = Vec::with_capacity(value_count);
        let mut buf = [0u8; 4];
        for _ in 0..value_count {
            r.read_exact(&mut buf)?;
            values.push(f32::from_le_bytes(buf));
        }
        tensors.push(Tensor { name, values });
    }
    Ok(tensors)
}

fn read_u32(r: &mut impl Read) -> std::io::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_u64(r: &mut impl Read) -> std::io::Result<u64> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_symbols_are_skipped_not_counted_as_zero() {
        // Znak spoza slownika wciagniety do sredniej jako zero przesunalby wynik ku
        // poczatkowi ukladu i zdanie z jednym nieznanym slowem wygladaloby inaczej
        // niz to samo zdanie bez niego.
        let table = vec![1.0, 0.0, /* id 0 */ 0.0, 1.0 /* id 1 */];
        let (with_unknown, used_a) = sentence_vector(&table, &[0, 999], 2, 2);
        let (without, used_b) = sentence_vector(&table, &[0], 2, 2);
        assert_eq!(used_a, 1, "nieznany znak nie moze byc liczony");
        assert_eq!(used_b, 1);
        assert_eq!(with_unknown, without, "nieznany znak nie moze zmieniac wyniku");
    }

    #[test]
    fn empty_rows_are_skipped_too() {
        // Znaki nieobecne w korpusie maja same zera - tak samo nie niosa kierunku.
        let table = vec![1.0, 0.0, 0.0, 0.0];
        let (_, used) = sentence_vector(&table, &[1], 2, 2);
        assert_eq!(used, 0, "wiersz z samych zer nie jest opisem znaku");
    }

    #[test]
    fn result_has_unit_length() {
        let table = vec![3.0, 4.0, 6.0, 8.0];
        let (v, used) = sentence_vector(&table, &[0, 1], 2, 2);
        assert_eq!(used, 2);
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "dlugosc {norm}, oczekiwano 1.0");
    }

    #[test]
    fn hidden_size_is_read_from_the_data() {
        // Ten test wywrocil pierwsza wersje i dlatego zostaje: zgadywanie wymiaru
        // z rozmiaru tablicy dawalo 256 zamiast 128, bo 53746 x 128 dzieli sie przez oba.
        let tensors = vec![
            Tensor { name: EMBED_NAME.into(), values: vec![0.0; 53746 * 128] },
            Tensor { name: NORM_NAME.into(), values: vec![1.0; 128] },
        ];
        assert_eq!(hidden_size(&tensors), Some(128));

        // Bez warstwy okreslajacej wymiar wolimy odmowic niz zgadnac.
        let without = vec![Tensor { name: EMBED_NAME.into(), values: vec![0.0; 256] }];
        assert_eq!(hidden_size(&without), None);
    }

    #[test]
    fn identical_sentences_are_fully_aligned() {
        let table = vec![1.0, 2.0, 3.0, 4.0];
        let (a, _) = sentence_vector(&table, &[0, 1], 2, 2);
        let (b, _) = sentence_vector(&table, &[0, 1], 2, 2);
        let sim = cosine(&a, &b).expect("porownanie");
        assert!((sim - 1.0).abs() < 1e-6, "to samo zdanie musi dac 1.0, jest {sim}");
    }
}
