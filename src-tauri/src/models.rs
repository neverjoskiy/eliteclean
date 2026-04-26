//! Модели данных (аналог Pydantic моделей)
//! Все структуры с сериализацией через serde

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Статус приложения
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppStatus {
    Ready,
    Running,
    Error,
}

/// Ответ статуса приложения (аналог /api/status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: AppStatus,
    pub launches: usize,
    pub file_exists: bool,
    pub file_size: u64,
    pub file_path: String,
    pub timestamp: DateTime<Utc>,
}

/// Общий ответ успеха/ошибки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Запись лога
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    #[serde(rename = "type")]
    pub log_type: String,
}

/// Ответ логов (аналог /api/logs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsResponse {
    pub logs: Vec<LogEntry>,
}

/// Результат загрузки файла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions_scanned: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions_matched: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared_count: Option<usize>,
}

/// Результат очистки памяти javaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanJavawResult {
    pub success: bool,
    pub message: String,
    pub regions_scanned: usize,
    pub regions_matched: usize,
    pub cleared_count: usize,
}

/// Статус инструмента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolState {
    pub running: bool,
    pub progress: u8,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Ответ статуса инструментов (аналог /api/tools/status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsStatusResponse {
    pub tools: std::collections::HashMap<String, ToolState>,
}

/// Опция глобальной очистки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanOption {
    pub name: String,
    pub description: String,
}

/// Ответ опций глобальной очистки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCleanOptionsResponse {
    pub options: std::collections::HashMap<String, CleanOption>,
}

/// Параметры глобальной очистки (входные)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCleanParams {
    pub event_logs: Option<bool>,
    pub mft: Option<bool>,
    pub amcache: Option<bool>,
    pub jump_lists: Option<bool>,
    pub recent_files: Option<bool>,
    pub browser_history: Option<bool>,
    pub usn_journal: Option<bool>,
    pub temp_files: Option<bool>,
}

/// Результат глобальной очистки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCleanResultItem {
    pub success: bool,
    pub message: String,
}

/// Ответ глобальной очистки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCleanResponse {
    pub success: bool,
    pub message: String,
    pub results: std::collections::HashMap<String, GlobalCleanResultItem>,
    pub total: usize,
    pub completed: usize,
}

/// Шаг очистки строк
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanStep {
    pub name: String,
    pub status: String,
}

/// Ответ чистки строк
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanStringsResponse {
    pub success: bool,
    pub message: String,
    pub steps: Vec<CleanStep>,
}

/// Ответ сетевой очистки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCleanResponse {
    pub success: bool,
    pub message: String,
    pub details: Vec<String>,
}

/// Ответ системной очистки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCleanResponse {
    pub success: bool,
    pub message: String,
    pub details: Vec<String>,
}

/// Ответ очистки приватности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyCleanResponse {
    pub success: bool,
    pub message: String,
    pub details: Vec<String>,
}

/// Информация о сетевых адаптерах
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfoResponse {
    pub adapters: Vec<String>,
    pub dns_servers: Vec<String>,
    pub connections: usize,
}

/// Одна найденная категория при сканировании
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub file_count: usize,
    pub size_bytes: u64,
    pub selected: bool,
}

/// Ответ сканирования системы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResponse {
    pub categories: Vec<ScanCategory>,
    pub total_size_bytes: u64,
    pub total_files: usize,
}

/// Параметры очистки по результатам сканирования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCleanParams {
    pub ids: Vec<String>,
}

/// Ответ очистки по результатам сканирования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCleanResponse {
    pub success: bool,
    pub cleaned_files: usize,
    pub cleaned_bytes: u64,
    pub details: Vec<String>,
}

/// Статус приложения (упрощённый, без jar)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStatusResponse {
    pub status: AppStatus,
    pub timestamp: DateTime<Utc>,
}
