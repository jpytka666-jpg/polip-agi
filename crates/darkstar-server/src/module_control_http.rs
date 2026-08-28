//! Authenticated HTTP boundary for Darkstar module lifecycle requests.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-28 10:30:00
//! REASON FOR CREATION: Provide the browser-facing request boundary for authorized Start, Stop and Restart module actions.
//! MECHANICS: Authenticate the API request, resolve the referenced session, verify the command capability through the existing Darkstar policy path, and return a typed authorization result. This v1 boundary does not execute infrastructure.
//! SYSTEM PART: Darkstar Server / Layer 02 Connection + Module Control
//! ARCHITECTURE FUNCTION: Translate a browser module-action request into an authorized Darkstar command without allowing the browser to reach infrastructure directly.
//! DEPENDENCIES/LINKS: axum, darkstar-core::module_execution, darkstar-core::module_state, darkstar-core::policy, darkstar-server::http::AppState.
//! TECH STACK: Rust 2024 + Axum 0.8; selected to remain inside the existing Darkstar asynchronous server and preserve the trusted control-plane boundary.
//! LOCAL WORKSPACE: /home/owner/polip-agi
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-module-control
//! ==========================================

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use darkstar_core::{
    module_execution::authorize_request,
    module_state::{ModuleCommand, ModuleCommandRequest},
    policy::ApprovalState,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::AppState;

#[derive(Debug, Deserialize)]
pub struct ModuleActionRequest {
    pub session_id: Uuid,
    pub command: ModuleCommand,
    pub reason: String,
}

#[derive(Debug, Serialize)]
struct ModuleActionResponse {
    request_id: Uuid,
    session_id: Uuid,
    module_id: String,
    command: ModuleCommand,
    capability: String,
    status: &'static str,
    reason: String,
}

pub async fn module_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(module_id): Path<String>,
    body: axum::body::Body,
) -> impl IntoResponse {
    if !crate::http::authenticated(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "authentication_required"
            })),
        )
            .into_response();
    }
    let body = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(body) => body,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_request_body"
                })),
            )
                .into_response();
        }
    };

    let request: ModuleActionRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_request_body"
                })),
            )
                .into_response();
        }
    };

    let session = match state
        .sessions
        .read()
        .await
        .get(&request.session_id)
        .cloned()
    {
        Some(session) => session,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "session_not_found"
                })),
            )
                .into_response();
        }
    };

    let command_request = ModuleCommandRequest {
        module_id: module_id.clone(),
        command: request.command.clone(),
        reason: request.reason.clone(),
    };

    let authorized = match authorize_request(
        &session.capabilities,
        &command_request,
        ApprovalState::Granted,
    ) {
        Ok(command) => command,
        Err(darkstar_core::module_execution::ModuleExecutionError::Authorization(
            darkstar_core::policy::AuthorizationDecision::Deny,
        )) => {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "module_action_denied"
                })),
            )
                .into_response();
        }
        Err(darkstar_core::module_execution::ModuleExecutionError::Authorization(
            darkstar_core::policy::AuthorizationDecision::NeedsApproval,
        )) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "module_action_approval_required"
                })),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "module_action_denied"
                })),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(ModuleActionResponse {
            request_id: authorized.request_id,
            session_id: request.session_id,
            module_id: authorized.module_id,
            command: authorized.command,
            capability: authorized.capability,
            status: "authorized",
            reason: authorized.reason,
        }),
    )
        .into_response()
}
