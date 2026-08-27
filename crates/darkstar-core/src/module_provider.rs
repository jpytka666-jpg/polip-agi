//! Provider contract for executing module lifecycle commands.
//!
//! SYSTEM PART: Darkstar Core / Provider Boundary
//! ARCHITECTURE FUNCTION: Keep orchestration independent from systemd, Docker, Kali, Azure, Windows or other runtimes.
//! SECURITY: Providers never decide authorization; callers must pass policy-approved commands.
//! TECH STACK: Rust 2024 + serde + typed errors.

use serde::{Deserialize, Serialize};

use crate::module_state::{ModuleCommand, ModuleCommandRequest, ModuleState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderContext {
    pub request_id: String,
    pub principal_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderResult {
    pub provider_id: String,
    pub module_id: String,
    pub command: ModuleCommand,
    pub resulting_state: ModuleState,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider does not support module command")]
    Unsupported,
    #[error("provider rejected command: {0}")]
    Rejected(String),
    #[error("provider unavailable: {0}")]
    Unavailable(String),
}

pub trait ModuleProvider: Send + Sync {
    fn provider_id(&self) -> &str;

    fn supports(&self, request: &ModuleCommandRequest) -> bool;

    fn apply(
        &self,
        request: &ModuleCommandRequest,
        context: &ProviderContext,
    ) -> Result<ProviderResult, ProviderError>;
}

#[derive(Debug, Default)]
pub struct DryRunProvider;

impl ModuleProvider for DryRunProvider {
    fn provider_id(&self) -> &str {
        "dry-run"
    }

    fn supports(&self, _request: &ModuleCommandRequest) -> bool {
        true
    }

    fn apply(
        &self,
        request: &ModuleCommandRequest,
        context: &ProviderContext,
    ) -> Result<ProviderResult, ProviderError> {
        let resulting_state = match request.command {
            ModuleCommand::Start => ModuleState::Ready,
            ModuleCommand::Stop => ModuleState::Offline,
            ModuleCommand::Restart => ModuleState::Ready,
        };

        Ok(ProviderResult {
            provider_id: self.provider_id().into(),
            module_id: request.module_id.clone(),
            command: request.command.clone(),
            resulting_state,
            message: format!("dry-run {}: {}", context.request_id, request.reason),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_provider_never_executes_external_processes() {
        let provider = DryRunProvider;
        let request = ModuleCommandRequest {
            module_id: "wpc-engine".into(),
            command: ModuleCommand::Start,
            reason: "test".into(),
        };
        let context = ProviderContext {
            request_id: "req-1".into(),
            principal_id: "human:marcin".into(),
            reason: "test".into(),
        };

        let result = provider.apply(&request, &context).unwrap();
        assert_eq!(result.provider_id, "dry-run");
        assert_eq!(result.resulting_state, ModuleState::Ready);
    }
}
