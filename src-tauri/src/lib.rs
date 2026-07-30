mod adapters;
mod application;
mod commands;
mod desktop;
mod domain;
mod error;
mod infrastructure;

use std::io;

use adapters::AdapterRegistry;
use application::ApplicationService;
use commands::AppState;
use infrastructure::{database::Database, webview_memory};
use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Err(error) = desktop::show_main_window(app) {
                eprintln!("failed to restore Agent Hub window: {error}");
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            let database = Database::initialize(data_directory.join("agent-hub.db"))
                .map_err(|error| io::Error::other(error.to_string()))?;
            let service = ApplicationService::new(database, AdapterRegistry::standard())
                .map_err(|error| io::Error::other(error.to_string()))?;
            app.manage(AppState { service });
            desktop::install(app.handle())?;
            if let Some(main_window) = app.get_webview_window("main") {
                webview_memory::install(main_window);
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
        ])
        .build(tauri::generate_context!())
        .expect("error while building Agent Hub");

    app.run(|_app, event| {
        if let RunEvent::ExitRequested { api, code, .. } = event {
            if code.is_none() {
                api.prevent_exit();
            }
        }
    });
}
