//! Darkstar core contracts.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:55:00
//! REASON FOR CREATION: Expose the stable Rust control-plane contracts used by Darkstar.
//! MECHANICS: Re-exports versioned domain contracts for identity, sessions, plugins, events, policy, audit, discovery, capability indexing, gateway and module lifecycle, network topology, orchestration, the Round Table and the Ghost Gate boundary.
//! SYSTEM PART: Darkstar Core
//! ARCHITECTURE FUNCTION: Model-agnostic control-plane foundation shared by transports, plugins, workflows and AIONS ecosystem modules, including the trusted-core Ghost Gate boundary.
//! DEPENDENCIES/LINKS: serde, serde_json, uuid, thiserror, tracing; consumed by Darkstar server and adapters.
//! TECH STACK: Rust 2024; selected for memory safety, deterministic data handling and safe concurrency.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch integration/2026-09-16-salvage
//! REVISION 2026-09-16 (Claude Opus 5, on Marcin instruction): merged feat/darkstar-ghost-gate-contract into the headscale/hotspot line. The working branch had reduced this memo block to two lines; the full block is restored per AGENTS.md section 2. ghost_gate is declared alongside the gateway, module, topology and orchestration modules.
//! ==========================================

pub mod audit;
pub mod capability_gate;
pub mod capability_index;
pub mod capability_selector;
pub mod context_client;
pub mod discovery;
pub mod event;
pub mod gateway_module;
pub mod gateway_provider;
pub mod gateway_status;
pub mod ghost_gate;
pub mod memory;
pub mod module_execution;
pub mod module_provider;
pub mod module_registry;
pub mod module_state;
pub mod network_topology;
pub mod orchestrator;
pub mod plugin;
pub mod plugin_host;
pub mod policy;
pub mod registry;
pub mod round_table;
pub mod service_status;
pub mod session;
pub mod stdio;
pub mod system_graph;

pub const API_VERSION: &str = "darkstar.core/v1";
