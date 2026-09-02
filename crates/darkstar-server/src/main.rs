// darkstar-header-v1
// po co: main.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
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

mod context_http;
mod gateway_http;
mod git_http;
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

    let state = AppState::from_env();
    // Odczyt stanu bramy jest osobnym routerem z wlasnym stanem: wystawia wylacznie
    // GET, wiec dolaczenie go nie dodaje zadnej sciezki zmieniajacej siec.
    let gateway = gateway_http::gateway_router(gateway_http::GatewayState::new(
        state.api_token.clone(),
        std::sync::Arc::new(gateway_http::NmcliRunner),
    ));
    let context = context_http::context_router(context_http::ContextState::new(
        state.api_token.clone(),
        std::sync::Arc::new(context_http::ReadOnlyHttp),
    ));
    // Worktree do odczytu przez gita. W kontenerze repozytorium jest podmontowane, wiec
    // sciezka przychodzi ze srodowiska - nigdy nie jest wpisana w kod.
    let git_worktree = env::var("DARKSTAR_GIT_WORKTREE").unwrap_or_else(|_| ".".into());
    let git = git_http::git_router(git_http::GitState::new(
        state.api_token.clone(),
        std::sync::Arc::new(git_http::SystemGitRunner::new(&git_worktree)),
    ));
    let app = http::router(state)
        .merge(gateway)
        .merge(context)
        .merge(git);
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
