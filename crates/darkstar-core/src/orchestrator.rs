//! Deterministic planning for module activation and deactivation.
//!
//! SYSTEM PART: Darkstar Core / Orchestrator
//! ARCHITECTURE FUNCTION: Convert desired state into policy-ready commands.
//! TECH STACK: Rust 2024 + serde.

use serde::{Deserialize, Serialize};

use crate::module_state::{ModuleCommand, ModuleDescriptor, ModuleState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DesiredModuleState {
    pub module_id: String,
    pub desired: ModuleState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrchestrationPlan {
    pub commands: Vec<PlannedCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannedCommand {
    pub module_id: String,
    pub command: ModuleCommand,
}

pub fn plan(modules: &[ModuleDescriptor], desired: &[DesiredModuleState]) -> OrchestrationPlan {
    let mut commands = Vec::new();

    for target in desired {
        let Some(module) = modules.iter().find(|module| module.module_id == target.module_id) else {
            continue;
        };

        match (module.state, target.desired) {
            (ModuleState::Offline, ModuleState::Ready | ModuleState::Running) => commands.push(PlannedCommand {
                module_id: module.module_id.clone(),
                command: ModuleCommand::Start,
            }),
            (ModuleState::Ready, ModuleState::Offline | ModuleState::Running) if target.desired == ModuleState::Offline => commands.push(PlannedCommand {
                module_id: module.module_id.clone(),
                command: ModuleCommand::Stop,
            }),
            (ModuleState::Running, ModuleState::Ready | ModuleState::Offline) => commands.push(PlannedCommand {
                module_id: module.module_id.clone(),
                command: ModuleCommand::Stop,
            }),
            (ModuleState::Failed | ModuleState::Blocked, _) => {}
            _ => {}
        }
    }

    OrchestrationPlan { commands }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(id: &str, state: ModuleState) -> ModuleDescriptor {
        ModuleDescriptor {
            module_id: id.into(),
            name: id.into(),
            system: "test".into(),
            language: "rust".into(),
            version: None,
            capabilities: vec![],
            dependencies: vec![],
            state,
            health: None,
        }
    }

    #[test]
    fn offline_module_gets_start_command() {
        let plan = plan(&[module("wpc", ModuleState::Offline)], &[DesiredModuleState {
            module_id: "wpc".into(),
            desired: ModuleState::Ready,
        }]);
        assert_eq!(plan.commands[0].command, ModuleCommand::Start);
    }

    #[test]
    fn running_module_can_be_stopped() {
        let plan = plan(&[module("camera", ModuleState::Running)], &[DesiredModuleState {
            module_id: "camera".into(),
            desired: ModuleState::Offline,
        }]);
        assert_eq!(plan.commands[0].command, ModuleCommand::Stop);
    }

    #[test]
    fn blocked_module_is_never_activated() {
        let plan = plan(&[module("security", ModuleState::Blocked)], &[DesiredModuleState {
            module_id: "security".into(),
            desired: ModuleState::Ready,
        }]);
        assert!(plan.commands.is_empty());
    }
}
