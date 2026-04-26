//! Утилиты: работа с путями, логирование, общие функции

use std::path::{Path, PathBuf};
use log::{info, warn};

/// Получить базовый путь (для сборки и обычной работы)
/// Аналог get_base_path() из Python
pub fn get_base_path() -> PathBuf {
    // В Tauri ресурсы находятся относительно исполняемого файла
    // или в специальном ресурсном бандле
    
    // Путь к исполняемому файлу
    if let Some(exe_path) = std::env::current_exe().ok() {
        if let Some(parent) = exe_path.parent() {
            return parent.to_path_buf();
        }
    }
    
    // Fallback на текущую директорию
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Получить путь к директории логов
pub fn get_log_dir() -> PathBuf {
    let log_dir = if cfg!(windows) {
        // Windows: %TEMP%/EliteCleaner/logs
        if let Ok(temp) = std::env::var("TEMP") {
            PathBuf::from(temp).join("EliteCleaner").join("logs")
        } else {
            get_base_path().join("logs")
        }
    } else {
        // Linux/macOS: ~/.local/share/elite-cleaner/logs
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(home).join(".local").join("share").join("elite-cleaner").join("logs");
            if dir.exists() {
                dir
            } else {
                get_base_path().join("logs")
            }
        } else {
            get_base_path().join("logs")
        }
    };
    
    // Создать директорию если не существует
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        warn!("Не удалось создать директорию логов {:?}: {}", log_dir, e);
    }
    
    log_dir
}

/// Получить путь к временному файлу Microsoft.Ink.dll
pub fn get_target_jar_path() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    temp_dir.join("Microsoft.Ink.dll")
}

/// Получить путь к директории скриптов
pub fn get_scripts_dir() -> PathBuf {
    // В production ресурсы копируются в бандл
    // В dev режиме - относительный путь от исполняемого файла
    let base = get_base_path();
    
    // Проверяем несколько возможных путей
    let candidates = [
        base.join("scripts"),
        base.parent().map(|p| p.join("scripts")).unwrap_or_else(|| base.join("scripts")),
    ];
    
    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }
    
    // Fallback
    base.join("scripts")
}

/// Инициализация логирования
pub fn init_logging() {
    let log_dir = get_log_dir();
    
    // Настраиваем env_logger
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();
    
    info!("Логирование инициализировано. Лог-директория: {:?}", log_dir);
}

/// Проверка существования файла
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Получить размер файла в байтах
pub fn get_file_size(path: &Path) -> u64 {
    std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0)
}
