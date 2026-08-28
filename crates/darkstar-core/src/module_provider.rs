//! Provider contract for executing authorized module lifecycle commands.
//!
//! SYSTEM PART: Darkstar Core / Provider Boundary
//! ARCHITECTURE FUNCTION: Keep orchestration independent from systemd, Docker, Kali, Azure, Windows or other runtimes while making authorization a type boundary.
//! SECURITY: Providers never decide authorization; ModuleProvider::apply accepts only AuthorizedModuleCommand values created after Darkstar policy allows the request.
//! TECH STACK: Rust 2024 + serde + uuid + typed errors.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::module_state::{ModuleCommand, ModuleCommandRequest, ModuleState};
use crate::policy::{
    ActionRisk, ApprovalState, AuthorizationDecision, AuthorizationRequest, authorize,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderContext {
    pub request_id: String,
    pub principal_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizedModuleCommand {
    pub request_id: Uuid,
    pub module_id: String,
    pub command: ModuleCommand,
    pub capability: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderResult {
    pub provider_id: String,
    pub module_id: String,
    pub command: ModuleCommand,
    pub resulting_state: ModuleState,
    pub message: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProviderError {
    #[error("provider does not support module command")]
    Unsupported,
    #[error("provider rejected command: {0}")]
    Rejected(String),
    #[error("provider unavailable: {0}")]
    Unavailable(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorizationFailure {
    pub decision: AuthorizationDecision,
}

impl AuthorizationFailure {
    fn new(decision: AuthorizationDecision) -> Self {
        Self { decision }
    }
}

pub fn capability_for_command(command: ModuleCommand) -> &'static str {
    match command {
        ModuleCommand::Start => "module.start",
        ModuleCommand::Stop => "module.stop",
        ModuleCommand::Restart => "module.restart",
    }
}

pub fn authorize_module_command(
    session_capabilities: &[String],
    request: &ModuleCommandRequest,
    approval: ApprovalState,
) -> Result<AuthorizedModuleCommand, AuthorizationFailure> {
    let capability = capability_for_command(request.command.clone());
    let authorization = AuthorizationRequest {
        request_id: Uuid::new_v4(),
        capability: capability.into(),
        risk: ActionRisk::Execute,
    };

    match authorize(session_capabilities, &authorization, approval) {
        AuthorizationDecision::Allow => Ok(AuthorizedModuleCommand {
            request_id: authorization.request_id,
            module_id: request.module_id.clone(),
            command: request.command.clone(),
            capability: capability.into(),
            reason: request.reason.clone(),
        }),
        decision => Err(AuthorizationFailure::new(decision)),
    }
}

pub trait ModuleProvider: Send + Sync {
    fn provider_id(&self) -> &str;

    fn supports(&self, request: &AuthorizedModuleCommand) -> bool;

    fn apply(
        &self,
        request: &AuthorizedModuleCommand,
        context: &ProviderContext,
    ) -> Result<ProviderResult, ProviderError>;
}

#[derive(Debug, Default)]
pub struct DryRunProvider;

impl ModuleProvider for DryRunProvider {
    fn provider_id(&self) -> &str {
        "dry-run"
    }

    fn supports(&self, _request: &AuthorizedModuleCommand) -> bool {
        true
    }

    fn apply(
        &self,
        request: &AuthorizedModuleCommand,
        context: &ProviderContext,
    ) -> Result<ProviderResult, ProviderError> {
        let resulting_state = match request.command {
            ModuleCommand::Start => ModuleState::Ready,
            ModuleCommand::Stop => ModuleState::Offline,
            ModuleCommand::Restart => ModuleState::Ready,
        };

        Ok(ProviderResult {
            provider_id: self.provider_id().into(),
            module_id: request.module_id.clone(),
            command: request.command.clone(),
            resulting_state,
            message: format!("dry-run {}: {}", context.request_id, request.reason),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ModuleCommandRequest {
        ModuleCommandRequest {
            module_id: "wpc-engine".into(),
            command: ModuleCommand::Start,
            reason: "test".into(),
        }
    }

    #[test]
    fn execute_is_authorized_before_reaching_provider() {
        let authorized =
            authorize_module_command(&["module.start".into()], &request(), ApprovalState::Granted)
                .expect("policy should authorize");

        assert_eq!(authorized.capability, "module.start");
        assert_eq!(authorized.module_id, "wpc-engine");
    }

    #[test]
    fn missing_capability_cannot_create_authorized_command() {
        let failure = authorize_module_command(&[], &request(), ApprovalState::Granted)
            .expect_err("policy must deny missing capability");
        assert_eq!(failure.decision, AuthorizationDecision::Deny);
    }

    #[test]
    fn execute_without_approval_cannot_create_authorized_command() {
        let failure =
            authorize_module_command(&["module.start".into()], &request(), ApprovalState::Pending)
                .expect_err("policy must require approval");
        assert_eq!(failure.decision, AuthorizationDecision::NeedsApproval);
    }
}
