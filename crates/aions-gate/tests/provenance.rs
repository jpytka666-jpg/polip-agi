//! ================================================================================
//! AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER
//! ================================================================================
//! AUTHOR:              Claude Code agent, working for Marcin (jpytka666-jpg)
//! AI MODEL:            Anthropic Claude via Claude Code remote session (exact model id WITHHELD by harness policy)
//! TIMESTAMP:           2026-09-25T23:14:45Z
//! REASON FOR CREATION: SCCE rules 8 and 10: the provenance validator must be shown to detect each defect it claims to guard against.
//! MECHANICS:           Starts from a known-good header, then injects one fault per test (missing field, placeholder, bad timestamp, fake commit, misplaced header, rewritten creation fields) and asserts the exact verdict. Also runs the real binary against a temporary git repo.
//! SYSTEM PART:         polip-agi / aions-gate / tests
//! ARCHITECTURE FUNC:   Verification plane for the provenance validator.
//! DEPENDENCIES/LINKS:  aions_gate::provenance; git CLI; target binary aions-gate
//! TECH STACK:          Rust 2024 integration tests
//! LOCAL WORKSPACE:     polip-agi/crates/aions-gate/tests
//! GIT COMMIT:          PENDING
//! GITHUB METADATA:     jpytka666-jpg/polip-agi, branch claude/autonomous-automation-agent-ocxwke
//! ================================================================================

use aions_gate::provenance::{self, Policy, Verdict};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

fn good(prefix: &str) -> String {
    [
        "================================================================================",
        "AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER",
        "================================================================================",
        "AUTHOR:              Test",
        "AI MODEL:            UNKNOWN",
        "TIMESTAMP:           2026-09-25T23:14:45+02:00",
        "REASON FOR CREATION: test",
        "MECHANICS:           test",
        "SYSTEM PART:         test",
        "ARCHITECTURE FUNC:   test",
        "DEPENDENCIES/LINKS:  none",
        "TECH STACK:          sh",
        "LOCAL WORKSPACE:     tmp",
        "GIT COMMIT:          PENDING",
        "GITHUB METADATA:     N/A - test fixture",
        "================================================================================",
    ]
    .iter()
    .map(|l| format!("{prefix} {l}\n"))
    .collect()
}

fn fails_with(content: &str, needle: &str) {
    match provenance::validate_content(content, None) {
        Verdict::Fail(r) => assert!(
            r.iter().any(|x| x.contains(needle)),
            "expected '{needle}' in {r:?}"
        ),
        other => panic!("expected FAIL containing '{needle}', got {other:?}"),
    }
}

#[test]
fn good_header_passes_in_several_syntaxes() {
    assert_eq!(
        provenance::validate_content(&good("//!"), None),
        Verdict::Pass
    );
    assert_eq!(
        provenance::validate_content(&format!("#!/bin/sh\n{}echo hi\n", good("#")), None),
        Verdict::Pass
    );
    let md = format!("---\ntitle: x\n---\n<!--\n{}-->\n# Doc\n", good(""));
    assert_eq!(provenance::validate_content(&md, None), Verdict::Pass);
}

#[test]
fn no_header_is_brak_headera() {
    assert_eq!(
        provenance::validate_content("fn main() {}\n", None),
        Verdict::BrakHeadera
    );
}

#[test]
fn each_missing_field_is_detected() {
    for field in provenance::FIELDS {
        let mutated: String = good("#")
            .lines()
            .filter(|l| !l.trim_start_matches("# ").starts_with(&format!("{field}:")))
            .map(|l| format!("{l}\n"))
            .collect();
        fails_with(&mutated, &format!("missing field {field}"));
    }
}

#[test]
fn template_placeholder_is_detected() {
    let m = good("#").replace(
        "AUTHOR:              Test",
        "AUTHOR:              [Human or Agent Identity]",
    );
    fails_with(&m, "placeholder left in AUTHOR");
}

#[test]
fn timestamp_without_timezone_is_detected() {
    let m = good("#").replace("2026-09-25T23:14:45+02:00", "2026-09-25 23:14:45");
    fails_with(&m, "not ISO-8601 with timezone");
}

#[test]
fn timestamp_formats() {
    for ok in [
        "2026-09-25T23:14Z",
        "2026-09-25T23:14:45Z",
        "2026-09-25T23:14:45.123-05:30",
    ] {
        assert!(provenance::is_iso8601_with_tz(ok), "{ok}");
    }
    for bad in [
        "2026-09-25",
        "2026-09-25T23:14:45",
        "2026-09-25T23:14:45+0200",
        "yesterday",
    ] {
        assert!(!provenance::is_iso8601_with_tz(bad), "{bad}");
    }
}

#[test]
fn free_text_commit_is_detected() {
    let m = good("#").replace(
        "GIT COMMIT:          PENDING",
        "GIT COMMIT:          latest",
    );
    fails_with(&m, "is not a hash");
}

#[test]
fn header_below_content_is_detected() {
    let m = format!("//! Some module docs first.\n{}", good("//!"));
    fails_with(&m, "not at earliest valid location");
}

#[test]
fn legacy_alias_is_accepted() {
    let m = good("#").replace("ARCHITECTURE FUNC:  ", "ARCHITECTURE FUNCTION:");
    assert_eq!(provenance::validate_content(&m, None), Verdict::Pass);
}

#[test]
fn rewritten_creation_provenance_is_detected() {
    let before = good("#");
    let after = before.replace(
        "AUTHOR:              Test",
        "AUTHOR:              Someone Else",
    );
    let p = provenance::creation_provenance_problems(&before, &after);
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(p[0].contains("AUTHOR rewritten"));
    assert_eq!(
        provenance::creation_provenance_problems(&before, "no header\n").len(),
        3
    );
    assert!(provenance::creation_provenance_problems(&before, &before).is_empty());
}

#[test]
fn policy_classification() {
    let none: &[(String, String)] = &[];
    assert_eq!(
        provenance::policy_for("src/a.rs", Some(""), none),
        Policy::Required
    );
    assert_eq!(
        provenance::policy_for("run", Some("#!/bin/sh\n"), none),
        Policy::Required
    );
    assert!(matches!(
        provenance::policy_for("Cargo.lock", Some(""), none),
        Policy::Excluded(_)
    ));
    assert!(matches!(
        provenance::policy_for("LICENSE", Some(""), none),
        Policy::Excluded(_)
    ));
    assert!(matches!(
        provenance::policy_for("a/b.json", Some("{}"), none),
        Policy::Excluded(_)
    ));
    assert!(matches!(
        provenance::policy_for("x.png", None, none),
        Policy::Excluded(_)
    ));
    assert!(matches!(
        provenance::policy_for("vendor/x.rs", Some(""), none),
        Policy::Excluded(_)
    ));
    assert!(matches!(
        provenance::policy_for("gen.rs", Some("// @generated\n"), none),
        Policy::Excluded(_)
    ));
    assert!(matches!(
        provenance::policy_for("data.xyz", Some(""), none),
        Policy::Unknown(_)
    ));
    let ex = vec![("docs/".to_string(), "external".to_string())];
    assert!(matches!(
        provenance::policy_for("docs/a.md", Some(""), &ex),
        Policy::Excluded(_)
    ));
}

// ---- real git + real binary -------------------------------------------------

static N: AtomicU32 = AtomicU32::new(0);

fn tmp_repo() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "aions-prov-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for args in [
        vec!["init", "-q", "-b", "main"],
        vec!["config", "user.email", "t@t"],
        vec!["config", "user.name", "t"],
        vec!["config", "commit.gpgsign", "false"],
    ] {
        git(&dir, &args);
    }
    dir
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn bin(dir: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aions-gate"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn commit_claims_are_checked_against_git() {
    let dir = tmp_repo();
    fs::write(dir.join("a.sh"), good("#")).unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-qm", "init"]);
    let real = git(&dir, &["rev-parse", "HEAD"]);
    let ok = good("#").replace(
        "GIT COMMIT:          PENDING",
        &format!("GIT COMMIT:          {real}"),
    );
    assert_eq!(provenance::validate_content(&ok, Some(&dir)), Verdict::Pass);
    let fake = good("#").replace(
        "GIT COMMIT:          PENDING",
        "GIT COMMIT:          deadbeefdeadbeef",
    );
    match provenance::validate_content(&fake, Some(&dir)) {
        Verdict::Fail(r) => assert!(r[0].contains("does not exist")),
        v => panic!("{v:?}"),
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn binary_since_mode_blocks_new_file_without_header_and_reports_legacy() {
    let dir = tmp_repo();
    fs::write(dir.join("legacy.sh"), "echo old\n").unwrap();
    fs::write(dir.join("kept.sh"), good("#")).unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-qm", "base"]);

    // Legacy edit without header: reported, not blocking.
    fs::write(dir.join("legacy.sh"), "echo new\n").unwrap();
    let (code, out) = bin(&dir, &["provenance", "--since", "main"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("BRAK_HEADERA") && out.contains("not blocking"),
        "{out}"
    );

    // New file without header: blocking.
    fs::write(dir.join("new.rs"), "fn main() {}\n").unwrap();
    git(&dir, &["add", "new.rs"]);
    let (code, out) = bin(&dir, &["provenance", "--since", "main"]);
    assert_eq!(code, 1, "{out}");

    // Staged mode checks the index, not the working tree.
    fs::write(
        dir.join("new.rs"),
        format!("{}fn main() {{}}\n", good("//!")),
    )
    .unwrap();
    let (code, _) = bin(&dir, &["provenance", "--staged"]);
    assert_eq!(
        code, 1,
        "working-tree fix must not satisfy the staged check"
    );
    git(&dir, &["add", "new.rs"]);
    let (code, out) = bin(&dir, &["provenance", "--staged"]);
    assert_eq!(code, 0, "{out}");

    // Destroying creation provenance on an existing file: blocking.
    fs::write(
        dir.join("kept.sh"),
        good("#").replace("AUTHOR:              Test", "AUTHOR:              Mallory"),
    )
    .unwrap();
    let (code, out) = bin(&dir, &["provenance", "--json", "--since", "main"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("\"verdict\":\"FAIL\"") && out.contains("AUTHOR rewritten"),
        "{out}"
    );
    fs::remove_dir_all(dir).unwrap();
}
