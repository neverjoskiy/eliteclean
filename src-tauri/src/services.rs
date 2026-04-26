//! Сервисы: бизнес-логика приложения

use log::{info, warn, error};
use std::time::Duration;
use tauri::State;
use tokio::time::sleep;
use crate::state::SharedAppState;
use crate::models::*;

// ══════════════════════════════════════════
// СКАНИРОВАНИЕ СИСТЕМЫ
// ══════════════════════════════════════════

/// Сервис очистки системы
pub struct CleanupService;

impl CleanupService {

    // ── Сканирование ──

    pub async fn scan_system(state: State<'_, SharedAppState>) -> Result<ScanResponse, String> {
        use std::fs;
        use std::path::PathBuf;

        {
            let mut s = state.write().await;
            s.add_log("Сканирование системы...".to_string(), "info".to_string());
        }

        let mut categories: Vec<ScanCategory> = Vec::new();

        // helper: считает файлы и размер в папке (не рекурсивно для скорости)
        fn scan_dir_shallow(path: &PathBuf) -> (usize, u64) {
            let mut count = 0usize;
            let mut size = 0u64;
            if let Ok(entries) = fs::read_dir(path) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_file() {
                        count += 1;
                        size += fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                    }
                }
            }
            (count, size)
        }

        fn scan_dir_recursive(path: &PathBuf, max: usize) -> (usize, u64) {
            let mut count = 0usize;
            let mut size = 0u64;
            if let Ok(entries) = fs::read_dir(path) {
                for e in entries.flatten() {
                    if count >= max { break; }
                    let p = e.path();
                    if p.is_file() {
                        count += 1;
                        size += fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                    } else if p.is_dir() {
                        let (c, s) = scan_dir_recursive(&p, max - count);
                        count += c; size += s;
                    }
                }
            }
            (count, size)
        }

        #[cfg(windows)]
        {
            let temp = std::env::var("TEMP").unwrap_or_default();
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let appdata = std::env::var("APPDATA").unwrap_or_default();

            // 1. Temp файлы
            {
                let mut count = 0usize; let mut size = 0u64;
                for dir in [temp.as_str(), &format!("{}\\Temp", windir)] {
                    let p = PathBuf::from(dir);
                    let (c, s) = scan_dir_shallow(&p);
                    count += c; size += s;
                }
                categories.push(ScanCategory {
                    id: "temp_files".to_string(),
                    name: "Временные файлы".to_string(),
                    description: "%TEMP%, Windows\\Temp".to_string(),
                    file_count: count, size_bytes: size, selected: true,
                });
            }

            // 2. Prefetch
            {
                let p = PathBuf::from(&windir).join("Prefetch");
                let (count, size) = scan_dir_shallow(&p);
                categories.push(ScanCategory {
                    id: "prefetch".to_string(),
                    name: "Prefetch".to_string(),
                    description: "Кэш предзагрузки программ".to_string(),
                    file_count: count, size_bytes: size, selected: true,
                });
            }

            // 3. Thumbnail кэш
            {
                let p = PathBuf::from(&local).join("Microsoft").join("Windows").join("Explorer");
                let mut count = 0usize; let mut size = 0u64;
                if let Ok(entries) = fs::read_dir(&p) {
                    for e in entries.flatten() {
                        let ep = e.path();
                        let name = ep.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if name.starts_with("thumbcache") && ep.is_file() {
                            count += 1;
                            size += fs::metadata(&ep).map(|m| m.len()).unwrap_or(0);
                        }
                    }
                }
                categories.push(ScanCategory {
                    id: "thumbnails".to_string(),
                    name: "Thumbnail кэш".to_string(),
                    description: "thumbcache_*.db".to_string(),
                    file_count: count, size_bytes: size, selected: true,
                });
            }

            // 4. Дампы памяти
            {
                let mut count = 0usize; let mut size = 0u64;
                let dump_paths = [
                    PathBuf::from(&windir).join("Minidump"),
                    PathBuf::from(&local).join("CrashDumps"),
                ];
                for dp in &dump_paths {
                    let (c, s) = scan_dir_shallow(dp);
                    count += c; size += s;
                }
                let mem_dmp = PathBuf::from(&windir).join("MEMORY.DMP");
                if mem_dmp.exists() {
                    count += 1;
                    size += fs::metadata(&mem_dmp).map(|m| m.len()).unwrap_or(0);
                }
                categories.push(ScanCategory {
                    id: "dumps".to_string(),
                    name: "Дампы памяти".to_string(),
                    description: "Minidump, MEMORY.DMP, CrashDumps".to_string(),
                    file_count: count, size_bytes: size, selected: false,
                });
            }

            // 5. Recent files
            {
                let p = PathBuf::from(&appdata).join("Microsoft").join("Windows").join("Recent");
                let (count, size) = scan_dir_shallow(&p);
                categories.push(ScanCategory {
                    id: "recent_files".to_string(),
                    name: "Недавние файлы".to_string(),
                    description: "История открытых файлов".to_string(),
                    file_count: count, size_bytes: size, selected: true,
                });
            }

            // 6. Jump Lists
            {
                let mut count = 0usize; let mut size = 0u64;
                for sub in ["AutomaticDestinations", "CustomDestinations"] {
                    let p = PathBuf::from(&appdata).join("Microsoft").join("Windows").join("Recent").join(sub);
                    let (c, s) = scan_dir_shallow(&p);
                    count += c; size += s;
                }
                categories.push(ScanCategory {
                    id: "jump_lists".to_string(),
                    name: "Jump Lists".to_string(),
                    description: "Закреплённые и последние документы".to_string(),
                    file_count: count, size_bytes: size, selected: true,
                });
            }

            // 7. Кэш браузеров
            {
                let mut count = 0usize; let mut size = 0u64;
                let browser_caches = [
                    PathBuf::from(&local).join("Google").join("Chrome").join("User Data").join("Default").join("Cache"),
                    PathBuf::from(&local).join("Microsoft").join("Edge").join("User Data").join("Default").join("Cache"),
                    PathBuf::from(&local).join("Mozilla").join("Firefox").join("Profiles"),
                ];
                for bp in &browser_caches {
                    let (c, s) = scan_dir_recursive(bp, 500);
                    count += c; size += s;
                }
                categories.push(ScanCategory {
                    id: "browser_cache".to_string(),
                    name: "Кэш браузеров".to_string(),
                    description: "Chrome, Edge, Firefox".to_string(),
                    file_count: count, size_bytes: size, selected: false,
                });
            }

            // 8. Windows Update кэш
            {
                let p = PathBuf::from(&windir).join("SoftwareDistribution").join("Download");
                let (count, size) = scan_dir_recursive(&p, 200);
                categories.push(ScanCategory {
                    id: "wu_cache".to_string(),
                    name: "Кэш Windows Update".to_string(),
                    description: "SoftwareDistribution\\Download".to_string(),
                    file_count: count, size_bytes: size, selected: false,
                });
            }
        }

        #[cfg(not(windows))]
        {
            categories.push(ScanCategory {
                id: "temp_files".to_string(),
                name: "Временные файлы".to_string(),
                description: "/tmp".to_string(),
                file_count: 0, size_bytes: 0, selected: true,
            });
        }

        let total_size_bytes = categories.iter().map(|c| c.size_bytes).sum();
        let total_files = categories.iter().map(|c| c.file_count).sum();

        {
            let mut s = state.write().await;
            s.add_log(format!("Сканирование завершено: {} файлов, {} МБ", total_files, total_size_bytes / 1024 / 1024), "success".to_string());
        }

        Ok(ScanResponse { categories, total_size_bytes, total_files })
    }

    pub async fn clean_scan_results(
        state: State<'_, SharedAppState>,
        params: ScanCleanParams,
    ) -> Result<ScanCleanResponse, String> {
        use std::fs;
        use std::path::PathBuf;
        use std::process::Command;

        let mut cleaned_files = 0usize;
        let mut cleaned_bytes = 0u64;
        let mut details = Vec::new();

        {
            let mut s = state.write().await;
            s.add_log(format!("Очистка {} категорий...", params.ids.len()), "info".to_string());
        }

        fn remove_dir_files(path: &PathBuf, filter: Option<&dyn Fn(&str) -> bool>) -> (usize, u64) {
            let mut count = 0usize; let mut size = 0u64;
            if let Ok(entries) = fs::read_dir(path) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_file() {
                        let name = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if filter.map(|f| f(&name)).unwrap_or(true) {
                            size += fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                            if fs::remove_file(&p).is_ok() { count += 1; }
                        }
                    }
                }
            }
            (count, size)
        }

        fn remove_dir_recursive(path: &PathBuf) -> (usize, u64) {
            let mut count = 0usize; let mut size = 0u64;
            if let Ok(entries) = fs::read_dir(path) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_file() {
                        size += fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                        if fs::remove_file(&p).is_ok() { count += 1; }
                    } else if p.is_dir() {
                        let (c, s) = remove_dir_recursive(&p);
                        count += c; size += s;
                        let _ = fs::remove_dir(&p);
                    }
                }
            }
            (count, size)
        }

        #[cfg(windows)]
        {
            let temp = std::env::var("TEMP").unwrap_or_default();
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let appdata = std::env::var("APPDATA").unwrap_or_default();

            for id in &params.ids {
                match id.as_str() {
                    "temp_files" => {
                        let mut c = 0usize; let mut s = 0u64;
                        for dir in [temp.as_str(), &format!("{}\\Temp", windir)] {
                            let (dc, ds) = remove_dir_files(&PathBuf::from(dir), None);
                            c += dc; s += ds;
                        }
                        cleaned_files += c; cleaned_bytes += s;
                        details.push(format!("✓ Temp: {} файлов", c));
                    }
                    "prefetch" => {
                        let p = PathBuf::from(&windir).join("Prefetch");
                        let (c, s) = remove_dir_files(&p, Some(&|n: &str| n.ends_with(".pf")));
                        cleaned_files += c; cleaned_bytes += s;
                        details.push(format!("✓ Prefetch: {} файлов", c));
                    }
                    "thumbnails" => {
                        let p = PathBuf::from(&local).join("Microsoft").join("Windows").join("Explorer");
                        let (c, s) = remove_dir_files(&p, Some(&|n: &str| n.starts_with("thumbcache")));
                        cleaned_files += c; cleaned_bytes += s;
                        details.push(format!("✓ Thumbnails: {} файлов", c));
                    }
                    "dumps" => {
                        let mut c = 0usize; let mut s = 0u64;
                        for dp in [PathBuf::from(&windir).join("Minidump"), PathBuf::from(&local).join("CrashDumps")] {
                            let (dc, ds) = remove_dir_files(&dp, None);
                            c += dc; s += ds;
                        }
                        let mem = PathBuf::from(&windir).join("MEMORY.DMP");
                        if mem.exists() {
                            s += fs::metadata(&mem).map(|m| m.len()).unwrap_or(0);
                            if fs::remove_file(&mem).is_ok() { c += 1; }
                        }
                        cleaned_files += c; cleaned_bytes += s;
                        details.push(format!("✓ Dumps: {} файлов", c));
                    }
                    "recent_files" => {
                        let p = PathBuf::from(&appdata).join("Microsoft").join("Windows").join("Recent");
                        let (c, s) = remove_dir_files(&p, None);
                        cleaned_files += c; cleaned_bytes += s;
                        details.push(format!("✓ Recent: {} файлов", c));
                    }
                    "jump_lists" => {
                        let mut c = 0usize; let mut s = 0u64;
                        for sub in ["AutomaticDestinations", "CustomDestinations"] {
                            let p = PathBuf::from(&appdata).join("Microsoft").join("Windows").join("Recent").join(sub);
                            let (dc, ds) = remove_dir_files(&p, None);
                            c += dc; s += ds;
                        }
                        cleaned_files += c; cleaned_bytes += s;
                        details.push(format!("✓ Jump Lists: {} файлов", c));
                    }
                    "browser_cache" => {
                        let mut c = 0usize; let mut s = 0u64;
                        for bp in [
                            PathBuf::from(&local).join("Google").join("Chrome").join("User Data").join("Default").join("Cache"),
                            PathBuf::from(&local).join("Microsoft").join("Edge").join("User Data").join("Default").join("Cache"),
                        ] {
                            let (dc, ds) = remove_dir_recursive(&bp);
                            c += dc; s += ds;
                        }
                        cleaned_files += c; cleaned_bytes += s;
                        details.push(format!("✓ Browser cache: {} файлов", c));
                    }
                    "wu_cache" => {
                        let _ = Command::new("net").args(["stop", "wuauserv"]).output();
                        let p = PathBuf::from(&windir).join("SoftwareDistribution").join("Download");
                        let (c, s) = remove_dir_recursive(&p);
                        let _ = Command::new("net").args(["start", "wuauserv"]).output();
                        cleaned_files += c; cleaned_bytes += s;
                        details.push(format!("✓ WU cache: {} объектов", c));
                    }
                    _ => {}
                }
            }
        }

        {
            let mut s = state.write().await;
            s.add_log(format!("Очистка завершена: {} файлов, {} МБ", cleaned_files, cleaned_bytes / 1024 / 1024), "success".to_string());
        }

        Ok(ScanCleanResponse {
            success: true,
            cleaned_files,
            cleaned_bytes,
            details,
        })
    }

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
            let status = if result.success { "completed" } else { "error" };
            app_state.update_tool_state("clean_tracks", false, 100, status);
            let msg = if result.success {
                "Очистка следов завершена".to_string()
            } else {
                format!("Ошибка очистки следов: {}", result.message)
            };
            app_state.add_log(msg, if result.success { "success" } else { "error" }.to_string());
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
            let del = Command::new("fsutil")
                .args(["usn", "deletejournal", "/D", "C:"])
                .output();
            
            if del.map(|o| !o.status.success()).unwrap_or(true) {
                return GlobalCleanResultItem {
                    success: false,
                    message: "Не удалось удалить USN журнал (требуются права администратора)".to_string(),
                };
            }
            
            thread::sleep(Duration::from_secs(1));
            
            // Создание нового
            let create = Command::new("fsutil")
                .args(["usn", "createjournal", "m=67108864", "a=8388608"])
                .output();
            
            if create.map(|o| !o.status.success()).unwrap_or(true) {
                return GlobalCleanResultItem {
                    success: false,
                    message: "Не удалось создать USN журнал".to_string(),
                };
            }
            
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

    // ══════════════════════════════════════════
    // СЕТЕВЫЕ МЕТОДЫ
    // ══════════════════════════════════════════

    pub async fn flush_dns(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
        use std::process::Command;
        let mut details = Vec::new();
        {
            let mut s = state.write().await;
            s.add_log("Сброс DNS кэша...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            if Command::new("ipconfig").arg("/flushdns").output()
                .map(|o| o.status.success()).unwrap_or(false)
            {
                details.push("✓ DNS кэш сброшен (ipconfig /flushdns)".to_string());
            } else {
                details.push("✗ Ошибка сброса DNS кэша".to_string());
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        let success = details.iter().any(|d| d.starts_with('✓'));
        {
            let mut s = state.write().await;
            s.add_log(details.join(", "), if success { "success" } else { "error" }.to_string());
        }
        Ok(NetworkCleanResponse { success, message: details.join("; "), details })
    }

    pub async fn reset_network(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
        use std::process::Command;
        let mut details = Vec::new();
        {
            let mut s = state.write().await;
            s.add_log("Сброс сетевых настроек...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            let cmds: &[(&str, &[&str])] = &[
                ("netsh", &["winsock", "reset"]),
                ("netsh", &["int", "ip", "reset"]),
                ("netsh", &["int", "ipv6", "reset"]),
                ("ipconfig", &["/flushdns"]),
            ];
            for (cmd, args) in cmds {
                let label = format!("{} {}", cmd, args.join(" "));
                if Command::new(cmd).args(*args).output()
                    .map(|o| o.status.success()).unwrap_or(false)
                {
                    details.push(format!("✓ {}", label));
                } else {
                    details.push(format!("✗ {} (нужны права администратора)", label));
                }
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        let ok = details.iter().filter(|d| d.starts_with('✓')).count();
        let success = ok > 0;
        {
            let mut s = state.write().await;
            s.add_log(format!("Сброс сети: {}/{} успешно", ok, details.len()),
                if success { "success" } else { "error" }.to_string());
        }
        Ok(NetworkCleanResponse { success, message: format!("Выполнено {}/{}", ok, details.len()), details })
    }

    pub async fn clear_arp(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
        use std::process::Command;
        let mut details = Vec::new();
        {
            let mut s = state.write().await;
            s.add_log("Очистка ARP таблицы...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            if Command::new("netsh").args(["interface", "ip", "delete", "arpcache"])
                .output().map(|o| o.status.success()).unwrap_or(false)
            {
                details.push("✓ ARP таблица очищена".to_string());
            } else if Command::new("arp").arg("-d").arg("*")
                .output().map(|o| o.status.success()).unwrap_or(false)
            {
                details.push("✓ ARP таблица очищена (arp -d)".to_string());
            } else {
                details.push("✗ Ошибка очистки ARP (нужны права администратора)".to_string());
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        let success = details.iter().any(|d| d.starts_with('✓'));
        {
            let mut s = state.write().await;
            s.add_log(details.join(", "), if success { "success" } else { "error" }.to_string());
        }
        Ok(NetworkCleanResponse { success, message: details.join("; "), details })
    }

    pub async fn clear_netbios(state: State<'_, SharedAppState>) -> Result<NetworkCleanResponse, String> {
        use std::process::Command;
        let mut details = Vec::new();
        {
            let mut s = state.write().await;
            s.add_log("Очистка NetBIOS кэша...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            if Command::new("nbtstat").arg("-R")
                .output().map(|o| o.status.success()).unwrap_or(false)
            {
                details.push("✓ NetBIOS кэш очищен (nbtstat -R)".to_string());
            } else {
                details.push("✗ Ошибка очистки NetBIOS".to_string());
            }
            if Command::new("nbtstat").arg("-RR")
                .output().map(|o| o.status.success()).unwrap_or(false)
            {
                details.push("✓ NetBIOS имена обновлены".to_string());
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        let success = details.iter().any(|d| d.starts_with('✓'));
        {
            let mut s = state.write().await;
            s.add_log(details.join(", "), if success { "success" } else { "error" }.to_string());
        }
        Ok(NetworkCleanResponse { success, message: details.join("; "), details })
    }

    // ══════════════════════════════════════════
    // СИСТЕМНЫЕ МЕТОДЫ
    // ══════════════════════════════════════════

    pub async fn clean_registry(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
        use std::process::Command;
        let mut details = Vec::new();
        {
            let mut s = state.write().await;
            s.add_log("Очистка реестра...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            let keys = [
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\RunMRU", "RunMRU"),
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\TypedPaths", "TypedPaths"),
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\RecentDocs", "RecentDocs"),
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\ComDlg32\\OpenSavePidlMRU", "OpenSaveMRU"),
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\UserAssist", "UserAssist"),
            ];
            for (key, label) in &keys {
                if Command::new("reg").args(["delete", key, "/va", "/f"])
                    .output().map(|o| o.status.success()).unwrap_or(false)
                {
                    details.push(format!("✓ {}", label));
                }
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        let cleaned = details.iter().filter(|d| d.starts_with('✓')).count();
        let success = cleaned > 0;
        {
            let mut s = state.write().await;
            s.add_log(format!("Реестр: очищено {} ключей", cleaned),
                if success { "success" } else { "error" }.to_string());
        }
        Ok(SystemCleanResponse { success, message: format!("Очищено {} ключей реестра", cleaned), details })
    }

    pub async fn clean_dumps(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
        use std::fs;
        use std::path::PathBuf;
        let mut details = Vec::new();
        let mut deleted = 0usize;
        {
            let mut s = state.write().await;
            s.add_log("Очистка дампов памяти...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            let dump_paths = [
                PathBuf::from(&windir).join("Minidump"),
                PathBuf::from(&windir).join("MEMORY.DMP"),
                PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default()).join("CrashDumps"),
            ];
            for path in &dump_paths {
                if path.is_file() {
                    if fs::remove_file(path).is_ok() {
                        deleted += 1;
                        details.push(format!("✓ {}", path.file_name().unwrap_or_default().to_string_lossy()));
                    }
                } else if path.is_dir() {
                    let before = deleted;
                    if let Ok(entries) = fs::read_dir(path) {
                        for entry in entries.flatten() {
                            if fs::remove_file(entry.path()).is_ok() { deleted += 1; }
                        }
                    }
                    if deleted > before {
                        details.push(format!("✓ {} ({} файлов)", path.file_name().unwrap_or_default().to_string_lossy(), deleted - before));
                    }
                }
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        {
            let mut s = state.write().await;
            s.add_log(format!("Дампы: удалено {} файлов", deleted), "success".to_string());
        }
        Ok(SystemCleanResponse { success: true, message: format!("Удалено дампов: {}", deleted), details })
    }

    pub async fn clean_update_cache(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
        use std::fs;
        use std::path::PathBuf;
        use std::process::Command;
        let mut details = Vec::new();
        let mut deleted = 0usize;
        {
            let mut s = state.write().await;
            s.add_log("Очистка кэша Windows Update...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            let _ = Command::new("net").args(["stop", "wuauserv"]).output();
            let _ = Command::new("net").args(["stop", "bits"]).output();
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            let cache_path = PathBuf::from(&windir).join("SoftwareDistribution").join("Download");
            if cache_path.exists() {
                if let Ok(entries) = fs::read_dir(&cache_path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() && fs::remove_file(&p).is_ok() { deleted += 1; }
                        else if p.is_dir() && fs::remove_dir_all(&p).is_ok() { deleted += 1; }
                    }
                }
                details.push(format!("✓ SoftwareDistribution\\Download: {} объектов", deleted));
            }
            let _ = Command::new("net").args(["start", "wuauserv"]).output();
            let _ = Command::new("net").args(["start", "bits"]).output();
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        {
            let mut s = state.write().await;
            s.add_log(format!("WU кэш: удалено {} объектов", deleted), "success".to_string());
        }
        Ok(SystemCleanResponse { success: true, message: format!("Удалено из кэша WU: {}", deleted), details })
    }

    pub async fn clean_thumbnails(state: State<'_, SharedAppState>) -> Result<SystemCleanResponse, String> {
        use std::fs;
        use std::path::PathBuf;
        let mut details = Vec::new();
        let mut deleted = 0usize;
        {
            let mut s = state.write().await;
            s.add_log("Очистка thumbnail кэша...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let thumb_path = PathBuf::from(&local).join("Microsoft").join("Windows").join("Explorer");
            if thumb_path.exists() {
                if let Ok(entries) = fs::read_dir(&thumb_path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        let name = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if name.starts_with("thumbcache") && p.is_file() {
                            if fs::remove_file(&p).is_ok() { deleted += 1; }
                        }
                    }
                }
                details.push(format!("✓ Thumbcache: {} файлов", deleted));
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        {
            let mut s = state.write().await;
            s.add_log(format!("Thumbnails: удалено {} файлов", deleted), "success".to_string());
        }
        Ok(SystemCleanResponse { success: true, message: format!("Удалено thumbnail файлов: {}", deleted), details })
    }

    // ══════════════════════════════════════════
    // ПРИВАТНОСТЬ
    // ══════════════════════════════════════════

    pub async fn clear_clipboard(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
        use std::process::Command;
        let mut details = Vec::new();
        {
            let mut s = state.write().await;
            s.add_log("Очистка буфера обмена...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            if Command::new("cmd").args(["/c", "echo off | clip"])
                .output().map(|o| o.status.success()).unwrap_or(false)
            {
                details.push("✓ Буфер обмена очищен".to_string());
            } else {
                details.push("✗ Ошибка очистки буфера".to_string());
            }
            if Command::new("reg").args([
                "delete", "HKCU\\Software\\Microsoft\\Clipboard", "/va", "/f"
            ]).output().map(|o| o.status.success()).unwrap_or(false) {
                details.push("✓ История буфера обмена очищена".to_string());
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        let success = details.iter().any(|d| d.starts_with('✓'));
        {
            let mut s = state.write().await;
            s.add_log("Буфер обмена очищен".to_string(), if success { "success" } else { "error" }.to_string());
        }
        Ok(PrivacyCleanResponse { success, message: details.join("; "), details })
    }

    pub async fn clean_icon_cache(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
        use std::fs;
        use std::path::PathBuf;
        let mut details = Vec::new();
        let mut deleted = 0usize;
        {
            let mut s = state.write().await;
            s.add_log("Очистка кэша иконок...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let icon_db = PathBuf::from(&local).join("IconCache.db");
            if icon_db.exists() && fs::remove_file(&icon_db).is_ok() {
                deleted += 1;
                details.push("✓ IconCache.db удалён".to_string());
            }
            let explorer_path = PathBuf::from(&local).join("Microsoft").join("Windows").join("Explorer");
            if explorer_path.exists() {
                if let Ok(entries) = fs::read_dir(&explorer_path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        let name = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if name.starts_with("iconcache") && p.is_file() {
                            if fs::remove_file(&p).is_ok() { deleted += 1; }
                        }
                    }
                }
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        {
            let mut s = state.write().await;
            s.add_log(format!("Иконки: удалено {} файлов", deleted), "success".to_string());
        }
        Ok(PrivacyCleanResponse { success: true, message: format!("Удалено файлов кэша иконок: {}", deleted), details })
    }

    pub async fn clean_search_history(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
        use std::process::Command;
        let mut details = Vec::new();
        {
            let mut s = state.write().await;
            s.add_log("Очистка истории поиска...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            let keys = [
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\WordWheelQuery", "WordWheelQuery"),
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Search", "Search"),
            ];
            for (key, label) in &keys {
                if Command::new("reg").args(["delete", key, "/va", "/f"])
                    .output().map(|o| o.status.success()).unwrap_or(false)
                {
                    details.push(format!("✓ {}", label));
                }
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        let success = details.iter().any(|d| d.starts_with('✓'));
        {
            let mut s = state.write().await;
            s.add_log("История поиска очищена".to_string(), if success { "success" } else { "error" }.to_string());
        }
        Ok(PrivacyCleanResponse { success, message: details.join("; "), details })
    }

    pub async fn clean_run_history(state: State<'_, SharedAppState>) -> Result<PrivacyCleanResponse, String> {
        use std::process::Command;
        let mut details = Vec::new();
        {
            let mut s = state.write().await;
            s.add_log("Очистка истории запуска...".to_string(), "info".to_string());
        }
        #[cfg(windows)]
        {
            let keys = [
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\RunMRU", "Run MRU"),
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\TypedPaths", "TypedPaths"),
            ];
            for (key, label) in &keys {
                if Command::new("reg").args(["delete", key, "/va", "/f"])
                    .output().map(|o| o.status.success()).unwrap_or(false)
                {
                    details.push(format!("✓ {}", label));
                }
            }
        }
        #[cfg(not(windows))]
        { details.push("Доступно только на Windows".to_string()); }
        let success = details.iter().any(|d| d.starts_with('✓'));
        {
            let mut s = state.write().await;
            s.add_log("История запуска очищена".to_string(), if success { "success" } else { "error" }.to_string());
        }
        Ok(PrivacyCleanResponse { success, message: details.join("; "), details })
    }
}
