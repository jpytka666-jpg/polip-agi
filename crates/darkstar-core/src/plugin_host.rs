//! Transport-neutral plugin host contracts.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 20:54:30
//! REASON FOR CREATION: Keep plugin discovery and execution independent from the transport used by a plugin.
//! MECHANICS: A host selects an adapter such as stdio, HTTP, MCP or authenticated peer transport while the core sees only manifests and typed requests/results.
//! SYSTEM PART: Darkstar Core / Plugin Host
//! ARCHITECTURE FUNCTION: Enable plug-and-play extensions across languages and operating systems without linking them into the Rust core.
//! DEPENDENCIES/LINKS: crate::plugin; serde; future transport adapters implement PluginAdapter.
//! TECH STACK: Rust 2024; selected for the trusted orchestration boundary while protocol payloads remain language-neutral.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use crate::plugin::{PluginManifest, PluginRequest, PluginResult};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransportKind {
    Stdio,
    Http,
    Mcp,
    AuthenticatedPeer,
}

#[derive(Debug, Error)]
pub enum PluginHostError {
    #[error("plugin transport is unavailable: {0}")]
    Unavailable(String),
    #[error("plugin rejected request: {0}")]
    Rejected(String),
    #[error("plugin protocol error: {0}")]
    Protocol(String),
}

pub trait PluginAdapter: Send + Sync {
    fn transport(&self) -> TransportKind;
    fn manifest(&self) -> &PluginManifest;
    fn invoke(&self, request: PluginRequest) -> Result<PluginResult, PluginHostError>;
}

pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), PluginHostError> {
    if manifest.api_version != crate::API_VERSION {
        return Err(PluginHostError::Protocol(format!(
            "unsupported API version: {}",
            manifest.api_version
        )));
    }

    if manifest.plugin.name.trim().is_empty() {
        return Err(PluginHostError::Protocol(
            "plugin name cannot be empty".into(),
        ));
    }

    if manifest.capabilities.is_empty() {
        return Err(PluginHostError::Protocol(
            "plugin must advertise at least one capability".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{Capability, PluginId};

    #[test]
    fn valid_manifest_is_accepted() {
        let manifest = PluginManifest {
            api_version: crate::API_VERSION.into(),
            plugin: PluginId {
                name: "example.echo".into(),
                version: "0.1.0".into(),
            },
            runtime: "python".into(),
            platform: "linux-x86_64".into(),
            capabilities: vec![Capability {
                name: "echo".into(),
                description: "Return input.".into(),
                read_only: true,
            }],
        };

        assert!(validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn unknown_api_version_is_rejected() {
        let manifest = PluginManifest {
            api_version: "darkstar.core/v999".into(),
            plugin: PluginId {
                name: "example.echo".into(),
                version: "0.1.0".into(),
            },
            runtime: "python".into(),
            platform: "linux-x86_64".into(),
            capabilities: vec![Capability {
                name: "echo".into(),
                description: "Return input.".into(),
                read_only: true,
            }],
        };

        assert!(matches!(
            validate_manifest(&manifest),
            Err(PluginHostError::Protocol(_))
        ));
    }
}
