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
        // Landing /world/ jest osobnym katalogiem w zbudowanym froncie. ServeDir dokleja
        // index.html do katalogu, wiec zapytanie o "/world/" ma trafic w ten plik.
        fs::create_dir_all(root.join("world")).expect("create frontend fixture world directory");
        fs::write(
            root.join("world/index.html"),
            "<!doctype html><title>DARKSTAR WORLD</title>",
        )
        .expect("write frontend fixture world index");
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

/// Landing swiata ma odpowiadac BEZ naglowka Authorization.
///
/// Statyka wisi na `fallback_service`, poza bramka Bearer, ktora kazdy router sprawdza
/// dopiero we wlasnym uchwycie. Ten test jest zamkiem na te wlasnosc: gdyby ktos kiedys
/// zalozyl autoryzacje warstwa na cale drzewo sciezek, landing przestalby dzialac dla
/// przegladarki z sieci lokalnej i test zapali sie tutaj, a nie dopiero na urzadzeniu.
#[tokio::test]
async fn world_landing_is_served_without_a_token() {
    let fixture = FrontendFixture::new();
    let response = router_with_frontend_dist(test_state(), fixture.path())
        .oneshot(
            Request::builder()
                .uri("/world/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/html");
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        body.as_ref(),
        b"<!doctype html><title>DARKSTAR WORLD</title>"
    );
}

/// Ten sam router, to samo zapytanie bez naglowka - i sciezka /v1/* ma dac 401.
///
/// Para z testem wyzej: dowodzi, ze otwarcie landingu NIE otwiera API. Bez ConnectInfo
/// warstwa petli zwrotnej z main.rs tu nie dziala, wiec widac czysty stan bramki.
#[tokio::test]
async fn api_stays_closed_while_world_landing_is_open() {
    let fixture = FrontendFixture::new();
    let response = router_with_frontend_dist(test_state(), fixture.path())
        .oneshot(
            Request::builder()
                .uri("/v1/system-graph")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
