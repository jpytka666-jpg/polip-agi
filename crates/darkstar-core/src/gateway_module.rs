// darkstar-header-v1
// po co: gateway_module.rs
// nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie
// autor: Marcin
// powstal: 2026-09-01
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: Claude Opus 5
//! TIMESTAMP: 2026-09-01 22:40:00
//! REASON FOR CREATION: Wpiecie natywnej bramy Darkstar do rejestru modulow control plane (Task 7, Step 7.4). Dokumentacja zadala tego modulu; kod go nie mial.
//! MECHANICS: Buduje deskryptor `darkstar-native-gateway` ze zdolnosciami network.inspect,
//! network.start, network.stop i network.restart, rejestruje go w ModuleRegistry i rozstrzyga,
//! ktore z tych zdolnosci sa faktycznie wykonalne. Dzis wykonalny jest wylacznie odczyt -
//! warstwy mutujacej nie ma, wiec `is_implemented` mowi o tym wprost, zamiast udawac gotowosc.
//! SYSTEM PART: Darkstar Core / Module registry - modul bramy.
//! ARCHITECTURE FUNCTION: Domyka petle miedzy dokumentacja a kodem: brama przestaje byc
//! skryptem obok systemu i staje sie modulem widocznym w rejestrze, z jawnie opisanym zakresem.
//! DEPENDENCIES/LINKS: module_registry (ModuleRegistry, ModuleDescriptor), module_state
//! (ModuleState, ModuleCommand), gateway_provider (odczyt), gateway_status (kontrakt).
//! TECH STACK: Rust 2024 + serde, bez nowych zaleznosci - modul opisuje i rejestruje, nie wykonuje.
//! LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
//! ==========================================

use crate::module_registry::{ModuleRegistry, ModuleRegistryError};
use crate::module_state::{ModuleCommand, ModuleDescriptor, ModuleState};

pub const GATEWAY_MODULE_ID: &str = "darkstar-native-gateway";

/// Zdolnosci deklarowane przez modul bramy, w kolejnosci z planu (Task 7, Step 7.4).
pub const GATEWAY_CAPABILITIES: [&str; 4] = [
    "network.inspect",
    "network.start",
    "network.stop",
    "network.restart",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GatewayModuleError {
    #[error("capability {0} is declared but has no implementation yet")]
    NotImplemented(&'static str),
}

/// Dzis brama umie wylacznie czytac swoj stan. Reszta zdolnosci jest zadeklarowana,
/// zeby rejestr opisywal docelowy ksztalt modulu, ale nie udaje gotowosci.
pub fn capability_is_implemented(capability: &str) -> bool {
    capability == "network.inspect"
}

/// Odpowiada, czy polecenie sterujace da sie dzis wykonac. Zwraca blad zamiast
/// przepuszczac zadanie do warstwy, ktorej nie ma.
pub fn gateway_command_support(command: ModuleCommand) -> Result<(), GatewayModuleError> {
    let capability = command.capability();
    if capability_is_implemented(capability) {
        Ok(())
    } else {
        Err(GatewayModuleError::NotImplemented(capability))
    }
}

pub fn gateway_module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor {
        module_id: GATEWAY_MODULE_ID.into(),
        name: "Darkstar Native Gateway".into(),
        system: "Darkstar".into(),
        language: "Rust".into(),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        capabilities: GATEWAY_CAPABILITIES
            .iter()
            .map(|c| (*c).to_string())
            .collect(),
        dependencies: vec!["networkmanager".into(), "nftables".into()],
        // Modul jest gotowy do tego, co potrafi: odczytu. Stan mowi o dostepnosci
        // modulu, a health precyzuje jego dzisiejszy zakres.
        state: ModuleState::Ready,
        health: Some("read-only".into()),
    }
}

pub fn register_gateway_module(registry: &ModuleRegistry) -> Result<(), ModuleRegistryError> {
    registry.register(gateway_module_descriptor())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_registry::ModuleRegistry;
    use crate::module_state::{ModuleCommand, ModuleState};

    #[test]
    fn descriptor_declares_the_planned_capabilities() {
        let d = gateway_module_descriptor();
        assert_eq!(d.module_id, GATEWAY_MODULE_ID);
        assert_eq!(
            d.capabilities,
            vec![
                "network.inspect".to_string(),
                "network.start".to_string(),
                "network.stop".to_string(),
                "network.restart".to_string(),
            ]
        );
        assert_eq!(d.system, "Darkstar");
        assert_eq!(d.language, "Rust");
    }

    #[test]
    fn only_inspection_is_implemented_today() {
        assert!(capability_is_implemented("network.inspect"));
        for planned in ["network.start", "network.stop", "network.restart"] {
            assert!(
                !capability_is_implemented(planned),
                "{planned} must not claim readiness without a mutating provider"
            );
        }
    }

    #[test]
    fn mutating_commands_are_refused_while_unimplemented() {
        for command in [
            ModuleCommand::Start,
            ModuleCommand::Stop,
            ModuleCommand::Restart,
        ] {
            assert_eq!(
                gateway_command_support(command.clone()),
                Err(GatewayModuleError::NotImplemented(command.capability()))
            );
        }
    }

    #[test]
    fn registers_once_and_rejects_a_duplicate() {
        let registry = ModuleRegistry::new();
        assert!(register_gateway_module(&registry).is_ok());

        let stored = registry.get(GATEWAY_MODULE_ID).unwrap().unwrap();
        assert_eq!(stored.state, ModuleState::Ready);
        assert_eq!(stored.health.as_deref(), Some("read-only"));

        // Drugie wywolanie nie moze cicho nadpisac wpisu.
        assert!(register_gateway_module(&registry).is_err());
        assert_eq!(registry.list().unwrap().len(), 1);
    }

    #[test]
    fn descriptor_carries_no_secret_fields() {
        let json = serde_json::to_string(&gateway_module_descriptor()).unwrap();
        for forbidden in ["psk", "password", "token", "secret"] {
            assert!(!json.contains(forbidden), "descriptor leaked {forbidden}");
        }
    }
}
