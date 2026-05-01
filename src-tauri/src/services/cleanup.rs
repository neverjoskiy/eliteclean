//! Основные операции очистки: сканирование, USN, следы, память, глобальная очистка

use std::time::Duration;
use tauri::State;
use tokio::time::sleep;
use crate::state::SharedAppState;
use crate::models::*;
use super::system::SystemService;
use super::privacy::PrivacyService;

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
                    if p.is_symlink() { continue; }
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
            s.add_log(
                format!("Сканирование завершено: {} файлов, {} МБ", total_files, total_size_bytes / 1024 / 1024),
                "success".to_string(),
            );
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
                    if p.is_symlink() { continue; }
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
            s.add_log(
                format!("Очистка завершена: {} файлов, {} МБ", cleaned_files, cleaned_bytes / 1024 / 1024),
                "success".to_string(),
            );
        }

        Ok(ScanCleanResponse { success: true, cleaned_files, cleaned_bytes, details })
    }

    /// Чистка строк (USN Journal)
    pub async fn clean_strings(state: State<'_, SharedAppState>) -> Result<CleanStringsResponse, String> {
        {
            let mut s = state.write().await;
            s.update_tool_state("clean_strings", true, 10, "running");
            s.add_log("Запуск чистки строк".to_string(), "info".to_string());
        }

        let scripts_dir = crate::utils::get_scripts_dir();
        let virus_bat = scripts_dir.join("вирус.bat");

        {
            let mut s = state.write().await;
            s.update_tool_state("clean_strings", true, 30, "running");
            s.add_log("Выполнение вирус.bat...".to_string(), "info".to_string());
        }

        let result1 = Self::run_batch_file(&virus_bat);

        if !result1.success {
            let mut s = state.write().await;
            s.update_tool_state("clean_strings", false, 0, "error");
            s.add_log(format!("Ошибка на шаге 1: {}", result1.message), "error".to_string());
            return Ok(CleanStringsResponse {
                success: false,
                message: format!("Шаг 1 (удаление журнала USN): {}", result1.message),
                steps: vec![],
            });
        }

        {
            let mut s = state.write().await;
            s.add_log("Шаг 1 выполнен успешно".to_string(), "success".to_string());
            s.update_tool_state("clean_strings", true, 60, "running");
        }

        sleep(Duration::from_secs(2)).await;

        let not_virus_bat = scripts_dir.join("не вирус.bat");

        {
            let mut s = state.write().await;
            s.add_log("Выполнение не вирус.bat...".to_string(), "info".to_string());
            s.update_tool_state("clean_strings", true, 80, "running");
        }

        let result2 = Self::run_batch_file(&not_virus_bat);

        if !result2.success {
            let mut s = state.write().await;
            s.update_tool_state("clean_strings", false, 0, "error");
            s.add_log(format!("Ошибка на шаге 2: {}", result2.message), "error".to_string());
            return Ok(CleanStringsResponse {
                success: false,
                message: format!("Шаг 2 (создание журнала USN): {}", result2.message),
                steps: vec![],
            });
        }

        {
            let mut s = state.write().await;
            s.add_log("Шаг 2 выполнен успешно".to_string(), "success".to_string());
            s.update_tool_state("clean_strings", false, 100, "completed");
            s.add_log("Чистка строк завершена".to_string(), "success".to_string());
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
            let mut s = state.write().await;
            s.update_tool_state("clean_tracks", true, 10, "running");
            s.add_log("Запуск очистки следов".to_string(), "info".to_string());
        }

        let scripts_dir = crate::utils::get_scripts_dir();
        let winlocker_bat = scripts_dir.join("винлокер.bat");

        if !winlocker_bat.exists() {
            let mut s = state.write().await;
            s.update_tool_state("clean_tracks", false, 0, "error");
            return Ok(ApiResponse {
                success: false,
                message: format!("Файл не найден: {:?}", winlocker_bat),
                exists: None,
                data: None,
            });
        }

        {
            let mut s = state.write().await;
            s.add_log("Запуск винлокер.bat (требуются права администратора)...".to_string(), "warning".to_string());
            s.update_tool_state("clean_tracks", true, 30, "running");
        }

        let result = Self::run_batch_file_as_admin(&winlocker_bat);

        {
            let mut s = state.write().await;
            let status = if result.success { "completed" } else { "error" };
            s.update_tool_state("clean_tracks", false, 100, status);
            let (msg, level) = if result.success {
                ("Очистка следов завершена".to_string(), "success")
            } else {
                (format!("Ошибка очистки следов: {}", result.message), "error")
            };
            s.add_log(msg, level.to_string());
        }

        Ok(result)
    }

    /// Симуляция открытия папок
    pub async fn simulate_folders(state: State<'_, SharedAppState>) -> Result<ApiResponse, String> {
        {
            let mut s = state.write().await;
            s.update_tool_state("simulate", true, 50, "running");
            s.add_log("Запуск симуляции открытия папок".to_string(), "info".to_string());
        }

        let scripts_dir = crate::utils::get_scripts_dir();
        let simulate_exe = scripts_dir.join("simulate.exe");
        let result = Self::run_executable(&simulate_exe);

        {
            let mut s = state.write().await;
            s.update_tool_state("simulate", false, 100, if result.success { "completed" } else { "error" });
            let (msg, level) = if result.success {
                ("Симуляция запущена".to_string(), "success")
            } else {
                (format!("Ошибка симуляции: {}", result.message), "error")
            };
            s.add_log(msg, level.to_string());
        }

        Ok(result)
    }

    /// Очистка памяти javaw.exe (Windows only)
    #[cfg(windows)]
    pub async fn clean_javaw_memory(state: State<'_, SharedAppState>) -> Result<CleanJavawResult, String> {
        use crate::memory::MemoryCleaner;

        {
            let mut s = state.write().await;
            s.update_tool_state("clean_javaw", true, 10, "running");
            s.add_log("Запуск очистки памяти javaw.exe".to_string(), "info".to_string());
        }

        let result = tokio::task::spawn_blocking(|| MemoryCleaner::clean_javaw_memory())
            .await
            .map_err(|e| e.to_string())?;

        {
            let mut s = state.write().await;
            s.update_tool_state("clean_javaw", false, 100, if result.success { "completed" } else { "error" });
            let (msg, level) = if result.success {
                ("Очистка памяти javaw.exe завершена".to_string(), "success")
            } else {
                ("Ошибка при очистке памяти javaw.exe".to_string(), "error")
            };
            s.add_log(msg, level.to_string());
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

    /// Глобальная очистка — принимает Vec<String> вместо Option<bool> полей
    pub async fn run_global_clean(
        state: State<'_, SharedAppState>,
        params: GlobalCleanParams,
    ) -> Result<GlobalCleanResponse, String> {
        let selected = params.options;
        let total = selected.len();

        if total == 0 {
            let mut s = state.write().await;
            s.update_tool_state("global_clean", false, 0, "error");
            return Ok(GlobalCleanResponse {
                success: false,
                message: "Не выбрано ни одной опции".to_string(),
                results: std::collections::HashMap::new(),
                total: 0,
                completed: 0,
            });
        }

        {
            let mut s = state.write().await;
            s.update_tool_state("global_clean", true, 0, "running");
            s.add_log("Запуск глобальной очистки".to_string(), "info".to_string());
        }

        let mut results = std::collections::HashMap::new();
        let mut completed = 0;

        for (i, option_key) in selected.iter().enumerate() {
            let option_name = match option_key.as_str() {
                "event_logs"     => "Очистка Event Log",
                "prefetch"       => "Очистка Prefetch",
                "amcache"        => "Очистка Amcache",
                "jump_lists"     => "Очистка Jump Lists",
                "recent_files"   => "Очистка Recent Files",
                "browser_history"=> "Очистка Browser History",
                "usn_journal"    => "Очистка USN Journal",
                "temp_files"     => "Очистка Temp Files",
                _                => "Неизвестная опция",
            };

            {
                let mut s = state.write().await;
                let progress = ((i as f32 / total as f32) * 100.0) as u8;
                s.update_tool_state("global_clean", true, progress, "running");
                s.add_log(format!("Очистка: {}...", option_name), "info".to_string());
            }

            let result = match option_key.as_str() {
                "event_logs"      => SystemService::clean_event_logs(),
                "prefetch"        => SystemService::clean_prefetch(),
                "amcache"         => SystemService::clean_amcache(),
                "jump_lists"      => PrivacyService::clean_jump_lists(),
                "recent_files"    => PrivacyService::clean_recent_files(),
                "browser_history" => PrivacyService::clean_browser_history(),
                "usn_journal"     => SystemService::clean_usn_journal(),
                "temp_files"      => SystemService::clean_temp_files(),
                _ => GlobalCleanResultItem { success: false, message: "Неизвестная опция".to_string() },
            };

            {
                let mut s = state.write().await;
                let (msg, level) = if result.success {
                    completed += 1;
                    (format!("✓ {}: {}", option_name, result.message), "success")
                } else {
                    (format!("✗ {}: {}", option_name, result.message), "error")
                };
                s.add_log(msg, level.to_string());
            }

            results.insert(option_key.clone(), result);
            sleep(Duration::from_millis(500)).await;
        }

        {
            let mut s = state.write().await;
            s.update_tool_state("global_clean", false, 100, "completed");
            s.add_log(
                format!("Глобальная очистка завершена: {}/{} успешно", completed, total),
                "success".to_string(),
            );
        }

        Ok(GlobalCleanResponse {
            success: true,
            message: format!("Завершено: {}/{}", completed, total),
            results,
            total,
            completed,
        })
    }

    // ── Вспомогательные методы запуска процессов ──

    pub fn run_batch_file(path: &std::path::Path) -> ApiResponse {
        if !path.exists() {
            return ApiResponse {
                success: false,
                message: format!("Файл не найден: {:?}", path),
                exists: None,
                data: None,
            };
        }
        use std::process::Command;
        match Command::new("cmd").arg("/c").arg(path).output() {
            Ok(output) if output.status.success() => ApiResponse {
                success: true,
                message: format!("Выполнен: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                exists: None,
                data: None,
            },
            Ok(output) => ApiResponse {
                success: false,
                message: String::from_utf8_lossy(&output.stderr).to_string(),
                exists: None,
                data: None,
            },
            Err(e) => ApiResponse { success: false, message: e.to_string(), exists: None, data: None },
        }
    }

    pub fn run_batch_file_as_admin(path: &std::path::Path) -> ApiResponse {
        if !path.exists() {
            return ApiResponse {
                success: false,
                message: format!("Файл не найден: {:?}", path),
                exists: None,
                data: None,
            };
        }
        use std::process::Command;
        let cmd = format!(
            "Start-Process cmd -ArgumentList '/c','{}' -Verb RunAs -Wait",
            path.display()
        );
        match Command::new("powershell").arg("-Command").arg(&cmd).output() {
            Ok(output) if output.status.success() => ApiResponse {
                success: true,
                message: format!("Запущен от администратора: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                exists: None,
                data: None,
            },
            Ok(output) => ApiResponse {
                success: false,
                message: String::from_utf8_lossy(&output.stderr).to_string(),
                exists: None,
                data: None,
            },
            Err(e) => ApiResponse { success: false, message: e.to_string(), exists: None, data: None },
        }
    }

    pub fn run_executable(path: &std::path::Path) -> ApiResponse {
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
            match Command::new(path).creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS).spawn() {
                Ok(_) => ApiResponse {
                    success: true,
                    message: format!("Запущен: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                    exists: None,
                    data: None,
                },
                Err(e) => ApiResponse { success: false, message: e.to_string(), exists: None, data: None },
            }
        }
        #[cfg(not(windows))]
        {
            match Command::new(path).spawn() {
                Ok(_) => ApiResponse {
                    success: true,
                    message: format!("Запущен: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                    exists: None,
                    data: None,
                },
                Err(e) => ApiResponse { success: false, message: e.to_string(), exists: None, data: None },
            }
        }
    }

    /// FunTime — запуск 1fc.exe, автовыбор процесса, очистка
    pub async fn fun_time(state: State<'_, SharedAppState>) -> Result<FunTimeCleanResult, String> {
        use std::process::{Command, Stdio};

        {
            let mut s = state.write().await;
            s.update_tool_state("fun_time", true, 10, "running");
            s.add_log("FunTime: запуск 1fc.exe...".to_string(), "info".to_string());
        }

        let scripts_dir = crate::utils::get_scripts_dir();
        let exe_path = scripts_dir.join("1fc.exe");

        if !exe_path.exists() {
            let mut s = state.write().await;
            s.update_tool_state("fun_time", false, 0, "error");
            let msg = format!("Файл не найден: {:?}", exe_path);
            s.add_log(msg.clone(), "error".to_string());
            return Ok(FunTimeCleanResult {
                success: false,
                message: msg,
                selected_pid: 0,
                selected_name: String::new(),
                regions_cleared: 0,
                sus_deleted: 0,
                cmdline_cleared: false,
                details: vec![],
            });
        }

        {
            let mut s = state.write().await;
            s.update_tool_state("fun_time", true, 25, "running");
            s.add_log("FunTime: получение списка процессов...".to_string(), "info".to_string());
        }

        let result = tokio::task::spawn_blocking(move || -> Result<FunTimeCleanResult, String> {
            use std::io::{BufRead, BufReader};

            // Шаг 1: получаем список процессов
            let mut child1 = Command::new(&exe_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Не удалось запустить 1fc.exe: {}", e))?;

            let stdout1 = child1.stdout.take().ok_or("нет stdout")?;
            let mut reader = BufReader::new(stdout1);
            let mut process_list: Vec<(usize, u32, String)> = Vec::new();
            let mut buf = String::new();

            let start = std::time::Instant::now();
            loop {
                if start.elapsed().as_secs() > 5 { break; }
                buf.clear();
                match reader.read_line(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => {
                        let line = buf.trim();
                        if line.starts_with('[') {
                            if let Some(rest) = line.strip_prefix('[') {
                                if let Some(bi) = rest.find(']') {
                                    if let Ok(idx) = rest[..bi].parse::<usize>() {
                                        let after = rest[bi+1..].trim();
                                        if let Some(pid_part) = after.strip_prefix("PID:") {
                                            if let Some(pipe) = pid_part.find('|') {
                                                let pid_str = pid_part[..pipe].trim();
                                                let name = pid_part[pipe+1..].trim().to_string();
                                                if let Ok(pid) = pid_str.parse::<u32>() {
                                                    process_list.push((idx, pid, name));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if line.starts_with('>') { break; }
                    }
                    Err(_) => break,
                }
            }
            drop(reader);
            let _ = child1.kill();
            let _ = child1.wait();

            let chosen = process_list.iter()
                .find(|(_, _, name)| { let n = name.to_lowercase(); !n.is_empty() && n != "unknown" })
                .or_else(|| process_list.first())
                .cloned();

            let (chosen_idx, chosen_pid, chosen_name) = match chosen {
                Some(c) => c,
                None => return Ok(FunTimeCleanResult {
                    success: false,
                    message: "Нет java-процессов для анализа".to_string(),
                    selected_pid: 0, selected_name: String::new(),
                    regions_cleared: 0, sus_deleted: 0, cmdline_cleared: false,
                    details: vec![],
                }),
            };

            // Шаг 2: запускаем с автовводом через bat
            let bat_path = std::env::temp_dir().join("_1fc_run.bat");
            let bat_content = format!(
                "@echo off\r\n(echo {}&echo.) | \"{}\"\r\n",
                chosen_idx, exe_path.display()
            );
            std::fs::write(&bat_path, bat_content)
                .map_err(|e| format!("Не удалось создать bat: {}", e))?;

            let output2 = Command::new("cmd")
                .args(["/c", bat_path.to_str().unwrap_or("_1fc_run.bat")])
                .output()
                .map_err(|e| format!("Ошибка запуска: {}", e))?;

            let _ = std::fs::remove_file(&bat_path);

            let raw_output = String::from_utf8_lossy(&output2.stdout).to_string()
                + &String::from_utf8_lossy(&output2.stderr);

            log::info!("FunTime raw_output (len={}): {:?}",
                raw_output.len(), &raw_output[..raw_output.len().min(3000)]);

            let mut sus_files: Vec<String> = Vec::new();
            let mut cmdline: Option<String> = None;
            let mut details: Vec<String> = Vec::new();

            let text = raw_output.replace("\r\n", "\n").replace('\r', "\n");

            // Debug: показываем длину вывода
            details.push(format!("— вывод 1fc.exe: {} байт", text.len()));

            // Парсим ASM-регионы только из строк с "DoomsDay" в поле client
            // Формат: PID: XXXX | type: N | client: DoomsDay Client | ... | regions: N
            //         -> 0xADDR size:SIZE tags:[ASM, RWX]
            let mut doomsday_regions: Vec<(usize, usize)> = Vec::new();

            // Проходим по тексту блоками: ищем строку с doomsday, затем собираем регионы до следующей PID-строки
            {
                let lines: Vec<&str> = text.lines().collect();
                let mut in_doomsday = false;
                let mut doomsday_found = false;
                for line in &lines {
                    let trimmed = line.trim();
                    // Проверяем, начинается ли строка с "PID:"
                    if trimmed.starts_with("PID:") {
                        // Сбрасываем флаг при новой PID-строке
                        in_doomsday = false;
                        // Проверяем наличие "DoomsDay" в поле client (регистронезависимо)
                        if let Some(client_pos) = trimmed.find("client:") {
                            let after_client = &trimmed[client_pos + 7..].trim();
                            if after_client.to_lowercase().contains("doomsday") {
                                in_doomsday = true;
                                doomsday_found = true;
                                log::info!("FunTime: найдена строка DoomsDay: {}", line);
                                details.push(format!("— найдена строка DoomsDay: {}", line.chars().take(80).collect::<String>()));
                            }
                        }
                    } else if in_doomsday && trimmed.starts_with("->") {
                        // -> 0xADDR size:SIZE tags:[ASM, ...]
                        log::info!("FunTime: парсинг региона: {}", line);
                        let chunk = trimmed.trim_start_matches("->").trim();
                        let addr_str = chunk.trim_start_matches("0x")
                            .split(|c: char| !c.is_ascii_hexdigit()).next().unwrap_or("");
                        let size_val = if let Some(s_pos) = chunk.find("size:") {
                            chunk[s_pos + 5..].split(|c: char| !c.is_ascii_digit()).next().unwrap_or("0")
                        } else { "0" };
                        let has_asm = chunk.to_uppercase().contains("ASM");
                        if has_asm && !addr_str.is_empty() {
                            if let (Ok(addr), Ok(size)) = (
                                usize::from_str_radix(addr_str, 16),
                                size_val.parse::<usize>(),
                            ) {
                                if addr > 0 && size > 0 {
                                    log::info!("FunTime: добавлен регион 0x{:X} size:{}", addr, size);
                                    doomsday_regions.push((addr, size));
                                }
                            }
                        }
                    }
                }
                if !doomsday_found {
                    details.push("— строка с DoomsDay Client не найдена в выводе".to_string());
                }
            }

            // Парсим SUS-файлы
            let mut sus_search = text.as_str();
            while let Some(pos) = sus_search.find("SUS") {
                let after = &sus_search[pos + 3..];
                if let Some(pipe) = after.find('|') {
                    let path_raw = after[pipe + 1..].trim_start();
                    let path = path_raw.split("[]").next().unwrap_or("").trim().to_string();
                    if !path.is_empty() && path.contains('\\') { sus_files.push(path); }
                }
                sus_search = &sus_search[pos + 3..];
            }

            // Парсим командную строку: формат многострочный блок P:\n"путь"\nD:
            // Также может быть формат: P:"путь"D: (всё в одной строке после разделителя)
            let mut cmdline_found = false;
            {
                let lines: Vec<&str> = text.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    // Вариант 1: P: на отдельной строке
                    if trimmed == "P:" && i + 1 < lines.len() {
                        let next = lines[i + 1].trim().trim_matches('"');
                        if !next.is_empty() && !next.starts_with("D:") {
                            log::info!("FunTime: найдена командная строка (многострочный формат): {}", next);
                            cmdline = Some(next.to_string());
                            cmdline_found = true;
                            details.push(format!("— найдена командная строка: {}", next.chars().take(60).collect::<String>()));
                            break;
                        }
                    }
                    // Вариант 2: P:"путь"D: в одной строке (из разделителя "---------------")
                    if trimmed.starts_with("P:") && trimmed.len() > 2 {
                        let after_p = &trimmed[2..];
                        // Ищем путь между кавычками или до D:
                        if let Some(quote_start) = after_p.find('"') {
                            if let Some(quote_end) = after_p[quote_start + 1..].find('"') {
                                let path = &after_p[quote_start + 1..quote_start + 1 + quote_end];
                                if !path.is_empty() {
                                    log::info!("FunTime: найдена командная строка (однострочный формат): {}", path);
                                    cmdline = Some(path.to_string());
                                    cmdline_found = true;
                                    details.push(format!("— найдена командная строка: {}", path.chars().take(60).collect::<String>()));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if !cmdline_found {
                details.push("— строка с командной строкой (P:) не найдена".to_string());
            }

            let mut details_final = details.clone();

            // Зануляем ASM-регионы DoomsDay Client целиком
            let regions_cleared = if !doomsday_regions.is_empty() {
                #[cfg(windows)]
                {
                    use windows::Win32::System::Threading::{
                        OpenProcess, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
                    };
                    use windows::Win32::Foundation::CloseHandle;

                    let handle = unsafe {
                        OpenProcess(PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, chosen_pid)
                    };
                    match handle {
                        Ok(h) => {
                            let mut cleared = 0usize;
                            for (addr, size) in &doomsday_regions {
                                let zeros = vec![0u8; *size];
                                let ok = crate::memory::write_region(h, *addr, &zeros);
                                if ok {
                                    cleared += 1;
                                    details_final.push(format!("✓ регион 0x{:X} size:{} очищен", addr, size));
                                } else {
                                    details_final.push(format!("✗ регион 0x{:X} — ошибка записи", addr));
                                }
                            }
                            unsafe { let _ = CloseHandle(h); }
                            cleared
                        }
                        Err(_) => {
                            details_final.push("✗ не удалось открыть процесс (нужны права администратора)".to_string());
                            0
                        }
                    }
                }
                #[cfg(not(windows))]
                { 0 }
            } else {
                details_final.push("— регионы DoomsDay Client (ASM) не обнаружены".to_string());
                0
            };

            let mut sus_deleted = 0usize;
            for path in &sus_files {
                let p = std::path::Path::new(path);
                if p.exists() {
                    match std::fs::remove_file(p) {
                        Ok(_) => { sus_deleted += 1; details_final.push(format!("✓ удалён SUS: {}", path)); }
                        Err(e) => { details_final.push(format!("✗ SUS {}: {}", path, e)); }
                    }
                } else {
                    details_final.push(format!("— SUS не найден на диске: {}", path));
                }
            }

            let cmdline_cleared = if cmdline.is_some() {
                #[cfg(windows)]
                {
                    let ok = crate::memory::clear_process_cmdline(chosen_pid);
                    if ok { details_final.push("✓ командная строка процесса очищена".to_string()); }
                    else  { details_final.push("✗ не удалось очистить командную строку".to_string()); }
                    ok
                }
                #[cfg(not(windows))]
                { false }
            } else {
                details_final.push("— командная строка не обнаружена в выводе".to_string());
                false
            };

            let msg = format!(
                "PID {} ({}): регионов очищено: {}, SUS удалено: {}, cmdline: {}",
                chosen_pid, chosen_name, regions_cleared, sus_deleted,
                if cmdline_cleared { "очищена" } else { "не изменена" }
            );

            Ok(FunTimeCleanResult {
                success: true,
                message: msg,
                selected_pid: chosen_pid,
                selected_name: chosen_name,
                regions_cleared,
                sus_deleted,
                cmdline_cleared,
                details: details_final,
            })
        })
        .await
        .map_err(|e| e.to_string())??;

        {
            let mut s = state.write().await;
            s.update_tool_state("fun_time", false, 100, if result.success { "completed" } else { "error" });
            s.add_log(
                format!("FunTime: {}", result.message),
                if result.success { "success" } else { "error" }.to_string(),
            );
        }

        Ok(result)
    }
}
