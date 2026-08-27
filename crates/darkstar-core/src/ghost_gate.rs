//! Ghost Gate egress contract for Darkstar.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:30:00
//! REASON FOR CREATION: Bring the proven WPC Ghost Gate request contract into Darkstar without duplicating VPN, Tor, DNS or firewall implementations.
//! MECHANICS: Validates an explicit egress request and converts it into a fail-closed decision when no Gate policy is available. Actual network enforcement stays outside the trusted core.
//! SYSTEM PART: Darkstar Core / Ghost Gate Boundary
//! ARCHITECTURE FUNCTION: Define the authenticated, language-neutral boundary between Darkstar execution and the hardened network egress VM.
//! DEPENDENCIES/LINKS: Mirrors the WPC Ghost Gate `egress_request.schema.json`; future provider owns firewall/VPN/Tor/DNS enforcement and health checks.
//! TECH STACK: Rust 2024 + serde; selected for a small, deterministic security contract with no network-side effects.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-ghost-gate-contract
//! ==========================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EgressProtocol {
    Https,
    Http,
    Dns,
    Tcp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EgressPolicy {
    Direct,
    Vpn,
    Tor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EgressRequest {
    pub request_id: String,
    pub destination: String,
    pub protocol: EgressProtocol,
    pub reason: String,
    pub policy: Option<EgressPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GhostGateDecision {
    Deny,
    Allow(EgressRequest),
}

pub fn evaluate(request: EgressRequest, gate_available: bool) -> GhostGateDecision {
    if !gate_available || request.request_id.is_empty() || request.destination.is_empty() || request.reason.is_empty() {
        GhostGateDecision::Deny
    } else {
        GhostGateDecision::Allow(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> EgressRequest {
        EgressRequest {
            request_id: "req-1".into(),
            destination: "api.github.com".into(),
            protocol: EgressProtocol::Https,
            reason: "read repository metadata".into(),
            policy: Some(EgressPolicy::Vpn),
        }
    }

    #[test]
    fn gate_is_fail_closed_when_unavailable() {
        assert_eq!(evaluate(request(), false), GhostGateDecision::Deny);
    }

    #[test]
    fn valid_request_can_be_forwarded_to_gate() {
        assert!(matches!(evaluate(request(), true), GhostGateDecision::Allow(_)));
    }

    #[test]
    fn malformed_request_is_denied() {
        let mut malformed = request();
        malformed.destination.clear();
        assert_eq!(evaluate(malformed, true), GhostGateDecision::Deny);
    }

    #[test]
    fn schema_shape_round_trips() {
        let request = request();
        let encoded = serde_json::to_string(&request).expect("serialize request");
        let decoded: EgressRequest = serde_json::from_str(&encoded).expect("deserialize request");
        assert_eq!(request, decoded);
    }
}
