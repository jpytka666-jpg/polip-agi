//! Live execution event stream primitives for the Darkstar browser graph.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:45:00
//! REASON FOR CREATION: Provide a small in-memory event stream so the browser can observe a concrete Darkstar run as it moves through architecture nodes.
//! MECHANICS: Each run owns a Tokio broadcast channel keyed by run UUID. Publishers append structured execution events; SSE subscribers receive those events without giving the UI execution authority.
//! SYSTEM PART: Darkstar Server / Live Run Graph
//! ARCHITECTURE FUNCTION: Bridge existing DarkstarEvent-compatible execution signals to the read-only browser visualization.
//! DEPENDENCIES/LINKS: tokio broadcast, uuid; future execution/audit adapters can publish the same run events.
//! TECH STACK: Rust 2024 + Tokio broadcast; selected for bounded, low-overhead fan-out inside the current single-process runtime.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-live-run-graph
//! ==========================================

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunEvent {
    pub run_id: Uuid,
    pub sequence: u64,
    pub node_id: String,
    pub event_type: String,
    pub status: String,
    pub message: Option<String>,
    pub timestamp_unix_ms: i64,
}

#[derive(Clone, Default)]
pub struct RunStreamHub {
    runs: Arc<RwLock<HashMap<Uuid, broadcast::Sender<RunEvent>>>>,
}

impl RunStreamHub {
    pub async fn sender(&self, run_id: Uuid) -> broadcast::Sender<RunEvent> {
        if let Some(sender) = self.runs.read().await.get(&run_id).cloned() {
            return sender;
        }

        let mut runs = self.runs.write().await;
        runs.entry(run_id)
            .or_insert_with(|| broadcast::channel(64).0)
            .clone()
    }

    pub async fn publish(&self, event: RunEvent) -> usize {
        self.sender(event.run_id).await.send(event).unwrap_or(0)
    }

    pub async fn subscribe(&self, run_id: Uuid) -> broadcast::Receiver<RunEvent> {
        self.sender(run_id).await.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn subscribers_receive_published_run_event() {
        let hub = RunStreamHub::default();
        let run_id = Uuid::new_v4();
        let mut receiver = hub.subscribe(run_id).await;

        hub.publish(RunEvent {
            run_id,
            sequence: 1,
            node_id: "policy".into(),
            event_type: "policy.checked".into(),
            status: "allow".into(),
            message: Some("github.read allowed".into()),
            timestamp_unix_ms: 1,
        })
        .await;

        let event = receiver.recv().await.expect("event");
        assert_eq!(event.run_id, run_id);
        assert_eq!(event.node_id, "policy");
        assert_eq!(event.event_type, "policy.checked");
    }
}
