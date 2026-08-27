//! Darkstar core contracts.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 20:54:30
//! REASON FOR CREATION: Establish the stable Rust control-plane contract for Darkstar.
//! MECHANICS: Re-exports versioned domain contracts for sessions, plugins, events, policy and audit.
//! SYSTEM PART: Darkstar Core
//! ARCHITECTURE FUNCTION: Model-agnostic control-plane foundation shared by all transports and plugins.
//! DEPENDENCIES/LINKS: serde, serde_json, uuid, thiserror, tracing; consumed by Darkstar server and adapters.
//! TECH STACK: Rust 2024; selected for memory safety, concurrency and a stable infrastructure core.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

pub mod audit;
pub mod event;
pub mod plugin;
pub mod policy;
pub mod session;

pub const API_VERSION: &str = "darkstar.core/v1";
