use crate::domain::resource::ResourceKind;

use super::common::{ConfiguredAdapter, ResourceRule};

pub fn adapter() -> ConfiguredAdapter {
    ConfiguredAdapter::new(
        "claude-code",
        "Claude Code",
        "claude",
        ".claude",
        &["CLAUDE_CONFIG_DIR"],
        &["claude", "claude-code"],
        &[".claude/bin"],
        &["settings.json", "CLAUDE.md", "skills", "commands"],
        vec![
            ResourceRule::file("settings", "settings.json", ResourceKind::Config, false),
            ResourceRule::file(
                "settings-local",
                "settings.local.json",
                ResourceKind::Config,
                false,
            ),
            ResourceRule::file("identity", "CLAUDE.md", ResourceKind::Identity, false),
            ResourceRule::file("mcp", "mcp.json", ResourceKind::Mcp, false),
            ResourceRule::directory("agents", "agents", ResourceKind::Prompt),
            ResourceRule::directory("commands", "commands", ResourceKind::Prompt),
            ResourceRule::directory("skills", "skills", ResourceKind::Skill),
            ResourceRule::directory("plugins", "plugins", ResourceKind::Other),
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
        fs::write(npm.join("claude"), "#!/bin/sh\necho wrong\n").expect("unix shim");
        fs::write(npm.join("claude.cmd"), "@echo 2.1.220 (Claude Code)\r\n").expect("windows shim");
        let context =
            DiscoveryContext::for_test(fixture.path().to_path_buf(), vec![npm], HashMap::new());

        let runtime = adapter().detect_runtime(&context).expect("runtime");

        assert!(runtime
            .executable_path
            .as_deref()
            .is_some_and(|path| path.ends_with("claude.cmd")));
        assert_eq!(runtime.version.as_deref(), Some("2.1.220 (Claude Code)"));
        assert_eq!(runtime.version_probe_status, VersionProbeStatus::Detected);
    }
}
