use serde::Serialize;

#[derive(Debug, Clone)]
pub struct DiscoveryEvidenceDraft {
    pub evidence_type: String,
    pub source: String,
    pub observed_value: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryEvidence {
    pub id: String,
    pub agent_instance_id: String,
    pub evidence_type: String,
    pub source: String,
    pub observed_value: String,
    pub success: bool,
    pub message: String,
    pub observed_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryResult {
    pub run_id: String,
    pub discovered_count: usize,
    pub changed_count: usize,
    pub missing_count: usize,
    pub finished_at: i64,
}
