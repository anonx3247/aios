use crate::types::config::{McpConfig, McpServerConfig, McpServerStatus};
use crate::types::errors::AppError;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;

/// Process handle with metadata
struct ManagedProcess {
    child: Child,
    retry_count: u32,
}

/// Manager for MCP server processes
pub struct McpManager {
    config: McpConfig,
    processes: RwLock<HashMap<String, ManagedProcess>>,
    statuses: RwLock<HashMap<String, McpServerStatus>>,
}

impl McpManager {
    /// Create a new McpManager with the given configuration
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            processes: RwLock::new(HashMap::new()),
            statuses: RwLock::new(HashMap::new()),
        }
    }

    /// Start an MCP server with the given name and optional environment overrides
    pub async fn start_server(
        &self,
        name: &str,
        env_overrides: HashMap<String, String>,
    ) -> Result<(), AppError> {
        // Check if server is already running
        {
            let statuses = self.statuses.read().await;
            if let Some(McpServerStatus::Running) = statuses.get(name) {
                return Ok(());
            }
        }

        // Get server config
        let server_config = self
            .config
            .mcp_servers
            .get(name)
            .ok_or_else(|| AppError::NotFound(format!("MCP server '{}' not found", name)))?
            .clone();

        // Update status to Starting
        {
            let mut statuses = self.statuses.write().await;
            statuses.insert(name.to_string(), McpServerStatus::Starting);
        }

        // Attempt to start with retry logic
        match self
            .start_with_retry(name, &server_config, env_overrides)
            .await
        {
            Ok(child) => {
                let mut processes = self.processes.write().await;
                processes.insert(
                    name.to_string(),
                    ManagedProcess {
                        child,
                        retry_count: 0,
                    },
                );

                let mut statuses = self.statuses.write().await;
                statuses.insert(name.to_string(), McpServerStatus::Running);

                Ok(())
            }
            Err(e) => {
                let mut statuses = self.statuses.write().await;
                statuses.insert(name.to_string(), McpServerStatus::Failed(e.to_string()));
                Err(e)
            }
        }
    }

    /// Start a server with exponential backoff retry (max 3 attempts)
    async fn start_with_retry(
        &self,
        name: &str,
        config: &McpServerConfig,
        env_overrides: HashMap<String, String>,
    ) -> Result<Child, AppError> {
        let mut backoff = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(30)),
            max_interval: Duration::from_secs(10),
            ..Default::default()
        };

        let mut attempts = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            attempts += 1;

            match self.spawn_process(config, &env_overrides).await {
                Ok(child) => {
                    if attempts > 1 {
                        eprintln!(
                            "MCP server '{}' started successfully after {} attempts",
                            name, attempts
                        );
                    }
                    return Ok(child);
                }
                Err(e) => {
                    if attempts >= MAX_RETRIES {
                        return Err(AppError::Process(format!(
                            "Failed to start MCP server '{}' after {} attempts: {}",
                            name, attempts, e
                        )));
                    }

                    eprintln!(
                        "Failed to start MCP server '{}' (attempt {}/{}): {}",
                        name, attempts, MAX_RETRIES, e
                    );

                    // Calculate backoff delay
                    if let Some(delay) = backoff.next_backoff() {
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(AppError::Process(format!(
                            "Backoff timeout starting MCP server '{}'",
                            name
                        )));
                    }
                }
            }
        }
    }

    /// Spawn an MCP server process with the given configuration
    async fn spawn_process(
        &self,
        config: &McpServerConfig,
        env_overrides: &HashMap<String, String>,
    ) -> Result<Child, AppError> {
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);

        // Set environment variables from config
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Apply environment overrides (e.g., from keyring)
        for (key, value) in env_overrides {
            cmd.env(key, value);
        }

        // Configure stdio
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set kill_on_drop to ensure cleanup
        cmd.kill_on_drop(true);

        // Spawn the process
        let child = cmd
            .spawn()
            .map_err(|e| AppError::Process(format!("Failed to spawn process: {}", e)))?;

        Ok(child)
    }

    /// Stop an MCP server
    pub async fn stop_server(&self, name: &str) -> Result<(), AppError> {
        let mut processes = self.processes.write().await;

        if let Some(mut managed) = processes.remove(name) {
            // Kill the process
            managed
                .child
                .kill()
                .await
                .map_err(|e| AppError::Process(format!("Failed to kill process: {}", e)))?;

            // Update status
            let mut statuses = self.statuses.write().await;
            statuses.insert(name.to_string(), McpServerStatus::Stopped);

            Ok(())
        } else {
            Err(AppError::NotFound(format!(
                "MCP server '{}' is not running",
                name
            )))
        }
    }

    /// List all MCP servers with their current status
    pub async fn list_servers(&self) -> HashMap<String, McpServerStatus> {
        let statuses = self.statuses.read().await;
        let mut result = HashMap::new();

        // Include all configured servers
        for name in self.config.mcp_servers.keys() {
            let status = statuses
                .get(name)
                .cloned()
                .unwrap_or(McpServerStatus::Stopped);
            result.insert(name.clone(), status);
        }

        result
    }

    /// Stop all running MCP servers
    pub async fn stop_all(&self) -> Result<(), AppError> {
        let mut processes = self.processes.write().await;
        let mut statuses = self.statuses.write().await;

        for (name, mut managed) in processes.drain() {
            if let Err(e) = managed.child.kill().await {
                eprintln!("Failed to kill MCP server '{}': {}", name, e);
            }
            statuses.insert(name, McpServerStatus::Stopped);
        }

        Ok(())
    }
}

impl Drop for McpManager {
    fn drop(&mut self) {
        // Processes will be killed automatically due to kill_on_drop(true)
        // This ensures no zombie processes on app exit
    }
}
