use tauri::{AppHandle, State};

use crate::{
    application::ApplicationService,
    desktop,
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
};

#[derive(Clone)]
pub struct AppState {
    pub service: ApplicationService,
}

#[tauri::command]
pub async fn discover_agents(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<DiscoveryResult, AppError> {
    let service = state.service.clone();
    let result = tauri::async_runtime::spawn_blocking(move || service.discover_agents())
        .await
        .map_err(|error| AppError::internal(format!("扫描任务意外终止：{error}")))?;
    if result.is_ok() {
        desktop::refresh_tray_menu(&app);
    }
    result
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
    app: AppHandle,
) -> Result<AgentSummary, AppError> {
    let service = state.service.clone();
    let result = tauri::async_runtime::spawn_blocking(move || service.add_manual_location(request))
        .await
        .map_err(|error| AppError::internal(format!("手动扫描任务意外终止：{error}")))?;
    if result.is_ok() {
        desktop::refresh_tray_menu(&app);
    }
    result
}

#[tauri::command]
pub async fn rescan_agent(
    agent_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<DiscoveryResult, AppError> {
    let service = state.service.clone();
    let result = tauri::async_runtime::spawn_blocking(move || service.rescan_agent(&agent_id))
        .await
        .map_err(|error| AppError::internal(format!("重新扫描任务意外终止：{error}")))?;
    if result.is_ok() {
        desktop::refresh_tray_menu(&app);
    }
    result
}

#[tauri::command]
pub async fn remove_manual_agent(
    agent_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let service = state.service.clone();
    let result =
        tauri::async_runtime::spawn_blocking(move || service.remove_manual_agent(&agent_id))
            .await
            .map_err(|error| AppError::internal(format!("移除手动 Agent 任务意外终止：{error}")))?;
    if result.is_ok() {
        desktop::refresh_tray_menu(&app);
    }
    result
}

#[tauri::command]
pub fn open_agent_directory(agent_id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.service.open_agent_directory(&agent_id)
}

#[tauri::command]
pub fn open_resource(resource_id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.service.open_resource(&resource_id)
}

#[tauri::command]
pub fn list_quick_locations(state: State<'_, AppState>) -> Result<Vec<QuickLocation>, AppError> {
    state.service.list_quick_locations()
}

#[tauri::command]
pub fn create_quick_location(
    request: CreateQuickLocationRequest,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<QuickLocation, AppError> {
    let result = state.service.create_quick_location(request);
    if result.is_ok() {
        desktop::refresh_tray_menu(&app);
    }
    result
}

#[tauri::command]
pub fn update_quick_location(
    request: UpdateQuickLocationRequest,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<QuickLocation, AppError> {
    let result = state.service.update_quick_location(request);
    if result.is_ok() {
        desktop::refresh_tray_menu(&app);
    }
    result
}

#[tauri::command]
pub fn reorder_quick_locations(
    request: ReorderQuickLocationsRequest,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let result = state.service.reorder_quick_locations(request);
    if result.is_ok() {
        desktop::refresh_tray_menu(&app);
    }
    result
}

#[tauri::command]
pub fn remove_quick_location(
    location_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let result = state.service.remove_quick_location(&location_id);
    if result.is_ok() {
        desktop::refresh_tray_menu(&app);
    }
    result
}

#[tauri::command]
pub fn open_quick_location(
    location_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.service.open_quick_location(&location_id)
}
