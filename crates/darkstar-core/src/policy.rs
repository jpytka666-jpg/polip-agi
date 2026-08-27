//! Authorization primitives separating agent intent from execution authority.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 20:54:30
//! REASON FOR CREATION: Prevent model requests and plugin capabilities from becoming implicit authority.
//! MECHANICS: A request is evaluated against session capability and a trusted approval state that the agent cannot forge.
//! SYSTEM PART: Darkstar Core / Policy Engine
//! ARCHITECTURE FUNCTION: Enforce the rule MODEL MAY PROPOSE; DARKSTAR AUTHORIZES.
//! DEPENDENCIES/LINKS: serde, uuid; consumes session capabilities and trusted approval state, then feeds execution and audit.
//! TECH STACK: Rust 2024; selected for explicit security state and predictable control flow.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionRisk {
    Read,
    Propose,
    Execute,
    Destructive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    pub request_id: Uuid,
    pub capability: String,
    pub risk: ActionRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalState {
    NotRequired,
    Pending,
    Granted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorizationDecision {
    Allow,
    Deny,
    NeedsApproval,
}

pub fn authorize(
    session_capabilities: &[String],
    request: &AuthorizationRequest,
    approval: ApprovalState,
) -> AuthorizationDecision {
    let permitted = session_capabilities
        .iter()
        .any(|item| item == &request.capability);

    if !permitted {
        return AuthorizationDecision::Deny;
    }

    match request.risk {
        ActionRisk::Read | ActionRisk::Propose => AuthorizationDecision::Allow,
        ActionRisk::Execute | ActionRisk::Destructive => match approval {
            ApprovalState::Granted => AuthorizationDecision::Allow,
            ApprovalState::Pending | ApprovalState::NotRequired => {
                AuthorizationDecision::NeedsApproval
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_caps() -> Vec<String> {
        vec!["github.read".into(), "github.write".into()]
    }

    #[test]
    fn reads_are_allowed_when_capability_is_present() {
        let request = AuthorizationRequest {
            request_id: Uuid::new_v4(),
            capability: "github.read".into(),
            risk: ActionRisk::Read,
        };
        assert_eq!(
            authorize(&session_caps(), &request, ApprovalState::NotRequired),
            AuthorizationDecision::Allow
        );
    }

    #[test]
    fn execution_without_trusted_approval_stops_at_policy() {
        let request = AuthorizationRequest {
            request_id: Uuid::new_v4(),
            capability: "github.write".into(),
            risk: ActionRisk::Execute,
        };
        assert_eq!(
            authorize(&session_caps(), &request, ApprovalState::Pending),
            AuthorizationDecision::NeedsApproval
        );
    }

    #[test]
    fn model_cannot_forge_approval_through_request_data() {
        let request = AuthorizationRequest {
            request_id: Uuid::new_v4(),
            capability: "github.write".into(),
            risk: ActionRisk::Execute,
        };
        assert_ne!(
            authorize(&session_caps(), &request, ApprovalState::Pending),
            AuthorizationDecision::Allow
        );
    }

    #[test]
    fn missing_capability_is_denied_even_with_approval() {
        let request = AuthorizationRequest {
            request_id: Uuid::new_v4(),
            capability: "host.shell".into(),
            risk: ActionRisk::Execute,
        };
        assert_eq!(
            authorize(&session_caps(), &request, ApprovalState::Granted),
            AuthorizationDecision::Deny
        );
    }
}
