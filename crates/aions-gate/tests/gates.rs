//! ================================================================================
//! AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER
//! ================================================================================
//! AUTHOR:              Claude Code agent, working for Marcin (jpytka666-jpg)
//! AI MODEL:            Anthropic Claude via Claude Code remote session (exact model id WITHHELD by harness policy)
//! TIMESTAMP:           2026-09-25T23:14:45Z
//! REASON FOR CREATION: SCCE rules 8 and 10: prove that the state machine really refuses forbidden transitions, rather than asserting that it does.
//! MECHANICS:           Builds temporary workspaces with small gate configs and injects faults (failing gate, UNKNOWN exit, undeclared transition, timeout, endless repair, forward rewind, bad config). Asserts the state never moves forward on anything but PASS and that evidence is written.
//! SYSTEM PART:         polip-agi / aions-gate / tests
//! ARCHITECTURE FUNC:   Verification plane for the workspace state machine.
//! DEPENDENCIES/LINKS:  aions_gate::gates; sh
//! TECH STACK:          Rust 2024 integration tests
//! LOCAL WORKSPACE:     polip-agi/crates/aions-gate/tests
//! GIT COMMIT:          PENDING
//! GITHUB METADATA:     jpytka666-jpg/polip-agi, branch claude/autonomous-automation-agent-ocxwke
//! ================================================================================

use aions_gate::gates::{Config, Outcome, STATES, Workspace};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static N: AtomicU32 = AtomicU32::new(0);

fn ws(conf: &str) -> Workspace {
    let dir: PathBuf = std::env::temp_dir().join(format!(
        "aions-gates-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("aions-gates.conf"), conf).unwrap();
    let w = Workspace::load(&dir).expect("config");
    w.init("t1").unwrap();
    w
}

fn state(w: &Workspace) -> String {
    w.read_state().unwrap().state
}

#[test]
fn pass_advances_exactly_one_state() {
    let w = ws("gate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED ok = true\n");
    let r = w.advance().unwrap();
    assert_eq!(r.outcome, Outcome::Pass);
    assert_eq!(state(&w), "SOURCE_OF_TRUTH_LOADED");
}

#[test]
fn fail_keeps_state_and_records_evidence() {
    let w = ws("gate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED bad = echo broken; exit 1\n");
    let r = w.advance().unwrap();
    assert_eq!(r.outcome, Outcome::Fail);
    assert_eq!(state(&w), "PRE_FLIGHT");
    let ev = fs::read_to_string(w.root.join(".aions/evidence.jsonl")).unwrap();
    assert!(
        ev.contains("\"verdict\":\"FAIL\"") && ev.contains("broken"),
        "{ev}"
    );
}

#[test]
fn one_failing_gate_among_passing_blocks() {
    let w = ws(
        "gate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED a = true\ngate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED b = false\ngate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED c = true\n",
    );
    assert_eq!(w.advance().unwrap().outcome, Outcome::Fail);
    assert_eq!(state(&w), "PRE_FLIGHT");
}

#[test]
fn unknown_is_not_pass() {
    let w = ws("gate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED u = exit 3\n");
    assert_eq!(w.advance().unwrap().outcome, Outcome::Unknown);
    assert_eq!(state(&w), "PRE_FLIGHT");
}

#[test]
fn undeclared_transition_is_unknown() {
    let w = ws("# nothing declared\n");
    let r = w.advance().unwrap();
    assert_eq!(r.outcome, Outcome::Unknown);
    assert!(r.reason.contains("UNKNOWN is not PASS"));
    assert_eq!(state(&w), "PRE_FLIGHT");
}

#[test]
fn declared_open_transition_passes_with_reason() {
    let w = ws("open PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED judgment only\n");
    let r = w.advance().unwrap();
    assert_eq!(r.outcome, Outcome::Pass);
    assert!(r.reason.contains("judgment only"));
}

#[test]
fn timeout_is_unknown() {
    let w = ws("timeout 1\ngate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED slow = sleep 5\n");
    let start = std::time::Instant::now();
    assert_eq!(w.advance().unwrap().outcome, Outcome::Unknown);
    assert!(start.elapsed().as_secs() < 4, "timeout not enforced");
}

#[test]
fn repair_then_reverify_succeeds() {
    let w =
        ws("gate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED f = test -f fixed\nrepair f = touch fixed\n");
    let r = w.advance().unwrap();
    assert_eq!(r.outcome, Outcome::Pass);
    let names: Vec<_> = r
        .runs
        .iter()
        .map(|g| (g.gate.as_str(), g.outcome))
        .collect();
    assert_eq!(
        names,
        vec![
            ("f", Outcome::Fail),
            ("repair:f", Outcome::Pass),
            ("f", Outcome::Pass)
        ]
    );
}

#[test]
fn repair_loop_is_bounded_even_if_config_asks_for_more() {
    let w =
        ws("max-repair 99\ngate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED f = false\nrepair f = true\n");
    let r = w.advance().unwrap();
    assert_eq!(r.outcome, Outcome::Fail);
    let gate_runs = r.runs.iter().filter(|g| g.gate == "f").count();
    assert_eq!(gate_runs as u32, aions_gate::gates::MAX_REPAIR_CAP + 1);
}

#[test]
fn full_walk_reaches_complete_only_through_gates() {
    let mut conf = String::new();
    for pair in STATES.windows(2) {
        conf.push_str(&format!(
            "gate {}->{} g{} = true\n",
            pair[0], pair[1], pair[1]
        ));
    }
    let w = ws(&conf);
    for _ in 1..STATES.len() {
        assert_eq!(w.advance().unwrap().outcome, Outcome::Pass);
    }
    assert_eq!(state(&w), "COMPLETE");
    assert!(w.advance().is_err(), "nothing beyond COMPLETE");
}

#[test]
fn rewind_only_goes_backwards() {
    let w = ws("open PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED x\n");
    w.advance().unwrap();
    assert!(w.rewind("COMPLETE").is_err());
    assert!(w.rewind("SOURCE_OF_TRUTH_LOADED").is_err());
    assert_eq!(w.rewind("PRE_FLIGHT").unwrap().state, "PRE_FLIGHT");
}

#[test]
fn init_is_idempotent_and_refuses_to_clobber() {
    let w = ws("open PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED x\n");
    w.advance().unwrap();
    assert_eq!(w.init("t1").unwrap().state, "SOURCE_OF_TRUTH_LOADED");
    assert!(w.init("other").is_err());
}

#[test]
fn config_rejects_skips_and_contradictions() {
    assert!(Config::parse("gate PRE_FLIGHT->COMPLETE skip = true\n").is_err());
    assert!(Config::parse("gate TEST->MUTATION back = true\n").is_err());
    assert!(Config::parse("repair ghost = true\n").is_err());
    assert!(Config::parse("frobnicate\n").is_err());
    assert!(Config::parse("gate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED a = true\ngate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED a = true\n").is_err());
    assert!(Config::parse("open PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED x\ngate PRE_FLIGHT->SOURCE_OF_TRUTH_LOADED a = true\n").is_err());
}

#[test]
fn repo_config_parses() {
    let text = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../aions-gates.conf"
    ))
    .unwrap();
    let cfg = Config::parse(&text).expect("repo aions-gates.conf must stay valid");
    for pair in STATES.windows(2) {
        let t = format!("{}->{}", pair[0], pair[1]);
        assert!(
            cfg.gates.iter().any(|g| g.transition == t) || cfg.open.iter().any(|(o, _)| *o == t),
            "{t} is neither gated nor declared open"
        );
    }
}
