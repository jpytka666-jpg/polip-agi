// THIS IS VERY IMPORTANT!!!
// ==========================================
// AUTHOR: M. SZUL
// AI MODEL: GPT-5 Codex
// TIMESTAMP: 2026-09-06  Europe/London
// REASON FOR CREATION: Deterministic forward/reverse conformance audit of eo_morf Hunspell rules.
// SYSTEM PART: Darkstar / Esperanto morphology diagnostics.
// ARCHITECTURE FUNCTION: Exercises the existing eo_morph rule parser without changing production acceptance.
// MECHANICS: Generates bounded legal PFX/SFX chains and checks reverse provenance recovery.
// DEPENDENCIES / LINKS: eo_morf.dic/.aff supplied through CLI arguments.
// SAFETY / DO-NOT: No Book/CBMS/NWRD/weights/training/network; no hardcoded paths.
// REVISIONS: 2026-09-06 — initial conformance harness.
// ==========================================

use darkstar_embed::eo_morph::Analyzer;
use std::{
    env,
    io::{self, Write},
    process,
    process::Command,
    thread,
    time::{Duration, Instant},
};

fn arg(name: &str) -> String {
    let mut args = env::args().skip(1);
    while let Some(key) = args.next() {
        if key == name {
            return args.next().unwrap_or_default();
        }
    }
    eprintln!("missing {name}");
    process::exit(2);
}

#[derive(Clone)]
struct CaseRecord {
    index: usize,
    rule_chain: String,
    root: String,
    surface: String,
    expected: String,
    result: String,
    runtime_ms: u128,
    reverse_calls: usize,
    unique_states: usize,
    repeated_states: usize,
    max_repeat: usize,
    candidates: usize,
}

fn expected_for(case: &darkstar_embed::eo_morph::ForwardCase) -> String {
    case.morphemes
        .iter()
        .filter(|m| !m.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("|")
}

#[allow(clippy::too_many_arguments)]
fn execute_case(
    dic: &str,
    aff: &str,
    mode: &str,
    index: usize,
    surface: &str,
    expected: &str,
    root: &str,
    rules: &[usize],
) -> CaseRecord {
    let mut child = Command::new(env::current_exe().unwrap())
        .args([
            "--child",
            "--dic",
            dic,
            "--aff",
            aff,
            if mode == "single" {
                "--single-index"
            } else {
                "--chain-index"
            },
            &index.to_string(),
            "--surface",
            surface,
            "--expected",
            expected,
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn case");
    let start = Instant::now();
    loop {
        if child.try_wait().expect("wait case").is_some() {
            let mut out = String::new();
            if let Some(mut pipe) = child.stdout.take() {
                use std::io::Read;
                let _ = pipe.read_to_string(&mut out);
            }
            let f: Vec<&str> = out.trim().split('\t').collect();
            if f.len() >= 10 {
                return CaseRecord {
                    index,
                    rule_chain: rules
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    root: root.to_string(),
                    surface: surface.to_string(),
                    expected: expected.to_string(),
                    result: f[3].to_string(),
                    runtime_ms: f[4].parse().unwrap_or(start.elapsed().as_millis()),
                    reverse_calls: f[5].parse().unwrap_or(0),
                    unique_states: f[6].parse().unwrap_or(0),
                    repeated_states: f[7].parse().unwrap_or(0),
                    max_repeat: f[8].parse().unwrap_or(0),
                    candidates: f[9].parse().unwrap_or(0),
                };
            }
            return CaseRecord {
                index,
                rule_chain: rules
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                root: root.to_string(),
                surface: surface.to_string(),
                expected: expected.to_string(),
                result: "RECOVERY_FAIL".into(),
                runtime_ms: start.elapsed().as_millis(),
                reverse_calls: 0,
                unique_states: 0,
                repeated_states: 0,
                max_repeat: 0,
                candidates: 0,
            };
        }
        if start.elapsed() > Duration::from_millis(10_000) {
            let _ = child.kill();
            return CaseRecord {
                index,
                rule_chain: rules
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                root: root.to_string(),
                surface: surface.to_string(),
                expected: expected.to_string(),
                result: "BUDGET_EXCEEDED".into(),
                runtime_ms: start.elapsed().as_millis(),
                reverse_calls: 0,
                unique_states: 0,
                repeated_states: 0,
                max_repeat: 0,
                candidates: 0,
            };
        }
        thread::sleep(Duration::from_millis(2));
    }
}

fn main() {
    let dic = arg("--dic");
    let aff = arg("--aff");
    if env::args().any(|x| x == "--child") {
        let mut args = env::args().skip(1);
        let mut selected = None;
        while let Some(k) = args.next() {
            if k == "--single-index" || k == "--chain-index" {
                selected = args.next().and_then(|x| x.parse::<usize>().ok());
            }
        }
        let index = selected.unwrap_or(0);
        let analyzer = Analyzer::from_files(&dic, &aff).expect("load eo_morf");
        let surface = arg("--surface");
        let expected = arg("--expected");
        analyzer.reset_profile();
        let start = Instant::now();
        let ok = analyzer.analyze(&surface).iter().any(|analysis| {
            analysis
                .morphemes
                .iter()
                .map(|m| m.text.as_str())
                .eq(expected.split('|'))
        });
        let p = analyzer.profile();
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            index,
            if env::args().any(|x| x == "--chain-index") {
                "chain"
            } else {
                "single"
            },
            surface,
            if ok { "PASS" } else { "RECOVERY_FAIL" },
            start.elapsed().as_millis(),
            p.reverse_step_calls,
            p.unique_reverse_states,
            p.repeated_reverse_states,
            p.max_repeat_count,
            p.total_reverse_candidates
        );
        return;
    }
    let construct_start = Instant::now();
    let analyzer = Analyzer::from_files(&dic, &aff).expect("load eo_morf");
    let construct_ms = construct_start.elapsed().as_millis();
    let wall_start = Instant::now();
    let singles = analyzer.generate_single_forward_cases();
    let continuation_cases = analyzer.generate_continuation_cases(8, 64);
    let mut records = Vec::new();
    for (index, case) in singles.iter().enumerate() {
        analyzer.reset_profile();
        let start = Instant::now();
        let expected = expected_for(case);
        let analyses = analyzer.analyze(&case.surface);
        let ok = analyses.iter().any(|analysis| {
            analysis
                .morphemes
                .iter()
                .map(|m| m.text.as_str())
                .eq(expected.split('|'))
        });
        let profile = analyzer.profile();
        let record = CaseRecord {
            index,
            rule_chain: case
                .rule_ids
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(","),
            root: case.root.clone(),
            surface: case.surface.clone(),
            expected,
            result: if ok {
                "PASS".into()
            } else {
                "RECOVERY_FAIL".into()
            },
            runtime_ms: start.elapsed().as_millis(),
            reverse_calls: profile.reverse_step_calls,
            unique_states: profile.unique_reverse_states,
            repeated_states: profile.repeated_reverse_states,
            max_repeat: profile.max_repeat_count,
            candidates: profile.total_reverse_candidates,
        };
        if record.result != "PASS" {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                record.index,
                record.rule_chain,
                record.root,
                record.surface,
                record.expected,
                record.result,
                record.runtime_ms,
                record.reverse_calls,
                record.unique_states,
                record.repeated_states,
                record.max_repeat,
                record.candidates
            );
            io::stdout().flush().unwrap();
        }
        records.push(record);
    }
    let single_pass = records.iter().filter(|r| r.result == "PASS").count();
    let continuation_records: Vec<_> = if single_pass == singles.len() {
        continuation_cases
            .iter()
            .enumerate()
            .map(|(index, case)| {
                execute_case(
                    &dic,
                    &aff,
                    "chain",
                    index,
                    &case.surface,
                    &expected_for(case),
                    &case.root,
                    &case.rule_ids,
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    let single_budget = records
        .iter()
        .filter(|r| r.result == "BUDGET_EXCEEDED")
        .count();
    let single_fail = records.len() - single_pass - single_budget;
    for r in &records {
        if r.result != "PASS" {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                r.index,
                r.rule_chain,
                r.root,
                r.surface,
                r.expected,
                r.result,
                r.runtime_ms,
                r.reverse_calls,
                r.unique_states,
                r.repeated_states,
                r.max_repeat,
                r.candidates
            );
            io::stdout().flush().unwrap();
        }
    }
    println!(
        "TOTAL\t{}\nPASS\t{}\nRECOVERY_FAIL\t{}\nBUDGET_EXCEEDED\t{}",
        records.len(),
        single_pass,
        single_fail,
        single_budget
    );
    let continuation_pass = continuation_records
        .iter()
        .filter(|r| r.result == "PASS")
        .count();
    let continuation_budget = continuation_records
        .iter()
        .filter(|r| r.result == "BUDGET_EXCEEDED")
        .count();
    let continuation_fail = continuation_records.len() - continuation_pass - continuation_budget;
    println!("CONTINUATION_TOTAL\t{}\nCONTINUATION_PASS\t{}\nCONTINUATION_RECOVERY_FAIL\t{}\nCONTINUATION_BUDGET_EXCEEDED\t{}", continuation_records.len(), continuation_pass, continuation_fail, continuation_budget);
    println!("ANALYZER_CONSTRUCTION_MS\t{}", construct_ms);
    println!("SINGLE_WALL_MS\t{}", wall_start.elapsed().as_millis());
    for r in &continuation_records {
        if r.result != "PASS" {
            println!(
                "CHAIN\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                r.index,
                r.rule_chain,
                r.root,
                r.surface,
                r.expected,
                r.result,
                r.runtime_ms,
                r.reverse_calls,
                r.unique_states,
                r.repeated_states,
                r.max_repeat,
                r.candidates
            );
            io::stdout().flush().unwrap();
        }
    }
    println!("APPLICABLE_RULES\t{}", analyzer.applicable_rule_count());
    println!(
        "RULES_EXERCISED\t{}",
        singles
            .iter()
            .map(|c| c.rule_ids[0])
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
    println!("GENERATED_SINGLE_AFFIX_FORMS\t{}", singles.len());
    println!(
        "CONTINUATION_CHAINS_GENERATED\t{}",
        continuation_cases.len()
    );
    println!("RECOVERY_CHECKED\t{}", singles.len());
    println!("RECOVERED\t{single_pass}");
    println!("FAILED_OR_BUDGET\t{}", single_fail + single_budget);
    println!(
        "CROSS_PRODUCT_CASES\t{}",
        continuation_cases
            .iter()
            .filter(|c| c.rule_ids.len() > 1)
            .count()
    );
    println!(
        "NEEDAFFIX_REAL_ENTRIES\t{}",
        analyzer.need_affix_root_count()
    );
    println!("FALSE_ACCEPTS\t0_UNASSESSED");
    let profile = analyzer.profile();
    println!("REVERSE_STEP_CALLS\t{}", profile.reverse_step_calls);
    println!("UNIQUE_REVERSE_STATES\t{}", profile.unique_reverse_states);
    println!(
        "REPEATED_REVERSE_STATES\t{}",
        profile.repeated_reverse_states
    );
    println!("MAX_REPEAT_COUNT\t{}", profile.max_repeat_count);
    println!(
        "TOTAL_REVERSE_CANDIDATES\t{}",
        profile.total_reverse_candidates
    );
    println!(
        "AVG_REVERSE_CANDIDATES\t{:.3}",
        profile.total_reverse_candidates as f64 / profile.reverse_step_calls.max(1) as f64
    );
    println!("FORWARD_CHAIN_CALLS\t{}", profile.forward_chain_calls);
    println!(
        "TOTAL_FORWARD_CANDIDATES\t{}",
        profile.total_forward_candidates
    );
}
