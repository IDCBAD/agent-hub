use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Config,
    Prompt,
    Skill,
    Mcp,
    Identity,
    Memory,
    Other,
}

impl ResourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Prompt => "prompt",
            Self::Skill => "skill",
            Self::Mcp => "mcp",
            Self::Identity => "identity",
            Self::Memory => "memory",
            Self::Other => "other",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "config" => Self::Config,
            "prompt" => Self::Prompt,
            "skill" => Self::Skill,
            "mcp" => Self::Mcp,
            "identity" => Self::Identity,
            "memory" => Self::Memory,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceObservation {
    pub kind: ResourceKind,
    pub logical_key: String,
    pub path: PathBuf,
    pub normalized_path: PathBuf,
    pub format: String,
    pub scope: String,
    pub is_sensitive: bool,
    pub exists: bool,
    pub writable: bool,
    pub content_hash: Option<String>,
    pub modified_at: Option<i64>,
    pub size_bytes: Option<i64>,
    pub entry_count: Option<i64>,
    pub scan_truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: String,
    pub agent_instance_id: String,
    pub agent_display_name: String,
    pub kind: ResourceKind,
    pub logical_key: String,
    pub path: String,
    pub format: String,
    pub scope: String,
    pub is_sensitive: bool,
    pub exists: bool,
    pub writable: bool,
    pub content_hash: Option<String>,
    pub modified_at: Option<i64>,
    pub size_bytes: Option<i64>,
    pub entry_count: Option<i64>,
    pub scan_truncated: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceFilter {
    pub search: Option<String>,
    pub agent_id: Option<String>,
    pub kinds: Option<Vec<ResourceKind>>,
}
