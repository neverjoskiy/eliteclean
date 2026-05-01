//! Сетевые операции очистки

use tauri::State;
use crate::state::SharedAppState;
use crate::models::NetworkCleanResponse;

pub struct NetworkService;

impl NetworkService {
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
            s.add_log(
                format!("Сброс сети: {}/{} успешно", ok, details.len()),
                if success { "success" } else { "error" }.to_string(),
            );
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
}
