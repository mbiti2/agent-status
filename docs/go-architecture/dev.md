# Developer Onboarding

Quick guide to get started with Wazuh Agent Status development.

---

## Prerequisites

| Tool | Version | Linux | macOS | Windows |
|------|---------|-------|-------|---------|
| Go | 1.22+ | `sudo apt install golang-go` | `brew install go` | [Download](https://go.dev/dl/) |
| GCC | 11.x | `sudo apt install gcc` | Xcode CLT | MinGW-w64 |

**Linux only:** `sudo apt install libayatana-appindicator3-dev netcat`

Verify: `go version && gcc --version`

---

## Quick Start

```bash
# Clone
git clone https://github.com/ADORSYS-GIS/wazuh-agent-status.git
cd wazuh-agent-status

# Build backend
cd wazuh-agent-status && go mod tidy && go build -o ../dist/wazuh-agent-status . && cd ..

# Build frontend
cd wazuh-agent-status-client && go mod tidy && go build -o ../dist/wazuh-agent-status-client . && cd ..

# Run (2 terminals)
sudo ./dist/wazuh-agent-status      # Terminal 1: Backend (needs sudo)
./dist/wazuh-agent-status-client    # Terminal 2: Frontend
```

---

## Project Structure

```
wazuh-agent-status/
├── wazuh-agent-status/          # Backend (TCP server, root)
│   ├── main.go                  # TCP server, state manager, notifier
│   ├── helpers.go               # Version checking, file operations
│   ├── linux.go                 # Linux: wazuh-control integration
│   ├── darwin.go                # macOS: wazuh-control integration
│   └── windows.go               # Windows: Service Manager
├── wazuh-agent-status-client/   # Frontend (systray, user)
│   ├── main.go                  # UI, TCP client, status stream
│   └── assets/                  # Icons
└── scripts/                     # Installers
```

**Architecture:** Split-process for security
- Backend runs as root (controls Wazuh service)
- Frontend runs as user (system tray GUI)
- Communication via TCP localhost:50505 (push-based)

---

## Key Code Locations

| Component | Key Functions | File |
|-----------|-------------|------|
| Backend | TCP server, command handling | `main.go:handleConnection()` |
| Backend | State management | `main.go:StateManager` |
| Backend | Event notification | `main.go:EventNotifier` |
| Backend | Agent status check | `linux.go:checkServiceStatus()` |
| Backend | Agent update | `linux.go:updateAgent()` |
| Frontend | Systray init | `main.go:onReady()` |
| Frontend | Status stream | `main.go:monitorStatusStream()` |
| Frontend | Update handling | `main.go:startUpdateStream()` |
| Frontend | Version check | `main.go:handleVersionCheck()` |

---

## Development Workflow

```bash
# Make changes to backend
cd wazuh-agent-status
go build -o ../dist/wazuh-agent-status .
sudo ../dist/wazuh-agent-status  # Test

# Make changes to frontend
cd wazuh-agent-status-client
go build -o ../dist/wazuh-agent-status-client .
../dist/wazuh-agent-status-client  # Test
```

**Commit format:** `feat:`, `fix:`, `docs:`, `refactor:`

---

## Testing

### Without Wazuh Agent
Expected: Status shows "Inactive/Disconnected" - this is correct behavior.

### With Wazuh Agent
- Verify tray icon appears
- Status updates automatically (push-based)
- Click **Update** → Update starts (if available)

### TCP Testing
```bash
nc localhost 50505
> subscribe-status
> get-version
> update
```

### Logs
```bash
tail -f /var/log/wazuh-agent-status.log          # Backend
tail -f ~/.wazuh/wazuh-agent-status-client.log   # Frontend
```

---

## Common Issues

| Issue | Solution |
|-------|----------|
| Port 50505 in use | `sudo lsof -ti:50505 \| xargs sudo kill -9` |
| Connection refused | Start backend first: `sudo ./dist/wazuh-agent-status` |
| `undefined: systray.Run` | `export CGO_ENABLED=1 && go mod tidy` |
| `cannot find package` | `go mod tidy && go mod download` |
| Tray icon not visible (Linux) | `sudo apt install gnome-shell-extension-appindicator` |

---

## See Also

- [Setup Guide](setup.md) - Detailed platform-specific setup
- [Architecture](architecture.md) - System architecture
- [API Reference](api.md) - TCP protocol details
