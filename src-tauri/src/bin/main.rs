//! EliteCleaner — 1337 Cleaner. Main Entry Point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;
use elite_cleaner_lib::state::AppState;

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();
    
    info!("EliteCleaner starting...");
    
    let app_state = Arc::new(RwLock::new(AppState::new()));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            elite_cleaner_lib::commands::get_status,
            elite_cleaner_lib::commands::scan_system,
            elite_cleaner_lib::commands::clean_scan_results,
            elite_cleaner_lib::commands::get_logs,
            elite_cleaner_lib::commands::clear_logs,
            elite_cleaner_lib::commands::clean_strings,
            elite_cleaner_lib::commands::clean_tracks,
            elite_cleaner_lib::commands::simulate_folders,
            elite_cleaner_lib::commands::clean_javaw_memory,
            elite_cleaner_lib::commands::get_tools_status,
            elite_cleaner_lib::commands::get_global_clean_options,
            elite_cleaner_lib::commands::run_global_clean,
            // сеть
            elite_cleaner_lib::commands::flush_dns,
            elite_cleaner_lib::commands::reset_network,
            elite_cleaner_lib::commands::clear_arp,
            elite_cleaner_lib::commands::clear_netbios,
            // система
            elite_cleaner_lib::commands::clean_registry,
            elite_cleaner_lib::commands::clean_dumps,
            elite_cleaner_lib::commands::clean_update_cache,
            elite_cleaner_lib::commands::clean_thumbnails,
            // приватность
            elite_cleaner_lib::commands::clear_clipboard,
            elite_cleaner_lib::commands::clean_icon_cache,
            elite_cleaner_lib::commands::clean_search_history,
            elite_cleaner_lib::commands::clean_run_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
