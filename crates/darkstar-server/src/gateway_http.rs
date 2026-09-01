// darkstar-header-v1
// po co: gateway_http.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-01 22:05:00
//! REASON FOR CREATION: Granica HTTP dla stanu bramy Darkstar - wylacznie odczyt (Task 8, Step 8.6).
//! MECHANICS: Wystawia jedna sciezke `GET /v1/gateway/status`. Router nie rejestruje zadnej
//! metody zmieniajacej, wiec POST, PUT i DELETE na tej sciezce koncza sie odmowa metody, a nie
//! przypadkowa mutacja. Dane pochodza z GatewayProvider, ktory sam potrafi wylacznie czytac.
//! Stan tego routera jest osobny od AppState, zeby nie ruszac dzialajacych sciezek i ich testow.
//! SYSTEM PART: Darkstar Server / HTTP boundary - warstwa bramy.
//! ARCHITECTURE FUNCTION: Jedyne wyjscie stanu bramy na zewnatrz. Sterowanie bramą powstanie
//! osobno, z wlasna autoryzacja zdolnosci - tutaj celowo go nie ma.
//! DEPENDENCIES/LINKS: darkstar_core::gateway_provider (CommandRunner, GatewayProvider),
//! darkstar_core::gateway_status (GatewayStatus); nmcli i ip na hoscie Ubuntu.
//! TECH STACK: Rust 2024 + Axum 0.8. Wykonanie polecen wstrzykiwane przez trait, wiec test
//! integracyjny nie potrzebuje ani hosta, ani SSH.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::{
    process::Command,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use darkstar_core::gateway_provider::{CommandRunner, GatewayProvider, GatewayProviderError};

#[derive(Clone)]
pub struct GatewayState {
    pub api_token: Option<Arc<str>>,
    pub runner: Arc<dyn CommandRunner + Send + Sync>,
    pub upstream_interface: Arc<str>,
    pub downstream_interface: Arc<str>,
    pub connection_profile: Arc<str>,
}

impl GatewayState {
    /// Wartosci domyslne odpowiadaja zywej konfiguracji hosta CBMS.
    pub fn new(api_token: Option<Arc<str>>, runner: Arc<dyn CommandRunner + Send + Sync>) -> Self {
        Self {
            api_token,
            runner,
            upstream_interface: Arc::from("wlp2s0"),
            downstream_interface: Arc::from("enp1s0"),
            connection_profile: Arc::from("DARKSTAR-WiFi"),
        }
    }
}

/// Przekazanie wspoldzielonego wykonawcy dalej, bez kopiowania implementacji.
struct SharedRunner(Arc<dyn CommandRunner + Send + Sync>);

impl CommandRunner for SharedRunner {
    fn active_connections(&self) -> Result<String, GatewayProviderError> {
        self.0.active_connections()
    }
    fn interface_addresses(&self, iface: &str) -> Result<String, GatewayProviderError> {
        self.0.interface_addresses(iface)
    }
    fn profile_ipv4(&self, profile: &str) -> Result<String, GatewayProviderError> {
        self.0.profile_ipv4(profile)
    }
    fn neighbours(&self, iface: &str) -> Result<String, GatewayProviderError> {
        self.0.neighbours(iface)
    }
}

/// Produkcyjny wykonawca: cztery polecenia odczytu i nic wiecej.
/// Nazwy interfejsow i profilu sa walidowane w rdzeniu, zanim tu trafia.
pub struct NmcliRunner;

impl NmcliRunner {
    fn run(program: &str, args: &[&str]) -> Result<String, GatewayProviderError> {
        let out = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| GatewayProviderError::CommandFailed(e.to_string()))?;
        if !out.status.success() {
            return Err(GatewayProviderError::CommandFailed(format!(
                "{program} exited with {}",
                out.status
            )));
        }
        String::from_utf8(out.stdout).map_err(|e| GatewayProviderError::Unparseable(e.to_string()))
    }
}

impl CommandRunner for NmcliRunner {
    fn active_connections(&self) -> Result<String, GatewayProviderError> {
        Self::run(
            "nmcli",
            &[
                "-t",
                "-f",
                "NAME,DEVICE,STATE",
                "connection",
                "show",
                "--active",
            ],
        )
    }
    fn interface_addresses(&self, iface: &str) -> Result<String, GatewayProviderError> {
        Self::run("ip", &["-4", "-o", "addr", "show", iface])
    }
    fn profile_ipv4(&self, profile: &str) -> Result<String, GatewayProviderError> {
        Self::run(
            "nmcli",
            &[
                "-t",
                "-g",
                "ipv4.method,ipv4.addresses",
                "connection",
                "show",
                profile,
            ],
        )
    }
    fn neighbours(&self, iface: &str) -> Result<String, GatewayProviderError> {
        Self::run("ip", &["-o", "neigh", "show", "dev", iface])
    }
}

fn authenticated(state: &GatewayState, headers: &HeaderMap) -> bool {
    // Fail-closed: brak skonfigurowanego tokenu oznacza brak dostepu.
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

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

async fn gateway_status(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication_required" })),
        )
            .into_response();
    }

    let provider = GatewayProvider::new(
        SharedRunner(Arc::clone(&state.runner)),
        &state.upstream_interface,
        &state.downstream_interface,
        &state.connection_profile,
    );

    match provider.read_status(now_unix_ms()) {
        Ok(status) => (StatusCode::OK, Json(serde_json::json!(status))).into_response(),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "gateway_unreadable", "detail": err.to_string() })),
        )
            .into_response(),
    }
}

/// Wylacznie GET. Zadna metoda mutujaca nie jest tu zarejestrowana i nie wolno jej dodac -
/// sterowanie bramą wymaga osobnej sciezki z autoryzacja zdolnosci.
pub fn gateway_router(state: GatewayState) -> Router {
    Router::new()
        .route("/v1/gateway/status", get(gateway_status))
        .with_state(state)
}
