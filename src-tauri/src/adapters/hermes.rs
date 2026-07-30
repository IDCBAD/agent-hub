use crate::domain::resource::ResourceKind;

use super::common::{ConfiguredAdapter, ResourceRule};

pub fn adapter() -> ConfiguredAdapter {
    ConfiguredAdapter::new(
        "hermes",
        "Hermes Agent",
        "hermes",
        ".hermes",
        &["HERMES_HOME", "HERMES_CONFIG_DIR"],
        &["hermes", "hermes-agent"],
        &[
            ".hermes/bin",
            "AppData/Local/hermes",
            "AppData/Local/hermes/bin",
        ],
        &["config.yaml", "config.yml", "skills", "memory"],
        vec![
            ResourceRule::file("config-yaml", "config.yaml", ResourceKind::Config, false),
            ResourceRule::file("config-yml", "config.yml", ResourceKind::Config, false),
            ResourceRule::file("settings", "settings.json", ResourceKind::Config, false),
            ResourceRule::file(
                "system-prompt",
                "system_prompt.md",
                ResourceKind::Prompt,
                false,
            ),
            ResourceRule::file("auth", "auth.json", ResourceKind::Config, true),
            ResourceRule::directory("skills", "skills", ResourceKind::Skill),
            ResourceRule::directory("mcp", "mcp", ResourceKind::Mcp),
            ResourceRule::directory("memory", "memory", ResourceKind::Memory),
        ],
    )
}
