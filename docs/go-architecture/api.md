# API Reference

TCP protocol between frontend (client) and backend (server).

---

## Connection

| Property | Value |
|----------|-------|
| Protocol | TCP |
| Host | `localhost` |
| Port | `50505` |
| Format | Plain text + newline (`\n`) |

---

## Commands

| Command | Request | Response | Description |
|---------|---------|----------|-------------|
| `subscribe-status` | `subscribe-status\n` | `STATUS_UPDATE: <Active/Inactive>, <Connected/Disconnected>` | Subscribe to real-time status updates |
| `get-version` | `get-version\n` | `VERSION_CHECK: <version info>` | Get version status |
| `update` | `update\n` | `UPDATE_PROGRESS: <status>` | Trigger stable update |
| `update-prerelease` | `update-prerelease\n` | `UPDATE_PROGRESS: <status>` | Trigger prerelease update |
| `initiate-update-stream` | Internal use | Stream output | Internal: execute stable update |
| `initiate-prerelease-update-stream` | Internal use | Stream output | Internal: execute prerelease update |

---

## Status Updates (Push-Based)

The frontend subscribes via `subscribe-status` and receives real-time pushes:

```
STATUS_UPDATE: Active, Connected
STATUS_UPDATE: Inactive, Disconnected
```

---

## Example Session

```bash
$ nc localhost 50505

> subscribe-status
STATUS_UPDATE: Active, Connected

> get-version
VERSION_CHECK: Up to date, v4.7.2

> update
UPDATE_PROGRESS: Starting...
UPDATE_PROGRESS: Downloading...
UPDATE_PROGRESS: Complete
```

---

## Implementation

### Go
```go
// Subscribe to status updates
conn, _ := net.Dial("tcp", "localhost:50505")
fmt.Fprintln(conn, "subscribe-status")
reader := bufio.NewReader(conn)
for {
    line, _ := reader.ReadString('\n')
    // Handle STATUS_UPDATE
}
```

---

## Extending

To add a command:
1. Backend: Add case in `handleConnection()` switch in `main.go`
2. Frontend: Add handler in `main.go`
3. Document here
