use serde::Serialize;
use std::path::PathBuf;

use super::resource::ResourceObservation;

#[derive(Debug, Clone)]
pub struct ConfigurationObservation {
    pub root_path: PathBuf,
    pub config_files: Vec<PathBuf>,
    pub exists: bool,
    pub readable: bool,
    pub valid: bool,
    pub detection_source: String,
    pub resources: Vec<ResourceObservation>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationSummary {
    pub root_path: String,
    pub config_files: Vec<String>,
    pub exists: bool,
    pub readable: bool,
    pub valid: bool,
    pub detection_source: String,
    pub resource_count: i64,
    pub manually_added: bool,
}
