//! Darkstar server entrypoint.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 21:12:00
//! REASON FOR CREATION: Start the first network-facing Darkstar runtime around the stable core contracts.
//! MECHANICS: Reads bind configuration from the environment, creates the HTTP application state, and serves health, readiness, and authenticated session routes.
//! SYSTEM PART: Darkstar Server / Runtime
//! ARCHITECTURE FUNCTION: Entry point connecting the Rust control plane to HTTP transport without placing business logic in the executable entrypoint.
//! DEPENDENCIES/LINKS: darkstar-core, darkstar-server::http, axum, tokio, tracing-subscriber.
//! TECH STACK: Rust 2024 + Axum 0.8 + Tokio; selected for a small, safe asynchronous control-plane server.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

mod http;

use std::{env, net::SocketAddr};

use http::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = env::var("DARKSTAR_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("DARKSTAR_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let address: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("DARKSTAR_HOST and DARKSTAR_PORT must form a valid socket address");

    let app = http::router(AppState::from_env());
    tracing::info!(%address, api_version = darkstar_core::API_VERSION, "darkstar server starting");

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("bind Darkstar listener");
    axum::serve(listener, app)
        .await
        .expect("serve Darkstar HTTP application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn core_api_version_is_present() {
        assert_eq!(darkstar_core::API_VERSION, "darkstar.core/v1");
    }
}
