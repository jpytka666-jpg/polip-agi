// darkstar-header-v1
// po co: headscale_http.rs
// nie wolno: zapisywac czegokolwiek w Headscale, gasic tailscaled, wiazac na 0.0.0.0
// Autor: Marcin Szul
// Model: Claude
// Czas: 2026-09-02T22:25:10+01:00
// Po co: Odczyt zdrowia wlasnego serwera mesh Headscale i listy jego wezlow, zeby Sterownia
//        widziala prywatny mesh obok dzialajacego Tailscale, nie zamiast niego.
//
//! MECHANICS: Jedna sciezka `GET /v1/headscale/health`. Dwa odczyty przez trait
//! HeadscaleReader: `/health` bez klucza i `/api/v1/node` z kluczem API. Brak klucza NIE
//! jest bledem - zdrowie wraca normalnie, a lista wezlow jako `null` z podanym powodem.
//! Dzieki temu sciezka jest uzyteczna zanim ktokolwiek wygeneruje klucz.
//! Headscale nieosiagalny konczy sie 503 z golym kodem, tak samo jak brama i git.
//! SYSTEM PART: Darkstar Server / HTTP boundary - warstwa prywatnego mesh.
//! ARCHITECTURE FUNCTION: Wylacznie okno na Headscale. Router rejestruje sam GET, wiec
//! zadna metoda zmieniajaca tu nie istnieje - dolaczanie wezlow, klucze i polityka
//! pozostaja poza API. Tailscale nie jest tu w ogole dotykany: to osobna, dzialajaca
//! droga ratunkowa i ma zostac podniesiona.
//! DEPENDENCIES/LINKS: axum 0.8; Headscale v0.29.3 pod adresem z DARKSTAR_HEADSCALE_URL
//! (domyslnie http://127.0.0.1:8080), klucz API z DARKSTAR_HEADSCALE_APIKEY.
//! TECH STACK: Rust 2024 + Axum 0.8, spojnie z reszta warstwy HTTP. Odczyt wstrzykiwany
//! przez trait, wiec test nie potrzebuje ani Headscale, ani sieci.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};

/// Blad odczytu. Tresc trafia wylacznie do logu - nigdy do odpowiedzi HTTP, bo niesie
/// adresy hosta i nazwy uslug.
#[derive(Debug)]
pub struct HeadscaleError(String);

impl HeadscaleError {
    pub fn new(detail: impl Into<String>) -> Self {
        Self(detail.into())
    }
}

impl std::fmt::Display for HeadscaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Odczyt z Headscale. Kazde wywolanie jest GET-em - implementacja produkcyjna nie ma
/// zadnej metody, ktora moglaby cokolwiek w mesh zmienic.
pub trait HeadscaleReader {
    fn get(&self, url: &str, api_key: Option<&str>) -> Result<String, HeadscaleError>;
}

#[derive(Clone)]
pub struct HeadscaleState {
    pub api_token: Option<Arc<str>>,
    pub reader: Arc<dyn HeadscaleReader + Send + Sync>,
    pub base_url: Arc<str>,
    pub headscale_key: Option<Arc<str>>,
}

impl HeadscaleState {
    pub fn new(
        api_token: Option<Arc<str>>,
        reader: Arc<dyn HeadscaleReader + Send + Sync>,
        base_url: impl Into<String>,
        headscale_key: Option<Arc<str>>,
    ) -> Self {
        Self {
            api_token,
            reader,
            base_url: Arc::from(base_url.into()),
            headscale_key,
        }
    }
}

fn authenticated(state: &HeadscaleState, headers: &HeaderMap) -> bool {
    // Fail-closed, tak samo jak brama, kontekst i git.
    let Some(expected) = state.api_token.as_deref() else {
        return false;
    };
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token == expected)
        .unwrap_or(false)
}

/// Headscale odpowiada `{"status":"pass"}`. Gdy ksztalt bedzie inny, oddajemy surowa
/// tresc zamiast udawac, ze rozumiemy - zgadywanie ukryloby prawdziwy stan.
fn health_status(raw: &str) -> serde_json::Value {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(value) => value
            .get("status")
            .cloned()
            .unwrap_or_else(|| serde_json::Value::String(raw.trim().to_string())),
        Err(_) => serde_json::Value::String(raw.trim().to_string()),
    }
}

/// Lista wezlow. Zwraca `None` razem z powodem zamiast pustej listy - "nie wiem" i
/// "wiem, ze zero" to dwa rozne stany i operator ma je rozroznic.
fn read_nodes(state: &HeadscaleState) -> (Option<serde_json::Value>, Option<&'static str>) {
    let Some(key) = state.headscale_key.as_deref() else {
        return (None, Some("no_api_key"));
    };
    let url = format!("{}/api/v1/node", state.base_url);
    match state.reader.get(&url, Some(key)) {
        Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(value) => {
                let nodes = value.get("nodes").cloned().unwrap_or(value);
                (Some(nodes), None)
            }
            Err(_) => (None, Some("unparseable")),
        },
        Err(_) => (None, Some("unreadable")),
    }
}

async fn headscale_health(
    State(state): State<HeadscaleState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication_required" })),
        )
            .into_response();
    }

    let health_url = format!("{}/health", state.base_url);
    let raw = match state.reader.get(&health_url, None) {
        Ok(raw) => raw,
        Err(err) => {
            tracing::warn!(error = %err, "headscale unreachable");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "headscale_unavailable" })),
            )
                .into_response();
        }
    };

    let (nodes, nodes_reason) = read_nodes(&state);
    let node_count = nodes.as_ref().and_then(|value| value.as_array()).map(Vec::len);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "reachable": true,
            "health": health_status(&raw),
            "baseUrl": state.base_url.as_ref(),
            "nodes": nodes,
            "nodeCount": node_count,
            "nodesReason": nodes_reason,
        })),
    )
        .into_response()
}

/// Wylacznie GET. Dolaczanie wezlow, klucze i polityka nie sa tu wystawione i nie wolno
/// ich dolozyc - to osobna zdolnosc z osobna autoryzacja.
pub fn headscale_router(state: HeadscaleState) -> Router {
    Router::new()
        .route("/v1/headscale/health", get(headscale_health))
        .with_state(state)
}
