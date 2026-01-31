use crate::services::keyring_service::KeyringService;
use crate::services::mcp_manager::McpManager;
use std::sync::Arc;

/// Application state managed by Tauri
/// Wrapped in Mutex when added to Tauri's managed state for thread safety
pub struct AppState {
    pub keyring_service: KeyringService,
    pub mcp_manager: Arc<McpManager>,
}

impl AppState {
    pub fn new(keyring_service: KeyringService, mcp_manager: McpManager) -> Self {
        Self {
            keyring_service,
            mcp_manager: Arc::new(mcp_manager),
        }
    }
}
