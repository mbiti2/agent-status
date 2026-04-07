use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent},
    image::Image,
    AppHandle, Wry,
};
use tracing::{info, error, debug};
use tokio::sync::Mutex as AsyncMutex;

use crate::version_checker::{
    VersionState, VersionStatus, get_version_title, get_update_title, 
    is_update_enabled, should_show_prerelease_update
};
use crate::backend_client::BackendClient;

/// Shared state for tray menu items
pub struct TrayState {
    pub status_item: Arc<Mutex<MenuItem<Wry>>>,
    pub connection_item: Arc<Mutex<MenuItem<Wry>>>,
    pub update_item: Arc<Mutex<MenuItem<Wry>>>,
    pub version_item: Arc<Mutex<MenuItem<Wry>>>,
    pub prerelease_item: Arc<Mutex<Option<MenuItem<Wry>>>>,
    pub prerelease_update_item: Arc<Mutex<Option<MenuItem<Wry>>>>,
    pub version_state: VersionState,
    pub enabled_icon: Arc<Vec<u8>>,
    pub disabled_icon: Arc<Vec<u8>>,
    pub update_in_progress: Arc<AsyncMutex<bool>>,
}

impl TrayState {
    pub fn new(
        status_item: MenuItem<Wry>,
        connection_item: MenuItem<Wry>,
        update_item: MenuItem<Wry>,
        version_item: MenuItem<Wry>,
        version_state: VersionState,
        enabled_icon: Vec<u8>,
        disabled_icon: Vec<u8>,
    ) -> Self {
        Self {
            status_item: Arc::new(Mutex::new(status_item)),
            connection_item: Arc::new(Mutex::new(connection_item)),
            update_item: Arc::new(Mutex::new(update_item)),
            version_item: Arc::new(Mutex::new(version_item)),
            prerelease_item: Arc::new(Mutex::new(None)),
            prerelease_update_item: Arc::new(Mutex::new(None)),
            version_state,
            enabled_icon: Arc::new(enabled_icon),
            disabled_icon: Arc::new(disabled_icon),
            update_in_progress: Arc::new(AsyncMutex::new(false)),
        }
    }
    
    /// Update status items with new status values
    pub fn update_status(&self, status: &str, connection: &str) {
        if let Ok(item) = self.status_item.lock() {
            let is_active = status == "Active";
            let title = format!("Agent: {}", if is_active { "Active" } else { "Inactive" });
            let _ = item.set_text(&title);
        }
        
        if let Ok(item) = self.connection_item.lock() {
            let is_connected = connection == "Connected";
            let title = format!("Connection: {}", if is_connected { "Connected" } else { "Disconnected" });
            let _ = item.set_text(&title);
        }
    }
    
    /// Update version-related menu items
    pub fn update_version_display(&self, status: VersionStatus) {
        let version_title = get_version_title(&status);
        let update_title = get_update_title(&status);
        let update_enabled = is_update_enabled(&status);
        
        if let Ok(item) = self.version_item.lock() {
            let _ = item.set_text(&version_title);
        }
        
        if let Ok(item) = self.update_item.lock() {
            let _ = item.set_text(&update_title);
            let _ = item.set_enabled(update_enabled);
        }
        
        // Handle prerelease notification
        if let Some(prerelease_version) = should_show_prerelease_update(&status) {
            self.show_prerelease_notification(&prerelease_version);
        }
        
        // Hide prerelease items if current version is a prerelease and up to date
        if let VersionStatus::UpToDate(ref v) = status {
            if v.contains("rc") {
                self.hide_prerelease_items();
            }
        }
    }
    
    /// Show prerelease notification menu items
    fn show_prerelease_notification(&self, prerelease_version: &str) {
        // Only show once per session
        if self.version_state.is_prerelease_shown() {
            return;
        }
        
        info!("Showing prerelease notification: {}", prerelease_version);
        
        // Note: Dynamic menu item addition is limited in Tauri v2
        // The prerelease info is conveyed through the version text
    }
    
    /// Hide prerelease notification items
    fn hide_prerelease_items(&self) {
        info!("Hiding prerelease notification items");
        // Implementation would hide/remove menu items if dynamically added
        // For now, we just clear the prerelease shown flag
        if let Ok(mut guard) = self.prerelease_item.lock() {
            *guard = None;
        }
        if let Ok(mut guard) = self.prerelease_update_item.lock() {
            *guard = None;
        }
    }
}

impl Clone for TrayState {
    fn clone(&self) -> Self {
        Self {
            status_item: Arc::clone(&self.status_item),
            connection_item: Arc::clone(&self.connection_item),
            update_item: Arc::clone(&self.update_item),
            version_item: Arc::clone(&self.version_item),
            prerelease_item: Arc::clone(&self.prerelease_item),
            prerelease_update_item: Arc::clone(&self.prerelease_update_item),
            version_state: self.version_state.clone(),
            enabled_icon: Arc::clone(&self.enabled_icon),
            disabled_icon: Arc::clone(&self.disabled_icon),
            update_in_progress: Arc::clone(&self.update_in_progress),
        }
    }
}

/// Build the system tray
pub fn build_tray(
    app: &AppHandle,
    enabled_icon: Vec<u8>,
    disabled_icon: Vec<u8>,
    main_icon: Vec<u8>,
) -> anyhow::Result<TrayState> {
    // Create menu items
    let status_item = MenuItem::with_id(app, "status", "Agent: Unknown", false, None::<&str>)?;
    let connection_item = MenuItem::with_id(app, "connection", "Connection: Unknown", false, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let update_item = MenuItem::with_id(app, "update", "---", false, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let version_item = MenuItem::with_id(app, "version", "v---", false, None::<&str>)?;
    
    // Create menu
    let menu = Menu::with_items(app, &[
        &status_item,
        &connection_item,
        &separator1,
        &update_item,
        &separator2,
        &version_item,
    ])?;
    
    // Create tray state
    let version_state = VersionState::new();
    let tray_state = TrayState::new(
        status_item,
        connection_item,
        update_item,
        version_item,
        version_state,
        enabled_icon,
        disabled_icon,
    );
    
    // Build tray icon
    let main_icon_clone = main_icon.clone();
    let tray_icon = TrayIconBuilder::with_id("main-tray")
        .tooltip("Wazuh Agent Status")
        .icon(Image::from_bytes(&main_icon)?)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_tray_icon_event({
            let tray_state = tray_state.clone();
            move |tray, event| {
                match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        debug!("Tray left click detected");
                        // Could show/hide a window here if needed
                    }
                    _ => {}
                }
            }
        })
        .on_menu_event({
            let tray_state = tray_state.clone();
            move |app, event| {
                let id = event.id.as_ref();
                debug!("Menu item clicked: {}", id);
                
                match id {
                    "update" => {
                        let tray_state = tray_state.clone();
                        tokio::spawn(async move {
                            handle_update_click(tray_state, false).await;
                        });
                    }
                    "prerelease_update" => {
                        let tray_state = tray_state.clone();
                        tokio::spawn(async move {
                            handle_update_click(tray_state, true).await;
                        });
                    }
                    _ => {}
                }
            }
        })
        .build(app)?;
    
    info!("System tray built successfully");
    Ok(tray_state)
}

/// Handle update button click
async fn handle_update_click(tray_state: TrayState, is_prerelease: bool) {
    // Check if update already in progress
    let mut in_progress = tray_state.update_in_progress.lock().await;
    if *in_progress {
        info!("Update already in progress, skipping");
        return;
    }
    *in_progress = true;
    drop(in_progress);
    
    info!("Starting {} update...", if is_prerelease { "prerelease" } else { "stable" });
    
    // Update menu item to show progress
    let update_item = Arc::clone(&tray_state.update_item);
    if let Ok(item) = update_item.lock() {
        let _ = item.set_text("Starting Update...");
        let _ = item.set_enabled(false);
    }
    
    // Start update stream
    let update_item = Arc::clone(&tray_state.update_item);
    let result = BackendClient::start_update_stream(is_prerelease, move |progress| {
        if let Ok(item) = update_item.lock() {
            let text = format!("Updating: {}", progress);
            let _ = item.set_text(&text);
        }
    }).await;
    
    if let Err(e) = result {
        error!("Update failed: {}", e);
        if let Ok(item) = tray_state.update_item.lock() {
            let _ = item.set_text("Update Failed");
            let _ = item.set_enabled(true);
        }
    } else {
        info!("Update completed successfully");
        // Version check will be triggered by the version monitor
    }
    
    // Release lock
    let mut in_progress = tray_state.update_in_progress.lock().await;
    *in_progress = false;
}
