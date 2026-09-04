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
mod world_http;

use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

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

/// Sondy world landing są celowo węższe niż pozostałe API: dwa GET-y i jedno połączenie
/// TCP do dokładnie wskazanej pętli. Nie ma tu komendy procesu ani metody HTTP z zapisem.
struct HostWorldStatusReader;

impl world_http::WorldStatusReader for HostWorldStatusReader {
    fn http_ok(&self, url: &str) -> bool {
        context_http::ReadOnlyHttp.get(url).is_ok()
    }

    fn tcp_open(&self, address: &str) -> bool {
        let Ok(address) = address.parse::<SocketAddr>() else {
            return false;
        };
        if address.ip() != IpAddr::V4(Ipv4Addr::LOCALHOST) {
            return false;
        }
        TcpStream::connect_timeout(&address, Duration::from_secs(3)).is_ok()
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

    let primary: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("DARKSTAR_HOST and DARKSTAR_PORT must form a valid socket address");
    // 0.0.0.0 jest zakazane przez konfiguracje, wiec zakaz stoi TU, w kodzie, a nie
    // wylacznie w komentarzu pliku compose. Pomylka w srodowisku ma zatrzymac start,
    // a nie po cichu wystawic brame na siec nadrzedna.
    assert!(
        !primary.ip().is_unspecified(),
        "DARKSTAR_HOST must name one interface; 0.0.0.0 and :: are refused because they \
         would also expose Darkstar on the upstream Vodafone segment"
    );
    let addresses = bind_addresses(primary);

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
    let headscale_url =
        env::var("DARKSTAR_HEADSCALE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let headscale_health_url = format!("{}/health", headscale_url.trim_end_matches('/'));
    let headscale = headscale_http::headscale_router(headscale_http::HeadscaleState::new(
        state.api_token.clone(),
        std::sync::Arc::new(HeadscaleViaReadOnlyHttp(context_http::ReadOnlyHttp)),
        headscale_url,
        env::var("DARKSTAR_HEADSCALE_APIKEY")
            .ok()
            .filter(|value| !value.is_empty())
            .map(std::sync::Arc::from),
    ));
    let world = world_http::world_status_router(world_http::WorldStatusState::new(
        std::sync::Arc::new(HostWorldStatusReader),
        format!("http://127.0.0.1:{port}/health"),
        headscale_health_url,
        "127.0.0.1:3000",
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
        .merge(world)
        .layer(axum::middleware::from_fn_with_state(
            loopback_state,
            loopback::allow_loopback,
        ));
    tracing::info!(
        addresses = ?addresses,
        api_version = darkstar_core::API_VERSION,
        "darkstar server starting"
    );

    // DWA GNIAZDA, nie jedno przestawione. Petla zwrotna niesie tunel Sterowni z Windows
    // i musi przezyc te zmiane; adres bramy dokladamy OBOK niej, dla telefonu i /world/.
    // Zamiana jednego na drugie zerwalaby tunel, a 0.0.0.0 objeloby tez wlp2s0.
    let mut servers = tokio::task::JoinSet::new();
    let mut live: Vec<SocketAddr> = Vec::new();
    for address in addresses {
        match tokio::net::TcpListener::bind(address).await {
            Ok(listener) => {
                live.push(address);
                let app = app.clone();
                servers.spawn(async move {
                    // Adres drugiej strony polaczenia musi dojechac do warstwy posredniej,
                    // inaczej nie da sie odroznic petli zwrotnej od reszty i wszystko
                    // konczy sie 401.
                    axum::serve(
                        listener,
                        app.into_make_service_with_connect_info::<SocketAddr>(),
                    )
                    .await
                    .expect("serve Darkstar HTTP application");
                });
            }
            // Petla zwrotna jest obowiazkowa: bez niej ginie dostep operatora do maszyny,
            // czyli jedyna droga, ktora pozwala cokolwiek naprawic zdalnie.
            Err(err) if address.ip().is_loopback() => {
                panic!("bind Darkstar loopback listener {address}: {err}");
            }
            // Adres bramy jest najlepszym staraniem. Gdy interfejs jeszcze nie wstal,
            // GLOSNO to zglaszamy, ale NIE zabijamy procesu - inaczej awaria kabla
            // zabralaby przy okazji tunel Sterowni i nie byloby czym diagnozowac.
            Err(err) => {
                tracing::error!(
                    %address,
                    %err,
                    "nie udalo sie stanac na adresie sieci prywatnej - telefon nie zobaczy \
                     /world/; serwer dziala dalej na petli zwrotnej"
                );
            }
        }
    }
    tracing::info!(listening = ?live, "darkstar server listening");

    while let Some(finished) = servers.join_next().await {
        finished.expect("serve Darkstar HTTP application");
    }
}

/// Adresy, na ktorych serwer ma stanac: petla zwrotna ZAWSZE plus adres podany w
/// srodowisku, gdy jest inny. Zwracana kolejnosc jest stabilna, a duplikat znika -
/// `DARKSTAR_HOST=127.0.0.1` ma dawac jedno gniazdo, nie dwa razy to samo.
fn bind_addresses(primary: SocketAddr) -> Vec<SocketAddr> {
    let loopback = SocketAddr::from(([127, 0, 0, 1], primary.port()));
    if primary == loopback {
        return vec![loopback];
    }
    vec![loopback, primary]
}

#[cfg(test)]
mod tests {
    use super::bind_addresses;
    use std::net::SocketAddr;

    #[test]
    fn core_api_version_is_present() {
        assert_eq!(darkstar_core::API_VERSION, "darkstar.core/v1");
    }

    #[test]
    fn lan_bind_keeps_the_loopback_socket_alive() {
        // Tunel Sterowni z Windows wchodzi przez petle zwrotna. Dolozenie adresu bramy
        // NIE moze go zabrac - to jest cala tresc tej zmiany.
        let primary: SocketAddr = "192.168.2.1:18080".parse().expect("valid socket address");

        let addresses = bind_addresses(primary);

        assert_eq!(
            addresses,
            vec![
                "127.0.0.1:18080".parse::<SocketAddr>().expect("loopback"),
                primary,
            ],
            "petla zwrotna musi zostac, a adres bramy dojsc OBOK niej"
        );
    }

    #[test]
    fn loopback_only_configuration_does_not_bind_twice() {
        let primary: SocketAddr = "127.0.0.1:18080".parse().expect("valid socket address");

        let addresses = bind_addresses(primary);

        assert_eq!(addresses, vec![primary], "duplikat gniazda nie ma sensu");
    }

    #[test]
    fn every_bind_address_names_one_interface() {
        // Zaden zwracany adres nie moze byc adresem wszystkich interfejsow: 0.0.0.0
        // objeloby takze wlp2s0, czyli siec nadrzedna Vodafone.
        let primary: SocketAddr = "192.168.2.1:18080".parse().expect("valid socket address");

        for address in bind_addresses(primary) {
            assert!(
                !address.ip().is_unspecified(),
                "{address} jest adresem wszystkich interfejsow"
            );
        }
    }
}
