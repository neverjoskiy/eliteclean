//! Системные операции очистки: реестр, дампы, WU кэш, thumbnails, prefetch, amcache, USN

use tauri::State;
use crate::state::SharedAppState;
use crate::models::{SystemCleanResponse, GlobalCleanResultItem};

pub struct SystemService;

impl SystemService {

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
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\RecentDocs", "RecentDocs"),
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\ComDlg32\\OpenSavePidlMRU", "OpenSaveMRU"),
                ("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\UserAssist", "UserAssist"),
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
        let cleaned = details.iter().filter(|d| d.starts_with('✓')).count();
        let success = cleaned > 0;
        {
            let mut s = state.write().await;
            s.add_log(
                format!("Реестр: очищено {} ключей", cleaned),
                if success { "success" } else { "error" }.to_string(),
            );
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

    // ── Методы для GlobalClean (синхронные) ──

    pub fn clean_event_logs() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::process::Command;
            let mut cleared = 0usize;
            for log_name in &["Application", "System", "Security"] {
                if Command::new("wevtutil").args(["cl", log_name])
                    .output().map(|o| o.status.success()).unwrap_or(false)
                {
                    cleared += 1;
                }
            }
            GlobalCleanResultItem { success: true, message: format!("Очищено логов: {}", cleared) }
        }
        #[cfg(not(windows))]
        { GlobalCleanResultItem { success: false, message: "Доступно только на Windows".to_string() } }
    }

    pub fn clean_prefetch() -> GlobalCleanResultItem {
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
                            if fs::remove_file(entry.path()).is_ok() { deleted += 1; }
                        }
                    }
                }
                GlobalCleanResultItem { success: true, message: format!("Удалено файлов Prefetch: {}", deleted) }
            } else {
                GlobalCleanResultItem { success: true, message: "Prefetch пуст".to_string() }
            }
        }
        #[cfg(not(windows))]
        { GlobalCleanResultItem { success: false, message: "Доступно только на Windows".to_string() } }
    }

    pub fn clean_amcache() -> GlobalCleanResultItem {
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
                        if entry.path().is_file() && fs::remove_file(entry.path()).is_ok() { deleted += 1; }
                    }
                }
            }
            let _ = Command::new("reg").args([
                "delete",
                "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager\\AppCompatCache",
                "/f",
            ]).output();
            GlobalCleanResultItem { success: true, message: format!("Удалено файлов Amcache: {}", deleted) }
        }
        #[cfg(not(windows))]
        { GlobalCleanResultItem { success: false, message: "Доступно только на Windows".to_string() } }
    }

    pub fn clean_usn_journal() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::process::Command;
            use std::thread;
            use std::time::Duration;
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            let drive: String = windir.chars().take(2).collect();
            let del = Command::new("fsutil").args(["usn", "deletejournal", "/D", &drive]).output();
            if del.map(|o| !o.status.success()).unwrap_or(true) {
                return GlobalCleanResultItem {
                    success: false,
                    message: "Не удалось удалить USN журнал (требуются права администратора)".to_string(),
                };
            }
            thread::sleep(Duration::from_secs(1));
            let create = Command::new("fsutil")
                .args(["usn", "createjournal", "m=67108864", "a=8388608"])
                .output();
            if create.map(|o| !o.status.success()).unwrap_or(true) {
                return GlobalCleanResultItem {
                    success: false,
                    message: "Не удалось создать USN журнал".to_string(),
                };
            }
            GlobalCleanResultItem { success: true, message: "USN журнал пересоздан".to_string() }
        }
        #[cfg(not(windows))]
        { GlobalCleanResultItem { success: false, message: "Доступно только на Windows".to_string() } }
    }

    pub fn clean_temp_files() -> GlobalCleanResultItem {
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
        'outer: for dir in temp_dirs {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if entry.path().is_file() && fs::remove_file(entry.path()).is_ok() {
                        deleted += 1;
                        if deleted >= 500 { break 'outer; }
                    }
                }
            }
        }
        GlobalCleanResultItem { success: true, message: format!("Удалено временных файлов: {}", deleted) }
    }
}
