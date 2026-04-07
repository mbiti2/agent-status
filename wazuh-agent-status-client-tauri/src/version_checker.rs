use std::sync::{Arc, Mutex};
use std::time::Duration;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::time;
use tracing::{info, error, debug, warn};
use crate::backend_client::BackendClient;

// Regex patterns for parsing version responses
static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^v?\d+\.\d+\.\d+(-rc[.\d]*)?$").unwrap()
});

static PRERELEASE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Prerelease available: ([^\s]+) \(current: (v[^\)]+|Prerelease: v[^\)]+)\)").unwrap()
});

static COMBINED_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Outdated with Prerelease available: (.+?) \(stable: ([^,]+), prerelease: ([^\)]+)\)").unwrap()
});

/// Version status types
#[derive(Debug, Clone, PartialEq)]
pub enum VersionStatus {
    UpToDate(String),
    Outdated(String),
    PrereleaseAvailable(String, String), // (current_version, prerelease_version)
    Combined(String, String, String),    // (current_version, stable_version, prerelease_version)
    Unknown,
}

/// Shared state for version information
pub struct VersionState {
    pub status: Arc<Mutex<VersionStatus>>,
    pub prerelease_shown: Arc<Mutex<bool>>,
}

impl VersionState {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(VersionStatus::Unknown)),
            prerelease_shown: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn set_status(&self, status: VersionStatus) {
        if let Ok(mut guard) = self.status.lock() {
            *guard = status;
        }
    }
    
    pub fn get_status(&self) -> VersionStatus {
        self.status.lock()
            .map(|guard| guard.clone())
            .unwrap_or(VersionStatus::Unknown)
    }
    
    pub fn is_prerelease_shown(&self) -> bool {
        self.prerelease_shown.lock()
            .map(|guard| *guard)
            .unwrap_or(false)
    }
    
    pub fn mark_prerelease_shown(&self) {
        if let Ok(mut guard) = self.prerelease_shown.lock() {
            *guard = true;
        }
    }
}

impl Clone for VersionState {
    fn clone(&self) -> Self {
        Self {
            status: Arc::clone(&self.status),
            prerelease_shown: Arc::clone(&self.prerelease_shown),
        }
    }
}

impl Default for VersionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the version monitoring loop
pub async fn monitor_version<F>(version_state: VersionState, mut update_callback: F)
where
    F: FnMut(VersionStatus) + Send + 'static,
{
    loop {
        // Inner loop: keep checking until we get a valid version
        loop {
            match check_version(&version_state).await {
                Some(status) => {
                    let is_valid = matches!(&status, 
                        VersionStatus::UpToDate(v) | VersionStatus::Outdated(v) 
                        if VERSION_REGEX.is_match(v) || v.starts_with("Prerelease: ")
                    );
                    
                    if is_valid || !matches!(status, VersionStatus::Unknown) {
                        info!("Version check successful: {:?}", status);
                        update_callback(status);
                        break;
                    }
                    
                    warn!("Version is in default/error state, retrying...");
                }
                None => {
                    warn!("Version check failed, retrying...");
                }
            }
            
            time::sleep(Duration::from_secs(5)).await;
        }
        
        // Switch to 8-hour polling interval once we have a valid version
        info!("Switching to 8-hour polling interval.");
        time::sleep(Duration::from_secs(8 * 60 * 60)).await;
    }
}

/// Check version by communicating with the backend
async fn check_version(version_state: &VersionState) -> Option<VersionStatus> {
    // Use blocking task for synchronous backend call
    let response = tokio::task::spawn_blocking(|| {
        BackendClient::send_command_and_receive("get-version")
    }).await.ok()?;
    
    let response = response.ok()?;
    let parsed = parse_version_response(&response, version_state);
    
    version_state.set_status(parsed.clone());
    Some(parsed)
}

/// Parse the version response from the backend
fn parse_version_response(response: &str, version_state: &VersionState) -> VersionStatus {
    let response = response.strip_prefix("VERSION_CHECK: ").unwrap_or(response);
    
    debug!("Parsing version response: {}", response);
    
    let is_outdated = response.contains("Outdated");
    let is_prerelease = response.contains("Prerelease available");
    let is_combined = response.contains("Outdated with Prerelease available");
    
    if is_combined {
        parse_combined_status(response)
    } else if is_outdated {
        parse_outdated_status(response)
    } else if is_prerelease {
        parse_prerelease_status(response, version_state)
    } else if response.contains("Up to date") {
        parse_up_to_date_status(response)
    } else {
        VersionStatus::Unknown
    }
}

fn parse_combined_status(response: &str) -> VersionStatus {
    if let Some(caps) = COMBINED_REGEX.captures(response) {
        let current = caps.get(1).map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let stable = caps.get(2).map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let prerelease = caps.get(3).map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        let current_version = if !current.starts_with('v') && !current.starts_with("Prerelease:") {
            format!("v{}", current)
        } else {
            current
        };
        
        info!("Combined status - Current: {}, Stable: {}, Prerelease: {}", 
              current_version, stable, prerelease);
        
        return VersionStatus::Combined(current_version, stable, prerelease);
    }
    
    warn!("Failed to parse combined status from: {}", response);
    VersionStatus::Outdated("Unknown".to_string())
}

fn parse_outdated_status(response: &str) -> VersionStatus {
    let parts: Vec<&str> = response.splitn(2, ", ").collect();
    let version = if parts.len() == 2 {
        parts[1].to_string()
    } else {
        warn!("Unexpected version response (Outdated): {}", response);
        "Unknown".to_string()
    };
    
    VersionStatus::Outdated(version)
}

fn parse_prerelease_status(response: &str, version_state: &VersionState) -> VersionStatus {
    if !response.starts_with("Prerelease available:") {
        return VersionStatus::Unknown;
    }
    
    if let Some(caps) = PRERELEASE_REGEX.captures(response) {
        let prerelease_version = caps.get(1).map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let current_version = caps.get(2).map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        info!("Prerelease available - Current: {}, Prerelease: {}", 
              current_version, prerelease_version);
        
        // Show prerelease notification once per session
        if !version_state.is_prerelease_shown() {
            version_state.mark_prerelease_shown();
        }
        
        return VersionStatus::PrereleaseAvailable(current_version, prerelease_version);
    }
    
    warn!("Failed to parse prerelease info from: {}", response);
    VersionStatus::Unknown
}

fn parse_up_to_date_status(response: &str) -> VersionStatus {
    let parts: Vec<&str> = response.splitn(2, ", ").collect();
    let version = if parts.len() == 2 {
        parts[1].to_string()
    } else {
        warn!("Unexpected version response (Up to date): {}", response);
        "Unknown".to_string()
    };
    
    VersionStatus::UpToDate(version)
}

/// Get a display title for the version status
pub fn get_version_title(status: &VersionStatus) -> String {
    match status {
        VersionStatus::UpToDate(v) => v.clone(),
        VersionStatus::Outdated(v) => v.clone(),
        VersionStatus::PrereleaseAvailable(v, _) => v.clone(),
        VersionStatus::Combined(v, _, _) => v.clone(),
        VersionStatus::Unknown => "v---".to_string(),
    }
}

/// Get a display title for the update button
pub fn get_update_title(status: &VersionStatus) -> String {
    match status {
        VersionStatus::UpToDate(_) => "Up to date".to_string(),
        VersionStatus::Outdated(_) => "Update Available".to_string(),
        VersionStatus::PrereleaseAvailable(_, _) => "Up to date".to_string(),
        VersionStatus::Combined(_, _, _) => "Update Available (Stable)".to_string(),
        VersionStatus::Unknown => "---".to_string(),
    }
}

/// Check if update button should be enabled
pub fn is_update_enabled(status: &VersionStatus) -> bool {
    matches!(status, 
        VersionStatus::Outdated(_) | VersionStatus::Combined(_, _, _)
    )
}

/// Check if prerelease update should be shown
pub fn should_show_prerelease_update(status: &VersionStatus) -> Option<String> {
    match status {
        VersionStatus::PrereleaseAvailable(_, prerelease) => Some(prerelease.clone()),
        VersionStatus::Combined(_, _, prerelease) => Some(prerelease.clone()),
        _ => None,
    }
}
