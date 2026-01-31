use crate::types::config::McpConfig;
use crate::types::errors::AppError;
use std::fs;
use std::path::{Path, PathBuf};

/// Load MCP configuration from a file
pub fn load(path: &Path) -> Result<McpConfig, AppError> {
    if !path.exists() {
        // Create default empty config if file doesn't exist
        let default_config = McpConfig::default();
        save(&default_config, path)?;
        return Ok(default_config);
    }

    let contents = fs::read_to_string(path)?;
    let config: McpConfig = serde_json::from_str(&contents)
        .map_err(|e| AppError::Io(format!("Failed to parse MCP config: {}", e)))?;

    Ok(config)
}

/// Save MCP configuration to a file
pub fn save(config: &McpConfig, path: &Path) -> Result<(), AppError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Io(format!("Failed to serialize MCP config: {}", e)))?;

    fs::write(path, json)?;
    Ok(())
}

/// Get the default path for the MCP config file
pub fn default_config_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("mcp_config.json")
}
