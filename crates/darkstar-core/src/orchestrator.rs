//! Deterministic planning for module lifecycle commands.
//!
//! SYSTEM PART: Darkstar Core / Orchestrator
//! ARCHITECTURE FUNCTION: Compare current and desired module state and emit only explicit, representable commands. No authorization or external execution occurs here.
//! TECH STACK: Rust 2024 + serde.

use serde::{Deserialize, Serialize};

use crate::module_state::{ModuleCommand, ModuleCommandRequest, ModuleDescriptor, ModuleState};

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

impl PlannedCommand {
    pub fn into_request(self, reason: impl Into<String>) -> ModuleCommandRequest {
        ModuleCommandRequest {
            module_id: self.module_id,
            command: self.command,
            reason: reason.into(),
        }
    }
}

pub fn plan(modules: &[ModuleDescriptor], desired: &[DesiredModuleState]) -> OrchestrationPlan {
    let mut commands = Vec::new();

    for target in desired {
        let Some(module) = modules
            .iter()
            .find(|module| module.module_id == target.module_id)
        else {
            continue;
        };

        let command = match (module.state, target.desired) {
            (ModuleState::Offline, ModuleState::Ready | ModuleState::Running) => {
                Some(ModuleCommand::Start)
            }
            (ModuleState::Ready, ModuleState::Offline) => Some(ModuleCommand::Stop),
            (ModuleState::Running, ModuleState::Ready) => Some(ModuleCommand::Restart),
            (ModuleState::Running, ModuleState::Offline) => Some(ModuleCommand::Stop),
            (ModuleState::Starting | ModuleState::Stopping, _) => None,
            (ModuleState::Failed | ModuleState::Blocked, _) => None,
            _ => None,
        };

        if let Some(command) = command {
            commands.push(PlannedCommand {
                module_id: module.module_id.clone(),
                command,
            });
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
        let plan = plan(
            &[module("wpc", ModuleState::Offline)],
            &[DesiredModuleState {
                module_id: "wpc".into(),
                desired: ModuleState::Ready,
            }],
        );
        assert_eq!(
            plan.commands,
            vec![PlannedCommand {
                module_id: "wpc".into(),
                command: ModuleCommand::Start,
            }]
        );
    }

    #[test]
    fn ready_module_can_be_stopped() {
        let plan = plan(
            &[module("camera", ModuleState::Ready)],
            &[DesiredModuleState {
                module_id: "camera".into(),
                desired: ModuleState::Offline,
            }],
        );
        assert_eq!(plan.commands[0].command, ModuleCommand::Stop);
    }

    #[test]
    fn running_module_uses_restart_to_return_to_ready() {
        let plan = plan(
            &[module("camera", ModuleState::Running)],
            &[DesiredModuleState {
                module_id: "camera".into(),
                desired: ModuleState::Ready,
            }],
        );
        assert_eq!(plan.commands[0].command, ModuleCommand::Restart);
    }

    #[test]
    fn planned_command_converts_to_execution_request() {
        let request = PlannedCommand {
            module_id: "wpc".into(),
            command: ModuleCommand::Start,
        }
        .into_request("orchestrator test");

        assert_eq!(request.module_id, "wpc");
        assert_eq!(request.command, ModuleCommand::Start);
        assert_eq!(request.reason, "orchestrator test");
    }

    #[test]
    fn blocked_module_is_never_activated() {
        let plan = plan(
            &[module("security", ModuleState::Blocked)],
            &[DesiredModuleState {
                module_id: "security".into(),
                desired: ModuleState::Ready,
            }],
        );
        assert!(plan.commands.is_empty());
    }

    #[test]
    fn in_flight_and_failed_states_are_not_replanned() {
        let plan = plan(
            &[
                module("starting", ModuleState::Starting),
                module("stopping", ModuleState::Stopping),
                module("failed", ModuleState::Failed),
            ],
            &[
                DesiredModuleState {
                    module_id: "starting".into(),
                    desired: ModuleState::Ready,
                },
                DesiredModuleState {
                    module_id: "stopping".into(),
                    desired: ModuleState::Offline,
                },
                DesiredModuleState {
                    module_id: "failed".into(),
                    desired: ModuleState::Ready,
                },
            ],
        );
        assert!(plan.commands.is_empty());
    }

    #[test]
    fn unsupported_ready_to_running_transition_is_not_invented() {
        let plan = plan(
            &[module("wpc", ModuleState::Ready)],
            &[DesiredModuleState {
                module_id: "wpc".into(),
                desired: ModuleState::Running,
            }],
        );
        assert!(plan.commands.is_empty());
    }
}
