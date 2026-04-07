use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream as AsyncTcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use anyhow::{Result, Context};
use tracing::{info, error, debug};

const BACKEND_ADDRESS: &str = "localhost:50505";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Backend client for communicating with the wazuh-agent-status server
pub struct BackendClient;

impl BackendClient {
    /// Send a command and receive a single response (synchronous for simple commands)
    pub fn send_command_and_receive(command: &str) -> Result<String> {
        debug!("Sending command: {}", command);
        
        let mut stream = TcpStream::connect_timeout(&BACKEND_ADDRESS.parse()?, CONNECT_TIMEOUT)
            .with_context(|| format!("Failed to connect to backend at {}", BACKEND_ADDRESS))?;
        
        stream.set_read_timeout(Some(READ_TIMEOUT))?;
        
        // Send command
        writeln!(stream, "{}", command)
            .with_context(|| "Failed to send command")?;
        
        // Read response
        let reader = io::BufReader::new(&stream);
        let mut lines = reader.lines();
        
        if let Some(line) = lines.next() {
            let response = line?;
            let trimmed = response.trim();
            
            if trimmed.starts_with("ERROR:") {
                return Err(anyhow::anyhow!("Server error: {}", trimmed));
            }
            
            debug!("Received response: {}", trimmed);
            return Ok(trimmed.to_string());
        }
        
        Err(anyhow::anyhow!("No response from server"))
    }
    
    /// Subscribe to status updates (async for streaming)
    pub async fn subscribe_status<F>(mut callback: F) -> Result<()>
    where
        F: FnMut(String, String) + Send + 'static,
    {
        loop {
            match Self::connect_and_subscribe(&mut callback).await {
                Ok(_) => {
                    info!("Status subscription ended normally");
                }
                Err(e) => {
                    error!("Status subscription error: {}. Reconnecting in 5s...", e);
                    callback("Unknown".to_string(), "Unknown".to_string());
                }
            }
            
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
    
    async fn connect_and_subscribe<F>(callback: &mut F) -> Result<()>
    where
        F: FnMut(String, String) + Send,
    {
        info!("Connecting to backend for status subscription...");
        
        let stream = AsyncTcpStream::connect(BACKEND_ADDRESS).await
            .with_context(|| format!("Failed to connect to backend at {}", BACKEND_ADDRESS))?;
        
        let (reader, mut writer) = stream.into_split();
        
        // Send subscription command
        writer.write_all(b"subscribe-status\n").await
            .with_context(|| "Failed to send subscribe command")?;
        
        let reader = AsyncBufReader::new(reader);
        let mut lines = reader.lines();
        
        // Read status updates
        while let Some(line) = lines.next_line().await? {
            let trimmed = line.trim();
            debug!("Status stream received: {}", trimmed);
            
            if let Some((status, connection)) = Self::parse_status_update(trimmed) {
                callback(status, connection);
            } else if trimmed.starts_with("ERROR:") {
                error!("Error from status stream: {}", trimmed);
                return Err(anyhow::anyhow!("Server error: {}", trimmed));
            }
        }
        
        Ok(())
    }
    
    /// Parse a STATUS_UPDATE message
    fn parse_status_update(response: &str) -> Option<(String, String)> {
        if !response.starts_with("STATUS_UPDATE:") {
            return None;
        }
        
        let parts: Vec<&str> = response.splitn(2, ": ").collect();
        if parts.len() < 2 {
            return None;
        }
        
        let data: Vec<&str> = parts[1].splitn(2, ", ").collect();
        if data.len() == 2 {
            return Some((data[0].to_string(), data[1].to_string()));
        }
        
        None
    }
    
    /// Start an update stream and process progress updates
    pub async fn start_update_stream<F>(is_prerelease: bool, mut progress_callback: F) -> Result<()>
    where
        F: FnMut(String) + Send + 'static,
    {
        let command = if is_prerelease { "update-prerelease" } else { "update" };
        let init_command = if is_prerelease { "initiate-prerelease-update-stream" } else { "initiate-update-stream" };
        
        info!("Starting {} stream...", if is_prerelease { "prerelease update" } else { "update" });
        
        // First connection to trigger the update
        let stream = AsyncTcpStream::connect(BACKEND_ADDRESS).await
            .with_context(|| format!("Failed to connect to backend at {}", BACKEND_ADDRESS))?;
        
        let (reader, mut writer) = stream.into_split();
        
        // Send update command
        writer.write_all(format!("{}\n", command).as_bytes()).await
            .with_context(|| "Failed to send update command")?;
        
        let reader = AsyncBufReader::new(reader);
        let mut lines = reader.lines();
        
        // Read the stream response and pipe it
        let stream = AsyncTcpStream::connect(BACKEND_ADDRESS).await
            .with_context(|| "Failed to connect for update initiation")?;
        
        let (update_reader, mut update_writer) = stream.into_split();
        
        update_writer.write_all(format!("{}\n", init_command).as_bytes()).await
            .with_context(|| "Failed to send update initiation command")?;
        
        // Drop the writer to signal EOF
        drop(update_writer);
        
        let update_reader = AsyncBufReader::new(update_reader);
        let mut update_lines = update_reader.lines();
        
        // Process update progress
        while let Some(line) = update_lines.next_line().await? {
            let trimmed = line.trim();
            debug!("Update stream: {}", trimmed);
            
            if trimmed.starts_with("UPDATE_PROGRESS:") {
                let status = trimmed.strip_prefix("UPDATE_PROGRESS: ").unwrap_or(trimmed);
                progress_callback(status.to_string());
                
                if status == "Complete" || status == "Error" || status.starts_with("ERROR:") {
                    break;
                }
            }
        }
        
        Ok(())
    }
}
