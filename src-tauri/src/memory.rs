//! Модуль для работы с памятью процессов на Windows
//! Аналог библиотеки pymem из Python

#[cfg(windows)]
pub struct MemoryCleaner;

#[cfg(windows)]
impl MemoryCleaner {
    /// Очистка целевых строк из памяти javaw.exe
    pub fn clean_javaw_memory() -> crate::models::CleanJavawResult {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
            PROCESSENTRY32W, TH32CS_SNAPPROCESS,
        };
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
        };
        use windows::Win32::System::Memory::{
            VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT,
            PAGE_READWRITE, PAGE_EXECUTE_READWRITE,
        };
        use windows::Win32::System::Diagnostics::Debug::{
            ReadProcessMemory, WriteProcessMemory,
        };
        
        const TARGET_STRINGS: [&[u8]; 15] = [
            b"OgUwQPNl",
            b"oGUqpcAZTe",
            b"ovxiXMKoGUAc",
            b"RbVJsoGuiS",
            b"huKhKgjtoGUh]T",
            b"RoCQXjiLhWcfmsb",
            b"tXcNogulsu",
            b"oGUDpcYLI",
            b"PiSOGUNKFtgu",
            b"fbKomagcVoGUv",
            b"CQauDfNVDeQv_xfM`Bn",
            b"+$L\"<d\"d4!7BKMhc0",
            b"7JUBRL5EO!N",
            b"QLMtl_vQTL",
            b"IZn]laU",
        ];
        
        // Находим процесс javaw.exe
        let snapshot = unsafe {
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()
        };
        
        let snapshot = match snapshot {
            Some(s) => s,
            None => {
                return crate::models::CleanJavawResult {
                    success: false,
                    message: "Не удалось создать снапшот процессов".to_string(),
                    regions_scanned: 0,
                    regions_matched: 0,
                    cleared_count: 0,
                };
            }
        };
        
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        
        let mut javaw_pid: Option<u32> = None;
        
        unsafe {
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let exe_name = String::from_utf16_lossy(&entry.szExeFile);
                    if exe_name.to_lowercase() == "javaw.exe" {
                        javaw_pid = Some(entry.th32ProcessID);
                        break;
                    }
                    
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
        }
        
        unsafe { let _ = CloseHandle(snapshot); }
        
        let pid = match javaw_pid {
            Some(p) => p,
            None => {
                return crate::models::CleanJavawResult {
                    success: false,
                    message: "Процесс javaw.exe не найден".to_string(),
                    regions_scanned: 0,
                    regions_matched: 0,
                    cleared_count: 0,
                };
            }
        };
        
        // Открываем процесс с нужными правами
        let process_handle = unsafe {
            OpenProcess(PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE, false, pid)
        };
        
        let process_handle = match process_handle {
            Ok(h) => h,
            Err(_) => {
                return crate::models::CleanJavawResult {
                    success: false,
                    message: format!("Не удалось открыть процесс javaw.exe (PID: {}). Требуется запуск от администратора.", pid),
                    regions_scanned: 0,
                    regions_matched: 0,
                    cleared_count: 0,
                };
            }
        };
        
        log::info!("Подключено к javaw.exe (PID: {})", pid);
        
        let mut cleared_count = 0;
        let mut regions_scanned = 0;
        let mut regions_matched = 0;
        
        // Сканируем память процесса
        let mut current_address: usize = 0;
        let max_address: usize = 0x7FFFFFFFFFFF;
        
        while current_address < max_address {
            let mut mbi = MEMORY_BASIC_INFORMATION::default();
            
            let result = unsafe {
                VirtualQueryEx(
                    process_handle,
                    Some(current_address as *const _),
                    &mut mbi,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            
            if result == 0 {
                break;
            }
            
            regions_scanned += 1;
            
            // Проверяем регион на читаемость/записываемость
            if mbi.State.0 == MEM_COMMIT.0 
                && (mbi.Protect.0 == PAGE_READWRITE.0 || mbi.Protect.0 == PAGE_EXECUTE_READWRITE.0) 
            {
                let region_size = mbi.RegionSize;
                let read_size = std::cmp::min(region_size, 10 * 1024 * 1024); // Максимум 10MB за раз
                
                // Читаем регион
                let mut buffer = vec![0u8; read_size];
                let mut bytes_read: usize = 0;
                
                let read_result = unsafe {
                    ReadProcessMemory(
                        process_handle,
                        current_address as *const _,
                        buffer.as_mut_ptr() as *mut _,
                        read_size,
                        Some(&mut bytes_read),
                    )
                };
                
                if read_result.is_ok() && bytes_read > 0 {
                    let region_data = &buffer[..bytes_read];
                    let mut found_in_region = 0;
                    
                    // Ищем паттерны
                    for pattern in &TARGET_STRINGS {
                        // UTF-8 вариант
                        let mut start = 0;
                        while let Some(idx) = region_data.windows(pattern.len()).position(|w| w == *pattern) {
                            if idx < start {
                                break;
                            }
                            
                            let target_addr = current_address + idx;
                            log::info!("Найден паттерн по адресу 0x{:X}", target_addr);
                            
                            // Генерируем случайные байты для замены
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            let random_bytes: Vec<u8> = (0..pattern.len()).map(|_| rng.gen()).collect();
                            
                            // Записываем случайные байты
                            let write_result = unsafe {
                                WriteProcessMemory(
                                    process_handle,
                                    target_addr as *const _,
                                    random_bytes.as_ptr() as *const _,
                                    random_bytes.len(),
                                    None,
                                )
                            };
                            
                            if write_result.is_ok() {
                                cleared_count += 1;
                                found_in_region += 1;
                            }
                            
                            start = idx + pattern.len();
                        }
                        
                        // UTF-16 LE вариант
                        let pattern_utf16: Vec<u8> = pattern.iter()
                            .flat_map(|&c| (c as u16).to_le_bytes())
                            .collect();
                        
                        start = 0;
                        while let Some(idx) = region_data.windows(pattern_utf16.len()).position(|w| w == pattern_utf16.as_slice()) {
                            if idx < start {
                                break;
                            }
                            
                            let target_addr = current_address + idx;
                            log::info!("Найден UTF-16 паттерн по адресу 0x{:X}", target_addr);
                            
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            let random_bytes: Vec<u8> = (0..pattern_utf16.len()).map(|_| rng.gen()).collect();
                            
                            let write_result = unsafe {
                                WriteProcessMemory(
                                    process_handle,
                                    target_addr as *const _,
                                    random_bytes.as_ptr() as *const _,
                                    random_bytes.len(),
                                    None,
                                )
                            };
                            
                            if write_result.is_ok() {
                                cleared_count += 1;
                                found_in_region += 1;
                            }
                            
                            start = idx + pattern_utf16.len();
                        }
                    }
                    
                    if found_in_region > 0 {
                        regions_matched += 1;
                        log::info!("  -> Удалено {} совпадений в регионе", found_in_region);
                    }
                }
            }
            
            current_address += mbi.RegionSize;
        }
        
        unsafe { let _ = CloseHandle(process_handle); }
        
        let result_msg = format!(
            "Очистка завершена. Регионов просканировано: {}, совпадений удалено: {}",
            regions_scanned, cleared_count
        );
        
        log::info!("{}", result_msg);
        
        crate::models::CleanJavawResult {
            success: true,
            message: result_msg,
            regions_scanned,
            regions_matched,
            cleared_count,
        }
    }
}
