# Phase 2: Tauri Shell - Research

**Researched:** 2026-01-31
**Domain:** Tauri 2.0 desktop application development with Rust backend
**Confidence:** HIGH

## Summary

This research covers the implementation of a Tauri 2.0 shell with system keyring integration, MCP server process management, frameless window UI, and SQLite database persistence. Tauri 2.0 (currently v2.9.5 as of December 2024) provides a mature framework for building native desktop applications with a Rust backend and web frontend.

The standard approach uses Tauri's plugin architecture for system integration (tray, global shortcuts, SQL), the keyring crate for cross-platform secrets management, and tokio for async process lifecycle management. The new permissions/capabilities ACL system replaces Tauri 1.x's allowlist and provides granular control over IPC command access.

For the locked decisions from CONTEXT.md, the research validates that all requirements are achievable: system keyring via the keyring crate (v3.6.3), frameless window with tray-only activation via `ActivationPolicy::Accessory`, lazy MCP server spawning via tokio::process, and dismiss-on-blur via window event listeners.

**Primary recommendation:** Use official Tauri plugins (tray-icon, global-shortcut) for UI behavior, the keyring crate for secrets, tokio::process with the backoff crate for MCP server lifecycle, and Tauri's built-in state management with Mutex for shared state. Avoid hand-rolling process management or custom IPC protocols.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2.9.5+ | Desktop app framework with IPC and window management | Official framework, mature v2 release with mobile support |
| keyring | 3.6.3 | Cross-platform system keyring access | Standard Rust crate for secure credential storage |
| tokio | 1.x | Async runtime for process management | De facto standard async runtime in Rust ecosystem |
| rusqlite | Latest | SQLite database interface | Most popular SQLite binding for Rust |
| serde | 1.x | Serialization for IPC payloads | Required by Tauri for command serialization |
| thiserror | 1.x | Error type derivation | Standard for creating serializable error types |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tauri-plugin-global-shortcut | 2.0.0+ | Keyboard shortcut registration | For global hotkeys like Escape key |
| backoff | 0.4.0 | Exponential backoff for retries | For MCP server auto-restart with backoff |
| tauri-plugin-sql | 2.x | SQLite plugin (optional) | If using Tauri's plugin approach vs direct rusqlite |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| keyring crate | Manual platform-specific code | Keyring handles macOS Keychain, Windows Credential Manager, Linux Secret Service automatically |
| tokio::process | std::process::Command | Tokio provides async interface, essential for non-blocking MCP server lifecycle |
| tauri-plugin-sql | Direct rusqlite | Plugin provides IPC layer; direct rusqlite better for Rust-only database access |

**Installation:**
```bash
# In src-tauri/Cargo.toml
cd src-tauri
cargo add tauri@2
cargo add keyring@3
cargo add tokio --features full
cargo add rusqlite
cargo add serde --features derive
cargo add thiserror
cargo add backoff --features tokio

# For optional plugins
pnpm run tauri add global-shortcut
```

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/
├── src/
│   ├── main.rs                 # App entry, builder setup, state initialization
│   ├── lib.rs                  # Tauri commands, command registration
│   ├── commands/               # IPC command handlers
│   │   ├── mod.rs
│   │   ├── secrets.rs          # get_secrets, set_secret, delete_secret
│   │   ├── mcp.rs              # start_mcp_server, stop_mcp_server, mcp_health
│   │   └── agent.rs            # spawn_agent_run (if in this phase)
│   ├── services/               # Business logic
│   │   ├── mod.rs
│   │   ├── keyring_service.rs  # Keyring crate wrapper
│   │   ├── mcp_manager.rs      # Process lifecycle, restart logic
│   │   └── config_loader.rs    # MCP config file parsing
│   ├── types/                  # Shared types and errors
│   │   ├── mod.rs
│   │   ├── errors.rs           # Custom error types with Serialize
│   │   └── config.rs           # MCP server config structures
│   └── state.rs                # Application state structures
├── capabilities/
│   └── default.json            # Permission definitions for commands
├── tauri.conf.json             # Window config, tray config, bundle settings
└── Cargo.toml
```

### Pattern 1: Tauri Command with State and Error Handling
**What:** IPC commands use the `#[tauri::command]` macro with snake_case names, inject state via `State<'_, T>` parameter, and return `Result<T, E>` where E implements `Serialize`.
**When to use:** All IPC-exposed functions.
**Example:**
```rust
// Source: https://v2.tauri.app/develop/calling-rust/
use tauri::State;
use std::sync::Mutex;

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum CommandError {
    #[error("Keyring error: {0}")]
    Keyring(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

#[tauri::command]
async fn get_secrets(
    state: State<'_, Mutex<AppState>>
) -> Result<Vec<Secret>, CommandError> {
    let state = state.lock().unwrap();
    state.keyring_service.list_all()
        .map_err(|e| CommandError::Keyring(e.to_string()))
}
```

### Pattern 2: System Keyring Integration
**What:** Use keyring::Entry with service and username pattern for credential storage.
**When to use:** All secrets management operations.
**Example:**
```rust
// Source: https://docs.rs/keyring/latest/keyring/
use keyring::{Entry, Result};

pub struct KeyringService {
    service_name: String,
}

impl KeyringService {
    pub fn new(service_name: &str) -> Self {
        Self { service_name: service_name.to_string() }
    }

    pub fn set_secret(&self, key: &str, value: &str) -> Result<()> {
        let entry = Entry::new(&self.service_name, key)?;
        entry.set_password(value)?;
        Ok(())
    }

    pub fn get_secret(&self, key: &str) -> Result<String> {
        let entry = Entry::new(&self.service_name, key)?;
        entry.get_password()
    }

    pub fn delete_secret(&self, key: &str) -> Result<()> {
        let entry = Entry::new(&self.service_name, key)?;
        entry.delete_credential()
    }
}
```

### Pattern 3: MCP Server Process Lifecycle with Backoff
**What:** Spawn child processes with tokio::process, track state, implement auto-restart with exponential backoff.
**When to use:** MCP server lazy startup and crash recovery.
**Example:**
```rust
// Source: https://docs.rs/tokio/latest/tokio/process/
// Source: https://docs.rs/backoff
use tokio::process::Command;
use backoff::{future::retry, ExponentialBackoff};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub child: Option<tokio::process::Child>,
    pub retry_count: u32,
}

impl McpManager {
    pub async fn start_server(&mut self, name: &str) -> Result<(), McpError> {
        let config = self.get_config(name)?;

        // Exponential backoff retry logic
        retry(ExponentialBackoff::default(), || async {
            self.spawn_process(config).await
                .map_err(|e| match e {
                    McpError::Transient(_) => backoff::Error::transient(e),
                    _ => backoff::Error::permanent(e),
                })
        }).await
    }

    async fn spawn_process(&mut self, config: &McpConfig) -> Result<(), McpError> {
        let mut child = Command::new(&config.command)
            .args(&config.args)
            .envs(&config.env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)  // Critical: ensures cleanup on drop
            .spawn()?;

        // Store child handle
        self.servers.insert(config.name.clone(), child);
        Ok(())
    }
}
```

### Pattern 4: Frameless Window with Tray-Only Activation
**What:** Configure window as frameless, centered, and hide from dock using ActivationPolicy::Accessory.
**When to use:** Menu bar-only applications (Raycast-style).
**Example:**
```rust
// Source: https://v2.tauri.app/learn/window-customization/
// Source: https://github.com/tauri-apps/tauri/discussions/10774
use tauri::{Manager, ActivationPolicy};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Hide from dock (macOS), set accessory activation policy
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            // Get main window and configure
            let window = app.get_webview_window("main").unwrap();
            window.set_decorations(false)?;  // Frameless
            window.center()?;  // Center on screen

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Window configuration in tauri.conf.json:**
```json
{
  "app": {
    "windows": [{
      "label": "main",
      "title": "AIOS",
      "width": 800,
      "height": 600,
      "center": true,
      "decorations": false,
      "visible": true,
      "focus": true
    }]
  }
}
```

### Pattern 5: Dismiss on Blur
**What:** Listen to window blur events and hide window when focus is lost.
**When to use:** Launcher-style windows that should dismiss when clicked outside.
**Example:**
```javascript
// Source: https://v2.tauri.app/reference/javascript/api/namespacewindow/
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const window = getCurrentWebviewWindow();

// Listen for blur events
window.onFocusChanged(async ({ payload: focused }) => {
  if (!focused) {
    await window.hide();
  }
});
```

**Alternative with global shortcut for Escape key:**
```rust
// Source: https://v2.tauri.app/plugin/global-shortcut/
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

app.plugin(tauri_plugin_global_shortcut::Builder::new().build())?;

let shortcut = "Escape".parse::<Shortcut>()?;
app.global_shortcut().register(shortcut, move |app, _shortcut, _event| {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
})?;
```

### Pattern 6: Tauri State Management with Mutex
**What:** Use `app.manage()` to register application state, wrap mutable state in `Mutex`, access via `State<'_, T>` in commands.
**When to use:** All shared application state (keyring service, MCP manager, config).
**Example:**
```rust
// Source: https://v2.tauri.app/develop/state-management/
use std::sync::Mutex;
use tauri::{Manager, State};

pub struct AppState {
    pub keyring: KeyringService,
    pub mcp_manager: Arc<Mutex<McpManager>>,
}

fn main() {
    let state = AppState {
        keyring: KeyringService::new("com.aios.app"),
        mcp_manager: Arc::new(Mutex::new(McpManager::new())),
    };

    tauri::Builder::default()
        .manage(Mutex::new(state))
        .invoke_handler(tauri::generate_handler![
            get_secrets,
            start_mcp_server,
        ])
        .run(tauri::generate_context!())
        .unwrap();
}
```

### Pattern 7: MCP Config File Format (Claude Desktop Compatible)
**What:** Use JSON config similar to claude_desktop_config.json with mcpServers object containing server definitions.
**When to use:** MCP server configuration persistence.
**Example:**
```json
// Source: https://modelcontextprotocol.io/docs/develop/connect-local-servers
{
  "mcpServers": {
    "server-name": {
      "command": "node",
      "args": ["/path/to/server.js"],
      "env": {
        "API_KEY": "secret_value"
      }
    },
    "another-server": {
      "command": "python",
      "args": ["-m", "mcp_server"],
      "env": {}
    }
  }
}
```

**Rust structure:**
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}
```

### Pattern 8: Capabilities and Permissions
**What:** Define permission sets in `capabilities/` directory, reference in tauri.conf.json.
**When to use:** All IPC commands require explicit permissions in Tauri 2.0.
**Example:**
```json
// Source: https://v2.tauri.app/security/capabilities/
// File: src-tauri/capabilities/default.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default permissions for main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-center",
    "core:window:allow-set-decorations",
    "core:window:allow-hide",
    "core:window:allow-show",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister"
  ]
}
```

### Anti-Patterns to Avoid
- **Multiple invoke_handler calls:** Only the last registration is used. Pass all commands to a single `tauri::generate_handler![]` call.
- **Dropping Child without waiting:** Results in zombie processes on Unix. Always use `.kill_on_drop(true)` or properly await `.wait()`.
- **Unserializable error types:** IPC requires `Serialize` on all return types including errors. Use `thiserror` and manual Serialize impl.
- **String-based error handling:** Converting all errors to String loses type information. Use custom error enums.
- **Blocking operations in commands:** Commands should be async and use tokio for I/O operations to avoid blocking the IPC channel.
- **Storing secrets in plain text:** Always use system keyring, never environment variables or config files for sensitive data.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cross-platform keyring | Manual platform detection + Win32/macOS APIs | keyring crate | Handles Windows Credential Manager, macOS Keychain, Linux Secret Service, FreeBSD, OpenBSD |
| Process lifecycle | Custom process tracking + restart logic | tokio::process + backoff crate | Handles async I/O, zombie prevention, exponential backoff automatically |
| Exponential backoff | Custom retry counters and sleep timers | backoff crate | Handles jitter, max retries, transient vs permanent errors |
| IPC serialization | Custom JSON protocol | Tauri's built-in invoke system | Type-safe, handles permissions, supports raw payloads for large data |
| Window management | Direct platform APIs | Tauri's window API | Cross-platform abstraction for decorations, positioning, focus |
| Global shortcuts | Platform-specific hotkey registration | tauri-plugin-global-shortcut | Cross-platform, handles conflicts, integrates with Tauri permissions |

**Key insight:** Tauri 2.0's plugin architecture and Rust ecosystem provide battle-tested solutions for all common desktop app patterns. Custom solutions introduce platform-specific bugs and security risks.

## Common Pitfalls

### Pitfall 1: Zombie Process Accumulation
**What goes wrong:** MCP server processes become zombies when Child handles are dropped without calling `.wait()`, exhausting process IDs on long-running applications.
**Why it happens:** Rust std::process and tokio::process do not automatically wait on child processes, even when dropped. On Unix, parent must reap children.
**How to avoid:** Always set `.kill_on_drop(true)` on Command builder, or ensure `.wait().await` is called before dropping Child handle. Tokio runtime provides best-effort cleanup but no guarantees.
**Warning signs:** Increasing process count in system monitor, "fork: Resource temporarily unavailable" errors.

### Pitfall 2: Keyring Thread Safety Violations
**What goes wrong:** Simultaneous access to same keyring entry from multiple threads causes errors on Windows and Linux (DBus Secret Service).
**Why it happens:** RPC-based credential stores (Windows, Linux DBus) don't handle concurrent access to same credential well.
**How to avoid:** Serialize access to keyring operations using a Mutex around KeyringService. Avoid rapid successive calls across threads.
**Warning signs:** Intermittent keyring errors on Linux/Windows but not macOS, race condition failures.

### Pitfall 3: IPC Serialization Performance
**What goes wrong:** Returning large data (files, binary blobs) via IPC causes severe slowdown due to JSON serialization.
**Why it happens:** Tauri IPC serializes all return values to JSON. Large data structures have quadratic serialization cost.
**How to avoid:** For large data transfers, use `tauri::ipc::Response` with raw byte arrays or stream data through filesystem. Keep IPC payloads small (<1MB).
**Warning signs:** UI freezes when loading large datasets, high CPU usage during IPC calls.

### Pitfall 4: Permission Denials on Commands
**What goes wrong:** IPC commands fail with permission denied errors despite being registered.
**Why it happens:** Tauri 2.0 requires explicit permission grants in capabilities files. Default-deny security model.
**How to avoid:** Create capability files in `src-tauri/capabilities/` with `allow` permissions for each command. Check console for permission denial warnings.
**Warning signs:** Commands work in Tauri 1.x but fail in 2.0, "Command not allowed" errors in console.

### Pitfall 5: Dock Icon Reappearance
**What goes wrong:** Setting `ActivationPolicy::Accessory` hides dock icon initially, but it reappears when window is shown/hidden.
**Why it happens:** Window visibility changes can reset activation policy on some macOS versions.
**How to avoid:** Re-apply activation policy in window event handlers if needed. Consider using `visible: false` in initial window config and showing programmatically.
**Warning signs:** Dock icon flickers or reappears after window operations.

### Pitfall 6: MCP Server Environment Variable Injection
**What goes wrong:** Secrets passed as environment variables to MCP servers are visible in process listings and logs.
**Why it happens:** Environment variables are not secure on Unix systems (visible via `/proc/<pid>/environ`).
**How to avoid:** Document that secrets in env vars have this limitation. Consider alternative approaches like temporary files with restricted permissions or stdin injection.
**Warning signs:** Security audit flags environment variable secret exposure.

### Pitfall 7: Window Blur Event Reliability
**What goes wrong:** Blur events don't fire consistently, especially on Linux or when interacting with drag regions.
**Why it happens:** Platform differences in focus event semantics. Known bugs with `data-tauri-drag-region` triggering blur on Linux.
**How to avoid:** Combine blur detection with global shortcut (Escape key) as fallback. Test on all target platforms. Consider polling `window.isFocused()` as backup.
**Warning signs:** Window doesn't hide on blur on Linux, blur fires unexpectedly on drag.

### Pitfall 8: Multiple States of Same Type
**What goes wrong:** Calling `app.manage()` multiple times with same type only registers the first instance.
**Why it happens:** Tauri's state management uses type-based lookup, not instance-based.
**How to avoid:** Use a single state struct containing all sub-services. If multiple instances needed, wrap in Vec or HashMap within a single state type.
**Warning signs:** State changes not reflected, `State<T>` retrieves wrong instance.

## Code Examples

Verified patterns from official sources:

### System Tray Setup with Menu
```rust
// Source: https://v2.tauri.app/learn/system-tray/
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::{Manager, AppHandle};

fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|_tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                println!("Tray icon clicked with {:?}", button);
            }
        })
        .build(app)?;

    Ok(())
}
```

### SQLite Database Initialization
```rust
// Source: https://v2.tauri.app/plugin/sql/
// Using tauri-plugin-sql approach
use tauri_plugin_sql::{Builder, Migration, MigrationKind};

fn main() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create initial tables",
            sql: "CREATE TABLE IF NOT EXISTS runs (
                id INTEGER PRIMARY KEY,
                agent_id TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                status TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY,
                run_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                FOREIGN KEY(run_id) REFERENCES runs(id)
            );",
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .plugin(
            Builder::default()
                .add_migrations("sqlite:aios.db", migrations)
                .build()
        )
        .run(tauri::generate_context!())
        .unwrap();
}
```

### Error Type with Serialization
```rust
// Source: https://tauritutorials.com/blog/handling-errors-in-tauri
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Keyring error: {0}")]
    Keyring(String),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Convert from other error types
impl From<keyring::Error> for AppError {
    fn from(err: keyring::Error) -> Self {
        AppError::Keyring(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}
```

### MCP Config File Loading
```rust
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

impl McpConfig {
    pub fn load(path: &PathBuf) -> Result<Self, AppError> {
        let content = fs::read_to_string(path)?;
        let config: McpConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri 1.x allowlist | Permissions/capabilities ACL system | Tauri 2.0 (Oct 2024) | More granular control, multiwindow support, security hardening |
| system-tray feature flag | tray-icon feature flag | Tauri 2.0 | Naming clarity, same functionality |
| Custom IPC protocol | Built-in invoke with raw payload support | Tauri 2.0 | Better performance for large data, type safety maintained |
| keyring::get_password/set_password | keyring::get_password/set_password/delete_credential | keyring v3.0 | delete_password renamed to clarify credential vs password |
| AppConfig path: manual | BaseDirectory::AppConfig/{bundle_id} | Tauri 2.0 | Automatic app-scoped directories |
| Manual error serialization | thiserror + manual Serialize impl | Current | Type-safe errors with Display, Error traits |

**Deprecated/outdated:**
- **Allowlist in tauri.conf.json**: Replaced by capabilities files in Tauri 2.0. Old configs will not work.
- **v1 window API**: Many methods renamed or moved to different modules in v2. Check migration guide.
- **delete_password method**: Renamed to `delete_credential` in keyring v3.x to clarify that entire entry is removed.

## Open Questions

Things that couldn't be fully resolved:

1. **SQLite persistence scope**
   - What we know: Phase context states "Harness handles its own SQLite persistence — Tauri doesn't manage the database schema"
   - What's unclear: Whether Phase 2 needs ANY SQLite setup, or if it's entirely deferred to harness implementation
   - Recommendation: Implement minimal SQLite schema only if needed for Tauri-side data (MCP config cache, etc.). Otherwise defer to harness phase.

2. **MCP server health check approach**
   - What we know: Context lists this under "Claude's Discretion"
   - What's unclear: Whether health checks should be process-level (PID exists), protocol-level (send MCP ping), or passive (restart on stderr output)
   - Recommendation: Start with process-level (check if child PID exists and hasn't exited). Add protocol-level checks if MCP spec defines health check messages.

3. **Window dimensions**
   - What we know: Context lists "Exact window dimensions" under Claude's discretion
   - What's unclear: Optimal width/height for launcher-style window
   - Recommendation: Research Raycast dimensions (appears to be ~800x600 for main launcher). Make configurable in tauri.conf.json.

4. **Keyring service naming scheme**
   - What we know: Context lists this under Claude's discretion
   - What's unclear: Service name format (reverse domain? app name?) and whether to use target parameter
   - Recommendation: Use reverse domain notation matching bundle_identifier (e.g., "com.aios.app"). Use key name as username, omit target.

## Sources

### Primary (HIGH confidence)
- [Tauri 2.0 Official Documentation](https://v2.tauri.app/) - Core concepts, IPC, state management
- [Tauri v2.9.5 Release](https://github.com/tauri-apps/tauri/releases) - Current stable version
- [Window Customization Guide](https://v2.tauri.app/learn/window-customization/) - Frameless windows, decorations
- [System Tray Guide](https://v2.tauri.app/learn/system-tray/) - Tray icon setup and events
- [Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/) - Command patterns, error handling
- [State Management](https://v2.tauri.app/develop/state-management/) - Manager API, Mutex patterns
- [Capabilities](https://v2.tauri.app/security/capabilities/) - Permission system, ACL configuration
- [Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/) - Keyboard shortcuts
- [keyring crate docs](https://docs.rs/keyring/latest/keyring/) - Version 3.6.3 API
- [tokio::process docs](https://docs.rs/tokio/latest/tokio/process/) - Async process management
- [backoff crate docs](https://docs.rs/backoff) - Exponential backoff patterns (v0.4.0)
- [MCP Server Config Format](https://modelcontextprotocol.io/docs/develop/connect-local-servers) - claude_desktop_config.json structure

### Secondary (MEDIUM confidence)
- [Toggle dock icon on macOS Discussion](https://github.com/tauri-apps/tauri/discussions/10774) - ActivationPolicy::Accessory pattern
- [Tauri Error Handling Tutorial](https://tauritutorials.com/blog/handling-errors-in-tauri) - thiserror + Serialize pattern
- [Dealing with process termination in Linux](https://iximiuz.com/en/posts/dealing-with-processes-termination-in-Linux/) - Zombie process prevention
- [How to Implement Retry Logic with Exponential Backoff in Rust](https://oneuptime.com/blog/post/2026-01-07-rust-retry-exponential-backoff/view) - January 2026 guide
- [IPC Common Mistakes](https://v2.tauri.app/concept/inter-process-communication/) - Performance issues, serialization limits

### Tertiary (LOW confidence)
- Window blur event reliability issues on Linux - based on GitHub issues, needs verification per platform

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries verified via official docs, current versions confirmed
- Architecture: HIGH - Patterns from official Tauri documentation and established Rust ecosystem practices
- Pitfalls: MEDIUM-HIGH - Mix of official documentation warnings and community-reported issues

**Research date:** 2026-01-31
**Valid until:** 2026-02-28 (30 days - Tauri 2.x is stable, keyring is mature)
