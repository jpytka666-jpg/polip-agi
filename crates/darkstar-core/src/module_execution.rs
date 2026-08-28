//! Policy-gated execution boundary for Darkstar module lifecycle commands.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-28 02:00:00
//! REASON FOR CREATION: Provide the single execution path from a module command request through Darkstar policy into a module provider.
//! MECHANICS: A raw ModuleCommandRequest is authorized against session capabilities and trusted approval before it can be converted into an AuthorizedModuleCommand and passed to a provider.
//! SYSTEM PART: Darkstar Core / Module Execution
//! ARCHITECTURE FUNCTION: Physically separate intent from executable authority so providers cannot receive an unauthorised module command.
//! DEPENDENCIES/LINKS: module_state, module_provider, policy, provider implementations; later connected to orchestrator plans and event/audit emission.
//! TECH STACK: Rust 2024 + existing Darkstar typed contracts.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-module-control, PR #6
//! ==========================================

use super::module_provider::{
    AuthorizedModuleCommand, ModuleProvider, ProviderContext, ProviderError,
    authorize_module_command,
};
use super::module_state::ModuleCommandRequest;
use super::policy::{ApprovalState, AuthorizationDecision};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ModuleExecutionError {
    #[error("module command authorization failed: {0:?}")]
    Authorization(AuthorizationDecision),
    #[error("module provider rejected the command")]
    Provider(#[source] ProviderError),
}

pub fn authorize_request(
    session_capabilities: &[String],
    request: &ModuleCommandRequest,
    approval: ApprovalState,
) -> Result<AuthorizedModuleCommand, ModuleExecutionError> {
    authorize_module_command(session_capabilities, request, approval)
        .map_err(|failure| ModuleExecutionError::Authorization(failure.decision))
}

pub fn execute_module_command<P: ModuleProvider>(
    provider: &P,
    session_capabilities: &[String],
    request: &ModuleCommandRequest,
    approval: ApprovalState,
    context: &ProviderContext,
) -> Result<super::module_provider::ProviderResult, ModuleExecutionError> {
    let authorized = authorize_request(session_capabilities, request, approval)?;
    provider
        .apply(&authorized, context)
        .map_err(ModuleExecutionError::Provider)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_provider::{DryRunProvider, ModuleProvider};
    use crate::module_state::{ModuleCommand, ModuleState};

    fn request() -> ModuleCommandRequest {
        ModuleCommandRequest {
            module_id: "wpc-engine".into(),
            command: ModuleCommand::Start,
            reason: "controlled test".into(),
        }
    }

    fn context() -> ProviderContext {
        ProviderContext {
            request_id: "execution-test".into(),
            principal_id: "human:operator".into(),
            reason: "controlled test".into(),
        }
    }

    #[test]
    fn approved_module_command_reaches_provider() {
        let result = execute_module_command(
            &DryRunProvider,
            &["module.start".into()],
            &request(),
            ApprovalState::Granted,
            &context(),
        )
        .expect("policy-approved command should reach provider");

        assert_eq!(result.provider_id, "dry-run");
        assert_eq!(result.resulting_state, ModuleState::Ready);
    }

    #[test]
    fn missing_session_capability_stops_before_provider() {
        let error = execute_module_command(
            &DryRunProvider,
            &[],
            &request(),
            ApprovalState::Granted,
            &context(),
        )
        .expect_err("missing capability must stop execution");

        assert_eq!(
            error,
            ModuleExecutionError::Authorization(AuthorizationDecision::Deny)
        );
    }

    #[test]
    fn execute_without_trusted_approval_stops_before_provider() {
        let error = execute_module_command(
            &DryRunProvider,
            &["module.start".into()],
            &request(),
            ApprovalState::Pending,
            &context(),
        )
        .expect_err("missing trusted approval must stop execution");

        assert_eq!(
            error,
            ModuleExecutionError::Authorization(AuthorizationDecision::NeedsApproval)
        );
    }

    #[test]
    fn provider_receives_only_authorized_command() {
        struct RecordingProvider;

        impl ModuleProvider for RecordingProvider {
            fn provider_id(&self) -> &str {
                "recording"
            }

            fn supports(&self, request: &AuthorizedModuleCommand) -> bool {
                request.capability == "module.start"
            }

            fn apply(
                &self,
                request: &AuthorizedModuleCommand,
                _context: &ProviderContext,
            ) -> Result<super::super::module_provider::ProviderResult, ProviderError> {
                assert_eq!(request.module_id, "wpc-engine");
                assert_eq!(request.capability, "module.start");
                Ok(super::super::module_provider::ProviderResult {
                    provider_id: self.provider_id().into(),
                    module_id: request.module_id.clone(),
                    command: request.command.clone(),
                    resulting_state: ModuleState::Ready,
                    message: "recorded".into(),
                })
            }
        }

        let result = execute_module_command(
            &RecordingProvider,
            &["module.start".into()],
            &request(),
            ApprovalState::Granted,
            &context(),
        )
        .expect("authorized command should be accepted");

        assert_eq!(result.provider_id, "recording");
    }
}
