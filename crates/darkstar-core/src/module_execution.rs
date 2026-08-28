//! Authorization-gated module execution contract tests.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-28 01:35:00
//! REASON FOR CREATION: Prove that module execution cannot reach a provider without Darkstar policy authorization.
//! MECHANICS: Tests require an authorized command object, reject missing capability or approval, and allow an authorized command to reach a provider.
//! SYSTEM PART: Darkstar Core / Module Execution
//! ARCHITECTURE FUNCTION: Close the execution boundary between policy decisions and provider execution.
//! DEPENDENCIES/LINKS: module_state, module_provider, policy; later connected to orchestrator plans.
//! TECH STACK: Rust 2024 + existing Darkstar contracts.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-module-control, PR #6
//! ==========================================

use super::module_provider::{authorize_module_command, DryRunProvider, ProviderContext};
use super::module_state::{ModuleCommand, ModuleCommandRequest};
use super::policy::{ApprovalState, AuthorizationDecision};
use uuid::Uuid;

#[test]
fn approved_module_command_reaches_provider() {
    let request = ModuleCommandRequest {
        request_id: Uuid::new_v4(),
        module_id: "wpc-engine".into(),
        command: ModuleCommand::Start,
        capability: "module.start".into(),
        reason: "controlled test".into(),
    };
    let authorized = authorize_module_command(
        &["module.start".into()],
        &request,
        ApprovalState::Granted,
    )
    .expect("policy should authorize");
    let provider = DryRunProvider;
    let context = ProviderContext {
        request_id: request.request_id.to_string(),
        principal_id: "human:operator".into(),
        reason: request.reason.clone(),
    };

    let result = provider
        .apply(&authorized, &context)
        .expect("authorized command should execute");
    assert_eq!(
        result.resulting_state,
        super::module_state::ModuleState::Ready
    );
}

#[test]
fn missing_session_capability_never_reaches_provider() {
    let request = ModuleCommandRequest {
        request_id: Uuid::new_v4(),
        module_id: "wpc-engine".into(),
        command: ModuleCommand::Start,
        capability: "module.start".into(),
        reason: "controlled test".into(),
    };

    let decision = authorize_module_command(&[], &request, ApprovalState::Granted);
    assert_eq!(
        decision.unwrap_err().decision,
        AuthorizationDecision::Deny
    );
}

#[test]
fn execute_without_approval_stops_before_provider() {
    let request = ModuleCommandRequest {
        request_id: Uuid::new_v4(),
        module_id: "wpc-engine".into(),
        command: ModuleCommand::Start,
        capability: "module.start".into(),
        reason: "controlled test".into(),
    };

    let decision = authorize_module_command(
        &["module.start".into()],
        &request,
        ApprovalState::Pending,
    );
    assert_eq!(
        decision.unwrap_err().decision,
        AuthorizationDecision::NeedsApproval
    );
}
