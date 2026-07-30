use tauri::State;

use crate::{
    application::ApplicationService,
    domain::{
        agent::{AgentFilter, AgentOverview, AgentSummary, ManualLocationRequest},
        discovery::{DiscoveryEvidence, DiscoveryResult},
        resource::{Resource, ResourceFilter},
    },
    error::AppError,
};

#[derive(Clone)]
pub struct AppState {
    pub service: ApplicationService,
}

#[tauri::command]
pub async fn discover_agents(state: State<'_, AppState>) -> Result<DiscoveryResult, AppError> {
    let service = state.service.clone();
    tauri::async_runtime::spawn_blocking(move || service.discover_agents())
        .await
        .map_err(|error| AppError::internal(format!("扫描任务意外终止：{error}")))?
}

#[tauri::command]
pub fn list_agents(
    filter: Option<AgentFilter>,
    state: State<'_, AppState>,
) -> Result<Vec<AgentSummary>, AppError> {
    state.service.list_agents(filter)
}

#[tauri::command]
pub fn get_agent_overview(
    agent_id: String,
    state: State<'_, AppState>,
) -> Result<AgentOverview, AppError> {
    state.service.get_agent_overview(&agent_id)
}

#[tauri::command]
pub fn get_agent_resources(
    agent_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Resource>, AppError> {
    state.service.get_agent_resources(&agent_id)
}

#[tauri::command]
pub fn list_resources(
    filter: Option<ResourceFilter>,
    state: State<'_, AppState>,
) -> Result<Vec<Resource>, AppError> {
    state.service.list_resources(filter)
}

#[tauri::command]
pub fn get_discovery_evidence(
    agent_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<DiscoveryEvidence>, AppError> {
    state.service.get_discovery_evidence(&agent_id)
}

#[tauri::command]
pub async fn add_manual_location(
    request: ManualLocationRequest,
    state: State<'_, AppState>,
) -> Result<AgentSummary, AppError> {
    let service = state.service.clone();
    tauri::async_runtime::spawn_blocking(move || service.add_manual_location(request))
        .await
        .map_err(|error| AppError::internal(format!("手动扫描任务意外终止：{error}")))?
}

#[tauri::command]
pub async fn rescan_agent(
    agent_id: String,
    state: State<'_, AppState>,
) -> Result<DiscoveryResult, AppError> {
    let service = state.service.clone();
    tauri::async_runtime::spawn_blocking(move || service.rescan_agent(&agent_id))
        .await
        .map_err(|error| AppError::internal(format!("重新扫描任务意外终止：{error}")))?
}

#[tauri::command]
pub async fn remove_manual_agent(
    agent_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let service = state.service.clone();
    tauri::async_runtime::spawn_blocking(move || service.remove_manual_agent(&agent_id))
        .await
        .map_err(|error| AppError::internal(format!("移除手动 Agent 任务意外终止：{error}")))?
}

#[tauri::command]
pub fn open_agent_directory(agent_id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.service.open_agent_directory(&agent_id)
}

#[tauri::command]
pub fn open_resource(resource_id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.service.open_resource(&resource_id)
}
