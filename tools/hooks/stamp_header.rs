// THIS IS VERY IMPORTANT!!!
// ==========================================
// AUTHOR: M. SZUL
// AI MODEL: Claude Opus 5
// TIMESTAMP: 2026-09-01 08:30:00
// REASON FOR CREATION: Hak nagłówków dla Claude Code na Windows, zastępujący stamp_metadata.py. Dopisuje krótki nagłówek raz i nigdy go nie przepisuje.
// MECHANICS: Czyta ścieżkę z argv[1], zmiennej CLAUDE_FILE albo z pola file_path w JSON na stdin. Jeśli w pierwszych 800 znakach pliku widzi znacznik darkstar-header-v1 albo frazę "po co:", kończy bez zapisu. W przeciwnym razie wstawia pięć linii komentarza za shebangiem i przepisuje plik atomowo przez plik tymczasowy obok celu. Data liczona z zegara systemowego algorytmem civil_from_days, bez zależności zewnętrznych.
// SYSTEM PART: C:\Users\User\.claude\hooks - warstwa haków PostToolUse Write|Edit.
// ARCHITECTURE FUNCTION: Utrzymuje ślad pochodzenia w plikach źródłowych bez tworzenia kopii .bak i bez odświeżania dat przy każdej edycji, co wcześniej zaśmiecało diffy.
// DEPENDENCIES/LINKS: Claude Code settings.json (PostToolUse, matcher Write|Edit); zastępuje hooks/stamp_metadata.py.
// TECH STACK: Rust 2021, wyłącznie biblioteka standardowa - hak musi wystartować w milisekundach i nie może zależeć od interpretera ani od pakietów.
// LOCAL WORKSPACE: C:\Users\User\.claude\hooks\stamp_header.rs
// GIT COMMIT: PENDING
// GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
// ==========================================

use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

const MARK: &str = "darkstar-header-v1";
const SCAN_WINDOW: usize = 800;
const RULES: &str =
    "nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie";
const AUTHOR: &str = "autor: Marcin";

/// Styl komentarza wybrany dla danego pliku.
enum Style {
    Hash,
    Slash,
    Html,
}

impl Style {
    fn wrap(&self, line: &str) -> String {
        match self {
            Style::Hash => format!("# {}", line),
            Style::Slash => format!("// {}", line),
            Style::Html => format!("<!-- {} -->", line),
        }
    }
}

fn main() {
    // Hak nie ma prawa przerwac sesji: kazda sciezka bledu konczy sie cicho kodem 0.
    let path = match resolve_path() {
        Some(p) => p,
        None => process::exit(0),
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => process::exit(0),
    };

    let style = match style_for(&path, &content) {
        Some(s) => s,
        None => {
            println!("skip: nieobslugiwany typ pliku");
            process::exit(0);
        }
    };

    if already_stamped(&content) {
        println!("exists");
        process::exit(0);
    }

    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("plik"));

    let header = build_header(&style, &name, &today());
    let updated = insert_after_shebang(&content, &header);

    if write_atomic(&path, &updated).is_err() {
        process::exit(0);
    }

    println!("stamped: {}", name);
}

/// argv[1], potem CLAUDE_FILE, na koncu pole file_path z JSON na stdin.
fn resolve_path() -> Option<PathBuf> {
    if let Some(arg) = env::args().nth(1) {
        if !arg.trim().is_empty() {
            return Some(PathBuf::from(arg));
        }
    }

    if let Ok(value) = env::var("CLAUDE_FILE") {
        if !value.trim().is_empty() {
            return Some(PathBuf::from(value));
        }
    }

    let mut payload = String::new();
    if std::io::stdin().read_to_string(&mut payload).is_err() {
        return None;
    }
    extract_file_path(&payload).map(PathBuf::from)
}

/// Wyciaga wartosc "file_path" bez parsera JSON - hak ma zostac zaleznosciowo pusty.
fn extract_file_path(payload: &str) -> Option<String> {
    let key = "\"file_path\"";
    let start = payload.find(key)? + key.len();
    let rest = &payload[start..];
    let open = rest.find('"')? + 1;
    let tail = &rest[open..];

    let mut out = String::new();
    let mut chars = tail.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next() {
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => return None,
            },
            other => out.push(other),
        }
    }
    None
}

/// Nagłówek juz jest, gdy w oknie poczatkowym widac znacznik albo fraze "po co:".
fn already_stamped(content: &str) -> bool {
    let window: String = content.chars().take(SCAN_WINDOW).collect();
    let lowered = window.to_lowercase();
    lowered.contains(MARK) || lowered.contains("po co:")
}

fn style_for(path: &Path, content: &str) -> Option<Style> {
    let name = path.file_name()?.to_string_lossy().to_lowercase();

    if name.contains(".env") {
        return Some(Style::Hash);
    }

    match path.extension().map(|e| e.to_string_lossy().to_lowercase()) {
        Some(ext) => match ext.as_str() {
            "sh" | "nft" | "env" | "profile" | "toml" | "yaml" | "yml" | "conf" | "service" => {
                Some(Style::Hash)
            }
            "rs" => Some(Style::Slash),
            "md" | "markdown" | "html" => Some(Style::Html),
            "example" => Some(Style::Hash),
            _ => None,
        },
        // Bez rozszerzenia: skrypt rozpoznajemy po shebangu, reszty nie ruszamy.
        None => {
            if content.starts_with("#!") {
                Some(Style::Hash)
            } else {
                None
            }
        }
    }
}

fn build_header(style: &Style, name: &str, date: &str) -> String {
    let lines = [
        MARK.to_string(),
        format!("po co: {}", name),
        RULES.to_string(),
        AUTHOR.to_string(),
        format!("powstal: {}", date),
    ];
    lines
        .iter()
        .map(|l| style.wrap(l))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Shebang musi zostac pierwsza linia pliku, inaczej skrypt przestaje sie uruchamiac.
fn insert_after_shebang(content: &str, header: &str) -> String {
    if content.starts_with("#!") {
        match content.find('\n') {
            Some(idx) => {
                let (shebang, rest) = content.split_at(idx + 1);
                format!("{}{}\n{}", shebang, header, rest)
            }
            None => format!("{}\n{}\n", content, header),
        }
    } else {
        format!("{}\n{}", header, content)
    }
}

/// Zapis przez plik tymczasowy obok celu; zadnych kopii .bak.
fn write_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    let mut tmp = path.to_path_buf();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp_name = format!(
        "{}.stamp{}",
        path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("out")),
        stamp
    );
    tmp.set_file_name(tmp_name);

    fs::write(&tmp, content)?;
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(e)
        }
    }
}

fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Howard Hinnant, civil_from_days: dni od 1970-01-01 na date kalendarzowa.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
