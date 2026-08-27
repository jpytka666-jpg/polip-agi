//! Round Table participant contracts for human and agent collaboration.
//!
//! SYSTEM PART: Darkstar Core / Agent Round Table
//! ARCHITECTURE FUNCTION: Shared typed identity for browser, CLI and agent participants.
//! TECH STACK: Rust 2024 + serde + uuid.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantKind {
    Human,
    Agent,
    Cli,
    Service,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Participant {
    pub participant_id: String,
    pub kind: ParticipantKind,
    pub display_name: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoundTable {
    pub table_id: Uuid,
    pub topic: String,
    pub participants: Vec<Participant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableMessage {
    pub message_id: Uuid,
    pub table_id: Uuid,
    pub participant_id: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_and_agent_have_distinct_capability_scopes() {
        let table = RoundTable {
            table_id: Uuid::new_v4(),
            topic: "model analysis".into(),
            participants: vec![
                Participant {
                    participant_id: "human:operator".into(),
                    kind: ParticipantKind::Human,
                    display_name: "Human".into(),
                    capabilities: vec!["approve".into()],
                },
                Participant {
                    participant_id: "agent:claude".into(),
                    kind: ParticipantKind::Agent,
                    display_name: "Claude".into(),
                    capabilities: vec!["propose".into()],
                },
            ],
        };

        assert_ne!(table.participants[0].capabilities, table.participants[1].capabilities);
    }
}
