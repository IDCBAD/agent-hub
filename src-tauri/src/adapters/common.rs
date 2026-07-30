use std::{
    collections::{HashMap, HashSet},
    env,
    ffi::OsString,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};
use wait_timeout::ChildExt;
use walkdir::WalkDir;

use crate::{
    domain::{
        agent::{AgentDraft, AgentTypeDescriptor, Confidence},
        configuration::ConfigurationObservation,
        discovery::DiscoveryEvidenceDraft,
        health::evaluate_health,
        resource::{ResourceKind, ResourceObservation},
        runtime::{
            RuntimeDistribution, RuntimeObservation, RuntimeResolutionSource, VersionProbeStatus,
        },
    },
    error::AppError,
};

use super::AgentAdapter;

const MAX_HASH_BYTES: u64 = 2 * 1024 * 1024;
const MAX_DIRECTORY_ENTRIES: usize = 250;
const VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug)]
struct VersionProbeResult {
    version: Option<String>,
    status: VersionProbeStatus,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiscoveryContext {
    home: PathBuf,
    env_values: HashMap<String, OsString>,
    path_entries: Vec<PathBuf>,
}

impl DiscoveryContext {
    pub fn from_system() -> Result<Self, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::internal("无法确定当前用户目录，Agent 自动发现无法启动。"))?;
        let env_values = env::vars_os()
            .map(|(key, value)| (key.to_string_lossy().into_owned(), value))
            .collect();
        let path_entries = env::var_os("PATH")
            .map(|value| env::split_paths(&value).collect())
            .unwrap_or_default();
        Ok(Self {
            home,
            env_values,
            path_entries,
        })
    }

    #[cfg(test)]
    pub fn for_test(
        home: PathBuf,
        path_entries: Vec<PathBuf>,
        env_values: HashMap<String, OsString>,
    ) -> Self {
        Self {
            home,
            env_values,
            path_entries,
        }
    }

    fn env_path(&self, name: &str) -> Option<PathBuf> {
        self.env_values.get(name).map(PathBuf::from)
    }

    fn find_executable_in(
        &self,
        directories: &[PathBuf],
        names: &[&str],
    ) -> Option<(PathBuf, String)> {
        for directory in directories {
            for name in names {
                let candidates = executable_candidates(directory, name);
                if let Some(path) = candidates.into_iter().find(|path| path.is_file()) {
                    return canonical_or_absolute(&path)
                        .ok()
                        .map(|path| (path, (*name).to_owned()));
                }
            }
        }
        None
    }

    fn default_runtime_locations(&self) -> Vec<PathBuf> {
        let mut locations = Vec::new();
        if let Some(app_data) = self.env_path("APPDATA") {
            locations.push(app_data.join("npm"));
        }
        locations.push(self.home.join("AppData/Roaming/npm"));
        locations.push(self.home.join(".local/bin"));
        locations.push(PathBuf::from("/usr/local/bin"));
        locations.push(PathBuf::from("/opt/homebrew/bin"));

        let mut seen = HashSet::new();
        locations
            .into_iter()
            .filter(|path| seen.insert(normalized_key(path)))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ResourceRule {
    logical_key: &'static str,
    relative_path: &'static str,
    kind: ResourceKind,
    sensitive: bool,
    recursive: bool,
}

impl ResourceRule {
    pub fn file(
        logical_key: &'static str,
        relative_path: &'static str,
        kind: ResourceKind,
        sensitive: bool,
    ) -> Self {
        Self {
            logical_key,
            relative_path,
            kind,
            sensitive,
            recursive: false,
        }
    }

    pub fn directory(
        logical_key: &'static str,
        relative_path: &'static str,
        kind: ResourceKind,
    ) -> Self {
        Self {
            logical_key,
            relative_path,
            kind,
            sensitive: false,
            recursive: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfiguredAdapter {
    id: &'static str,
    display_name: &'static str,
    icon_key: &'static str,
    default_directory: &'static str,
    environment_variables: Vec<&'static str>,
    executable_names: Vec<&'static str>,
    runtime_directories: Vec<&'static str>,
    markers: Vec<&'static str>,
    rules: Vec<ResourceRule>,
}

impl ConfiguredAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &'static str,
        display_name: &'static str,
        icon_key: &'static str,
        default_directory: &'static str,
        environment_variables: &[&'static str],
        executable_names: &[&'static str],
        runtime_directories: &[&'static str],
        markers: &[&'static str],
        rules: Vec<ResourceRule>,
    ) -> Self {
        Self {
            id,
            display_name,
            icon_key,
            default_directory,
            environment_variables: environment_variables.to_vec(),
            executable_names: executable_names.to_vec(),
            runtime_directories: runtime_directories.to_vec(),
            markers: markers.to_vec(),
            rules,
        }
    }

    fn configuration_candidates_for(&self, context: &DiscoveryContext) -> Vec<(PathBuf, String)> {
        let mut candidates = Vec::new();
        for variable in &self.environment_variables {
            if let Some(path) = context.env_path(variable) {
                candidates.push((path, format!("env:{variable}")));
            }
        }
        candidates.push((
            context.home.join(self.default_directory),
            "default".to_owned(),
        ));

        let mut seen = HashSet::new();
        candidates
            .into_iter()
            .filter(|(path, _)| seen.insert(normalized_key(path)))
            .collect()
    }

    fn detect_runtime_for(&self, context: &DiscoveryContext) -> RuntimeObservation {
        let path_match = context.find_executable_in(&context.path_entries, &self.executable_names);
        let (executable_path, command_name, resolution_source) =
            if let Some((path, command_name)) = path_match {
                (Some(path), command_name, RuntimeResolutionSource::Path)
            } else {
                let mut default_locations = self
                    .runtime_directories
                    .iter()
                    .map(|path| context.home.join(path))
                    .collect::<Vec<_>>();
                default_locations.extend(context.default_runtime_locations());
                let default_match =
                    context.find_executable_in(&default_locations, &self.executable_names);
                match default_match {
                    Some((path, command_name)) => (
                        Some(path),
                        command_name,
                        RuntimeResolutionSource::DefaultPath,
                    ),
                    None => (
                        None,
                        self.executable_names
                            .first()
                            .copied()
                            .unwrap_or(self.id)
                            .to_owned(),
                        RuntimeResolutionSource::NotFound,
                    ),
                }
            };
        let distribution = executable_path
            .as_deref()
            .map(runtime_distribution)
            .unwrap_or(RuntimeDistribution::Unknown);
        let version_probe =
            executable_path
                .as_deref()
                .map(detect_version)
                .unwrap_or(VersionProbeResult {
                    version: None,
                    status: VersionProbeStatus::NotAttempted,
                    error: None,
                });

        RuntimeObservation {
            command_name,
            installed: executable_path.is_some(),
            executable_path,
            version: version_probe.version,
            version_probe_status: version_probe.status,
            version_probe_error: version_probe.error,
            resolution_source,
            distribution,
        }
    }

    fn detect_configuration_for(
        &self,
        path: &Path,
        source: &str,
        allow_missing: bool,
        context: &DiscoveryContext,
    ) -> Result<(ConfigurationObservation, Option<&'static str>), AppError> {
        let display_path = expand_tilde(path, &context.home);
        let config_exists = display_path.is_dir();
        if !config_exists && !allow_missing {
            return Err(AppError::invalid_path(format!(
                "目录不存在或不是文件夹：{}",
                display_path.display()
            )));
        }

        let config_root = canonical_or_absolute(&display_path)?;
        let marker = self
            .markers
            .iter()
            .find(|marker| config_root.join(marker).exists())
            .copied();
        let readable = config_exists && fs::read_dir(&config_root).is_ok();
        let resources = if readable {
            self.inventory(&config_root)?
        } else {
            Vec::new()
        };
        let config_files = resources
            .iter()
            .filter(|resource| {
                resource.kind == ResourceKind::Config && resource.format != "directory"
            })
            .map(|resource| resource.path.clone())
            .collect();
        let valid = config_exists && (marker.is_some() || !resources.is_empty());

        Ok((
            ConfigurationObservation {
                root_path: config_root,
                config_files,
                exists: config_exists,
                readable,
                valid,
                detection_source: source.to_owned(),
                resources,
            },
            marker,
        ))
    }

    fn compose_draft(
        &self,
        runtime: RuntimeObservation,
        configuration: ConfigurationObservation,
        marker: Option<&'static str>,
    ) -> AgentDraft {
        let health = evaluate_health(&runtime, &configuration);
        let confidence = match (configuration.exists, runtime.installed, marker.is_some()) {
            (true, true, true) => Confidence::High,
            (true, _, true) | (true, true, false) => Confidence::Medium,
            _ => Confidence::Low,
        };
        let mut evidence = vec![
            DiscoveryEvidenceDraft {
                evidence_type: "config_root".to_owned(),
                source: configuration.detection_source.clone(),
                observed_value: configuration.root_path.to_string_lossy().into_owned(),
                success: configuration.exists && configuration.readable,
                message: if configuration.exists && configuration.readable {
                    "配置目录存在且可访问".to_owned()
                } else {
                    "预期配置目录尚不存在或不可访问".to_owned()
                },
            },
            DiscoveryEvidenceDraft {
                evidence_type: "signature".to_owned(),
                source: "adapter".to_owned(),
                observed_value: marker
                    .map(|value| {
                        configuration
                            .root_path
                            .join(value)
                            .to_string_lossy()
                            .into_owned()
                    })
                    .unwrap_or_else(|| "未发现已知特征文件".to_owned()),
                success: marker.is_some(),
                message: marker
                    .map(|value| format!("发现特征资源 {value}"))
                    .unwrap_or_else(|| "目录存在，但未发现强特征资源".to_owned()),
            },
        ];
        evidence.push(DiscoveryEvidenceDraft {
            evidence_type: "runtime".to_owned(),
            source: runtime.resolution_source.as_str().to_owned(),
            observed_value: runtime
                .executable_path
                .as_ref()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_else(|| "未找到可执行文件".to_owned()),
            success: runtime.installed,
            message: if runtime.installed {
                "已定位 Agent Runtime".to_owned()
            } else {
                "未在 PATH、npm 或默认安装位置确认 Runtime".to_owned()
            },
        });
        let config_file_count = configuration.config_files.len();
        let runtime_distribution = runtime.distribution.as_str();

        AgentDraft {
            agent_type_id: self.id.to_owned(),
            display_name: self.display_name.to_owned(),
            runtime,
            configuration,
            health,
            confidence,
            metadata: serde_json::json!({
                "marker": marker,
                "configFileCount": config_file_count,
                "inventoryRuleCount": self.rules.len(),
                "runtimeDistribution": runtime_distribution,
            }),
            evidence,
        }
    }

    fn inventory(&self, root: &Path) -> Result<Vec<ResourceObservation>, AppError> {
        let mut observations = Vec::new();

        for rule in &self.rules {
            let target = root.join(rule.relative_path);
            if !target.exists() {
                continue;
            }
            observations.push(observe_path(
                &target,
                rule.kind,
                rule.logical_key.to_owned(),
                rule.sensitive,
                rule.recursive,
            )?);
        }

        observations.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(observations)
    }
}

impl AgentAdapter for ConfiguredAdapter {
    fn descriptor(&self) -> AgentTypeDescriptor {
        AgentTypeDescriptor {
            id: self.id,
            display_name: self.display_name,
            icon_key: self.icon_key,
            adapter_version: 5,
        }
    }

    fn detect_runtime(&self, context: &DiscoveryContext) -> Result<RuntimeObservation, AppError> {
        Ok(self.detect_runtime_for(context))
    }

    fn configuration_candidates(&self, context: &DiscoveryContext) -> Vec<(PathBuf, String)> {
        self.configuration_candidates_for(context)
    }

    fn detect_configuration(
        &self,
        path: &Path,
        source: &str,
        allow_missing: bool,
        context: &DiscoveryContext,
    ) -> Result<ConfigurationObservation, AppError> {
        self.detect_configuration_for(path, source, allow_missing, context)
            .map(|(configuration, _)| configuration)
    }

    fn discover(&self, context: &DiscoveryContext) -> Result<Vec<AgentDraft>, AppError> {
        let runtime = self.detect_runtime(context)?;
        let mut drafts = Vec::new();

        for (path, source) in self.configuration_candidates(context) {
            let is_default = source == "default";
            if path.is_dir() || (is_default && runtime.installed) {
                let configuration = self.detect_configuration(&path, &source, true, context)?;
                let marker = self
                    .markers
                    .iter()
                    .find(|marker| configuration.root_path.join(marker).exists())
                    .copied();
                drafts.push(self.compose_draft(runtime.clone(), configuration, marker));
            }
        }

        Ok(drafts)
    }

    fn detect_path(
        &self,
        path: &Path,
        source: &str,
        allow_missing: bool,
    ) -> Result<AgentDraft, AppError> {
        let context = DiscoveryContext::from_system()?;
        let runtime = self.detect_runtime(&context)?;
        let configuration = self.detect_configuration(path, source, allow_missing, &context)?;
        let marker = self
            .markers
            .iter()
            .find(|marker| configuration.root_path.join(marker).exists())
            .copied();
        Ok(self.compose_draft(runtime, configuration, marker))
    }
}

fn observe_path(
    path: &Path,
    kind: ResourceKind,
    logical_key: String,
    explicitly_sensitive: bool,
    inventory_descendants: bool,
) -> Result<ResourceObservation, AppError> {
    let normalized_path = canonical_or_absolute(path)?;
    let metadata = fs::metadata(&normalized_path)?;
    let is_directory = metadata.is_dir();
    let size_bytes = (!is_directory).then_some(metadata.len() as i64);
    let modified_at = metadata.modified().ok().and_then(system_time_seconds);
    let (content_hash, entry_count, scan_truncated) = if is_directory && inventory_descendants {
        let fingerprint = fingerprint_directory(&normalized_path)?;
        (
            Some(fingerprint.content_hash),
            Some(fingerprint.entry_count),
            fingerprint.truncated,
        )
    } else if is_directory {
        (None, Some(0), false)
    } else if metadata.len() <= MAX_HASH_BYTES {
        (Some(hash_file(&normalized_path)?), None, false)
    } else {
        (None, None, false)
    };

    Ok(ResourceObservation {
        kind,
        logical_key,
        path: path.to_path_buf(),
        normalized_path,
        format: resource_format(path, is_directory),
        scope: "global".to_owned(),
        is_sensitive: explicitly_sensitive || is_sensitive_path(path),
        exists: true,
        writable: !metadata.permissions().readonly(),
        content_hash,
        modified_at,
        size_bytes,
        entry_count,
        scan_truncated,
    })
}

struct DirectoryFingerprint {
    content_hash: String,
    entry_count: i64,
    truncated: bool,
}

fn fingerprint_directory(path: &Path) -> Result<DirectoryFingerprint, AppError> {
    let mut hasher = Sha256::new();
    let mut entry_count = 0_i64;
    let mut truncated = false;

    for entry in WalkDir::new(path)
        .follow_links(false)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.depth() > 0)
    {
        if entry_count as usize >= MAX_DIRECTORY_ENTRIES {
            truncated = true;
            break;
        }
        let relative = entry
            .path()
            .strip_prefix(path)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        let metadata = fs::metadata(entry.path())?;
        let modified_nanos = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        hasher.update(relative.as_bytes());
        hasher.update([u8::from(metadata.is_dir())]);
        hasher.update(metadata.len().to_le_bytes());
        hasher.update(modified_nanos.to_le_bytes());
        entry_count += 1;
    }

    Ok(DirectoryFingerprint {
        content_hash: format!("{:x}", hasher.finalize()),
        entry_count,
        truncated,
    })
}

fn hash_file(path: &Path) -> Result<String, AppError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn resource_format(path: &Path, is_directory: bool) -> String {
    if is_directory {
        return "directory".to_owned();
    }
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "json" => "json",
        "jsonc" => "jsonc",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" => "markdown",
        extension if !extension.is_empty() => extension,
        _ => "file",
    }
    .to_owned()
}

fn is_sensitive_path(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    name == ".env"
        || name.starts_with(".env.")
        || name.contains("auth")
        || name.contains("token")
        || name.contains("credential")
        || name.ends_with(".pem")
        || name.ends_with(".key")
        || name.ends_with(".p12")
}

fn detect_version(executable: &Path) -> VersionProbeResult {
    let mut command = version_command(executable);
    let mut child = match command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return VersionProbeResult {
                version: None,
                status: if error.raw_os_error() == Some(193) {
                    VersionProbeStatus::Unsupported
                } else {
                    VersionProbeStatus::Failed
                },
                error: Some(format!("无法启动版本命令：{error}")),
            };
        }
    };
    let wait_result = match child.wait_timeout(VERSION_PROBE_TIMEOUT) {
        Ok(result) => result,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return VersionProbeResult {
                version: None,
                status: VersionProbeStatus::Failed,
                error: Some(format!("等待版本命令失败：{error}")),
            };
        }
    };
    match wait_result {
        Some(status) if status.success() => {
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(mut output) = child.stdout.take() {
                if let Err(error) = output.read_to_string(&mut stdout) {
                    return VersionProbeResult {
                        version: None,
                        status: VersionProbeStatus::Failed,
                        error: Some(format!("读取版本输出失败：{error}")),
                    };
                }
            }
            if let Some(mut output) = child.stderr.take() {
                if let Err(error) = output.read_to_string(&mut stderr) {
                    return VersionProbeResult {
                        version: None,
                        status: VersionProbeStatus::Failed,
                        error: Some(format!("读取版本错误输出失败：{error}")),
                    };
                }
            }
            let version_output = if stdout.trim().is_empty() {
                stderr.as_str()
            } else {
                stdout.as_str()
            };
            let version = version_output
                .lines()
                .next()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
            match version {
                Some(version) => VersionProbeResult {
                    version: Some(version),
                    status: VersionProbeStatus::Detected,
                    error: None,
                },
                None => VersionProbeResult {
                    version: None,
                    status: VersionProbeStatus::Failed,
                    error: Some("版本命令执行成功，但没有返回版本信息。".to_owned()),
                },
            }
        }
        Some(status) => {
            let unsupported = status.code() == Some(2);
            VersionProbeResult {
                version: None,
                status: if unsupported {
                    VersionProbeStatus::Unsupported
                } else {
                    VersionProbeStatus::Failed
                },
                error: Some(match status.code() {
                    Some(code) => format!("版本命令退出码为 {code}。"),
                    None => "版本命令异常终止。".to_owned(),
                }),
            }
        }
        None => {
            let _ = child.kill();
            let _ = child.wait();
            VersionProbeResult {
                version: None,
                status: VersionProbeStatus::TimedOut,
                error: Some(format!(
                    "版本命令在 {} 秒内没有完成。",
                    VERSION_PROBE_TIMEOUT.as_secs()
                )),
            }
        }
    }
}

fn version_command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    command.arg("--version");
    command
}

fn canonical_or_absolute(path: &Path) -> Result<PathBuf, AppError> {
    if path.exists() {
        return fs::canonicalize(path).map_err(AppError::from);
    }
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(env::current_dir()?.join(path))
}

fn expand_tilde(path: &Path, home: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    if raw == "~" {
        home.to_path_buf()
    } else if let Some(rest) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
        home.join(rest)
    } else {
        path.to_path_buf()
    }
}

fn normalized_key(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        value.to_ascii_lowercase()
    } else {
        value
    }
}

fn system_time_seconds(time: SystemTime) -> Option<i64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs() as i64)
}

fn executable_candidates(directory: &Path, name: &str) -> Vec<PathBuf> {
    if cfg!(windows) {
        [".exe", ".com", ".cmd", ".bat"]
            .into_iter()
            .map(|extension| directory.join(format!("{name}{extension}")))
            .collect()
    } else {
        vec![directory.join(name)]
    }
}

fn runtime_distribution(path: &Path) -> RuntimeDistribution {
    let components = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect::<Vec<_>>();
    if components
        .iter()
        .any(|value| value == "npm" || value == "node_modules")
    {
        RuntimeDistribution::Npm
    } else if components.iter().any(|value| {
        value == "pipx" || value == "site-packages" || value == "python" || value == "scripts"
    }) {
        RuntimeDistribution::Python
    } else if components.iter().any(|value| value == ".kimi-code") {
        RuntimeDistribution::Bundled
    } else if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("exe"))
    {
        RuntimeDistribution::Native
    } else {
        RuntimeDistribution::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_adapter_discovers_fixture_resources_without_scanning_home() {
        let fixture = tempfile::tempdir().expect("fixture");
        let root = fixture.path().join(".sample");
        fs::create_dir_all(root.join("skills/example")).expect("skill directory");
        fs::write(root.join("config.toml"), "model = \"local\"").expect("config");
        fs::write(root.join("skills/example/SKILL.md"), "# Example").expect("skill");
        let adapter = ConfiguredAdapter::new(
            "sample",
            "Sample",
            "sample",
            ".sample",
            &[],
            &["not-a-real-executable"],
            &[],
            &["config.toml"],
            vec![
                ResourceRule::file("config", "config.toml", ResourceKind::Config, false),
                ResourceRule::directory("skills", "skills", ResourceKind::Skill),
            ],
        );
        let context =
            DiscoveryContext::for_test(fixture.path().to_path_buf(), Vec::new(), HashMap::new());

        let drafts = adapter.discover(&context).expect("discovery");

        assert_eq!(drafts.len(), 1);
        assert_eq!(
            drafts[0].health,
            crate::domain::health::HealthStatus::ConfigOnly
        );
        assert_eq!(drafts[0].configuration.resources.len(), 2);
        assert!(drafts[0]
            .configuration
            .resources
            .iter()
            .any(|resource| resource.content_hash.is_some()));
        let skills = drafts[0]
            .configuration
            .resources
            .iter()
            .find(|resource| resource.logical_key == "skills")
            .expect("skills summary");
        assert_eq!(skills.entry_count, Some(2));
        assert!(!skills.scan_truncated);
    }

    #[test]
    fn sensitive_names_are_flagged_without_reading_content() {
        assert!(is_sensitive_path(Path::new("auth.json")));
        assert!(is_sensitive_path(Path::new(".env.local")));
        assert!(!is_sensitive_path(Path::new("settings.json")));
    }

    #[test]
    fn runtime_detection_reports_path_source() {
        let fixture = tempfile::tempdir().expect("fixture");
        let bin = fixture.path().join("bin");
        fs::create_dir_all(&bin).expect("bin");
        let executable = create_test_executable(&bin, "sample");
        #[cfg(windows)]
        fs::write(bin.join("sample"), "#!/bin/sh\necho wrong-unix-launcher\n")
            .expect("extensionless npm shim");
        let adapter = test_adapter();
        let context =
            DiscoveryContext::for_test(fixture.path().to_path_buf(), vec![bin], HashMap::new());

        let runtime = adapter.detect_runtime(&context).expect("runtime");

        assert!(runtime.installed);
        assert_eq!(
            runtime.executable_path.as_deref(),
            Some(executable.as_path())
        );
        assert_eq!(runtime.command_name, "sample");
        assert_eq!(runtime.resolution_source, RuntimeResolutionSource::Path);
        assert_eq!(runtime.version.as_deref(), Some("sample 1.0.0"));
        assert_eq!(runtime.version_probe_status, VersionProbeStatus::Detected);
        assert_eq!(runtime.version_probe_error, None);
    }

    #[cfg(windows)]
    #[test]
    fn windows_candidates_prefer_native_and_cmd_launchers_over_extensionless_shims() {
        let candidates = executable_candidates(Path::new("C:/npm"), "claude");
        assert_eq!(
            candidates,
            vec![
                PathBuf::from("C:/npm/claude.exe"),
                PathBuf::from("C:/npm/claude.com"),
                PathBuf::from("C:/npm/claude.cmd"),
                PathBuf::from("C:/npm/claude.bat"),
            ]
        );
        assert!(!candidates.contains(&PathBuf::from("C:/npm/claude")));
    }

    #[test]
    fn runtime_detection_distinguishes_npm_and_not_found() {
        let fixture = tempfile::tempdir().expect("fixture");
        let npm = fixture.path().join("npm");
        fs::create_dir_all(&npm).expect("npm");
        create_test_executable(&npm, "sample");
        let adapter = test_adapter();
        let context =
            DiscoveryContext::for_test(fixture.path().to_path_buf(), vec![npm], HashMap::new());
        let missing_context = DiscoveryContext::for_test(
            fixture.path().join("other-home"),
            Vec::new(),
            HashMap::new(),
        );

        assert_eq!(
            adapter
                .detect_runtime(&context)
                .expect("npm runtime")
                .distribution,
            RuntimeDistribution::Npm
        );
        let missing = adapter
            .detect_runtime(&missing_context)
            .expect("missing runtime");
        assert!(!missing.installed);
        assert_eq!(missing.resolution_source, RuntimeResolutionSource::NotFound);
    }

    #[test]
    fn runtime_detection_uses_default_locations_outside_path() {
        let fixture = tempfile::tempdir().expect("fixture");
        let app_data = fixture.path().join("roaming");
        let npm = app_data.join("npm");
        fs::create_dir_all(&npm).expect("npm");
        create_test_executable(&npm, "sample");
        let mut environment = HashMap::new();
        environment.insert("APPDATA".to_owned(), app_data.into_os_string());
        let context =
            DiscoveryContext::for_test(fixture.path().to_path_buf(), Vec::new(), environment);

        let runtime = test_adapter()
            .detect_runtime(&context)
            .expect("default runtime");

        assert!(runtime.installed);
        assert_eq!(
            runtime.resolution_source,
            RuntimeResolutionSource::DefaultPath
        );
        assert_eq!(runtime.distribution, RuntimeDistribution::Npm);
    }

    #[test]
    fn runtime_detection_reports_default_path_source() {
        let fixture = tempfile::tempdir().expect("fixture");
        let bin = fixture.path().join(".local/bin");
        fs::create_dir_all(&bin).expect("default bin");
        create_test_executable(&bin, "sample");
        let context =
            DiscoveryContext::for_test(fixture.path().to_path_buf(), Vec::new(), HashMap::new());

        let runtime = test_adapter()
            .detect_runtime(&context)
            .expect("default runtime");

        assert!(runtime.installed);
        assert_eq!(
            runtime.resolution_source,
            RuntimeResolutionSource::DefaultPath
        );
    }

    fn test_adapter() -> ConfiguredAdapter {
        ConfiguredAdapter::new(
            "sample",
            "Sample",
            "sample",
            ".sample",
            &[],
            &["sample"],
            &[],
            &["config.toml"],
            Vec::new(),
        )
    }

    fn create_test_executable(directory: &Path, name: &str) -> PathBuf {
        #[cfg(windows)]
        let path = directory.join(format!("{name}.cmd"));
        #[cfg(not(windows))]
        let path = directory.join(name);

        #[cfg(windows)]
        fs::write(&path, "@echo sample 1.0.0\r\n").expect("executable");
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::write(&path, "#!/bin/sh\necho sample 1.0.0\n").expect("executable");
            let mut permissions = fs::metadata(&path).expect("metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).expect("permissions");
        }

        canonical_or_absolute(&path).expect("canonical executable")
    }
}
