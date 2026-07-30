use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{
    adapters::AdapterRegistry,
    domain::{
        agent::{AgentFilter, AgentOverview, AgentSummary, ManualLocationRequest},
        discovery::{DiscoveryEvidence, DiscoveryResult},
        quick_location::{
            CreateQuickLocationRequest, QuickLocation, ReorderQuickLocationsRequest,
            UpdateQuickLocationRequest,
        },
        resource::{Resource, ResourceFilter},
    },
    error::AppError,
    infrastructure::{
        database::Database,
        platform::{
            canonical_directory, open_agent_directory, open_quick_directory, open_resource,
        },
    },
};

#[derive(Clone)]
pub struct ApplicationService {
    database: Database,
    registry: AdapterRegistry,
}

impl ApplicationService {
    pub fn new(database: Database, registry: AdapterRegistry) -> Result<Self, AppError> {
        database.upsert_agent_types(&registry.descriptors())?;
        Ok(Self { database, registry })
    }

    pub fn discover_agents(&self) -> Result<DiscoveryResult, AppError> {
        let mut drafts = self.registry.discover_all()?;
        let manual_locations = self.database.list_manual_locations()?;
        let manual_keys = manual_locations
            .iter()
            .map(|location| manual_location_key(&location.agent_type_id, Path::new(&location.path)))
            .collect::<HashSet<_>>();
        drafts.retain(|draft| {
            !manual_keys.contains(&manual_location_key(
                &draft.agent_type_id,
                &draft.configuration.root_path,
            ))
        });
        for location in manual_locations {
            drafts.push(self.registry.detect_path(
                &location.agent_type_id,
                Path::new(&location.path),
                "manual",
                true,
            )?);
        }
        Ok(self.database.reconcile(drafts, true)?.result)
    }

    pub fn list_agents(&self, filter: Option<AgentFilter>) -> Result<Vec<AgentSummary>, AppError> {
        self.database.list_agents(filter)
    }

    pub fn get_agent_overview(&self, agent_id: &str) -> Result<AgentOverview, AppError> {
        self.database.get_agent_overview(agent_id)
    }

    pub fn get_agent_resources(&self, agent_id: &str) -> Result<Vec<Resource>, AppError> {
        self.database.get_resources(agent_id)
    }

    pub fn list_resources(
        &self,
        filter: Option<ResourceFilter>,
    ) -> Result<Vec<Resource>, AppError> {
        self.database.list_resources(filter)
    }

    pub fn get_discovery_evidence(
        &self,
        agent_id: &str,
    ) -> Result<Vec<DiscoveryEvidence>, AppError> {
        self.database.get_evidence(agent_id)
    }

    pub fn add_manual_location(
        &self,
        request: ManualLocationRequest,
    ) -> Result<AgentSummary, AppError> {
        if request.path.trim().is_empty() {
            return Err(AppError::invalid_path("配置目录不能为空。"));
        }
        let draft = self.registry.detect_path(
            &request.agent_type_id,
            Path::new(request.path.trim()),
            "manual",
            false,
        )?;
        let outcome = self.database.reconcile_manual(draft)?;
        let agent_id = outcome
            .agent_ids
            .first()
            .ok_or_else(|| AppError::internal("手动添加未返回 Agent 记录。"))?;
        self.database.get_agent_by_id(agent_id)
    }

    pub fn rescan_agent(&self, agent_id: &str) -> Result<DiscoveryResult, AppError> {
        let existing = self.database.get_agent_overview(agent_id)?;
        let draft = self.registry.detect_path(
            &existing.summary.agent_type_id,
            &PathBuf::from(&existing.summary.configuration.root_path),
            &existing.summary.configuration.detection_source,
            true,
        )?;
        Ok(self.database.reconcile(vec![draft], false)?.result)
    }

    pub fn open_agent_directory(&self, agent_id: &str) -> Result<(), AppError> {
        let path = self.database.get_agent_path(agent_id)?;
        open_agent_directory(&path)
    }

    pub fn open_resource(&self, resource_id: &str) -> Result<(), AppError> {
        let (path, root) = self.database.get_resource_path_and_root(resource_id)?;
        open_resource(&path, &root)
    }

    pub fn remove_manual_agent(&self, agent_id: &str) -> Result<(), AppError> {
        self.database.remove_manual_agent(agent_id)
    }

    pub fn list_quick_locations(&self) -> Result<Vec<QuickLocation>, AppError> {
        self.database.list_quick_locations()
    }

    pub fn create_quick_location(
        &self,
        request: CreateQuickLocationRequest,
    ) -> Result<QuickLocation, AppError> {
        let name = validate_quick_location_name(&request.name)?;
        let path = request.path.trim();
        if path.is_empty() {
            return Err(AppError::invalid_path("快捷目录不能为空。"));
        }
        let canonical = canonical_directory(Path::new(path))?;
        self.database
            .create_quick_location(name, &canonical, request.show_in_tray)
    }

    pub fn update_quick_location(
        &self,
        request: UpdateQuickLocationRequest,
    ) -> Result<QuickLocation, AppError> {
        let name = validate_quick_location_name(&request.name)?;
        self.database
            .update_quick_location(&request.id, name, request.show_in_tray)
    }

    pub fn reorder_quick_locations(
        &self,
        request: ReorderQuickLocationsRequest,
    ) -> Result<(), AppError> {
        let existing = self.database.list_quick_locations()?;
        let existing_ids = existing
            .iter()
            .map(|location| location.id.as_str())
            .collect::<HashSet<_>>();
        let requested_ids = request
            .ids
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        if request.ids.len() != existing.len() || requested_ids != existing_ids {
            return Err(AppError::new(
                "invalid_location_order",
                "快捷目录顺序与当前列表不一致。",
                true,
                Some("请刷新快捷目录列表后重试。"),
            ));
        }
        self.database.reorder_quick_locations(&request.ids)
    }

    pub fn remove_quick_location(&self, id: &str) -> Result<(), AppError> {
        self.database.remove_quick_location(id)
    }

    pub fn open_quick_location(&self, id: &str) -> Result<(), AppError> {
        let path = self.database.get_quick_location_path(id)?;
        open_quick_directory(&path)?;
        self.database.mark_quick_location_opened(id)
    }
}

fn validate_quick_location_name(value: &str) -> Result<&str, AppError> {
    let name = value.trim();
    if name.is_empty() {
        return Err(AppError::new(
            "invalid_name",
            "快捷目录名称不能为空。",
            true,
            Some("请输入一个便于识别的名称。"),
        ));
    }
    if name.chars().count() > 60 {
        return Err(AppError::new(
            "invalid_name",
            "快捷目录名称不能超过 60 个字符。",
            true,
            Some("请使用更短的显示名称。"),
        ));
    }
    Ok(name)
}

fn manual_location_key(agent_type_id: &str, path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    let path = if cfg!(windows) {
        path.to_ascii_lowercase()
    } else {
        path
    };
    format!("{agent_type_id}:{path}")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn service() -> (tempfile::TempDir, ApplicationService) {
        let directory = tempfile::tempdir().expect("application fixture");
        let database =
            Database::initialize(directory.path().join("agent-hub.db")).expect("database");
        let service =
            ApplicationService::new(database, AdapterRegistry::standard()).expect("service");
        (directory, service)
    }

    #[test]
    fn manual_location_flows_through_adapter_database_and_queries() {
        let (directory, service) = service();
        let config_root = directory.path().join(".codex");
        fs::create_dir_all(config_root.join("skills/example")).expect("skill directory");
        fs::write(config_root.join("config.toml"), "model = \"local\"").expect("config");
        fs::write(config_root.join("skills/example/SKILL.md"), "# Local skill").expect("skill");

        let agent = service
            .add_manual_location(ManualLocationRequest {
                agent_type_id: "codex".to_owned(),
                path: config_root.to_string_lossy().into_owned(),
            })
            .expect("manual add");
        let overview = service
            .get_agent_overview(&agent.id)
            .expect("agent overview");
        let resources = service
            .get_agent_resources(&agent.id)
            .expect("agent resources");
        let evidence = service
            .get_discovery_evidence(&agent.id)
            .expect("discovery evidence");

        assert_eq!(overview.summary.configuration.detection_source, "manual");
        assert!(overview.summary.configuration.manually_added);
        assert_eq!(resources.len(), 2);
        assert_eq!(
            resources
                .iter()
                .find(|resource| resource.logical_key == "skills")
                .and_then(|resource| resource.entry_count),
            Some(2)
        );
        assert!(evidence
            .iter()
            .any(|item| item.evidence_type == "config_root" && item.success));

        assert_eq!(
            service
                .database
                .list_manual_locations()
                .expect("manual registrations")
                .len(),
            1
        );
    }

    #[test]
    fn manual_location_rejects_missing_directory() {
        let (directory, service) = service();
        let error = service
            .add_manual_location(ManualLocationRequest {
                agent_type_id: "claude-code".to_owned(),
                path: directory
                    .path()
                    .join("missing")
                    .to_string_lossy()
                    .into_owned(),
            })
            .expect_err("missing directory must fail");

        assert_eq!(error.code, "invalid_path");
        assert!(error.recoverable);
    }

    #[test]
    fn removing_manual_agent_only_deletes_hub_data() {
        let (directory, service) = service();
        let config_root = directory.path().join(".codex");
        fs::create_dir_all(&config_root).expect("config directory");
        fs::write(config_root.join("config.toml"), "model = \"local\"").expect("config");
        let agent = service
            .add_manual_location(ManualLocationRequest {
                agent_type_id: "codex".to_owned(),
                path: config_root.to_string_lossy().into_owned(),
            })
            .expect("manual add");

        service
            .remove_manual_agent(&agent.id)
            .expect("remove manual agent");

        assert!(config_root.exists());
        assert!(config_root.join("config.toml").exists());
        assert!(service.list_agents(None).expect("agents").is_empty());
    }

    #[test]
    fn quick_location_flow_keeps_the_original_directory() {
        let (directory, service) = service();
        let quick_root = directory.path().join("daily-prompts");
        fs::create_dir_all(&quick_root).expect("quick directory");

        let location = service
            .create_quick_location(CreateQuickLocationRequest {
                name: " Daily Prompts ".to_owned(),
                path: quick_root.to_string_lossy().into_owned(),
                show_in_tray: true,
            })
            .expect("create quick location");
        assert_eq!(location.name, "Daily Prompts");
        assert!(location.show_in_tray);

        let updated = service
            .update_quick_location(UpdateQuickLocationRequest {
                id: location.id.clone(),
                name: "Prompts".to_owned(),
                show_in_tray: false,
            })
            .expect("update quick location");
        assert_eq!(updated.name, "Prompts");
        assert!(!updated.show_in_tray);

        service
            .remove_quick_location(&location.id)
            .expect("remove quick location");
        assert!(quick_root.exists());
        assert!(service
            .list_quick_locations()
            .expect("list quick locations")
            .is_empty());
    }

    #[test]
    fn quick_location_rejects_missing_directory_and_empty_name() {
        let (directory, service) = service();
        let error = service
            .create_quick_location(CreateQuickLocationRequest {
                name: "Missing".to_owned(),
                path: directory
                    .path()
                    .join("missing")
                    .to_string_lossy()
                    .into_owned(),
                show_in_tray: true,
            })
            .expect_err("missing directory");
        assert_eq!(error.code, "invalid_path");

        let error = service
            .create_quick_location(CreateQuickLocationRequest {
                name: " ".to_owned(),
                path: directory.path().to_string_lossy().into_owned(),
                show_in_tray: true,
            })
            .expect_err("empty name");
        assert_eq!(error.code, "invalid_name");
    }
}
