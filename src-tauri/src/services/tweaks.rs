//! Системные твики через реестр Windows

use crate::models::{TweakInfo, TweakApplyResult};

pub struct TweaksService;

impl TweaksService {

    /// Получить список всех твиков с их текущим состоянием
    pub fn get_tweaks() -> Vec<TweakInfo> {
        vec![
            TweakInfo {
                id: "game_bar_dvr".to_string(),
                name: "Откл. Game Bar и DVR".to_string(),
                description: "Отключает захват экрана и игровую панель Xbox".to_string(),
                danger: false,
                applied: Self::is_game_bar_disabled(),
            },
            TweakInfo {
                id: "hags".to_string(),
                name: "HAGS (аппаратное планирование GPU)".to_string(),
                description: "Включает Hardware-Accelerated GPU Scheduling. Может нестабильно работать на старых драйверах".to_string(),
                danger: true,
                applied: Self::is_hags_enabled(),
            },
            TweakInfo {
                id: "telemetry".to_string(),
                name: "Откл. телеметрию".to_string(),
                description: "Запрещает сбор диагностических данных Windows".to_string(),
                danger: false,
                applied: Self::is_telemetry_disabled(),
            },
            TweakInfo {
                id: "cortana".to_string(),
                name: "Убить Кортану".to_string(),
                description: "Полностью отключает Cortana через групповую политику".to_string(),
                danger: false,
                applied: Self::is_cortana_disabled(),
            },
            TweakInfo {
                id: "multimedia_priority".to_string(),
                name: "Приоритет мультимедийных задач".to_string(),
                description: "SystemResponsiveness=0, NetworkThrottlingIndex=max — максимум ресурсов для игр".to_string(),
                danger: false,
                applied: Self::is_multimedia_priority_set(),
            },
            TweakInfo {
                id: "timer_resolution".to_string(),
                name: "Высокое разрешение таймера".to_string(),
                description: "GlobalTimerResolutionRequests=1 — снижает задержки планировщика".to_string(),
                danger: false,
                applied: Self::is_timer_resolution_set(),
            },
            TweakInfo {
                id: "uwp_background".to_string(),
                name: "Откл. фоновые UWP-приложения".to_string(),
                description: "Запрещает всем UWP-приложениям работать в фоне".to_string(),
                danger: false,
                applied: Self::is_uwp_background_disabled(),
            },
            TweakInfo {
                id: "cpu_parking".to_string(),
                name: "Откл. CPU Parking".to_string(),
                description: "Запрещает отключение ядер процессора — все ядра всегда активны".to_string(),
                danger: false,
                applied: Self::is_cpu_parking_disabled(),
            },
            TweakInfo {
                id: "power_throttling".to_string(),
                name: "Откл. Power Throttling".to_string(),
                description: "Отключает троттлинг процессора для экономии энергии".to_string(),
                danger: false,
                applied: Self::is_power_throttling_disabled(),
            },
            TweakInfo {
                id: "priority_separation".to_string(),
                name: "Приоритет активному окну".to_string(),
                description: "Win32PrioritySeparation=38 — больше ресурсов активной игре".to_string(),
                danger: false,
                applied: Self::is_priority_separation_set(),
            },
        ]
    }

    /// Применить твик по id
    pub fn apply_tweak(id: &str) -> TweakApplyResult {
        match id {
            "game_bar_dvr"        => Self::apply_game_bar_dvr(),
            "hags"                => Self::apply_hags(),
            "telemetry"           => Self::apply_telemetry(),
            "cortana"             => Self::apply_cortana(),
            "multimedia_priority" => Self::apply_multimedia_priority(),
            "timer_resolution"    => Self::apply_timer_resolution(),
            "uwp_background"      => Self::apply_uwp_background(),
            "cpu_parking"         => Self::apply_cpu_parking(),
            "power_throttling"    => Self::apply_power_throttling(),
            "priority_separation" => Self::apply_priority_separation(),
            _ => TweakApplyResult { success: false, message: format!("Неизвестный твик: {}", id), applied: false },
        }
    }

    /// Откатить твик по id
    pub fn revert_tweak(id: &str) -> TweakApplyResult {
        match id {
            "game_bar_dvr"        => Self::revert_game_bar_dvr(),
            "hags"                => Self::revert_hags(),
            "telemetry"           => Self::revert_telemetry(),
            "cortana"             => Self::revert_cortana(),
            "multimedia_priority" => Self::revert_multimedia_priority(),
            "timer_resolution"    => Self::revert_timer_resolution(),
            "uwp_background"      => Self::revert_uwp_background(),
            "cpu_parking"         => Self::revert_cpu_parking(),
            "power_throttling"    => Self::revert_power_throttling(),
            "priority_separation" => Self::revert_priority_separation(),
            _ => TweakApplyResult { success: false, message: format!("Неизвестный твик: {}", id), applied: true },
        }
    }

    // ── Проверки текущего состояния ──

    fn is_game_bar_disabled() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKCU", r"SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled")
            .map(|v| v == 0).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    fn is_hags_enabled() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKLM", r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers", "HwSchMode")
            .map(|v| v == 2).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    fn is_telemetry_disabled() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKLM", r"SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry")
            .map(|v| v == 0).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    fn is_cortana_disabled() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKLM", r"SOFTWARE\Policies\Microsoft\Windows\Windows Search", "AllowCortana")
            .map(|v| v == 0).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    fn is_multimedia_priority_set() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKLM", r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile", "SystemResponsiveness")
            .map(|v| v == 0).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    fn is_timer_resolution_set() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKLM", r"SYSTEM\CurrentControlSet\Control\Session Manager\kernel", "GlobalTimerResolutionRequests")
            .map(|v| v == 1).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    fn is_uwp_background_disabled() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKCU", r"SOFTWARE\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications", "GlobalUserDisabled")
            .map(|v| v == 1).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    fn is_cpu_parking_disabled() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKLM", r"SYSTEM\CurrentControlSet\Control\Power\PowerSettings\54533251-82be-4824-96c1-47b60b740d00\0cc5b647-c1df-4637-891a-dec35c318583", "ValueMax")
            .map(|v| v == 0).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    fn is_power_throttling_disabled() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKLM", r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling", "PowerThrottlingOff")
            .map(|v| v == 1).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    fn is_priority_separation_set() -> bool {
        #[cfg(windows)]
        { reg_get_dword("HKLM", r"SYSTEM\CurrentControlSet\Control\PriorityControl", "Win32PrioritySeparation")
            .map(|v| v == 38).unwrap_or(false) }
        #[cfg(not(windows))] { false }
    }

    // ── Применение твиков ──

    fn apply_game_bar_dvr() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let r1 = reg_set_dword("HKCU", r"SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled", 0);
            let r2 = reg_set_dword("HKCU", r"System\GameConfigStore", "GameDVR_Enabled", 0);
            if r1 && r2 {
                TweakApplyResult { success: true, message: "Game Bar и DVR отключены".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Не удалось записать в реестр".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_game_bar_dvr() -> TweakApplyResult {
        #[cfg(windows)]
        {
            reg_set_dword("HKCU", r"SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled", 1);
            reg_set_dword("HKCU", r"System\GameConfigStore", "GameDVR_Enabled", 1);
            TweakApplyResult { success: true, message: "Game Bar и DVR включены".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }

    fn apply_hags() -> TweakApplyResult {
        #[cfg(windows)]
        {
            if reg_set_dword("HKLM", r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers", "HwSchMode", 2) {
                TweakApplyResult { success: true, message: "HAGS включён (требуется перезагрузка)".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Нет прав администратора или ошибка записи".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_hags() -> TweakApplyResult {
        #[cfg(windows)]
        {
            reg_set_dword("HKLM", r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers", "HwSchMode", 1);
            TweakApplyResult { success: true, message: "HAGS отключён (требуется перезагрузка)".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }

    fn apply_telemetry() -> TweakApplyResult {
        #[cfg(windows)]
        {
            if reg_set_dword("HKLM", r"SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry", 0) {
                TweakApplyResult { success: true, message: "Телеметрия отключена".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Нет прав администратора".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_telemetry() -> TweakApplyResult {
        #[cfg(windows)]
        {
            reg_set_dword("HKLM", r"SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry", 1);
            TweakApplyResult { success: true, message: "Телеметрия включена".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }

    fn apply_cortana() -> TweakApplyResult {
        #[cfg(windows)]
        {
            if reg_set_dword("HKLM", r"SOFTWARE\Policies\Microsoft\Windows\Windows Search", "AllowCortana", 0) {
                TweakApplyResult { success: true, message: "Cortana отключена".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Нет прав администратора".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_cortana() -> TweakApplyResult {
        #[cfg(windows)]
        {
            reg_set_dword("HKLM", r"SOFTWARE\Policies\Microsoft\Windows\Windows Search", "AllowCortana", 1);
            TweakApplyResult { success: true, message: "Cortana включена".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }

    fn apply_multimedia_priority() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
            let r1 = reg_set_dword("HKLM", key, "SystemResponsiveness", 0);
            let r2 = reg_set_dword("HKLM", key, "NetworkThrottlingIndex", 0xFFFFFFFF);
            if r1 && r2 {
                TweakApplyResult { success: true, message: "Приоритет мультимедиа установлен".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Нет прав администратора".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_multimedia_priority() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
            reg_set_dword("HKLM", key, "SystemResponsiveness", 20);
            reg_set_dword("HKLM", key, "NetworkThrottlingIndex", 10);
            TweakApplyResult { success: true, message: "Приоритет мультимедиа сброшен".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }

    fn apply_timer_resolution() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SYSTEM\CurrentControlSet\Control\Session Manager\kernel";
            if reg_set_dword("HKLM", key, "GlobalTimerResolutionRequests", 1) {
                TweakApplyResult { success: true, message: "Высокое разрешение таймера включено".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Нет прав администратора".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_timer_resolution() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SYSTEM\CurrentControlSet\Control\Session Manager\kernel";
            reg_set_dword("HKLM", key, "GlobalTimerResolutionRequests", 0);
            TweakApplyResult { success: true, message: "Разрешение таймера сброшено".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }

    fn apply_uwp_background() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SOFTWARE\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications";
            if reg_set_dword("HKCU", key, "GlobalUserDisabled", 1) {
                TweakApplyResult { success: true, message: "Фоновые UWP-приложения отключены".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Ошибка записи в реестр".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_uwp_background() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SOFTWARE\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications";
            reg_set_dword("HKCU", key, "GlobalUserDisabled", 0);
            TweakApplyResult { success: true, message: "Фоновые UWP-приложения включены".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }

    fn apply_cpu_parking() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SYSTEM\CurrentControlSet\Control\Power\PowerSettings\54533251-82be-4824-96c1-47b60b740d00\0cc5b647-c1df-4637-891a-dec35c318583";
            if reg_set_dword("HKLM", key, "ValueMax", 0) {
                TweakApplyResult { success: true, message: "CPU Parking отключён (требуется перезагрузка)".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Нет прав администратора".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_cpu_parking() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SYSTEM\CurrentControlSet\Control\Power\PowerSettings\54533251-82be-4824-96c1-47b60b740d00\0cc5b647-c1df-4637-891a-dec35c318583";
            reg_set_dword("HKLM", key, "ValueMax", 100);
            TweakApplyResult { success: true, message: "CPU Parking включён (требуется перезагрузка)".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }

    fn apply_power_throttling() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling";
            if reg_set_dword("HKLM", key, "PowerThrottlingOff", 1) {
                TweakApplyResult { success: true, message: "Power Throttling отключён".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Нет прав администратора".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_power_throttling() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling";
            reg_set_dword("HKLM", key, "PowerThrottlingOff", 0);
            TweakApplyResult { success: true, message: "Power Throttling включён".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }

    fn apply_priority_separation() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SYSTEM\CurrentControlSet\Control\PriorityControl";
            if reg_set_dword("HKLM", key, "Win32PrioritySeparation", 38) {
                TweakApplyResult { success: true, message: "Приоритет активному окну установлен".to_string(), applied: true }
            } else {
                TweakApplyResult { success: false, message: "Нет прав администратора".to_string(), applied: false }
            }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: false } }
    }

    fn revert_priority_separation() -> TweakApplyResult {
        #[cfg(windows)]
        {
            let key = r"SYSTEM\CurrentControlSet\Control\PriorityControl";
            reg_set_dword("HKLM", key, "Win32PrioritySeparation", 2);
            TweakApplyResult { success: true, message: "Приоритет окон сброшен на дефолт".to_string(), applied: false }
        }
        #[cfg(not(windows))]
        { TweakApplyResult { success: false, message: "Только Windows".to_string(), applied: true } }
    }
}

// ── Вспомогательные функции реестра ──

#[cfg(windows)]
fn reg_get_dword(hive: &str, subkey: &str, value: &str) -> Option<u32> {
    use std::process::Command;
    let hive_path = format!("{}\\{}", hive, subkey);
    let out = Command::new("reg")
        .args(["query", &hive_path, "/v", value])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Парсим строку вида: "    ValueName    REG_DWORD    0x0"
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0].eq_ignore_ascii_case(value) {
            if let Some(hex) = parts.last() {
                let hex = hex.trim_start_matches("0x").trim_start_matches("0X");
                return u32::from_str_radix(hex, 16).ok();
            }
        }
    }
    None
}

#[cfg(windows)]
fn reg_set_dword(hive: &str, subkey: &str, value: &str, data: u32) -> bool {
    use std::process::Command;
    let hive_path = format!("{}\\{}", hive, subkey);
    // Сначала создаём ключ если не существует
    let _ = Command::new("reg")
        .args(["add", &hive_path, "/f"])
        .output();
    // reg.exe принимает /d только как десятичное, но для u32::MAX нужен hex через PowerShell
    // Используем PowerShell для надёжности
    let ps_cmd = format!(
        "Set-ItemProperty -Path 'Registry::{}\\{}' -Name '{}' -Value {} -Type DWord -Force",
        hive, subkey, value, data
    );
    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
