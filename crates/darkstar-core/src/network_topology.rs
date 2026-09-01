// darkstar-header-v1
// po co: network_topology.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
//! Layered network topology contract for the Darkstar perimeter.
//!
//! SYSTEM PART: Darkstar Core / Network Boundary
//! ARCHITECTURE FUNCTION: Model the ordered path from public Internet through
//! Warlock Bridge, Kali Bridge, Darkstar and the protected AIONS endpoint.
//! SECURITY: This module describes topology and trust boundaries only. It does
//! not create tunnels, alter routes, spoof identity or execute network tools.
//! DEPLOYMENT MODEL: Azure starts with three isolated container roles; the
//! intended physical deployment places Warlock Bridge and Kali on separate
//! mini PCs. The AIONS endpoint remains outside this three-container edge.
//! TECH STACK: Rust 2024 + serde.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkLayer {
    Internet,
    /// Perymetr. Historycznie nazywany Sheriff; alias serde utrzymuje czytelnosc
    /// zdarzen i konfiguracji zapisanych pod stara nazwa.
    #[serde(alias = "sheriff_bridge")]
    WarlockBridge,
    KaliBridge,
    Darkstar,
    Aions,
}

/// Odwzorowuje historyczny identyfikator perymetru na obowiazujacy.
/// Nie tworzy drugiego wezla - istnieje dokladnie jeden perymetr.
pub fn resolve_legacy_node_id(node_id: &str) -> &str {
    match node_id {
        "sheriff-bridge" => "warlock-bridge",
        other => other,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Untrusted,
    Perimeter,
    Sandbox,
    ControlPlane,
    Protected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeploymentForm {
    Public,
    AzureContainer,
    MiniPc,
    ProtectedEndpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkNode {
    pub node_id: String,
    pub name: String,
    pub layer: NetworkLayer,
    pub trust: TrustLevel,
    pub deployment: DeploymentForm,
    pub physical_hint: Option<String>,
    pub visible_identity: Option<String>,
    pub route_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkHop {
    pub from: String,
    pub to: String,
    pub channel: String,
    pub required_gate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkTopology {
    pub topology_id: String,
    pub nodes: Vec<NetworkNode>,
    pub hops: Vec<NetworkHop>,
}

pub fn reference_topology() -> NetworkTopology {
    NetworkTopology {
        topology_id: "darkstar-perimeter-v2".into(),
        nodes: vec![
            NetworkNode {
                node_id: "internet".into(),
                name: "Internet".into(),
                layer: NetworkLayer::Internet,
                trust: TrustLevel::Untrusted,
                deployment: DeploymentForm::Public,
                physical_hint: None,
                visible_identity: None,
                route_role: "public network".into(),
            },
            NetworkNode {
                node_id: "warlock-bridge".into(),
                name: "Warlock Bridge".into(),
                layer: NetworkLayer::WarlockBridge,
                trust: TrustLevel::Perimeter,
                deployment: DeploymentForm::AzureContainer,
                physical_hint: Some("dedicated perimeter mini PC in physical deployment".into()),
                visible_identity: Some("warlock-egress".into()),
                route_role: "first perimeter gateway".into(),
            },
            NetworkNode {
                node_id: "kali-bridge".into(),
                name: "Kali Bridge".into(),
                layer: NetworkLayer::KaliBridge,
                trust: TrustLevel::Sandbox,
                deployment: DeploymentForm::AzureContainer,
                physical_hint: Some("separate Kali mini PC in physical deployment".into()),
                visible_identity: Some("kali-egress".into()),
                route_role: "web and security testing boundary".into(),
            },
            NetworkNode {
                node_id: "darkstar".into(),
                name: "Darkstar".into(),
                layer: NetworkLayer::Darkstar,
                trust: TrustLevel::ControlPlane,
                deployment: DeploymentForm::AzureContainer,
                physical_hint: Some("headless control service".into()),
                visible_identity: Some("darkstar-egress".into()),
                route_role: "policy, capability and orchestration gate".into(),
            },
            NetworkNode {
                node_id: "aions".into(),
                name: "AIONS".into(),
                layer: NetworkLayer::Aions,
                trust: TrustLevel::Protected,
                deployment: DeploymentForm::ProtectedEndpoint,
                physical_hint: Some("protected local ecosystem".into()),
                visible_identity: Some("private".into()),
                route_role: "protected system endpoint".into(),
            },
        ],
        hops: vec![
            NetworkHop {
                from: "internet".into(),
                to: "warlock-bridge".into(),
                channel: "internet ingress/egress".into(),
                required_gate: "warlock perimeter policy".into(),
            },
            NetworkHop {
                from: "warlock-bridge".into(),
                to: "kali-bridge".into(),
                channel: "segmented security link".into(),
                required_gate: "warlock-to-kali route policy".into(),
            },
            NetworkHop {
                from: "kali-bridge".into(),
                to: "darkstar".into(),
                channel: "security-to-control link".into(),
                required_gate: "Darkstar capability gate".into(),
            },
            NetworkHop {
                from: "darkstar".into(),
                to: "aions".into(),
                channel: "protected application link".into(),
                required_gate: "Darkstar policy + provider authorization".into(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_path_is_ordered() {
        let topology = reference_topology();
        let ids: Vec<&str> = topology
            .nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect();
        assert_eq!(
            ids,
            [
                "internet",
                "warlock-bridge",
                "kali-bridge",
                "darkstar",
                "aions"
            ]
        );
        assert_eq!(topology.hops.len(), 4);
        assert_eq!(topology.hops[3].to, "aions");
    }

    #[test]
    fn legacy_sheriff_layer_still_deserializes() {
        // Zapisane wczesniej zdarzenia i konfiguracje nadal musza sie wczytywac.
        let legacy: NetworkLayer = serde_json::from_str("\"sheriff_bridge\"").unwrap();
        assert_eq!(legacy, NetworkLayer::WarlockBridge);
    }

    #[test]
    fn layer_serializes_under_the_new_name() {
        let json = serde_json::to_string(&NetworkLayer::WarlockBridge).unwrap();
        assert_eq!(json, "\"warlock_bridge\"");
    }

    #[test]
    fn legacy_node_id_resolves_to_warlock() {
        assert_eq!(resolve_legacy_node_id("sheriff-bridge"), "warlock-bridge");
        assert_eq!(resolve_legacy_node_id("warlock-bridge"), "warlock-bridge");
        assert_eq!(resolve_legacy_node_id("darkstar"), "darkstar");
    }

    #[test]
    fn only_one_perimeter_node_exists() {
        let topology = reference_topology();
        assert_eq!(
            topology
                .nodes
                .iter()
                .filter(|node| node.trust == TrustLevel::Perimeter)
                .count(),
            1
        );
    }

    #[test]
    fn aions_is_the_only_protected_endpoint() {
        let topology = reference_topology();
        assert_eq!(
            topology
                .nodes
                .iter()
                .filter(|node| node.trust == TrustLevel::Protected)
                .count(),
            1
        );
    }

    #[test]
    fn first_azure_deployment_has_three_edge_roles() {
        let topology = reference_topology();
        let azure_nodes = topology
            .nodes
            .iter()
            .filter(|node| node.deployment == DeploymentForm::AzureContainer)
            .count();
        assert_eq!(azure_nodes, 3);
    }
}
