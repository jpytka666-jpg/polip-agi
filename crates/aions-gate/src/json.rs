//! ================================================================================
//! AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER
//! ================================================================================
//! AUTHOR:              Claude Code agent, working for Marcin (jpytka666-jpg)
//! AI MODEL:            Anthropic Claude via Claude Code remote session (exact model id WITHHELD by harness policy)
//! TIMESTAMP:           2026-09-25T23:14:45Z
//! REASON FOR CREATION: aions-gate must emit machine-readable evidence without pulling serde into the trusted enforcement path.
//! MECHANICS:           Escapes strings for JSON and renders UTC timestamps from SystemTime (civil-from-days algorithm).
//! SYSTEM PART:         polip-agi / aions-gate / evidence encoding
//! ARCHITECTURE FUNC:   Shared helper for the enforcement plane's PASS/FAIL/UNKNOWN evidence records.
//! DEPENDENCIES/LINKS:  std only; used by crate::gates and main.rs
//! TECH STACK:          Rust 2024, std only
//! LOCAL WORKSPACE:     polip-agi/crates/aions-gate
//! GIT COMMIT:          PENDING
//! GITHUB METADATA:     jpytka666-jpg/polip-agi, branch claude/autonomous-automation-agent-ocxwke
//! ================================================================================

use std::time::{SystemTime, UNIX_EPOCH};

/// Quote and escape a string as a JSON string literal.
pub fn string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Current UTC time as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn now_utc() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_utc(secs)
}

/// Format seconds since the Unix epoch as ISO-8601 UTC.
pub fn format_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_control_and_quotes() {
        assert_eq!(string("a\"b\\c\n\u{1}"), "\"a\\\"b\\\\c\\n\\u0001\"");
    }

    #[test]
    fn formats_known_instants() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format_utc(1_790_378_085), "2026-09-25T23:14:45Z");
    }
}
