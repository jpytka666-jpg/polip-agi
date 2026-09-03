// darkstar-header-v1
// po co: context_http.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-02 01:30:00
//! REASON FOR CREATION: Granica HTTP dla wyszukiwania w pamieci AIONS - wylacznie odczyt.
//! MECHANICS: Wystawia `GET /v1/context/search` i `GET /v1/context/health`. Zapytania trafiaja do
//! ContextClient z dwiema nogami: zdalna na E: i lokalna na CBMS. Gdy obie milcza, odpowiedzia
//! jest 503, nigdy pusta lista - awaria nie moze wygladac jak brak wiedzy. Router nie rejestruje
//! zadnej metody zmieniajacej, wiec baza na E: pozostaje nietykalna.
//! SYSTEM PART: Darkstar Server / HTTP boundary - warstwa kontekstu.
//! ARCHITECTURE FUNCTION: Pozwala Control Room i agentom siegac po pamiec AIONS przez Darkstara,
//! zamiast laczyc sie z Chroma bezposrednio.
//! DEPENDENCIES/LINKS: darkstar_core::context_client; Chroma HTTP API v1 na E: i na CBMS.
//! TECH STACK: Rust 2024 + Axum 0.8. Transport wstrzykiwany traitem, wiec testy nie potrzebuja sieci.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use darkstar_core::context_client::{ContextClient, ContextError, ContextLeg, ContextTransport};
use serde::Deserialize;

#[derive(Clone)]
pub struct ContextState {
    pub api_token: Option<Arc<str>>,
    pub transport: Arc<dyn ContextTransport + Send + Sync>,
    pub preferred: ContextLeg,
    pub fallback: ContextLeg,
}

impl ContextState {
    pub fn new(
        api_token: Option<Arc<str>>,
        transport: Arc<dyn ContextTransport + Send + Sync>,
    ) -> Self {
        // Adresy nog musza byc konfigurowalne: to samo wykonywalne dziala na Windows
        // (gdzie zrodlem jest E:) i na CBMS (gdzie noga lokalna slucha na petli zwrotnej).
        // Wpisanie ich na sztywno dawaloby 503 na jednej z maszyn.
        let local_url = std::env::var("DARKSTAR_CONTEXT_LOCAL")
            .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
        let remote_url = std::env::var("DARKSTAR_CONTEXT_REMOTE")
            .unwrap_or_else(|_| "http://192.168.2.50:8000".to_string());
        // Noga lokalna jest pierwsza: gdy dane sa na miejscu, nie ma powodu isc przez siec.
        Self {
            api_token,
            transport,
            preferred: ContextLeg::local_cbms(&local_url),
            fallback: ContextLeg::remote_e(&remote_url),
        }
    }
}

struct SharedTransport(Arc<dyn ContextTransport + Send + Sync>);

impl ContextTransport for SharedTransport {
    fn get(&self, url: &str) -> Result<String, ContextError> {
        self.0.get(url)
    }
}

/// Minimalny klient HTTP na std - wylacznie metoda GET, bez ciala zadania.
/// Nie da sie nim nic zapisac, nawet przez pomylke.
pub struct ReadOnlyHttp;

impl ContextTransport for ReadOnlyHttp {
    fn get(&self, url: &str) -> Result<String, ContextError> {
        let rest = url
            .strip_prefix("http://")
            .ok_or_else(|| ContextError::Unreachable(format!("only http supported: {url}")))?;
        let (authority, path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };

        // Sam connect MUSI miec limit czasu. Bez niego zapytanie do adresu, ktory zapora
        // cicho odrzuca, wisi do systemowego timeoutu TCP - okolo dwoch minut - i cala
        // sciezka HTTP zwraca 000 zamiast uczciwego 503.
        let addr = authority
            .to_socket_addrs()
            .map_err(|e| ContextError::Unreachable(format!("{authority}: {e}")))?
            .next()
            .ok_or_else(|| ContextError::Unreachable(format!("{authority}: brak adresu")))?;
        let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(3))
            .map_err(|e| ContextError::Unreachable(format!("{authority}: {e}")))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(5))))
            .map_err(|e| ContextError::Unreachable(e.to_string()))?;

        let request =
            format!("GET {path} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n");
        stream
            .write_all(request.as_bytes())
            .map_err(|e| ContextError::Unreachable(e.to_string()))?;

        let mut reader = BufReader::new(stream);
        let mut status = String::new();
        reader
            .read_line(&mut status)
            .map_err(|e| ContextError::Unreachable(e.to_string()))?;
        if !status.contains(" 200 ") {
            return Err(ContextError::Unreachable(status.trim().to_string()));
        }
        // Pominiecie naglowkow: czytamy do pustej linii.
        loop {
            let mut line = String::new();
            let read = reader
                .read_line(&mut line)
                .map_err(|e| ContextError::Unreachable(e.to_string()))?;
            if read == 0 || line.trim().is_empty() {
                break;
            }
        }
        let mut body = String::new();
        reader
            .read_to_string(&mut body)
            .map_err(|e| ContextError::Malformed(e.to_string()))?;
        Ok(body)
    }
}

fn authenticated(state: &ContextState, headers: &HeaderMap) -> bool {
    let Some(expected) = state.api_token.as_deref() else {
        return false;
    };
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|token| token == expected)
        .unwrap_or(false)
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

fn client(state: &ContextState) -> ContextClient<SharedTransport> {
    ContextClient::new(
        SharedTransport(Arc::clone(&state.transport)),
        state.preferred.clone(),
        state.fallback.clone(),
    )
}

async fn context_search(
    State(state): State<ContextState>,
    headers: HeaderMap,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication_required" })),
        )
            .into_response();
    }

    match client(&state).search(&params.q, params.limit) {
        Ok(listing) => (StatusCode::OK, Json(serde_json::json!(listing))).into_response(),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "context_unreachable", "detail": err.to_string() })),
        )
            .into_response(),
    }
}

async fn context_health(
    State(state): State<ContextState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication_required" })),
        )
            .into_response();
    }

    let health = client(&state).health();
    let code = if health.any_ok() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (code, Json(serde_json::json!(health))).into_response()
}

/// Wylacznie GET. Zadna metoda zmieniajaca nie jest tu zarejestrowana.
pub fn context_router(state: ContextState) -> Router {
    Router::new()
        .route("/v1/context/search", get(context_search))
        .route("/v1/context/health", get(context_health))
        .with_state(state)
}
