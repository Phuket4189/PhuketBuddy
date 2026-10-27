use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::OnceLock;

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

fn try_dir(p: &PathBuf) -> bool {
    std::fs::create_dir_all(p).is_ok() && std::fs::write(p.join(".w"), b"1").is_ok()
}

/// 应用数据目录：优先 %APPDATA%，不可写时回退到 exe 同级 data\ 或 %TEMP%。
pub fn data_dir() -> PathBuf {
    DATA_DIR
        .get_or_init(|| {
            let mut candidates: Vec<PathBuf> = Vec::new();
            #[cfg(target_os = "macos")]
            if let Some(home) = std::env::var_os("HOME") {
                candidates.push(PathBuf::from(home).join("Library/Application Support/PhuketBuddy"));
            }
            if let Ok(a) = std::env::var("APPDATA") {
                candidates.push(PathBuf::from(a).join("PhuketBuddy"));
            }
            if let Some(d) = exe_dir() {
                candidates.push(d.join("data"));
            }
            if let Ok(t) = std::env::var("TEMP") {
                candidates.push(PathBuf::from(t).join("PhuketBuddy"));
            }
            for c in candidates {
                if try_dir(&c) {
                    let _ = std::fs::remove_file(c.join(".w"));
                    return c;
                }
            }
            PathBuf::from(".")
        })
        .clone()
}

/// WebView2 用户数据目录：同样带回退，避免受限环境下窗口创建失败。
pub fn webview_data_dir() -> PathBuf {
    let mut candidates: Vec<PathBuf> = Vec::new();
    #[cfg(target_os = "macos")]
    candidates.push(data_dir().join("webview"));
    if let Ok(l) = std::env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(l).join("PhuketBuddy").join("webview"));
    }
    if let Some(d) = exe_dir() {
        candidates.push(d.join("data").join("webview"));
    }
    if let Ok(t) = std::env::var("TEMP") {
        candidates.push(PathBuf::from(t).join("PhuketBuddy").join("webview"));
    }
    for c in candidates {
        if try_dir(&c) {
            let _ = std::fs::remove_file(c.join(".w"));
            return c;
        }
    }
    std::env::temp_dir().join("PhuketBuddy").join("webview")
}

/* ---------- DPAPI 加密 ---------- */
#[cfg(windows)]
mod dpapi {
    use windows_sys::Win32::Foundation::{LocalFree, HLOCAL};
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        }
    }

    pub fn protect(data: &[u8]) -> Option<Vec<u8>> {
        unsafe {
            let input = blob(data);
            let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
            let ok = CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            );
            if ok == 0 || out.pbData.is_null() {
                return None;
            }
            let v = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
            LocalFree(out.pbData as HLOCAL);
            Some(v)
        }
    }

    pub fn unprotect(data: &[u8]) -> Option<Vec<u8>> {
        unsafe {
            let input = blob(data);
            let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
            let ok = CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            );
            if ok == 0 || out.pbData.is_null() {
                return None;
            }
            let v = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
            LocalFree(out.pbData as HLOCAL);
            Some(v)
        }
    }
}

#[cfg(windows)]
pub fn save_session(session: &crate::api::Session) {
    let Ok(raw) = serde_json::to_vec(session) else { return };
    let Some(enc) = dpapi::protect(&raw) else { return };
    let _ = std::fs::write(data_dir().join("session.bin"), enc);
}

#[cfg(windows)]
pub fn load_session() -> Option<crate::api::Session> {
    let enc = std::fs::read(data_dir().join("session.bin")).ok()?;
    let raw = dpapi::unprotect(&enc)?;
    serde_json::from_slice(&raw).ok()
}

#[cfg(windows)]
pub fn clear_session() {
    let _ = std::fs::remove_file(data_dir().join("session.bin"));
}

/* ---------- 设置 ---------- */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub refresh_minutes: u32,
    pub autostart: bool,
    pub alert_threshold: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { refresh_minutes: 5, autostart: false, alert_threshold: 1.0 }
    }
}

const SETTINGS_FILE: &str = "settings.json";

pub fn load_settings() -> Settings {
    std::fs::read(data_dir().join(SETTINGS_FILE))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub fn save_settings(s: &Settings) {
    if let Ok(b) = serde_json::to_vec_pretty(s) {
        let _ = std::fs::write(data_dir().join(SETTINGS_FILE), b);
    }
}

/* ---------- 缓存 ---------- */
pub fn save_cache(v: &Value) {
    let mut safe = v.clone();
    if let Some(keys) = safe.get_mut("keys").and_then(Value::as_array_mut) {
        for key in keys { if let Some(obj) = key.as_object_mut() { obj.remove("key"); } }
    }
    if let Ok(b) = serde_json::to_vec(&safe) {
        let _ = std::fs::write(data_dir().join("cache.json"), b);
    }
}

pub fn load_cache() -> Option<Value> {
    let b = std::fs::read(data_dir().join("cache.json")).ok()?;
    serde_json::from_slice(&b).ok()
}

/* ---------- 诊断日志 ---------- */
pub fn log_line(msg: &str) {
    use std::io::Write;
    let path = data_dir().join("error.log");
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > 256 * 1024 {
            let _ = std::fs::remove_file(&path);
        }
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[{ts}] {msg}");
    }
}

/* ---------- 诊断：只落结构，不落数值 ---------- */
pub fn save_raw(v: &Value) {
    if let Ok(b) = serde_json::to_vec_pretty(v) {
        let _ = std::fs::write(data_dir().join("raw-shape.json"), b);
    }
}

/* ---------- 开机自启 ---------- */
#[cfg(windows)]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    use std::process::Command;
    let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let status = Command::new("reg")
            .args([
                "add",
                key,
                "/v",
                "PhuketBuddy",
                "/t",
                "REG_SZ",
                "/d",
                &format!("\"{}\"", exe.display()),
                "/f",
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("写入注册表失败".into())
        }
    } else {
        let _ = Command::new("reg")
            .args(["delete", key, "/v", "PhuketBuddy", "/f"])
            .status();
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub fn save_session(session: &crate::api::Session) {
    if let Ok(raw) = serde_json::to_vec(session) {
        if let Err(e) = security_framework::passwords::set_generic_password("com.phuketbuddy.desktop", "session", &raw) {
            log_line(&format!("Keychain save failed: {e}"));
        }
    }
}
#[cfg(target_os = "macos")]
pub fn load_session() -> Option<crate::api::Session> {
    let raw = security_framework::passwords::get_generic_password("com.phuketbuddy.desktop", "session").ok()?;
    serde_json::from_slice(&raw).ok()
}
#[cfg(target_os = "macos")]
pub fn clear_session() {
    let _ = security_framework::passwords::delete_generic_password("com.phuketbuddy.desktop", "session");
}
#[cfg(target_os = "macos")]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let home = std::env::var_os("HOME").ok_or("无法确定用户目录")?;
    let dir = PathBuf::from(home).join("Library/LaunchAgents");
    let path = dir.join("com.phuketbuddy.desktop.plist");
    if !enabled {
        return match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        };
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let escaped = exe.to_string_lossy().replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
    let plist = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>Label</key><string>com.phuketbuddy.desktop</string>
<key>ProgramArguments</key><array><string>{escaped}</string></array>
<key>RunAtLoad</key><true/></dict></plist>"#);
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    std::fs::write(path, plist).map_err(|e| e.to_string())
}
