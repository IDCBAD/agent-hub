mod adapters;
mod application;
mod commands;
mod desktop;
mod domain;
mod error;
mod infrastructure;

use std::{
    io,
    sync::{atomic::AtomicBool, Arc},
};

use adapters::AdapterRegistry;
use application::ApplicationService;
use commands::AppState;
use infrastructure::database::Database;
use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if !args.iter().any(|argument| argument == "--background") {
                if let Err(error) = desktop::show_main_window(app) {
                    eprintln!("failed to restore Agent Hub window: {error}");
                }
            }
        }))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--background")
                .app_name("Agent Hub")
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            let database = Database::initialize(data_directory.join("agent-hub.db"))
                .map_err(|error| io::Error::other(error.to_string()))?;
            let service = ApplicationService::new(database, AdapterRegistry::standard())
                .map_err(|error| io::Error::other(error.to_string()))?;
            let settings = service
                .get_app_settings()
                .map_err(|error| io::Error::other(error.to_string()))?;
            app.manage(AppState {
                service: service.clone(),
                keep_running_in_tray: Arc::new(AtomicBool::new(settings.keep_running_in_tray)),
            });
            desktop::install(app.handle())?;
            let background = std::env::args().any(|argument| argument == "--background");
            if !background {
                desktop::show_main_window(app.handle())?;
            }
            if settings.scan_on_launch {
                desktop::schedule_startup_scan(app.handle().clone(), service);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::discover_agents,
            commands::list_agents,
            commands::get_agent_overview,
            commands::get_agent_resources,
            commands::list_resources,
            commands::get_discovery_evidence,
            commands::add_manual_location,
            commands::rescan_agent,
            commands::remove_manual_agent,
            commands::open_agent_directory,
            commands::open_resource,
            commands::list_quick_locations,
            commands::create_quick_location,
            commands::update_quick_location,
            commands::reorder_quick_locations,
            commands::remove_quick_location,
            commands::open_quick_location,
            commands::get_app_settings,
            commands::update_app_settings,
            commands::get_app_info,
            commands::open_app_data_directory,
            commands::rebuild_agent_index,
            commands::open_project_page,
            commands::open_releases_page,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Agent Hub");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { api, code, .. } = event {
            if code.is_none()
                && app
                    .state::<AppState>()
                    .keep_running_in_tray
                    .load(std::sync::atomic::Ordering::Relaxed)
            {
                api.prevent_exit();
            }
        }
    });
}
