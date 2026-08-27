//! Deterministic capability candidate selection for Darkstar.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:25:00
//! REASON FOR CREATION: Select the best available plugin candidates for a requested capability without granting execution authority.
//! MECHANICS: Query the read-only CapabilityIndex, optionally prefer a runtime and platform, and return deterministic candidates. This module never executes plugins and never makes authorization decisions.
//! SYSTEM PART: Darkstar Core / Capability Selector
//! ARCHITECTURE FUNCTION: Bridge capability discovery to the policy layer by narrowing a large plugin library to a small, deterministic candidate set.
//! DEPENDENCIES/LINKS: crate::capability_index::CapabilityIndex; crate::capability_index::CapabilityMatch.
//! TECH STACK: Rust 2024 + standard sorting; selected for deterministic, dependency-free selection in the trusted core.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use crate::capability_index::{CapabilityIndex, CapabilityMatch};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySelection {
    pub candidates: Vec<CapabilityMatch>,
}

impl CapabilitySelection {
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn first(&self) -> Option<&CapabilityMatch> {
        self.candidates.first()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CapabilitySelector;

impl CapabilitySelector {
    pub fn select(
        index: &CapabilityIndex,
        capability_name: &str,
        preferred_runtime: Option<&str>,
        preferred_platform: Option<&str>,
    ) -> CapabilitySelection {
        let mut candidates = index.find(capability_name).to_vec();

        candidates.sort_by(|left, right| {
            preference_rank(left, preferred_runtime, preferred_platform)
                .cmp(&preference_rank(right, preferred_runtime, preferred_platform))
                .then_with(|| left.plugin.name.cmp(&right.plugin.name))
                .then_with(|| left.plugin.version.cmp(&right.plugin.version))
                .then_with(|| left.runtime.cmp(&right.runtime))
                .then_with(|| left.platform.cmp(&right.platform))
        });

        CapabilitySelection { candidates }
    }
}

fn preference_rank(
    candidate: &CapabilityMatch,
    preferred_runtime: Option<&str>,
    preferred_platform: Option<&str>,
) -> (u8, u8) {
    let runtime_rank = match preferred_runtime {
        Some(runtime) if candidate.runtime == runtime => 0,
        _ => 1,
    };

    let platform_rank = match preferred_platform {
        Some(platform) if candidate.platform == platform => 0,
        Some(_) if candidate.platform == "any" => 1,
        _ => 2,
    };

    (runtime_rank, platform_rank)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{Capability, PluginId, PluginManifest};
    use crate::registry::PluginRegistry;
    use crate::API_VERSION;

    fn manifest(
        name: &str,
        version: &str,
        runtime: &str,
        platform: &str,
        capability_name: &str,
    ) -> PluginManifest {
        PluginManifest {
            api_version: API_VERSION.into(),
            plugin: PluginId {
                name: name.into(),
                version: version.into(),
            },
            runtime: runtime.into(),
            platform: platform.into(),
            capabilities: vec![Capability {
                name: capability_name.into(),
                description: format!("Capability {capability_name}"),
                read_only: true,
            }],
        }
    }

    #[test]
    fn prefers_requested_runtime_and_platform() {
        let registry = PluginRegistry::new();

        registry
            .register(manifest(
                "example.python",
                "0.1.0",
                "python",
                "any",
                "echo",
            ))
            .expect("register python");
        registry
            .register(manifest(
                "example.rust",
                "0.1.0",
                "rust",
                "linux-x86_64",
                "echo",
            ))
            .expect("register rust");

        let index = CapabilityIndex::from_registry(&registry).expect("build index");
        let selection = CapabilitySelector::select(
            &index,
            "echo",
            Some("rust"),
            Some("linux-x86_64"),
        );

        assert_eq!(
            selection.first().map(|candidate| candidate.plugin.name.as_str()),
            Some("example.rust")
        );
    }

    #[test]
    fn selection_is_deterministic() {
        let registry = PluginRegistry::new();

        registry
            .register(manifest(
                "example.beta",
                "0.1.0",
                "python",
                "any",
                "echo",
            ))
            .expect("register beta");
        registry
            .register(manifest(
                "example.alpha",
                "0.1.0",
                "python",
                "any",
                "echo",
            ))
            .expect("register alpha");

        let index = CapabilityIndex::from_registry(&registry).expect("build index");
        let selection = CapabilitySelector::select(&index, "echo", None, None);

        assert_eq!(
            selection
                .candidates
                .iter()
                .map(|candidate| candidate.plugin.name.as_str())
                .collect::<Vec<_>>(),
            vec!["example.alpha", "example.beta"]
        );
    }

    #[test]
    fn unknown_capability_returns_no_candidates() {
        let registry = PluginRegistry::new();
        let index = CapabilityIndex::from_registry(&registry).expect("build index");
        let selection = CapabilitySelector::select(&index, "missing", None, None);

        assert!(selection.is_empty());
    }
}
