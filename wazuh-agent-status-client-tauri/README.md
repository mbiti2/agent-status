# Wazuh Agent Status Client (Tauri)

A Rust-based system tray application for monitoring Wazuh Agent status, built with Tauri v2.

## Overview

This application provides a system tray interface for the Wazuh Agent Status backend. It displays:
- Agent status (Active/Inactive)
- Connection status (Connected/Disconnected)
- Current version information
- Available updates (stable and prerelease)

## Architecture

The client communicates with the existing Go backend via TCP on `localhost:50505` and provides a native system tray UI.

```
┌─────────────────┐     TCP (50505)     ┌─────────────────┐
│  Tauri Client   │ ◄──────────────────► │  Go Backend     │
│   (Rust)        │   Status Updates     │  (localhost)    │
│  System Tray    │   Version Checks     │  Wazuh Agent    │
└─────────────────┘   Update Commands    └─────────────────┘
```

## Features

- **System Tray Integration**: Native tray icon with customizable menu
- **Real-time Status Updates**: Subscribes to backend status stream
- **Version Management**: Checks for updates (stable and prerelease versions)
- **Auto-update Support**: Triggers agent updates through the backend
- **Cross-platform**: Supports Windows, macOS, and Linux
- **Logging**: File-based logging with rotation

## Prerequisites

- **Rust** (1.70 or later)
- **Cargo** (comes with Rust)
- **Tauri CLI** (optional, for development)

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Building

### Development Build

```bash
cd wazuh-agent-status-client-tauri
cargo build
```

### Release Build

```bash
cd wazuh-agent-status-client-tauri
cargo build --release
```

### With Tauri CLI (for bundling)

If you have the Tauri CLI installed:

```bash
# Install Tauri CLI (once)
cargo install tauri-cli

# Build and bundle
cd wazuh-agent-status-client-tauri
cargo tauri build
```

## Running

### From Source

```bash
cd wazuh-agent-status-client-tauri
cargo run
```

### Check Version

```bash
cargo run -- --version
# or
cargo run -- -v
```

## Project Structure

```
wazuh-agent-status-client-tauri/
├── Cargo.toml              # Rust dependencies
├── tauri.conf.json         # Tauri configuration
├── src/
│   ├── main.rs             # Application entry point
│   ├── backend_client.rs   # TCP client for backend communication
│   ├── tray.rs             # System tray implementation
│   ├── version_checker.rs  # Version checking & update logic
│   └── logging.rs          # File logging with rotation
├── icons/                  # Tray icons
│   ├── wazuh-logo.png      # Main icon (Linux/macOS)
│   ├── wazuh-logo.ico      # Main icon (Windows)
│   ├── green-dot.png       # Status: active/connected
│   └── gray-dot.png        # Status: inactive/disconnected
└── README.md
```

## Backend Protocol

The client communicates with the Go backend using simple text commands over TCP:

### Commands

| Command | Description |
|---------|-------------|
| `subscribe-status` | Subscribe to real-time status updates |
| `get-version` | Request version information |
| `update` | Trigger stable version update |
| `update-prerelease` | Trigger prerelease version update |

### Status Updates

Status updates are received in the format:
```
STATUS_UPDATE: Active, Connected
```

### Version Response

```
VERSION_CHECK: Up to date, v4.8.0
VERSION_CHECK: Outdated, v4.7.2
VERSION_CHECK: Prerelease available: 4.9.0-rc1 (current: v4.8.0)
VERSION_CHECK: Outdated with Prerelease available: v4.7.2 (stable: 4.8.0, prerelease: 4.9.0-rc1)
```

## Configuration

### Logging

Logs are stored at:
- **Linux/macOS**: `~/.wazuh/wazuh-agent-status-client.log`
- **Windows**: `%APPDATA%\wazuh\logs\wazuh-agent-status-client.log`

Log files are rotated daily with a 30-day retention.

### Environment Variables

- `RUST_LOG`: Set log level (e.g., `RUST_LOG=debug`)
- `HOME` / `USERPROFILE`: Used to determine log directory
- `APPDATA`: Windows-specific log directory override

## Dependencies

Key dependencies:

- **tauri** (v2.0): System tray, menu management, cross-platform windowing
- **tokio** (v1): Async runtime for TCP communication
- **tracing** + **tracing-appender**: Structured logging with file rotation
- **serde**: JSON serialization
- **regex**: Version string parsing

See `Cargo.toml` for complete dependency list.

## Development

### Adding Menu Items

The tray menu is defined in `src/tray.rs`:

```rust
let menu = Menu::with_items(app, &[
    &status_item,
    &connection_item,
    &separator,
    &update_item,
    &version_item,
])?;
```

### Handling Menu Clicks

Menu events are handled in the `on_menu_event` closure:

```rust
.on_menu_event(move |app, event| {
    match event.id.as_ref() {
        "update" => { /* handle update */ }
        "quit" => { app.exit(0); }
        _ => {}
    }
})
```

### Adding New Backend Commands

Extend `src/backend_client.rs`:

```rust
pub fn send_custom_command() -> Result<String> {
    BackendClient::send_command_and_receive("custom-command")
}
```

## Platform-Specific Notes

### Windows

- Uses `.ico` format for tray icon
- Logs to `%APPDATA%\wazuh\logs\`

### macOS

- Uses `.png` format for tray icon
- Logs to `~/.wazuh/`

### Linux

- Uses `.png` format for tray icon
- Logs to `~/.wazuh/`
- Requires system tray support in desktop environment

## Troubleshooting

### Backend Connection Failed

Ensure the Go backend is running:
```bash
# Check if backend is listening on port 50505
lsof -i :50505
netstat -tlnp | grep 50505
```

### Tray Icon Not Visible

- Linux: Ensure your desktop environment supports system trays
- macOS: Look in the menu bar (top-right)
- Windows: Look in the system tray (bottom-right)

### Build Errors

Update Rust toolchain:
```bash
rustup update
```

## License

MIT License - See project root for details.

## Contributing

This is part of the ADORSYS-GIS wazuh-agent-status project. Please follow the existing code style and include appropriate tests for new features.
