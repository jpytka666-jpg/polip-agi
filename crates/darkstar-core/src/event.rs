//! Common event envelope shared by sessions, plugins, workflows and audit.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 20:54:30
//! REASON FOR CREATION: Create one durable, attributable event shape for all Darkstar activity.
//! MECHANICS: Every event carries an ID, type, actor, session and JSON payload so transports and storage can remain independent.
//! SYSTEM PART: Darkstar Core / Event Bus
//! ARCHITECTURE FUNCTION: Provide the common language between memory, security, agents, plugins and automation.
//! DEPENDENCIES/LINKS: serde, serde_json, uuid; future persistence and transport layers consume this envelope.
//! TECH STACK: Rust 2024; selected for strongly typed control-plane state with JSON interoperability.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActorRef {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkstarEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub actor: ActorRef,
    pub session_id: Option<Uuid>,
    pub timestamp_unix_ms: i64,
    pub payload: serde_json::Value,
}

impl DarkstarEvent {
    pub fn new(
        event_type: impl Into<String>,
        actor: ActorRef,
        session_id: Option<Uuid>,
        timestamp_unix_ms: i64,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            actor,
            session_id,
            timestamp_unix_ms,
            payload,
        }
    }
}
