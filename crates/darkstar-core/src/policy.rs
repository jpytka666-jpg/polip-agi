//! Authorization primitives separating agent intent from execution authority.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 20:54:30
//! REASON FOR CREATION: Prevent model requests and plugin capabilities from becoming implicit authority.
//! MECHANICS: A request is evaluated against session capability, action risk and whether an explicit approval is required.
//! SYSTEM PART: Darkstar Core / Policy Engine
//! ARCHITECTURE FUNCTION: Enforce the rule MODEL MAY PROPOSE; DARKSTAR AUTHORIZES.
//! DEPENDENCIES/LINKS: serde, uuid; consumes Session and feeds plugin execution and audit.
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
    pub explicit_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorizationDecision {
    Allow,
    Deny,
    NeedsApproval,
}

pub fn authorize(
    session_capabilities: &[String],
    request: &AuthorizationRequest,
) -> AuthorizationDecision {
    let permitted = session_capabilities
        .iter()
        .any(|item| item == &request.capability);

    if !permitted {
        return AuthorizationDecision::Deny;
    }

    match request.risk {
        ActionRisk::Read | ActionRisk::Propose => AuthorizationDecision::Allow,
        ActionRisk::Execute | ActionRisk::Destructive => {
            if request.explicit_approval {
                AuthorizationDecision::Allow
            } else {
                AuthorizationDecision::NeedsApproval
            }
        }
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
            explicit_approval: false,
        };
        assert_eq!(authorize(&session_caps(), &request), AuthorizationDecision::Allow);
    }

    #[test]
    fn execution_without_approval_stops_at_policy() {
        let request = AuthorizationRequest {
            request_id: Uuid::new_v4(),
            capability: "github.write".into(),
            risk: ActionRisk::Execute,
            explicit_approval: false,
        };
        assert_eq!(
            authorize(&session_caps(), &request),
            AuthorizationDecision::NeedsApproval
        );
    }

    #[test]
    fn missing_capability_is_denied() {
        let request = AuthorizationRequest {
            request_id: Uuid::new_v4(),
            capability: "host.shell".into(),
            risk: ActionRisk::Execute,
            explicit_approval: true,
        };
        assert_eq!(authorize(&session_caps(), &request), AuthorizationDecision::Deny);
    }
}
