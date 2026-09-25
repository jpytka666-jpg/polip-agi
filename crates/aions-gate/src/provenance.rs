//! ================================================================================
//! AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER
//! ================================================================================
//! AUTHOR:              Claude Code agent, working for Marcin (jpytka666-jpg)
//! AI MODEL:            Anthropic Claude via Claude Code remote session (exact model id WITHHELD by harness policy)
//! TIMESTAMP:           2026-09-25T23:14:45Z
//! REASON FOR CREATION: AIONS master protocol deliverable 5: provenance headers were enforced only by prose and model compliance.
//! MECHANICS:           Classifies a file (required / excluded / unknown policy), skips the structural preamble, locates the header, checks all 12 fields, the TIMESTAMP format, the GIT COMMIT claim against git, and whether creation provenance survived a modification.
//! SYSTEM PART:         polip-agi / aions-gate / provenance validator
//! ARCHITECTURE FUNC:   Enforcement plane: a deterministic verdict (PASS/FAIL/BRAK_HEADERA/EXCLUDED/UNKNOWN) that replaces model self-assessment of header compliance.
//! DEPENDENCIES/LINKS:  std only; git CLI (commit verification, history lookup); crate::json; tests/provenance.rs
//! TECH STACK:          Rust 2024, std only
//! LOCAL WORKSPACE:     polip-agi/crates/aions-gate
//! GIT COMMIT:          PENDING
//! GITHUB METADATA:     jpytka666-jpg/polip-agi, branch claude/autonomous-automation-agent-ocxwke
//! ================================================================================

use std::fs;
use std::path::Path;
use std::process::Command;

/// The 12 mandatory header fields, in template order.
pub const FIELDS: [&str; 12] = [
    "AUTHOR",
    "AI MODEL",
    "TIMESTAMP",
    "REASON FOR CREATION",
    "MECHANICS",
    "SYSTEM PART",
    "ARCHITECTURE FUNC",
    "DEPENDENCIES/LINKS",
    "TECH STACK",
    "LOCAL WORKSPACE",
    "GIT COMMIT",
    "GITHUB METADATA",
];

/// Historical spellings accepted so legacy headers are not misreported.
const ALIASES: [(&str, &str); 1] = [("ARCHITECTURE FUNCTION", "ARCHITECTURE FUNC")];

/// Fields that identify creation and must survive later edits.
const CREATION_FIELDS: [&str; 3] = ["AUTHOR", "AI MODEL", "TIMESTAMP"];

/// How many lines from the top are searched for the header.
const SCAN_LINES: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail(Vec<String>),
    BrakHeadera,
    Excluded(String),
    Unknown(String),
}

impl Verdict {
    pub fn label(&self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Fail(_) => "FAIL",
            Verdict::BrakHeadera => "BRAK_HEADERA",
            Verdict::Excluded(_) => "EXCLUDED",
            Verdict::Unknown(_) => "UNKNOWN",
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Verdict::Fail(reasons) => reasons.join("; "),
            Verdict::Excluded(r) | Verdict::Unknown(r) => r.clone(),
            Verdict::BrakHeadera => "no provenance header found".into(),
            Verdict::Pass => String::new(),
        }
    }
}

/// Header policy for a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Policy {
    Required,
    Excluded(String),
    Unknown(String),
}

const REQUIRED_EXT: [&str; 27] = [
    "rs", "toml", "sh", "bash", "ps1", "psm1", "py", "c", "h", "cpp", "hpp", "cc", "js", "mjs",
    "ts", "tsx", "jsx", "go", "md", "yml", "yaml", "conf", "service", "html", "css", "sql",
    "cmake",
];

const BINARY_EXT: [&str; 17] = [
    "png", "jpg", "jpeg", "gif", "ico", "pdf", "zip", "gz", "tar", "bin", "exe", "dll", "so",
    "wasm", "woff", "woff2", "ttf",
];

const LOCK_NAMES: [&str; 6] = [
    "Cargo.lock",
    "package-lock.json",
    "bun.lock",
    "yarn.lock",
    "go.sum",
    "dependencies.lock",
];

const EXCLUDED_DIRS: [&str; 6] = [
    "vendor/",
    "third_party/",
    "third-party/",
    "node_modules/",
    "target/",
    ".git/",
];

/// Decide whether a file must carry provenance. `extra_excludes` are
/// (path-prefix, reason) pairs from the workspace config.
pub fn policy_for(
    rel_path: &str,
    content: Option<&str>,
    extra_excludes: &[(String, String)],
) -> Policy {
    let rel = rel_path.trim_start_matches("./");
    for (prefix, reason) in extra_excludes {
        if rel.starts_with(prefix.as_str()) {
            return Policy::Excluded(format!("workspace exclude: {reason}"));
        }
    }
    for dir in EXCLUDED_DIRS {
        if rel.starts_with(dir) || rel.contains(&format!("/{dir}")) {
            return Policy::Excluded(format!("third-party or build directory ({dir})"));
        }
    }
    let name = rel.rsplit('/').next().unwrap_or(rel);
    if LOCK_NAMES.contains(&name) || name.ends_with(".lock") {
        return Policy::Excluded("lock file (machine-managed)".into());
    }
    let upper = name.to_ascii_uppercase();
    if upper.starts_with("LICENSE") || upper.starts_with("COPYING") || upper.starts_with("NOTICE") {
        return Policy::Excluded("license text (externally defined)".into());
    }
    if name.starts_with('.') && !name[1..].contains('.') {
        return Policy::Excluded("dotfile configuration (.gitignore style)".into());
    }
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    if let Some(ext) = ext.as_deref() {
        if BINARY_EXT.contains(&ext) {
            return Policy::Excluded("binary format".into());
        }
        if ext == "json" {
            return Policy::Excluded("JSON cannot carry comments".into());
        }
    }
    let Some(content) = content else {
        return Policy::Excluded("not valid UTF-8 (binary)".into());
    };
    let head: String = content.lines().take(5).collect::<Vec<_>>().join("\n");
    if head.contains("@generated") || head.contains("DO NOT EDIT") {
        return Policy::Excluded("generated file".into());
    }
    match ext.as_deref() {
        Some(e) if REQUIRED_EXT.contains(&e) => Policy::Required,
        _ if content.starts_with("#!") => Policy::Required,
        Some(e) => Policy::Unknown(format!("no header policy for .{e} files")),
        None => Policy::Unknown("no extension and no shebang: header policy unknown".into()),
    }
}

/// Remove one leading comment marker and surrounding whitespace.
fn strip_comment(line: &str) -> &str {
    let mut s = line.trim();
    for marker in [
        "//!", "///", "//", "<!--", "/*", "#", "--", ";", "REM ", "*",
    ] {
        if let Some(rest) = s.strip_prefix(marker) {
            s = rest.trim();
            break;
        }
    }
    for tail in ["-->", "*/"] {
        if let Some(rest) = s.strip_suffix(tail) {
            s = rest.trim();
        }
    }
    s
}

/// Lines that may legally precede the first field (banner, comment openers).
fn is_decorative(stripped: &str) -> bool {
    stripped.is_empty()
        || stripped
            .chars()
            .all(|c| matches!(c, '=' | '-' | '*' | '#' | '/'))
        || stripped.contains("AIONS ARCHITECTURE")
}

/// Index of the first line after the structural preamble.
fn preamble_end(lines: &[&str]) -> usize {
    let mut i = 0;
    if lines.first().is_some_and(|l| l.starts_with("#!")) {
        i = 1;
    }
    if lines
        .get(i)
        .is_some_and(|l| l.contains("coding:") || l.contains("coding="))
    {
        i += 1;
    }
    if lines
        .get(i)
        .is_some_and(|l| l.trim_start().starts_with("<?xml"))
    {
        i += 1;
    }
    if lines.get(i).is_some_and(|l| l.trim() == "---") {
        // Markdown / YAML front matter: header goes after it.
        if let Some(end) = lines.iter().skip(i + 1).position(|l| l.trim() == "---") {
            i = i + 1 + end + 1;
        }
    }
    i
}

/// Parse `KEY: value` header fields. Returns (line index of first field, fields).
pub fn parse_header(content: &str) -> Option<(usize, usize, Vec<(String, String)>)> {
    let lines: Vec<&str> = content.lines().take(SCAN_LINES).collect();
    let start = preamble_end(&lines);
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut first: Option<usize> = None;
    for (idx, line) in lines.iter().enumerate().skip(start) {
        let s = strip_comment(line);
        let Some((key, value)) = s.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_uppercase();
        let canonical = ALIASES
            .iter()
            .find(|(alias, _)| *alias == key)
            .map(|(_, c)| c.to_string())
            .or_else(|| FIELDS.iter().find(|f| **f == key).map(|f| f.to_string()));
        if let Some(name) = canonical {
            if !fields.iter().any(|(k, _)| *k == name) {
                first.get_or_insert(idx);
                fields.push((name, value.trim().to_string()));
            }
        }
    }
    first.map(|f| (start, f, fields))
}

/// `YYYY-MM-DDTHH:MM[:SS[.frac]](Z|+HH:MM|-HH:MM)`
pub fn is_iso8601_with_tz(value: &str) -> bool {
    let v = value.trim();
    let b = v.as_bytes();
    let digit = |i: usize| b.get(i).is_some_and(u8::is_ascii_digit);
    let date_ok = b.len() >= 16
        && (0..4).all(digit)
        && b[4] == b'-'
        && digit(5)
        && digit(6)
        && b[7] == b'-'
        && digit(8)
        && digit(9)
        && b[10] == b'T'
        && digit(11)
        && digit(12)
        && b[13] == b':'
        && digit(14)
        && digit(15);
    if !date_ok {
        return false;
    }
    let mut i = 16;
    if b.get(i) == Some(&b':') {
        if !(digit(i + 1) && digit(i + 2)) {
            return false;
        }
        i += 3;
        if b.get(i) == Some(&b'.') {
            i += 1;
            let s = i;
            while digit(i) {
                i += 1;
            }
            if i == s {
                return false;
            }
        }
    }
    match &v[i..] {
        "Z" => true,
        tz => {
            let t = tz.as_bytes();
            t.len() == 6
                && (t[0] == b'+' || t[0] == b'-')
                && t[1].is_ascii_digit()
                && t[2].is_ascii_digit()
                && t[3] == b':'
                && t[4].is_ascii_digit()
                && t[5].is_ascii_digit()
        }
    }
}

fn is_sentinel(v: &str) -> bool {
    let u = v.trim().to_ascii_uppercase();
    u == "UNKNOWN" || u == "PENDING" || u.starts_with("N/A")
}

/// Result of asking git whether a commit exists.
#[derive(Debug, PartialEq, Eq)]
pub enum CommitCheck {
    Exists,
    Missing,
    CannotVerify,
}

pub fn git_commit_exists(repo: &Path, hash: &str) -> CommitCheck {
    match Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["cat-file", "-e", &format!("{hash}^{{commit}}")])
        .output()
    {
        Ok(out) if out.status.success() => CommitCheck::Exists,
        Ok(_) => CommitCheck::Missing,
        Err(_) => CommitCheck::CannotVerify,
    }
}

/// Validate header content. `repo` enables GIT COMMIT verification.
pub fn validate_content(content: &str, repo: Option<&Path>) -> Verdict {
    let Some((start, first, fields)) = parse_header(content) else {
        return Verdict::BrakHeadera;
    };
    let mut problems = Vec::new();
    let mut unknown = None;

    let lines: Vec<&str> = content.lines().collect();
    if let Some(bad) = (start..first).find(|&i| !is_decorative(strip_comment(lines[i]))) {
        problems.push(format!(
            "header not at earliest valid location: content before it at line {}",
            bad + 1
        ));
    }
    for field in FIELDS {
        match fields.iter().find(|(k, _)| k == field) {
            None => problems.push(format!("missing field {field}")),
            Some((_, v)) if v.is_empty() => problems.push(format!("empty field {field}")),
            Some((_, v)) if v.starts_with('[') && v.ends_with(']') => {
                problems.push(format!("template placeholder left in {field}"))
            }
            _ => {}
        }
    }
    if let Some((_, ts)) = fields.iter().find(|(k, _)| k == "TIMESTAMP") {
        if !ts.is_empty() && !is_sentinel(ts) && !is_iso8601_with_tz(ts) {
            problems.push(format!("TIMESTAMP '{ts}' is not ISO-8601 with timezone"));
        }
    }
    if let Some((_, commit)) = fields.iter().find(|(k, _)| k == "GIT COMMIT") {
        let c = commit.trim();
        let is_hash = (7..=40).contains(&c.len()) && c.chars().all(|ch| ch.is_ascii_hexdigit());
        if is_hash {
            match repo.map(|r| git_commit_exists(r, c)) {
                Some(CommitCheck::Exists) => {}
                Some(CommitCheck::Missing) => problems.push(format!(
                    "GIT COMMIT claims {c}, which does not exist in this repository"
                )),
                _ => unknown = Some(format!("cannot verify GIT COMMIT {c}")),
            }
        } else if !c.is_empty() && !is_sentinel(c) {
            problems.push(format!(
                "GIT COMMIT '{c}' is not a hash, PENDING, UNKNOWN or N/A"
            ));
        }
    }
    if !problems.is_empty() {
        Verdict::Fail(problems)
    } else if let Some(u) = unknown {
        Verdict::Unknown(u)
    } else {
        Verdict::Pass
    }
}

/// Existing-file protocol: creation provenance present at `before` must be unchanged in `after`.
pub fn creation_provenance_problems(before: &str, after: &str) -> Vec<String> {
    let Some((_, _, old)) = parse_header(before) else {
        return Vec::new();
    };
    let new = parse_header(after).map(|(_, _, f)| f).unwrap_or_default();
    CREATION_FIELDS
        .iter()
        .filter_map(|field| {
            let o = old.iter().find(|(k, _)| k == field)?;
            match new.iter().find(|(k, _)| k == field) {
                Some(n) if n.1 == o.1 => None,
                Some(n) => Some(format!(
                    "creation provenance {field} rewritten ('{}' -> '{}')",
                    o.1, n.1
                )),
                None => Some(format!("creation provenance {field} removed")),
            }
        })
        .collect()
}

/// Full check for one file on disk.
pub fn check_file(repo: &Path, rel: &str, extra_excludes: &[(String, String)]) -> Verdict {
    let path = repo.join(rel);
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) => return Verdict::Unknown(format!("cannot read: {e}")),
    };
    check_bytes(repo, rel, bytes, extra_excludes)
}

/// Check already-loaded content (e.g. the staged blob from the git index).
pub fn check_bytes(
    repo: &Path,
    rel: &str,
    bytes: Vec<u8>,
    extra_excludes: &[(String, String)],
) -> Verdict {
    let text = String::from_utf8(bytes).ok();
    match policy_for(rel, text.as_deref(), extra_excludes) {
        Policy::Excluded(r) => Verdict::Excluded(r),
        Policy::Unknown(r) => Verdict::Unknown(r),
        Policy::Required => validate_content(text.as_deref().unwrap_or_default(), Some(repo)),
    }
}
