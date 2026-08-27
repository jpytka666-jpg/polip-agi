//! Darkstar core contracts.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:55:00
//! REASON FOR CREATION: Expose the stable Rust control-plane contracts used by Darkstar.
//! MECHANICS: Re-exports versioned domain contracts for identity, sessions, plugins, events, policy, audit, discovery, capabilities and the Ghost Gate boundary.
//! SYSTEM PART: Darkstar Core
//! ARCHITECTURE FUNCTION: Shared trusted-core foundation for all transports, plugins and workflows.
//! DEPENDENCIES/LINKS: serde, serde_json, uuid, thiserror, tracing; consumed by Darkstar server and adapters.
//! TECH STACK: Rust 2024; selected for memory safety, deterministic data handling and safe concurrency.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-ghost-gate-contract
//! ==========================================

pub mod audit;
pub mod capability_gate;
pub mod capability_index;
pub mod capability_selector;
pub mod discovery;
pub mod event;
pub mod ghost_gate;
pub mod plugin;
pub mod plugin_host;
pub mod policy;
pub mod registry;
pub mod session;
pub mod stdio;

pub const API_VERSION: &str = "darkstar.core/v1";
