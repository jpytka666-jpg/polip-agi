//! Filesystem discovery for validated Darkstar plugin manifests.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 21:45:00
//! REASON FOR CREATION: Let Darkstar discover plugin manifests without coupling discovery to any plugin language or runtime.
//! MECHANICS: Scan one configured directory for manifest.json files, parse each manifest, validate it with the existing plugin contract, and return manifests for registration. Discovery never executes plugin code.
//! SYSTEM PART: Darkstar Core / Plugin Discovery
//! ARCHITECTURE FUNCTION: Convert a filesystem plugin directory into validated registry inputs while preserving the language-neutral plugin boundary.
//! DEPENDENCIES/LINKS: std::fs, serde_json, crate::plugin::PluginManifest, crate::plugin_host::validate_manifest, crate::registry::PluginRegistry.
//! TECH STACK: Rust 2024 + standard filesystem APIs; selected to keep local discovery dependency-light and portable across Linux and Windows.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use std::fs;
use std::path::{Path, PathBuf};

use crate::plugin::PluginManifest;
use crate::plugin_host::{PluginHostError, validate_manifest};
use crate::registry::PluginRegistry;

const MANIFEST_FILE: &str = "manifest.json";

pub fn discover_manifests(root: impl AsRef<Path>) -> Result<Vec<(PathBuf, PluginManifest)>, PluginHostError> {
    let root = root.as_ref();
    let entries = fs::read_dir(root).map_err(|error| {
        PluginHostError::Unavailable(format!("cannot read plugin directory {}: {error}", root.display()))
    })?;

    let mut discovered = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| {
            PluginHostError::Unavailable(format!("cannot read plugin directory entry: {error}"))
        })?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join(MANIFEST_FILE);
        if !manifest_path.is_file() {
            continue;
        }

        let content = fs::read_to_string(&manifest_path).map_err(|error| {
            PluginHostError::Protocol(format!("cannot read {}: {error}", manifest_path.display()))
        })?;
        let manifest: PluginManifest = serde_json::from_str(&content).map_err(|error| {
            PluginHostError::Protocol(format!("invalid manifest {}: {error}", manifest_path.display()))
        })?;
        validate_manifest(&manifest)?;
        discovered.push((manifest_path, manifest));
    }

    discovered.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(discovered)
}

pub fn discover_into_registry(
    root: impl AsRef<Path>,
    registry: &PluginRegistry,
) -> Result<usize, PluginHostError> {
    let manifests = discover_manifests(root)?;
    let mut registered = 0;

    for (_, manifest) in manifests {
        registry.register(manifest)?;
        registered += 1;
    }

    Ok(registered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::API_VERSION;
    use crate::plugin::{Capability, PluginId};

    fn manifest(name: &str) -> PluginManifest {
        PluginManifest {
            api_version: API_VERSION.into(),
            plugin: PluginId {
                name: name.into(),
                version: "0.1.0".into(),
            },
            runtime: "python".into(),
            platform: "any".into(),
            capabilities: vec![Capability {
                name: "echo".into(),
                description: "Return input.".into(),
                read_only: true,
            }],
        }
    }

    #[test]
    fn empty_directory_discovers_nothing() {
        let root = std::env::temp_dir().join(format!("darkstar-discovery-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create temp directory");

        let discovered = discover_manifests(&root).expect("discover manifests");
        assert!(discovered.is_empty());

        fs::remove_dir_all(&root).expect("remove temp directory");
    }

    #[test]
    fn discovers_and_registers_manifest() {
        let root = std::env::temp_dir().join(format!("darkstar-discovery-{}", uuid::Uuid::new_v4()));
        let plugin_dir = root.join("example.echo");
        fs::create_dir_all(&plugin_dir).expect("create plugin directory");
        fs::write(
            plugin_dir.join(MANIFEST_FILE),
            serde_json::to_string(&manifest("example.echo")).expect("serialize manifest"),
        )
        .expect("write manifest");

        let registry = PluginRegistry::new();
        let count = discover_into_registry(&root, &registry).expect("discover into registry");

        assert_eq!(count, 1);
        assert!(registry.get("example.echo").unwrap().is_some());

        fs::remove_dir_all(&root).expect("remove temp directory");
    }
}
