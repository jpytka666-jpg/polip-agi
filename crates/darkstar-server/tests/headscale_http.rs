// darkstar-header-v1
// po co: headscale_http.rs
// nie wolno: wystawiac metody zmieniajacej mesh, mylic "brak klucza" z "zero wezlow"
// Autor: Marcin Szul
// Model: Claude
// Czas: 2026-09-02T22:25:10+01:00
// Po co: Przypina kontrakt odczytu Headscale - zdrowie, wezly, brak klucza i Headscale
//        nieosiagalny musza dawac cztery rozne, jednoznaczne odpowiedzi.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

#[allow(dead_code)]
#[path = "../src/headscale_http.rs"]
mod headscale_http;

use headscale_http::{HeadscaleError, HeadscaleReader, HeadscaleState, headscale_router};

const BASE: &str = "http://127.0.0.1:8080";

struct Fixture;

impl HeadscaleReader for Fixture {
    fn get(&self, url: &str, api_key: Option<&str>) -> Result<String, HeadscaleError> {
        match (url, api_key) {
            ("http://127.0.0.1:8080/health", None) => Ok(r#"{"status":"pass"}"#.into()),
            ("http://127.0.0.1:8080/api/v1/node", Some("klucz")) => Ok(r#"{"nodes":[
                {"id":"1","name":"darkstar-cbms","online":true},
                {"id":"2","name":"desktop-udi6m9f","online":false}
            ]}"#
            .into()),
            _ => Err(HeadscaleError::new("unexpected call")),
        }
    }
}

struct Down;

impl HeadscaleReader for Down {
    fn get(&self, _url: &str, _api_key: Option<&str>) -> Result<String, HeadscaleError> {
        Err(HeadscaleError::new("host path and connection detail"))
    }
}

fn state(reader: Arc<dyn HeadscaleReader + Send + Sync>, key: Option<&str>) -> HeadscaleState {
    HeadscaleState::new(Some(Arc::from("secret")), reader, BASE, key.map(Arc::from))
}

fn get(uri: &str, authorization: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().uri(uri);
    if let Some(value) = authorization {
        builder = builder.header("authorization", value);
    }
    builder.body(Body::empty()).unwrap()
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn health_and_nodes_are_read_together() {
    let response = headscale_router(state(Arc::new(Fixture), Some("klucz")))
        .oneshot(get("/v1/headscale/health", Some("Bearer secret")))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["reachable"], true);
    assert_eq!(json["health"], "pass");
    assert_eq!(json["baseUrl"], BASE);
    assert_eq!(json["nodeCount"], 2);
    assert_eq!(json["nodes"][0]["name"], "darkstar-cbms");
    assert!(json["nodesReason"].is_null());
}

#[tokio::test]
async fn a_missing_api_key_is_reported_rather_than_faked_as_zero_nodes() {
    let response = headscale_router(state(Arc::new(Fixture), None))
        .oneshot(get("/v1/headscale/health", Some("Bearer secret")))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["health"], "pass");
    assert!(json["nodes"].is_null());
    assert!(json["nodeCount"].is_null());
    assert_eq!(json["nodesReason"], "no_api_key");
}

#[tokio::test]
async fn headscale_down_reports_service_unavailable_without_raw_detail() {
    let response = headscale_router(state(Arc::new(Down), Some("klucz")))
        .oneshot(get("/v1/headscale/health", Some("Bearer secret")))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let json = body_json(response).await;
    assert_eq!(json, serde_json::json!({ "error": "headscale_unavailable" }));
}

#[tokio::test]
async fn the_operator_token_is_still_required() {
    let response = headscale_router(state(Arc::new(Fixture), Some("klucz")))
        .oneshot(get("/v1/headscale/health", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn mutating_methods_are_not_registered() {
    for method in ["POST", "PUT", "DELETE", "PATCH"] {
        let request = Request::builder()
            .method(method)
            .uri("/v1/headscale/health")
            .header("authorization", "Bearer secret")
            .body(Body::empty())
            .unwrap();
        let response = headscale_router(state(Arc::new(Fixture), Some("klucz")))
            .oneshot(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
