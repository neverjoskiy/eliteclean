//! Управление автозагрузкой Windows

use std::process::Command;
use std::path::Path;
use std::fs;
use crate::models::{StartupEntry, StartupEntryRequest, ApiResponse};

fn guid() -> String {
    format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
}

fn run_reg_query(key: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let _output = Command::new("reg")
        .args(["query", key, "/ve"])
        .output();
    // Пробуем получить все значения через /v для каждого
    let output2 = Command::new("reg")
        .args(["query", key])
        .output();
    if let Ok(o) = output2 {
        let stdout = String::from_utf8_lossy(&o.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("HKEY") || line.starts_with("ERROR") {
                continue;
            }
            if line.starts_with("<") || line.starts_with("(") {
                continue;
            }
            // reg query выводит:  Name    Type    Data
            // потом каждая строка: имя    тип    значение
            let parts: Vec<&str> = line.splitn(3, "  ").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let data = if parts.len() >= 3 { parts[2] } else { "" };
                if name != "(Default)" && name != "Name" {
                    entries.push((name, data.to_string()));
                }
            }
        }
    }
    entries
}

fn read_run_key(hive: &str, subkey: &str, loc: &str) -> Vec<StartupEntry> {
    let key = format!("{}\\{}", hive, subkey);
    let items = run_reg_query(&key);
    items.into_iter().map(|(name, path)| {
        let id = format!("{}_{}_{}", loc, name, guid());
        StartupEntry {
            id,
            name,
            path,
            location: loc.to_string(),
            enabled: true,
        }
    }).collect()
}

fn read_disabled_run_key(hive: &str, subkey: &str, loc: &str) -> Vec<StartupEntry> {
    let key = format!("{}\\{}-Disabled", hive, subkey);
    let items = run_reg_query(&key);
    items.into_iter().map(|(name, path)| {
        let id = format!("{}_disabled_{}_{}", loc, name, guid());
        StartupEntry {
            id,
            name,
            path,
            location: loc.to_string(),
            enabled: false,
        }
    }).collect()
}

fn read_winlogon() -> Vec<StartupEntry> {
    let mut entries = Vec::new();
    let key = "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon";
    let out = Command::new("reg").args(["query", key, "/v", "Shell"]).output();
    if let Ok(o) = out {
        let stdout = String::from_utf8_lossy(&o.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.contains("Shell") && line.contains("REG_") {
                let parts: Vec<&str> = line.splitn(3, "  ").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                if parts.len() >= 3 {
                    entries.push(StartupEntry {
                        id: format!("winlogon_shell_{}", guid()),
                        name: "Shell".to_string(),
                        path: parts[2].to_string(),
                        location: "winlogon_shell".to_string(),
                        enabled: true,
                    });
                }
            }
        }
    }
    let out2 = Command::new("reg").args(["query", key, "/v", "Userinit"]).output();
    if let Ok(o) = out2 {
        let stdout = String::from_utf8_lossy(&o.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.contains("Userinit") && line.contains("REG_") {
                let parts: Vec<&str> = line.splitn(3, "  ").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                if parts.len() >= 3 {
                    entries.push(StartupEntry {
                        id: format!("winlogon_userinit_{}", guid()),
                        name: "Userinit".to_string(),
                        path: parts[2].to_string(),
                        location: "winlogon_userinit".to_string(),
                        enabled: true,
                    });
                }
            }
        }
    }
    entries
}

fn read_startup_folder(path: &str, loc: &str) -> Vec<StartupEntry> {
    let mut entries = Vec::new();
    let expanded = std::env::var("APPDATA").unwrap_or_else(|_| std::env::var("USERPROFILE").unwrap_or_default() + "\\AppData\\Roaming");
    let expanded_common = std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string());
    let real_path = if path.contains("%AppData%") {
        path.replace("%AppData%", &expanded)
    } else if path.contains("%ProgramData%") {
        path.replace("%ProgramData%", &expanded_common)
    } else {
        path.to_string()
    };
    let p = Path::new(&real_path);
    if let Ok(entries_dir) = fs::read_dir(p) {
        for entry in entries_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path_str = entry.path().to_string_lossy().to_string();
            let enabled = !name.ends_with(".disabled");
            let clean_name = if enabled { name.clone() } else { name.trim_end_matches(".disabled").to_string() };
            let id = format!("{}_{}_{}", loc, clean_name, guid());
            entries.push(StartupEntry {
                id,
                name: clean_name,
                path: path_str.clone(),
                location: loc.to_string(),
                enabled,
            });
        }
    }
    entries
}

fn read_task_scheduler() -> Vec<StartupEntry> {
    let mut entries = Vec::new();
    // Получаем список задач через schtasks
    let out = Command::new("schtasks")
        .args(["/query", "/fo", "LIST", "/v"])
        .output();
    if let Ok(o) = out {
        let stdout = String::from_utf8_lossy(&o.stdout);
        let mut current_name = String::new();
        let mut current_path = String::new();
        let mut current_enabled = true;
        let mut current_trigger = String::new();
        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("TaskName:") {
                if !current_name.is_empty() && (current_trigger.to_lowercase().contains("at logon") || current_trigger.to_lowercase().contains("at startup") || current_trigger.to_lowercase().contains("when user logs on")) {
                    let id = format!("task_scheduler_{}_{}", current_name.replace('\\', "_"), guid());
                    entries.push(StartupEntry {
                        id,
                        name: current_name.clone(),
                        path: current_path.clone(),
                        location: "task_scheduler".to_string(),
                        enabled: current_enabled,
                    });
                }
                current_name = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
                current_path = String::new();
                current_enabled = true;
                current_trigger = String::new();
            } else if line.starts_with("Task To Run:") || line.starts_with("Задача для выполнения:") {
                current_path = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Scheduled Task State:") || line.starts_with("Состояние запланированной задачи:") {
                let state = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_lowercase();
                current_enabled = state.contains("enabled") || state.contains("включено") || state.contains("ready");
            } else if line.starts_with("Start Triggers:") || line.starts_with("Triggers:") || line.starts_with("Триггеры:") || line.starts_with("Начать триггеры:") {
                // начало секции триггеров, далее идут строки
            } else if line.to_lowercase().contains("logon") || line.to_lowercase().contains("startup") || line.to_lowercase().contains("при входе") || line.to_lowercase().contains("при запуске") {
                current_trigger = line.to_string();
            }
        }
        // последняя задача
        if !current_name.is_empty() && (current_trigger.to_lowercase().contains("at logon") || current_trigger.to_lowercase().contains("at startup") || current_trigger.to_lowercase().contains("when user logs on")) {
            let id = format!("task_scheduler_{}_{}", current_name.replace('\\', "_"), guid());
            entries.push(StartupEntry {
                id,
                name: current_name,
                path: current_path,
                location: "task_scheduler".to_string(),
                enabled: current_enabled,
            });
        }
    }
    entries
}

pub struct StartupService;

impl StartupService {
    pub fn list_entries() -> Vec<StartupEntry> {
        let mut entries = Vec::new();
        #[cfg(windows)]
        {
            entries.extend(read_run_key("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "Run", "hklm_run"));
            entries.extend(read_run_key("HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "Run", "hkcu_run"));
            entries.extend(read_run_key("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "RunOnce", "hklm_runonce"));
            entries.extend(read_run_key("HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "RunOnce", "hkcu_runonce"));
            entries.extend(read_run_key("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "RunServicesOnce", "hklm_runservicesonce"));
            entries.extend(read_run_key("HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "RunServicesOnce", "hkcu_runservicesonce"));
            entries.extend(read_disabled_run_key("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "Run", "hklm_run"));
            entries.extend(read_disabled_run_key("HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "Run", "hkcu_run"));
            entries.extend(read_disabled_run_key("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "RunOnce", "hklm_runonce"));
            entries.extend(read_disabled_run_key("HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "RunOnce", "hkcu_runonce"));
            entries.extend(read_disabled_run_key("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "RunServicesOnce", "hklm_runservicesonce"));
            entries.extend(read_disabled_run_key("HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", "RunServicesOnce", "hkcu_runservicesonce"));
            entries.extend(read_winlogon());
            entries.extend(read_startup_folder("%AppData%\\Microsoft\\Windows\\Start Menu\\Programs\\Startup", "startup_folder_user"));
            entries.extend(read_startup_folder("%ProgramData%\\Microsoft\\Windows\\Start Menu\\Programs\\Startup", "startup_folder_common"));
            entries.extend(read_task_scheduler());
        }
        #[cfg(not(windows))]
        {
            entries.push(StartupEntry {
                id: "demo_1".to_string(),
                name: "demo-app".to_string(),
                path: "/usr/bin/demo".to_string(),
                location: "demo".to_string(),
                enabled: true,
            });
        }
        entries
    }

    pub fn toggle_entry(id: &str) -> ApiResponse {
        #[cfg(windows)]
        {
            // парсим id: location_name_guid
            let parts: Vec<&str> = id.splitn(3, '_').collect();
            if parts.len() < 2 {
                return ApiResponse { success: false, message: "Неверный ID".to_string(), exists: None, data: None };
            }
            let loc = parts[0];
            let is_disabled = id.contains("_disabled_");
            match loc {
                "hklm_run" | "hkcu_run" | "hklm_runonce" | "hkcu_runonce" | "hklm_runservicesonce" | "hkcu_runservicesonce" => {
                    let hive = if loc.starts_with("hklm") { "HKLM" } else { "HKCU" };
                    let subkey = match loc {
                        "hklm_run" | "hkcu_run" => "Run",
                        "hklm_runonce" | "hkcu_runonce" => "RunOnce",
                        "hklm_runservicesonce" | "hkcu_runservicesonce" => "RunServicesOnce",
                        _ => "Run",
                    };
                    let src_key = if is_disabled {
                        format!("{}\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\{}-Disabled", hive, subkey)
                    } else {
                        format!("{}\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\{}", hive, subkey)
                    };
                    let dst_key = if is_disabled {
                        format!("{}\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\{}", hive, subkey)
                    } else {
                        format!("{}\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\{}-Disabled", hive, subkey)
                    };
                    // получаем имя записи
                    let name = if is_disabled { parts[2] } else { parts[1] };
                    // читаем значение
                    let out = Command::new("reg").args(["query", &src_key, "/v", name]).output();
                    let mut val = String::new();
                    if let Ok(o) = out {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        for line in stdout.lines() {
                            let line = line.trim();
                            if line.contains(name) && line.contains("REG_") {
                                let parts_line: Vec<&str> = line.splitn(3, "  ").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                                if parts_line.len() >= 3 {
                                    val = parts_line[2].to_string();
                                }
                            }
                        }
                    }
                    if val.is_empty() {
                        return ApiResponse { success: false, message: "Не удалось прочитать значение".to_string(), exists: None, data: None };
                    }
                    // удаляем из src
                    let _ = Command::new("reg").args(["delete", &src_key, "/v", name, "/f"]).output();
                    // добавляем в dst
                    let _ = Command::new("reg").args(["add", &dst_key, "/v", name, "/d", &val, "/f"]).output();
                    return ApiResponse { success: true, message: if is_disabled { "Запись включена" } else { "Запись отключена" }.to_string(), exists: None, data: None };
                }
                "winlogon_shell" | "winlogon_userinit" => {
                    return ApiResponse { success: false, message: "Winlogon параметры нельзя отключить через это меню".to_string(), exists: None, data: None };
                }
                "startup_folder_user" | "startup_folder_common" => {
                    let name = if is_disabled { parts[2] } else { parts[1] };
                    let folder = if loc == "startup_folder_user" {
                        std::env::var("APPDATA").unwrap_or_default() + "\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"
                    } else {
                        std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string()) + "\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"
                    };
                    let src = format!("{}\\{}{}", folder, name, if is_disabled { ".disabled" } else { "" });
                    let dst = format!("{}\\{}{}", folder, name, if is_disabled { "" } else { ".disabled" });
                    let _ = std::fs::rename(&src, &dst);
                    return ApiResponse { success: true, message: if is_disabled { "Запись включена" } else { "Запись отключена" }.to_string(), exists: None, data: None };
                }
                "task_scheduler" => {
                    let name = id.strip_prefix("task_scheduler_").unwrap_or("").rsplitn(2, '_').nth(1).unwrap_or("");
                    if name.is_empty() {
                        return ApiResponse { success: false, message: "Не удалось определить имя задачи".to_string(), exists: None, data: None };
                    }
                    let action = if is_disabled { "/ENABLE" } else { "/DISABLE" };
                    let out = Command::new("schtasks").args(["/change", "/tn", name, action]).output();
                    if let Ok(o) = out {
                        if o.status.success() {
                            return ApiResponse { success: true, message: if is_disabled { "Задача включена" } else { "Задача отключена" }.to_string(), exists: None, data: None };
                        }
                    }
                    return ApiResponse { success: false, message: "Не удалось изменить задачу".to_string(), exists: None, data: None };
                }
                _ => return ApiResponse { success: false, message: "Неизвестное расположение".to_string(), exists: None, data: None },
            }
        }
        #[cfg(not(windows))]
        {
            ApiResponse { success: true, message: "Демо-режим".to_string(), exists: None, data: None }
        }
    }

    pub fn delete_entry(id: &str) -> ApiResponse {
        #[cfg(windows)]
        {
            let parts: Vec<&str> = id.splitn(3, '_').collect();
            if parts.len() < 2 {
                return ApiResponse { success: false, message: "Неверный ID".to_string(), exists: None, data: None };
            }
            let loc = parts[0];
            let is_disabled = id.contains("_disabled_");
            match loc {
                "hklm_run" | "hkcu_run" | "hklm_runonce" | "hkcu_runonce" | "hklm_runservicesonce" | "hkcu_runservicesonce" => {
                    let hive = if loc.starts_with("hklm") { "HKLM" } else { "HKCU" };
                    let subkey = match loc {
                        "hklm_run" | "hkcu_run" => "Run",
                        "hklm_runonce" | "hkcu_runonce" => "RunOnce",
                        "hklm_runservicesonce" | "hkcu_runservicesonce" => "RunServicesOnce",
                        _ => "Run",
                    };
                    let key = if is_disabled {
                        format!("{}\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\{}-Disabled", hive, subkey)
                    } else {
                        format!("{}\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\{}", hive, subkey)
                    };
                    let name = if is_disabled { parts[2] } else { parts[1] };
                    let out = Command::new("reg").args(["delete", &key, "/v", name, "/f"]).output();
                    if let Ok(o) = out {
                        if o.status.success() {
                            return ApiResponse { success: true, message: "Запись удалена".to_string(), exists: None, data: None };
                        }
                    }
                    return ApiResponse { success: false, message: "Не удалось удалить запись".to_string(), exists: None, data: None };
                }
                "winlogon_shell" | "winlogon_userinit" => {
                    return ApiResponse { success: false, message: "Winlogon параметры нельзя удалить".to_string(), exists: None, data: None };
                }
                "startup_folder_user" | "startup_folder_common" => {
                    let name = if is_disabled { parts[2] } else { parts[1] };
                    let folder = if loc == "startup_folder_user" {
                        std::env::var("APPDATA").unwrap_or_default() + "\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"
                    } else {
                        std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string()) + "\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"
                    };
                    let path = format!("{}\\{}{}", folder, name, if is_disabled { ".disabled" } else { "" });
                    let _ = std::fs::remove_file(&path);
                    return ApiResponse { success: true, message: "Запись удалена".to_string(), exists: None, data: None };
                }
                "task_scheduler" => {
                    let name = id.strip_prefix("task_scheduler_").unwrap_or("").rsplitn(2, '_').nth(1).unwrap_or("");
                    if name.is_empty() {
                        return ApiResponse { success: false, message: "Не удалось определить имя задачи".to_string(), exists: None, data: None };
                    }
                    let out = Command::new("schtasks").args(["/delete", "/tn", name, "/f"]).output();
                    if let Ok(o) = out {
                        if o.status.success() {
                            return ApiResponse { success: true, message: "Задача удалена".to_string(), exists: None, data: None };
                        }
                    }
                    return ApiResponse { success: false, message: "Не удалось удалить задачу".to_string(), exists: None, data: None };
                }
                _ => return ApiResponse { success: false, message: "Неизвестное расположение".to_string(), exists: None, data: None },
            }
        }
        #[cfg(not(windows))]
        {
            ApiResponse { success: true, message: "Демо-режим".to_string(), exists: None, data: None }
        }
    }

    pub fn add_entry(req: StartupEntryRequest) -> ApiResponse {
        #[cfg(windows)]
        {
            match req.location.as_str() {
                "hkcu_run" => {
                    let out = Command::new("reg").args(["add", "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", &req.name, "/d", &req.path, "/f"]).output();
                    if let Ok(o) = out {
                        if o.status.success() {
                            return ApiResponse { success: true, message: "Запись добавлена".to_string(), exists: None, data: None };
                        }
                    }
                }
                "hklm_run" => {
                    let out = Command::new("reg").args(["add", "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", &req.name, "/d", &req.path, "/f"]).output();
                    if let Ok(o) = out {
                        if o.status.success() {
                            return ApiResponse { success: true, message: "Запись добавлена".to_string(), exists: None, data: None };
                        }
                    }
                }
                "startup_folder" => {
                    let folder = std::env::var("APPDATA").unwrap_or_default() + "\\Microsoft\\Windows\\Start Menu\\Programs\\Startup";
                    let path = format!("{}\\{}.lnk", folder, req.name);
                    // Создаём ярлык через PowerShell
                    let ps = format!(
                        "$WshShell = New-Object -comObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('{}'); $Shortcut.TargetPath = '{}'; $Shortcut.Save()",
                        path, req.path
                    );
                    let out = Command::new("powershell").args(["-Command", &ps]).output();
                    if let Ok(o) = out {
                        if o.status.success() {
                            return ApiResponse { success: true, message: "Ярлык создан".to_string(), exists: None, data: None };
                        }
                    }
                    return ApiResponse { success: false, message: "Не удалось создать ярлык".to_string(), exists: None, data: None };
                }
                _ => return ApiResponse { success: false, message: "Неизвестное расположение".to_string(), exists: None, data: None },
            }
            ApiResponse { success: false, message: "Не удалось добавить запись".to_string(), exists: None, data: None }
        }
        #[cfg(not(windows))]
        {
            ApiResponse { success: true, message: "Демо-режим".to_string(), exists: None, data: None }
        }
    }

    pub fn edit_entry(id: &str, req: StartupEntryRequest) -> ApiResponse {
        // удаляем старую и добавляем новую
        let del = Self::delete_entry(id);
        if !del.success {
            return del;
        }
        Self::add_entry(req)
    }
}
