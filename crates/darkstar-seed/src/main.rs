// darkstar-header-v1
// po co: main.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Wpisuje wiedze wprost w wagi Noworodka, zamiast czekac, az sam sie jej nauczy.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 12:45:00
//! REASON FOR CREATION: Trzy przebiegi treningu skonczyly sie tym samym zdaniem: model
//! jest GORSZY niz zwykla tablica czestosci. Najlepszy z nich - duzy model po 53 minutach -
//! wciaz przegrywal o 2.262 bitu. Skoro tablica czestosci bije wytrenowany model, nie ma
//! powodu uczyc go tego przez godziny: mozna mu to wpisac i pozwolic uczyc sie tego,
//! czego tablica NIE umie - kontekstu.
//! MECHANICS: Czyta plik wag w formacie NWRD, przelicza wiersze glowicy wyjsciowej
//! (`model.lm_head.weight`) proporcjonalnie do log-czestosci symbolu w korpusie, zapisuje
//! NOWY plik. Zrodlo nigdy nie jest nadpisywane. Po zmianie liczy i wypisuje dokladnie to,
//! co `editor.rs` Noworodka nazywa TensorDiff: ile liczb sie ruszylo, suma zmian, najwieksza
//! pojedyncza zmiana - zeby dalo sie powiedziec, co sie stalo, a nie tylko ze cos sie stalo.
//! SYSTEM PART: Darkstar / zasiew wag.
//! ARCHITECTURE FUNCTION: Narzedzie do wpisywania wiedzy, ktora juz mamy, do modelu, ktory
//! jeszcze jej nie ma. Format NWRD obslugiwany wlasnym kodem, bo Noworodek zyje w innym
//! repozytorium, na innej galezi, i trwa tam praca - nie wolno go dotykac.
//! DEPENDENCIES/LINKS: format NWRD z noworodek/src/bin/train-cbms.rs (magic, wersja u32,
//! liczba tensorow u32, dalej per tensor: dlugosc nazwy u32, nazwa, liczba wartosci u64,
//! wartosci f32 little-endian).
//! TECH STACK: Rust 2024, zero zaleznosci. Format jest prosty i czytanie go wprost jest
//! tansze niz wciaganie calej biblioteki Noworodka z innego repozytorium.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//!
//! Uzycie:
//! ```text
//! darkstar-seed <wagi-wejsciowe.nwrd> <log-czestosci.txt> <wagi-wyjsciowe.nwrd> [sila]
//! ```
//! `sila` domyslnie 0.5. Zero znaczy brak zmiany - przydatne do sprawdzenia, ze samo
//! przepisanie pliku niczego nie psuje.

use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

const MAGIC: &[u8; 4] = b"NWRD";
const TARGET: &str = "model.lm_head.weight";

/// Najwiekszy dopuszczalny mnoznik wiersza (i najmniejszy, jako jego odwrotnosc).
///
/// ZMIERZONE, dlaczego to musi tu byc: bez ograniczenia przy sile 0.5 mnozniki siegnely
/// 239.9x, a trening na tak zmienionych wagach zwrocil `odlozone: inf` - liczby wybuchly
/// i model przestal cokolwiek liczyc.
///
/// Przyczyna lezy w ksztalcie danych, nie w sile: z 53746 symbolow slownika w korpusie
/// wystepuje tylko 5312. Pozostale 48434 maja te sama, bardzo niska log-czestosc, wiec
/// zaniżaja srednia i scieskaja odchylenie. Symbole obecne wychodza wtedy daleko poza
/// skale, a `exp` zamienia to w mnozniki rzedu setek.
const MAX_SCALE: f32 = 2.0;

struct Tensor {
    name: String,
    values: Vec<f32>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!(
            "uzycie: darkstar-seed <wagi.nwrd> <log-czestosci.txt> <wynik.nwrd> [sila=0.5]"
        );
        std::process::exit(2);
    }
    let strength: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.5);

    let mut tensors = match read_nwrd(&args[0]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: nie moge odczytac wag: {e}");
            std::process::exit(1);
        }
    };
    println!("wczytano {} tensorow z {}", tensors.len(), args[0]);

    let log_freq = match read_log_frequencies(&args[1]) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("FAIL: nie moge odczytac czestosci: {e}");
            std::process::exit(1);
        }
    };
    println!("wczytano {} log-czestosci", log_freq.len());

    let Some(head) = tensors.iter_mut().find(|t| t.name == TARGET) else {
        eprintln!("FAIL: brak tensora {TARGET} w pliku wag");
        std::process::exit(1);
    };

    // Wymiar bierzemy z danych, nie z zalozenia: liczba wartosci podzielona przez slownik.
    if head.values.len() % log_freq.len() != 0 {
        eprintln!(
            "FAIL: {} liczb nie dzieli sie przez {} symboli - inny slownik niz korpus",
            head.values.len(),
            log_freq.len()
        );
        std::process::exit(1);
    }
    let hidden = head.values.len() / log_freq.len();
    println!("glowica wyjsciowa: {} symboli x {hidden} liczb", log_freq.len());

    let before = head.values.clone();
    let scales = row_scales(&log_freq, strength);
    apply_row_scales(&mut head.values, &scales, hidden);

    report(&before, &head.values, &scales, strength);

    if let Err(e) = write_nwrd(&args[2], &tensors) {
        eprintln!("FAIL: nie moge zapisac: {e}");
        std::process::exit(1);
    }
    println!("\nzapisano: {}", args[2]);
    println!("zrodlo NIETKNIETE: {}", args[0]);
}

/// Mnoznik dla kazdego symbolu: czestszy symbol dostaje glosniejszy wiersz w glowicy.
///
/// Log-czestosci sa najpierw sprowadzane do wspolnej skali (odjeta srednia, podzielone
/// przez odchylenie), zeby `sila` znaczyla to samo niezaleznie od korpusu. Bez tego ta
/// sama wartosc dawalaby zupelnie inny efekt na innym materiale.
fn row_scales(log_freq: &[f32], strength: f32) -> Vec<f32> {
    let n = log_freq.len() as f32;
    let mean = log_freq.iter().sum::<f32>() / n;
    let variance = log_freq.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    let sd = variance.sqrt().max(1e-6);
    log_freq
        .iter()
        .map(|v| ((strength * (v - mean) / sd).exp()).clamp(1.0 / MAX_SCALE, MAX_SCALE))
        .collect()
}

fn apply_row_scales(values: &mut [f32], scales: &[f32], hidden: usize) {
    for (row, scale) in scales.iter().enumerate() {
        let start = row * hidden;
        for v in &mut values[start..start + hidden] {
            *v *= scale;
        }
    }
}

/// To samo, co liczy `editor.rs` Noworodka w `TensorDiff` - zeby dalo sie porownac.
fn report(before: &[f32], after: &[f32], scales: &[f32], strength: f32) {
    let changed = before.iter().zip(after).filter(|(a, b)| a != b).count();
    let l1: f64 = before
        .iter()
        .zip(after)
        .map(|(a, b)| f64::from(b - a).abs())
        .sum();
    let l2: f64 = before
        .iter()
        .zip(after)
        .map(|(a, b)| f64::from(b - a).powi(2))
        .sum::<f64>()
        .sqrt();
    let max_abs = before
        .iter()
        .zip(after)
        .map(|(a, b)| (b - a).abs())
        .fold(0.0f32, f32::max);

    let min_scale = scales.iter().copied().fold(f32::MAX, f32::min);
    let max_scale = scales.iter().copied().fold(0.0f32, f32::max);

    println!("\nsila                : {strength}");
    println!("mnoznik od/do       : {min_scale:.4} .. {max_scale:.4}");
    println!("zmienionych liczb   : {changed} z {}", before.len());
    println!("suma zmian (l1)     : {l1:.3}");
    println!("dlugosc zmiany (l2) : {l2:.3}");
    println!("najwieksza zmiana   : {max_abs:.6}");
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

fn write_nwrd(path: &str, tensors: &[Tensor]) -> std::io::Result<()> {
    let mut w = BufWriter::new(File::create(path)?);
    w.write_all(MAGIC)?;
    w.write_all(&1u32.to_le_bytes())?;
    w.write_all(&(tensors.len() as u32).to_le_bytes())?;
    for t in tensors {
        let name = t.name.as_bytes();
        w.write_all(&(name.len() as u32).to_le_bytes())?;
        w.write_all(name)?;
        w.write_all(&(t.values.len() as u64).to_le_bytes())?;
        for v in &t.values {
            w.write_all(&v.to_le_bytes())?;
        }
    }
    w.flush()
}

fn read_log_frequencies(path: &str) -> std::io::Result<Vec<f32>> {
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match line.parse::<f32>() {
            Ok(v) if v.is_finite() => out.push(v),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("wiersz {} nie jest liczba: {line}", i + 1),
                ));
            }
        }
    }
    Ok(out)
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
    fn frequent_symbols_get_a_louder_row() {
        // Sedno narzedzia: symbol czestszy ma dostac wiekszy mnoznik. Gdyby kolejnosc
        // sie odwrocila, model uczylby sie odwrotnosci tego, co chcemy mu wpisac.
        let log_freq = [-1.0f32, -5.0, -3.0];
        let scales = row_scales(&log_freq, 0.5);
        assert!(scales[0] > scales[2], "czestszy musi byc glosniejszy");
        assert!(scales[2] > scales[1], "kolejnosc musi byc zachowana");
    }

    #[test]
    fn zero_strength_changes_nothing() {
        // Sprawdzian, ze samo przepisanie pliku niczego nie psuje - bez tego nie dalo by
        // sie odroznic bledu zapisu od skutku zmiany.
        let scales = row_scales(&[-1.0, -5.0, -3.0], 0.0);
        for s in scales {
            assert!((s - 1.0).abs() < 1e-6, "mnoznik przy sile 0 musi byc 1.0, jest {s}");
        }
    }

    #[test]
    fn no_multiplier_can_blow_the_weights_up() {
        // Ten test istnieje z powodu prawdziwej awarii: bez ograniczenia mnozniki doszly
        // do 239.9x i trening zwrocil `odlozone: inf`. Ksztalt danych jest tu skrajny -
        // wiekszosc slownika nie wystepuje w korpusie - wiec zabezpieczenie musi trzymac
        // niezaleznie od zadanej sily.
        let mut log_freq = vec![-14.1f32; 48434]; // nieobecne w korpusie
        log_freq.extend(std::iter::repeat_n(-3.0f32, 5312)); // obecne

        for strength in [0.1, 0.5, 2.0, 10.0] {
            let scales = row_scales(&log_freq, strength);
            let max = scales.iter().copied().fold(0.0f32, f32::max);
            let min = scales.iter().copied().fold(f32::MAX, f32::min);
            assert!(max <= MAX_SCALE, "sila {strength}: mnoznik {max} ponad limit");
            assert!(min >= 1.0 / MAX_SCALE, "sila {strength}: mnoznik {min} ponizej limitu");
            assert!(scales.iter().all(|s| s.is_finite()), "sila {strength}: mnoznik nieskonczony");
        }
    }

    #[test]
    fn scaling_touches_whole_rows_and_only_them() {
        let mut values = vec![1.0f32; 6]; // 3 symbole x 2 liczby
        apply_row_scales(&mut values, &[2.0, 1.0, 0.5], 2);
        assert_eq!(values, vec![2.0, 2.0, 1.0, 1.0, 0.5, 0.5]);
    }

    #[test]
    fn nwrd_round_trips_through_disk() {
        // Format zapisu jest kontraktem z Noworodkiem. Plik, ktorego on nie wczyta,
        // jest bezwartosciowy, a bledu nie zobaczylibysmy az do uruchomienia treningu.
        let dir = std::env::temp_dir().join(format!("darkstar-seed-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("katalog testowy");
        let path = dir.join("t.nwrd");
        let p = path.to_string_lossy().to_string();

        let original = vec![
            Tensor { name: "a.weight".into(), values: vec![1.0, -2.5, 3.25] },
            Tensor { name: "b.weight".into(), values: vec![0.0; 4] },
        ];
        write_nwrd(&p, &original).expect("zapis");
        let back = read_nwrd(&p).expect("odczyt");

        assert_eq!(back.len(), 2);
        assert_eq!(back[0].name, "a.weight");
        assert_eq!(back[0].values, vec![1.0, -2.5, 3.25]);
        assert_eq!(back[1].values.len(), 4);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn wrong_magic_is_refused() {
        let dir = std::env::temp_dir().join(format!("darkstar-seed-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("katalog");
        let path = dir.join("bad.nwrd");
        std::fs::write(&path, b"XXXX\x01\x00\x00\x00").expect("zapis");
        // `match` zamiast `expect_err`, bo to drugie wymagaloby wypisania calej zawartosci
        // wag w komunikacie bledu - a to dziesiatki megabajtow liczb.
        match read_nwrd(&path.to_string_lossy()) {
            Ok(_) => panic!("plik z zlym znacznikiem nie moze zostac przyjety"),
            Err(e) => assert!(e.to_string().contains("NWRD"), "komunikat: {e}"),
        }
        std::fs::remove_dir_all(&dir).ok();
    }
}
