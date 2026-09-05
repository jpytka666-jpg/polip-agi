// darkstar-header-v1
// po co: nwrd_diff.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-05
//! Pokazuje, CZEGO warstwy sie nauczyly - przez porownanie wag przed i po.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Sonnet 5
//! TIMESTAMP: 2026-09-05 16:00:00
//! REASON FOR CREATION: Wiemy, co wpisac w opisy znakow (pary) i w wyjscie (czestosci), bo
//! to da sie policzyc z korpusu. W warstwach srodkowych NIE WIEMY, co tam powinno byc - i to
//! jest ograniczenie pomyslu, nie narzedzia. Ale mozna sie dowiedziec przez obserwacje:
//! zrobic migawke, pouczyc, porownac i zobaczyc, czego warstwa nauczyla sie sama. Dopiero
//! wtedy edytowanie jej przestaje byc strzelaniem w ciemno.
//! MECHANICS: Czyta dwa pliki NWRD i dla kazdego wspolnego tensora liczy to samo, co
//! `editor.rs` Noworodka nazywa TensorDiff: ile liczb sie ruszylo, suma zmian, dlugosc
//! zmiany i najwieksza pojedyncza. Wypisuje posortowane wedlug wielkosci zmiany, bo pytanie
//! brzmi "ktora warstwa zrobila najwiecej", nie "co sie zmienilo w kolejnosci alfabetycznej".
//! SYSTEM PART: Darkstar / obserwacja wag.
//! ARCHITECTURE FUNCTION: Zamyka petle wpisywania: wpisz -> naucz -> ZOBACZ CO SIE ZMIENILO
//! -> wpisz lepiej. Bez tego kroku kazdy kolejny zasiew jest zgadywaniem.
//! DEPENDENCIES/LINKS: format NWRD, ten sam czytnik co darkstar-seed.
//! TECH STACK: Rust 2024, zero zaleznosci.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================
//!
//! Uzycie:
//! ```text
//! nwrd-diff <przed.nwrd> <po.nwrd>
//! ```

use std::{
    fs::File,
    io::{BufReader, Read},
};

const MAGIC: &[u8; 4] = b"NWRD";

struct Tensor {
    name: String,
    values: Vec<f32>,
}

struct Change {
    name: String,
    total: usize,
    changed: usize,
    l1: f64,
    l2: f64,
    max_abs: f32,
    /// Sredni ruch na zmieniona liczbe - mowi, czy warstwa drgnela, czy przebudowala sie.
    per_changed: f64,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("uzycie: nwrd-diff <przed.nwrd> <po.nwrd>");
        std::process::exit(2);
    }

    let before = match read_nwrd(&args[0]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: {}: {e}", args[0]);
            std::process::exit(1);
        }
    };
    let after = match read_nwrd(&args[1]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: {}: {e}", args[1]);
            std::process::exit(1);
        }
    };

    println!("przed: {} tensorow | po: {} tensorow", before.len(), after.len());

    // Tensory obecne tylko po jednej stronie sa wazna informacja, nie szumem: tak wyglada
    // warstwa dolozona albo usunieta.
    for t in &after {
        if !before.iter().any(|b| b.name == t.name) {
            println!("  NOWY : {} ({} liczb)", t.name, t.values.len());
        }
    }
    for t in &before {
        if !after.iter().any(|a| a.name == t.name) {
            println!("  ZNIKL: {} ({} liczb)", t.name, t.values.len());
        }
    }

    let mut changes = Vec::new();
    for b in &before {
        let Some(a) = after.iter().find(|t| t.name == b.name) else {
            continue;
        };
        if a.values.len() != b.values.len() {
            println!("  ROZMIAR SIE ZMIENIL: {} ({} -> {})", b.name, b.values.len(), a.values.len());
            continue;
        }
        changes.push(compare(&b.name, &b.values, &a.values));
    }

    changes.sort_by(|x, y| y.l2.partial_cmp(&x.l2).unwrap_or(std::cmp::Ordering::Equal));

    println!("\n{:<44} {:>9} {:>12} {:>12} {:>11}", "warstwa", "zmienione", "dlugosc", "sr.na liczbe", "najwieksza");
    println!("{}", "-".repeat(92));
    for c in &changes {
        let percent = if c.total > 0 {
            100.0 * c.changed as f64 / c.total as f64
        } else {
            0.0
        };
        println!(
            "{:<44} {:>7.1}% {:>12.4} {:>12.6} {:>11.6}",
            short_name(&c.name),
            percent,
            c.l2,
            c.per_changed,
            c.max_abs
        );
    }

    // Warstwa nietknieta to tez odpowiedz - znaczy, ze nauka nie miala jej po co ruszac.
    let untouched: Vec<&Change> = changes.iter().filter(|c| c.changed == 0).collect();
    if !untouched.is_empty() {
        println!("\nNIETKNIETE ({}):", untouched.len());
        for c in untouched {
            println!("  {}", short_name(&c.name));
        }
    }

    let total_l2: f64 = changes.iter().map(|c| c.l2 * c.l2).sum::<f64>().sqrt();
    println!("\nlaczna dlugosc zmiany: {total_l2:.4}");
}

/// To samo, co liczy `editor.rs` Noworodka w `TensorDiff`.
fn compare(name: &str, before: &[f32], after: &[f32]) -> Change {
    let mut changed = 0usize;
    let mut l1 = 0.0f64;
    let mut l2 = 0.0f64;
    let mut max_abs = 0.0f32;

    for (b, a) in before.iter().zip(after) {
        let d = a - b;
        if d != 0.0 {
            changed += 1;
        }
        let dd = f64::from(d);
        l1 += dd.abs();
        l2 += dd * dd;
        max_abs = max_abs.max(d.abs());
    }

    Change {
        name: name.to_string(),
        total: before.len(),
        changed,
        l1,
        l2: l2.sqrt(),
        max_abs,
        per_changed: if changed > 0 { l1 / changed as f64 } else { 0.0 },
    }
}

/// Skraca `model.layers.00.attention.q_proj.weight` do `L00.attn.q`, zeby tabela miescila
/// sie na ekranie. Pelne nazwy i tak sa w plikach.
fn short_name(name: &str) -> String {
    name.trim_start_matches("model.")
        .replace("layers.", "L")
        .replace(".weight", "")
        .replace("attention.", "attn.")
        .replace("_proj", "")
        .replace("embeddings.", "emb.")
}

fn read_nwrd(path: &str) -> std::io::Result<Vec<Tensor>> {
    let mut r = BufReader::new(File::open(path)?);
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "zly znacznik pliku, oczekiwano NWRD",
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
        let name_len = u32::from_le_bytes(b4) as usize;
        let mut name_bytes = vec![0u8; name_len];
        r.read_exact(&mut name_bytes)?;
        r.read_exact(&mut b8)?;
        let value_count = u64::from_le_bytes(b8) as usize;
        let mut values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            r.read_exact(&mut b4)?;
            values.push(f32::from_le_bytes(b4));
        }
        out.push(Tensor {
            name: String::from_utf8_lossy(&name_bytes).into_owned(),
            values,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_weights_show_no_change() {
        let v = vec![1.0f32, -2.0, 3.5];
        let c = compare("x", &v, &v);
        assert_eq!(c.changed, 0);
        assert_eq!(c.l2, 0.0);
        assert_eq!(c.per_changed, 0.0, "brak zmian nie moze dac dzielenia przez zero");
    }

    #[test]
    fn change_is_measured_where_it_happened() {
        let before = vec![0.0f32, 0.0, 0.0];
        let after = vec![0.0f32, 3.0, -4.0];
        let c = compare("x", &before, &after);
        assert_eq!(c.changed, 2, "dwie liczby sie ruszyly");
        assert_eq!(c.total, 3);
        assert!((c.l2 - 5.0).abs() < 1e-9, "dlugosc zmiany (3,4) to 5, jest {}", c.l2);
        assert!((c.max_abs - 4.0).abs() < 1e-6);
        assert!((c.per_changed - 3.5).abs() < 1e-9, "sredni ruch na zmieniona liczbe");
    }

    #[test]
    fn names_shorten_to_something_readable() {
        assert_eq!(
            short_name("model.layers.00.attention.q_proj.weight"),
            "L00.attn.q"
        );
        assert_eq!(short_name("model.embeddings.token.weight"), "emb.token");
        assert_eq!(short_name("model.lm_head.bias"), "lm_head.bias");
    }
}
