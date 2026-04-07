# Architecture Overview

Brief architectural documentation for the Wazuh Agent Status project.

---

## System Overview

Wazuh Agent Status is a cross-platform system tray application for monitoring Wazuh agents. It uses a **split-process architecture** for security:

```
┌──────────────────┐         TCP:50505         ┌──────────────────┐
│  Frontend Client │◄─────────────────────────►│  Backend Service │
│  (User Session)  │      Push-based updates   │  (Root/Admin)    │
│  • System Tray   │                           │  • TCP Server    │
│  • UI Actions    │                           │  • Agent Control │
└──────────────────┘                           └──────────────────┘
```

**Why split-process?**
- System tray requires a user GUI session
- Controlling system services requires root/admin privileges
- Separation prevents privilege escalation attacks

---

## Components

### Frontend (`wazuh-agent-status-client`)

| Aspect | Details |
|--------|---------|
| **Purpose** | System tray GUI for user interaction |
| **Runs As** | Logged-in user |
| **Key Libraries** | `fyne.io/systray` (GUI), `lumberjack` (logging) |
| **Main File** | `main.go` - Systray init, TCP client, status monitoring |

**Key Functions:**
- `onReady()` - Initialize tray, menu items, icons
- `monitorStatusStream()` - Subscribe to real-time status updates
- `startUpdateStream()` - Trigger and monitor agent updates
- `handleVersionCheck()` - Check and display version status

### Backend (`wazuh-agent-status`)

| Aspect | Details |
|--------|---------|
| **Purpose** | TCP server for agent control and status queries |
| **Runs As** | root/administrator (service) |
| **Key Libraries** | `kardianos/service` (Windows service), `lumberjack` |
| **Main File** | `main.go` - TCP server, command handlers |

**Key Functions:**
- `handleConnection()` - Parse TCP commands, dispatch handlers
- `checkServiceStatus()` - Platform-specific agent status
- `updateAgent()` - Execute agent updates
- `monitorAgentStatus()` - Poll agent status every 5s

---

## TCP Protocol

**Port:** 50505 (localhost)

| Command | Request | Response | Purpose |
|---------|---------|----------|---------|
| `subscribe-status` | `subscribe-status\n` | `STATUS_UPDATE: <status>, <connection>` | Subscribe to real-time updates |
| `get-version` | `get-version\n` | `VERSION_CHECK: <info>` | Get version status |
| `update` | `update\n` | `UPDATE_PROGRESS: <status>` | Trigger stable update |
| `update-prerelease` | `update-prerelease\n` | `UPDATE_PROGRESS: <status>` | Trigger prerelease update |

---

## Platform-Specific Code

Build tags control platform compilation:

| File | Build Tag | Purpose |
|------|-----------|---------|
| `linux.go` | `//go:build linux` | wazuh-control integration |
| `darwin.go` | `//go:build darwin` | wazuh-control integration |
| `windows.go` | `//go:build windows` | Service Manager/PowerShell |

---

## Key Paths

### Linux
- **Backend Binary:** `/usr/local/bin/wazuh-agent-status`
- **Frontend Binary:** `/usr/local/bin/wazuh-agent-status-client`
- **Backend Log:** `/var/log/wazuh-agent-status.log`
- **Frontend Log:** `~/.wazuh/wazuh-agent-status-client.log`
- **Wazuh:** `/var/ossec/`

### macOS
- **Backend Binary:** `/usr/local/bin/wazuh-agent-status`
- **Frontend Binary:** `/usr/local/bin/wazuh-agent-status-client`
- **Backend Log:** `/var/log/wazuh-agent-status.log`
- **Frontend Log:** `~/.wazuh/wazuh-agent-status-client.log`
- **Wazuh:** `/Library/Ossec/`

### Windows
- **Backend Binary:** `C:\Program Files\wazuh-agent-status\wazuh-agent-status.exe`
- **Frontend Binary:** `C:\Program Files\wazuh-agent-status\wazuh-agent-status-client.exe`
- **Backend Log:** `C:\ProgramData\wazuh\logs\wazuh-agent-status.log`
- **Frontend Log:** `%APPDATA%\wazuh\logs\wazuh-agent-status-client.log`
- **Wazuh:** `C:\Program Files (x86)\ossec-agent\`

---

## Architecture Decisions

1. **Go with CGO**: Cross-platform compilation, native systray via CGO
2. **TCP over Unix Sockets**: Works identically on Linux/macOS/Windows
3. **Push-based Updates**: Frontend subscribes; backend pushes status changes
4. **Split Binaries**: Clear privilege separation (user GUI + root service)
5. **Build Tags**: Platform-specific code isolation (linux.go, darwin.go, windows.go)

---

## Data Flow

**Startup:**
1. Backend starts → Opens TCP listener on 50505
2. Frontend starts → Connects to backend
3. Frontend sends `subscribe-status` → Receives initial state
4. Backend pushes `STATUS_UPDATE` whenever state changes

**Status Check (push-based):**
1. Backend polls Wazuh agent every 5s (via `wazuh-control`)
2. On state change, backend pushes update to all subscribers
3. Frontend updates UI (icon + text)

**User Action (Update):**
1. User clicks "Update"
2. Frontend sends `update`
3. Backend executes update and streams progress
4. Frontend displays progress in menu

---

## Dependencies

| Dependency | Purpose | Used By |
|------------|---------|---------|
| `fyne.io/systray` | Cross-platform system tray | Frontend |
| `github.com/kardianos/service` | Windows service management | Backend |
| `gopkg.in/natefinch/lumberjack.v2` | Log rotation | Both |

---

## See Also

- [Setup Guide](setup.md) - Build and run instructions
- [API Documentation](api.md) - Detailed TCP protocol
- [Developer Guide](dev.md) - Developer workflow
