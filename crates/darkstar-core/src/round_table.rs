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
//! ARCHITECTURE FUNCTION: Provide the common language between the browser control deck and remote agent clients before policy-gated execution. Shared typed identity for browser, CLI and agent participants.
//! DEPENDENCIES/LINKS: session, capability contracts, event bus, future persistence and agent adapters. Design is specified in DARKSTAR_LAYER_03_AGENT_ROUND_TABLE.md.
//! TECH STACK: Rust 2024 + serde + uuid; selected for typed identity and JSON interoperability.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch integration/2026-09-16-salvage
//! REVISION 2026-09-16 (Claude Opus 5, on Marcin instruction): memo block restored. It was written on feat/darkstar-control-deck and reduced to three lines when this file reached the headscale/hotspot line, against AGENTS.md section 2. Code is unchanged; the generalised test names from the working branch are kept.
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

        assert_ne!(
            table.participants[0].capabilities,
            table.participants[1].capabilities
        );
    }
}
