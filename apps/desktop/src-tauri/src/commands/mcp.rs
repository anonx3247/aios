use crate::state::AppState;
use crate::types::config::McpServerStatus;
use crate::types::errors::AppError;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

/// Start an MCP server by name
#[tauri::command]
pub async fn start_mcp_server(
    name: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    // Retrieve all secrets from keyring to pass as environment variables
    // and get reference to mcp_manager (both in same lock scope)
    let (env_overrides, mcp_manager) = {
        let app_state = state.lock().unwrap();
        let keys = app_state.keyring_service.list_keys();

        let mut env_vars = HashMap::new();
        for key in keys {
            if let Ok(value) = app_state.keyring_service.get_secret(&key) {
                env_vars.insert(key, value);
            }
        }

        // Clone the Arc to mcp_manager so we can use it outside the lock
        (env_vars, app_state.mcp_manager.clone())
    };

    // Start the server with environment overrides (lock is now released)
    mcp_manager.start_server(&name, env_overrides).await
}

/// Stop an MCP server by name
#[tauri::command]
pub async fn stop_mcp_server(
    name: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), AppError> {
    // Clone Arc outside the lock
    let mcp_manager = {
        let app_state = state.lock().unwrap();
        app_state.mcp_manager.clone()
    };

    mcp_manager.stop_server(&name).await
}

/// List all MCP servers with their current status
#[tauri::command]
pub async fn list_mcp_servers(
    state: State<'_, Mutex<AppState>>,
) -> Result<HashMap<String, McpServerStatus>, AppError> {
    // Clone Arc outside the lock
    let mcp_manager = {
        let app_state = state.lock().unwrap();
        app_state.mcp_manager.clone()
    };

    Ok(mcp_manager.list_servers().await)
}
