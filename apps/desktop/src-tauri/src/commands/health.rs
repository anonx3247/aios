use crate::types::errors::AppError;

/// Health check command to validate IPC pipeline
#[tauri::command]
pub async fn health_check() -> Result<String, AppError> {
    Ok("ok".to_string())
}
