//! Deterministic capability index for the Darkstar control plane.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:05:00
//! REASON FOR CREATION: Let Darkstar find the small set of plugin capabilities relevant to a task without exposing the full plugin library to the agent at once.
//! MECHANICS: Build a read-only index from validated registry manifests, group capabilities by name, and return compact matches containing plugin identity and capability metadata.
//! SYSTEM PART: Darkstar Core / Capability Index
//! ARCHITECTURE FUNCTION: Connect plugin discovery and registry state to policy/tool selection while keeping the agent-facing capability view compact.
//! DEPENDENCIES/LINKS: crate::plugin::{Capability, PluginId, PluginManifest}, crate::registry::PluginRegistry.
//! TECH STACK: Rust 2024 + std::collections::BTreeMap; selected for deterministic lookup and a dependency-free core implementation.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use std::collections::BTreeMap;

use crate::plugin::{Capability, PluginId, PluginManifest};
use crate::registry::PluginRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityMatch {
    pub plugin: PluginId,
    pub runtime: String,
    pub platform: String,
    pub capability: Capability,
}

#[derive(Debug, Default)]
pub struct CapabilityIndex {
    by_name: BTreeMap<String, Vec<CapabilityMatch>>,
}

impl CapabilityIndex {
    pub fn from_registry(registry: &PluginRegistry) -> Result<Self, crate::plugin_host::PluginHostError> {
        let manifests = registry.list()?;
        let mut by_name: BTreeMap<String, Vec<CapabilityMatch>> = BTreeMap::new();

        for manifest in manifests {
            Self::index_manifest(&mut by_name, &manifest);
        }

        for matches in by_name.values_mut() {
            matches.sort_by(|left, right| {
                left.plugin
                    .name
                    .cmp(&right.plugin.name)
                    .then_with(|| left.plugin.version.cmp(&right.plugin.version))
            });
        }

        Ok(Self { by_name })
    }

    fn index_manifest(
        by_name: &mut BTreeMap<String, Vec<CapabilityMatch>>,
        manifest: &PluginManifest,
    ) {
        for capability in &manifest.capabilities {
            by_name
                .entry(capability.name.clone())
                .or_default()
                .push(CapabilityMatch {
                    plugin: manifest.plugin.clone(),
                    runtime: manifest.runtime.clone(),
                    platform: manifest.platform.clone(),
                    capability: capability.clone(),
                });
        }
    }

    pub fn find(&self, capability_name: &str) -> &[CapabilityMatch] {
        self.by_name
            .get(capability_name)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn capability_names(&self) -> Vec<String> {
        self.by_name.keys().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{Capability, PluginId};
    use crate::API_VERSION;

    fn manifest(name: &str, version: &str, capabilities: &[&str]) -> PluginManifest {
        PluginManifest {
            api_version: API_VERSION.into(),
            plugin: PluginId {
                name: name.into(),
                version: version.into(),
            },
            runtime: "python".into(),
            platform: "any".into(),
            capabilities: capabilities
                .iter()
                .map(|name| Capability {
                    name: (*name).into(),
                    description: format!("Capability {name}"),
                    read_only: true,
                })
                .collect(),
        }
    }

    #[test]
    fn indexes_capabilities_from_registry() {
        let registry = PluginRegistry::new();
        registry
            .register(manifest("example.alpha", "0.1.0", &["echo", "search"]))
            .expect("register alpha");
        registry
            .register(manifest("example.beta", "0.1.0", &["echo"]))
            .expect("register beta");

        let index = CapabilityIndex::from_registry(&registry).expect("build capability index");

        assert_eq!(index.capability_names(), vec!["echo", "search"]);
        assert_eq!(index.find("echo").len(), 2);
        assert_eq!(index.find("search").len(), 1);
        assert!(index.find("missing").is_empty());
    }

    #[test]
    fn returns_empty_index_for_empty_registry() {
        let registry = PluginRegistry::new();
        let index = CapabilityIndex::from_registry(&registry).expect("build capability index");

        assert!(index.is_empty());
        assert!(index.capability_names().is_empty());
    }
}
