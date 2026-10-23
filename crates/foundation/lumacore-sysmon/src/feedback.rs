// ███████╗███████╗███████╗██████╗ ██████╗  █████╗  ██████╗██╗  ██╗
// ██╔════╝██╔════╝██╔════╝██╔══██╗██╔══██╗██╔══██╗██╔════╝██║ ██╔╝
// █████╗  █████╗  █████╗  ██║  ██║██████╔╝███████║██║     █████╔╝
// ██╔══╝  ██╔══╝  ██╔══╝  ██║  ██║██╔══██╗██╔══██║██║     ██╔═██╗
// ██║     ███████╗███████╗██████╔╝██████╔╝██║  ██║╚██████╗██║  ██╗
// ╚═╝     ╚══════╝╚══════╝╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝
// FILE: feedback

//! One feedback system: progress, flashes, messages (notifications) and statuses. Every event is a
//! System Monitor line carrying a Feedback; UI engines draw it (the feedback feed). Importance is
//! the safety floor, set by the emitter: Critical is always a pop-up to everyone, Important reaches
//! everyone, Normal follows the user's notification settings, Info stays in the System Monitor.
//!
//! Owns:          feedback events: progress, flashes, messages, statuses, client presence; their categories, levels and surfaces.
//! Doesn't own:   what reads the show for them (lumacore-state show_feedback.rs), drawing them (the kit, the TUI).
//! Entry points:  flash, message, status, Progress, Feedback, Importance, ClientPresence.
//! Decisions:     §30 "12b2", §34 25.10.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::{Level, record_feedback};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Progress,
    Flash,
    Message,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Importance {
    Info,
    Normal,
    Important,
    Critical,
}

/// Message categories (message center rows + plugins).
pub mod category {
    pub const SYSTEM: &str = "system";
    pub const SHOW: &str = "show";
    pub const CMD: &str = "cmd";
    pub const POWER: &str = "power";
    pub const NETWORK: &str = "network";
    pub const USB: &str = "usb";
    pub const CHAT: &str = "chat";
    pub const PLUGIN: &str = "plugin";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressState {
    /// `(from, to)`; None = an open-ended counter (no fill).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<(i64, i64)>,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feedback {
    /// Progress / status: the same id for every update; messages: unique.
    pub id: u64,
    pub kind: Kind,
    pub importance: Importance,
    pub category: String,
    pub title: String,
    #[serde(default)]
    pub text: String,
    /// Audience: one user; None = everyone on the station.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressState>,
    /// Status key (`highlight`, `sacn`, …) and its icon glyph / short label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glyph: Option<String>,
    /// Progress running / status on; false = stopped / off.
    pub active: bool,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Running progress and statuses that are on (as their latest sysmon line,
/// JSON) — a UI that connects later gets them first.
static ACTIVE: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

fn active_key(f: &Feedback) -> Option<String> {
    match f.kind {
        Kind::Progress => Some(format!("progress.{}", f.id)),
        Kind::Status => f.key.as_ref().map(|k| format!("status.{k}")),
        _ => None,
    }
}

fn subsystem(category: &str) -> String {
    let mut c = category.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

fn emit(level: Level, f: Feedback, transient: bool) {
    let message = match (&f.kind, f.text.is_empty()) {
        (Kind::Status, _) => format!("{} {}", f.title, if f.active { "on" } else { "off" }),
        (_, true) => f.title.clone(),
        (_, false) => format!("{}: {}", f.title, f.text),
    };
    let key = active_key(&f);
    let active = f.active;
    let line = record_feedback(&subsystem(&f.category), level, &message, f, transient);
    if let Some(key) = key {
        let mut map = ACTIVE.lock().unwrap_or_else(|e| e.into_inner());
        if active {
            if let Ok(json) = serde_json::to_string(&line) {
                map.insert(key, json);
            }
        } else {
            map.remove(&key);
        }
    }
}

/// Running progress and statuses that are on, for `user` (station-wide ones
/// and their own), as sysmon line JSON.
pub fn active_json(user: &str) -> Vec<String> {
    let map = ACTIVE.lock().unwrap_or_else(|e| e.into_inner());
    map.values()
        .filter(|json| {
            serde_json::from_str::<serde_json::Value>(json)
                .ok()
                .and_then(|v| {
                    v["feedback"]["user"]
                        .as_str()
                        .map(|u| u.eq_ignore_ascii_case(user))
                })
                .unwrap_or(true)
        })
        .cloned()
        .collect()
}

// *************************************************************************************************
// Notification settings
// *************************************************************************************************
pub const CATEGORIES: &[&str] = &[
    category::SYSTEM,
    category::SHOW,
    category::CMD,
    category::POWER,
    category::NETWORK,
    category::USB,
    category::CHAT,
    category::PLUGIN,
];
pub const LEVELS: &[&str] = &["info", "warn", "error", "alert"];
pub const SURFACES: &[&str] = &["none", "card", "popup"];
/// Timed cards stay this long (5 s).
pub const TIMED_MS: u32 = 5000;

/// Default surface of a message by level: info → none (System Monitor only),
/// warnings and errors → card, alerts → pop-up.
pub fn default_surface(level: &str) -> &'static str {
    match level {
        "warn" | "error" => "card",
        "alert" => "popup",
        _ => "none",
    }
}

// *************************************************************************************************
// Flash and message
// *************************************************************************************************
/// A short confirmation for everyone on the station ("Show saved"): the
/// bottom flash.
pub fn flash(text: &str, importance: Importance) {
    let f = Feedback {
        id: next_id(),
        kind: Kind::Flash,
        importance,
        category: category::SHOW.into(),
        title: text.into(),
        text: String::new(),
        user: None,
        progress: None,
        key: None,
        glyph: None,
        active: true,
    };
    emit(Level::Info, f, false);
}

/// A message (notification): `level` colours it, `importance` decides who
/// can mute it; `user` = None for the whole station.
pub fn message(
    level: Level,
    category: &str,
    importance: Importance,
    title: &str,
    text: &str,
    user: Option<&str>,
) {
    let f = Feedback {
        id: next_id(),
        kind: Kind::Message,
        importance,
        category: category.into(),
        title: title.into(),
        text: text.into(),
        user: user.map(str::to_string),
        progress: None,
        key: None,
        glyph: None,
        active: true,
    };
    emit(level, f, false);
}

// *************************************************************************************************
// Stations and clients (important: everyone sees them)
// *************************************************************************************************
/// Another station appeared in the session (discovery).
pub fn station_joined(name: &str, address: &str) {
    message(
        Level::Info,
        category::NETWORK,
        Importance::Important,
        "Station joined",
        &format!("{name} ({address})"),
        None,
    );
}

/// A station stopped answering (discovery timeout).
pub fn station_lost(name: &str, key: &str) {
    message(
        Level::Warn,
        category::NETWORK,
        Importance::Important,
        "Station lost",
        &format!("{name} ({key})"),
        None,
    );
}

/// Connected clients per (user, address). Reloads and extra screen windows
/// don't notify: "connected" only for the first client, "disconnected" only
/// if the last one stays away for [`LEAVE_GRACE`].
static PRESENCE: Mutex<BTreeMap<(String, String), (usize, bool)>> = Mutex::new(BTreeMap::new());
const LEAVE_GRACE: Duration = Duration::from_secs(3);

/// A UI / shell client of `user` from `from` is connected while this lives.
pub struct ClientPresence {
    key: (String, String),
}

impl ClientPresence {
    pub fn join(user: &str, from: &str) -> Self {
        let key = (user.to_string(), from.to_string());
        let announce = {
            let mut map = PRESENCE.lock().unwrap_or_else(|e| e.into_inner());
            let entry = map.entry(key.clone()).or_insert((0, false));
            entry.0 += 1;
            // Back within the grace time: no message either way.
            let first = entry.0 == 1 && !entry.1;
            entry.1 = false;
            first
        };
        if announce {
            message(
                Level::Info,
                category::NETWORK,
                Importance::Important,
                "Connected",
                &format!("{user} from {from}"),
                None,
            );
        }
        Self { key }
    }
}

impl Drop for ClientPresence {
    fn drop(&mut self) {
        let last = {
            let mut map = PRESENCE.lock().unwrap_or_else(|e| e.into_inner());
            let Some(entry) = map.get_mut(&self.key) else {
                return;
            };
            entry.0 = entry.0.saturating_sub(1);
            if entry.0 == 0 {
                entry.1 = true;
            }
            entry.0 == 0
        };
        if !last {
            return;
        }
        let key = self.key.clone();
        std::thread::spawn(move || {
            std::thread::sleep(LEAVE_GRACE);
            let gone = {
                let mut map = PRESENCE.lock().unwrap_or_else(|e| e.into_inner());
                match map.get(&key) {
                    Some((0, true)) => {
                        map.remove(&key);
                        true
                    }
                    _ => false,
                }
            };
            if gone {
                message(
                    Level::Warn,
                    category::NETWORK,
                    Importance::Important,
                    "Disconnected",
                    &format!("{} from {}", key.0, key.1),
                    None,
                );
            }
        });
    }
}

// *************************************************************************************************
// Status
// *************************************************************************************************
/// Last emitted state per status key (emit only changes).
static STATUSES: Mutex<BTreeMap<String, (bool, String)>> = Mutex::new(BTreeMap::new());

/// A station status (command line icon): `key` on / off with a short label
/// ("HI", "sACN 2") and glyph. Emits only when it changes.
pub fn status(key: &str, active: bool, label: &str, glyph: &str, level: Level) {
    {
        let mut map = STATUSES.lock().unwrap_or_else(|e| e.into_inner());
        let now = (active, label.to_string());
        if map.get(key) == Some(&now) {
            return;
        }
        if !active && !map.contains_key(key) {
            return; // never shown: nothing to switch off
        }
        map.insert(key.to_string(), now);
    }
    let f = Feedback {
        id: 0,
        kind: Kind::Status,
        importance: Importance::Info,
        category: category::SYSTEM.into(),
        title: label.into(),
        text: String::new(),
        user: None,
        progress: None,
        key: Some(key.into()),
        glyph: Some(glyph.into()),
        active,
    };
    emit(level, f, false);
}

// *************************************************************************************************
// Progress
// *************************************************************************************************
/// A progress bar (`StartProgress`): stacked top-centre pills. With a
/// range it fills; without one it's a counter. Updates are throttled and kept
/// out of the log; start and stop are logged. Dropping it stops it, so no bar
/// can be left behind.
pub struct Progress {
    f: Feedback,
    last_emit: Instant,
    stopped: bool,
}

const PROGRESS_INTERVAL: Duration = Duration::from_millis(50);

impl Progress {
    /// `StartProgress(title)`; everyone on the station sees it.
    pub fn start(title: &str, category: &str, importance: Importance) -> Self {
        let f = Feedback {
            id: next_id(),
            kind: Kind::Progress,
            importance,
            category: category.into(),
            title: title.into(),
            text: String::new(),
            user: None,
            progress: Some(ProgressState {
                range: None,
                value: 0,
            }),
            key: None,
            glyph: None,
            active: true,
        };
        emit(Level::Info, f.clone(), false);
        Self {
            f,
            last_emit: Instant::now(),
            stopped: false,
        }
    }

    pub fn id(&self) -> u64 {
        self.f.id
    }

    /// `SetProgressRange`.
    pub fn range(&mut self, from: i64, to: i64) {
        if let Some(p) = &mut self.f.progress {
            p.range = Some((from, to));
        }
        self.tick(true);
    }

    /// `SetProgress`.
    pub fn set(&mut self, value: i64) {
        if let Some(p) = &mut self.f.progress {
            p.value = value;
        }
        self.tick(false);
    }

    /// `IncProgress`.
    pub fn inc(&mut self, by: i64) {
        if let Some(p) = &mut self.f.progress {
            p.value += by;
        }
        self.tick(false);
    }

    /// `SetProgressText`.
    pub fn text(&mut self, text: &str) {
        self.f.text = text.into();
        self.tick(false);
    }

    /// Text and value together (one update).
    pub fn update(&mut self, value: i64, text: &str) {
        if let Some(p) = &mut self.f.progress {
            p.value = value;
        }
        self.f.text = text.into();
        self.tick(false);
    }

    fn tick(&mut self, force: bool) {
        if force || self.last_emit.elapsed() >= PROGRESS_INTERVAL {
            self.last_emit = Instant::now();
            emit(Level::Info, self.f.clone(), true);
        }
    }

    /// `StopProgress`.
    pub fn stop(mut self) {
        self.finish();
    }

    fn finish(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        // The last state, then the stop (so a fast operation still shows its end).
        emit(Level::Info, self.f.clone(), true);
        self.f.active = false;
        emit(Level::Info, self.f.clone(), false);
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.finish();
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
