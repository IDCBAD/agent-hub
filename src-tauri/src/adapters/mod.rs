mod claude;
mod codex;
mod common;
mod hermes;
mod kimi;

use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{
    domain::{
        agent::{AgentDraft, AgentTypeDescriptor},
        configuration::ConfigurationObservation,
        runtime::RuntimeObservation,
    },
    error::AppError,
};
use common::{ConfiguredAdapter, DiscoveryContext};

pub trait AgentAdapter: Send + Sync {
    fn descriptor(&self) -> AgentTypeDescriptor;
    fn detect_runtime(&self, context: &DiscoveryContext) -> Result<RuntimeObservation, AppError>;
    fn configuration_candidates(
        &self,
        context: &DiscoveryContext,
    ) -> Vec<(std::path::PathBuf, String)>;
    fn detect_configuration(
        &self,
        path: &Path,
        source: &str,
        allow_missing: bool,
        context: &DiscoveryContext,
    ) -> Result<ConfigurationObservation, AppError>;
    fn discover(&self, context: &DiscoveryContext) -> Result<Vec<AgentDraft>, AppError>;
    fn detect_path(
        &self,
        path: &Path,
        source: &str,
        allow_missing: bool,
    ) -> Result<AgentDraft, AppError>;
}

#[derive(Clone)]
pub struct AdapterRegistry {
    adapters: Vec<Arc<ConfiguredAdapter>>,
    by_id: HashMap<String, Arc<ConfiguredAdapter>>,
}

impl AdapterRegistry {
    pub fn standard() -> Self {
        let adapters = vec![
            claude::adapter(),
            codex::adapter(),
            hermes::adapter(),
            kimi::adapter(),
        ]
        .into_iter()
        .map(Arc::new)
        .collect::<Vec<_>>();
        let by_id = adapters
            .iter()
            .map(|adapter| (adapter.descriptor().id.to_owned(), Arc::clone(adapter)))
            .collect();
        Self { adapters, by_id }
    }

    pub fn descriptors(&self) -> Vec<AgentTypeDescriptor> {
        self.adapters
            .iter()
            .map(|adapter| adapter.descriptor())
            .collect()
    }

    pub fn discover_all(&self) -> Result<Vec<AgentDraft>, AppError> {
        let context = DiscoveryContext::from_system()?;
        let mut drafts = Vec::new();
        for adapter in &self.adapters {
            drafts.extend(adapter.discover(&context)?);
        }
        Ok(drafts)
    }

    pub fn detect_path(
        &self,
        agent_type_id: &str,
        path: &Path,
        source: &str,
        allow_missing: bool,
    ) -> Result<AgentDraft, AppError> {
        let adapter = self.by_id.get(agent_type_id).ok_or_else(|| {
            AppError::new(
                "unsupported_agent_type",
                "不支持指定的 Agent 类型。",
                true,
                Some("请选择 Claude Code、Codex、Hermes Agent 或 Kimi Code。"),
            )
        })?;
        adapter.detect_path(path, source, allow_missing)
    }
}
