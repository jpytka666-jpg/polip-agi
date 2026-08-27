//! HTTP transport and first authenticated session boundary for Darkstar.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 21:12:00
//! REASON FOR CREATION: Expose the first real Darkstar HTTP surface while keeping health public and session creation behind an explicit server credential.
//! MECHANICS: Axum routes health/readiness publicly; authenticated clients can create an ephemeral scoped session. Authorization credentials are read from the process environment and are never returned by the API.
//! SYSTEM PART: Darkstar Server / Layer 02 Connection
//! ARCHITECTURE FUNCTION: Provide the transport boundary between remote clients and Darkstar's session/policy core.
//! DEPENDENCIES/LINKS: axum, tokio, serde_json, uuid, darkstar-core session/audit models.
//! TECH STACK: Rust 2024 + Axum 0.8 + Tokio; selected for a small asynchronous HTTP control plane consistent with the Rust-first architecture.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use std::{
    collections::HashMap,
    env,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::IntoResponse,
    routing::{get, post},
};
use darkstar_core::session::{Principal, Session};
use serde::Serialize;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub api_token: Option<Arc<str>>,
    pub sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
}

impl AppState {
    pub fn from_env() -> Self {
        Self {
            api_token: env::var("DARKSTAR_API_TOKEN").ok().map(Arc::<str>::from),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Serialize)]
struct StatusResponse<'a> {
    service: &'a str,
    api_version: &'a str,
    status: &'a str,
}

#[derive(Debug, Serialize)]
struct ReadyResponse<'a> {
    service: &'a str,
    status: &'a str,
    authentication_configured: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateSessionRequest {
    pub principal_id: String,
    #[serde(default = "default_principal_kind")]
    pub principal_kind: String,
    #[serde(default = "default_owner_id")]
    pub owner_id: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

fn default_principal_kind() -> String {
    "agent".into()
}
fn default_owner_id() -> String {
    "unknown".into()
}
fn default_source() -> String {
    "remote".into()
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_millis() as i64
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/v1/sessions", post(create_session))
        .with_state(state)
}

async fn health() -> Json<StatusResponse<'static>> {
    Json(StatusResponse {
        service: "darkstar",
        api_version: darkstar_core::API_VERSION,
        status: "ok",
    })
}

async fn ready(State(state): State<AppState>) -> Json<ReadyResponse<'static>> {
    Json(ReadyResponse {
        service: "darkstar",
        status: "ready",
        authentication_configured: state.api_token.is_some(),
    })
}

async fn create_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "authentication_required"
            })),
        );
    }

    let now = now_unix_ms();
    let session = Session {
        session_id: Uuid::new_v4(),
        principal: Principal {
            kind: request.principal_kind,
            id: request.principal_id,
        },
        owner_id: request.owner_id,
        source: request.source,
        created_at_unix_ms: now,
        expires_at_unix_ms: now + 3_600_000,
        capabilities: request.capabilities,
    };

    let session_id = session.session_id;
    state
        .sessions
        .write()
        .await
        .insert(session_id, session.clone());

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "session": session })),
    )
}

fn authenticated(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(expected) = state.api_token.as_deref() else {
        return false;
    };

    let Some(value) = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };

    value
        .strip_prefix("Bearer ")
        .is_some_and(|token| token == expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_state(token: &str) -> AppState {
        AppState {
            api_token: Some(Arc::<str>::from(token)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[tokio::test]
    async fn health_is_public() {
        let response = router(test_state("secret"))
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn session_creation_requires_bearer_token() {
        let response = router(test_state("secret"))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/sessions")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"principal_id":"agent-1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn valid_token_creates_scoped_session() {
        let response = router(test_state("secret"))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/sessions")
                    .header("authorization", "Bearer secret")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"principal_id":"agent-1","capabilities":["github.read"]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
