#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use tracing::{info, error, debug};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent},
    AppHandle, Wry,
};

mod backend_client;
mod logging;
mod tray;
mod version_checker;

use backend_client::BackendClient;
use logging::{init_logging, ensure_log_directory};
use tray::{build_tray, TrayState};
use version_checker::{monitor_version, VersionState};

// Include embedded icon files
const MAIN_ICON_PNG: &[u8] = include_bytes!("../icons/wazuh-logo.png");
const MAIN_ICON_ICO: &[u8] = include_bytes!("../icons/wazuh-logo.ico");
const ENABLED_ICON: &[u8] = include_bytes!("../icons/green-dot.png");
const DISABLED_ICON: &[u8] = include_bytes!("../icons/gray-dot.png");

/// Application version - set at build time via environment variable
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    // Handle command line arguments before initializing
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("{}", VERSION);
        return;
    }
    
    // Ensure log directory exists
    if let Err(e) = ensure_log_directory() {
        eprintln!("Failed to create log directory: {}", e);
    }
    
    // Initialize logging
    init_logging();
    
    info!("Starting Wazuh Agent Status Client (version: {})...", VERSION);
    
    // Build and run the Tauri application
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            setup_app(handle)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app(app: &AppHandle) -> anyhow::Result<()> {
    info!("Setting up application...");
    
    // Determine which icon to use based on platform
    let main_icon = if cfg!(target_os = "windows") {
        MAIN_ICON_ICO.to_vec()
    } else {
        MAIN_ICON_PNG.to_vec()
    };
    
    let enabled_icon = ENABLED_ICON.to_vec();
    let disabled_icon = DISABLED_ICON.to_vec();
    
    // Build the system tray
    let tray_state = build_tray(
        app,
        enabled_icon,
        disabled_icon,
        main_icon,
    )?;
    
    // Start background monitoring tasks
    let tray_state_clone = tray_state.clone();
    tokio::spawn(async move {
        monitor_status_stream(tray_state_clone).await;
    });
    
    let tray_state_clone = tray_state.clone();
    tokio::spawn(async move {
        let version_state = tray_state_clone.version_state.clone();
        monitor_version(version_state, move |status| {
            tray_state_clone.update_version_display(status);
        }).await;
    });
    
    info!("Application setup complete");
    Ok(())
}

/// Monitor the status stream from the backend
async fn monitor_status_stream(tray_state: TrayState) {
    info!("Starting status stream monitor...");
    
    BackendClient::subscribe_status(move |status, connection| {
        debug!("Status update received: {} - {}", status, connection);
        tray_state.update_status(&status, &connection);
    }).await;
}
