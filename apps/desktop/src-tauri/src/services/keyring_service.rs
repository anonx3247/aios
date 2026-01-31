use crate::types::errors::AppError;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Service for managing secrets in the system keyring
pub struct KeyringService {
    service_name: String,
    keys_file_path: PathBuf,
    known_keys: HashSet<String>,
}

impl KeyringService {
    /// Create a new KeyringService instance
    pub fn new(service_name: &str, app_data_dir: PathBuf) -> Result<Self, AppError> {
        // Ensure app data directory exists
        fs::create_dir_all(&app_data_dir)?;

        let keys_file_path = app_data_dir.join("keyring_keys.json");

        // Load existing keys from file if it exists
        let known_keys = if keys_file_path.exists() {
            let contents = fs::read_to_string(&keys_file_path)?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            HashSet::new()
        };

        Ok(Self {
            service_name: service_name.to_string(),
            keys_file_path,
            known_keys,
        })
    }

    /// Set a secret in the keyring
    pub fn set_secret(&mut self, key: &str, value: &str) -> Result<(), AppError> {
        let entry = keyring::Entry::new(&self.service_name, key)?;
        entry.set_password(value)?;

        // Add key to known keys and persist
        self.known_keys.insert(key.to_string());
        self.save_keys()?;

        Ok(())
    }

    /// Get a secret from the keyring
    pub fn get_secret(&self, key: &str) -> Result<String, AppError> {
        let entry = keyring::Entry::new(&self.service_name, key)?;
        Ok(entry.get_password()?)
    }

    /// Delete a secret from the keyring
    pub fn delete_secret(&mut self, key: &str) -> Result<(), AppError> {
        let entry = keyring::Entry::new(&self.service_name, key)?;
        entry.delete_credential()?;

        // Remove key from known keys and persist
        self.known_keys.remove(key);
        self.save_keys()?;

        Ok(())
    }

    /// List all known secret keys
    pub fn list_keys(&self) -> Vec<String> {
        self.known_keys.iter().cloned().collect()
    }

    /// Remove a key from the known keys list (used for cleanup of stale entries)
    pub fn remove_key(&mut self, key: &str) -> Result<(), AppError> {
        self.known_keys.remove(key);
        self.save_keys()?;
        Ok(())
    }

    /// Save the known keys list to disk
    fn save_keys(&self) -> Result<(), AppError> {
        let json = serde_json::to_string_pretty(&self.known_keys)
            .map_err(|e| AppError::Io(e.to_string()))?;
        fs::write(&self.keys_file_path, json)?;
        Ok(())
    }
}
