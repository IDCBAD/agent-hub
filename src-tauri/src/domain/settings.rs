use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub launch_at_login: bool,
    pub keep_running_in_tray: bool,
    pub scan_on_launch: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            keep_running_in_tray: true,
            scan_on_launch: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub schema_version: i64,
    pub data_directory: String,
}
