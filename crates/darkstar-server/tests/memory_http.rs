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

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

#[path = "../src/http.rs"]
mod http;

use http::{AppState, router};

fn test_state() -> AppState {
    AppState::from_env()
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
