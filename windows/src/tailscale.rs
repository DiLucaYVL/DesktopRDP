//! Tailscale peer discovery through `tailscale status --json`.

use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct Peer {
    pub hostname: String,
    pub ip: String,
    pub os: String,
    pub online: bool,
}

pub enum Error {
    NotInstalled,
    Unavailable,
    Invalid(String),
}

#[derive(Deserialize)]
struct Status {
    #[serde(rename = "BackendState", default)]
    backend_state: String,
    #[serde(rename = "Peer", default)]
    peer: Option<HashMap<String, PeerEntry>>,
}

#[derive(Deserialize)]
struct PeerEntry {
    #[serde(rename = "OS", default)]
    os: String,
    #[serde(rename = "HostName", default)]
    host_name: String,
    #[serde(rename = "DNSName", default)]
    dns_name: String,
    #[serde(rename = "TailscaleIPs", default)]
    ips: Vec<String>,
    #[serde(rename = "Online", default)]
    online: bool,
}

fn binary() -> Option<std::path::PathBuf> {
    let exe = if cfg!(windows) { "tailscale.exe" } else { "tailscale" };
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let p = dir.join(exe);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    for env in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(dir) = std::env::var_os(env) {
            let p = std::path::Path::new(&dir).join("Tailscale").join("tailscale.exe");
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Returns the backend state and desktop peers, online first, then Linux, Windows, others.
pub fn query() -> Result<(String, Vec<Peer>), Error> {
    let bin = binary().ok_or(Error::NotInstalled)?;
    let mut cmd = Command::new(bin);
    cmd.args(["status", "--json"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd.output().map_err(|_| Error::Unavailable)?;
    if out.stdout.is_empty() {
        return Err(Error::Unavailable);
    }
    parse(&out.stdout)
}

pub fn parse(raw: &[u8]) -> Result<(String, Vec<Peer>), Error> {
    let st: Status = serde_json::from_slice(raw).map_err(|e| Error::Invalid(e.to_string()))?;
    let state = if st.backend_state.is_empty() { "Unknown".to_string() } else { st.backend_state };
    let mut peers: Vec<Peer> = st
        .peer
        .unwrap_or_default()
        .into_values()
        .filter_map(|e| {
            let os = e.os.trim().to_lowercase();
            if os == "android" || os == "ios" {
                return None;
            }
            let ip = e.ips.iter().find(|a| !a.contains(':'))?.clone();
            let mut hostname = e.host_name;
            if hostname.is_empty() {
                hostname = e.dns_name.split('.').next().unwrap_or_default().to_string();
            }
            if hostname.is_empty() {
                hostname = ip.clone();
            }
            Some(Peer { hostname, ip, os, online: e.online })
        })
        .collect();
    let rank = |os: &str| match os {
        "linux" => 0,
        "windows" => 1,
        "macos" => 2,
        _ => 3,
    };
    peers.sort_by(|a, b| {
        b.online
            .cmp(&a.online)
            .then(rank(&a.os).cmp(&rank(&b.os)))
            .then(a.hostname.to_lowercase().cmp(&b.hostname.to_lowercase()))
    });
    Ok((state, peers))
}
