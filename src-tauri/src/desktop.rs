use std::collections::HashMap;
use std::{thread, time::Duration};

use tauri::{
    menu::{Menu, MenuBuilder, MenuItem, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::{commands::AppState, infrastructure::webview_memory};

const TRAY_ID: &str = "agent-hub-tray";
const DATA_CHANGED_EVENT: &str = "agent-hub-data-changed";

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_tray_menu(app);
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Agent Hub")
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                if let Err(error) = show_main_window(tray.app_handle()) {
                    eprintln!("failed to show Agent Hub window: {error}");
                }
            }
        });
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    if let Ok(menu) = menu {
        builder = builder.menu(&menu);
    }
    builder.build(app)?;
    Ok(())
}

pub fn refresh_tray_menu(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    match build_tray_menu(app) {
        Ok(menu) => {
            if let Err(error) = tray.set_menu(Some(menu)) {
                eprintln!("failed to refresh Agent Hub tray menu: {error}");
            }
        }
        Err(error) => eprintln!("failed to build Agent Hub tray menu: {error}"),
    }
}

pub fn emit_data_changed(app: &AppHandle) {
    if let Err(error) = app.emit(DATA_CHANGED_EVENT, ()) {
        eprintln!("failed to notify Agent Hub window: {error}");
    }
}

pub fn schedule_startup_scan(app: AppHandle, service: crate::application::ApplicationService) {
    tauri::async_runtime::spawn_blocking(move || {
        thread::sleep(Duration::from_secs(3));
        match service.discover_agents() {
            Ok(_) => {
                refresh_tray_menu(&app);
                emit_data_changed(&app);
            }
            Err(error) => eprintln!("startup Agent scan failed: {error}"),
        }
    });
}

pub fn show_main_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        let _ = window.unminimize();
        window.set_focus()?;
        return Ok(window);
    }

    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Agent Hub")
        .inner_size(1360.0, 860.0)
        .min_inner_size(760.0, 560.0)
        .resizable(true)
        .center()
        .build()?;
    webview_memory::install(window.clone());
    window.set_focus()?;
    Ok(window)
}

fn build_tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let service = app.state::<AppState>().service.clone();
    let agents = service.list_agents(None).unwrap_or_else(|error| {
        eprintln!("failed to read agents for tray menu: {error}");
        Vec::new()
    });
    let locations = service.list_quick_locations().unwrap_or_else(|error| {
        eprintln!("failed to read quick locations for tray menu: {error}");
        Vec::new()
    });

    let mut name_counts = HashMap::new();
    for agent in &agents {
        *name_counts
            .entry(agent.display_name.as_str())
            .or_insert(0_usize) += 1;
    }

    let mut agent_menu = SubmenuBuilder::with_id(app, "agent-directories", "Agent 目录");
    let available_agents = agents
        .iter()
        .filter(|agent| agent.configuration.exists)
        .collect::<Vec<_>>();
    if available_agents.is_empty() {
        let empty = MenuItem::with_id(app, "agent:none", "尚未发现可用目录", false, None::<&str>)?;
        agent_menu = agent_menu.item(&empty);
    } else {
        for agent in available_agents {
            let label = if name_counts
                .get(agent.display_name.as_str())
                .copied()
                .unwrap_or_default()
                > 1
            {
                format!("{} — {}", agent.display_name, agent.configuration.root_path)
            } else {
                agent.display_name.clone()
            };
            agent_menu = agent_menu.text(format!("agent:{}", agent.id), label);
        }
    }
    let agent_menu = agent_menu.build()?;

    let tray_locations = locations
        .iter()
        .filter(|location| location.show_in_tray)
        .collect::<Vec<_>>();
    let mut quick_menu = SubmenuBuilder::with_id(app, "quick-directories", "快捷目录");
    if tray_locations.is_empty() {
        let empty = MenuItem::with_id(app, "quick:none", "尚未绑定快捷目录", false, None::<&str>)?;
        quick_menu = quick_menu.item(&empty);
    } else {
        for location in tray_locations {
            quick_menu = quick_menu.text(format!("quick:{}", location.id), &location.name);
        }
    }
    let quick_menu = quick_menu.build()?;

    MenuBuilder::new(app)
        .item(&agent_menu)
        .item(&quick_menu)
        .separator()
        .text("open-main", "打开 Agent Hub")
        .text("scan-agents", "重新扫描 Agent")
        .separator()
        .text("quit", "退出")
        .build()
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    match id {
        "open-main" => {
            if let Err(error) = show_main_window(app) {
                eprintln!("failed to show Agent Hub window: {error}");
            }
        }
        "scan-agents" => {
            let app = app.clone();
            let service = app.state::<AppState>().service.clone();
            tauri::async_runtime::spawn_blocking(move || match service.discover_agents() {
                Ok(_) => {
                    refresh_tray_menu(&app);
                    emit_data_changed(&app);
                }
                Err(error) => eprintln!("tray Agent scan failed: {error}"),
            });
        }
        "quit" => app.exit(0),
        _ => {
            let service = app.state::<AppState>().service.clone();
            if let Some(agent_id) = id.strip_prefix("agent:") {
                if let Err(error) = service.open_agent_directory(agent_id) {
                    eprintln!("failed to open Agent directory from tray: {error}");
                }
            } else if let Some(location_id) = id.strip_prefix("quick:") {
                match service.open_quick_location(location_id) {
                    Ok(()) => emit_data_changed(app),
                    Err(error) => {
                        eprintln!("failed to open quick directory from tray: {error}")
                    }
                }
            }
        }
    }
}
