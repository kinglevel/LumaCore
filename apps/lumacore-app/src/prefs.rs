//! Client-side settings of this machine (never in the show): the last
//! connection, auto-start, and per server + user the screen config and each
//! screen's window (open, position, size) and page state (◀ ▶ bars). The usual console
//! split: screen contents are show data, display positions are
//! station settings.
//!
//! Stored as JSON in `~/Library/Application Support/LumaCore/client.json`
//! (macOS) or `~/.config/lumacore/client.json`. Passwords and tokens are never
//! stored.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    /// Count down and reconnect to `last` (or start the local server) at launch.
    #[serde(default = "yes")]
    pub auto_start: bool,
    #[serde(default)]
    pub last: Option<LastConnection>,
    /// Station key (station id, else `host:port`) → its users.
    #[serde(default)]
    pub servers: BTreeMap<String, ServerPrefs>,
    /// This app's device id (decisions §30 "12a3"): every window of the app
    /// is one device; the station keeps its lock and normal theme by it.
    #[serde(default)]
    pub device: String,
    /// Kiosk: a console or kiosk screen without an OS keyboard — the lock
    /// surface opens the built-in keyboard by itself.
    #[serde(default)]
    pub kiosk: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            auto_start: true,
            last: None,
            servers: BTreeMap::new(),
            device: String::new(),
            kiosk: false,
        }
    }
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastConnection {
    /// The local server (started or attached by the app).
    pub local: bool,
    #[serde(default)]
    pub station_id: String,
    #[serde(default)]
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerPrefs {
    #[serde(default)]
    pub users: BTreeMap<String, UserPrefs>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPrefs {
    /// The screen config this machine uses; None = the user's default.
    #[serde(default)]
    pub config: Option<u32>,
    /// `"<config>.<screen>"` → that screen's window here.
    #[serde(default)]
    pub screens: BTreeMap<String, ScreenPrefs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenPrefs {
    #[serde(default = "yes")]
    pub open: bool,
    #[serde(default)]
    pub x: Option<i32>,
    #[serde(default)]
    pub y: Option<i32>,
    #[serde(default)]
    pub w: Option<u32>,
    #[serde(default)]
    pub h: Option<u32>,
    #[serde(default)]
    pub maximized: bool,
    /// The display it was on (its name) — x / y are then relative to that
    /// display; a missing display puts the window on the main one, scaled.
    #[serde(default)]
    pub display: Option<String>,
    /// State the page keeps per screen (`sidebars`, …).
    #[serde(default)]
    pub page: BTreeMap<String, Value>,
}

impl Default for ScreenPrefs {
    fn default() -> Self {
        Self {
            open: true,
            x: None,
            y: None,
            w: None,
            h: None,
            maximized: false,
            display: None,
            page: BTreeMap::new(),
        }
    }
}

pub fn screen_key(config: u32, screen: u32) -> String {
    format!("{config}.{screen}")
}

impl Prefs {
    pub fn path() -> PathBuf {
        crate::platform::prefs_path()
    }

    pub fn load() -> Self {
        let mut prefs: Self = std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        if prefs.device.is_empty() {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos());
            prefs.device = format!("app-{:x}{:x}", nanos, std::process::id());
            prefs.save();
        }
        prefs
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                let tmp = path.with_extension("json.tmp");
                if std::fs::write(&tmp, json).is_ok() {
                    let _ = std::fs::rename(&tmp, &path);
                }
            }
            Err(e) => log::warn!("client settings: {e}"),
        }
    }

    pub fn user(&mut self, station: &str, user: &str) -> &mut UserPrefs {
        self.servers
            .entry(station.to_string())
            .or_default()
            .users
            .entry(user.to_string())
            .or_default()
    }

    pub fn user_ref(&self, station: &str, user: &str) -> Option<&UserPrefs> {
        self.servers.get(station)?.users.get(user)
    }
}
