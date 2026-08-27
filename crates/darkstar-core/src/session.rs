//! Session and identity model for authenticated Darkstar clients and agents.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 20:54:30
//! REASON FOR CREATION: Give every connection a durable identity and a narrowly scoped session boundary.
//! MECHANICS: A session binds a client to an owner, capabilities and lifecycle timestamps without granting implicit authority.
//! SYSTEM PART: Darkstar Core / Connection & Session
//! ARCHITECTURE FUNCTION: Carry identity context from authentication through policy, tools and audit.
//! DEPENDENCIES/LINKS: serde, uuid; authentication adapters will populate this model and policy will consume its capabilities.
//! TECH STACK: Rust 2024; selected for explicit state modeling and safe concurrency in the control plane.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Principal {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: Uuid,
    pub principal: Principal,
    pub owner_id: String,
    pub source: String,
    pub created_at_unix_ms: i64,
    pub expires_at_unix_ms: i64,
    pub capabilities: Vec<String>,
}

impl Session {
    pub fn is_expired(&self, now_unix_ms: i64) -> bool {
        now_unix_ms >= self.expires_at_unix_ms
    }

    pub fn permits(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|item| item == capability)
    }
}
