//! Address parsing, port test, Windows Credential Manager and mstsc.exe sessions.

use crate::i18n::t;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::time::Duration;

use crate::profiles::DEFAULT_PORT;

pub const DISPLAY_MODES: [&str; 4] = ["windowed", "fullscreen", "1080", "768"];

#[derive(Clone, Debug)]
pub struct Options {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub display_mode: String,
    pub sound: bool,
    pub clipboard: bool,
    pub multimon: bool,
}

pub fn join_host_port(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

/// Splits "host", "host:port" or "[ipv6]:port".
pub fn parse_host_port(raw: &str) -> Result<(String, u16), String> {
    let text = raw.trim();
    if text.is_empty() {
        return Err(t("err_host_empty"));
    }
    if let Some(rest) = text.strip_prefix('[') {
        if let Some((host, port)) = rest.split_once("]:") {
            return Ok((host.trim().to_string(), parse_port(port)?));
        }
        if let Some(host) = rest.strip_suffix(']') {
            return Ok((host.trim().to_string(), DEFAULT_PORT));
        }
        return Err(t("err_ipv6_unclosed"));
    }
    match text.matches(':').count() {
        0 => Ok((text.trim_end_matches('.').to_string(), DEFAULT_PORT)),
        1 => {
            let (host, port) = text.split_once(':').unwrap_or((text, ""));
            let host = host.trim().trim_end_matches('.');
            if host.is_empty() {
                return Err(t("err_host_before_port"));
            }
            Ok((host.to_string(), parse_port(port)?))
        }
        _ => Ok((text.to_string(), DEFAULT_PORT)), // bare IPv6
    }
}

fn parse_port(s: &str) -> Result<u16, String> {
    match s.trim().parse::<u16>() {
        Ok(p) if p >= 1 => Ok(p),
        _ => Err(t("err_port_invalid").replace("{port}", s)),
    }
}

pub fn check_port(host: &str, port: u16) -> Result<(), String> {
    let addrs = (host, port).to_socket_addrs().map_err(|e| e.to_string())?;
    let mut last = String::from("no address");
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
            Ok(_) => return Ok(()),
            Err(e) => last = e.to_string(),
        }
    }
    Err(last)
}

// ---- Windows Credential Manager: mstsc.exe reads TERMSRV/<host> ----

fn cred_target(host: &str) -> String {
    format!("TERMSRV/{host}")
}

#[cfg(windows)]
mod cred {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn store(target: &str, user: &str, password: &str) -> Result<(), String> {
        let mut target_w = wide(target);
        let mut user_w = wide(user);
        let mut comment_w = wide("DesktopRDP");
        let mut blob: Vec<u8> = password.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
        let cred = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target_w.as_mut_ptr(),
            Comment: comment_w.as_mut_ptr(),
            LastWritten: FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 },
            CredentialBlobSize: blob.len() as u32,
            CredentialBlob: if blob.is_empty() { std::ptr::null_mut() } else { blob.as_mut_ptr() },
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: std::ptr::null_mut(),
            TargetAlias: std::ptr::null_mut(),
            UserName: user_w.as_mut_ptr(),
        };
        let ok = unsafe { CredWriteW(&cred, 0) };
        blob.iter_mut().for_each(|b| *b = 0);
        if ok == 0 {
            Err(std::io::Error::last_os_error().to_string())
        } else {
            Ok(())
        }
    }

    pub fn exists(target: &str) -> bool {
        let target_w = wide(target);
        let mut pc: *mut CREDENTIALW = std::ptr::null_mut();
        let ok = unsafe { CredReadW(target_w.as_ptr(), CRED_TYPE_GENERIC, 0, &mut pc) };
        if ok == 0 {
            return false;
        }
        unsafe { CredFree(pc as *const _) };
        true
    }

    pub fn delete(target: &str) {
        let target_w = wide(target);
        unsafe { CredDeleteW(target_w.as_ptr(), CRED_TYPE_GENERIC, 0) };
    }
}

#[cfg(not(windows))]
mod cred {
    // Development builds on Linux: no credential store, nothing is saved.
    pub fn store(_: &str, _: &str, _: &str) -> Result<(), String> {
        Err("Credential Manager is only available on Windows".into())
    }
    pub fn exists(_: &str) -> bool {
        false
    }
    pub fn delete(_: &str) {}
}

pub fn store_credential(host: &str, user: &str, password: &str) -> Result<(), String> {
    cred::store(&cred_target(host), user, password)
}
pub fn has_credential(host: &str) -> bool {
    cred::exists(&cred_target(host))
}
pub fn delete_credential(host: &str) {
    cred::delete(&cred_target(host))
}

// ---- .rdp file and mstsc.exe ----

fn sessions_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("DesktopRDP")
        .join("sessions")
}

pub fn rdp_file_content(o: &Options) -> String {
    let mut lines: Vec<String> = vec![
        format!("full address:s:{}", join_host_port(&o.host, o.port)),
        "prompt for credentials:i:0".into(),
        "enablecredsspsupport:i:1".into(),
        "negotiate security layer:i:1".into(),
        "authentication level:i:2".into(),
        "autoreconnection enabled:i:1".into(),
        "session bpp:i:32".into(),
        "connection type:i:7".into(),
        "networkautodetect:i:1".into(),
        "bandwidthautodetect:i:1".into(),
        "compression:i:1".into(),
        "redirectprinters:i:0".into(),
        "redirectsmartcards:i:0".into(),
        "redirectcomports:i:0".into(),
    ];
    if !o.username.is_empty() {
        lines.push(format!("username:s:{}", o.username));
    }
    let mode: &[&str] = match o.display_mode.as_str() {
        // Using every monitor only works as a full-screen session.
        _ if o.multimon => &["screen mode id:i:2", "use multimon:i:1", "dynamic resolution:i:1"],
        "fullscreen" => &["screen mode id:i:2", "dynamic resolution:i:1"],
        "1080" => &["screen mode id:i:1", "desktopwidth:i:1920", "desktopheight:i:1080", "smart sizing:i:1"],
        "768" => &["screen mode id:i:1", "desktopwidth:i:1366", "desktopheight:i:768", "smart sizing:i:1"],
        _ => &["screen mode id:i:1", "dynamic resolution:i:1", "smart sizing:i:0"],
    };
    lines.extend(mode.iter().map(|s| s.to_string()));
    if !o.multimon {
        lines.push("use multimon:i:0".into());
    }
    lines.push(format!("audiomode:i:{}", if o.sound { 0 } else { 2 }));
    lines.push(format!("redirectclipboard:i:{}", u8::from(o.clipboard)));
    lines.join("\r\n") + "\r\n"
}

/// Writes a .rdp file and runs mstsc.exe on it; `done` runs on a background thread when it exits.
pub fn start_session(o: &Options, done: impl FnOnce() + Send + 'static) -> Result<(), String> {
    let dir = sessions_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = dir.join(format!("session-{}.rdp", crate::profiles::new_id()));
    std::fs::write(&file, rdp_file_content(o)).map_err(|e| e.to_string())?;

    let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    let mstsc = PathBuf::from(system_root).join("System32").join("mstsc.exe");
    let mut child = match std::process::Command::new(&mstsc).arg(&file).spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = std::fs::remove_file(&file);
            return Err(format!("{}: {e}", mstsc.display()));
        }
    };
    std::thread::spawn(move || {
        let _ = child.wait();
        // mstsc reads the file at start-up; keep it briefly in case it re-reads on reconnect.
        std::thread::sleep(Duration::from_secs(2));
        let _ = std::fs::remove_file(&file);
        done();
    });
    Ok(())
}
