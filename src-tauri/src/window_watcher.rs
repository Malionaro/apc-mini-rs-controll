use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crate::midi::MidiState;

use std::os::raw::c_void;
type HWND = *mut c_void;
type HANDLE = *mut c_void;
type DWORD = u32;
type BOOL = i32;
type WCHAR = u16;

#[link(name = "user32")]
extern "system" {
    fn GetForegroundWindow() -> HWND;
    fn GetWindowThreadProcessId(hwnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;
}

#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn QueryFullProcessImageNameW(hProcess: HANDLE, dwFlags: DWORD, lpExeName: *mut WCHAR, lpdwSize: *mut DWORD) -> BOOL;
}

const PROCESS_QUERY_LIMITED_INFORMATION: DWORD = 0x1000;

pub fn start_window_watcher(state: Arc<MidiState>) {
    thread::spawn(move || {
        let mut last_process = String::new();
        while *state.is_listening.lock().unwrap() {
            // Check active window every 1 second
            thread::sleep(Duration::from_secs(1));

            let enabled = { state.config.lock().unwrap().smart_profiles_enabled };
            if !enabled {
                continue;
            }

            unsafe {
                let hwnd = GetForegroundWindow();
                if !hwnd.is_null() {
                    let mut pid: DWORD = 0;
                    GetWindowThreadProcessId(hwnd, &mut pid);
                    if pid != 0 {
                        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
                        if !handle.is_null() {
                            let mut size: DWORD = 260;
                            let mut buffer = vec![0u16; 260];
                            if QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size) != 0 {
                                let path = String::from_utf16_lossy(&buffer[..size as usize]);
                                if let Some(exe_name) = std::path::Path::new(&path).file_name().and_then(|f| f.to_str()) {
                                    let exe_lower = exe_name.to_lowercase();
                                    if exe_lower != last_process {
                                        last_process = exe_lower.clone();
                                        
                                        let mut target_page = None;
                                        let current_active_page = {
                                            let config = state.config.lock().unwrap();
                                            for profile in &config.smart_profiles {
                                                if profile.process_name.to_lowercase() == exe_lower {
                                                    target_page = Some(profile.target_page.clone());
                                                    break;
                                                }
                                            }
                                            config.active_page.clone()
                                        };
                                        
                                        if let Some(page) = target_page {
                                            if page != current_active_page {
                                                let mut config = state.config.lock().unwrap();
                                                if config.pages.iter().any(|p| p.name == page) {
                                                    config.active_page = page.clone();
                                                    crate::midi::add_log(&state, format!("Smart Profile: Umschalten auf Seite '{}' (Prozess: {})", page, exe_name));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            CloseHandle(handle);
                        }
                    }
                }
            }
        }
    });
}
