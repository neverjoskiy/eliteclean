//! Tauri v2 Main Entry Point
//! Миграция с Python/FastAPI на Rust/Tauri

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;
use noxum_launcher_lib::state::AppState;

fn main() {
    // Инициализация логирования
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();
    
    info!("Noxum Launcher starting...");
    
    // Создаём глобальное состояние
    let app_state = AppState::new();
    
    // Запускаем Tauri приложение
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            noxum_launcher_lib::commands::get_status,
            noxum_launcher_lib::commands::launch_app,
            noxum_launcher_lib::commands::get_logs,
            noxum_launcher_lib::commands::clear_logs,
            noxum_launcher_lib::commands::clean_strings,
            noxum_launcher_lib::commands::clean_tracks,
            noxum_launcher_lib::commands::simulate_folders,
            noxum_launcher_lib::commands::clean_javaw_memory,
            noxum_launcher_lib::commands::get_tools_status,
            noxum_launcher_lib::commands::get_global_clean_options,
            noxum_launcher_lib::commands::run_global_clean,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
