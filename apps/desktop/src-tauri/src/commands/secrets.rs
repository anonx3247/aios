use crate::state::AppState;
use crate::types::errors::AppError;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize)]
pub struct SecretEntry {
    pub key: String,
    pub value: String,
}

/// Get all secrets as key-value pairs
#[tauri::command]
pub fn get_secrets(state: State<'_, Mutex<AppState>>) -> Result<Vec<SecretEntry>, AppError> {
    let mut app_state = state.lock().unwrap();
    let keys = app_state.keyring_service.list_keys();

    let mut secrets = Vec::new();
    let mut keys_to_remove = Vec::new();

    for key in keys {
        match app_state.keyring_service.get_secret(&key) {
            Ok(value) => secrets.push(SecretEntry { key, value }),
            Err(_) => {
                // Track keys that can't be retrieved (might have been deleted outside app)
                keys_to_remove.push(key);
            }
        }
    }

    // Clean up stale keys
    for key in keys_to_remove {
        let _ = app_state.keyring_service.remove_key(&key);
    }

    Ok(secrets)
}

/// Get a single secret by key
#[tauri::command]
pub fn get_secret(state: State<'_, Mutex<AppState>>, key: String) -> Result<String, AppError> {
    let app_state = state.lock().unwrap();
    app_state.keyring_service.get_secret(&key)
}

/// Set a secret
#[tauri::command]
pub fn set_secret(
    state: State<'_, Mutex<AppState>>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    let mut app_state = state.lock().unwrap();
    app_state.keyring_service.set_secret(&key, &value)
}

/// Delete a secret
#[tauri::command]
pub fn delete_secret(state: State<'_, Mutex<AppState>>, key: String) -> Result<(), AppError> {
    let mut app_state = state.lock().unwrap();
    app_state.keyring_service.delete_secret(&key)
}
