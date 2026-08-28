//! Layered network topology contract for the Darkstar perimeter.
//!
//! SYSTEM PART: Darkstar Core / Network Boundary
//! ARCHITECTURE FUNCTION: Model the ordered path from public Internet through
//! Watcher, Kali security zone, Darkstar and the protected AIONS endpoint.
//! SECURITY: This module describes topology and trust boundaries only. It does
//! not create tunnels, alter routes, spoof identity or execute network tools.
//! TECH STACK: Rust 2024 + serde.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkLayer {
    Internet,
    Watcher,
    Kali,
    Darkstar,
    Aions,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkNode {
    pub node_id: String,
    pub name: String,
    pub layer: NetworkLayer,
    pub trust: TrustLevel,
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
        topology_id: "darkstar-perimeter-v1".into(),
        nodes: vec![
            NetworkNode {
                node_id: "internet".into(),
                name: "Internet".into(),
                layer: NetworkLayer::Internet,
                trust: TrustLevel::Untrusted,
                physical_hint: None,
                visible_identity: None,
                route_role: "public network".into(),
            },
            NetworkNode {
                node_id: "watcher".into(),
                name: "Watcher".into(),
                layer: NetworkLayer::Watcher,
                trust: TrustLevel::Perimeter,
                physical_hint: Some("small network appliance / mini PC".into()),
                visible_identity: Some("watcher-egress".into()),
                route_role: "first perimeter gateway".into(),
            },
            NetworkNode {
                node_id: "kali".into(),
                name: "Kali Security Zone".into(),
                layer: NetworkLayer::Kali,
                trust: TrustLevel::Sandbox,
                physical_hint: Some("isolated security runtime".into()),
                visible_identity: Some("kali-egress".into()),
                route_role: "web and security testing boundary".into(),
            },
            NetworkNode {
                node_id: "darkstar".into(),
                name: "Darkstar".into(),
                layer: NetworkLayer::Darkstar,
                trust: TrustLevel::ControlPlane,
                physical_hint: Some("headless control service".into()),
                visible_identity: Some("darkstar-egress".into()),
                route_role: "policy, capability and orchestration gate".into(),
            },
            NetworkNode {
                node_id: "aions".into(),
                name: "AIONS".into(),
                layer: NetworkLayer::Aions,
                trust: TrustLevel::Protected,
                physical_hint: Some("protected local ecosystem".into()),
                visible_identity: Some("private".into()),
                route_role: "protected system endpoint".into(),
            },
        ],
        hops: vec![
            NetworkHop {
                from: "internet".into(),
                to: "watcher".into(),
                channel: "internet ingress/egress".into(),
                required_gate: "watcher perimeter policy".into(),
            },
            NetworkHop {
                from: "watcher".into(),
                to: "kali".into(),
                channel: "segmented security link".into(),
                required_gate: "watcher-to-kali route policy".into(),
            },
            NetworkHop {
                from: "kali".into(),
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
        let ids: Vec<&str> = topology.nodes.iter().map(|node| node.node_id.as_str()).collect();
        assert_eq!(ids, ["internet", "watcher", "kali", "darkstar", "aions"]);
        assert_eq!(topology.hops.len(), 4);
        assert_eq!(topology.hops[3].to, "aions");
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
}
