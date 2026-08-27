//! Structured audit records for security-sensitive Darkstar operations.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 20:54:30
//! REASON FOR CREATION: Preserve an attributable record of proposals, authorization decisions and executions.
//! MECHANICS: Audit records reference actor, session, action, decision and outcome without forcing the core to own storage.
//! SYSTEM PART: Darkstar Core / Audit
//! ARCHITECTURE FUNCTION: Provide traceability across agents, plugins, workflows and external side effects.
//! DEPENDENCIES/LINKS: serde, serde_json, uuid; future storage adapters persist these records.
//! TECH STACK: Rust 2024; selected for strong data modeling and safe concurrent infrastructure.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub record_id: Uuid,
    pub timestamp_unix_ms: i64,
    pub session_id: Option<Uuid>,
    pub actor_id: String,
    pub action: String,
    pub decision: String,
    pub outcome: Option<String>,
    pub metadata: serde_json::Value,
}

impl AuditRecord {
    pub fn new(
        timestamp_unix_ms: i64,
        session_id: Option<Uuid>,
        actor_id: impl Into<String>,
        action: impl Into<String>,
        decision: impl Into<String>,
        outcome: Option<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            record_id: Uuid::new_v4(),
            timestamp_unix_ms,
            session_id,
            actor_id: actor_id.into(),
            action: action.into(),
            decision: decision.into(),
            outcome,
            metadata,
        }
    }
}
