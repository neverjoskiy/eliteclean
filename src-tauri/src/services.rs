//! Сервисы: бизнес-логика приложения
//! Аналог функций из main.py (launch_stealth, clean_javaw_memory, etc.)

use log::{info, warn, error};
use std::time::Duration;
use tauri::State;
use tokio::time::sleep;
use crate::state::SharedAppState;
use crate::models::*;

/// Сервис запуска приложений
pub struct LauncherService;

impl LauncherService {
    /// Запустить целевое приложение в скрытом режиме с эмуляцией Steam окружения
    /// Аналог launch_stealth() из Python
    pub async fn launch_stealth(state: &SharedAppState) -> ApiResponse {
        const DOWNLOAD_URL: &str = "https://github.com/neverjoskiy/nebula/releases/download/1234123/Microsoft.Ink.dll";
        
        // Проверяем наличие файла и скачиваем если нужно
        let target_path = crate::utils::get_target_jar_path();
        
        if !target_path.exists() {
            info!("Файл не найден, начинаем загрузку: {:?}", target_path);
            
            {
                let mut app_state = state.write().await;
                app_state.add_log("Файл Microsoft.Ink.dll не найден. Загрузка...".to_string(), "warning".to_string());
            }
            
            let download_result = Self::download_target_file(DOWNLOAD_URL).await;
            
            if !download_result.success {
                return ApiResponse {
                    success: false,
                    message: download_result.message,
                    exists: None,
                    data: None,
                };
            }
        }
        
        // Формируем команду для запуска Java
        let java_cmd = "java";
        let args = vec![
            "-Xms128M".to_string(),
            "-Xmx512M".to_string(),
            "-jar".to_string(),
            target_path.to_string_lossy().to_string(),
            "-steam".to_string(),
            "-silent".to_string(),
        ];
        
        // Переменные окружения Steam
        let steam_env = vec![
            ("SteamAppId", "220"),
            ("SteamGameId", "220"),
            ("SteamUser", "User"),
        ];
        
        {
            let mut app_state = state.write().await;
            app_state.status = AppStatus::Running;
            app_state.add_log("Запуск приложения...".to_string(), "info".to_string());
        }
        
        info!("Запуск приложения: {} {:?}", java_cmd, args);
        
        // Запускаем процесс
        let result = Self::run_process_hidden(java_cmd, &args, &steam_env);
        
        // После завершения удаляем файл
        if target_path.exists() {
            match std::fs::remove_file(&target_path) {
                Ok(_) => {
                    info!("Файл удален: {:?}", target_path);
                    let mut app_state = state.write().await;
                    app_state.add_log("Файл Microsoft.Ink.dll удален".to_string(), "success".to_string());
                }
                Err(e) => {
                    warn!("Файл занят другим процессом, не удалось удалить: {:?}: {}", target_path, e);
                    let mut app_state = state.write().await;
                    app_state.add_log("Файл занят, удаление невозможно".to_string(), "warning".to_string());
                }
            }
        }
        
        // Добавляем в историю запусков
        {
            let mut app_state = state.write().await;
            app_state.launch_history.push(crate::state::LaunchRecord {
                timestamp: chrono::Utc::now(),
                status: if result.success { "success" } else { "error" }.to_string(),
            });
            
            // Оставляем только последние 10 записей
            if app_state.launch_history.len() > 10 {
                app_state.launch_history.remove(0);
            }
            
            app_state.status = AppStatus::Ready;
        }
        
        if result.success {
            ApiResponse {
                success: true,
                message: "Приложение запущено и завершено, файл удален".to_string(),
                exists: None,
                data: None,
            }
        } else {
            ApiResponse {
                success: false,
                message: result.message,
                exists: None,
                data: None,
            }
        }
    }
    
    /// Скачать целевой файл (async — не использует blocking внутри tokio runtime)
    async fn download_target_file(url: &str) -> DownloadResult {
        info!("Загрузка файла из {}", url);

        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .danger_accept_invalid_certs(true)
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                error!("Ошибка создания HTTP клиента: {}", e);
                return DownloadResult {
                    success: false,
                    message: format!("Ошибка создания HTTP клиента: {}", e),
                    regions_scanned: None,
                    regions_matched: None,
                    cleared_count: None,
                };
            }
        };

        let response = match client
            .get(url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                error!("Ошибка загрузки: {}", e);
                return DownloadResult {
                    success: false,
                    message: format!("Ошибка загрузки: {}", e),
                    regions_scanned: None,
                    regions_matched: None,
                    cleared_count: None,
                };
            }
        };

        let target_path = crate::utils::get_target_jar_path();

        match response.bytes().await {
            Ok(bytes) => {
                match std::fs::write(&target_path, &bytes) {
                    Ok(_) => {
                        if target_path.exists()
                            && target_path.metadata().map(|m| m.len()).unwrap_or(0) > 0
                        {
                            info!("Файл успешно загружен: {:?}", target_path);
                            DownloadResult {
                                success: true,
                                message: "Файл успешно загружен".to_string(),
                                regions_scanned: None,
                                regions_matched: None,
                                cleared_count: None,
                            }
                        } else {
                            error!("Файл загрузился пустым или поврежден");
                            DownloadResult {
                                success: false,
                                message: "Файл загрузился пустым или поврежден".to_string(),
                                regions_scanned: None,
                                regions_matched: None,
                                cleared_count: None,
                            }
                        }
                    }
                    Err(e) => {
                        error!("Ошибка записи файла: {}", e);
                        DownloadResult {
                            success: false,
                            message: format!("Ошибка записи файла: {}", e),
                            regions_scanned: None,
                            regions_matched: None,
                            cleared_count: None,
                        }
                    }
                }
            }
            Err(e) => {
                error!("Ошибка чтения ответа: {}", e);
                DownloadResult {
                    success: false,
                    message: format!("Ошибка чтения ответа: {}", e),
                    regions_scanned: None,
                    regions_matched: None,
                    cleared_count: None,
                }
            }
        }
    }
    
    /// Запустить процесс скрыто
    fn run_process_hidden(cmd: &str, args: &[String], env: &[(&str, &str)]) -> ApiResponse {
        use std::process::Command;
        
        let mut command = Command::new(cmd);
        command.args(args);
        
        for (key, value) in env {
            command.env(key, value);
        }
        
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            const DETACHED_PROCESS: u32 = 0x00000008;
            command.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
            command.stdout(std::process::Stdio::null());
            command.stderr(std::process::Stdio::null());
        }
        
        #[cfg(not(windows))]
        {
            command.stdout(std::process::Stdio::null());
            command.stderr(std::process::Stdio::null());
        }
        
        match command.spawn() {
            Ok(mut child) => {
                info!("Ожидание завершения процесса PID: {}", child.id());
                
                match child.wait() {
                    Ok(status) => {
                        info!("Процесс завершен со статусом: {:?}", status);
                        ApiResponse {
                            success: true,
                            message: "Процесс завершен успешно".to_string(),
                            exists: None,
                            data: None,
                        }
                    }
                    Err(e) => {
                        error!("Ошибка ожидания процесса: {}", e);
                        ApiResponse {
                            success: false,
                            message: format!("Ошибка ожидания процесса: {}", e),
                            exists: None,
                            data: None,
                        }
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    error!("Java не найдена. Убедитесь, что Java установлена и доступна в PATH");
                    ApiResponse {
                        success: false,
                        message: "Java не найдена. Убедитесь, что Java установлена и доступна в PATH".to_string(),
                        exists: None,
                        data: None,
                    }
                } else {
                    error!("Ошибка запуска: {}", e);
                    ApiResponse {
                        success: false,
                        message: format!("Ошибка запуска: {}", e),
                        exists: None,
                        data: None,
                    }
                }
            }
        }
    }
}

/// Сервис очистки системы
pub struct CleanupService;

impl CleanupService {
    /// Чистка строк (USN Journal)
    pub async fn clean_strings(state: State<'_, SharedAppState>) -> Result<CleanStringsResponse, String> {
        
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("clean_strings", true, 10, "running");
            app_state.add_log("Запуск чистки строк".to_string(), "info".to_string());
        }
        
        // Шаг 1: Удаление журнала USN
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("clean_strings", true, 30, "running");
            app_state.add_log("Выполнение вирус.bat...".to_string(), "info".to_string());
        }
        
        let scripts_dir = crate::utils::get_scripts_dir();
        let virus_bat = scripts_dir.join("вирус.bat");
        
        let result1 = Self::run_batch_file(&virus_bat);
        
        if !result1.success {
            let mut app_state = state.write().await;
            app_state.update_tool_state("clean_strings", false, 0, "error");
            app_state.add_log(format!("Ошибка на шаге 1: {}", result1.message), "error".to_string());
            
            return Ok(CleanStringsResponse {
                success: false,
                message: format!("Шаг 1 (удаление журнала USN): {}", result1.message),
                steps: vec![],
            });
        }
        
        {
            let mut app_state = state.write().await;
            app_state.add_log("Шаг 1 выполнен успешно".to_string(), "success".to_string());
            app_state.update_tool_state("clean_strings", true, 60, "running");
        }
        
        sleep(Duration::from_secs(2)).await;
        
        // Шаг 2: Создание журнала USN
        {
            let mut app_state = state.write().await;
            app_state.add_log("Выполнение не вирус.bat...".to_string(), "info".to_string());
            app_state.update_tool_state("clean_strings", true, 80, "running");
        }
        
        let not_virus_bat = scripts_dir.join("не вирус.bat");
        let result2 = Self::run_batch_file(&not_virus_bat);
        
        if !result2.success {
            let mut app_state = state.write().await;
            app_state.update_tool_state("clean_strings", false, 0, "error");
            app_state.add_log(format!("Ошибка на шаге 2: {}", result2.message), "error".to_string());
            
            return Ok(CleanStringsResponse {
                success: false,
                message: format!("Шаг 2 (создание журнала USN): {}", result2.message),
                steps: vec![],
            });
        }
        
        {
            let mut app_state = state.write().await;
            app_state.add_log("Шаг 2 выполнен успешно".to_string(), "success".to_string());
            app_state.update_tool_state("clean_strings", false, 100, "completed");
            app_state.add_log("Чистка строк завершена".to_string(), "success".to_string());
        }
        
        Ok(CleanStringsResponse {
            success: true,
            message: "Чистка строк успешно завершена".to_string(),
            steps: vec![
                CleanStep { name: "Удаление журнала USN".to_string(), status: "completed".to_string() },
                CleanStep { name: "Создание журнала USN".to_string(), status: "completed".to_string() },
            ],
        })
    }
    
    /// Очистка следов
    pub async fn clean_tracks(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
        
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("clean_tracks", true, 10, "running");
            app_state.add_log("Запуск очистки следов".to_string(), "info".to_string());
        }
        
        let scripts_dir = crate::utils::get_scripts_dir();
        let winlocker_bat = scripts_dir.join("винлокер.bat");
        
        if !winlocker_bat.exists() {
            let mut app_state = state.write().await;
            app_state.update_tool_state("clean_tracks", false, 0, "error");
            return Ok(ApiResponse {
                success: false,
                message: format!("Файл не найден: {:?}", winlocker_bat),
                exists: None,
                data: None,
            });
        }
        
        {
            let mut app_state = state.write().await;
            app_state.add_log("Запуск винлокер.bat (требуются права администратора)...".to_string(), "warning".to_string());
            app_state.update_tool_state("clean_tracks", true, 30, "running");
        }
        
        let result = Self::run_batch_file_as_admin(&winlocker_bat);
        
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("clean_tracks", false, 100, "completed");
        }
        
        if result.success {
            let mut app_state = state.write().await;
            app_state.add_log("Очистка следов завершена".to_string(), "success".to_string());
        }
        
        Ok(result)
    }
    
    /// Симуляция открытия папок
    pub async fn simulate_folders(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("simulate", true, 50, "running");
            app_state.add_log("Запуск симуляции открытия папок".to_string(), "info".to_string());
        }
        
        let scripts_dir = crate::utils::get_scripts_dir();
        let simulate_exe = scripts_dir.join("simulate.exe");
        
        let result = Self::run_executable(&simulate_exe);
        
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("simulate", false, 100, 
                if result.success { "completed" } else { "error" });
        }
        
        if result.success {
            let mut app_state = state.write().await;
            app_state.add_log("Симуляция запущена".to_string(), "success".to_string());
        } else {
            let mut app_state = state.write().await;
            app_state.add_log(format!("Ошибка симуляции: {}", result.message), "error".to_string());
        }
        
        Ok(result)
    }
    
    /// Очистка памяти javaw.exe (Windows only)
    #[cfg(windows)]
    pub async fn clean_javaw_memory(state: State<'_, SharedAppState>) -> Result<CleanJavawResult, String> {
        use crate::memory::MemoryCleaner;
        
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("clean_javaw", true, 10, "running");
            app_state.add_log("Запуск очистки памяти javaw.exe".to_string(), "info".to_string());
        }
        
        let result = tokio::task::spawn_blocking(|| {
            MemoryCleaner::clean_javaw_memory()
        }).await
        .map_err(|e| e.to_string())?;
        
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("clean_javaw", false, 100, 
                if result.success { "completed" } else { "error" });
        }
        
        if result.success {
            let mut app_state = state.write().await;
            app_state.add_log("Очистка памяти javaw.exe завершена".to_string(), "success".to_string());
        } else {
            let mut app_state = state.write().await;
            app_state.add_log("Ошибка при очистке памяти javaw.exe".to_string(), "error".to_string());
        }
        
        Ok(result)
    }
    
    #[cfg(not(windows))]
    pub async fn clean_javaw_memory(_state: State<'_, SharedAppState>) -> Result<CleanJavawResult, String> {
        Ok(CleanJavawResult {
            success: false,
            message: "Функция доступна только на Windows".to_string(),
            regions_scanned: 0,
            regions_matched: 0,
            cleared_count: 0,
        })
    }
    
    /// Глобальная очистка
    pub async fn run_global_clean(
        state: State<'_, SharedAppState>,
        params: GlobalCleanParams,
    ) -> Result<GlobalCleanResponse, String> {
        
        // Собираем выбранные опции
        let mut selected = Vec::new();
        if params.event_logs.unwrap_or(false) { selected.push("event_logs"); }
        if params.mft.unwrap_or(false) { selected.push("mft"); }
        if params.amcache.unwrap_or(false) { selected.push("amcache"); }
        if params.jump_lists.unwrap_or(false) { selected.push("jump_lists"); }
        if params.recent_files.unwrap_or(false) { selected.push("recent_files"); }
        if params.browser_history.unwrap_or(false) { selected.push("browser_history"); }
        if params.usn_journal.unwrap_or(false) { selected.push("usn_journal"); }
        if params.temp_files.unwrap_or(false) { selected.push("temp_files"); }
        
        let total = selected.len();
        
        if total == 0 {
            let mut app_state = state.write().await;
            app_state.update_tool_state("global_clean", false, 0, "error");
            return Ok(GlobalCleanResponse {
                success: false,
                message: "Не выбрано ни одной опции".to_string(),
                results: std::collections::HashMap::new(),
                total: 0,
                completed: 0,
            });
        }
        
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("global_clean", true, 0, "running");
            app_state.add_log("Запуск глобальной очистки".to_string(), "info".to_string());
        }
        
        let mut results = std::collections::HashMap::new();
        let mut completed = 0;
        
        for (i, option_key) in selected.iter().enumerate() {
            let option_name = match *option_key {
                "event_logs" => "Очистка Event Log",
                "mft" => "Очистка $MFT",
                "amcache" => "Очистка Amcache",
                "jump_lists" => "Очистка Jump Lists",
                "recent_files" => "Очистка Recent Files",
                "browser_history" => "Очистка Browser History",
                "usn_journal" => "Очистка USN Journal",
                "temp_files" => "Очистка Temp Files",
                _ => "Неизвестная опция",
            };
            
            {
                let mut app_state = state.write().await;
                let progress = ((i as f32 / total as f32) * 100.0) as u8;
                app_state.update_tool_state("global_clean", true, progress, "running");
                app_state.add_log(format!("Очистка: {}...", option_name), "info".to_string());
            }
            
            let result = match *option_key {
                "event_logs" => Self::clean_event_logs(),
                "mft" => Self::clean_mft(),
                "amcache" => Self::clean_amcache(),
                "jump_lists" => Self::clean_jump_lists(),
                "recent_files" => Self::clean_recent_files(),
                "browser_history" => Self::clean_browser_history(),
                "usn_journal" => Self::clean_usn_journal(),
                "temp_files" => Self::clean_temp_files(),
                _ => GlobalCleanResultItem { success: false, message: "Неизвестная опция".to_string() },
            };
            
            if result.success {
                completed += 1;
                let mut app_state = state.write().await;
                app_state.add_log(format!("✓ {}: {}", option_name, result.message), "success".to_string());
            } else {
                let mut app_state = state.write().await;
                app_state.add_log(format!("✗ {}: {}", option_name, result.message), "error".to_string());
            }
            
            results.insert(option_key.to_string(), result);
            
            sleep(Duration::from_millis(500)).await;
        }
        
        {
            let mut app_state = state.write().await;
            app_state.update_tool_state("global_clean", false, 100, "completed");
            app_state.add_log(format!("Глобальная очистка завершена: {}/{} успешно", completed, total), "success".to_string());
        }
        
        Ok(GlobalCleanResponse {
            success: true,
            message: format!("Завершено: {}/{}", completed, total),
            results,
            total,
            completed,
        })
    }
    
    // === Вспомогательные методы очистки ===
    
    fn run_batch_file(path: &std::path::Path) -> ApiResponse {
        if !path.exists() {
            return ApiResponse {
                success: false,
                message: format!("Файл не найден: {:?}", path),
                exists: None,
                data: None,
            };
        }
        
        use std::process::Command;
        
        let result = Command::new("cmd")
            .arg("/c")
            .arg(path)
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    ApiResponse {
                        success: true,
                        message: format!("Выполнен: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                        exists: None,
                        data: None,
                    }
                } else {
                    ApiResponse {
                        success: false,
                        message: String::from_utf8_lossy(&output.stderr).to_string(),
                        exists: None,
                        data: None,
                    }
                }
            }
            Err(e) => ApiResponse {
                success: false,
                message: e.to_string(),
                exists: None,
                data: None,
            },
        }
    }
    
    fn run_batch_file_as_admin(path: &std::path::Path) -> ApiResponse {
        if !path.exists() {
            return ApiResponse {
                success: false,
                message: format!("Файл не найден: {:?}", path),
                exists: None,
                data: None,
            };
        }
        
        use std::process::Command;
        
        let result = Command::new("powershell")
            .arg("-Command")
            .arg(format!("Start-Process cmd -ArgumentList '/c','{}' -Verb RunAs", path.display()))
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    ApiResponse {
                        success: true,
                        message: format!("Запущен от администратора: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                        exists: None,
                        data: None,
                    }
                } else {
                    ApiResponse {
                        success: false,
                        message: String::from_utf8_lossy(&output.stderr).to_string(),
                        exists: None,
                        data: None,
                    }
                }
            }
            Err(e) => ApiResponse {
                success: false,
                message: e.to_string(),
                exists: None,
                data: None,
            },
        }
    }

    fn run_executable(path: &std::path::Path) -> ApiResponse {
        if !path.exists() {
            return ApiResponse {
                success: false,
                message: format!("Файл не найден: {:?}", path),
                exists: None,
                data: None,
            };
        }
        
        use std::process::Command;
        
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            const DETACHED_PROCESS: u32 = 0x00000008;
            
            let result = Command::new(path)
                .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
                .spawn();
            
            match result {
                Ok(_) => ApiResponse {
                    success: true,
                    message: format!("Запущен: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                    exists: None,
                    data: None,
                },
                Err(e) => ApiResponse {
                    success: false,
                    message: e.to_string(),
                    exists: None,
                    data: None,
                },
            }
        }
        
        #[cfg(not(windows))]
        {
            let result = Command::new(path)
                .spawn();
            
            match result {
                Ok(_) => ApiResponse {
                    success: true,
                    message: format!("Запущен: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                    exists: None,
                    data: None,
                },
                Err(e) => ApiResponse {
                    success: false,
                    message: e.to_string(),
                    exists: None,
                    data: None,
                },
            }
        }
    }
    
    // === Методы очистки системы (Windows specific) ===
    
    fn clean_event_logs() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::process::Command;
            let mut cleared = Vec::new();
            
            for log_name in &["Application", "System", "Security"] {
                if Command::new("wevtutil")
                    .args(["cl", log_name])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
                {
                    cleared.push(*log_name);
                }
            }
            
            GlobalCleanResultItem {
                success: true,
                message: format!("Очищено логов: {}", cleared.len()),
            }
        }
        #[cfg(not(windows))]
        {
            GlobalCleanResultItem {
                success: false,
                message: "Доступно только на Windows".to_string(),
            }
        }
    }
    
    fn clean_mft() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::env;
            use std::fs;
            use std::path::PathBuf;
            
            let windir = env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            let prefetch_path = PathBuf::from(windir).join("Prefetch");
            
            if prefetch_path.exists() {
                let mut deleted = 0;
                if let Ok(entries) = fs::read_dir(&prefetch_path) {
                    for entry in entries.flatten() {
                        if entry.path().extension().map(|e| e == "pf").unwrap_or(false) {
                            if fs::remove_file(entry.path()).is_ok() {
                                deleted += 1;
                            }
                        }
                    }
                }
                GlobalCleanResultItem {
                    success: true,
                    message: format!("Удалено файлов Prefetch: {}", deleted),
                }
            } else {
                GlobalCleanResultItem {
                    success: true,
                    message: "Prefetch пуст".to_string(),
                }
            }
        }
        #[cfg(not(windows))]
        {
            GlobalCleanResultItem {
                success: false,
                message: "Доступно только на Windows".to_string(),
            }
        }
    }
    
    fn clean_amcache() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::env;
            use std::fs;
            use std::path::PathBuf;
            use std::process::Command;
            
            let windir = env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            let amcache_dir = PathBuf::from(&windir).join("appcompat").join("Programs");
            
            let mut deleted = 0;
            if amcache_dir.exists() {
                if let Ok(entries) = fs::read_dir(&amcache_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_file() && fs::remove_file(entry.path()).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }
            
            // Попытка удалить реестр
            let _ = Command::new("reg")
                .args([
                    "delete",
                    "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\AppCompatCache",
                    "/f"
                ])
                .output();
            
            GlobalCleanResultItem {
                success: true,
                message: format!("Удалено файлов Amcache: {}", deleted),
            }
        }
        #[cfg(not(windows))]
        {
            GlobalCleanResultItem {
                success: false,
                message: "Доступно только на Windows".to_string(),
            }
        }
    }
    
    fn clean_jump_lists() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::env;
            use std::fs;
            use std::path::PathBuf;
            
            let appdata = env::var("APPDATA").unwrap_or_default();
            if appdata.is_empty() {
                return GlobalCleanResultItem {
                    success: false,
                    message: "Не найдена папка AppData".to_string(),
                };
            }
            
            let paths = [
                PathBuf::from(&appdata).join("Microsoft").join("Windows").join("Recent").join("AutomaticDestinations"),
                PathBuf::from(&appdata).join("Microsoft").join("Windows").join("Recent").join("CustomDestinations"),
            ];
            
            let mut deleted = 0;
            for path in &paths {
                if path.exists() {
                    if let Ok(entries) = fs::read_dir(path) {
                        for entry in entries.flatten() {
                            if fs::remove_file(entry.path()).is_ok() {
                                deleted += 1;
                            }
                        }
                    }
                }
            }
            
            GlobalCleanResultItem {
                success: true,
                message: format!("Удалено Jump Lists: {}", deleted),
            }
        }
        #[cfg(not(windows))]
        {
            GlobalCleanResultItem {
                success: false,
                message: "Доступно только на Windows".to_string(),
            }
        }
    }
    
    fn clean_recent_files() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::env;
            use std::fs;
            use std::path::PathBuf;
            
            let appdata = env::var("APPDATA").unwrap_or_default();
            let recent_path = PathBuf::from(&appdata).join("Microsoft").join("Windows").join("Recent");
            
            let mut deleted = 0;
            if recent_path.exists() {
                if let Ok(entries) = fs::read_dir(&recent_path) {
                    for entry in entries.flatten() {
                        if entry.path().is_file() && fs::remove_file(entry.path()).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }
            
            GlobalCleanResultItem {
                success: true,
                message: format!("Удалено файлов: {}", deleted),
            }
        }
        #[cfg(not(windows))]
        {
            GlobalCleanResultItem {
                success: false,
                message: "Доступно только на Windows".to_string(),
            }
        }
    }
    
    fn clean_browser_history() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::env;
            use std::fs;
            use std::path::PathBuf;
            
            let local_appdata = env::var("LOCALAPPDATA").unwrap_or_default();
            let appdata = env::var("APPDATA").unwrap_or_default();
            
            if local_appdata.is_empty() {
                return GlobalCleanResultItem {
                    success: false,
                    message: "Не найдена папка LocalAppData".to_string(),
                };
            }
            
            let browsers = [
                ("Chrome", PathBuf::from(&local_appdata).join("Google").join("Chrome").join("User Data").join("Default")),
                ("Edge", PathBuf::from(&local_appdata).join("Microsoft").join("Edge").join("User Data").join("Default")),
            ];
            
            let history_files = ["History", "Visited Links", "Favicons"];
            let mut deleted = 0;
            
            for (_, path) in &browsers {
                if path.exists() {
                    for hf in &history_files {
                        if fs::remove_file(path.join(hf)).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }
            
            // Firefox
            if !appdata.is_empty() {
                let firefox_path = PathBuf::from(&appdata).join("Mozilla").join("Firefox").join("Profiles");
                if firefox_path.exists() {
                    if let Ok(entries) = fs::read_dir(&firefox_path) {
                        for profile in entries.flatten() {
                            if profile.path().is_dir() {
                                for hf in &history_files {
                                    if fs::remove_file(profile.path().join(hf)).is_ok() {
                                        deleted += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            GlobalCleanResultItem {
                success: true,
                message: format!("Очищено истории браузеров: {}", deleted),
            }
        }
        #[cfg(not(windows))]
        {
            GlobalCleanResultItem {
                success: false,
                message: "Доступно только на Windows".to_string(),
            }
        }
    }
    
    fn clean_usn_journal() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::process::Command;
            use std::thread;
            use std::time::Duration;
            
            // Удаление журнала
            let _ = Command::new("fsutil")
                .args(["usn", "deletejournal", "/D", "C:"])
                .output();
            
            thread::sleep(Duration::from_secs(1));
            
            // Создание нового
            let _ = Command::new("fsutil")
                .args(["usn", "createjournal", "m=67108864", "a=8388608"])
                .output();
            
            GlobalCleanResultItem {
                success: true,
                message: "USN журнал пересоздан".to_string(),
            }
        }
        #[cfg(not(windows))]
        {
            GlobalCleanResultItem {
                success: false,
                message: "Доступно только на Windows".to_string(),
            }
        }
    }
    
    fn clean_temp_files() -> GlobalCleanResultItem {
        use std::env;
        use std::fs;
        use std::path::PathBuf;
        
        let temp_dirs: Vec<PathBuf> = [
            env::var("TEMP").ok().map(PathBuf::from),
            env::var("TMP").ok().map(PathBuf::from),
            env::var("WINDIR").ok().map(|w| PathBuf::from(w).join("Temp")),
        ]
        .into_iter()
        .flatten()
        .collect();
        
        let mut deleted = 0;
        
        for temp_dir_path in temp_dirs {
            let temp_dir = PathBuf::from(&temp_dir_path);
            if !temp_dir.exists() {
                continue;
            }
            
            // Проходим по файлам (без рекурсии для безопасности)
            if let Ok(entries) = fs::read_dir(&temp_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_file() {
                        if fs::remove_file(entry.path()).is_ok() {
                            deleted += 1;
                            if deleted >= 500 {
                                break;
                            }
                        }
                    }
                }
            }
            
            if deleted >= 500 {
                break;
            }
        }
        
        GlobalCleanResultItem {
            success: true,
            message: format!("Удалено временных файлов: {}", deleted),
        }
    }
}
