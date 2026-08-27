//! In-memory plugin registry for the Darkstar control plane.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 21:35:00
//! REASON FOR CREATION: Provide one authoritative place where Darkstar records validated plugin identities and their advertised capabilities.
//! MECHANICS: Validate a plugin manifest, index it by stable plugin name, reject duplicate names, and expose read-only lookup/list operations. The registry never executes plugin code.
//! SYSTEM PART: Darkstar Core / Plugin Registry
//! ARCHITECTURE FUNCTION: Bridge plugin discovery to policy and execution without coupling the core to a plugin's language or transport.
//! DEPENDENCIES/LINKS: crate::plugin::PluginManifest, crate::plugin_host::validate_manifest; future registry persistence and discovery layers will build on this contract.
//! TECH STACK: Rust 2024; std::collections::BTreeMap and std::sync::RwLock keep the first registry deterministic, lightweight, and dependency-free.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::plugin::{PluginId, PluginManifest};
use crate::plugin_host::{PluginHostError, validate_manifest};

#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: RwLock<BTreeMap<String, PluginManifest>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, manifest: PluginManifest) -> Result<(), PluginHostError> {
        validate_manifest(&manifest)?;

        let name = manifest.plugin.name.clone();
        let mut plugins = self
            .plugins
            .write()
            .map_err(|_| PluginHostError::Unavailable("plugin registry lock poisoned".into()))?;

        if plugins.contains_key(&name) {
            return Err(PluginHostError::Rejected(format!(
                "plugin already registered: {name}"
            )));
        }

        plugins.insert(name, manifest);
        Ok(())
    }

    pub fn get(&self, plugin_name: &str) -> Result<Option<PluginManifest>, PluginHostError> {
        let plugins = self
            .plugins
            .read()
            .map_err(|_| PluginHostError::Unavailable("plugin registry lock poisoned".into()))?;
        Ok(plugins.get(plugin_name).cloned())
    }

    pub fn list(&self) -> Result<Vec<PluginManifest>, PluginHostError> {
        let plugins = self
            .plugins
            .read()
            .map_err(|_| PluginHostError::Unavailable("plugin registry lock poisoned".into()))?;
        Ok(plugins.values().cloned().collect())
    }

    pub fn contains(&self, plugin_id: &PluginId) -> Result<bool, PluginHostError> {
        let plugins = self
            .plugins
            .read()
            .map_err(|_| PluginHostError::Unavailable("plugin registry lock poisoned".into()))?;
        Ok(plugins
            .get(&plugin_id.name)
            .map(|manifest| manifest.plugin == *plugin_id)
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::API_VERSION;
    use crate::plugin::{Capability, PluginId};

    fn manifest(name: &str, version: &str) -> PluginManifest {
        PluginManifest {
            api_version: API_VERSION.into(),
            plugin: PluginId {
                name: name.into(),
                version: version.into(),
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
    fn register_and_lookup_work() {
        let registry = PluginRegistry::new();
        let plugin = manifest("example.echo", "0.1.0");

        registry.register(plugin.clone()).expect("register plugin");

        assert_eq!(registry.get("example.echo").unwrap(), Some(plugin.clone()));
        assert!(registry.contains(&plugin.plugin).unwrap());
        assert_eq!(registry.list().unwrap(), vec![plugin]);
    }

    #[test]
    fn duplicate_plugin_name_is_rejected() {
        let registry = PluginRegistry::new();
        registry
            .register(manifest("example.echo", "0.1.0"))
            .expect("first registration");

        let error = registry.register(manifest("example.echo", "0.2.0"));
        assert!(
            matches!(error, Err(PluginHostError::Rejected(message)) if message.contains("already registered"))
        );
    }

    #[test]
    fn invalid_manifest_is_rejected_before_storage() {
        let registry = PluginRegistry::new();
        let mut invalid = manifest("example.echo", "0.1.0");
        invalid.api_version = "darkstar.core/v999".into();

        assert!(matches!(
            registry.register(invalid),
            Err(PluginHostError::Protocol(_))
        ));
        assert!(registry.list().unwrap().is_empty());
    }
}
