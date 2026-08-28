//! Tests for the first session-scoped Darkstar memory contract.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-28 08:15:00
//! REASON FOR CREATION: Define and verify the minimal session-scoped memory contract before implementation.
//! MECHANICS: The test requires writes to remain isolated by session and values to be retrievable by their key.
//! SYSTEM PART: Darkstar Core / Layer 01 Context & Memory
//! ARCHITECTURE FUNCTION: Red-phase contract test for the in-memory backend that will sit behind a stable memory interface.
//! DEPENDENCIES/LINKS: darkstar-core::memory, uuid, serde_json.
//! TECH STACK: Rust 2024 integration test; selected to exercise the public core contract from a consumer boundary.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi / feat/darkstar-module-control
//! ==========================================

use darkstar_core::memory::{MemoryStore, StoredMemory};
use serde_json::json;
use uuid::Uuid;

#[test]
fn stores_and_reads_memory_with_session_isolation() {
    let store = MemoryStore::new();
    let session_a = Uuid::new_v4();
    let session_b = Uuid::new_v4();

    store
        .put(session_a, "project", json!({"name": "Darkstar"}), 1_700_000_000_000)
        .expect("session A write should succeed");

    let found = store
        .get(session_a, "project")
        .expect("session A read should succeed");

    assert_eq!(
        found,
        Some(StoredMemory {
            key: "project".into(),
            value: json!({"name": "Darkstar"}),
            created_at_unix_ms: 1_700_000_000_000,
            version: 1,
        })
    );

    assert_eq!(
        store
            .get(session_b, "project")
            .expect("session B read should succeed"),
        None
    );
}
