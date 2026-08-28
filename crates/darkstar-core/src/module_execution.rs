//! Authorization-gated module execution contract tests.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-28 01:35:00
//! REASON FOR CREATION: Prove that module execution cannot reach a provider without Darkstar policy authorization.
//! MECHANICS: Tests require an AuthorizedModuleCommand produced only after policy approval; denied capability or missing approval cannot reach the provider.
//! SYSTEM PART: Darkstar Core / Module Execution
//! ARCHITECTURE FUNCTION: Close the execution boundary between policy decisions and provider execution.
//! DEPENDENCIES/LINKS: module_state, module_provider, policy; later connected to orchestrator plans.
//! TECH STACK: Rust 2024 + existing Darkstar contracts.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-module-control, PR #6
//! ==========================================

use super::module_provider::{
    DryRunProvider, ModuleProvider, ProviderContext, authorize_module_command,
};
use super::module_state::{ModuleCommand, ModuleCommandRequest, ModuleState};
use super::policy::{ApprovalState, AuthorizationDecision};

fn request() -> ModuleCommandRequest {
    ModuleCommandRequest {
        module_id: "wpc-engine".into(),
        command: ModuleCommand::Start,
        reason: "controlled test".into(),
    }
}

#[test]
fn approved_module_command_reaches_provider() {
    let request = request();
    let authorized = authorize_module_command(
        &["module.start".into()],
        &request,
        ApprovalState::Granted,
    )
    .expect("policy should authorize");
    let provider = DryRunProvider;
    let context = ProviderContext {
        request_id: authorized.request_id.to_string(),
        principal_id: "human:operator".into(),
        reason: authorized.reason.clone(),
    };

    let result = provider
        .apply(&authorized, &context)
        .expect("authorized command should execute");
    assert_eq!(result.resulting_state, ModuleState::Ready);
}

#[test]
fn missing_session_capability_never_reaches_provider() {
    let failure = authorize_module_command(&[], &request(), ApprovalState::Granted)
        .expect_err("missing capability must be denied");
    assert_eq!(failure.decision, AuthorizationDecision::Deny);
}

#[test]
fn execute_without_approval_stops_before_provider() {
    let failure = authorize_module_command(
        &["module.start".into()],
        &request(),
        ApprovalState::Pending,
    )
    .expect_err("execution without trusted approval must stop");
    assert_eq!(failure.decision, AuthorizationDecision::NeedsApproval);
}
