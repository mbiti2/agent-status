# Wazuh Agent Status - Setup Guide

This guide covers setting up the Wazuh Agent Status project from scratch on a fresh system **without requiring the Wazuh agent to be installed**.

---

## Project Overview

This project consists of two components:
- **Backend (`wazuh-agent-status`)**: Service running on port 50505 with elevated privileges
- **Frontend (`wazuh-agent-status-client`)**: System tray GUI application

**Note:** You do NOT need the Wazuh agent installed to build and run this project. Without the agent, the status will show as "Inactive/Disconnected" or "Unknown", which is expected behavior.

---

## Phase 1: Prerequisites Installation

### Step 1.1 - Update Package List
```bash
sudo apt-get update
```

### Step 1.2 - Install Go (1.22.5 or later)
```bash
# Ubuntu/Debian:
sudo apt-get install -y golang-go

# Or download latest from https://go.dev/dl/
# wget https://go.dev/dl/go1.22.5.linux-amd64.tar.gz
# sudo tar -C /usr/local -xzf go1.22.5.linux-amd64.tar.gz
# export PATH=$PATH:/usr/local/go/bin

# Verify installation
go version
```

### Step 1.3 - Install C Compiler (required for systray)
```bash
# Ubuntu/Debian:
sudo apt-get install -y gcc

# macOS:
xcode-select --install

# Windows: Install MinGW
```

### Step 1.4 - Install Linux-specific Dependency (Ubuntu/Debian only)
```bash
# Required for system tray functionality
sudo apt-get install -y libayatana-appindicator3-dev
# Alternative if the above is not available:
# sudo apt-get install -y libappindicator3-dev
```

### Step 1.5 - Install Git (if not already installed)
```bash
sudo apt-get install -y git
```

### Step 1.6 - Install helper tools for testing
```bash
sudo apt-get install -y netcat-openbsd curl
```

---

## Phase 2: Get the Source Code

### Step 2.1 - Clone the repository
```bash
# Navigate to your preferred directory
cd ~
# Or: cd /home/louise/pen-testing

git clone https://github.com/ADORSYS-GIS/wazuh-agent-status.git
cd wazuh-agent-status
```

---

## Phase 3: Build the Project

### Step 3.1 - Create distribution directory
```bash
mkdir -p dist
```

### Step 3.2 - Build the Backend Server
```bash
cd wazuh-agent-status
go mod tidy
go mod download
go build -o ../dist/wazuh-agent-status .
cd ..
```

### Step 3.3 - Build the Frontend Client
```bash
cd wazuh-agent-status-client
go mod tidy
go mod download
go build -o ../dist/wazuh-agent-status-client .
cd ..
```

### Step 3.4 - Verify binaries were created
```bash
ls -la dist/
# Should show: wazuh-agent-status and wazuh-agent-status-client
```

---

## Phase 4: Run the Application

### Step 4.1 - Start the Backend Server (Terminal 1)
```bash
# From the project root directory
sudo ./dist/wazuh-agent-status

# Expected output: Server starts listening on port 50505
# Note: Without Wazuh agent installed, status checks will return "Unknown" or "Inactive"
```

### Step 4.2 - Start the Frontend Client (Terminal 2)
```bash
# Open a new terminal, navigate to project directory
cd ~/wazuh-agent-status
./dist/wazuh-agent-status-client

# Expected: System tray icon appears with Wazuh logo
# Note: On first run, you may need to approve the application in your system tray settings
```

---

## Phase 5: Test the Application

### Step 5.1 - Test Backend via TCP
```bash
# In a new terminal, connect to the backend
nc localhost 50505

# Test commands (type each followed by Enter):
status          # Shows: "Status: Inactive, Connection: Disconnected" (expected without agent)
check-version   # Shows version comparison (may show "Unknown" without agent)
pause           # Will fail gracefully without Wazuh agent
restart         # Will fail gracefully without Wazuh agent
update-status   # Shows update status

# Press Ctrl+C to exit
```

### Step 5.2 - Test via System Tray
- Click the Wazuh icon in your system tray
- Verify "Agent" shows **Inactive** (expected without Wazuh agent)
- Verify "Connection" shows **Disconnected** (expected without Wazuh agent)
- Verify the version info displays at the bottom
- Test that menu items are present (Pause/Restart may be disabled or show errors when clicked)

### Step 5.3 - Expected Behavior Without Wazuh Agent
| Feature | Expected Result |
|---------|-----------------|
| Agent Status | Shows "Inactive" or "Unknown" |
| Connection Status | Shows "Disconnected" or "Unknown" |
| Pause/Restart | May show error or have no effect |
| Version Check | May show "Unknown" or fail gracefully |
| Tray Icon | Gray dot (inactive state) |

---

## Phase 6: Cleanup (Complete Removal)

### Step 6.1 - Stop running processes
```bash
# Stop the client first (Ctrl+C in Terminal 2, or click "Quit" in the tray menu)
# If running in background, find and kill:
pkill wazuh-agent-status-client

# Stop the backend server (Ctrl+C in Terminal 1)
# Or kill if running in background:
sudo pkill wazuh-agent-status
```

### Step 6.2 - Remove build artifacts
```bash
cd ~/wazuh-agent-status
rm -rf dist/
```

### Step 6.3 - Remove Go dependencies (optional)
```bash
# Clean Go module cache (frees disk space)
go clean -modcache

# Remove module directories
rm -rf wazuh-agent-status/go.sum wazuh-agent-status/go.mod
cd wazuh-agent-status && go mod init wazuh-agent-status && go mod tidy
cd ../wazuh-agent-status-client && go mod init wazuh-agent-status-client && go mod tidy
cd ..
```

### Step 6.4 - Remove the cloned repository
```bash
cd ~
rm -rf wazuh-agent-status
```

### Step 6.5 - Uninstall system dependencies (optional - complete cleanup)
```bash
# Remove installed packages (use with caution)
sudo apt-get remove -y golang-go gcc libayatana-appindicator3-dev netcat-openbsd curl git

# Remove unused dependencies
sudo apt-get autoremove -y

# Clean package cache
sudo apt-get clean
```

### Step 6.6 - Remove user data (if any was created)
```bash
# Remove any log files that may have been created
rm -rf ~/.wazuh-agent-status
sudo rm -rf /var/log/wazuh-agent-status*
```

---

## Alternative: Quick Install/Run (Pre-built Binaries)

If you prefer not to build from source:

### Install (downloads and installs binaries + services):
```bash
# Admin version (full control)
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.sh | PROFILE=admin sh

# Or user version (monitoring only)
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.sh | sh
```

### Run:
```bash
wazuh-agent-status-client
```

### Uninstall (complete cleanup):
```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/uninstall.sh | sh
```

---

## Summary of Key Files Locations

| Component | Path |
|-----------|------|
| Project root | `~/wazuh-agent-status/` |
| Backend source | `./wazuh-agent-status/` |
| Frontend source | `./wazuh-agent-status-client/` |
| Build output | `./dist/` |
| Install scripts | `./scripts/` |
| Logs (if created) | `/var/log/wazuh-agent-status*.log` |

---

## Troubleshooting

### Issue: "cannot find package" during build
**Solution:** Run `go mod tidy` before building

### Issue: System tray icon not visible
**Solution:** Check your desktop environment's system tray settings. Some DEs require whitelisting or enabling tray icons.

### Issue: "permission denied" when running backend
**Solution:** Use `sudo` to run the backend server

### Issue: "port already in use" error
**Solution:** Kill any existing processes using port 50505:
```bash
sudo lsof -ti:50505 | xargs sudo kill -9
```