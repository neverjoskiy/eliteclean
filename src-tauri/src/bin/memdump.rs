/// Утилита для дампа памяти javaw.exe
/// Запуск: cargo run --bin memdump
/// Результат:
///   memdump_found.txt  — найденные паттерны (если есть)
///   memdump_strings.txt — все printable строки длиной >= 6 символов (как strings.exe)

#[cfg(windows)]
fn main() {
    use std::io::Write;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
        PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_VM_READ, PROCESS_VM_OPERATION,
    };
    use windows::Win32::System::Memory::{
        VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT,
        PAGE_NOACCESS, PAGE_GUARD,
    };
    use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

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

    // Находим javaw.exe
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap() };
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut javaw_pid: Option<u32> = None;
    unsafe {
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szExeFile[..entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260)]
                );
                if name.to_lowercase() == "javaw.exe" {
                    javaw_pid = Some(entry.th32ProcessID);
                    println!("Найден javaw.exe PID={}", entry.th32ProcessID);
                    break;
                }
                if Process32NextW(snapshot, &mut entry).is_err() { break; }
            }
        }
        let _ = CloseHandle(snapshot);
    }

    let pid = match javaw_pid {
        Some(p) => p,
        None => { eprintln!("javaw.exe не найден"); return; }
    };

    let handle = unsafe {
        OpenProcess(PROCESS_VM_READ | PROCESS_VM_OPERATION, false, pid).unwrap()
    };

    let mut found_file = std::fs::File::create("memdump_found.txt").unwrap();
    let mut strings_file = std::fs::File::create("memdump_strings.txt").unwrap();
    let mut total_found = 0usize;
    let mut current: usize = 0;
    const MIN_STR_LEN: usize = 6;

    while current < 0x7FFFFFFFFFFF {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();
        let r = unsafe {
            VirtualQueryEx(handle, Some(current as *const _), &mut mbi, std::mem::size_of::<MEMORY_BASIC_INFORMATION>())
        };
        if r == 0 { break; }

        let accessible = mbi.State.0 == MEM_COMMIT.0
            && (mbi.Protect.0 & PAGE_NOACCESS.0) == 0
            && (mbi.Protect.0 & PAGE_GUARD.0) == 0;

        if accessible {
            let read_size = std::cmp::min(mbi.RegionSize, 10 * 1024 * 1024);
            let mut buf = vec![0u8; read_size];
            let mut bytes_read = 0usize;

            let ok = unsafe {
                ReadProcessMemory(handle, current as *const _, buf.as_mut_ptr() as *mut _, read_size, Some(&mut bytes_read)).is_ok()
            };

            if ok && bytes_read > 0 {
                let data = &buf[..bytes_read];

                // 1. Ищем целевые паттерны (UTF-8 и UTF-16)
                for pattern in &TARGET_STRINGS {
                    let mut start = 0;
                    while start + pattern.len() <= data.len() {
                        match data[start..].windows(pattern.len()).position(|w| w == *pattern) {
                            None => break,
                            Some(rel) => {
                                let addr = current + start + rel;
                                let line = format!(
                                    "UTF-8  | 0x{:016X} | protect=0x{:02X} | {:?}\n",
                                    addr, mbi.Protect.0,
                                    std::str::from_utf8(pattern).unwrap_or("?")
                                );
                                print!("{}", line);
                                found_file.write_all(line.as_bytes()).unwrap();
                                total_found += 1;
                                start += rel + pattern.len();
                            }
                        }
                    }

                    let p16: Vec<u8> = pattern.iter().flat_map(|&c| (c as u16).to_le_bytes()).collect();
                    let mut start = 0;
                    while start + p16.len() <= data.len() {
                        match data[start..].windows(p16.len()).position(|w| w == p16.as_slice()) {
                            None => break,
                            Some(rel) => {
                                let addr = current + start + rel;
                                let line = format!(
                                    "UTF-16 | 0x{:016X} | protect=0x{:02X} | {:?}\n",
                                    addr, mbi.Protect.0,
                                    std::str::from_utf8(pattern).unwrap_or("?")
                                );
                                print!("{}", line);
                                found_file.write_all(line.as_bytes()).unwrap();
                                total_found += 1;
                                start += rel + p16.len();
                            }
                        }
                    }
                }

                // 2. Извлекаем все printable ASCII строки >= MIN_STR_LEN (как утилита strings)
                let mut run_start: Option<usize> = None;
                for (i, &b) in data.iter().enumerate() {
                    let printable = b >= 0x20 && b < 0x7F;
                    if printable {
                        if run_start.is_none() { run_start = Some(i); }
                    } else {
                        if let Some(s) = run_start.take() {
                            let len = i - s;
                            if len >= MIN_STR_LEN {
                                let addr = current + s;
                                let s_str = std::str::from_utf8(&data[s..i]).unwrap_or("?");
                                let line = format!("0x{:016X} | {}\n", addr, s_str);
                                strings_file.write_all(line.as_bytes()).unwrap();
                            }
                        }
                    }
                }
                // flush последнюю строку
                if let Some(s) = run_start {
                    let len = bytes_read - s;
                    if len >= MIN_STR_LEN {
                        let addr = current + s;
                        let s_str = std::str::from_utf8(&data[s..bytes_read]).unwrap_or("?");
                        let line = format!("0x{:016X} | {}\n", addr, s_str);
                        strings_file.write_all(line.as_bytes()).unwrap();
                    }
                }
            }
        }

        current += mbi.RegionSize;
    }

    unsafe { let _ = CloseHandle(handle); }

    println!("\nЦелевых паттернов найдено: {}. Результаты в memdump_found.txt", total_found);
    println!("Все строки из памяти сохранены в memdump_strings.txt");
    println!("Ищи в memdump_strings.txt фрагменты из TARGET_STRINGS чтобы понять реальный формат хранения");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Только Windows");
}
