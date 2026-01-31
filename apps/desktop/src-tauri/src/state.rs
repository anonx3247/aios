/// Application state managed by Tauri
/// Will be wrapped in Mutex when added to Tauri's managed state
/// Future plans will add secrets and MCP server process tracking
#[derive(Default)]
pub struct AppState {
    // Placeholder - secrets and MCP fields will be added by plans 02/03
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
