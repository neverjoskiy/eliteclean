//! Сервис управления процессами: список, приоритет, заморозка, разморозка, убийство

use tauri::State;
use crate::state::SharedAppState;
use crate::models::{ProcessInfo, ProcessListResponse, ProcessActionResponse};

pub struct ProcessService;

#[cfg(windows)]
impl ProcessService {
    pub async fn list_processes(state: State<'_, SharedAppState>) -> ProcessListResponse {
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
            PROCESSENTRY32W, TH32CS_SNAPPROCESS,
        };
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
            PROCESS_SUSPEND_RESUME, PROCESS_TERMINATE,
        };

        let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok() } {
            Some(s) => s,
            None => {
                return ProcessListResponse { processes: Vec::new(), total: 0 };
            }
        };

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut processes = Vec::new();
        let access_rights = PROCESS_QUERY_INFORMATION | PROCESS_SET_INFORMATION | PROCESS_SUSPEND_RESUME | PROCESS_TERMINATE;

        unsafe {
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let exe_name = String::from_utf16_lossy(
                        &entry.szExeFile[..entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len())]
                    );

                    let pid = entry.th32ProcessID;
                    let thread_count = entry.cntThreads;

                    let (memory_mb, priority, priority_class) = if pid != 0 {
                        if let Ok(handle) = OpenProcess(access_rights, false, pid) {
                            let mem = Self::get_memory_mb(handle);
                            let (name, raw) = Self::get_priority_info(handle);
                            let _ = CloseHandle(handle);
                            (mem, name, raw)
                        } else {
                            (0.0, Self::priority_name(0), 0)
                        }
                    } else {
                        (0.0, Self::priority_name(0), 0)
                    };

                    let is_suspended = Self::check_suspended(pid);

                    processes.push(ProcessInfo {
                        pid,
                        name: exe_name,
                        memory_mb,
                        priority,
                        priority_class,
                        is_suspended,
                        thread_count,
                    });

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
        }

        let _ = unsafe { CloseHandle(snapshot) };

        processes.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap_or(std::cmp::Ordering::Equal));

        let total = processes.len();
        {
            let mut s = state.write().await;
            s.add_log(format!("загружено {} процессов", total), "info".to_string());
        }

        ProcessListResponse { processes, total }
    }

    pub async fn set_priority(pid: u32, priority_class: u32, state: State<'_, SharedAppState>) -> ProcessActionResponse {
        use windows::Win32::System::Threading::{OpenProcess, SetPriorityClass, PROCESS_SET_INFORMATION, PROCESS_CREATION_FLAGS};
        use windows::Win32::Foundation::CloseHandle;

        let handle = match unsafe { OpenProcess(PROCESS_SET_INFORMATION, false, pid) } {
            Ok(h) => h,
            Err(_) => {
                return ProcessActionResponse {
                    success: false,
                    message: format!("Не удалось открыть процесс (PID: {})", pid),
                };
            }
        };

        let pc = PROCESS_CREATION_FLAGS(priority_class);
        let result = unsafe { SetPriorityClass(handle, pc) };

        let _ = unsafe { CloseHandle(handle) };

        if result.is_ok() {
            let msg = format!("Приоритет PID {} изменён на {}", pid, Self::priority_name(priority_class));
            {
                let mut s = state.write().await;
                s.add_log(msg.clone(), "success".to_string());
            }
            ProcessActionResponse { success: true, message: msg }
        } else {
            ProcessActionResponse {
                success: false,
                message: "Не удалось изменить приоритет".to_string(),
            }
        }
    }

    pub async fn suspend_process(pid: u32, state: State<'_, SharedAppState>) -> ProcessActionResponse {
        use windows::Win32::System::Threading::OpenProcess;
        use windows::Win32::System::Threading::PROCESS_SUSPEND_RESUME;
        use windows::Win32::Foundation::CloseHandle;

        let handle = match unsafe { OpenProcess(PROCESS_SUSPEND_RESUME, false, pid) } {
            Ok(h) => h,
            Err(_) => {
                return ProcessActionResponse {
                    success: false,
                    message: format!("Не удалось открыть процесс (PID: {})", pid),
                };
            }
        };

        let status = unsafe {
            ntapi::ntpsapi::NtSuspendProcess(handle.0 as ntapi::winapi::um::winnt::HANDLE)
        };

        let _ = unsafe { CloseHandle(handle) };

        if status == 0 {
            let msg = format!("Процесс {} заморожен", pid);
            {
                let mut s = state.write().await;
                s.add_log(msg.clone(), "success".to_string());
            }
            ProcessActionResponse { success: true, message: msg }
        } else {
            ProcessActionResponse {
                success: false,
                message: format!("Не удалось заморозить процесс (PID: {})", pid),
            }
        }
    }

    pub async fn resume_process(pid: u32, state: State<'_, SharedAppState>) -> ProcessActionResponse {
        use windows::Win32::System::Threading::OpenProcess;
        use windows::Win32::System::Threading::PROCESS_SUSPEND_RESUME;
        use windows::Win32::Foundation::CloseHandle;

        let handle = match unsafe { OpenProcess(PROCESS_SUSPEND_RESUME, false, pid) } {
            Ok(h) => h,
            Err(_) => {
                return ProcessActionResponse {
                    success: false,
                    message: format!("Не удалось открыть процесс (PID: {})", pid),
                };
            }
        };

        let status = unsafe {
            ntapi::ntpsapi::NtResumeProcess(handle.0 as ntapi::winapi::um::winnt::HANDLE)
        };

        let _ = unsafe { CloseHandle(handle) };

        if status == 0 {
            let msg = format!("Процесс {} разморожен", pid);
            {
                let mut s = state.write().await;
                s.add_log(msg.clone(), "success".to_string());
            }
            ProcessActionResponse { success: true, message: msg }
        } else {
            ProcessActionResponse {
                success: false,
                message: format!("Не удалось разморозить процесс (PID: {})", pid),
            }
        }
    }

    pub async fn kill_process(pid: u32, state: State<'_, SharedAppState>) -> ProcessActionResponse {
        use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
        use windows::Win32::Foundation::CloseHandle;

        let handle = match unsafe { OpenProcess(PROCESS_TERMINATE, false, pid) } {
            Ok(h) => h,
            Err(_) => {
                return ProcessActionResponse {
                    success: false,
                    message: format!("Не удалось открыть процесс (PID: {})", pid),
                };
            }
        };

        let result = unsafe { TerminateProcess(handle, 1) };
        let _ = unsafe { CloseHandle(handle) };

        if result.is_ok() {
            let msg = format!("Процесс {} завершён", pid);
            {
                let mut s = state.write().await;
                s.add_log(msg.clone(), "success".to_string());
            }
            ProcessActionResponse { success: true, message: msg }
        } else {
            ProcessActionResponse {
                success: false,
                message: format!("Не удалось завершить процесс (PID: {})", pid),
            }
        }
    }

    fn get_memory_mb(handle: windows::Win32::Foundation::HANDLE) -> f64 {
        use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

        let mut counters = PROCESS_MEMORY_COUNTERS::default();
        let result = unsafe {
            GetProcessMemoryInfo(
                handle,
                &mut counters as *mut _ as *mut _,
                std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            )
        };
        if result.is_ok() {
            counters.WorkingSetSize as f64 / (1024.0 * 1024.0)
        } else {
            0.0
        }
    }

    fn get_priority_info(handle: windows::Win32::Foundation::HANDLE) -> (String, u32) {
        use windows::Win32::System::Threading::GetPriorityClass;
        let raw = unsafe { GetPriorityClass(handle) };
        if raw != 0 {
            (Self::priority_name(raw), raw)
        } else {
            (Self::priority_name(0), 0)
        }
    }

    fn check_suspended(pid: u32) -> bool {
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Thread32First, Thread32Next,
            TH32CS_SNAPTHREAD, THREADENTRY32,
        };
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenThread, ResumeThread, SuspendThread, THREAD_SUSPEND_RESUME};

        let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).ok() } {
            Some(s) => s,
            None => return false,
        };

        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };

        let mut first_thread_id: Option<u32> = None;

        unsafe {
            if Thread32First(snapshot, &mut entry).is_ok() {
                loop {
                    if entry.th32OwnerProcessID == pid && first_thread_id.is_none() {
                        first_thread_id = Some(entry.th32ThreadID);
                        break;
                    }
                    if Thread32Next(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
        }

        let _ = unsafe { CloseHandle(snapshot) };

        let tid = match first_thread_id {
            Some(t) => t,
            None => return false,
        };

        let thread_handle = match unsafe { OpenThread(THREAD_SUSPEND_RESUME, false, tid) } {
            Ok(h) => h,
            Err(_) => return false,
        };

        let prev_count = unsafe { ResumeThread(thread_handle) };
        if prev_count > 0 {
            let _ = unsafe { SuspendThread(thread_handle) };
        }
        let _ = unsafe { CloseHandle(thread_handle) };

        prev_count > 0
    }

    pub fn priority_name(class: u32) -> String {
        match class {
            0x00000100 => "реального времени".to_string(),
            0x00000080 => "высокий".to_string(),
            0x00008000 => "выше среднего".to_string(),
            0x00000020 => "нормальный".to_string(),
            0x00004000 => "ниже среднего".to_string(),
            0x00000010 => "низкий".to_string(),
            _ => "неизвестный".to_string(),
        }
    }
}

#[cfg(not(windows))]
impl ProcessService {
    pub async fn list_processes(_state: State<'_, SharedAppState>) -> ProcessListResponse {
        ProcessListResponse { processes: Vec::new(), total: 0 }
    }

    pub async fn set_priority(_pid: u32, _priority_class: u32, _state: State<'_, SharedAppState>) -> ProcessActionResponse {
        ProcessActionResponse { success: false, message: "Только Windows".to_string() }
    }

    pub async fn suspend_process(_pid: u32, _state: State<'_, SharedAppState>) -> ProcessActionResponse {
        ProcessActionResponse { success: false, message: "Только Windows".to_string() }
    }

    pub async fn resume_process(_pid: u32, _state: State<'_, SharedAppState>) -> ProcessActionResponse {
        ProcessActionResponse { success: false, message: "Только Windows".to_string() }
    }

    pub async fn kill_process(_pid: u32, _state: State<'_, SharedAppState>) -> ProcessActionResponse {
        ProcessActionResponse { success: false, message: "Только Windows".to_string() }
    }

    pub fn priority_name(_class: u32) -> String {
        "неизвестный".to_string()
    }
}
