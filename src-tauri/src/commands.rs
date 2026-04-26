//! Tauri Commands

use tauri::State;
use crate::state::SharedAppState;
use crate::models::*;
use crate::services::CleanupService;

/// Статус приложения
#[tauri::command]
pub async fn get_status(state: State<'_, SharedAppState>) -> Result<AppStatusResponse, String> {
    let app_state = state.read().await;
    Ok(AppStatusResponse {
        status: app_state.status.clone(),
        timestamp: chrono::Utc::now(),
    })
}

/// Сканирование системы — возвращает категории с размерами
#[tauri::command]
pub async fn scan_system(state: State<'_, SharedAppState>) -> Result<ScanResponse, String> {
    CleanupService::scan_system(state).await
}

/// Очистка выбранных категорий из результатов сканирования
#[tauri::command]
pub async fn clean_scan_results(
    state: State<'_, SharedAppState>,
    params: ScanCleanParams,
) -> Result<ScanCleanResponse, String> {
    CleanupService::clean_scan_results(state, params).await
}

/// GET /api/logs
#[tauri::command]
pub async fn get_logs(state: State<'_, SharedAppState>, lines: Option<usize>) -> Result<LogsResponse, String> {
    let app_state = state.read().await;
    Ok(LogsResponse { logs: app_state.get_logs(lines.unwrap_or(50)) })
}

/// POST /api/logs/clear
#[tauri::command]
pub async fn clear_logs(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
    let mut app_state = state.write().await;
    app_state.clear_logs();
    Ok(ApiResponse { success: true, message: "Логи очищены".to_string(), exists: None, data: None })
}

/// Чистка строк (USN Journal)
#[tauri::command]
pub async fn clean_strings(state: State<'_, SharedAppState>) -> Result<CleanStringsResponse, String> {
    CleanupService::clean_strings(state).await
}

/// Очистка следов
#[tauri::command]
pub async fn clean_tracks(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
    CleanupService::clean_tracks(state).await
}

/// Симуляция открытия папок
#[tauri::command]
pub async fn simulate_folders(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
    CleanupService::simulate_folders(state).await
}

/// Очистка памяти javaw.exe
#[tauri::command]
pub async fn clean_javaw_memory(state: State<'_, SharedAppState>) -> Result<CleanJavawResult, String> {
    #[cfg(windows)]
    { CleanupService::clean_javaw_memory(state).await }
    #[cfg(not(windows))]
    {
        let _ = state;
        Ok(CleanJavawResult { success: false, message: "Только Windows".to_string(), regions_scanned: 0, regions_matched: 0, cleared_count: 0 })
    }
}

/// Статус инструментов
#[tauri::command]
pub async fn get_tools_status(state: State<'_, SharedAppState>) -> Result<ToolsStatusResponse, String> {
    let app_state = state.read().await;
    Ok(ToolsStatusResponse { tools: app_state.tool_states.clone() })
}

/// Опции глобальной очистки
#[tauri::command]
pub fn get_global_clean_options() -> Result<GlobalCleanOptionsResponse, String> {
    let mut options = std::collections::HashMap::new();
    options.insert("event_logs".to_string(), CleanOption { name: "Очистка Event Log".to_string(), description: "Security, System, Application".to_string() });
    options.insert("mft".to_string(), CleanOption { name: "Очистка $MFT".to_string(), description: "Prefetch + Master File Table".to_string() });
    options.insert("amcache".to_string(), CleanOption { name: "Очистка Amcache".to_string(), description: "Следы запуска программ".to_string() });
    options.insert("jump_lists".to_string(), CleanOption { name: "Очистка Jump Lists".to_string(), description: "Последние документы".to_string() });
    options.insert("recent_files".to_string(), CleanOption { name: "Очистка Recent Files".to_string(), description: "История открытых файлов".to_string() });
    options.insert("browser_history".to_string(), CleanOption { name: "Очистка Browser History".to_string(), description: "Chrome, Firefox, Edge".to_string() });
    options.insert("usn_journal".to_string(), CleanOption { name: "Очистка USN Journal".to_string(), description: "Удаление и пересоздание".to_string() });
    options.insert("temp_files".to_string(), CleanOption { name: "Очистка Temp Files".to_string(), description: "Временные файлы системы".to_string() });
    Ok(GlobalCleanOptionsResponse { options })
}

/// Глобальная очистка
#[tauri::command]
pub async fn run_global_clean(state: State<'_, SharedAppState>, params: GlobalCleanParams) -> Result<GlobalCleanResponse, String> {
    CleanupService::run_global_clean(state, params).await
}

// ── Сеть ──

#[tauri::command]
pub async fn flush_dns(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
    CleanupService::flush_dns(state).await
}

#[tauri::command]
pub async fn reset_network(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
    CleanupService::reset_network(state).await
}

#[tauri::command]
pub async fn clear_arp(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
    CleanupService::clear_arp(state).await
}

#[tauri::command]
pub async fn clear_netbios(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
    CleanupService::clear_netbios(state).await
}

// ── Система ──

#[tauri::command]
pub async fn clean_registry(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
    CleanupService::clean_registry(state).await
}

#[tauri::command]
pub async fn clean_dumps(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
    CleanupService::clean_dumps(state).await
}

#[tauri::command]
pub async fn clean_update_cache(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
    CleanupService::clean_update_cache(state).await
}

#[tauri::command]
pub async fn clean_thumbnails(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
    CleanupService::clean_thumbnails(state).await
}

// ── Приватность ──

#[tauri::command]
pub async fn clear_clipboard(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
    CleanupService::clear_clipboard(state).await
}

#[tauri::command]
pub async fn clean_icon_cache(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
    CleanupService::clean_icon_cache(state).await
}

#[tauri::command]
pub async fn clean_search_history(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
    CleanupService::clean_search_history(state).await
}

#[tauri::command]
pub async fn clean_run_history(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
    CleanupService::clean_run_history(state).await
}
