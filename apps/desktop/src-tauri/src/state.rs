use crate::services::keyring_service::KeyringService;

/// Application state managed by Tauri
/// Wrapped in Mutex when added to Tauri's managed state for thread safety
pub struct AppState {
    pub keyring_service: KeyringService,
}

impl AppState {
    pub fn new(keyring_service: KeyringService) -> Self {
        Self { keyring_service }
    }
}
