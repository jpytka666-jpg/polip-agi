//! ================================================================================
//! AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER
//! ================================================================================
//! AUTHOR:              Claude Code agent, working for Marcin (jpytka666-jpg)
//! AI MODEL:            Anthropic Claude via Claude Code remote session (exact model id WITHHELD by harness policy)
//! TIMESTAMP:           2026-09-25T23:14:45Z
//! REASON FOR CREATION: Give humans, agents, git hooks and CI one deterministic, non-interactive command for gate decisions (SCCE rule 18).
//! MECHANICS:           CLI over the library: `provenance` (file headers), `state` (init/show/rewind), `advance` (gated transition), `check-config`. Exit codes: 0 PASS, 1 FAIL, 2 usage/config error, 3 UNKNOWN.
//! SYSTEM PART:         polip-agi / aions-gate / CLI
//! ARCHITECTURE FUNC:   Boundary between the reasoning plane (whoever calls the command) and the enforcement plane (the verdict it returns).
//! DEPENDENCIES/LINKS:  aions_gate::{gates, provenance, json}; git CLI; hooks/pre-commit; .github/workflows/aions-gate.yml
//! TECH STACK:          Rust 2024, std only
//! LOCAL WORKSPACE:     polip-agi/crates/aions-gate
//! GIT COMMIT:          PENDING
//! GITHUB METADATA:     jpytka666-jpg/polip-agi, branch claude/autonomous-automation-agent-ocxwke
//! ================================================================================

use aions_gate::gates::{self, Outcome, Workspace};
use aions_gate::json;
use aions_gate::provenance::{self, Verdict};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "usage:
  aions-gate provenance [--json] [--since REF | --staged | PATH...]
  aions-gate state init <task> | show | rewind <STATE>
  aions-gate advance [--json]
  aions-gate check-config
exit: 0 PASS, 1 FAIL, 2 usage/config error, 3 UNKNOWN";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("provenance") => cmd_provenance(&args[1..]),
        Some("state") => cmd_state(&args[1..]),
        Some("advance") => cmd_advance(&args[1..]),
        Some("check-config") => with_workspace(|ws| {
            println!(
                "config OK: {} gates, {} open transitions, {} repairs, max-repair {}, timeout {:?}",
                ws.config.gates.len(),
                ws.config.open.len(),
                ws.config.repairs.len(),
                ws.config.max_repair,
                ws.config.timeout
            );
            0
        }),
        _ => {
            eprintln!("{USAGE}");
            2
        }
    };
    ExitCode::from(code)
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("git unavailable: {e}"))?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn git_lines(root: &Path, args: &[&str]) -> Result<Vec<String>, String> {
    Ok(String::from_utf8_lossy(&git(root, args)?)
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

fn repo_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    git(&cwd, &["rev-parse", "--show-toplevel"])
        .ok()
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o).trim()))
        .unwrap_or(cwd)
}

fn with_workspace(f: impl FnOnce(&Workspace) -> u8) -> u8 {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let Some(root) = gates::find_root(&cwd) else {
        eprintln!(
            "aions-gate: no {} found in {} or its parents",
            gates::CONFIG_NAME,
            cwd.display()
        );
        return 2;
    };
    match Workspace::load(&root) {
        Ok(ws) => f(&ws),
        Err(e) => {
            eprintln!("aions-gate: config error: {}", e.0);
            2
        }
    }
}

struct Entry {
    path: String,
    verdict: Verdict,
    enforced: bool,
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut items: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    items.sort();
    for p in items {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name == ".git" || name == "target" || name == ".aions" {
            continue;
        }
        if p.is_dir() {
            walk(root, &p, out);
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn cmd_provenance(args: &[String]) -> u8 {
    let root = repo_root();
    let excludes = gates::find_root(&root)
        .and_then(|r| Workspace::load(&r).ok())
        .map(|w| w.config.excludes)
        .unwrap_or_default();
    let as_json = args.iter().any(|a| a == "--json");
    let rest: Vec<&String> = args.iter().filter(|a| *a != "--json").collect();
    let mut entries = Vec::new();

    let result: Result<(), String> = (|| {
        match rest.first().map(|s| s.as_str()) {
            Some("--since") | Some("--staged") => {
                let staged = rest[0] == "--staged";
                let base: Vec<&str> = if staged {
                    vec!["diff", "--cached", "--name-only"]
                } else {
                    let r = rest.get(1).ok_or("--since needs a REF")?;
                    vec!["diff", "--name-only", r.as_str()]
                };
                let with = |filter: &str| {
                    let mut a = base.clone();
                    a.push(filter);
                    git_lines(&root, &a)
                };
                let before_ref = if staged {
                    "HEAD".to_string()
                } else {
                    rest[1].to_string()
                };
                let read_now = |p: &str| -> Option<Vec<u8>> {
                    if staged {
                        git(&root, &["show", &format!(":{p}")]).ok()
                    } else {
                        fs::read(root.join(p)).ok()
                    }
                };
                for p in with("--diff-filter=A")? {
                    let verdict = match read_now(&p) {
                        Some(b) => provenance::check_bytes(&root, &p, b, &excludes),
                        None => Verdict::Unknown("cannot read new content".into()),
                    };
                    entries.push(Entry {
                        path: p,
                        verdict,
                        enforced: true,
                    });
                }
                for p in with("--diff-filter=M")? {
                    let now = read_now(&p)
                        .map(|b| String::from_utf8_lossy(&b).into_owned())
                        .unwrap_or_default();
                    let before = git(&root, &["show", &format!("{before_ref}:{p}")])
                        .map(|b| String::from_utf8_lossy(&b).into_owned())
                        .unwrap_or_default();
                    let problems = provenance::creation_provenance_problems(&before, &now);
                    let verdict = if !problems.is_empty() {
                        Verdict::Fail(problems)
                    } else {
                        match provenance::policy_for(&p, Some(&now), &excludes) {
                            provenance::Policy::Required
                                if provenance::parse_header(&now).is_none() =>
                            {
                                Verdict::BrakHeadera
                            }
                            provenance::Policy::Excluded(r) => Verdict::Excluded(r),
                            _ => Verdict::Pass,
                        }
                    };
                    // Legacy files without a header are reported, not blocked; destroyed provenance is blocked.
                    let enforced = !matches!(verdict, Verdict::BrakHeadera);
                    entries.push(Entry {
                        path: p,
                        verdict,
                        enforced,
                    });
                }
            }
            None => {
                for p in git_lines(&root, &["ls-files"])? {
                    let verdict = provenance::check_file(&root, &p, &excludes);
                    entries.push(Entry {
                        path: p,
                        verdict,
                        enforced: true,
                    });
                }
            }
            Some(_) => {
                let mut files = Vec::new();
                for arg in &rest {
                    let abs = std::env::current_dir().unwrap_or_default().join(arg);
                    let abs = abs.canonicalize().map_err(|e| format!("{arg}: {e}"))?;
                    if abs.is_dir() {
                        walk(&root, &abs, &mut files);
                    } else {
                        let rel = abs
                            .strip_prefix(&root)
                            .map_err(|_| format!("{arg} is outside {}", root.display()))?;
                        files.push(rel.to_string_lossy().replace('\\', "/"));
                    }
                }
                for p in files {
                    let verdict = provenance::check_file(&root, &p, &excludes);
                    entries.push(Entry {
                        path: p,
                        verdict,
                        enforced: true,
                    });
                }
            }
        }
        Ok(())
    })();

    if let Err(e) = result {
        eprintln!("aions-gate: {e}");
        return 3;
    }

    let count = |l: &str| entries.iter().filter(|e| e.verdict.label() == l).count();
    let blocking_fail = entries
        .iter()
        .any(|e| e.enforced && matches!(e.verdict, Verdict::Fail(_) | Verdict::BrakHeadera));
    let blocking_unknown = entries
        .iter()
        .any(|e| e.enforced && matches!(e.verdict, Verdict::Unknown(_)));
    let overall = if blocking_fail {
        Outcome::Fail
    } else if blocking_unknown {
        Outcome::Unknown
    } else {
        Outcome::Pass
    };

    if as_json {
        let items: Vec<String> = entries
            .iter()
            .map(|e| {
                format!(
                    "{{\"path\":{},\"verdict\":{},\"enforced\":{},\"detail\":{}}}",
                    json::string(&e.path),
                    json::string(e.verdict.label()),
                    e.enforced,
                    json::string(&e.verdict.detail())
                )
            })
            .collect();
        println!(
            "{{\"check\":\"provenance\",\"verdict\":{},\"files\":[{}]}}",
            json::string(overall.label()),
            items.join(",")
        );
    } else {
        for e in &entries {
            let note = if e.enforced {
                ""
            } else {
                " (reported, not blocking)"
            };
            println!(
                "{:<12} {}{}{}",
                e.verdict.label(),
                e.path,
                if e.verdict.detail().is_empty() {
                    String::new()
                } else {
                    format!(" :: {}", e.verdict.detail())
                },
                note
            );
        }
        println!(
            "provenance: {} PASS, {} FAIL, {} BRAK_HEADERA, {} UNKNOWN, {} EXCLUDED => {}",
            count("PASS"),
            count("FAIL"),
            count("BRAK_HEADERA"),
            count("UNKNOWN"),
            count("EXCLUDED"),
            overall.label()
        );
    }
    exit_for(overall)
}

fn exit_for(o: Outcome) -> u8 {
    match o {
        Outcome::Pass => 0,
        Outcome::Fail => 1,
        Outcome::Unknown => 3,
    }
}

fn cmd_state(args: &[String]) -> u8 {
    with_workspace(|ws| match (args.first().map(String::as_str), args.get(1)) {
        (Some("init"), Some(task)) => match ws.init(task) {
            Ok(st) => {
                println!("task={} state={}", st.task, st.state);
                0
            }
            Err(e) => {
                eprintln!("aions-gate: {e}");
                1
            }
        },
        (Some("show"), None) => match ws.read_state() {
            Some(st) => {
                let i = gates::index_of(&st.state).unwrap_or(0);
                let next = gates::STATES.get(i + 1).copied().unwrap_or("-");
                println!("task={} state={} next={}", st.task, st.state, next);
                0
            }
            None => {
                println!("state=NONE");
                3
            }
        },
        (Some("rewind"), Some(target)) => match ws.rewind(target) {
            Ok(st) => {
                println!("task={} state={}", st.task, st.state);
                0
            }
            Err(e) => {
                eprintln!("aions-gate: {e}");
                1
            }
        },
        _ => {
            eprintln!("{USAGE}");
            2
        }
    })
}

fn cmd_advance(args: &[String]) -> u8 {
    let as_json = args.iter().any(|a| a == "--json");
    with_workspace(|ws| match ws.advance() {
        Err(e) => {
            eprintln!("aions-gate: {e}");
            2
        }
        Ok(r) => {
            if as_json {
                let runs: Vec<String> = r
                    .runs
                    .iter()
                    .map(|g| {
                        format!(
                            "{{\"gate\":{},\"attempt\":{},\"verdict\":{}}}",
                            json::string(&g.gate),
                            g.attempt,
                            json::string(g.outcome.label())
                        )
                    })
                    .collect();
                println!(
                    "{{\"from\":{},\"to\":{},\"verdict\":{},\"reason\":{},\"runs\":[{}]}}",
                    json::string(&r.from),
                    json::string(&r.to),
                    json::string(r.outcome.label()),
                    json::string(&r.reason),
                    runs.join(",")
                );
            } else {
                for g in &r.runs {
                    println!(
                        "  {:<8} {} (attempt {})",
                        g.outcome.label(),
                        g.gate,
                        g.attempt
                    );
                    if g.outcome != Outcome::Pass && !g.output_tail.trim().is_empty() {
                        for line in g
                            .output_tail
                            .lines()
                            .rev()
                            .take(10)
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                        {
                            println!("           | {line}");
                        }
                    }
                }
                let state = if r.outcome == Outcome::Pass {
                    &r.to
                } else {
                    &r.from
                };
                println!(
                    "{} {} -> {} :: state={} {}",
                    r.outcome.label(),
                    r.from,
                    r.to,
                    state,
                    r.reason
                );
            }
            exit_for(r.outcome)
        }
    })
}
