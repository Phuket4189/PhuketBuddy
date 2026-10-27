use serde::{Deserialize, Serialize};
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};
#[cfg(windows)]
use windows_sys::Win32::Foundation::POINT;
#[cfg(windows)]
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn right(&self) -> i32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }
}

/// 取指定屏幕坐标所在显示器的工作区（已排除任务栏），单位：物理像素。
#[cfg(windows)]
pub fn work_area_at(x: i32, y: i32) -> Rect {
    unsafe {
        let pt = POINT { x, y };
        let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(hmon, &mut info) != 0 {
            let r = info.rcWork;
            Rect { x: r.left, y: r.top, w: r.right - r.left, h: r.bottom - r.top }
        } else {
            Rect { x: 0, y: 0, w: 1920, h: 1040 }
        }
    }
}

#[cfg(windows)]
pub fn left_button_down() -> bool {
    unsafe { (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0 }
}

/// 前台窗口信息（pid + 标题），用于判断焦点去了哪里。
#[cfg(windows)]
pub fn foreground_info() -> String {
    #[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return "none".into();
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let mut buf = [0u16; 128];
        let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        let title = String::from_utf16_lossy(&buf[..len.max(0) as usize]);
        format!("pid={pid} ours={} title={title}", pid == std::process::id())
    }
}

#[cfg(windows)]
pub fn foreground_is_ours() -> bool {
    #[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return false;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        pid == std::process::id()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SavedPos {
    pub x: i32,
    pub y: i32,
}

fn pos_file() -> std::path::PathBuf {
    crate::secure::data_dir().join("position.json")
}

pub fn save_pos(x: i32, y: i32) {
    if let Ok(b) = serde_json::to_vec(&SavedPos { x, y }) {
        let _ = std::fs::write(pos_file(), b);
    }
}

pub fn load_pos() -> Option<SavedPos> {
    let b = std::fs::read(pos_file()).ok()?;
    serde_json::from_slice(&b).ok()
}

/// 气泡默认位置：主屏右侧、偏上。
pub fn default_pos(size: PhysicalSize<u32>) -> PhysicalPosition<i32> {
    let area = work_area_at(0, 0);
    let x = area.right() - size.width as i32 - 24;
    let y = area.y + (area.h as f64 * 0.32) as i32;
    PhysicalPosition::new(x, y)
}

/// 松手后吸附：距工作区边缘小于阈值就贴边，保留 6px 间隙。
pub fn snap(win: &WebviewWindow) -> bool {
    let (Ok(pos), Ok(size), Ok(scale)) =
        (win.outer_position(), win.outer_size(), win.scale_factor())
    else {
        return false;
    };
    let area = work_area_for(win);

    let threshold = (26.0 * scale) as i32;
    let gap = (6.0 * scale) as i32;

    let mut x = pos.x;
    let mut y = pos.y;
    let mut snapped = false;

    let d_left = pos.x - area.x;
    let d_right = area.right() - (pos.x + size.width as i32);
    if d_left <= threshold && d_left <= d_right {
        x = area.x + gap;
        snapped = true;
    } else if d_right <= threshold {
        x = area.right() - size.width as i32 - gap;
        snapped = true;
    }

    let d_top = pos.y - area.y;
    let d_bottom = area.bottom() - (pos.y + size.height as i32);
    if d_top <= threshold && d_top <= d_bottom {
        y = area.y + gap;
        snapped = true;
    } else if d_bottom <= threshold {
        y = area.bottom() - size.height as i32 - gap;
        snapped = true;
    }

    x = x.clamp(area.x + gap, (area.right() - size.width as i32 - gap).max(area.x + gap));
    y = y.clamp(area.y + gap, (area.bottom() - size.height as i32 - gap).max(area.y + gap));

    if x != pos.x || y != pos.y {
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
    save_pos(x, y);
    snapped
}

/// 按气泡位置把面板贴在其左侧或右侧，并保证不越出工作区。
pub fn place_panel(panel: &WebviewWindow, bubble: &WebviewWindow) {
    let (Ok(bpos), Ok(bsize), Ok(psize)) =
        (bubble.outer_position(), bubble.outer_size(), panel.outer_size())
    else {
        return;
    };
    let scale = panel.scale_factor().unwrap_or(1.0);
    let gap = (12.0 * scale) as i32;
    let margin = (8.0 * scale) as i32;

    let area = work_area_for(bubble);
    // macOS window positions are physical pixels at each window's own scale.
    // Convert to the panel's scale before placing across mixed-DPI screens.
    #[cfg(target_os = "macos")]
    let (bpos, bsize, area) = {
        let ratio = scale / bubble.scale_factor().unwrap_or(scale);
        let px = |v: i32| (v as f64 * ratio).round() as i32;
        (PhysicalPosition::new(px(bpos.x), px(bpos.y)),
         PhysicalSize::new((bsize.width as f64 * ratio).round() as u32, (bsize.height as f64 * ratio).round() as u32),
         Rect { x: px(area.x), y: px(area.y), w: px(area.w), h: px(area.h) })
    };

    let right_space = area.right() - (bpos.x + bsize.width as i32);
    let left_space = bpos.x - area.x;

    let mut x = if right_space >= psize.width as i32 + gap || left_space < psize.width as i32 + gap {
        bpos.x + bsize.width as i32 + gap
    } else {
        bpos.x - psize.width as i32 - gap
    };
    let mut y = bpos.y;

    if x + psize.width as i32 > area.right() - margin {
        x = bpos.x - psize.width as i32 - gap;
    }
    if x < area.x + margin {
        x = area.x + margin;
    }
    if y + psize.height as i32 > area.bottom() - margin {
        y = area.bottom() - psize.height as i32 - margin;
    }
    if y < area.y + margin {
        y = area.y + margin;
    }

    let _ = panel.set_position(PhysicalPosition::new(x, y));
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn pb_window_work_area(window: *mut std::ffi::c_void, out: *mut i32);
    fn pb_work_area(x: i32, y: i32, out: *mut i32);
    fn pb_left_button_down() -> bool;
    fn pb_is_active() -> bool;
}
#[cfg(target_os = "macos")]
pub fn work_area_at(x: i32, y: i32) -> Rect {
    let mut r = [0; 4];
    unsafe { pb_work_area(x, y, r.as_mut_ptr()); }
    Rect { x: r[0], y: r[1], w: r[2], h: r[3] }
}
#[cfg(target_os = "macos")]
pub fn left_button_down() -> bool { unsafe { pb_left_button_down() } }
#[cfg(target_os = "macos")]
pub fn foreground_is_ours() -> bool { unsafe { pb_is_active() } }
#[cfg(target_os = "macos")]
pub fn foreground_info() -> String { format!("ours={}", foreground_is_ours()) }

fn work_area_for(window: &WebviewWindow) -> Rect {
    #[cfg(target_os = "macos")]
    if let Ok(native) = window.ns_window() {
        let mut r = [0; 4];
        unsafe { pb_window_work_area(native, r.as_mut_ptr()); }
        return Rect { x: r[0], y: r[1], w: r[2], h: r[3] };
    }
    let pos = window.outer_position().unwrap_or_default();
    let size = window.outer_size().unwrap_or(PhysicalSize::new(72, 72));
    work_area_at(pos.x + size.width as i32 / 2, pos.y + size.height as i32 / 2)
}
