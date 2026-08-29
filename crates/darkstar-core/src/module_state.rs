//! Module lifecycle state contracts for the Darkstar control plane.
//!
//! SYSTEM PART: Darkstar Core / Module Control
//! ARCHITECTURE FUNCTION: Shared state model for AIONS ecosystem modules.
//! TECH STACK: Rust 2024 + serde.

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

impl ModuleCommand {
    pub fn capability(self) -> &'static str {
        match self {
            Self::Start => "module.start",
            Self::Stop => "module.stop",
            Self::Restart => "module.restart",
        }
    }
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
    fn command_round_trips_as_json() {
        let request = ModuleCommandRequest {
            module_id: "wpc-engine".into(),
            command: ModuleCommand::Start,
            reason: "model analysis".into(),
        };
        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: ModuleCommandRequest = serde_json::from_str(&encoded).unwrap();
        assert_eq!(request, decoded);
    }
    #[test]
    fn commands_expose_control_capabilities() {
        assert_eq!(ModuleCommand::Start.capability(), "module.start");
        assert_eq!(ModuleCommand::Stop.capability(), "module.stop");
        assert_eq!(ModuleCommand::Restart.capability(), "module.restart");
    }
}
