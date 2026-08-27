//! Module lifecycle state contracts for the Darkstar control plane.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 23:25:00
//! REASON FOR CREATION: Give Darkstar one language-neutral state model for activating, stopping and observing external AIONS ecosystem modules.
//! MECHANICS: A module advertises identity, language, system, capabilities, dependencies and lifecycle state. This file defines state, not process execution.
//! SYSTEM PART: Darkstar Core / Module Control
//! ARCHITECTURE FUNCTION: Common contract consumed by the orchestrator, browser control deck and future local/cloud providers.
//! DEPENDENCIES/LINKS: serde; future orchestrator/provider adapters and system graph projection.
//! TECH STACK: Rust 2024 + serde; selected for a small typed control-plane contract with JSON interoperability.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-control-deck
//! ==========================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleState {
    Offline,
    Starting,
    Ready,
    Running,
    Stopping,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleDescriptor {
    pub module_id: String,
    pub name: String,
    pub system: String,
    pub language: String,
    pub version: Option<String>,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub state: ModuleState,
    pub health: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleCommand {
    Start,
    Stop,
    Restart,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleCommandRequest {
    pub module_id: String,
    pub command: ModuleCommand,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_command_round_trips_as_json() {
        let request = ModuleCommandRequest {
            module_id: "wpc-engine".into(),
            command: ModuleCommand::Start,
            reason: "model analysis".into(),
        };
        let encoded = serde_json::to_string(&request).expect("serialize command");
        let decoded: ModuleCommandRequest = serde_json::from_str(&encoded).expect("deserialize command");
        assert_eq!(request, decoded);
    }

    #[test]
    fn lifecycle_has_explicit_failure_and_blocked_states() {
        assert_ne!(ModuleState::Failed, ModuleState::Blocked);
    }
}
