//! Win32 implementation: a layered, click-through-free popup window kept
//! just left of the taskbar's notification area, redrawn on demand and every
//! second (to follow tray width, theme, DPI and full-screen apps).

use std::cell::RefCell;
use std::sync::{Mutex, OnceLock};
use std::thread;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject, DrawTextW, GdiFlush,
    GetDC, GetMonitorInfoW, GetTextExtentPoint32W, GetTextFaceW, MonitorFromWindow, ReleaseDC,
    SelectObject, SetBkMode, SetTextColor, AC_SRC_ALPHA, AC_SRC_OVER, ANTIALIASED_QUALITY,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    DIB_RGB_COLORS, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FW_NORMAL, FW_SEMIBOLD, HDC,
    HFONT, HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST, OUT_DEFAULT_PRECIS, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_WIN95_CLASSES, INITCOMMONCONTROLSEX, TOOLTIPS_CLASSW, TTF_SUBCLASS,
    TTM_ADDTOOLW, TTM_DELTOOLW, TTM_SETMAXTIPWIDTH, TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW,
};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FindWindowExW, FindWindowW,
    GetClassNameW, GetForegroundWindow, GetMessageW, GetWindowRect, LoadCursorW, PostMessageW,
    PostQuitMessage, RegisterClassExW, SendMessageW, SetTimer, SetWindowPos, ShowWindow,
    TranslateMessage, UpdateLayeredWindow, HWND_TOPMOST, IDC_HAND, MA_NOACTIVATE, MSG,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SW_HIDE, SW_SHOWNOACTIVATE, ULW_ALPHA, WINDOW_STYLE,
    WM_APP, WM_CLOSE, WM_DESTROY, WM_LBUTTONUP, WM_MOUSEACTIVATE, WM_TIMER, WNDCLASSEXW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::{colorize, Item};

const WM_REDRAW: u32 = WM_APP + 1;
const TIMER_ID: usize = 1;
const TIMER_MS: u32 = 1000;

static ITEMS: Mutex<Vec<Item>> = Mutex::new(Vec::new());
/// Readout window handle (as an integer, so it can be shared between threads).
static WINDOW: Mutex<Option<isize>> = Mutex::new(None);
static ON_CLICK: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

thread_local! {
    /// Tooltip control of the readout window (window thread only).
    static TOOLTIP: RefCell<Option<HWND>> = const { RefCell::new(None) };
    /// Tool areas registered with the tooltip, so they are only replaced when
    /// they change (replacing them hides a tooltip being shown).
    static TOOLS: RefCell<Vec<(RECT, String)>> = const { RefCell::new(Vec::new()) };
}

pub fn set_on_click(callback: Box<dyn Fn() + Send + Sync>) {
    let _ = ON_CLICK.set(callback);
}

pub fn set_items(items: Vec<Item>) {
    let empty = items.is_empty();
    if let Ok(mut current) = ITEMS.lock() {
        *current = items;
    }
    let Ok(mut window) = WINDOW.lock() else {
        return;
    };
    match *window {
        Some(hwnd) => {
            // SAFETY: posting to our own window; a stale handle just fails.
            let _ = unsafe { PostMessageW(Some(HWND(hwnd as _)), WM_REDRAW, WPARAM(0), LPARAM(0)) };
        }
        None if !empty => {
            let (tx, rx) = std::sync::mpsc::channel();
            let spawned = thread::Builder::new()
                .name("taskbar-readout".into())
                .spawn(move || run(tx));
            if spawned.is_ok() {
                *window = rx.recv().ok().flatten();
            }
        }
        None => {}
    }
}

/// Window thread: creates the window and pumps its messages.
fn run(ready: std::sync::mpsc::Sender<Option<isize>>) {
    // SAFETY: standard Win32 window creation and message loop on this thread.
    unsafe {
        let Ok(module) = GetModuleHandleW(PCWSTR::null()) else {
            let _ = ready.send(None);
            return;
        };
        let class = w!("PeriphOSTaskbarReadout");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window_proc),
            hInstance: module.into(),
            hCursor: LoadCursorW(None, IDC_HAND).unwrap_or_default(),
            lpszClassName: class,
            ..Default::default()
        };
        RegisterClassExW(&wc);
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
            class,
            w!("PeriphOS"),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            Some(module.into()),
            None,
        );
        let Ok(hwnd) = hwnd else {
            let _ = ready.send(None);
            return;
        };
        let _ = ready.send(Some(hwnd.0 as isize));
        create_tooltip(hwnd, module.into());
        SetTimer(Some(hwnd), TIMER_ID, TIMER_MS, None);
        redraw(hwnd);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    if let Ok(mut window) = WINDOW.lock() {
        *window = None;
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TIMER | WM_REDRAW => {
            redraw(hwnd);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            if let Some(callback) = ON_CLICK.get() {
                callback();
            }
            LRESULT(0)
        }
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_CLOSE => {
            // SAFETY: destroying our own window from its thread.
            let _ = unsafe { DestroyWindow(hwnd) };
            LRESULT(0)
        }
        WM_DESTROY => {
            // SAFETY: ends this thread's message loop.
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        // SAFETY: default handling for everything else.
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Creates the tooltip that names the device under the pointer.
fn create_tooltip(owner: HWND, module: windows::Win32::Foundation::HINSTANCE) {
    // SAFETY: standard common-controls tooltip creation on the window thread.
    unsafe {
        let _ = InitCommonControlsEx(&INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_WIN95_CLASSES,
        });
        let Ok(tip) = CreateWindowExW(
            WS_EX_TOPMOST,
            TOOLTIPS_CLASSW,
            PCWSTR::null(),
            WS_POPUP | WINDOW_STYLE(TTS_ALWAYSTIP | TTS_NOPREFIX),
            0,
            0,
            0,
            0,
            Some(owner),
            None,
            Some(module),
            None,
        ) else {
            return;
        };
        SendMessageW(tip, TTM_SETMAXTIPWIDTH, None, Some(LPARAM(400)));
        TOOLTIP.with(|t| *t.borrow_mut() = Some(tip));
    }
}

/// `TTTOOLINFOW` without its last field, a size every comctl32 version accepts.
const TOOL_INFO_SIZE: u32 =
    (std::mem::size_of::<TTTOOLINFOW>() - std::mem::size_of::<*mut core::ffi::c_void>()) as u32;

/// Registers one tooltip area per item (client coordinates of `owner`).
fn set_tools(owner: HWND, tools: Vec<(RECT, String)>) {
    let Some(tip) = TOOLTIP.with(|t| *t.borrow()) else {
        return;
    };
    let unchanged = TOOLS.with(|current| {
        let current = current.borrow();
        current.len() == tools.len()
            && current.iter().zip(&tools).all(|((a, s), (b, t))| {
                s == t && (a.left, a.top, a.right, a.bottom) == (b.left, b.top, b.right, b.bottom)
            })
    });
    if unchanged {
        return;
    }
    let previous = TOOLS.with(|t| t.borrow().len());
    // SAFETY: messages to our own tooltip; the text buffers outlive each call
    // (the control copies them).
    unsafe {
        for id in 0..previous {
            let info = TTTOOLINFOW {
                cbSize: TOOL_INFO_SIZE,
                hwnd: owner,
                uId: id,
                ..Default::default()
            };
            SendMessageW(
                tip,
                TTM_DELTOOLW,
                None,
                Some(LPARAM(&info as *const _ as isize)),
            );
        }
        for (id, (rect, text)) in tools.iter().enumerate() {
            let mut text: Vec<u16> = text.encode_utf16().chain([0]).collect();
            let info = TTTOOLINFOW {
                cbSize: TOOL_INFO_SIZE,
                uFlags: TTF_SUBCLASS,
                hwnd: owner,
                uId: id,
                rect: *rect,
                lpszText: windows::core::PWSTR(text.as_mut_ptr()),
                ..Default::default()
            };
            SendMessageW(
                tip,
                TTM_ADDTOOLW,
                None,
                Some(LPARAM(&info as *const _ as isize)),
            );
        }
    }
    TOOLS.with(|t| *t.borrow_mut() = tools);
}

fn hide(hwnd: HWND) {
    // SAFETY: plain call on our own window.
    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
}

fn window_rect(hwnd: HWND) -> Option<RECT> {
    let mut rect = RECT::default();
    // SAFETY: `rect` outlives the call.
    unsafe { GetWindowRect(hwnd, &mut rect) }
        .ok()
        .map(|()| rect)
}

fn class_name(hwnd: HWND) -> String {
    let mut buffer = [0u16; 64];
    // SAFETY: the slice is valid during the call.
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

/// True while a full-screen app (typically a game) covers the taskbar's monitor.
fn fullscreen_app_in_front(taskbar: HWND) -> bool {
    // SAFETY: read-only queries on window handles.
    unsafe {
        let front = GetForegroundWindow();
        if front.is_invalid() || front == taskbar {
            return false;
        }
        if matches!(
            class_name(front).as_str(),
            "Progman" | "WorkerW" | "Shell_TrayWnd"
        ) {
            return false; // the desktop also covers the screen
        }
        let Some(rect) = window_rect(front) else {
            return false;
        };
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(
            MonitorFromWindow(front, MONITOR_DEFAULTTONEAREST),
            &mut info,
        )
        .as_bool()
        {
            return false;
        }
        let m = info.rcMonitor;
        rect.left <= m.left && rect.top <= m.top && rect.right >= m.right && rect.bottom >= m.bottom
    }
}

fn light_taskbar() -> bool {
    let mut value: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: `value` and `size` outlive the call and match RRF_RT_REG_DWORD.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("SystemUsesLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut core::ffi::c_void),
            Some(&mut size),
        )
    };
    status.is_ok() && value == 1
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// GDI resources for one redraw, released on drop.
struct Canvas {
    screen: HDC,
    dc: HDC,
    fonts: Vec<HFONT>,
}

impl Drop for Canvas {
    fn drop(&mut self) {
        // SAFETY: every handle was created by this canvas and is released once.
        unsafe {
            for font in &self.fonts {
                let _ = DeleteObject(HGDIOBJ(font.0));
            }
            let _ = DeleteDC(self.dc);
            ReleaseDC(None, self.screen);
        }
    }
}

impl Canvas {
    fn font(&mut self, face: PCWSTR, pixels: i32, semibold: bool) -> HFONT {
        // SAFETY: plain GDI object creation, released in Drop.
        let font = unsafe {
            CreateFontW(
                -pixels,
                0,
                0,
                0,
                if semibold {
                    FW_SEMIBOLD.0 as i32
                } else {
                    FW_NORMAL.0 as i32
                },
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                ANTIALIASED_QUALITY,
                0,
                face,
            )
        };
        self.fonts.push(font);
        font
    }

    /// Segoe Fluent Icons (Windows 11), else Segoe MDL2 Assets (Windows 10).
    fn icon_font(&mut self, pixels: i32) -> HFONT {
        let fluent = self.font(w!("Segoe Fluent Icons"), pixels, false);
        let mut face = [0u16; 64];
        // SAFETY: selecting our own font into our own DC to read its face name.
        let len = unsafe {
            SelectObject(self.dc, HGDIOBJ(fluent.0));
            GetTextFaceW(self.dc, Some(&mut face))
        };
        let name = String::from_utf16_lossy(&face[..len.max(1) as usize - 1]);
        if name.eq_ignore_ascii_case("Segoe Fluent Icons") {
            fluent
        } else {
            self.font(w!("Segoe MDL2 Assets"), pixels, false)
        }
    }

    fn measure(&self, font: HFONT, text: &[u16]) -> i32 {
        let mut size = SIZE::default();
        // SAFETY: our font and DC; `size` outlives the call.
        unsafe {
            SelectObject(self.dc, HGDIOBJ(font.0));
            let _ = GetTextExtentPoint32W(self.dc, text, &mut size);
        }
        size.cx
    }
}

fn redraw(hwnd: HWND) {
    let items = ITEMS.lock().map(|i| i.clone()).unwrap_or_default();
    // SAFETY: FindWindowW only reads the window list.
    let Ok(taskbar) = (unsafe { FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) }) else {
        return hide(hwnd);
    };
    let Some(bar) = window_rect(taskbar) else {
        return hide(hwnd);
    };
    let (bar_width, bar_height) = (bar.right - bar.left, bar.bottom - bar.top);
    if items.is_empty()
        || bar_height <= 0
        || bar_height > bar_width
        || fullscreen_app_in_front(taskbar)
    {
        return hide(hwnd);
    }
    // Auto-hidden taskbar: mostly off screen.
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` outlives the call.
    if unsafe {
        GetMonitorInfoW(
            MonitorFromWindow(taskbar, MONITOR_DEFAULTTONEAREST),
            &mut info,
        )
    }
    .as_bool()
        && bar.top >= info.rcMonitor.bottom - 4
    {
        return hide(hwnd);
    }

    // SAFETY: reads the DPI of an existing window.
    let dpi = match unsafe { GetDpiForWindow(taskbar) } {
        0 => 96,
        d => d as i32,
    };
    let px = |v: i32| (v * dpi + 48) / 96;
    // Right edge: the notification area, or an estimate if it cannot be found.
    // SAFETY: FindWindowExW only reads the window tree.
    let tray = unsafe { FindWindowExW(Some(taskbar), None, w!("TrayNotifyWnd"), PCWSTR::null()) };
    let right = tray
        .ok()
        .and_then(window_rect)
        .map_or(bar.right - px(320), |r| r.left)
        - px(6);

    // SAFETY: plain GDI DC creation, released by Canvas.
    let (screen, dc) = unsafe {
        let screen = GetDC(None);
        (screen, CreateCompatibleDC(Some(screen)))
    };
    let mut canvas = Canvas {
        screen,
        dc,
        fonts: Vec::new(),
    };
    let text_font = canvas.font(w!("Segoe UI"), px(12), false);
    let icon_font = canvas.icon_font(px(15));

    let (pad, gap, spacing) = (px(6), px(5), px(14));
    let parts: Vec<(Vec<u16>, i32, Vec<u16>, i32)> = items
        .iter()
        .map(|item| {
            let glyph = wide(&item.glyph.code_point().to_string());
            let text = wide(&item.text);
            let gw = canvas.measure(icon_font, &glyph);
            let tw = canvas.measure(text_font, &text);
            (glyph, gw, text, tw)
        })
        .collect();
    let width = pad * 2
        + parts
            .iter()
            .map(|(_, gw, _, tw)| gw + gap + tw)
            .sum::<i32>()
        + spacing * (parts.len() as i32 - 1);
    let height = bar_height;
    let (x, y) = (right - width, bar.top);

    let header = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height, // top-down
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };
    let bmi = BITMAPINFO {
        bmiHeader: header,
        ..Default::default()
    };
    let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
    // SAFETY: `bmi` describes a 32-bit top-down DIB; `bits` receives its memory.
    let Ok(dib) = (unsafe { CreateDIBSection(Some(dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) })
    else {
        return hide(hwnd);
    };
    let light = light_taskbar();
    let mut ranges = Vec::new();
    let mut tools = Vec::new();
    // SAFETY: drawing into our DIB through our DC; all objects are ours.
    unsafe {
        let previous = SelectObject(dc, HGDIOBJ(dib.0));
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, COLORREF(0x00FF_FFFF));
        let mut cursor = pad;
        for (item, (mut glyph, gw, mut text, tw)) in items.iter().zip(parts) {
            let start = cursor;
            let format = DT_SINGLELINE | DT_VCENTER | DT_LEFT | DT_NOPREFIX;
            SelectObject(dc, HGDIOBJ(icon_font.0));
            let mut rect = RECT {
                left: cursor,
                top: 0,
                right: cursor + gw,
                bottom: height,
            };
            DrawTextW(dc, &mut glyph, &mut rect, format);
            cursor += gw + gap;
            SelectObject(dc, HGDIOBJ(text_font.0));
            let mut rect = RECT {
                left: cursor,
                top: 0,
                right: cursor + tw,
                bottom: height,
            };
            DrawTextW(dc, &mut text, &mut rect, format);
            cursor += tw;
            ranges.push((start as usize, cursor as usize, item.tone.rgb(light)));
            tools.push((
                RECT {
                    left: start - spacing / 2,
                    top: 0,
                    right: cursor + spacing / 2,
                    bottom: height,
                },
                item.tooltip.clone(),
            ));
            cursor += spacing;
        }
        let _ = GdiFlush();
        let pixels = std::slice::from_raw_parts_mut(bits as *mut u8, (width * height * 4) as usize);
        colorize(pixels, width as usize, &ranges);

        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let _ = UpdateLayeredWindow(
            hwnd,
            Some(screen),
            Some(&POINT { x, y }),
            Some(&SIZE {
                cx: width,
                cy: height,
            }),
            Some(dc),
            Some(&POINT { x: 0, y: 0 }),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(dib.0));
        // Stay above the taskbar, which is also top-most.
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
    set_tools(hwnd, tools);
}
