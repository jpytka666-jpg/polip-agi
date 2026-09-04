// THIS IS VERY IMPORTANT!!!
// ==========================================
// AUTHOR: M. SZUL
// AI MODEL: GPT-5 Codex
// TIMESTAMP: 2026-09-04 14:08:55
// REASON FOR CREATION: Expose three narrow read-only health probes for the public Darkstar world tiles.
// ==========================================

use std::{collections::BTreeMap, sync::Arc};

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

/// Najmniejsza możliwa granica odczytu: dwa rodzaje sond i żadnej operacji cyklu życia.
pub trait WorldStatusReader {
    fn http_ok(&self, url: &str) -> bool;
    fn tcp_open(&self, address: &str) -> bool;
}

#[derive(Clone)]
pub struct WorldStatusState {
    reader: Arc<dyn WorldStatusReader + Send + Sync>,
    darkstar_health_url: Arc<str>,
    headscale_health_url: Arc<str>,
    headplane_address: Arc<str>,
}

impl WorldStatusState {
    pub fn new(
        reader: Arc<dyn WorldStatusReader + Send + Sync>,
        darkstar_health_url: impl Into<String>,
        headscale_health_url: impl Into<String>,
        headplane_address: impl Into<String>,
    ) -> Self {
        Self {
            reader,
            darkstar_health_url: Arc::from(darkstar_health_url.into()),
            headscale_health_url: Arc::from(headscale_health_url.into()),
            headplane_address: Arc::from(headplane_address.into()),
        }
    }
}

#[derive(Serialize)]
struct ServiceStatus {
    state: &'static str,
    probe: &'static str,
    target: String,
}

#[derive(Serialize)]
struct WorldStatusResponse {
    #[serde(rename = "readOnly")]
    read_only: bool,
    services: BTreeMap<&'static str, ServiceStatus>,
}

fn state(up: bool) -> &'static str {
    if up { "up" } else { "down" }
}

async fn world_status(State(status): State<WorldStatusState>) -> Json<WorldStatusResponse> {
    let mut services = BTreeMap::new();
    services.insert(
        "darkstar",
        ServiceStatus {
            state: state(status.reader.http_ok(&status.darkstar_health_url)),
            probe: "http_get",
            target: status.darkstar_health_url.to_string(),
        },
    );
    services.insert(
        "headscale",
        ServiceStatus {
            state: state(status.reader.http_ok(&status.headscale_health_url)),
            probe: "http_get",
            target: status.headscale_health_url.to_string(),
        },
    );
    services.insert(
        "headplane",
        ServiceStatus {
            state: state(status.reader.tcp_open(&status.headplane_address)),
            probe: "tcp_connect",
            target: status.headplane_address.to_string(),
        },
    );

    Json(WorldStatusResponse {
        read_only: true,
        services,
    })
}

/// Publiczny jest tylko ten GET. Brak rejestracji POST/PUT/PATCH/DELETE daje dla nich 405.
pub fn world_status_router(state: WorldStatusState) -> Router {
    Router::new()
        .route("/v1/world/status", get(world_status))
        .with_state(state)
}
