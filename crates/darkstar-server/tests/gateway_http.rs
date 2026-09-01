// darkstar-header-v1
// po co: gateway_http.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-01 22:10:00
//! REASON FOR CREATION: Test granicy HTTP bramy Darkstar (Task 8, Step 8.5 i 8.6).
//! MECHANICS: Buduje router z wykonawca zwracajacym utrwalone wyjscie hosta CBMS. Zadnego SSH,
//! zadnej sieci. Sprawdza tresc odpowiedzi GET, odmowe bez tokenu oraz to, ze metody mutujace
//! nie sa zarejestrowane.
//! SYSTEM PART: Darkstar Server / testy granicy HTTP.
//! ARCHITECTURE FUNCTION: Utrwala kontrakt "tylko odczyt" - test upadnie, jesli ktos dolozy
//! POST, PUT albo DELETE na sciezce bramy.
//! DEPENDENCIES/LINKS: darkstar-server gateway_http, darkstar_core::gateway_provider.
//! TECH STACK: Rust 2024 + Axum 0.8 + tower::ServiceExt::oneshot - wzorzec juz uzywany w
//! tests/memory_http.rs, wiec bez nowej zaleznosci.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use darkstar_core::gateway_provider::{CommandRunner, GatewayProviderError};
use serde_json::Value;
use tower::ServiceExt;

// Modul jest tu kompilowany osobno od binarki, wiec produkcyjny NmcliRunner
// nie ma w tym zakresie wolajacego. To zamierzone: test celowo go nie uruchamia,
// bo wolalby prawdziwe nmcli na hoscie.
#[allow(dead_code)]
#[path = "../src/gateway_http.rs"]
mod gateway_http;

use gateway_http::{GatewayState, gateway_router};

/// Utrwalone wyjscie z hosta CBMS, odczytane 2026-09-01.
struct FixtureRunner;

impl CommandRunner for FixtureRunner {
    fn active_connections(&self) -> Result<String, GatewayProviderError> {
        Ok("DARKSTAR-WiFi:enp1s0:activated\nVodafone480B38 1:wlp2s0:activated\n".into())
    }
    fn interface_addresses(&self, _iface: &str) -> Result<String, GatewayProviderError> {
        Ok("2: enp1s0    inet 192.168.2.1/24 brd 192.168.2.255 scope global noprefixroute enp1s0\n"
            .into())
    }
    fn profile_ipv4(&self, _profile: &str) -> Result<String, GatewayProviderError> {
        Ok("shared\n192.168.2.1/24\n".into())
    }
    fn neighbours(&self, _iface: &str) -> Result<String, GatewayProviderError> {
        Ok("192.168.2.50 lladdr d4:25:8b:d3:79:5d REACHABLE\n\
192.168.1.254 lladdr a0:2d:db:d7:a9:20 STALE\n"
            .into())
    }
}

fn test_state() -> GatewayState {
    GatewayState::new(Some(Arc::from("secret")), Arc::new(FixtureRunner))
}

#[tokio::test]
async fn get_returns_live_gateway_status() {
    let response = gateway_router(test_state())
        .oneshot(
            Request::builder()
                .uri("/v1/gateway/status")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(json["upstream_interface"], "wlp2s0");
    assert_eq!(json["downstream_interface"], "enp1s0");
    assert_eq!(json["downstream_cidr"], "192.168.2.1/24");
    assert_eq!(json["downstream_subnet"], "192.168.2.0/24");
    assert_eq!(json["connection_profile"], "DARKSTAR-WiFi");
    assert_eq!(json["mode"], "ethernet");
    assert_eq!(json["health"], "ready");
    // Windows liczy sie jako klient; hub w innej adresacji nie.
    assert_eq!(json["connected_clients"], 1);
}

#[tokio::test]
async fn missing_token_is_refused() {
    let response = gateway_router(test_state())
        .oneshot(
            Request::builder()
                .uri("/v1/gateway/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unconfigured_token_fails_closed() {
    let state = GatewayState::new(None, Arc::new(FixtureRunner));
    let response = gateway_router(state)
        .oneshot(
            Request::builder()
                .uri("/v1/gateway/status")
                .header("authorization", "Bearer anything")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn mutating_methods_are_not_registered() {
    for method in ["POST", "PUT", "DELETE", "PATCH"] {
        let response = gateway_router(test_state())
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri("/v1/gateway/status")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} must not be routed on the gateway path"
        );
    }
}

#[tokio::test]
async fn unreadable_host_reports_service_unavailable() {
    struct Broken;
    impl CommandRunner for Broken {
        fn active_connections(&self) -> Result<String, GatewayProviderError> {
            Err(GatewayProviderError::CommandFailed("nmcli missing".into()))
        }
        fn interface_addresses(&self, _i: &str) -> Result<String, GatewayProviderError> {
            Ok(String::new())
        }
        fn profile_ipv4(&self, _p: &str) -> Result<String, GatewayProviderError> {
            Ok(String::new())
        }
        fn neighbours(&self, _i: &str) -> Result<String, GatewayProviderError> {
            Ok(String::new())
        }
    }

    let state = GatewayState::new(Some(Arc::from("secret")), Arc::new(Broken));
    let response = gateway_router(state)
        .oneshot(
            Request::builder()
                .uri("/v1/gateway/status")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}
