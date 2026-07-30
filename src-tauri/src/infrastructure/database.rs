use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
use uuid::Uuid;

use crate::{
    domain::{
        agent::{
            AgentDraft, AgentFilter, AgentOverview, AgentSummary, AgentTypeDescriptor, Confidence,
            ManualLocationRequest,
        },
        configuration::ConfigurationSummary,
        discovery::{DiscoveryEvidence, DiscoveryResult},
        health::HealthStatus,
        quick_location::QuickLocation,
        resource::{Resource, ResourceFilter, ResourceKind},
        runtime::{
            RuntimeDistribution, RuntimeResolutionSource, RuntimeSummary, VersionProbeStatus,
        },
    },
    error::AppError,
};

const SCHEMA_VERSION: i64 = 6;

#[derive(Debug, Clone)]
pub struct Database {
    path: PathBuf,
}

#[derive(Debug)]
pub struct ReconcileOutcome {
    pub result: DiscoveryResult,
    pub agent_ids: Vec<String>,
}

impl Database {
    pub fn initialize(path: PathBuf) -> Result<Self, AppError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let database = Self { path };
        let mut connection = database.open()?;
        database.migrate(&mut connection)?;
        Ok(database)
    }

    fn open(&self) -> Result<Connection, AppError> {
        let connection = Connection::open(&self.path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.busy_timeout(Duration::from_secs(3))?;
        Ok(connection)
    }

    fn migrate(&self, connection: &mut Connection) -> Result<(), AppError> {
        let current: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if current >= SCHEMA_VERSION {
            return Ok(());
        }

        let transaction = connection.transaction()?;
        transaction.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS agent_types (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                adapter_version INTEGER NOT NULL,
                icon_key TEXT NOT NULL,
                capabilities_json TEXT NOT NULL DEFAULT '{}',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS agent_instances (
                id TEXT PRIMARY KEY,
                agent_type_id TEXT NOT NULL REFERENCES agent_types(id),
                display_name TEXT NOT NULL,
                executable_path TEXT,
                config_root TEXT NOT NULL,
                normalized_config_root TEXT NOT NULL,
                detected_version TEXT,
                discovery_source TEXT NOT NULL,
                status TEXT NOT NULL,
                confidence TEXT NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                last_seen_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(agent_type_id, normalized_config_root)
            );

            CREATE TABLE IF NOT EXISTS resources (
                id TEXT PRIMARY KEY,
                agent_instance_id TEXT NOT NULL REFERENCES agent_instances(id) ON DELETE CASCADE,
                kind TEXT NOT NULL,
                logical_key TEXT NOT NULL,
                path TEXT NOT NULL,
                normalized_path TEXT NOT NULL,
                format TEXT NOT NULL,
                scope TEXT NOT NULL,
                is_sensitive INTEGER NOT NULL,
                exists_flag INTEGER NOT NULL,
                writable_flag INTEGER NOT NULL,
                content_hash TEXT,
                modified_at INTEGER,
                size_bytes INTEGER,
                structure_json TEXT,
                last_observed_at INTEGER NOT NULL,
                UNIQUE(agent_instance_id, logical_key, normalized_path)
            );

            CREATE TABLE IF NOT EXISTS discovery_runs (
                id TEXT PRIMARY KEY,
                started_at INTEGER NOT NULL,
                finished_at INTEGER,
                status TEXT NOT NULL,
                discovered_count INTEGER NOT NULL DEFAULT 0,
                changed_count INTEGER NOT NULL DEFAULT 0,
                missing_count INTEGER NOT NULL DEFAULT 0,
                error_summary TEXT
            );

            CREATE TABLE IF NOT EXISTS discovery_evidence (
                id TEXT PRIMARY KEY,
                agent_instance_id TEXT NOT NULL REFERENCES agent_instances(id) ON DELETE CASCADE,
                evidence_type TEXT NOT NULL,
                source TEXT NOT NULL,
                observed_value TEXT NOT NULL,
                success INTEGER NOT NULL,
                message TEXT NOT NULL,
                observed_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                normalized_path TEXT NOT NULL UNIQUE,
                status TEXT NOT NULL,
                git_metadata_json TEXT NOT NULL DEFAULT '{}',
                last_used_at INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS agent_workspace_links (
                agent_instance_id TEXT NOT NULL REFERENCES agent_instances(id) ON DELETE CASCADE,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY(agent_instance_id, workspace_id)
            );

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value_json TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_resources_agent ON resources(agent_instance_id);
            CREATE INDEX IF NOT EXISTS idx_resources_kind ON resources(kind);
            CREATE INDEX IF NOT EXISTS idx_evidence_agent ON discovery_evidence(agent_instance_id);
            "#,
        )?;
        if current < 2 {
            transaction.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS agent_runtimes (
                    id TEXT PRIMARY KEY,
                    agent_instance_id TEXT NOT NULL UNIQUE
                        REFERENCES agent_instances(id) ON DELETE CASCADE,
                    executable_path TEXT,
                    installed_flag INTEGER NOT NULL,
                    version TEXT,
                    detection_source TEXT NOT NULL,
                    detected_at INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS agent_configurations (
                    id TEXT PRIMARY KEY,
                    agent_instance_id TEXT NOT NULL UNIQUE
                        REFERENCES agent_instances(id) ON DELETE CASCADE,
                    agent_type_id TEXT NOT NULL REFERENCES agent_types(id),
                    root_path TEXT NOT NULL,
                    normalized_root_path TEXT NOT NULL,
                    exists_flag INTEGER NOT NULL,
                    readable_flag INTEGER NOT NULL,
                    valid_flag INTEGER NOT NULL,
                    detection_source TEXT NOT NULL,
                    detected_at INTEGER NOT NULL,
                    UNIQUE(agent_type_id, normalized_root_path)
                );

                ALTER TABLE resources
                    ADD COLUMN agent_configuration_id TEXT
                    REFERENCES agent_configurations(id) ON DELETE CASCADE;

                INSERT OR IGNORE INTO agent_runtimes (
                    id, agent_instance_id, executable_path, installed_flag,
                    version, detection_source, detected_at
                )
                SELECT
                    'runtime:' || id, id, executable_path,
                    CASE WHEN executable_path IS NULL THEN 0 ELSE 1 END,
                    detected_version,
                    CASE WHEN executable_path IS NULL THEN 'not_found' ELSE 'path' END,
                    last_seen_at
                FROM agent_instances;

                INSERT OR IGNORE INTO agent_configurations (
                    id, agent_instance_id, agent_type_id, root_path,
                    normalized_root_path, exists_flag, readable_flag, valid_flag,
                    detection_source, detected_at
                )
                SELECT
                    'configuration:' || id, id, agent_type_id, config_root,
                    normalized_config_root,
                    CASE WHEN status = 'missing' THEN 0 ELSE 1 END,
                    CASE WHEN status IN ('missing', 'invalid') THEN 0 ELSE 1 END,
                    CASE WHEN status = 'invalid' THEN 0 ELSE 1 END,
                    discovery_source, last_seen_at
                FROM agent_instances;

                UPDATE resources
                SET agent_configuration_id = 'configuration:' || agent_instance_id
                WHERE agent_configuration_id IS NULL;

                CREATE INDEX IF NOT EXISTS idx_runtime_agent
                    ON agent_runtimes(agent_instance_id);
                CREATE INDEX IF NOT EXISTS idx_configuration_agent
                    ON agent_configurations(agent_instance_id);
                CREATE INDEX IF NOT EXISTS idx_resources_configuration
                    ON resources(agent_configuration_id);
                "#,
            )?;
        }
        if current < 3 {
            transaction.execute_batch(
                r#"
                ALTER TABLE agent_runtimes
                    ADD COLUMN command_name TEXT NOT NULL DEFAULT '';
                ALTER TABLE agent_runtimes
                    ADD COLUMN resolution_source TEXT NOT NULL DEFAULT 'not_found';
                ALTER TABLE agent_runtimes
                    ADD COLUMN distribution TEXT NOT NULL DEFAULT 'unknown';

                UPDATE agent_runtimes
                SET command_name = CASE
                        WHEN agent_instance_id IN (
                            SELECT id FROM agent_instances WHERE agent_type_id = 'claude-code'
                        ) THEN 'claude'
                        WHEN agent_instance_id IN (
                            SELECT id FROM agent_instances WHERE agent_type_id = 'codex'
                        ) THEN 'codex'
                        WHEN agent_instance_id IN (
                            SELECT id FROM agent_instances WHERE agent_type_id = 'hermes'
                        ) THEN 'hermes'
                        WHEN agent_instance_id IN (
                            SELECT id FROM agent_instances WHERE agent_type_id = 'kimi-cli'
                        ) THEN 'kimi'
                        ELSE ''
                    END,
                    resolution_source = CASE
                        WHEN detection_source IN ('path', 'npm') THEN 'path'
                        WHEN detection_source = 'default_path' THEN 'default_path'
                        ELSE 'not_found'
                    END,
                    distribution = CASE
                        WHEN detection_source = 'npm' THEN 'npm'
                        ELSE 'unknown'
                    END;

                CREATE TABLE IF NOT EXISTS manual_agent_locations (
                    id TEXT PRIMARY KEY,
                    agent_type_id TEXT NOT NULL REFERENCES agent_types(id),
                    path TEXT NOT NULL,
                    normalized_path TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    UNIQUE(agent_type_id, normalized_path)
                );

                INSERT OR IGNORE INTO manual_agent_locations (
                    id, agent_type_id, path, normalized_path, created_at
                )
                SELECT
                    'manual:' || ai.id,
                    ai.agent_type_id,
                    ac.root_path,
                    ac.normalized_root_path,
                    ai.created_at
                FROM agent_instances ai
                JOIN agent_configurations ac ON ac.agent_instance_id = ai.id
                WHERE ai.discovery_source = 'manual'
                   OR ac.detection_source = 'manual';

                CREATE INDEX IF NOT EXISTS idx_manual_location_type_path
                    ON manual_agent_locations(agent_type_id, normalized_path);
                "#,
            )?;
        }
        if current < 4 {
            transaction.execute_batch(
                r#"
                ALTER TABLE agent_runtimes
                    ADD COLUMN version_probe_status TEXT NOT NULL DEFAULT 'not_attempted';
                ALTER TABLE agent_runtimes
                    ADD COLUMN version_probe_error TEXT;

                UPDATE agent_runtimes
                SET version_probe_status = CASE
                    WHEN version IS NOT NULL THEN 'detected'
                    WHEN installed_flag = 1 THEN 'failed'
                    ELSE 'not_attempted'
                END;
                "#,
            )?;
        }
        if current < 5 {
            transaction.execute_batch(
                r#"
                UPDATE agent_instances
                SET display_name = CASE agent_type_id
                    WHEN 'claude-code' THEN 'Claude Code'
                    WHEN 'codex' THEN 'Codex'
                    WHEN 'hermes' THEN 'Hermes Agent'
                    WHEN 'kimi-cli' THEN 'Kimi Code'
                    ELSE display_name
                END;
                "#,
            )?;
        }
        if current < 6 {
            transaction.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS quick_locations (
                    id TEXT PRIMARY KEY,
                    display_name TEXT NOT NULL,
                    path TEXT NOT NULL,
                    normalized_path TEXT NOT NULL UNIQUE,
                    show_in_tray INTEGER NOT NULL DEFAULT 1,
                    sort_order INTEGER NOT NULL,
                    last_opened_at INTEGER,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_quick_locations_order
                    ON quick_locations(sort_order, created_at);
                "#,
            )?;
        }
        transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn upsert_agent_types(&self, descriptors: &[AgentTypeDescriptor]) -> Result<(), AppError> {
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let now = now_seconds();
        for descriptor in descriptors {
            transaction.execute(
                r#"
                INSERT INTO agent_types (
                    id, display_name, adapter_version, icon_key,
                    capabilities_json, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, '{}', ?5, ?5)
                ON CONFLICT(id) DO UPDATE SET
                    display_name = excluded.display_name,
                    adapter_version = excluded.adapter_version,
                    icon_key = excluded.icon_key,
                    updated_at = excluded.updated_at
                "#,
                params![
                    descriptor.id,
                    descriptor.display_name,
                    descriptor.adapter_version,
                    descriptor.icon_key,
                    now
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn reconcile(
        &self,
        drafts: Vec<AgentDraft>,
        mark_unseen_missing: bool,
    ) -> Result<ReconcileOutcome, AppError> {
        self.reconcile_internal(drafts, mark_unseen_missing, false)
    }

    pub fn reconcile_manual(&self, draft: AgentDraft) -> Result<ReconcileOutcome, AppError> {
        self.reconcile_internal(vec![draft], false, true)
    }

    fn reconcile_internal(
        &self,
        drafts: Vec<AgentDraft>,
        mark_unseen_missing: bool,
        register_manual: bool,
    ) -> Result<ReconcileOutcome, AppError> {
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let run_id = Uuid::new_v4().to_string();
        let started_at = now_seconds();
        transaction.execute(
            "INSERT INTO discovery_runs (id, started_at, status) VALUES (?1, ?2, 'running')",
            params![run_id, started_at],
        )?;

        let mut agent_ids = Vec::new();
        let mut seen_ids = HashSet::new();
        let mut changed_count = 0_usize;

        for mut draft in drafts {
            let normalized_root = normalized_path_key(&draft.configuration.root_path);
            let existing: Option<(String, String)> = transaction
                .query_row(
                    r#"
                    SELECT ai.id, ac.id
                    FROM agent_instances ai
                    JOIN agent_configurations ac ON ac.agent_instance_id = ai.id
                    WHERE ac.agent_type_id = ?1 AND ac.normalized_root_path = ?2
                    "#,
                    params![draft.agent_type_id, normalized_root],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            let agent_id = existing
                .as_ref()
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let configuration_id = existing
                .as_ref()
                .map(|(_, id)| id.clone())
                .unwrap_or_else(|| format!("configuration:{agent_id}"));
            let display_name = draft.display_name.clone();

            if existing.is_some()
                && resources_changed(&transaction, &agent_id, &draft)?
                && draft.health != HealthStatus::Missing
            {
                draft.health = HealthStatus::Changed;
                changed_count += 1;
            }

            transaction.execute(
                r#"
                INSERT INTO agent_instances (
                    id, agent_type_id, display_name, executable_path, config_root,
                    normalized_config_root, detected_version, discovery_source,
                    status, confidence, metadata_json, last_seen_at, created_at, updated_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12, ?12
                )
                ON CONFLICT(agent_type_id, normalized_config_root) DO UPDATE SET
                    display_name = excluded.display_name,
                    executable_path = excluded.executable_path,
                    config_root = excluded.config_root,
                    detected_version = excluded.detected_version,
                    discovery_source = excluded.discovery_source,
                    status = excluded.status,
                    confidence = excluded.confidence,
                    metadata_json = excluded.metadata_json,
                    last_seen_at = excluded.last_seen_at,
                    updated_at = excluded.updated_at
                "#,
                params![
                    agent_id,
                    draft.agent_type_id,
                    display_name,
                    draft
                        .runtime
                        .executable_path
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned()),
                    draft.configuration.root_path.to_string_lossy(),
                    normalized_root,
                    draft.runtime.version,
                    draft.configuration.detection_source,
                    draft.health.as_str(),
                    draft.confidence.as_str(),
                    draft.metadata.to_string(),
                    started_at,
                ],
            )?;

            transaction.execute(
                r#"
                INSERT INTO agent_runtimes (
                    id, agent_instance_id, executable_path, installed_flag,
                    version, detection_source, detected_at, command_name,
                    resolution_source, distribution, version_probe_status,
                    version_probe_error
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ON CONFLICT(agent_instance_id) DO UPDATE SET
                    executable_path = excluded.executable_path,
                    installed_flag = excluded.installed_flag,
                    version = excluded.version,
                    detection_source = excluded.detection_source,
                    detected_at = excluded.detected_at,
                    command_name = excluded.command_name,
                    resolution_source = excluded.resolution_source,
                    distribution = excluded.distribution,
                    version_probe_status = excluded.version_probe_status,
                    version_probe_error = excluded.version_probe_error
                "#,
                params![
                    format!("runtime:{agent_id}"),
                    agent_id,
                    draft
                        .runtime
                        .executable_path
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned()),
                    draft.runtime.installed as i64,
                    draft.runtime.version,
                    draft.runtime.resolution_source.as_str(),
                    started_at,
                    draft.runtime.command_name,
                    draft.runtime.resolution_source.as_str(),
                    draft.runtime.distribution.as_str(),
                    draft.runtime.version_probe_status.as_str(),
                    draft.runtime.version_probe_error,
                ],
            )?;
            transaction.execute(
                r#"
                INSERT INTO agent_configurations (
                    id, agent_instance_id, agent_type_id, root_path,
                    normalized_root_path, exists_flag, readable_flag, valid_flag,
                    detection_source, detected_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT(agent_instance_id) DO UPDATE SET
                    root_path = excluded.root_path,
                    normalized_root_path = excluded.normalized_root_path,
                    exists_flag = excluded.exists_flag,
                    readable_flag = excluded.readable_flag,
                    valid_flag = excluded.valid_flag,
                    detection_source = excluded.detection_source,
                    detected_at = excluded.detected_at
                "#,
                params![
                    configuration_id,
                    agent_id,
                    draft.agent_type_id,
                    draft.configuration.root_path.to_string_lossy(),
                    normalized_root,
                    draft.configuration.exists as i64,
                    draft.configuration.readable as i64,
                    draft.configuration.valid as i64,
                    draft.configuration.detection_source,
                    started_at,
                ],
            )?;

            if register_manual {
                transaction.execute(
                    r#"
                    INSERT INTO manual_agent_locations (
                        id, agent_type_id, path, normalized_path, created_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5)
                    ON CONFLICT(agent_type_id, normalized_path) DO UPDATE SET
                        path = excluded.path
                    "#,
                    params![
                        format!("manual:{agent_id}"),
                        draft.agent_type_id,
                        draft.configuration.root_path.to_string_lossy(),
                        normalized_root,
                        started_at,
                    ],
                )?;
            }

            replace_resources(
                &transaction,
                &agent_id,
                &configuration_id,
                &draft,
                started_at,
            )?;
            replace_evidence(&transaction, &agent_id, &draft, started_at)?;
            seen_ids.insert(agent_id.clone());
            agent_ids.push(agent_id);
        }

        let mut missing_count = 0_usize;
        if mark_unseen_missing {
            let mut statement = transaction.prepare(
                r#"
                SELECT ai.id
                FROM agent_instances ai
                JOIN agent_configurations ac ON ac.agent_instance_id = ai.id
                WHERE ai.status != 'disabled'
                  AND NOT EXISTS (
                      SELECT 1
                      FROM manual_agent_locations ml
                      WHERE ml.agent_type_id = ai.agent_type_id
                        AND ml.normalized_path = ac.normalized_root_path
                  )
                "#,
            )?;
            let known = statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            drop(statement);
            for id in known {
                if !seen_ids.contains(&id) {
                    transaction.execute(
                        "UPDATE agent_instances SET status = 'missing', updated_at = ?2 WHERE id = ?1",
                        params![id, started_at],
                    )?;
                    missing_count += 1;
                }
            }
        }

        let finished_at = now_seconds();
        transaction.execute(
            r#"
            UPDATE discovery_runs
            SET finished_at = ?2, status = 'completed',
                discovered_count = ?3, changed_count = ?4, missing_count = ?5
            WHERE id = ?1
            "#,
            params![
                run_id,
                finished_at,
                agent_ids.len() as i64,
                changed_count as i64,
                missing_count as i64
            ],
        )?;
        transaction.commit()?;

        Ok(ReconcileOutcome {
            result: DiscoveryResult {
                run_id,
                discovered_count: agent_ids.len(),
                changed_count,
                missing_count,
                finished_at,
            },
            agent_ids,
        })
    }

    pub fn list_agents(&self, filter: Option<AgentFilter>) -> Result<Vec<AgentSummary>, AppError> {
        let connection = self.open()?;
        let mut statement = connection.prepare(
            r#"
            SELECT
                ai.id, ai.agent_type_id, at.display_name, ai.display_name,
                ar.command_name, ar.executable_path, ar.installed_flag, ar.version,
                ar.version_probe_status, ar.version_probe_error,
                ar.resolution_source, ar.distribution,
                ac.root_path, ac.exists_flag, ac.readable_flag, ac.valid_flag,
                ac.detection_source,
                EXISTS (
                    SELECT 1 FROM manual_agent_locations ml
                    WHERE ml.agent_type_id = ai.agent_type_id
                      AND ml.normalized_path = ac.normalized_root_path
                ),
                ai.status, ai.confidence, COUNT(r.id),
                GROUP_CONCAT(
                    CASE WHEN r.kind = 'config' AND r.format != 'directory'
                         THEN r.path END,
                    char(31)
                ),
                ai.last_seen_at
            FROM agent_instances ai
            JOIN agent_types at ON at.id = ai.agent_type_id
            JOIN agent_runtimes ar ON ar.agent_instance_id = ai.id
            JOIN agent_configurations ac ON ac.agent_instance_id = ai.id
            LEFT JOIN resources r ON r.agent_configuration_id = ac.id
            GROUP BY ai.id
            ORDER BY
                CASE ai.status
                    WHEN 'changed' THEN 0
                    WHEN 'missing' THEN 1
                    WHEN 'degraded' THEN 2
                    ELSE 3
                END,
                lower(ai.display_name)
            "#,
        )?;
        let mut agents = statement
            .query_map([], map_agent_summary)?
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(filter) = filter {
            if let Some(search) = filter.search.filter(|value| !value.trim().is_empty()) {
                let search = search.to_lowercase();
                agents.retain(|agent| {
                    agent.display_name.to_lowercase().contains(&search)
                        || agent
                            .configuration
                            .root_path
                            .to_lowercase()
                            .contains(&search)
                });
            }
            if let Some(statuses) = filter.statuses.filter(|values| !values.is_empty()) {
                agents.retain(|agent| statuses.contains(&agent.health));
            }
        }
        Ok(agents)
    }

    pub fn get_agent_overview(&self, id: &str) -> Result<AgentOverview, AppError> {
        let connection = self.open()?;
        connection
            .query_row(
                r#"
                SELECT
                    ai.id, ai.agent_type_id, at.display_name, ai.display_name,
                    ar.command_name, ar.executable_path, ar.installed_flag, ar.version,
                    ar.version_probe_status, ar.version_probe_error,
                    ar.resolution_source, ar.distribution,
                    ac.root_path, ac.exists_flag, ac.readable_flag, ac.valid_flag,
                    ac.detection_source,
                    EXISTS (
                        SELECT 1 FROM manual_agent_locations ml
                        WHERE ml.agent_type_id = ai.agent_type_id
                          AND ml.normalized_path = ac.normalized_root_path
                    ),
                    ai.status, ai.confidence,
                    (SELECT COUNT(*) FROM resources r WHERE r.agent_configuration_id = ac.id),
                    (SELECT GROUP_CONCAT(r.path, char(31))
                     FROM resources r
                     WHERE r.agent_configuration_id = ac.id
                       AND r.kind = 'config' AND r.format != 'directory'),
                    ai.last_seen_at, at.adapter_version, ai.metadata_json
                FROM agent_instances ai
                JOIN agent_types at ON at.id = ai.agent_type_id
                JOIN agent_runtimes ar ON ar.agent_instance_id = ai.id
                JOIN agent_configurations ac ON ac.agent_instance_id = ai.id
                WHERE ai.id = ?1
                "#,
                [id],
                |row| {
                    let summary = map_agent_summary(row)?;
                    let metadata_raw: String = row.get(24)?;
                    Ok(AgentOverview {
                        summary,
                        adapter_version: row.get(23)?,
                        metadata: serde_json::from_str(&metadata_raw)
                            .unwrap_or_else(|_| serde_json::json!({})),
                    })
                },
            )
            .optional()?
            .ok_or_else(|| AppError::not_found("Agent"))
    }

    pub fn get_resources(&self, agent_id: &str) -> Result<Vec<Resource>, AppError> {
        let connection = self.open()?;
        let mut statement = connection.prepare(
            r#"
            SELECT
                r.id, r.agent_instance_id, ai.display_name, r.kind, r.logical_key,
                r.path, r.format, r.scope, r.is_sensitive, r.exists_flag,
                r.writable_flag, r.content_hash, r.modified_at, r.size_bytes,
                r.structure_json
            FROM resources r
            JOIN agent_configurations ac ON ac.id = r.agent_configuration_id
            JOIN agent_instances ai ON ai.id = ac.agent_instance_id
            WHERE ac.agent_instance_id = ?1
            ORDER BY r.kind, lower(r.path)
            "#,
        )?;
        let resources = statement
            .query_map([agent_id], map_resource)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(resources)
    }

    pub fn list_resources(
        &self,
        filter: Option<ResourceFilter>,
    ) -> Result<Vec<Resource>, AppError> {
        let connection = self.open()?;
        let mut statement = connection.prepare(
            r#"
            SELECT
                r.id, r.agent_instance_id, ai.display_name, r.kind, r.logical_key,
                r.path, r.format, r.scope, r.is_sensitive, r.exists_flag,
                r.writable_flag, r.content_hash, r.modified_at, r.size_bytes,
                r.structure_json
            FROM resources r
            JOIN agent_configurations ac ON ac.id = r.agent_configuration_id
            JOIN agent_instances ai ON ai.id = ac.agent_instance_id
            ORDER BY lower(ai.display_name), r.kind, lower(r.path)
            "#,
        )?;
        let mut resources = statement
            .query_map([], map_resource)?
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(filter) = filter {
            if let Some(agent_id) = filter.agent_id {
                resources.retain(|resource| resource.agent_instance_id == agent_id);
            }
            if let Some(kinds) = filter.kinds.filter(|values| !values.is_empty()) {
                resources.retain(|resource| kinds.contains(&resource.kind));
            }
            if let Some(search) = filter.search.filter(|value| !value.trim().is_empty()) {
                let search = search.to_lowercase();
                resources.retain(|resource| {
                    resource.path.to_lowercase().contains(&search)
                        || resource.agent_display_name.to_lowercase().contains(&search)
                });
            }
        }
        Ok(resources)
    }

    pub fn get_evidence(&self, agent_id: &str) -> Result<Vec<DiscoveryEvidence>, AppError> {
        let connection = self.open()?;
        let mut statement = connection.prepare(
            r#"
            SELECT
                id, agent_instance_id, evidence_type, source, observed_value,
                success, message, observed_at
            FROM discovery_evidence
            WHERE agent_instance_id = ?1
            ORDER BY success DESC, observed_at DESC
            "#,
        )?;
        let evidence = statement
            .query_map([agent_id], |row| {
                Ok(DiscoveryEvidence {
                    id: row.get(0)?,
                    agent_instance_id: row.get(1)?,
                    evidence_type: row.get(2)?,
                    source: row.get(3)?,
                    observed_value: display_path(row.get::<_, String>(4)?),
                    success: row.get::<_, i64>(5)? != 0,
                    message: row.get(6)?,
                    observed_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(evidence)
    }

    pub fn get_agent_path(&self, id: &str) -> Result<PathBuf, AppError> {
        let connection = self.open()?;
        connection
            .query_row(
                "SELECT normalized_root_path FROM agent_configurations WHERE agent_instance_id = ?1",
                [id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(PathBuf::from)
            .ok_or_else(|| AppError::not_found("Agent"))
    }

    pub fn get_resource_path_and_root(
        &self,
        resource_id: &str,
    ) -> Result<(PathBuf, PathBuf), AppError> {
        let connection = self.open()?;
        connection
            .query_row(
                r#"
                SELECT r.normalized_path, ac.normalized_root_path
                FROM resources r
                JOIN agent_configurations ac ON ac.id = r.agent_configuration_id
                WHERE r.id = ?1
                "#,
                [resource_id],
                |row| {
                    Ok((
                        PathBuf::from(row.get::<_, String>(0)?),
                        PathBuf::from(row.get::<_, String>(1)?),
                    ))
                },
            )
            .optional()?
            .ok_or_else(|| AppError::not_found("资源"))
    }

    pub fn get_agent_by_id(&self, id: &str) -> Result<AgentSummary, AppError> {
        self.list_agents(None)?
            .into_iter()
            .find(|agent| agent.id == id)
            .ok_or_else(|| AppError::not_found("Agent"))
    }

    pub fn list_manual_locations(&self) -> Result<Vec<ManualLocationRequest>, AppError> {
        let connection = self.open()?;
        let mut statement = connection.prepare(
            r#"
            SELECT agent_type_id, path
            FROM manual_agent_locations
            ORDER BY created_at, id
            "#,
        )?;
        let locations = statement
            .query_map([], |row| {
                Ok(ManualLocationRequest {
                    agent_type_id: row.get(0)?,
                    path: display_path(row.get(1)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(locations)
    }

    pub fn remove_manual_agent(&self, id: &str) -> Result<(), AppError> {
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let location: Option<(String, String)> = transaction
            .query_row(
                r#"
                SELECT ai.agent_type_id, ac.normalized_root_path
                FROM agent_instances ai
                JOIN agent_configurations ac ON ac.agent_instance_id = ai.id
                WHERE ai.id = ?1
                "#,
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (agent_type_id, normalized_path) =
            location.ok_or_else(|| AppError::not_found("Agent"))?;
        let removed = transaction.execute(
            r#"
            DELETE FROM manual_agent_locations
            WHERE agent_type_id = ?1 AND normalized_path = ?2
            "#,
            params![agent_type_id, normalized_path],
        )?;
        if removed == 0 {
            return Err(AppError::new(
                "not_manual_agent",
                "该 Agent 不是手动添加的记录，不能通过此操作移除。",
                true,
                Some("自动发现的 Agent 会由扫描结果管理。"),
            ));
        }
        transaction.execute("DELETE FROM agent_instances WHERE id = ?1", [id])?;
        transaction.commit()?;
        Ok(())
    }

    pub fn list_quick_locations(&self) -> Result<Vec<QuickLocation>, AppError> {
        let connection = self.open()?;
        let mut statement = connection.prepare(
            r#"
            SELECT
                id, display_name, path, show_in_tray, sort_order,
                last_opened_at, created_at, updated_at
            FROM quick_locations
            ORDER BY sort_order, created_at, id
            "#,
        )?;
        let locations = statement
            .query_map([], |row| {
                Ok(QuickLocation {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: display_path(row.get(2)?),
                    show_in_tray: row.get::<_, i64>(3)? != 0,
                    sort_order: row.get(4)?,
                    last_opened_at: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(locations)
    }

    pub fn create_quick_location(
        &self,
        name: &str,
        path: &Path,
        show_in_tray: bool,
    ) -> Result<QuickLocation, AppError> {
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let now = now_seconds();
        let id = Uuid::new_v4().to_string();
        let path_value = path.to_string_lossy().into_owned();
        let normalized_path = normalized_path_key(path);
        let sort_order: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM quick_locations",
            [],
            |row| row.get(0),
        )?;
        let inserted = transaction.execute(
            r#"
            INSERT INTO quick_locations (
                id, display_name, path, normalized_path, show_in_tray,
                sort_order, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
            ON CONFLICT(normalized_path) DO NOTHING
            "#,
            params![
                id,
                name,
                path_value,
                normalized_path,
                i64::from(show_in_tray),
                sort_order,
                now
            ],
        )?;
        if inserted == 0 {
            return Err(AppError::new(
                "duplicate_location",
                "这个目录已经绑定过了。",
                true,
                Some("可以编辑现有快捷目录的名称或托盘设置。"),
            ));
        }
        transaction.commit()?;
        self.get_quick_location(&id)
    }

    pub fn update_quick_location(
        &self,
        id: &str,
        name: &str,
        show_in_tray: bool,
    ) -> Result<QuickLocation, AppError> {
        let connection = self.open()?;
        let updated = connection.execute(
            r#"
            UPDATE quick_locations
            SET display_name = ?2, show_in_tray = ?3, updated_at = ?4
            WHERE id = ?1
            "#,
            params![id, name, i64::from(show_in_tray), now_seconds()],
        )?;
        if updated == 0 {
            return Err(AppError::not_found("快捷目录"));
        }
        self.get_quick_location(id)
    }

    pub fn reorder_quick_locations(&self, ids: &[String]) -> Result<(), AppError> {
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let now = now_seconds();
        for (index, id) in ids.iter().enumerate() {
            transaction.execute(
                r#"
                UPDATE quick_locations
                SET sort_order = ?2, updated_at = ?3
                WHERE id = ?1
                "#,
                params![id, index as i64, now],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn remove_quick_location(&self, id: &str) -> Result<(), AppError> {
        let connection = self.open()?;
        let removed = connection.execute("DELETE FROM quick_locations WHERE id = ?1", [id])?;
        if removed == 0 {
            return Err(AppError::not_found("快捷目录"));
        }
        Ok(())
    }

    pub fn get_quick_location_path(&self, id: &str) -> Result<PathBuf, AppError> {
        let connection = self.open()?;
        connection
            .query_row(
                "SELECT path FROM quick_locations WHERE id = ?1",
                [id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(PathBuf::from)
            .ok_or_else(|| AppError::not_found("快捷目录"))
    }

    pub fn mark_quick_location_opened(&self, id: &str) -> Result<(), AppError> {
        let connection = self.open()?;
        connection.execute(
            r#"
            UPDATE quick_locations
            SET last_opened_at = ?2, updated_at = ?2
            WHERE id = ?1
            "#,
            params![id, now_seconds()],
        )?;
        Ok(())
    }

    fn get_quick_location(&self, id: &str) -> Result<QuickLocation, AppError> {
        self.list_quick_locations()?
            .into_iter()
            .find(|location| location.id == id)
            .ok_or_else(|| AppError::not_found("快捷目录"))
    }
}

fn resources_changed(
    transaction: &Transaction<'_>,
    agent_id: &str,
    draft: &AgentDraft,
) -> Result<bool, AppError> {
    let mut statement = transaction.prepare(
        "SELECT logical_key, normalized_path, content_hash FROM resources WHERE agent_instance_id = ?1",
    )?;
    let old = statement
        .query_map([agent_id], |row| {
            Ok((
                (row.get::<_, String>(0)?, row.get::<_, String>(1)?),
                row.get::<_, Option<String>>(2)?,
            ))
        })?
        .collect::<Result<HashMap<_, _>, _>>()?;
    if old.is_empty() {
        return Ok(false);
    }
    if old.keys().any(|(logical_key, _)| logical_key.contains(':')) {
        return Ok(false);
    }
    let new = draft
        .configuration
        .resources
        .iter()
        .map(|resource| {
            (
                (
                    resource.logical_key.clone(),
                    normalized_path_key(&resource.normalized_path),
                ),
                resource.content_hash.clone(),
            )
        })
        .collect::<HashMap<_, _>>();
    Ok(old != new)
}

fn replace_resources(
    transaction: &Transaction<'_>,
    agent_id: &str,
    configuration_id: &str,
    draft: &AgentDraft,
    observed_at: i64,
) -> Result<(), AppError> {
    transaction.execute(
        "DELETE FROM resources WHERE agent_instance_id = ?1",
        [agent_id],
    )?;
    for resource in &draft.configuration.resources {
        let structure_json = resource.entry_count.map(|entry_count| {
            serde_json::json!({
                "entryCount": entry_count,
                "scanTruncated": resource.scan_truncated,
            })
            .to_string()
        });
        transaction.execute(
            r#"
            INSERT INTO resources (
                id, agent_instance_id, agent_configuration_id, kind, logical_key, path, normalized_path,
                format, scope, is_sensitive, exists_flag, writable_flag,
                content_hash, modified_at, size_bytes, structure_json,
                last_observed_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17
            )
            "#,
            params![
                Uuid::new_v4().to_string(),
                agent_id,
                configuration_id,
                resource.kind.as_str(),
                resource.logical_key,
                resource.path.to_string_lossy(),
                normalized_path_key(&resource.normalized_path),
                resource.format,
                resource.scope,
                resource.is_sensitive as i64,
                resource.exists as i64,
                resource.writable as i64,
                resource.content_hash,
                resource.modified_at,
                resource.size_bytes,
                structure_json,
                observed_at,
            ],
        )?;
    }
    Ok(())
}

fn replace_evidence(
    transaction: &Transaction<'_>,
    agent_id: &str,
    draft: &AgentDraft,
    observed_at: i64,
) -> Result<(), AppError> {
    transaction.execute(
        "DELETE FROM discovery_evidence WHERE agent_instance_id = ?1",
        [agent_id],
    )?;
    for evidence in &draft.evidence {
        transaction.execute(
            r#"
            INSERT INTO discovery_evidence (
                id, agent_instance_id, evidence_type, source, observed_value,
                success, message, observed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                Uuid::new_v4().to_string(),
                agent_id,
                evidence.evidence_type,
                evidence.source,
                evidence.observed_value,
                evidence.success as i64,
                evidence.message,
                observed_at,
            ],
        )?;
    }
    Ok(())
}

fn map_agent_summary(row: &Row<'_>) -> rusqlite::Result<AgentSummary> {
    let status: String = row.get(18)?;
    let confidence: String = row.get(19)?;
    let executable_path = row.get::<_, Option<String>>(5)?.map(display_path);
    Ok(AgentSummary {
        id: row.get(0)?,
        agent_type_id: row.get(1)?,
        agent_type_name: row.get(2)?,
        display_name: row.get(3)?,
        runtime: RuntimeSummary {
            command_name: row.get(4)?,
            executable_path,
            installed: row.get::<_, i64>(6)? != 0,
            version: row.get(7)?,
            version_probe_status: VersionProbeStatus::from_db(&row.get::<_, String>(8)?),
            version_probe_error: row.get(9)?,
            resolution_source: RuntimeResolutionSource::from_db(&row.get::<_, String>(10)?),
            distribution: RuntimeDistribution::from_db(&row.get::<_, String>(11)?),
        },
        configuration: ConfigurationSummary {
            root_path: display_path(row.get(12)?),
            config_files: row
                .get::<_, Option<String>>(21)?
                .map(|value| {
                    value
                        .split('\u{1f}')
                        .map(|path| display_path(path.to_owned()))
                        .collect()
                })
                .unwrap_or_default(),
            exists: row.get::<_, i64>(13)? != 0,
            readable: row.get::<_, i64>(14)? != 0,
            valid: row.get::<_, i64>(15)? != 0,
            detection_source: row.get(16)?,
            resource_count: row.get(20)?,
            manually_added: row.get::<_, i64>(17)? != 0,
        },
        health: HealthStatus::from_db(&status),
        confidence: Confidence::from_db(&confidence),
        last_seen_at: row.get(22)?,
    })
}

fn map_resource(row: &Row<'_>) -> rusqlite::Result<Resource> {
    let kind: String = row.get(3)?;
    let structure_json = row
        .get::<_, Option<String>>(14)?
        .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok());
    let entry_count = structure_json
        .as_ref()
        .and_then(|value| value.get("entryCount"))
        .and_then(serde_json::Value::as_i64);
    let scan_truncated = structure_json
        .as_ref()
        .and_then(|value| value.get("scanTruncated"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    Ok(Resource {
        id: row.get(0)?,
        agent_instance_id: row.get(1)?,
        agent_display_name: row.get(2)?,
        kind: ResourceKind::from_db(&kind),
        logical_key: row.get(4)?,
        path: display_path(row.get(5)?),
        format: row.get(6)?,
        scope: row.get(7)?,
        is_sensitive: row.get::<_, i64>(8)? != 0,
        exists: row.get::<_, i64>(9)? != 0,
        writable: row.get::<_, i64>(10)? != 0,
        content_hash: row.get(11)?,
        modified_at: row.get(12)?,
        size_bytes: row.get(13)?,
        entry_count,
        scan_truncated,
    })
}

fn display_path(value: String) -> String {
    if !cfg!(windows) {
        return value;
    }
    if let Some(path) = value.strip_prefix("\\\\?\\UNC\\") {
        return format!("\\\\{path}");
    }
    value.strip_prefix("\\\\?\\").unwrap_or(&value).to_owned()
}

fn normalized_path_key(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        value.to_ascii_lowercase()
    } else {
        value
    }
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        agent::{AgentDraft, Confidence},
        configuration::ConfigurationObservation,
        discovery::DiscoveryEvidenceDraft,
        health::HealthStatus,
        resource::{ResourceKind, ResourceObservation},
        runtime::{
            RuntimeDistribution, RuntimeObservation, RuntimeResolutionSource, VersionProbeStatus,
        },
    };

    fn test_database() -> (tempfile::TempDir, Database) {
        let directory = tempfile::tempdir().expect("temp database");
        let database =
            Database::initialize(directory.path().join("agent-hub.db")).expect("database");
        database
            .upsert_agent_types(&[AgentTypeDescriptor {
                id: "codex",
                display_name: "Codex",
                icon_key: "codex",
                adapter_version: 1,
            }])
            .expect("agent type");
        (directory, database)
    }

    fn draft(root: &Path, hash: &str) -> AgentDraft {
        AgentDraft {
            agent_type_id: "codex".to_owned(),
            display_name: "Codex".to_owned(),
            runtime: RuntimeObservation {
                command_name: "codex".to_owned(),
                executable_path: None,
                installed: false,
                version: None,
                version_probe_status: VersionProbeStatus::NotAttempted,
                version_probe_error: None,
                resolution_source: RuntimeResolutionSource::NotFound,
                distribution: RuntimeDistribution::Unknown,
            },
            configuration: ConfigurationObservation {
                root_path: root.to_path_buf(),
                config_files: vec![root.join("config.toml")],
                exists: true,
                readable: true,
                valid: true,
                detection_source: "manual".to_owned(),
                resources: vec![ResourceObservation {
                    kind: ResourceKind::Config,
                    logical_key: "config".to_owned(),
                    path: root.join("config.toml"),
                    normalized_path: root.join("config.toml"),
                    format: "toml".to_owned(),
                    scope: "global".to_owned(),
                    is_sensitive: false,
                    exists: true,
                    writable: true,
                    content_hash: Some(hash.to_owned()),
                    modified_at: Some(1),
                    size_bytes: Some(1),
                    entry_count: None,
                    scan_truncated: false,
                }],
            },
            health: HealthStatus::ConfigOnly,
            confidence: Confidence::Medium,
            metadata: serde_json::json!({}),
            evidence: vec![DiscoveryEvidenceDraft {
                evidence_type: "manual".to_owned(),
                source: "manual".to_owned(),
                observed_value: root.to_string_lossy().into_owned(),
                success: true,
                message: "手动添加".to_owned(),
            }],
        }
    }

    #[test]
    fn initializes_empty_database_and_reopens() {
        let (directory, database) = test_database();
        assert!(database.list_agents(None).expect("list").is_empty());
        let reopened = Database::initialize(directory.path().join("agent-hub.db")).expect("reopen");
        assert!(reopened
            .list_agents(None)
            .expect("reopened list")
            .is_empty());
    }

    #[test]
    fn migrates_v1_rows_into_runtime_and_configuration_tables() {
        let directory = tempfile::tempdir().expect("legacy database");
        let path = directory.path().join("agent-hub.db");
        let connection = Connection::open(&path).expect("legacy connection");
        connection
            .execute_batch(
                r#"
                CREATE TABLE agent_types (
                    id TEXT PRIMARY KEY, display_name TEXT NOT NULL,
                    adapter_version INTEGER NOT NULL, icon_key TEXT NOT NULL,
                    capabilities_json TEXT NOT NULL, created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE TABLE agent_instances (
                    id TEXT PRIMARY KEY, agent_type_id TEXT NOT NULL,
                    display_name TEXT NOT NULL, executable_path TEXT,
                    config_root TEXT NOT NULL, normalized_config_root TEXT NOT NULL,
                    detected_version TEXT, discovery_source TEXT NOT NULL,
                    status TEXT NOT NULL, confidence TEXT NOT NULL,
                    metadata_json TEXT NOT NULL, last_seen_at INTEGER NOT NULL,
                    created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
                    UNIQUE(agent_type_id, normalized_config_root)
                );
                CREATE TABLE resources (
                    id TEXT PRIMARY KEY, agent_instance_id TEXT NOT NULL,
                    kind TEXT NOT NULL, logical_key TEXT NOT NULL,
                    path TEXT NOT NULL, normalized_path TEXT NOT NULL,
                    format TEXT NOT NULL, scope TEXT NOT NULL,
                    is_sensitive INTEGER NOT NULL, exists_flag INTEGER NOT NULL,
                    writable_flag INTEGER NOT NULL, content_hash TEXT,
                    modified_at INTEGER, size_bytes INTEGER, structure_json TEXT,
                    last_observed_at INTEGER NOT NULL,
                    UNIQUE(agent_instance_id, logical_key, normalized_path)
                );
                INSERT INTO agent_types VALUES
                    ('codex', 'Codex', 2, 'codex', '{}', 1, 1);
                INSERT INTO agent_instances VALUES
                    ('legacy-agent', 'codex', 'Codex', 'C:/bin/codex.exe',
                     'C:/Users/demo/.codex', 'c:/users/demo/.codex',
                     '1.0.0', 'manual', 'ready', 'high', '{}', 1, 1, 1);
                INSERT INTO resources VALUES
                    ('legacy-resource', 'legacy-agent', 'config', 'config',
                     'C:/Users/demo/.codex/config.toml',
                     'c:/users/demo/.codex/config.toml', 'toml', 'global',
                     0, 1, 1, 'hash', 1, 1, NULL, 1);
                PRAGMA user_version = 1;
                "#,
            )
            .expect("legacy schema");
        drop(connection);

        let database = Database::initialize(path).expect("migrated database");
        let agent = database
            .get_agent_by_id("legacy-agent")
            .expect("migrated agent");

        assert!(agent.runtime.installed);
        assert_eq!(agent.runtime.command_name, "codex");
        assert_eq!(agent.runtime.version.as_deref(), Some("1.0.0"));
        assert_eq!(
            agent.runtime.version_probe_status,
            VersionProbeStatus::Detected
        );
        assert!(agent.configuration.exists);
        assert!(agent.configuration.manually_added);
        assert_eq!(agent.configuration.resource_count, 1);
        assert_eq!(agent.health, HealthStatus::Healthy);
        assert_eq!(
            database
                .get_resources("legacy-agent")
                .expect("migrated resources")
                .len(),
            1
        );
    }

    #[test]
    fn repeated_scan_is_idempotent_and_hash_change_is_reported() {
        let (_directory, database) = test_database();
        let root = PathBuf::from("C:/fixture/.codex");
        let first = database
            .reconcile(vec![draft(&root, "one")], false)
            .expect("first");
        let second = database
            .reconcile(vec![draft(&root, "one")], false)
            .expect("second");
        let changed = database
            .reconcile(vec![draft(&root, "two")], false)
            .expect("changed");

        assert_eq!(first.agent_ids[0], second.agent_ids[0]);
        assert_eq!(database.list_agents(None).expect("agents").len(), 1);
        assert_eq!(second.result.changed_count, 0);
        assert_eq!(changed.result.changed_count, 1);
        assert_eq!(
            database.list_agents(None).expect("agents")[0].health,
            HealthStatus::Changed
        );
    }

    #[test]
    fn quick_locations_are_unique_editable_and_ordered() {
        let (_directory, database) = test_database();
        let first_path = PathBuf::from("C:/fixture/prompts");
        let second_path = PathBuf::from("C:/fixture/projects");
        let first = database
            .create_quick_location("Prompts", &first_path, true)
            .expect("first location");
        let second = database
            .create_quick_location("Projects", &second_path, false)
            .expect("second location");

        let duplicate = database
            .create_quick_location("Duplicate", &first_path, true)
            .expect_err("duplicate path");
        assert_eq!(duplicate.code, "duplicate_location");

        let updated = database
            .update_quick_location(&second.id, "Workspaces", true)
            .expect("updated location");
        assert_eq!(updated.name, "Workspaces");
        assert!(updated.show_in_tray);

        database
            .reorder_quick_locations(&[second.id.clone(), first.id.clone()])
            .expect("reorder");
        let reordered = database.list_quick_locations().expect("list");
        assert_eq!(reordered[0].id, second.id);
        assert_eq!(reordered[1].id, first.id);

        database.remove_quick_location(&first.id).expect("remove");
        assert_eq!(database.list_quick_locations().expect("list").len(), 1);
    }

    #[cfg(windows)]
    #[test]
    fn strips_windows_verbatim_prefix_only_from_display_values() {
        assert_eq!(
            display_path(r"\\?\C:\Users\demo\.codex".to_owned()),
            r"C:\Users\demo\.codex"
        );
        assert_eq!(
            display_path(r"\\?\UNC\server\share\.claude".to_owned()),
            r"\\server\share\.claude"
        );
    }
}
