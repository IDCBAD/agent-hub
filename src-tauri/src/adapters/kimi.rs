use crate::domain::resource::ResourceKind;

use super::common::{ConfiguredAdapter, ResourceRule};

pub fn adapter() -> ConfiguredAdapter {
    ConfiguredAdapter::new(
        "kimi-cli",
        "Kimi Code",
        "kimi",
        ".kimi-code",
        &["KIMI_HOME", "KIMI_CONFIG_DIR"],
        &["kimi"],
        &[".kimi-code/bin"],
        &["config.toml", "tui.toml", "bin/kimi.exe", "bin/kimi"],
        vec![
            ResourceRule::file("config", "config.toml", ResourceKind::Config, true),
            ResourceRule::file("tui", "tui.toml", ResourceKind::Config, false),
            ResourceRule::file("workspaces", "workspaces.json", ResourceKind::Other, false),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs};

    use super::*;
    use crate::{
        adapters::{common::DiscoveryContext, AgentAdapter},
        domain::runtime::{RuntimeDistribution, RuntimeResolutionSource},
    };

    #[test]
    fn discovers_kimi_cli_runtime_version_and_configuration() {
        let fixture = tempfile::tempdir().expect("fixture");
        let root = fixture.path().join(".kimi-code");
        let bin = root.join("bin");
        fs::create_dir_all(&bin).expect("kimi bin");
        fs::write(root.join("config.toml"), "default_model = \"kimi\"").expect("config");
        fs::write(root.join("tui.toml"), "theme = \"dark\"").expect("tui config");

        #[cfg(windows)]
        fs::write(bin.join("kimi.cmd"), "@echo 0.28.1\r\n").expect("kimi cli");
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let executable = bin.join("kimi");
            fs::write(&executable, "#!/bin/sh\necho 0.28.1\n").expect("kimi cli");
            let mut permissions = fs::metadata(&executable).expect("metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(executable, permissions).expect("permissions");
        }

        let context =
            DiscoveryContext::for_test(fixture.path().to_path_buf(), Vec::new(), HashMap::new());
        let drafts = adapter().discover(&context).expect("kimi discovery");

        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].runtime.command_name, "kimi");
        assert_eq!(drafts[0].runtime.version.as_deref(), Some("0.28.1"));
        assert_eq!(
            drafts[0].runtime.resolution_source,
            RuntimeResolutionSource::DefaultPath
        );
        assert_eq!(drafts[0].runtime.distribution, RuntimeDistribution::Bundled);
        assert_eq!(drafts[0].configuration.config_files.len(), 2);
    }
}
