//! Stdio transport for language-neutral Darkstar plugins.
//!
//! THIS IS VERY IMPORTANT!!!
//! ==========================================
//! AUTHOR: M. SZUL
//! AI MODEL: GPT-5.6 Luna
//! TIMESTAMP: 2026-08-27 21:30:00
//! REASON FOR CREATION: Provide the first real plug-and-play transport for external plugin processes.
//! MECHANICS: Spawn one child process, write one JSON request line to stdin, close stdin, read one JSON result line from stdout, and reject malformed protocol responses.
//! SYSTEM PART: Darkstar Core / Plugin Transport
//! ARCHITECTURE FUNCTION: Allow Rust, Python, C, C++, PowerShell and other executable runtimes to implement the same plugin contract without linking into Darkstar.
//! DEPENDENCIES/LINKS: crate::plugin, crate::plugin_host, serde_json; OS process stdin/stdout.
//! TECH STACK: Rust 2024; std::process selected for the first portable synchronous adapter, keeping the trusted protocol boundary small.
//! LOCAL WORKSPACE: N/A - GitHub-first workspace.
//! GIT COMMIT: PENDING
//! GITHUB METADATA: jpytka666-jpg/polip-agi, branch Darkstar
//! ==========================================

use std::io::Write;
use std::process::{Command, Stdio};

use crate::{
    plugin::{PluginManifest, PluginRequest, PluginResult},
    plugin_host::{PluginAdapter, PluginHostError, TransportKind},
};

pub struct StdioPluginAdapter {
    manifest: PluginManifest,
    program: String,
    args: Vec<String>,
}

impl StdioPluginAdapter {
    pub fn new(manifest: PluginManifest, program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            manifest,
            program: program.into(),
            args,
        }
    }
}

impl PluginAdapter for StdioPluginAdapter {
    fn transport(&self) -> TransportKind {
        TransportKind::Stdio
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn invoke(&self, request: PluginRequest) -> Result<PluginResult, PluginHostError> {
        let input = serde_json::to_string(&request)
            .map_err(|error| PluginHostError::Protocol(format!("encode request: {error}")))?;

        let mut child = Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| PluginHostError::Unavailable(format!("spawn plugin: {error}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            writeln!(stdin, "{input}")
                .map_err(|error| PluginHostError::Unavailable(format!("write plugin stdin: {error}")))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|error| PluginHostError::Unavailable(format!("wait for plugin: {error}")))?;

        if !output.status.success() {
            return Err(PluginHostError::Rejected(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        let response_line = String::from_utf8(output.stdout)
            .map_err(|error| PluginHostError::Protocol(format!("plugin stdout is not UTF-8: {error}")))?;
        let response_line = response_line.lines().next().unwrap_or_default();

        serde_json::from_str(response_line)
            .map_err(|error| PluginHostError::Protocol(format!("decode plugin result: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{plugin::{Capability, PluginId}, API_VERSION};

    #[test]
    fn adapter_reports_stdio_transport() {
        let manifest = PluginManifest {
            api_version: API_VERSION.into(),
            plugin: PluginId { name: "example.echo".into(), version: "0.1.0".into() },
            runtime: "external".into(),
            platform: "any".into(),
            capabilities: vec![Capability { name: "echo".into(), description: "Return input.".into(), read_only: true }],
        };
        let adapter = StdioPluginAdapter::new(manifest, "echo", vec!["{}".into()]);
        assert_eq!(adapter.transport(), TransportKind::Stdio);
    }
}
