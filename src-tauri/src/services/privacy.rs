//! Операции очистки приватности: буфер обмена, иконки, история поиска/запуска, jump lists, recent

use tauri::State;
use crate::state::SharedAppState;
use crate::models::{PrivacyCleanResponse, GlobalCleanResultItem};

pub struct PrivacyService;

impl PrivacyService {

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
                "delete", "HKCU\\Software\\Microsoft\\Clipboard", "/va", "/f",
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

    // ── Методы для GlobalClean (синхронные) ──

    pub fn clean_jump_lists() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::env;
            use std::fs;
            use std::path::PathBuf;
            let appdata = env::var("APPDATA").unwrap_or_default();
            if appdata.is_empty() {
                return GlobalCleanResultItem { success: false, message: "Не найдена папка AppData".to_string() };
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
                            if fs::remove_file(entry.path()).is_ok() { deleted += 1; }
                        }
                    }
                }
            }
            GlobalCleanResultItem { success: true, message: format!("Удалено Jump Lists: {}", deleted) }
        }
        #[cfg(not(windows))]
        { GlobalCleanResultItem { success: false, message: "Доступно только на Windows".to_string() } }
    }

    pub fn clean_recent_files() -> GlobalCleanResultItem {
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
                        if entry.path().is_file() && fs::remove_file(entry.path()).is_ok() { deleted += 1; }
                    }
                }
            }
            GlobalCleanResultItem { success: true, message: format!("Удалено файлов: {}", deleted) }
        }
        #[cfg(not(windows))]
        { GlobalCleanResultItem { success: false, message: "Доступно только на Windows".to_string() } }
    }

    pub fn clean_browser_history() -> GlobalCleanResultItem {
        #[cfg(windows)]
        {
            use std::env;
            use std::fs;
            use std::path::PathBuf;
            let local_appdata = env::var("LOCALAPPDATA").unwrap_or_default();
            let appdata = env::var("APPDATA").unwrap_or_default();
            if local_appdata.is_empty() {
                return GlobalCleanResultItem { success: false, message: "Не найдена папка LocalAppData".to_string() };
            }
            let browsers = [
                PathBuf::from(&local_appdata).join("Google").join("Chrome").join("User Data").join("Default"),
                PathBuf::from(&local_appdata).join("Microsoft").join("Edge").join("User Data").join("Default"),
            ];
            let history_files = ["History", "Visited Links", "Favicons"];
            let mut deleted = 0;
            for path in &browsers {
                if path.exists() {
                    for hf in &history_files {
                        if fs::remove_file(path.join(hf)).is_ok() { deleted += 1; }
                    }
                }
            }
            if !appdata.is_empty() {
                let firefox_path = PathBuf::from(&appdata).join("Mozilla").join("Firefox").join("Profiles");
                if firefox_path.exists() {
                    if let Ok(entries) = fs::read_dir(&firefox_path) {
                        for profile in entries.flatten() {
                            if profile.path().is_dir() {
                                for hf in &history_files {
                                    if fs::remove_file(profile.path().join(hf)).is_ok() { deleted += 1; }
                                }
                            }
                        }
                    }
                }
            }
            GlobalCleanResultItem { success: true, message: format!("Очищено истории браузеров: {}", deleted) }
        }
        #[cfg(not(windows))]
        { GlobalCleanResultItem { success: false, message: "Доступно только на Windows".to_string() } }
    }
}
