// darkstar-header-v1
// po co: lib.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-04
//! Klient magazynu pamieci Darkstar (darkstar-recall).
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-04 23:15:00
//! REASON FOR CREATION: Warstwa dostepu do pamieci dlugoterminowej AIONS v2. Pierwszym
//! uzytkownikiem jest przeprowadzka 4629 wspomnien ze starej Chromy na Windowsie; docelowo
//! ta sama biblioteka obsluguje odczyt i zapis pamieci z darkstar-server.
//! MECHANICS: Wlasny, minimalny klient HTTP na std::net - zero nowych zaleznosci, tak samo jak
//! context_http::ReadOnlyHttp, tylko z metodami zmieniajacymi (POST/PUT), bo migracja musi
//! zapisywac. Klucz API idzie naglowkiem, NIGDY w URL - adres trafia do logow, naglowek nie.
//! SYSTEM PART: Darkstar Recall / warstwa dostepu do pamieci.
//! ARCHITECTURE FUNCTION: Jedyne miejsce w kodzie, ktore wie, jak rozmawia sie z magazynem
//! pamieci. Reszta systemu dostaje typy, nie surowy JSON.
//! DEPENDENCIES/LINKS: deploy/recall/compose.yml (Qdrant na 127.0.0.1:6333),
//! deploy/.env (DARKSTAR_RECALL_API_KEY).
//! TECH STACK: Rust 2024 + serde. Bez klienta HTTP z crates.io: caly ruch idzie po petli
//! zwrotnej do wlasnej uslugi, wiec TLS, przekierowania i pula polaczen nie sa potrzebne,
//! a kazda zaleznosc to kolejna rzecz do pilnowania przy przenosinach na Redox.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

#[derive(Debug, thiserror::Error)]
pub enum RecallError {
    #[error("nieosiagalny: {0}")]
    Unreachable(String),
    #[error("odpowiedz HTTP {code}: {body}")]
    Status { code: u16, body: String },
    #[error("zla odpowiedz: {0}")]
    Malformed(String),
}

/// Minimalny klient HTTP. Obsluguje wylacznie to, czego potrzebuje magazyn pamieci na
/// petli zwrotnej: GET, POST i PUT z cialem JSON.
pub struct HttpClient {
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key,
            // Migracja wysyla paczki po kilkaset wektorow - baza potrzebuje na to wiecej
            // niz kilka sekund, ale wisiec w nieskonczonosc tez nie moze.
            timeout: Duration::from_secs(120),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn get(&self, path: &str) -> Result<String, RecallError> {
        self.request("GET", path, None)
    }

    pub fn post(&self, path: &str, body: &str) -> Result<String, RecallError> {
        self.request("POST", path, Some(body))
    }

    pub fn put(&self, path: &str, body: &str) -> Result<String, RecallError> {
        self.request("PUT", path, Some(body))
    }

    fn request(&self, method: &str, path: &str, body: Option<&str>) -> Result<String, RecallError> {
        let rest = self
            .base_url
            .strip_prefix("http://")
            .ok_or_else(|| RecallError::Unreachable(format!("tylko http: {}", self.base_url)))?;
        let (authority, base_path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, ""),
        };

        // Limit na samo polaczenie: bez niego zapytanie do adresu, ktory zapora cicho
        // odrzuca, wisi do systemowego timeoutu TCP.
        let addr = authority
            .to_socket_addrs()
            .map_err(|e| RecallError::Unreachable(format!("{authority}: {e}")))?
            .next()
            .ok_or_else(|| RecallError::Unreachable(format!("{authority}: brak adresu")))?;
        let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
            .map_err(|e| RecallError::Unreachable(format!("{authority}: {e}")))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .and_then(|_| stream.set_write_timeout(Some(self.timeout)))
            .map_err(|e| RecallError::Unreachable(e.to_string()))?;

        let mut head = format!(
            "{method} {base_path}{path} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n"
        );
        // Klucz w naglowku, nigdy w adresie: adresy trafiaja do logow serwera i do historii.
        if let Some(key) = &self.api_key {
            head.push_str(&format!("api-key: {key}\r\n"));
        }
        if let Some(b) = body {
            head.push_str("content-type: application/json\r\n");
            head.push_str(&format!("content-length: {}\r\n", b.len()));
        }
        head.push_str("\r\n");

        stream
            .write_all(head.as_bytes())
            .map_err(|e| RecallError::Unreachable(e.to_string()))?;
        if let Some(b) = body {
            stream
                .write_all(b.as_bytes())
                .map_err(|e| RecallError::Unreachable(e.to_string()))?;
        }

        let mut reader = BufReader::new(stream);
        let mut status_line = String::new();
        reader
            .read_line(&mut status_line)
            .map_err(|e| RecallError::Unreachable(e.to_string()))?;
        let code: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|c| c.parse().ok())
            .ok_or_else(|| RecallError::Malformed(status_line.trim().to_string()))?;

        // Naglowki pomijamy, ale zapamietujemy dlugosc: przy Connection: close serwer
        // zamyka gniazdo, wiec read_to_string wystarcza - a content-length sluzy
        // wylacznie do wykrycia uciecia odpowiedzi w polowie.
        let mut declared_len: Option<usize> = None;
        loop {
            let mut line = String::new();
            let read = reader
                .read_line(&mut line)
                .map_err(|e| RecallError::Unreachable(e.to_string()))?;
            if read == 0 || line.trim().is_empty() {
                break;
            }
            let lower = line.to_ascii_lowercase();
            if let Some(v) = lower.strip_prefix("content-length:") {
                declared_len = v.trim().parse().ok();
            }
        }

        let mut body_out = String::new();
        reader
            .read_to_string(&mut body_out)
            .map_err(|e| RecallError::Malformed(e.to_string()))?;

        if let Some(expected) = declared_len
            && body_out.len() < expected
        {
            return Err(RecallError::Malformed(format!(
                "odpowiedz ucieta: {} z {expected} bajtow",
                body_out.len()
            )));
        }

        if (200..300).contains(&code) {
            Ok(body_out)
        } else {
            Err(RecallError::Status {
                code,
                body: body_out.chars().take(300).collect(),
            })
        }
    }
}

/// Odczytuje wartosc klucza z pliku w formacie KLUCZ=wartosc.
///
/// Klucz API nie moze byc podawany argumentem polecenia - argumenty widzi kazdy uzytkownik
/// maszyny przez `ps`. Ta funkcja czyta go z pliku o prawach 0600.
pub fn read_env_value(path: &str, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let prefix = format!("{key}=");
    content
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_value_is_read_by_exact_key() {
        let dir = std::env::temp_dir().join(format!("darkstar-recall-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("katalog testowy");
        let file = dir.join("test.env");
        std::fs::write(
            &file,
            "INNY_KLUCZ=nie-ten\nDARKSTAR_RECALL_API_KEY=wartosc-oczekiwana\nPUSTY=\n",
        )
        .expect("zapis");

        let path = file.to_string_lossy().to_string();
        assert_eq!(
            read_env_value(&path, "DARKSTAR_RECALL_API_KEY").as_deref(),
            Some("wartosc-oczekiwana")
        );
        // Klucz o podobnym poczatku nie moze byc mylony z szukanym.
        assert_eq!(read_env_value(&path, "DARKSTAR_RECALL").as_deref(), None);
        // Pusta wartosc to brak wartosci - inaczej wyslalibysmy pusty naglowek zamiast
        // zadnego i dostali 401 zamiast jasnego bledu konfiguracji.
        assert_eq!(read_env_value(&path, "PUSTY"), None);
        assert_eq!(read_env_value(&path, "NIE_ISTNIEJE"), None);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_file_is_not_a_panic() {
        assert_eq!(read_env_value("/nie/ma/takiego/pliku.env", "COKOLWIEK"), None);
    }
}
