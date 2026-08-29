//! HTTP contract tests for session-scoped Darkstar memory.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-28 08:45:00
//! REASON FOR CREATION: Define the authenticated HTTP boundary between sessions and the Layer 01 memory store.
//! MECHANICS: Create a session with explicit memory capabilities, write JSON memory, then read it back through the same session.
//! SYSTEM PART: Darkstar Server / Layer 01 + Layer 02
//! ARCHITECTURE FUNCTION: RED-phase contract test for HTTP access to session-scoped memory.
//! DEPENDENCIES/LINKS: existing darkstar-server::http router, darkstar-core::memory, axum, serde_json, uuid.
//! TECH STACK: Rust 2024 integration test; selected to exercise the existing HTTP module without adding a new library target.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi / feat/darkstar-module-control
//! ==========================================

use std::{collections::HashMap, sync::Arc};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use darkstar_core::memory::MemoryStore;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

#[path = "../src/http.rs"]
mod http;

use http::{AppState, router};

fn test_state() -> AppState {
    // Keep the production constructor part of this integration target without reading its environment.
    let _ = AppState::from_env as fn() -> AppState;
    AppState {
        api_token: Some(Arc::<str>::from("secret")),
        sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        memory: Arc::new(MemoryStore::new()),
        run_streams: Default::default(),
    }
}

async fn create_session(app: Router) -> (Router, Uuid) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/sessions")
                .header("authorization", "Bearer secret")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"principal_id":"agent-1","capabilities":["memory.read","memory.write"]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let session_id = json["session"]["session_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    (app, session_id)
}
async fn create_control_session(app: Router) -> (Router, Uuid) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/sessions")
                .header("authorization", "Bearer secret")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"principal_id":"operator","capabilities":["module.start"]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    let session_id = json["session"]["session_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    (app, session_id)
}

#[tokio::test]
async fn session_can_write_and_read_memory() {
    let app = router(test_state());
    let (app, session_id) = create_session(app).await;

    let write = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/v1/sessions/{session_id}/memory/project"))
                .header("authorization", "Bearer secret")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Darkstar"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(write.status(), StatusCode::CREATED);

    let read = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/sessions/{session_id}/memory/project"))
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(read.status(), StatusCode::OK);
}

#[tokio::test]
async fn memory_routes_require_authentication() {
    let app = router(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/v1/sessions/{}/memory/project", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn missing_api_token_fails_closed() {
    let app = router(AppState {
        api_token: None,
        sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        memory: Arc::new(MemoryStore::new()),
        run_streams: Default::default(),
    });
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/system-graph")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn module_action_requires_bearer_token() {
    let response = router(test_state())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/modules/wpc-engine/actions")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"command":"start","reason":"operator requested start"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn module_action_denies_missing_capability() {
    let app = router(test_state());
    let (app, session_id) = create_session(app).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/modules/wpc-engine/actions")
                .header("authorization", "Bearer secret")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"session_id":"{}","command":"start","reason":"operator requested start"}}"#,
                    session_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn module_action_accepts_authorized_start_request() {
    let app = router(test_state());
    let (app, session_id) = create_control_session(app).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/modules/wpc-engine/actions")
                .header("authorization", "Bearer secret")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"session_id":"{}","command":"start","reason":"operator requested start"}}"#,
                    session_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
