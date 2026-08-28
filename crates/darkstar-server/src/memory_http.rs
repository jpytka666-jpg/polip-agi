//! Memory HTTP helpers for session-scoped Darkstar memory.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-28 09:10:00
//! REASON FOR CREATION: Keep memory HTTP parsing and response shaping separate from the main router implementation.
//! MECHANICS: Validate the session, enforce a named memory capability, then delegate JSON persistence to the shared in-memory store.
//! SYSTEM PART: Darkstar Server / Layer 01 + Layer 02
//! ARCHITECTURE FUNCTION: Controlled HTTP boundary for session-scoped memory reads and writes.
//! DEPENDENCIES/LINKS: axum, darkstar-core::memory, darkstar-core::session, serde_json, uuid.
//! TECH STACK: Rust 2024 + Axum 0.8; selected for typed request extraction and small async handlers.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi / feat/darkstar-module-control
//! ==========================================

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use darkstar_core::{memory::StoredMemory, session::Session};
use serde_json::Value;
use uuid::Uuid;

use crate::http::AppState;

fn authorized_session(
    state: &AppState,
    headers: &HeaderMap,
    session_id: Uuid,
    capability: &str,
) -> Result<Session, StatusCode> {
    if !crate::http::authenticated(state, headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let sessions = state
        .sessions
        .try_read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let session = sessions
        .get(&session_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    if !session.permits(capability) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(session)
}

pub async fn write_memory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((session_id, key)): Path<(Uuid, String)>,
    Json(value): Json<Value>,
) -> impl IntoResponse {
    if let Err(status) = authorized_session(&state, &headers, session_id, "memory.write") {
        return (
            status,
            Json(serde_json::json!({ "error": "memory_write_denied" })),
        )
            .into_response();
    }

    let now = crate::http::now_unix_ms();
    if state.memory.put(session_id, &key, value, now).is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "memory_store_unavailable" })),
        )
            .into_response();
    }

    match state.memory.get(session_id, &key) {
        Ok(Some(record)) => (StatusCode::CREATED, Json(record)).into_response(),
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "memory_store_unavailable" })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "memory_store_unavailable" })),
        )
            .into_response(),
    }
}

pub async fn read_memory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((session_id, key)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    if let Err(status) = authorized_session(&state, &headers, session_id, "memory.read") {
        return (
            status,
            Json(serde_json::json!({ "error": "memory_read_denied" })),
        )
            .into_response();
    }

    match state.memory.get(session_id, &key) {
        Ok(Some(record)) => (StatusCode::OK, Json(record)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "memory_not_found" })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "memory_store_unavailable" })),
        )
            .into_response(),
    }
}

#[allow(dead_code)]
fn _stored_memory_type_is_public(record: StoredMemory) {
    let _ = record;
}
