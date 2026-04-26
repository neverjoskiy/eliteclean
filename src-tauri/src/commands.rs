//! Tauri Commands - аналог FastAPI endpoints
//! Каждая функция экспортируется как команда для вызова из JavaScript через invoke()

use tauri::State;
use log::info;
use crate::state::SharedAppState;
use crate::models::*;
use crate::services::{LauncherService, CleanupService};

/// GET /api/status - Получить текущий статус приложения
#[tauri::command]
pub async fn get_status(state: State<'_, SharedAppState>) -> Result<StatusResponse, String> {
    let app_state = state.read().await;
    
    let target_path = crate::utils::get_target_jar_path();
    let file_exists = target_path.exists();
    let file_size = if file_exists {
        std::fs::metadata(&target_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    
    Ok(StatusResponse {
        status: app_state.status.clone(),
        launches: app_state.launch_history.len(),
        file_exists,
        file_size,
        file_path: target_path.to_string_lossy().to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// POST /api/launch - Запуск целевого приложения
#[tauri::command]
pub async fn launch_app(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
    let mut app_state = state.write().await;
    
    if app_state.status == AppStatus::Running {
        return Ok(ApiResponse {
            success: false,
            message: "Приложение уже запущено".to_string(),
            exists: None,
            data: None,
        });
    }
    
    app_state.add_log("Запрос на запуск приложения получен".to_string(), "info".to_string());
    drop(app_state); // Освобождаем lock перед запуском
    
    // Запускаем в отдельном потоке чтобы не блокировать UI
    let state_clone = state.inner().clone();
    
    tokio::spawn(async move {
        let result = LauncherService::launch_stealth(&state_clone).await;
        
        let mut app_state = state_clone.write().await;
        app_state.add_log(
            format!("Результат запуска: {}", if result.success { "успешно" } else { "неудачно" }),
            if result.success { "success" } else { "error" }.to_string(),
        );
        result
    });
    
    // Небольшая задержка перед возвратом
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(ApiResponse {
        success: true,
        message: "Процесс запуска инициирован".to_string(),
        exists: None,
        data: None,
    })
}

/// GET /api/logs - Получить логи
#[tauri::command]
pub async fn get_logs(state: State<'_, SharedAppState>, lines: Option<usize>) -> Result<LogsResponse, String> {
    let app_state = state.read().await;
    let logs = app_state.get_logs(lines.unwrap_or(50));
    
    Ok(LogsResponse { logs })
}

/// POST /api/logs/clear - Очистить логи
#[tauri::command]
pub async fn clear_logs(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
    let mut app_state = state.write().await;
    app_state.clear_logs();
    
    info!("Логи очищены");
    
    Ok(ApiResponse {
        success: true,
        message: "Логи очищены".to_string(),
        exists: None,
        data: None,
    })
}

/// POST /api/tools/clean-strings - Чистка строк (USN Journal)
#[tauri::command]
pub async fn clean_strings(state: State<'_, SharedAppState>) -> Result<CleanStringsResponse, String> {
    CleanupService::clean_strings(state).await
}

/// POST /api/tools/clean-tracks - Очистка следов
#[tauri::command]
pub async fn clean_tracks(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
    CleanupService::clean_tracks(state).await
}

/// POST /api/tools/simulate - Симуляция открытия папок
#[tauri::command]
pub async fn simulate_folders(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
    CleanupService::simulate_folders(state).await
}

/// POST /api/tools/clean-javaw - Очистка памяти javaw.exe
#[tauri::command]
pub async fn clean_javaw_memory(state: State<'_, SharedAppState>) -> Result<CleanJavawResult, String> {
    #[cfg(windows)]
    {
        CleanupService::clean_javaw_memory(state).await
    }
    #[cfg(not(windows))]
    {
        let _ = state;
        Ok(CleanJavawResult {
            success: false,
            message: "Функция доступна только на Windows".to_string(),
            regions_scanned: 0,
            regions_matched: 0,
            cleared_count: 0,
        })
    }
}

/// GET /api/tools/status - Получить статус инструментов
#[tauri::command]
pub async fn get_tools_status(state: State<'_, SharedAppState>) -> Result<ToolsStatusResponse, String> {
    let app_state = state.read().await;
    
    Ok(ToolsStatusResponse {
        tools: app_state.tool_states.clone(),
    })
}

/// GET /api/tools/global-clean/options - Получить опции глобальной очистки
#[tauri::command]
pub fn get_global_clean_options() -> Result<GlobalCleanOptionsResponse, String> {
    let mut options = std::collections::HashMap::new();
    
    options.insert("event_logs".to_string(), CleanOption {
        name: "Очистка Event Log".to_string(),
        description: "Удаление логов Windows (Security, System, Application)".to_string(),
    });
    options.insert("mft".to_string(), CleanOption {
        name: "Очистка $MFT".to_string(),
        description: "Сброс Master File Table (удаление Prefetch)".to_string(),
    });
    options.insert("amcache".to_string(), CleanOption {
        name: "Очистка Amcache".to_string(),
        description: "Удаление Amcache.hve (следы запуска программ)".to_string(),
    });
    options.insert("jump_lists".to_string(), CleanOption {
        name: "Очистка Jump Lists".to_string(),
        description: "Удаление последних документов и закреплённых файлов".to_string(),
    });
    options.insert("recent_files".to_string(), CleanOption {
        name: "Очистка Recent Files".to_string(),
        description: "История открытых файлов в Windows".to_string(),
    });
    options.insert("browser_history".to_string(), CleanOption {
        name: "Очистка Browser History".to_string(),
        description: "История браузеров (Chrome, Firefox, Edge)".to_string(),
    });
    options.insert("usn_journal".to_string(), CleanOption {
        name: "Очистка USN Journal".to_string(),
        description: "Удаление и пересоздание USN журнала".to_string(),
    });
    options.insert("temp_files".to_string(), CleanOption {
        name: "Очистка Temp Files".to_string(),
        description: "Удаление временных файлов системы".to_string(),
    });
    
    Ok(GlobalCleanOptionsResponse { options })
}

/// POST /api/tools/global-clean - Запустить глобальную очистку
#[tauri::command]
pub async fn run_global_clean(
    state: State<'_, SharedAppState>,
    params: GlobalCleanParams,
) -> Result<GlobalCleanResponse, String> {
    CleanupService::run_global_clean(state, params).await
}

// ── Сетевые команды ──

/// Сброс DNS кэша
#[tauri::command]
pub async fn flush_dns(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
    CleanupService::flush_dns(state).await
}

/// Сброс сетевых настроек (winsock, ip, firewall)
#[tauri::command]
pub async fn reset_network(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
    CleanupService::reset_network(state).await
}

/// Очистка ARP таблицы
#[tauri::command]
pub async fn clear_arp(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
    CleanupService::clear_arp(state).await
}

/// Очистка NetBIOS кэша
#[tauri::command]
pub async fn clear_netbios(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
    CleanupService::clear_netbios(state).await
}

// ── Системные команды ──

/// Очистка реестра (Run, RunOnce, MRU)
#[tauri::command]
pub async fn clean_registry(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
    CleanupService::clean_registry(state).await
}

/// Очистка дампов памяти
#[tauri::command]
pub async fn clean_dumps(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
    CleanupService::clean_dumps(state).await
}

/// Очистка Windows Update кэша
#[tauri::command]
pub async fn clean_update_cache(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
    CleanupService::clean_update_cache(state).await
}

/// Очистка thumbnail кэша
#[tauri::command]
pub async fn clean_thumbnails(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
    CleanupService::clean_thumbnails(state).await
}

// ── Приватность ──

/// Очистка буфера обмена
#[tauri::command]
pub async fn clear_clipboard(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
    CleanupService::clear_clipboard(state).await
}

/// Очистка иконок (IconCache)
#[tauri::command]
pub async fn clean_icon_cache(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
    CleanupService::clean_icon_cache(state).await
}

/// Очистка поиска Windows (search history)
#[tauri::command]
pub async fn clean_search_history(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
    CleanupService::clean_search_history(state).await
}

/// Очистка списка запуска (Run dialog history)
#[tauri::command]
pub async fn clean_run_history(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
    CleanupService::clean_run_history(state).await
}
