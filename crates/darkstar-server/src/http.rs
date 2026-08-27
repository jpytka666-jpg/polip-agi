//! HTTP transport and live execution boundary for Darkstar.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:45:00
//! REASON FOR CREATION: Expose authenticated control routes plus a live, read-only execution stream for the browser System Graph.
//! MECHANICS: Axum serves health, sessions, architecture inspection and run events. A Tokio broadcast hub carries structured run events; the demo runner only exercises the visualization path and does not execute external tools.
//! SYSTEM PART: Darkstar Server / Layer 02 Connection + Live Run Graph
//! ARCHITECTURE FUNCTION: Transport boundary between clients, Darkstar control-plane state and the human-facing execution observer.
//! DEPENDENCIES/LINKS: axum, tokio, tokio-stream, serde_json, uuid, darkstar-core session/system_graph models, system_graph_view, run_stream.
//! TECH STACK: Rust 2024 + Axum 0.8 + Tokio; selected for a small asynchronous control plane and native server-sent events.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-system-graph
//! ==========================================

#[path = "run_stream.rs"]
mod run_stream;
#[path = "system_graph_view.rs"]
mod system_graph_view;

use std::{
    collections::HashMap,
    env,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::{Html, IntoResponse, Sse, sse::Event},
    routing::{get, post},
};
use darkstar_core::session::{Principal, Session};
use run_stream::{RunEvent, RunStreamHub};
use serde::Serialize;
use tokio::{spawn, time::sleep};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub api_token: Option<Arc<str>>,
    pub sessions: Arc<tokio::sync::RwLock<HashMap<Uuid, Session>>>,
    pub run_streams: RunStreamHub,
}

impl AppState {
    pub fn from_env() -> Self {
        Self {
            api_token: env::var("DARKSTAR_API_TOKEN").ok().map(Arc::<str>::from),
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            run_streams: RunStreamHub::default(),
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

#[derive(Debug, serde::Deserialize)]
struct StartRunRequest {
    pub run_id: Uuid,
}

#[derive(Debug, Serialize)]
struct StartRunResponse {
    run_id: Uuid,
    status: &'static str,
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
        .route("/system-graph", get(system_graph_page))
        .route("/v1/system-graph", get(system_graph_json))
        .route("/v1/runs/start", post(start_demo_run))
        .route("/v1/runs/{run_id}/events", get(run_events))
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

async fn system_graph_page() -> Html<&'static str> {
    Html(system_graph_view::SYSTEM_GRAPH_HTML)
}

async fn system_graph_json(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication_required" })),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(serde_json::json!(
            darkstar_core::system_graph::current_snapshot()
        )),
    )
        .into_response()
}

async fn start_demo_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<StartRunRequest>,
) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication_required" })),
        )
            .into_response();
    }

    let run_id = request.run_id;
    let hub = state.run_streams.clone();
    spawn(async move {
        let steps = [
            (
                "session",
                "session.created",
                "running",
                "Session established",
            ),
            (
                "policy",
                "policy.checked",
                "allow",
                "Policy allowed github.read",
            ),
            (
                "capability-selector",
                "capability.selected",
                "selected",
                "Selected github.read",
            ),
            (
                "capability-gate",
                "authorization.allowed",
                "allow",
                "Authorization allowed",
            ),
            (
                "echo-python",
                "plugin.started",
                "running",
                "Python tentacle started",
            ),
            (
                "echo-python",
                "plugin.completed",
                "success",
                "Python tentacle returned",
            ),
            (
                "audit",
                "audit.appended",
                "recorded",
                "Audit record appended",
            ),
        ];
        for (sequence, (node_id, event_type, status, message)) in steps.into_iter().enumerate() {
            sleep(Duration::from_millis(250)).await;
            hub.publish(RunEvent {
                run_id,
                sequence: sequence as u64 + 1,
                node_id: node_id.into(),
                event_type: event_type.into(),
                status: status.into(),
                message: Some(message.into()),
                timestamp_unix_ms: now_unix_ms(),
            })
            .await;
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(StartRunResponse {
            run_id,
            status: "started",
        }),
    )
        .into_response()
}

async fn run_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication_required" })),
        )
            .into_response();
    }

    let receiver = state.run_streams.subscribe(run_id).await;
    let stream = BroadcastStream::new(receiver).filter_map(|item| {
        let event = item.ok()?;
        let json = serde_json::to_string(&event).ok()?;
        Some(Ok::<Event, std::convert::Infallible>(
            Event::default().data(json),
        ))
    });

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}

async fn create_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    if !authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication_required" })),
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
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            run_streams: RunStreamHub::default(),
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
    async fn system_graph_requires_bearer_token() {
        let response = router(test_state("secret"))
            .oneshot(
                Request::builder()
                    .uri("/v1/system-graph")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn system_graph_is_available_with_valid_token() {
        let response = router(test_state("secret"))
            .oneshot(
                Request::builder()
                    .uri("/v1/system-graph")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn demo_run_requires_bearer_token() {
        let response = router(test_state("secret"))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/runs/start")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"run_id":"{}"}}"#, Uuid::new_v4())))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn valid_token_starts_demo_run() {
        let run_id = Uuid::new_v4();
        let response = router(test_state("secret"))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/runs/start")
                    .header("authorization", "Bearer secret")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"run_id":"{}"}}"#, run_id)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn live_events_require_bearer_token() {
        let response = router(test_state("secret"))
            .oneshot(
                Request::builder()
                    .uri(format!("/v1/runs/{}/events", Uuid::new_v4()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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
