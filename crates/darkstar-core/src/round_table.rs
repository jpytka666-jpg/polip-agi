//! Round Table participant contracts for human and agent collaboration.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 23:29:00
//! REASON FOR CREATION: Give Darkstar one shared session model for humans, Claude CLI and other agents participating in a coordinated decision.
//! MECHANICS: Participants have explicit identities and capability scopes. Messages belong to a table session; this contract carries collaboration state but does not execute tools.
//! SYSTEM PART: Darkstar Core / Agent Round Table
//! ARCHITECTURE FUNCTION: Provide the common language between the browser control deck and remote agent clients before policy-gated execution.
//! DEPENDENCIES/LINKS: session, capability contracts, event bus, future persistence and agent adapters.
//! TECH STACK: Rust 2024 + serde; selected for typed identity and JSON interoperability.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-control-deck
//! ==========================================

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
    fn human_and_agent_can_share_one_table_contract() {
        let table = RoundTable {
            table_id: Uuid::new_v4(),
            topic: "model analysis".into(),
            participants: vec![
                Participant {
                    participant_id: "human:marcin".into(),
                    kind: ParticipantKind::Human,
                    display_name: "Marcin".into(),
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

        assert_eq!(table.participants.len(), 2);
        assert_ne!(
            table.participants[0].capabilities,
            table.participants[1].capabilities
        );
    }
}
