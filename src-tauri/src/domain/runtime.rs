use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeResolutionSource {
    Path,
    DefaultPath,
    NotFound,
}

impl RuntimeResolutionSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::DefaultPath => "default_path",
            Self::NotFound => "not_found",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "path" => Self::Path,
            "default_path" => Self::DefaultPath,
            _ => Self::NotFound,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeDistribution {
    Npm,
    Python,
    Bundled,
    Native,
    Unknown,
}

impl RuntimeDistribution {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Python => "python",
            Self::Bundled => "bundled",
            Self::Native => "native",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "npm" => Self::Npm,
            "python" => Self::Python,
            "bundled" => Self::Bundled,
            "native" => Self::Native,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionProbeStatus {
    NotAttempted,
    Detected,
    Failed,
    TimedOut,
    Unsupported,
}

impl VersionProbeStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotAttempted => "not_attempted",
            Self::Detected => "detected",
            Self::Failed => "failed",
            Self::TimedOut => "timed_out",
            Self::Unsupported => "unsupported",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "detected" => Self::Detected,
            "failed" => Self::Failed,
            "timed_out" => Self::TimedOut,
            "unsupported" => Self::Unsupported,
            _ => Self::NotAttempted,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeObservation {
    pub command_name: String,
    pub executable_path: Option<PathBuf>,
    pub installed: bool,
    pub version: Option<String>,
    pub version_probe_status: VersionProbeStatus,
    pub version_probe_error: Option<String>,
    pub resolution_source: RuntimeResolutionSource,
    pub distribution: RuntimeDistribution,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSummary {
    pub command_name: String,
    pub executable_path: Option<String>,
    pub installed: bool,
    pub version: Option<String>,
    pub version_probe_status: VersionProbeStatus,
    pub version_probe_error: Option<String>,
    pub resolution_source: RuntimeResolutionSource,
    pub distribution: RuntimeDistribution,
}
