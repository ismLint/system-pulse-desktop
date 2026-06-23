mod commands;
mod models;
mod services;

use tauri::Manager;
use tracing::info;

use commands::{account, auth, metrics, servers, window};
use system_pulse_db::{Database, DatabaseConfig};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("system_pulse_desktop_lib=debug,info")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                // The desktop app stores its SQLite file under the OS-managed
                // app-data directory. The exact same `system-pulse-db` crate
                // (models, migrations, queries) is reused by the standalone
                // server, which instead reads `DatabaseConfig::from_env()`
                // pointing at a Docker volume — see crates/system-pulse-server.
                let data_dir = app_handle
                    .path()
                    .app_data_dir()
                    .expect("failed to resolve app data dir");
                let db_path = data_dir.join("system_pulse.db");

                let config = DatabaseConfig::at_path(db_path);
                let db = Database::connect(config)
                    .await
                    .expect("failed to initialize database");

                app_handle.manage(db);
                info!("database initialized");
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // window
            window::minimize_window,
            window::maximize_window,
            window::close_window,
            // auth
            auth::login,
            auth::register,
            auth::logout,
            auth::get_me,
            // account
            account::change_password,
            account::change_email,
            account::change_login,
            // servers
            servers::list_servers,
            servers::get_server,
            servers::create_server,
            servers::update_server,
            servers::delete_server,
            servers::test_connection,
            // metrics
            metrics::get_metrics,
            metrics::get_latest_metric,
            metrics::collect_now,
            metrics::start_polling,
            metrics::stop_polling,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
