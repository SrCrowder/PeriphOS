//! Win32 implementation.

use periphos_core::apps::{file_name, AppIdentity};
use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_SUCCESS, HANDLE, HWND, LPARAM};
use windows::Win32::Storage::Packaging::Appx::GetPackageFamilyName;
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindow, GetWindowLongW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible, GWL_EXSTYLE, GW_OWNER, WS_EX_TOOLWINDOW,
};

/// Process handle closed on drop.
struct Process(HANDLE);

impl Process {
    fn open(pid: u32) -> Option<Self> {
        // SAFETY: plain Win32 call; the handle is owned by the returned value.
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
            .ok()
            .map(Self)
    }

    fn image_path(&self) -> Option<String> {
        let mut buffer = vec![0u16; 1024];
        let mut size = buffer.len() as u32;
        // SAFETY: `buffer` holds `size` UTF-16 units and outlives the call.
        unsafe {
            QueryFullProcessImageNameW(
                self.0,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            )
        }
        .ok()?;
        Some(String::from_utf16_lossy(&buffer[..size as usize]))
    }

    /// Package Family Name, for processes that run with package identity
    /// (Microsoft Store and Game Pass apps).
    fn package_family(&self) -> Option<String> {
        let mut buffer = vec![0u16; 256];
        let mut length = buffer.len() as u32;
        // SAFETY: `buffer` holds `length` UTF-16 units and outlives the call.
        let result =
            unsafe { GetPackageFamilyName(self.0, &mut length, Some(PWSTR(buffer.as_mut_ptr()))) };
        if result != ERROR_SUCCESS || length == 0 {
            return None; // APPMODEL_ERROR_NO_PACKAGE for ordinary desktop programs
        }
        // `length` includes the terminating NUL.
        let text = String::from_utf16_lossy(&buffer[..(length as usize).saturating_sub(1)]);
        (!text.is_empty()).then_some(text)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        // SAFETY: the handle came from OpenProcess and is closed exactly once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn window_title(hwnd: HWND) -> String {
    let mut buffer = [0u16; 512];
    // SAFETY: the slice is valid for the duration of the call.
    let len = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn window_pid(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    // SAFETY: `pid` outlives the call.
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid
}

fn identify(hwnd: HWND) -> Option<AppIdentity> {
    let pid = window_pid(hwnd);
    // SAFETY: no preconditions.
    if pid == 0 || pid == unsafe { GetCurrentProcessId() } {
        return None;
    }
    let process = Process::open(pid)?;
    let exe_path = process.image_path();
    let exe_name = file_name(exe_path.as_deref()?).to_owned();
    Some(AppIdentity {
        exe_name,
        exe_path,
        package_family: process.package_family(),
        title: window_title(hwnd),
    })
}

pub fn foreground_app() -> Option<AppIdentity> {
    // SAFETY: no preconditions.
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return None;
    }
    identify(hwnd)
}

/// A window the user would recognise as an application: visible, titled, not
/// owned by another window and not a tool window.
fn is_app_window(hwnd: HWND) -> bool {
    // SAFETY: plain queries on a window handle provided by EnumWindows.
    unsafe {
        IsWindowVisible(hwnd).as_bool()
            && GetWindow(hwnd, GW_OWNER).map_or(true, |owner| owner.is_invalid())
            && (GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0) == 0
    }
}

unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: `lparam` is the `&mut Vec<HWND>` passed to EnumWindows below,
    // which outlives the enumeration.
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<HWND>) };
    windows.push(hwnd);
    BOOL(1)
}

pub fn running_apps() -> Vec<AppIdentity> {
    let mut windows: Vec<HWND> = Vec::new();
    // SAFETY: the callback only writes into `windows`, which outlives the call.
    let _ = unsafe {
        EnumWindows(
            Some(collect),
            LPARAM(&mut windows as *mut Vec<HWND> as isize),
        )
    };
    windows
        .into_iter()
        .filter(|&hwnd| is_app_window(hwnd) && !window_title(hwnd).trim().is_empty())
        .filter_map(identify)
        .collect()
}
