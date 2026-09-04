// darkstar-header-v1
// po co: context_http.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-02 01:40:00
//! REASON FOR CREATION: Test granicy HTTP kontekstu AIONS - odczyt, fail-closed, brak metod zmieniajacych.
//! MECHANICS: Transport zwraca utrwalone wyjscie zywej bazy z E:. Zadnej sieci, zadnego SSH.
//! SYSTEM PART: Darkstar Server / testy granicy HTTP.
//! ARCHITECTURE FUNCTION: Utrwala kontrakt: milczenie obu nog daje 503, nigdy pusta liste.
//! DEPENDENCIES/LINKS: darkstar-server context_http, darkstar_core::context_client.
//! TECH STACK: Rust 2024 + Axum 0.8 + tower::ServiceExt - wzorzec z tests/gateway_http.rs.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use darkstar_core::context_client::{ContextError, ContextTransport};
use serde_json::Value;
use tower::ServiceExt;

#[allow(dead_code)]
#[path = "../src/context_http.rs"]
mod context_http;

use context_http::{ContextState, context_router};

/// Utrwalone wyjscie zywej bazy na E:, odczytane 2026-09-01.
const COLLECTIONS: &str = r#"[
  {"id":"0b45be0a-3ad5-403a-9fb1-36c74170c0bc","name":"session_AIONS_ARCHITECTURE_MASTERPLAN","dimension":384},
  {"id":"0d20b0b4-9a0b-4aa7-8076-f8773616ec49","name":"session_aions_skills_v4_diag2","dimension":384}
]"#;

struct Fixture {
    up: bool,
}

impl ContextTransport for Fixture {
    fn get(&self, url: &str) -> Result<String, ContextError> {
        if !self.up {
            return Err(ContextError::Unreachable(url.to_string()));
        }
        if url.ends_with("/heartbeat") {
            Ok(r#"{"nanosecond heartbeat":1}"#.into())
        } else {
            Ok(COLLECTIONS.into())
        }
    }
}

fn state(up: bool) -> ContextState {
    ContextState::new(Some(Arc::from("secret")), Arc::new(Fixture { up }))
}

#[tokio::test]
async fn search_returns_matching_collections() {
    let response = context_router(state(true))
        .oneshot(
            Request::builder()
                .uri("/v1/context/search?q=skills&limit=10")
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
    assert_eq!(json["served_by"], "local_cbms");
    assert_eq!(json["collections"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["collections"][0]["name"],
        "session_aions_skills_v4_diag2"
    );
}

#[tokio::test]
async fn both_legs_silent_gives_service_unavailable() {
    let response = context_router(state(false))
        .oneshot(
            Request::builder()
                .uri("/v1/context/search?q=session")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Fail-closed: awaria nie moze wygladac jak brak wiedzy.
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn missing_token_is_refused() {
    let response = context_router(state(true))
        .oneshot(
            Request::builder()
                .uri("/v1/context/search?q=session")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn health_reports_both_legs() {
    let response = context_router(state(true))
        .oneshot(
            Request::builder()
                .uri("/v1/context/health")
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
    assert_eq!(json["remote_e_ok"], true);
    assert_eq!(json["local_cbms_ok"], true);
}

struct UrlSpy {
    urls: std::sync::Mutex<Vec<String>>,
}

impl ContextTransport for UrlSpy {
    fn get(&self, url: &str) -> Result<String, ContextError> {
        self.urls.lock().unwrap().push(url.to_string());
        if url.ends_with("/heartbeat") {
            Ok(r#"{"nanosecond heartbeat":1}"#.into())
        } else {
            Ok("[]".into())
        }
    }
}

#[tokio::test]
async fn both_legs_default_to_chroma_v2_matching_the_live_host() {
    // Zmierzone naprawde na CBMS 2026-09-04: /api/v1/heartbeat -> 410 Gone na obu portach
    // 8000 i 8001, /api/v2/heartbeat -> 200 na obu. Zywy host mowi wylacznie v2, wiec
    // ContextState::new() musi zbudowac OBIE nogi na v2 bez zadnej zmiennej srodowiskowej -
    // to zabezpieczenie przed regresja, nie naprawa istniejacego bledu (bledu nie znaleziono).
    let spy = Arc::new(UrlSpy {
        urls: std::sync::Mutex::new(Vec::new()),
    });
    let response = context_router(ContextState::new(Some(Arc::from("secret")), spy.clone()))
        .oneshot(
            Request::builder()
                .uri("/v1/context/health")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let urls = spy.urls.lock().unwrap();
    assert_eq!(urls.len(), 2, "health() musi zapytac obie nogi");
    for url in urls.iter() {
        assert!(
            url.contains("/api/v2/"),
            "noga zapytala {url} - domyslna konstrukcja ma mowic v2 do obu nog, \
             bo to jedyna wersja, na ktora odpowiada zywy host"
        );
        assert!(
            !url.contains("/api/v1/"),
            "noga zapytala sciezke v1 ({url}) - zywy host zwraca na nia 410 Gone"
        );
    }
}

#[tokio::test]
async fn mutating_methods_are_not_registered() {
    for method in ["POST", "PUT", "DELETE", "PATCH"] {
        let response = context_router(state(true))
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri("/v1/context/search?q=x")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} must not be routed on the context path"
        );
    }
}
