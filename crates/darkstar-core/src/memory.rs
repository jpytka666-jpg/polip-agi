//! Session-scoped in-memory context store for Darkstar.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-28 08:25:00
//! REASON FOR CREATION: Provide the minimal Layer 01 memory backend required by the session-scoped memory contract.
//! MECHANICS: Store JSON values under a `(session_id, key)` pair behind a read-write lock; each record carries its creation timestamp and version.
//! SYSTEM PART: Darkstar Core / Layer 01 Context & Memory
//! ARCHITECTURE FUNCTION: Small backend behind the future stable memory interface; deliberately in-memory so storage can later be replaced without changing consumers.
//! DEPENDENCIES/LINKS: serde, serde_json, uuid; consumed by darkstar-core and later by HTTP/session layers.
//! TECH STACK: Rust 2024; selected for safe concurrent access and consistency with the existing Darkstar core.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi / feat/darkstar-module-control
//! ==========================================

use std::{collections::HashMap, sync::RwLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredMemory {
    pub key: String,
    pub value: Value,
    pub created_at_unix_ms: i64,
    pub version: u64,
}

#[derive(Debug, Default)]
pub struct MemoryStore {
    records: RwLock<HashMap<(Uuid, String), StoredMemory>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(
        &self,
        session_id: Uuid,
        key: impl Into<String>,
        value: Value,
        created_at_unix_ms: i64,
    ) -> Result<(), String> {
        let key = key.into();
        let mut records = self
            .records
            .write()
            .map_err(|_| "memory store lock poisoned".to_owned())?;
        let version = records
            .get(&(session_id, key.clone()))
            .map_or(1, |record| record.version + 1);
        records.insert(
            (session_id, key.clone()),
            StoredMemory {
                key,
                value,
                created_at_unix_ms,
                version,
            },
        );
        Ok(())
    }

    pub fn get(
        &self,
        session_id: Uuid,
        key: &str,
    ) -> Result<Option<StoredMemory>, String> {
        let records = self
            .records
            .read()
            .map_err(|_| "memory store lock poisoned".to_owned())?;
        Ok(records.get(&(session_id, key.to_owned())).cloned())
    }
}
