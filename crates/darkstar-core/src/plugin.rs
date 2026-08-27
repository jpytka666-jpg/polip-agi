//! Plugin contract for language- and runtime-neutral Darkstar extensions.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 20:54:30
//! REASON FOR CREATION: Define the stable boundary that lets Darkstar load capabilities implemented in Rust, Python, C, C++, PowerShell, MCP and remote services.
//! MECHANICS: Plugins advertise identity and capabilities, then exchange typed JSON requests/results through an adapter chosen by transport.
//! SYSTEM PART: Darkstar Core / Plugin Registry
//! ARCHITECTURE FUNCTION: Decouple the secure control plane from replaceable tool implementations.
//! DEPENDENCIES/LINKS: serde, serde_json, uuid; future adapters will map stdio, HTTP, MCP and authenticated peer transports onto this contract.
//! TECH STACK: Rust 2024; selected for the trusted control-plane types while keeping payloads language-neutral through JSON.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginId {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    pub api_version: String,
    pub plugin: PluginId,
    pub runtime: String,
    pub platform: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub request_id: Uuid,
    pub capability: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub request_id: Uuid,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_round_trips_as_json() {
        let manifest = PluginManifest {
            api_version: crate::API_VERSION.to_string(),
            plugin: PluginId {
                name: "example.echo".into(),
                version: "0.1.0".into(),
            },
            runtime: "native".into(),
            platform: "linux-x86_64".into(),
            capabilities: vec![Capability {
                name: "echo".into(),
                description: "Returns the supplied JSON value.".into(),
                read_only: true,
            }],
        };

        let encoded = serde_json::to_string(&manifest).expect("serialize manifest");
        let decoded: PluginManifest = serde_json::from_str(&encoded).expect("deserialize manifest");
        assert_eq!(manifest, decoded);
    }
}
