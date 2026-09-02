//! HTTP contract tests for serving the built Darkstar Control Room.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5 Codex
//! TIMESTAMP: 2026-09-02 17:17:58 Europe/London
//! REASON FOR CREATION: Verify that darkstar-server serves the built Control Room entrypoint and static assets without changing API routes.
//! ==========================================

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use darkstar_core::memory::MemoryStore;
use tower::ServiceExt;
use uuid::Uuid;

#[path = "../src/http.rs"]
mod http;

use http::{AppState, router_with_frontend_dist};

struct FrontendFixture {
    root: PathBuf,
}

impl FrontendFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("darkstar-frontend-{}", Uuid::new_v4()));
        fs::create_dir_all(root.join("assets")).expect("create frontend fixture directory");
        fs::write(
            root.join("index.html"),
            "<!doctype html><title>DARKSTAR CONTROL ROOM</title>",
        )
        .expect("write frontend fixture index");
        fs::write(
            root.join("assets/control-room.js"),
            "globalThis.darkstarControlRoom = true;",
        )
        .expect("write frontend fixture asset");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for FrontendFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn test_state() -> AppState {
    let _ = AppState::from_env as fn() -> AppState;
    AppState {
        api_token: Some(Arc::<str>::from("secret")),
        sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        memory: Arc::new(MemoryStore::new()),
        run_streams: Default::default(),
    }
}

#[tokio::test]
async fn root_serves_control_room_html() {
    let fixture = FrontendFixture::new();
    let response = router_with_frontend_dist(test_state(), fixture.path())
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/html");
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        body.as_ref(),
        b"<!doctype html><title>DARKSTAR CONTROL ROOM</title>"
    );
}

#[tokio::test]
async fn assets_path_serves_built_file() {
    let fixture = FrontendFixture::new();
    let response = router_with_frontend_dist(test_state(), fixture.path())
        .oneshot(
            Request::builder()
                .uri("/assets/control-room.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), b"globalThis.darkstarControlRoom = true;");
}
