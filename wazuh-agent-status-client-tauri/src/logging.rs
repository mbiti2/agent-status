use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::{info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

static LOGGING_INITIALIZED: OnceLock<()> = OnceLock::new();

/// Initialize the logging system with file rotation
pub fn init_logging() {
    LOGGING_INITIALIZED.get_or_init(|| {
        let log_path = get_log_file_path();
        let default_path = PathBuf::from("./logs");
        let log_dir = log_path.parent().unwrap_or(default_path.as_path());
        
        // Create rolling file appender (10MB max size, 3 backups, 30 days retention)
        let file_appender = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .max_log_files(30)
            .filename_prefix("wazuh-agent-status-client")
            .filename_suffix("log")
            .build(log_dir)
            .expect("Failed to create log file appender");
        
        let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
        
        // Initialize subscriber with both file and console output
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_writer(file_writer)
                    .with_ansi(false)
                    .with_target(false)
                    .with_level(true)
                    .with_thread_ids(false)
            )
            .with(
                fmt::layer()
                    .with_writer(std::io::stdout)
                    .with_ansi(true)
            )
            .with(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info"))
            )
            .init();
        
        info!("Logging initialized. Log file: {:?}", log_path);
    });
}

/// Get the log file path based on the operating system
fn get_log_file_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    
    #[cfg(target_os = "windows")]
    {
        let app_data = std::env::var("APPDATA").unwrap_or_else(|_| {
            format!("{}\\AppData\\Roaming", home)
        });
        PathBuf::from(format!("{}\\wazuh\\logs\\wazuh-agent-status-client.log", app_data))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from(format!("{}/.wazuh/wazuh-agent-status-client.log", home))
    }
}

/// Ensure the log directory exists
pub fn ensure_log_directory() -> anyhow::Result<()> {
    let log_path = get_log_file_path();
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
