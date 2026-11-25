//! Saved computers, stored in %APPDATA%\DesktopRDP\profiles.json.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_PORT: u16 = 3389;

fn default_port() -> u16 {
    DEFAULT_PORT
}
fn default_mode() -> String {
    "windowed".into()
}
fn yes() -> bool {
    true
}

/// One saved computer. JSON keys match the Linux app's profiles.json.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default = "default_mode")]
    pub display_mode: String,
    #[serde(default = "yes")]
    pub sound: bool,
    #[serde(default = "yes")]
    pub clipboard: bool,
    /// Span the session over all local monitors (mstsc "use multimon").
    #[serde(default)]
    pub multimon: bool,
    #[serde(default = "yes")]
    pub save_password: bool,
}

impl Profile {
    /// "host" or "host:port" when the port is not the default one.
    pub fn endpoint(&self) -> String {
        if self.port == DEFAULT_PORT {
            self.host.clone()
        } else {
            crate::rdp::join_host_port(&self.host, self.port)
        }
    }

    pub fn display_name(&self) -> &str {
        if self.name.trim().is_empty() {
            &self.host
        } else {
            &self.name
        }
    }
}

fn config_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(std::env::temp_dir);
    base.join("DesktopRDP")
}

fn profiles_path() -> PathBuf {
    config_dir().join("profiles.json")
}

pub fn load() -> Vec<Profile> {
    let Ok(raw) = std::fs::read(profiles_path()) else {
        return Vec::new();
    };
    let Ok(entries) = serde_json::from_slice::<Vec<serde_json::Value>>(&raw) else {
        return Vec::new();
    };
    entries
        .into_iter()
        .filter_map(|v| serde_json::from_value::<Profile>(v).ok())
        .filter(|p| !p.host.trim().is_empty())
        .collect()
}

pub fn save(profiles: &[Profile]) -> std::io::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let data = serde_json::to_vec_pretty(profiles).map_err(std::io::Error::other)?;
    let tmp = dir.join("profiles.json.tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(tmp, profiles_path())
}

pub fn new_id() -> String {
    let mut b = [0u8; 16];
    let _ = getrandom::fill(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn find_by_host<'a>(profiles: &'a [Profile], host: &str) -> Option<&'a Profile> {
    let norm = host.trim().to_lowercase();
    profiles.iter().find(|p| p.host.trim().to_lowercase() == norm)
}
