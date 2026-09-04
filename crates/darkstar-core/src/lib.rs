//! Darkstar core contracts.
//!
//! SYSTEM PART: Darkstar Core
//! ARCHITECTURE FUNCTION: Model-agnostic control-plane foundation shared by transports, plugins, workflows and AIONS ecosystem modules.
//! TECH STACK: Rust 2024.

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
