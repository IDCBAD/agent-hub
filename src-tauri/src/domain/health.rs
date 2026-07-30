use serde::{Deserialize, Serialize};

use super::{configuration::ConfigurationObservation, runtime::RuntimeObservation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    RuntimeOnly,
    ConfigOnly,
    Degraded,
    Changed,
    Missing,
    Disabled,
}

impl HealthStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::RuntimeOnly => "runtime_only",
            Self::ConfigOnly => "config_only",
            Self::Degraded => "degraded",
            Self::Changed => "changed",
            Self::Missing => "missing",
            Self::Disabled => "disabled",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value {
            "healthy" | "ready" => Self::Healthy,
            "runtime_only" => Self::RuntimeOnly,
            "config_only" => Self::ConfigOnly,
            "changed" => Self::Changed,
            "missing" => Self::Missing,
            "disabled" => Self::Disabled,
            _ => Self::Degraded,
        }
    }
}

pub fn evaluate_health(
    runtime: &RuntimeObservation,
    configuration: &ConfigurationObservation,
) -> HealthStatus {
    if configuration.exists && (!configuration.readable || !configuration.valid) {
        return HealthStatus::Degraded;
    }

    match (runtime.installed, configuration.exists) {
        (true, true) => HealthStatus::Healthy,
        (true, false) => HealthStatus::RuntimeOnly,
        (false, true) => HealthStatus::ConfigOnly,
        (false, false) => HealthStatus::Missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        configuration::ConfigurationObservation,
        runtime::{RuntimeObservation, VersionProbeStatus},
    };
    use std::path::PathBuf;

    fn configuration(exists: bool, readable: bool, valid: bool) -> ConfigurationObservation {
        ConfigurationObservation {
            root_path: PathBuf::from(".agent"),
            config_files: Vec::new(),
            exists,
            readable,
            valid,
            detection_source: "test".to_owned(),
            resources: Vec::new(),
        }
    }

    #[test]
    fn derives_health_from_runtime_and_configuration() {
        let installed = RuntimeObservation {
            command_name: "agent".to_owned(),
            executable_path: Some(PathBuf::from("agent")),
            installed: true,
            version: None,
            version_probe_status: VersionProbeStatus::Failed,
            version_probe_error: Some("test failure".to_owned()),
            resolution_source: crate::domain::runtime::RuntimeResolutionSource::Path,
            distribution: crate::domain::runtime::RuntimeDistribution::Native,
        };
        let missing_runtime = RuntimeObservation {
            command_name: "agent".to_owned(),
            executable_path: None,
            installed: false,
            version: None,
            version_probe_status: VersionProbeStatus::NotAttempted,
            version_probe_error: None,
            resolution_source: crate::domain::runtime::RuntimeResolutionSource::NotFound,
            distribution: crate::domain::runtime::RuntimeDistribution::Unknown,
        };

        assert_eq!(
            evaluate_health(&installed, &configuration(true, true, true)),
            HealthStatus::Healthy
        );
        assert_eq!(
            evaluate_health(&installed, &configuration(false, false, false)),
            HealthStatus::RuntimeOnly
        );
        assert_eq!(
            evaluate_health(&missing_runtime, &configuration(true, true, true)),
            HealthStatus::ConfigOnly
        );
        assert_eq!(
            evaluate_health(&missing_runtime, &configuration(true, false, false)),
            HealthStatus::Degraded
        );
        assert_eq!(
            evaluate_health(&missing_runtime, &configuration(false, false, false)),
            HealthStatus::Missing
        );
    }
}
