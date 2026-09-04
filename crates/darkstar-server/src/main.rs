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
mod headscale_http;
mod http;
mod loopback;

use std::{env, net::SocketAddr};

use darkstar_core::context_client::ContextTransport;
use headscale_http::{HeadscaleError, HeadscaleReader};
use http::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Odczyt Headscale oparty o klienta, ktory juz stoi w warstwie kontekstu.
///
/// Swiadomie NIE powstaje tu drugi klient HTTP. `ReadOnlyHttp` umie wylacznie GET bez
/// naglowkow i to wystarcza na `/health`, ktore Headscale wystawia bez autoryzacji.
/// Lista wezlow wymaga klucza API w naglowku, wiec dopoki klucza nie ma, ta sciezka
/// zwraca blad zamiast udawac, ze odczytala pusta liste - `read_nodes` i tak jej nie
/// wola, poki DARKSTAR_HEADSCALE_APIKEY nie jest ustawione.
struct HeadscaleViaReadOnlyHttp(context_http::ReadOnlyHttp);

impl HeadscaleReader for HeadscaleViaReadOnlyHttp {
    fn get(&self, url: &str, api_key: Option<&str>) -> Result<String, HeadscaleError> {
        if api_key.is_some() {
            return Err(HeadscaleError::new(
                "klucz API wymaga klienta z naglowkami - jeszcze niepodpiety",
            ));
        }
        self.0
            .get(url)
            .map_err(|err| HeadscaleError::new(err.to_string()))
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Adres nasluchu. Domyslna wartosc to brama prywatnej sieci DARKSTAR-WiFi, a NIE
    // 0.0.0.0: pominiecie zmiennej ma dawac adres waski, nie adres wszystkich interfejsow.
    // 0.0.0.0 wystawiloby usluge takze na wlp2s0, czyli do sieci nadrzednej Vodafone.
    let host = env::var("DARKSTAR_HOST").unwrap_or_else(|_| "192.168.2.1".into());
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
    // Prywatny mesh OBOK dzialajacego Tailscale, nigdy zamiast niego. Adres domyslny to
    // petla zwrotna - Headscale nie jest tu nigdy szukany pod adresem publicznym.
    let headscale_url = env::var("DARKSTAR_HEADSCALE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let headscale = headscale_http::headscale_router(headscale_http::HeadscaleState::new(
        state.api_token.clone(),
        std::sync::Arc::new(HeadscaleViaReadOnlyHttp(context_http::ReadOnlyHttp)),
        headscale_url,
        env::var("DARKSTAR_HEADSCALE_APIKEY")
            .ok()
            .filter(|value| !value.is_empty())
            .map(std::sync::Arc::from),
    ));
    // Zdjecie zamka dla lokalnego operatora. Warstwa jest JEDNA i obejmuje cale drzewo
    // sciezek: zapytanie z petli zwrotnej bez wlasnego naglowka dostaje doklejony token.
    // Adres spoza petli przechodzi nietkniety, wiec konczy sie 401 tak jak wczesniej.
    // Przywrocenie zamka to usuniecie tej jednej warstwy.
    let loopback_state = loopback::LoopbackState::new(state.api_token.clone());
    let app = http::router(state)
        .merge(gateway)
        .merge(context)
        .merge(git)
        .merge(headscale)
        .layer(axum::middleware::from_fn_with_state(
            loopback_state,
            loopback::allow_loopback,
        ));
    tracing::info!(%address, api_version = darkstar_core::API_VERSION, "darkstar server starting");

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("bind Darkstar listener");
    // Adres drugiej strony polaczenia musi dojechac do warstwy posredniej, inaczej
    // nie da sie odroznic petli zwrotnej od reszty i wszystko konczy sie 401.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
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
