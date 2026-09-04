// darkstar-header-v1
// po co: loopback.rs
// nie wolno: wiazac na 0.0.0.0, wpuszczac adresu spoza petli zwrotnej, ruszac sieci
// Autor: Marcin Szul
// Model: Claude
// Czas: 2026-09-02T22:25:10+01:00
// Po co: Wpuszcza zapytanie przychodzace z petli zwrotnej bez naglowka Authorization,
//        zostawiajac 401 dla kazdego innego adresu.
//
//! MECHANICS: Jedna warstwa posrednia przed calym drzewem sciezek. Czyta adres drugiej
//! strony polaczenia z ConnectInfo. Gdy adres jest petla zwrotna, a zapytanie nie ma
//! wlasnego naglowka Authorization, warstwa dokleja skonfigurowany token. Dalej nic sie
//! nie zmienia - kazdy router sprawdza token dokladnie tak, jak sprawdzal wczesniej.
//! Dzieki temu zdjecie zamka jest w JEDNYM miejscu i przywrocenie go to usuniecie tej
//! warstwy, a nie przepisywanie autoryzacji w czterech modulach.
//! SYSTEM PART: Darkstar Server / HTTP boundary - warstwa wejscia.
//! ARCHITECTURE FUNCTION: Granica zaufania. Serwer wiaze sie jawnie na 127.0.0.1 oraz
//! 192.168.2.1, dlatego ta warstwa musi odrozniac adres PEER od adresu listenera.
//! Zapytanie przychodzace z 192.168.2.1 nie jest petla zwrotna i nie moze dostac tokenu.
//! Warstwa nie nadaje innych uprawnien - jedynie zwalnia lokalnego operatora z wpisywania
//! tokenu, gdy polaczenie naprawde przychodzi z 127.0.0.0/8 albo ::1.
//! DEPENDENCIES/LINKS: axum 0.8 (ConnectInfo, middleware::from_fn_with_state); main.rs
//! musi serwowac przez into_make_service_with_connect_info, inaczej adres jest nieznany.
//! TECH STACK: Rust 2024 + Axum 0.8, zgodnie z reszta warstwy HTTP.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderValue, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

#[derive(Clone)]
pub struct LoopbackState {
    pub api_token: Option<Arc<str>>,
}

impl LoopbackState {
    pub fn new(api_token: Option<Arc<str>>) -> Self {
        Self { api_token }
    }
}

/// Petla zwrotna to 127.0.0.0/8 oraz ::1. Adres z sieci lokalnej petla NIE jest.
pub fn is_loopback(address: &SocketAddr) -> bool {
    address.ip().is_loopback()
}

/// Dokleja token wylacznie zapytaniu z petli zwrotnej, ktore samo naglowka nie przynioslo.
///
/// Fail-closed w trzech miejscach: brak ConnectInfo (adres nieznany) nie wpuszcza,
/// adres spoza petli nie wpuszcza, brak skonfigurowanego tokenu nie wpuszcza. Wlasny
/// naglowek zapytania nie jest nadpisywany - bledny token ma dalej konczyc sie 401,
/// takze z petli, zeby dalo sie sprawdzic sam mechanizm autoryzacji.
pub async fn allow_loopback(
    State(state): State<LoopbackState>,
    mut request: Request,
    next: Next,
) -> Response {
    let from_loopback = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| is_loopback(address))
        .unwrap_or(false);

    if from_loopback && !request.headers().contains_key(AUTHORIZATION) {
        if let Some(token) = state.api_token.as_deref() {
            if let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) {
                request.headers_mut().insert(AUTHORIZATION, value);
            }
        }
    }

    next.run(request).await
}
