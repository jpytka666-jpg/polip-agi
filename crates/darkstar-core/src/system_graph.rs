//! Read-only architecture graph projection for Darkstar.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 22:20:00
//! REASON FOR CREATION: Provide one architecture-aware graph contract for inspecting files, modules, systems, languages and their lineage without duplicating WPC's memory-graph engine.
//! MECHANICS: Stores a deterministic list of architecture nodes and typed relationships. Metadata is provenance-aware: creation time may be unknown until a Git provider derives it from commit history.
//! SYSTEM PART: Darkstar Core / System Graph
//! ARCHITECTURE FUNCTION: Expose a read-only projection used by the interactive System Graph UI and future GitHub/AIONS/WPC providers.
//! DEPENDENCIES/LINKS: Existing WPC memory-graph snapshot shape is the conceptual interoperability target; Darkstar plugin, registry and audit contracts remain separate.
//! TECH STACK: Rust 2024 + serde + serde_json; selected for deterministic serialization and zero new runtime dependencies.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch feat/darkstar-system-graph
//! ==========================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureNodeKind {
    Repository,
    Directory,
    File,
    Module,
    Dependency,
    Runtime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchitectureNode {
    pub id: String,
    pub kind: ArchitectureNodeKind,
    pub name: String,
    pub path: Option<String>,
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub language: Option<String>,
    pub system: Option<String>,
    pub role: Option<String>,
    pub created_at: Option<String>,
    pub last_modified_at: Option<String>,
    pub commit: Option<String>,
    pub status: String,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchitectureEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchitectureSnapshot {
    pub snapshot_id: String,
    pub generated_at: String,
    pub source: String,
    pub nodes: Vec<ArchitectureNode>,
    pub edges: Vec<ArchitectureEdge>,
}

pub fn current_snapshot() -> ArchitectureSnapshot {
    let nodes = vec![
        repo("polip", "polip-agi", "Darkstar", "active"),
        runtime("session", "Session", "execution context", "Rust", "active"),
        file("darkstar-core", "crates/darkstar-core/src/lib.rs", "Rust", "control-plane", "2026-08-27T19:54:44Z"),
        file("policy", "crates/darkstar-core/src/policy.rs", "Rust", "security policy", "2026-08-27T20:00:00Z"),
        file("discovery", "crates/darkstar-core/src/discovery.rs", "Rust", "plugin discovery", "2026-08-27T20:56:04Z"),
        file("capability-index", "crates/darkstar-core/src/capability_index.rs", "Rust", "capability index", "2026-08-27T20:56:04Z"),
        file("capability-selector", "crates/darkstar-core/src/capability_selector.rs", "Rust", "deterministic selector", "2026-08-27T21:27:03Z"),
        file("capability-gate", "crates/darkstar-core/src/capability_gate.rs", "Rust", "policy gate", "2026-08-27T21:32:53Z"),
        file("plugin", "crates/darkstar-core/src/plugin.rs", "Rust", "language-neutral plugin contract", "2026-08-27T19:54:59Z"),
        file("registry", "crates/darkstar-core/src/registry.rs", "Rust", "plugin registry", "2026-08-27T20:21:39Z"),
        file("http", "crates/darkstar-server/src/http.rs", "Rust", "HTTP transport", "2026-08-27T20:01:52Z"),
        file("echo-python", "plugins/echo-python/plugin.py", "Python", "example tentacle", "2026-08-27T19:00:00Z"),
        runtime("audit", "Audit", "execution trace", "Rust", "active"),
        runtime("aions-peer", "AIONS / Ionis", "trusted peer", "remote", "active"),
        runtime("wpc-engine", "WPC Engine", "computational substrate", "Rust", "active"),
        runtime("aions-server-wiedzy", "AIONS Server Wiedzy", "memory / retrieval", "Python + Rust", "active"),
        runtime("ghost-gate", "Ghost Gate", "network egress boundary", "Linux", "design-contract"),
    ];

    let edges = vec![
        edge("e1", "repo:polip", "file:darkstar-core", "contains"),
        edge("e2", "runtime:session", "file:policy", "enters"),
        edge("e3", "file:capability-index", "file:capability-selector", "feeds"),
        edge("e4", "file:capability-selector", "file:capability-gate", "selects_for"),
        edge("e5", "file:policy", "file:capability-gate", "authorizes"),
        edge("e6", "file:discovery", "file:capability-index", "indexes"),
        edge("e7", "file:registry", "file:discovery", "registers"),
        edge("e8", "file:plugin", "file:registry", "describes"),
        edge("e9", "file:http", "file:capability-gate", "exposes"),
        edge("e10", "file:capability-gate", "runtime:aions-peer", "guards"),
        edge("e11", "file:capability-gate", "runtime:wpc-engine", "guards"),
        edge("e12", "file:capability-gate", "runtime:aions-server-wiedzy", "guards"),
        edge("e13", "file:http", "file:echo-python", "hosts"),
        edge("e14", "runtime:aions-peer", "runtime:ghost-gate", "egress_through"),
        edge("e15", "file:capability-gate", "runtime:audit", "records"),
    ];

    ArchitectureSnapshot {
        snapshot_id: "darkstar-system-graph-v1".into(),
        generated_at: "2026-08-27T22:20:00Z".into(),
        source: "darkstar_static_projection_v1".into(),
        nodes,
        edges,
    }
}

fn repo(id: &str, name: &str, branch: &str, status: &str) -> ArchitectureNode {
    ArchitectureNode {
        id: format!("repo:{id}"),
        kind: ArchitectureNodeKind::Repository,
        name: name.into(),
        path: None,
        repository: Some(name.into()),
        branch: Some(branch.into()),
        language: Some("mixed".into()),
        system: Some("Darkstar".into()),
        role: Some("system root".into()),
        created_at: None,
        last_modified_at: None,
        commit: None,
        status: status.into(),
        provenance: "github_repo_metadata".into(),
    }
}

fn file(id: &str, path: &str, language: &str, role: &str, modified: &str) -> ArchitectureNode {
    ArchitectureNode {
        id: format!("file:{id}"),
        kind: ArchitectureNodeKind::File,
        name: path.rsplit('/').next().unwrap_or(path).into(),
        path: Some(path.into()),
        repository: Some("jpytka666-jpg/polip-agi".into()),
        branch: Some("Darkstar".into()),
        language: Some(language.into()),
        system: Some("Darkstar".into()),
        role: Some(role.into()),
        created_at: None,
        last_modified_at: Some(modified.into()),
        commit: None,
        status: "active".into(),
        provenance: "github_file_metadata;creation_not_yet_derived".into(),
    }
}

fn runtime(id: &str, name: &str, role: &str, language: &str, status: &str) -> ArchitectureNode {
    ArchitectureNode {
        id: format!("runtime:{id}"),
        kind: ArchitectureNodeKind::Runtime,
        name: name.into(),
        path: None,
        repository: None,
        branch: None,
        language: Some(language.into()),
        system: Some(name.into()),
        role: Some(role.into()),
        created_at: None,
        last_modified_at: None,
        commit: None,
        status: status.into(),
        provenance: "architecture_projection".into(),
    }
}

fn edge(id: &str, from: &str, to: &str, kind: &str) -> ArchitectureEdge {
    ArchitectureEdge {
        id: id.into(),
        from: from.into(),
        to: to.into(),
        kind: kind.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_contains_darkstar_core_and_external_systems() {
        let snapshot = current_snapshot();
        assert!(snapshot.nodes.iter().any(|n| n.id == "file:capability-gate"));
        assert!(snapshot.nodes.iter().any(|n| n.id == "runtime:wpc-engine"));
        assert!(snapshot.nodes.iter().any(|n| n.id == "runtime:ghost-gate"));
        assert!(snapshot.edges.iter().any(|e| e.kind == "egress_through"));
    }

    #[test]
    fn json_round_trip_preserves_metadata() {
        let snapshot = current_snapshot();
        let encoded = serde_json::to_string(&snapshot).expect("serialize");
        let decoded: ArchitectureSnapshot = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(snapshot, decoded);
    }
}
