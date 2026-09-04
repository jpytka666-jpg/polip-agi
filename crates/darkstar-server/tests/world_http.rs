// darkstar-header-v1
// po co: world_http.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-04
// THIS IS VERY IMPORTANT!!!
// ==========================================
// AUTHOR: M. SZUL
// AI MODEL: GPT-5 Codex
// TIMESTAMP: 2026-09-04 14:08:55
// REASON FOR CREATION: Pin the public, read-only world status contract before implementing live tiles.
// ==========================================

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

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

struct RequiresRuntimeProgress {
    progressed: Arc<AtomicBool>,
}

impl WorldStatusReader for RequiresRuntimeProgress {
    fn http_ok(&self, _url: &str) -> bool {
        let deadline = Instant::now() + Duration::from_millis(250);
        while !self.progressed.load(Ordering::SeqCst) && Instant::now() < deadline {
            std::thread::yield_now();
        }
        self.progressed.load(Ordering::SeqCst)
    }

    fn tcp_open(&self, _address: &str) -> bool {
        true
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

#[tokio::test(flavor = "current_thread")]
async fn blocking_probes_do_not_starve_the_request_executor() {
    let progressed = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&progressed);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        signal.store(true, Ordering::SeqCst);
    });

    let response = world_status_router(WorldStatusState::new(
        Arc::new(RequiresRuntimeProgress { progressed }),
        "http://127.0.0.1:18080/health",
        "http://192.168.2.1:8080/health",
        "127.0.0.1:3000",
    ))
    .oneshot(
        Request::builder()
            .uri("/v1/world/status")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    let json = body_json(response).await;
    assert_eq!(json["services"]["darkstar"]["state"], "up");
    assert_eq!(json["services"]["headscale"]["state"], "up");
}

#[test]
fn headplane_probe_target_is_pinned_to_loopback_never_all_interfaces() {
    // Task 14, Step 14.10: bramka uprawnien. main.rs wiaze sonde Headplane na sztywno,
    // nie ze zmiennej srodowiskowej (w przeciwienstwie do Darkstar i Headscale powyzej
    // w tym samym pliku) - wiec zaden blad konfiguracji hosta nie moze jej przesunac.
    // Ten test pilnuje tego samego literalu, ktorego uzywa `state()` powyzej.
    let main_rs = include_str!("../src/main.rs");
    assert!(
        main_rs.contains("\"127.0.0.1:3000\""),
        "headplane probe target must stay pinned to loopback 127.0.0.1:3000 in main.rs"
    );
    assert!(
        !main_rs.contains("0.0.0.0:3000") && !main_rs.contains("\"0.0.0.0\", \"3000\""),
        "headplane probe target must never become 0.0.0.0 - that would claim to check \
         a listener reachable from the upstream Vodafone segment"
    );
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
