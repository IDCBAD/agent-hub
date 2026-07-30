use serde::{Deserialize, Serialize};

use super::{
    configuration::{ConfigurationObservation, ConfigurationSummary},
    discovery::DiscoveryEvidenceDraft,
    health::HealthStatus,
    runtime::{RuntimeObservation, RuntimeSummary},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "high" => Self::High,
            "medium" => Self::Medium,
            _ => Self::Low,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentTypeDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub icon_key: &'static str,
    pub adapter_version: i64,
}

#[derive(Debug, Clone)]
pub struct AgentDraft {
    pub agent_type_id: String,
    pub display_name: String,
    pub runtime: RuntimeObservation,
    pub configuration: ConfigurationObservation,
    pub health: HealthStatus,
    pub confidence: Confidence,
    pub metadata: serde_json::Value,
    pub evidence: Vec<DiscoveryEvidenceDraft>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSummary {
    pub id: String,
    pub agent_type_id: String,
    pub agent_type_name: String,
    pub display_name: String,
    pub runtime: RuntimeSummary,
    pub configuration: ConfigurationSummary,
    pub health: HealthStatus,
    pub confidence: Confidence,
    pub last_seen_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentOverview {
    #[serde(flatten)]
    pub summary: AgentSummary,
    pub adapter_version: i64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentFilter {
    pub search: Option<String>,
    pub statuses: Option<Vec<HealthStatus>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualLocationRequest {
    pub agent_type_id: String,
    pub path: String,
}
