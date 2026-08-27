//! Capability selection and policy gate for Darkstar.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 21:35:00
//! REASON FOR CREATION: Join deterministic capability selection to the existing authorization policy without allowing selection to grant execution authority.
//! MECHANICS: Look up candidates through CapabilitySelector, choose the first deterministic candidate, create an AuthorizationRequest, and evaluate it with the existing policy engine and trusted approval state.
//! SYSTEM PART: Darkstar Core / Capability Gate
//! ARCHITECTURE FUNCTION: Security boundary between capability discovery and plugin execution; selection identifies a target while policy decides whether it may proceed.
//! DEPENDENCIES/LINKS: crate::capability_selector, crate::policy, crate::capability_index; execution remains outside this module.
//! TECH STACK: Rust 2024 + existing Darkstar contracts; selected for a small, deterministic and testable trusted-core gate.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use uuid::Uuid;

use crate::capability_index::{CapabilityIndex, CapabilityMatch};
use crate::capability_selector::{CapabilitySelection, CapabilitySelector};
use crate::policy::{
    ActionRisk, ApprovalState, AuthorizationDecision, AuthorizationRequest, authorize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGateDecision {
    pub decision: AuthorizationDecision,
    pub selection: CapabilitySelection,
}

impl CapabilityGateDecision {
    pub fn selected(&self) -> Option<&CapabilityMatch> {
        self.selection.first()
    }
}

pub fn evaluate(
    index: &CapabilityIndex,
    session_capabilities: &[String],
    capability_name: &str,
    risk: ActionRisk,
    approval: ApprovalState,
    preferred_runtime: Option<&str>,
    preferred_platform: Option<&str>,
) -> CapabilityGateDecision {
    let selection = CapabilitySelector::select(
        index,
        capability_name,
        preferred_runtime,
        preferred_platform,
    );

    if selection.is_empty() {
        return CapabilityGateDecision {
            decision: AuthorizationDecision::Deny,
            selection,
        };
    }

    let request = AuthorizationRequest {
        request_id: Uuid::new_v4(),
        capability: capability_name.into(),
        risk,
    };

    let decision = authorize(session_capabilities, &request, approval);

    CapabilityGateDecision {
        decision,
        selection,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::API_VERSION;
    use crate::plugin::{Capability, PluginId, PluginManifest};
    use crate::registry::PluginRegistry;

    fn manifest(name: &str, runtime: &str, platform: &str, capability: &str) -> PluginManifest {
        PluginManifest {
            api_version: API_VERSION.into(),
            plugin: PluginId {
                name: name.into(),
                version: "0.1.0".into(),
            },
            runtime: runtime.into(),
            platform: platform.into(),
            capabilities: vec![Capability {
                name: capability.into(),
                description: format!("Capability {capability}"),
                read_only: true,
            }],
        }
    }

    #[test]
    fn read_capability_is_allowed_after_selection() {
        let registry = PluginRegistry::new();
        registry
            .register(manifest("example.python", "python", "any", "echo"))
            .expect("register plugin");
        let index = CapabilityIndex::from_registry(&registry).expect("build index");

        let decision = evaluate(
            &index,
            &["echo".into()],
            "echo",
            ActionRisk::Read,
            ApprovalState::NotRequired,
            Some("python"),
            None,
        );

        assert_eq!(decision.decision, AuthorizationDecision::Allow);
        assert_eq!(decision.selected().unwrap().plugin.name, "example.python");
    }

    #[test]
    fn execute_requires_trusted_approval() {
        let registry = PluginRegistry::new();
        registry
            .register(manifest("example.python", "python", "any", "echo"))
            .expect("register plugin");
        let index = CapabilityIndex::from_registry(&registry).expect("build index");

        let decision = evaluate(
            &index,
            &["echo".into()],
            "echo",
            ActionRisk::Execute,
            ApprovalState::Pending,
            Some("python"),
            None,
        );

        assert_eq!(decision.decision, AuthorizationDecision::NeedsApproval);
        assert_eq!(decision.selected().unwrap().plugin.name, "example.python");
    }

    #[test]
    fn missing_capability_never_reaches_execution_permission() {
        let registry = PluginRegistry::new();
        registry
            .register(manifest("example.python", "python", "any", "echo"))
            .expect("register plugin");
        let index = CapabilityIndex::from_registry(&registry).expect("build index");

        let decision = evaluate(
            &index,
            &["echo".into()],
            "shell",
            ActionRisk::Execute,
            ApprovalState::Granted,
            Some("python"),
            None,
        );

        assert_eq!(decision.decision, AuthorizationDecision::Deny);
        assert!(decision.selected().is_none());
    }
}
