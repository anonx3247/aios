#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::{Manager, menu::{Menu, MenuItem}};
    use tauri::tray::{TrayIconBuilder, TrayIconEvent};

    #[cfg(target_os = "macos")]
    use tauri::ActivationPolicy;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(
                    "sqlite:aios.db",
                    vec![tauri_plugin_sql::Migration {
                        version: 1,
                        description: "create_initial_tables",
                        sql: r#"
                            CREATE TABLE IF NOT EXISTS runs (
                                id TEXT PRIMARY KEY,
                                task TEXT NOT NULL,
                                status TEXT NOT NULL DEFAULT 'pending',
                                started_at TEXT NOT NULL,
                                completed_at TEXT,
                                error TEXT
                            );
                            CREATE TABLE IF NOT EXISTS messages (
                                id INTEGER PRIMARY KEY AUTOINCREMENT,
                                run_id TEXT NOT NULL,
                                role TEXT NOT NULL,
                                content TEXT NOT NULL,
                                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                                FOREIGN KEY(run_id) REFERENCES runs(id)
                            );
                        "#,
                        kind: tauri_plugin_sql::MigrationKind::Up,
                    }],
                )
                .build(),
        )
        .setup(|app| {
            use std::sync::Mutex;
            use crate::services::keyring_service::KeyringService;
            use crate::services::mcp_manager::McpManager;
            use crate::services::config_loader;
            use crate::state::AppState;

            // Set activation policy on macOS to hide from dock (tray-only)
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            // Initialize KeyringService with app data directory
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            let keyring_service = KeyringService::new("com.aios.secrets", app_data_dir.clone())
                .expect("Failed to initialize KeyringService");

            // Load MCP configuration
            let mcp_config_path = config_loader::default_config_path(&app_data_dir);
            let mcp_config = config_loader::load(&mcp_config_path)
                .expect("Failed to load MCP configuration");

            // Initialize McpManager
            let mcp_manager = McpManager::new(mcp_config);

            // Initialize and manage AppState
            let app_state = AppState::new(keyring_service, mcp_manager);
            app.manage(Mutex::new(app_state));

            // Note: MCP processes are automatically cleaned up via kill_on_drop(true)
            // when the app exits. No explicit shutdown hook needed.

            // Create system tray menu
            let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            // Build tray icon with menu
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().expect("no app icon"))
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // Toggle window visibility on tray icon click
                    if let TrayIconEvent::Click { .. } = event {
                        if let Some(app) = tray.app_handle().get_webview_window("main") {
                            if app.is_visible().unwrap_or(false) {
                                let _ = app.hide();
                            } else {
                                let _ = app.show();
                                let _ = app.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health::health_check,
            commands::secrets::get_secrets,
            commands::secrets::get_secret,
            commands::secrets::set_secret,
            commands::secrets::delete_secret,
            commands::mcp::start_mcp_server,
            commands::mcp::stop_mcp_server,
            commands::mcp::list_mcp_servers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Module declarations
mod commands;
mod types;
mod state;
mod services;
