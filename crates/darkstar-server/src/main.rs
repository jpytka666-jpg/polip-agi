//! Minimal Darkstar process entrypoint.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 20:54:30
//! REASON FOR CREATION: Provide the first runnable Darkstar process around the core contracts.
//! MECHANICS: Starts a deterministic health-style process and exposes the core API version for later transport wiring.
//! SYSTEM PART: Darkstar Server
//! ARCHITECTURE FUNCTION: Runtime entrypoint that will host HTTP/MCP, plugin, session and workflow adapters.
//! DEPENDENCIES/LINKS: darkstar-core; future transport modules will attach here without changing domain contracts.
//! TECH STACK: Rust 2024; selected for the long-running control plane and safe asynchronous expansion.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

fn main() {
    println!("darkstar api={} status=ready", darkstar_core::API_VERSION);
}

#[cfg(test)]
mod tests {
    #[test]
    fn core_api_version_is_present() {
        assert_eq!(darkstar_core::API_VERSION, "darkstar.core/v1");
    }
}
