//! Runtime module catalog for the whole AIONS ecosystem.
//!
//! SYSTEM PART: Darkstar Core / Module Registry
//! ARCHITECTURE FUNCTION: Record discoverable modules separately from plugin manifests.
//! TECH STACK: Rust 2024 + std RwLock + BTreeMap.

use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::module_state::{ModuleDescriptor, ModuleState};

#[derive(Debug, thiserror::Error)]
pub enum ModuleRegistryError {
    #[error("module registry lock poisoned")]
    LockPoisoned,
    #[error("module already registered: {0}")]
    Duplicate(String),
}

#[derive(Debug, Default)]
pub struct ModuleRegistry {
    modules: RwLock<BTreeMap<String, ModuleDescriptor>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, descriptor: ModuleDescriptor) -> Result<(), ModuleRegistryError> {
        let mut modules = self
            .modules
            .write()
            .map_err(|_| ModuleRegistryError::LockPoisoned)?;
        if modules.contains_key(&descriptor.module_id) {
            return Err(ModuleRegistryError::Duplicate(descriptor.module_id));
        }
        modules.insert(descriptor.module_id.clone(), descriptor);
        Ok(())
    }

    pub fn get(&self, module_id: &str) -> Result<Option<ModuleDescriptor>, ModuleRegistryError> {
        let modules = self
            .modules
            .read()
            .map_err(|_| ModuleRegistryError::LockPoisoned)?;
        Ok(modules.get(module_id).cloned())
    }

    pub fn list(&self) -> Result<Vec<ModuleDescriptor>, ModuleRegistryError> {
        let modules = self
            .modules
            .read()
            .map_err(|_| ModuleRegistryError::LockPoisoned)?;
        Ok(modules.values().cloned().collect())
    }

    pub fn set_state(
        &self,
        module_id: &str,
        state: ModuleState,
    ) -> Result<bool, ModuleRegistryError> {
        let mut modules = self
            .modules
            .write()
            .map_err(|_| ModuleRegistryError::LockPoisoned)?;
        let Some(module) = modules.get_mut(module_id) else {
            return Ok(false);
        };
        module.state = state;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(id: &str) -> ModuleDescriptor {
        ModuleDescriptor {
            module_id: id.into(),
            name: id.into(),
            system: "aions".into(),
            language: "rust".into(),
            version: Some("0.1.0".into()),
            capabilities: vec!["inspect".into()],
            dependencies: vec![],
            state: ModuleState::Offline,
            health: None,
        }
    }

    #[test]
    fn registry_is_deterministic_and_stateful() {
        let registry = ModuleRegistry::new();
        registry.register(descriptor("wpc")).unwrap();
        registry.register(descriptor("cbms")).unwrap();
        assert_eq!(registry.list().unwrap()[0].module_id, "cbms");
        assert!(registry.set_state("wpc", ModuleState::Ready).unwrap());
        assert_eq!(
            registry.get("wpc").unwrap().unwrap().state,
            ModuleState::Ready
        );
    }

    #[test]
    fn duplicates_are_rejected() {
        let registry = ModuleRegistry::new();
        registry.register(descriptor("wpc")).unwrap();
        assert!(matches!(
            registry.register(descriptor("wpc")),
            Err(ModuleRegistryError::Duplicate(_))
        ));
    }
}
