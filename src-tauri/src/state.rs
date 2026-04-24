//! Глобальное состояние приложения (аналог глобальных переменных Python)
//! Используется Arc<RwLock<>> для потокобезопасности

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use crate::models::{AppStatus, LogEntry, ToolState};

/// Глобальное состояние приложения
#[derive(Debug)]
pub struct AppState {
    /// Текущий статус приложения
    pub status: AppStatus,
    /// История запусков (последние 10)
    pub launch_history: Vec<LaunchRecord>,
    /// Логи приложения (последние 100)
    pub logs: Vec<LogEntry>,
    /// Состояния инструментов
    pub tool_states: HashMap<String, ToolState>,
}

/// Запись истории запуска
#[derive(Debug, Clone)]
pub struct LaunchRecord {
    pub timestamp: DateTime<Utc>,
    pub status: String,
}

impl Default for AppState {
    fn default() -> Self {
        let mut tool_states = HashMap::new();
        
        // Инициализация состояний инструментов (как в Python)
        tool_states.insert("clean_strings".to_string(), ToolState {
            running: false,
            progress: 0,
            status: "idle".to_string(),
            results: None,
        });
        tool_states.insert("clean_tracks".to_string(), ToolState {
            running: false,
            progress: 0,
            status: "idle".to_string(),
            results: None,
        });
        tool_states.insert("clean_javaw".to_string(), ToolState {
            running: false,
            progress: 0,
            status: "idle".to_string(),
            results: None,
        });
        tool_states.insert("simulate".to_string(), ToolState {
            running: false,
            progress: 0,
            status: "idle".to_string(),
            results: None,
        });
        tool_states.insert("global_clean".to_string(), ToolState {
            running: false,
            progress: 0,
            status: "idle".to_string(),
            results: Some(serde_json::Map::new()),
        });
        
        Self {
            status: AppStatus::Ready,
            launch_history: Vec::new(),
            logs: Vec::new(),
            tool_states,
        }
    }
}

/// Обёртка для потокобезопасного доступа к состоянию
pub type SharedAppState = Arc<RwLock<AppState>>;

impl AppState {
    /// Создать новое состояние
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Добавить запись лога
    pub fn add_log(&mut self, message: String, log_type: String) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            message,
            log_type,
        };
        
        self.logs.push(entry);
        
        // Оставляем только последние 100 записей
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
    
    /// Получить последние N логов
    pub fn get_logs(&self, lines: usize) -> Vec<LogEntry> {
        if self.logs.is_empty() {
            return Vec::new();
        }
        
        let start = if self.logs.len() > lines {
            self.logs.len() - lines
        } else {
            0
        };
        
        self.logs[start..].to_vec()
    }
    
    /// Очистить логи
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
    
    /// Обновить статус инструмента
    pub fn update_tool_state(&mut self, name: &str, running: bool, progress: u8, status: &str) {
        if let Some(tool) = self.tool_states.get_mut(name) {
            tool.running = running;
            tool.progress = progress;
            tool.status = status.to_string();
        }
    }
}
