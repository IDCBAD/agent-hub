use crate::domain::resource::ResourceKind;

use super::common::{ConfiguredAdapter, ResourceRule};

pub fn adapter() -> ConfiguredAdapter {
    ConfiguredAdapter::new(
        "codex",
        "Codex",
        "codex",
        ".codex",
        &["CODEX_HOME"],
        &["codex"],
        &[".codex/bin"],
        &["config.toml", "AGENTS.md", "skills", "rules"],
        vec![
            ResourceRule::file("config", "config.toml", ResourceKind::Config, false),
            ResourceRule::file("auth", "auth.json", ResourceKind::Config, true),
            ResourceRule::file("identity", "AGENTS.md", ResourceKind::Identity, false),
            ResourceRule::directory("prompts", "prompts", ResourceKind::Prompt),
            ResourceRule::directory("skills", "skills", ResourceKind::Skill),
            ResourceRule::directory("rules", "rules", ResourceKind::Config),
        ],
    )
}

#[cfg(all(test, windows))]
mod tests {
    use std::{collections::HashMap, fs};

    use super::*;
    use crate::{
        adapters::{common::DiscoveryContext, AgentAdapter},
        domain::runtime::VersionProbeStatus,
    };

    #[test]
    fn detects_version_from_windows_npm_cmd_instead_of_extensionless_shim() {
        let fixture = tempfile::tempdir().expect("fixture");
        let npm = fixture.path().join("npm");
        fs::create_dir_all(&npm).expect("npm");
        fs::write(npm.join("codex"), "#!/bin/sh\necho wrong\n").expect("unix shim");
        fs::write(npm.join("codex.cmd"), "@echo codex-cli 0.145.0\r\n").expect("windows shim");
        let context =
            DiscoveryContext::for_test(fixture.path().to_path_buf(), vec![npm], HashMap::new());

        let runtime = adapter().detect_runtime(&context).expect("runtime");

        assert!(runtime
            .executable_path
            .as_deref()
            .is_some_and(|path| path.ends_with("codex.cmd")));
        assert_eq!(runtime.version.as_deref(), Some("codex-cli 0.145.0"));
        assert_eq!(runtime.version_probe_status, VersionProbeStatus::Detected);
    }
}
