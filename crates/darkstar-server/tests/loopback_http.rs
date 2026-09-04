// darkstar-header-v1
// po co: loopback_http.rs
// nie wolno: wpuszczac adresu spoza petli zwrotnej, nadpisywac cudzego naglowka
// Autor: Marcin Szul
// Model: Claude
// Czas: 2026-09-02T22:25:10+01:00
// Po co: Przypina kontrakt otwartej petli zwrotnej - lokalny operator wchodzi bez naglowka,
//        kazdy inny adres nadal dostaje 401.

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode, header::AUTHORIZATION},
    middleware::from_fn_with_state,
    routing::get,
};
use tower::ServiceExt;

#[allow(dead_code)]
#[path = "../src/loopback.rs"]
mod loopback;

use loopback::{LoopbackState, allow_loopback};

const TOKEN: &str = "secret";

/// Zastepnik prawdziwego routera: sprawdza token dokladnie tak, jak robia to moduly
/// bramy, kontekstu i gita - czyli fail-closed na dokladne dopasowanie.
async fn guarded(headers: axum::http::HeaderMap) -> StatusCode {
    let ok = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token == TOKEN)
        .unwrap_or(false);
    if ok {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}

fn app(api_token: Option<&str>) -> Router {
    let state = LoopbackState::new(api_token.map(Arc::from));
    Router::new()
        .route("/v1/gateway/status", get(guarded))
        .layer(from_fn_with_state(state, allow_loopback))
}

/// Buduje zapytanie z zadanym adresem drugiej strony polaczenia.
fn request_from(address: Option<SocketAddr>, authorization: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().uri("/v1/gateway/status");
    if let Some(value) = authorization {
        builder = builder.header(AUTHORIZATION, value);
    }
    let mut request = builder.body(Body::empty()).unwrap();
    if let Some(address) = address {
        request.extensions_mut().insert(ConnectInfo(address));
    }
    request
}

fn loopback_address() -> SocketAddr {
    SocketAddr::from((Ipv4Addr::LOCALHOST, 51234))
}

fn downstream_address() -> SocketAddr {
    SocketAddr::from((Ipv4Addr::new(192, 168, 2, 50), 51234))
}

fn gateway_address() -> SocketAddr {
    SocketAddr::from((Ipv4Addr::new(192, 168, 2, 1), 51234))
}

#[tokio::test]
async fn loopback_without_authorization_is_allowed() {
    let response = app(Some(TOKEN))
        .oneshot(request_from(Some(loopback_address()), None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn address_outside_loopback_still_needs_the_token() {
    let response = app(Some(TOKEN))
        .oneshot(request_from(Some(downstream_address()), None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn gateway_address_does_not_receive_the_loopback_token() {
    // Curl hosta do jego wlasnego listenera 192.168.2.1 przychodzi z adresem peer
    // 192.168.2.1. To nadal NIE jest petla zwrotna i bez jawnego tokenu ma dostac 401.
    let response = app(Some(TOKEN))
        .oneshot(request_from(Some(gateway_address()), None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_wrong_token_from_loopback_is_not_replaced() {
    let response = app(Some(TOKEN))
        .oneshot(request_from(
            Some(loopback_address()),
            Some("Bearer nieprawda"),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_unknown_peer_address_is_not_trusted() {
    let response = app(Some(TOKEN))
        .oneshot(request_from(None, None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn without_a_configured_token_loopback_opens_nothing() {
    let response = app(None)
        .oneshot(request_from(Some(loopback_address()), None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
