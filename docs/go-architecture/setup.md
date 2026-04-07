# Setup & Installation Guide

Complete guide for installing and running Wazuh Agent Status.

---

## 1. Quick Start (End Users)

Install pre-built binaries with one command.

### Linux / macOS

```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.sh | sh
```

**Verification:**
```bash
which wazuh-agent-status-client
wazuh-agent-status-client
# System tray icon should appear
```

### Windows

```powershell
# Run as Administrator
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.ps1" -OutFile "install.ps1"
.\install.ps1
```

**Verification:**
```powershell
Get-Command wazuh-agent-status-client
wazuh-agent-status-client
# Icon should appear in system tray
```

---

## 2. Manual Build from Source

Build binaries manually from source code.

### Prerequisites

| OS | Required Packages |
|----|-------------------|
| **Linux (Ubuntu/Debian)** | `sudo apt update && sudo apt install -y golang-go gcc git libayatana-appindicator3-dev netcat` |
| **macOS** | `xcode-select --install && brew install go` |
| **Windows** | [Go](https://go.dev/dl/), [MinGW-w64](https://www.mingw-w64.org/), [Git](https://git-scm.com/) |

Verify: `go version && gcc --version`

### Linux Build

```bash
# Clone repository
git clone https://github.com/ADORSYS-GIS/wazuh-agent-status.git
cd wazuh-agent-status

# Create dist directory
mkdir -p dist

# Build backend
cd wazuh-agent-status
go mod tidy
go mod download
go build -o ../dist/wazuh-agent-status .
cd ..

# Build frontend
cd wazuh-agent-status-client
go mod tidy
go mod download
go build -o ../dist/wazuh-agent-status-client .
cd ..

# Verify
ls -la dist/
```

### macOS Build

```bash
# Clone repository
git clone https://github.com/ADORSYS-GIS/wazuh-agent-status.git
cd wazuh-agent-status

# Create dist directory
mkdir -p dist

# Build backend
cd wazuh-agent-status
go mod tidy
go mod download
go build -o ../dist/wazuh-agent-status .
cd ..

# Build frontend
cd wazuh-agent-status-client
go mod tidy
go mod download
go build -o ../dist/wazuh-agent-status-client .
cd ..

# Verify
ls -la dist/
```

### Windows Build

```powershell
# Clone repository
git clone https://github.com/ADORSYS-GIS/wazuh-agent-status.git
cd wazuh-agent-status

# Create dist directory
mkdir dist

# Build backend
cd wazuh-agent-status
go mod tidy
go mod download
go build -o ..\dist\wazuh-agent-status.exe .
cd ..

# Build frontend
cd wazuh-agent-status-client
go mod tidy
go mod download
go build -o ..\dist\wazuh-agent-status-client.exe .
cd ..

# Verify
ls dist\
```

### Cross-Compilation

```bash
# Linux → macOS
GOOS=darwin GOARCH=amd64 go build -o dist/wazuh-agent-status-darwin

# Linux → Windows
GOOS=windows GOARCH=amd64 go build -o dist/wazuh-agent-status-windows.exe

# macOS → Linux
GOOS=linux GOARCH=amd64 go build -o dist/wazuh-agent-status-linux

# Windows → Linux (PowerShell)
$env:GOOS="linux"; $env:GOARCH="amd64"; go build -o dist/wazuh-agent-status-linux
```

---

## 3. Installation Methods

### Method A: Install Scripts (Recommended)

Uses the official install scripts to set up binaries, services, and autostart.

**Linux/macOS:**
```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.sh | sh
```

**Windows (as Administrator):**
```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.ps1" -OutFile "install.ps1"
.\install.ps1
```

**What it does:**
- Downloads binaries to `/usr/local/bin/` (Linux/macOS) or `C:\Program Files\wazuh-agent-status\` (Windows)
- Sets up backend as system service (systemd/LaunchDaemon/Windows Service)
- Configures frontend autostart
- Creates necessary directories and permissions

### Method B: Manual Binary Placement

Manually copy built binaries to system locations.

**Linux/macOS:**
```bash
# Copy binaries
sudo cp dist/wazuh-agent-status /usr/local/bin/
sudo cp dist/wazuh-agent-status-client /usr/local/bin/
sudo chmod +x /usr/local/bin/wazuh-agent-status*

# Create log directory
mkdir -p ~/.wazuh

# Run backend (manual, no service)
sudo wazuh-agent-status

# Run frontend (in another terminal)
wazuh-agent-status-client
```

**Windows:**
```powershell
# Create directory
mkdir "C:\Program Files\wazuh-agent-status"

# Copy binaries
copy dist\wazuh-agent-status.exe "C:\Program Files\wazuh-agent-status\"
copy dist\wazuh-agent-status-client.exe "C:\Program Files\wazuh-agent-status\"

# Add to PATH (optional)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Program Files\wazuh-agent-status", "User")

# Run backend as Administrator
& "C:\Program Files\wazuh-agent-status\wazuh-agent-status.exe"

# Run frontend
& "C:\Program Files\wazuh-agent-status\wazuh-agent-status-client.exe"
```

### Method C: Development Mode (No Service Setup)

Run directly from build directory without system installation.

```bash
# Terminal 1: Start backend
sudo ./dist/wazuh-agent-status

# Terminal 2: Start frontend
./dist/wazuh-agent-status-client
```

**Use case:** Development, testing, or when you don't want system service integration.

---

## 4. Troubleshooting

### Build Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `undefined: systray.Run` | CGO disabled | `export CGO_ENABLED=1 && go mod tidy` |
| `cannot find package` | Missing dependencies | `go mod tidy && go mod download` |
| `invalid go version '1.22.5'` | Go version format | Edit `go.mod`: change `go 1.22.5` to `go 1.22` |
| `package cmp/slices not in GOROOT` | Go version too old | Install Go 1.22+ from https://go.dev/dl/ |

### Runtime Issues by Platform

**All Platforms:**
| Issue | Cause | Solution |
|-------|-------|----------|
| Port 50505 in use | Another process using port | `sudo lsof -ti:50505 \| xargs kill -9` |
| Connection refused | Backend not running | Start backend first: `sudo ./dist/wazuh-agent-status` |
| Status "Unknown" | Can't read agent state | Check Wazuh agent is installed |

**Linux:**
| Issue | Solution |
|-------|----------|
| Tray icon not visible | `sudo apt install gnome-shell-extension-appindicator` |
| Permission denied | Run backend with `sudo` |

**macOS:**
| Issue | Solution |
|-------|----------|
| App blocked by security | System Settings → Privacy & Security → Allow Anyway |
| Tray icon not visible | System Settings → Control Center → Menu Bar |

**Windows:**
| Issue | Solution |
|-------|----------|
| Service won't start | Run PowerShell as Administrator |
| Tray icon hidden | Check hidden icons (up arrow in system tray) |

### Log Locations for Debugging
                                        

| Component | Linux/macOS | Windows |
|-----------|-------------|---------|
| Backend | `/var/log/wazuh-agent-status.log` | `C:\ProgramData\wazuh\logs\wazuh-agent-status.log` |
| Frontend | `~/.wazuh/wazuh-agent-status-client.log` | `%APPDATA%\wazuh\logs\wazuh-agent-status-client.log` |

View logs:
```bash
# Linux/macOS
tail -f /var/log/wazuh-agent-status.log
tail -f ~/.wazuh/wazuh-agent-status-client.log

# Windows (PowerShell)
Get-Content "C:\ProgramData\wazuh\logs\wazuh-agent-status.log" -Tail 20 -Wait
Get-Content "$env:APPDATA\wazuh\logs\wazuh-agent-status-client.log" -Tail 20 -Wait
```

---

## 5. Uninstallation & Cleanup

### Linux

```bash
# Stop processes
sudo pkill wazuh-agent-status
pkill wazuh-agent-status-client

# Remove binaries
sudo rm -f /usr/local/bin/wazuh-agent-status*

# Remove service
sudo systemctl stop wazuh-agent-status 2>/dev/null
sudo systemctl disable wazuh-agent-status 2>/dev/null
sudo rm -f /etc/systemd/system/wazuh-agent-status.service
sudo systemctl daemon-reload

# Remove logs and config
sudo rm -f /var/log/wazuh-agent-status.log
rm -f ~/.wazuh/wazuh-agent-status-client.log
rm -f ~/.config/autostart/wazuh-agent-status-client.desktop

# Remove project directory (if applicable)
cd ~
rm -rf wazuh-agent-status
```

### macOS

```bash
# Stop processes
sudo pkill wazuh-agent-status
pkill wazuh-agent-status-client

# Remove binaries
sudo rm -f /usr/local/bin/wazuh-agent-status*

# Remove LaunchAgent/LaunchDaemon
sudo rm -f /Library/LaunchDaemons/com.adorsys.wazuh-agent-status.plist
rm -f ~/Library/LaunchAgents/com.adorsys.wazuh-agent-status-client.plist

# Remove logs and config
sudo rm -f /var/log/wazuh-agent-status.log
rm -f ~/.wazuh/wazuh-agent-status-client.log

# Remove project directory (if applicable)
cd ~
rm -rf wazuh-agent-status
```

### Windows

```powershell
# Stop service (as Administrator)
Stop-Service -Name "GoWazuhService" -Force 2>$null

# Remove service
dist\wazuh-agent-status.exe stop 2>$null
dist\wazuh-agent-status.exe uninstall 2>$null

# Remove binaries
Remove-Item -Path "C:\Program Files\wazuh-agent-status" -Recurse -Force -ErrorAction SilentlyContinue

# Remove logs and config
Remove-Item -Path "$env:APPDATA\wazuh" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "$env:ProgramData\wazuh" -Recurse -Force -ErrorAction SilentlyContinue

# Remove project directory (if applicable)
Remove-Item -Path "$env:USERPROFILE\wazuh-agent-status" -Recurse -Force -ErrorAction SilentlyContinue
```

### Using Uninstall Scripts

**Linux/macOS:**
```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/uninstall.sh | sh
```

**Windows:**
```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/uninstall.ps1" -OutFile "uninstall.ps1"
.\uninstall.ps1
```

---

## Quick Reference

| Task | Command |
|------|---------|
| **Install (quick)** | `curl -sL .../install.sh \| sh` |
| **Build backend** | `cd wazuh-agent-status && go mod tidy && go build -o ../dist/wazuh-agent-status .` |
| **Build frontend** | `cd wazuh-agent-status-client && go mod tidy && go build -o ../dist/wazuh-agent-status-client .` |
| **Run backend** | `sudo ./dist/wazuh-agent-status` |
| **Run frontend** | `./dist/wazuh-agent-status-client` |
| **Test TCP** | `nc localhost 50505` → type `subscribe-status` |
| **View backend log** | `tail -f /var/log/wazuh-agent-status.log` |
| **View frontend log** | `tail -f ~/.wazuh/wazuh-agent-status-client.log` |
| **Uninstall** | `curl -sL .../uninstall.sh \| sh` |

---

## See Also

- [Architecture](architecture.md) - System architecture overview
- [Developer Guide](dev.md) - Developer guide
- [API Reference](api.md) - TCP protocol details
