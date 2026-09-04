// THIS IS VERY IMPORTANT!!!
// ==========================================
// AUTHOR: M. SZUL
// AI MODEL: GPT-5 Codex
// TIMESTAMP: 2026-09-04 14:08:55
// REASON FOR CREATION: Pin the public, read-only world status contract before implementing live tiles.
// ==========================================

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

#[allow(dead_code)]
#[path = "../src/world_http.rs"]
mod world_http;

use world_http::{WorldStatusReader, WorldStatusState, world_status_router};

struct Fixture;

impl WorldStatusReader for Fixture {
    fn http_ok(&self, url: &str) -> bool {
        matches!(
            url,
            "http://127.0.0.1:18080/health" | "http://192.168.2.1:8080/health"
        )
    }

    fn tcp_open(&self, address: &str) -> bool {
        address == "127.0.0.1:3000"
    }
}

fn state() -> WorldStatusState {
    WorldStatusState::new(
        Arc::new(Fixture),
        "http://127.0.0.1:18080/health",
        "http://192.168.2.1:8080/health",
        "127.0.0.1:3000",
    )
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn public_get_reports_three_fresh_read_only_probes() {
    let response = world_status_router(state())
        .oneshot(
            Request::builder()
                .uri("/v1/world/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["readOnly"], true);
    assert_eq!(json["services"]["darkstar"]["state"], "up");
    assert_eq!(json["services"]["headscale"]["state"], "up");
    assert_eq!(json["services"]["headplane"]["state"], "up");
    assert_eq!(json["services"]["headplane"]["target"], "127.0.0.1:3000");
}

#[tokio::test]
async fn mutating_methods_are_not_registered() {
    for method in ["POST", "PUT", "PATCH", "DELETE"] {
        let response = world_status_router(state())
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri("/v1/world/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
