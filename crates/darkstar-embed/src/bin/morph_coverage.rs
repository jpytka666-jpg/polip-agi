// THIS IS VERY IMPORTANT!!!
// ==========================================
// AUTHOR: M. SZUL
// AI MODEL: GPT-5 Codex
// TIMESTAMP: 2026-09-06  Europe/London
// REASON FOR CREATION: Read-only diagnostic census of verified Esperanto morphology
//   against the canonical CBMS Book/Vocabulary encoding.
// SYSTEM PART: Darkstar / experimental Esperanto morphology coverage.
// ARCHITECTURE FUNCTION: Classifies every unambiguous Book surface analysed by eo_morph
//   as one canonical CBMS symbol, multiple canonical symbols, literal-byte fallback, or
//   rejected. This binary does not alter Books, weights, or production tokenization.
// MECHANICS: CLI paths load a verified Esperanto surface list, eo_morf.dic/.aff and
// ksiega-max.txt. Vocabulary::encode is
//   classified from its public ID contract: symbol IDs are >= 272; literal byte IDs are
//   16..271; controls are < 16. Only unambiguous Analyzer results contribute support.
// DEPENDENCIES / LINKS: darkstar_embed::eo_morph; cbms-writing Book/Vocabulary.
// SAFETY / DO-NOT: No hardcoded machine paths, no network, no writes, no training, no
//   Book/NWRD mutation, no glyph bridge, no second ID space.
// REVISIONS: 2026-09-06 — initial diagnostic scanner.
// ==========================================

use std::{collections::BTreeMap, env, fs, process};

use cbms_writing::{Book, Vocabulary};
use darkstar_embed::eo_morph::{Analyzer, RejectionReason};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    A,
    B,
    C,
    D,
}

fn arg(name: &str) -> String {
    let mut it = env::args().skip(1);
    while let Some(k) = it.next() {
        if k == name {
            return it.next().unwrap_or_default();
        }
    }
    eprintln!("missing {name}");
    process::exit(2)
}

fn classify(ids: &[u32]) -> Class {
    if ids.is_empty() {
        return Class::D;
    }
    if ids.iter().any(|&id| (16..272).contains(&id)) {
        return Class::C;
    }
    if ids.iter().all(|&id| id >= 272) {
        return if ids.len() == 1 { Class::A } else { Class::B };
    }
    Class::D
}

fn main() {
    let book_text = fs::read_to_string(arg("--book")).expect("read --book");
    let (book, collisions) = Book::parse_lenient(&book_text).expect("parse --book");
    if !collisions.is_empty() {
        eprintln!("book collisions: {}", collisions.len());
    }
    let vocab = Vocabulary::new(&book).expect("build Vocabulary");
    let analyzer = Analyzer::from_files(arg("--dic"), arg("--aff")).expect("load eo_morf");
    let surfaces = fs::read_to_string(arg("--surfaces")).expect("read --surfaces");

    let mut total = 0usize;
    let mut analysed = 0usize;
    let mut rejected = 0usize;
    let mut no_analysis = 0usize;
    let mut ambiguous = 0usize;
    let mut counts = BTreeMap::<char, usize>::from([('A', 0), ('B', 0), ('C', 0), ('D', 0)]);
    let mut support = BTreeMap::<String, usize>::new();
    let mut examples = BTreeMap::<char, Vec<String>>::new();
    let mut required = BTreeMap::<String, String>::new();
    let mut cross = [[0usize; 4]; 3];
    let mut rejection_reasons = BTreeMap::<RejectionReason, usize>::new();
    let mut zero_path = [0usize; 2];
    let mut ambiguity_patterns = BTreeMap::<String, usize>::new();

    let mut seen = std::collections::BTreeSet::new();
    for raw in surfaces.lines() {
        let surface = raw
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();
        if surface.is_empty() || !seen.insert(surface.clone()) {
            continue;
        }
        total += 1;
        let analyses = analyzer.analyze(&surface);
        let diagnostics = analyzer.diagnose(&surface);
        for event in &diagnostics.rejections {
            *rejection_reasons.entry(event.reason).or_default() += 1;
        }
        if analyses.is_empty() {
            zero_path[usize::from(diagnostics.had_candidate_path)] += 1;
        }
        if analyses.len() > 1 {
            let roots = diagnostics
                .accepted
                .iter()
                .map(|a| {
                    a.analysis
                        .morphemes
                        .iter()
                        .find(|m| m.kind == darkstar_embed::eo_morph::Kind::Root)
                        .map(|m| m.text.clone())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>();
            let pattern = if roots
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 1
            {
                "MULTIPLE_ROOTS"
            } else {
                "SAME_ROOT_MULTIPLE_CHAINS"
            };
            *ambiguity_patterns.entry(pattern.into()).or_default() += 1;
        }
        let ids = vocab.encode(&surface);
        let class = classify(&ids);
        let ci = match class {
            Class::A => 0,
            Class::B => 1,
            Class::C => 2,
            Class::D => 3,
        };
        let ri = match analyses.len() {
            0 => 0,
            1 => 1,
            _ => 2,
        };
        cross[ri][ci] += 1;
        if analyses.len() != 1 {
            rejected += 1;
            if analyses.is_empty() {
                no_analysis += 1;
            } else {
                ambiguous += 1;
            }
            continue;
        }
        analysed += 1;
        let analysis = &analyses[0];
        let key = match class {
            Class::A => 'A',
            Class::B => 'B',
            Class::C => 'C',
            Class::D => 'D',
        };
        *counts.entry(key).or_default() += 1;
        if matches!(class, Class::A | Class::B) {
            for m in &analysis.morphemes {
                *support.entry(m.text.clone()).or_default() += 1;
            }
        }
        let item = format!(
            "{}={:?} ({})",
            surface,
            ids,
            analysis
                .morphemes
                .iter()
                .map(|m| m.text.as_str())
                .collect::<Vec<_>>()
                .join("+")
        );
        let list = examples.entry(key).or_default();
        if list.len() < 20 {
            list.push(item.clone());
        }
        if matches!(
            surface.as_str(),
            "memoro" | "memorilo" | "malsana" | "sanulo" | "malsanulejo" | "kodo" | "ĉifro"
        ) {
            required.insert(surface.clone(), item);
        }
    }

    let usable = counts[&'A'] + counts[&'B'];
    let mut supports: Vec<usize> = support.values().copied().collect();
    supports.sort_unstable();
    let median = supports.get(supports.len() / 2).copied().unwrap_or(0);
    let mean = if supports.is_empty() {
        0.0
    } else {
        supports.iter().sum::<usize>() as f64 / supports.len() as f64
    };
    println!("TOTAL_EO_FORMS\t{total}");
    println!("MORPH_ANALYZED\t{analysed}");
    println!("MORPH_REJECTED\t{rejected}");
    println!("REJECT_NO_ANALYSIS\t{no_analysis}");
    println!("REJECT_AMBIGUOUS\t{ambiguous}");
    println!("LEGACY_SINGLE_EXACT\t{}", counts[&'A']);
    println!("LEGACY_MULTI_CANONICAL\t{}", counts[&'B']);
    println!("LEGACY_WITH_LITERAL_FALLBACK\t{}", counts[&'C']);
    println!("OTHER_REJECTED\t{}", counts[&'D']);
    println!("USABLE_A_PLUS_B\t{usable}");
    for (name, row) in [
        ("ZERO", cross[0]),
        ("UNIQUE", cross[1]),
        ("AMBIGUOUS", cross[2]),
    ] {
        println!(
            "CROSSTAB_{name}\tA={} B={} C={} D={}",
            row[0], row[1], row[2], row[3]
        );
    }
    println!("UNIQUE_USABLE_MORPHEMES\t{}", support.len());
    println!("SUPPORT_MEDIAN\t{median}");
    println!("SUPPORT_MEAN\t{mean:.6}");
    for threshold in [2usize, 5, 10, 50] {
        println!(
            "SUPPORT_GE_{threshold}\t{}",
            support.values().filter(|&&n| n >= threshold).count()
        );
    }
    println!(
        "MORPHEME_SUPPORT\t{}",
        support
            .iter()
            .map(|(m, n)| format!("{m}:{n}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    for key in ['A', 'B', 'C'] {
        println!(
            "EXAMPLES_{key}\t{}",
            examples
                .get(&key)
                .map(|x| x.join(" | "))
                .unwrap_or_default()
        );
    }
    for word in [
        "memoro",
        "memorilo",
        "malsana",
        "sanulo",
        "malsanulejo",
        "kodo",
        "ĉifro",
    ] {
        let analyses = analyzer.analyze(word);
        println!(
            "REQUIRED_ANALYSIS\t{word}\tcount={}\t{}",
            analyses.len(),
            analyses
                .iter()
                .take(5)
                .map(|a| a
                    .morphemes
                    .iter()
                    .map(|m| m.text.as_str())
                    .collect::<Vec<_>>()
                    .join("+"))
                .collect::<Vec<_>>()
                .join(" | ")
        );
        println!(
            "REQUIRED\t{word}\t{}",
            required
                .get(word)
                .cloned()
                .unwrap_or_else(|| "NOT_IN_CANONICAL_BOOK".into())
        );
    }
    println!("ZERO_NO_CANDIDATE_PATH\t{}", zero_path[0]);
    println!("ZERO_CANDIDATES_ALL_REJECTED\t{}", zero_path[1]);
    println!(
        "REJECTION_REASONS\t{}",
        rejection_reasons
            .iter()
            .map(|(r, n)| format!("{r:?}:{n}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    println!(
        "AMBIGUITY_PATTERNS\t{}",
        ambiguity_patterns
            .iter()
            .map(|(r, n)| format!("{r}:{n}"))
            .collect::<Vec<_>>()
            .join(",")
    );
}
