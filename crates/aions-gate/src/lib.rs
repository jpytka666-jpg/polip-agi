//! ================================================================================
//! AIONS ARCHITECTURE - PROVENANCE / READ-ONLY CONTEXT HEADER
//! ================================================================================
//! AUTHOR:              Claude Code agent, working for Marcin (jpytka666-jpg)
//! AI MODEL:            Anthropic Claude via Claude Code remote session (exact model id WITHHELD by harness policy)
//! TIMESTAMP:           2026-09-25T23:14:45Z
//! REASON FOR CREATION: Expose the enforcement logic as a library so integration tests and future MCP services can call it without shelling out.
//! MECHANICS:           Re-exports the provenance, gates and json modules.
//! SYSTEM PART:         polip-agi / aions-gate
//! ARCHITECTURE FUNC:   Library entry point of the enforcement plane.
//! DEPENDENCIES/LINKS:  src/provenance.rs, src/gates.rs, src/json.rs, src/main.rs
//! TECH STACK:          Rust 2024, std only
//! LOCAL WORKSPACE:     polip-agi/crates/aions-gate
//! GIT COMMIT:          PENDING
//! GITHUB METADATA:     jpytka666-jpg/polip-agi, branch claude/autonomous-automation-agent-ocxwke
//! ================================================================================

pub mod gates;
pub mod json;
pub mod provenance;
