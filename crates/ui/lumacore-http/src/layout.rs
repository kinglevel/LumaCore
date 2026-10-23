//! Layout / view events of the live screens, for the `screen` feed on
//! `/ws/feeds` (decisions §30 "12d hardening": ReloadUI and capture requests
//! are Samples, so none is lost).
//!
//! The layouts themselves live in `Root.Runtime` screens
//! (`lumacore_tree::runtime::Screen`); stored views live on the user.

use std::collections::VecDeque;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum LayoutEvent {
    /// A user's stored views or view buttons changed (`GET /api/views`).
    ViewsChanged,
    /// A screen's layout or windows changed (`GET /api/screens/{no}`).
    ScreenChanged { no: u32, revision: u64 },
    /// `ReloadUI`: browsers showing one of `screens` reload (None = all).
    ReloadUi { screens: Option<Vec<u32>> },
    /// `Screenshot`: browsers showing one of `screens` render their screen
    /// as seen and upload it as `POST /api/screenshots?id=…&screen=…`.
    Capture { id: u64, screens: Vec<u32> },
}

/// How many screen events the station keeps for a page that resumes.
pub const SCREEN_EVENTS_KEPT: usize = 256;

/// The station's screen events with their `seq` (a ring): every event the
/// screen store reports, kept so a page that reconnects gets what it missed
/// (or an explicit gap). The newest seq is also a `watch`, which wakes the
/// feed sockets and the `windows` builds.
pub struct ScreenEvents {
    ring: Mutex<VecDeque<Value>>,
    seq: tokio::sync::watch::Sender<u64>,
}

impl Default for ScreenEvents {
    fn default() -> Self {
        Self {
            ring: Mutex::new(VecDeque::with_capacity(SCREEN_EVENTS_KEPT)),
            seq: tokio::sync::watch::channel(0).0,
        }
    }
}

impl ScreenEvents {
    /// Keep an event (called by the screen store's listeners: never touch the
    /// store from here).
    pub fn push(&self, event: &LayoutEvent) {
        let Ok(mut v) = serde_json::to_value(event) else {
            return;
        };
        let Ok(mut ring) = self.ring.lock() else {
            return;
        };
        let seq = *self.seq.borrow() + 1;
        v["seq"] = seq.into();
        if ring.len() == SCREEN_EVENTS_KEPT {
            ring.pop_front();
        }
        ring.push_back(v);
        self.seq.send_replace(seq);
    }

    /// The newest event's seq (0 = none yet).
    pub fn newest(&self) -> u64 {
        *self.seq.borrow()
    }

    /// Wakes on every new event.
    pub fn watch(&self) -> tokio::sync::watch::Receiver<u64> {
        self.seq.subscribe()
    }

    /// The events after `last`, and the seqs the ring no longer holds
    /// (`gap: (from, to)`).
    pub fn after(&self, last: u64) -> (Vec<Value>, Option<(u64, u64)>) {
        let Ok(ring) = self.ring.lock() else {
            return (Vec::new(), None);
        };
        let seq = |v: &Value| v["seq"].as_u64().unwrap_or(0);
        let gap = ring
            .front()
            .map(seq)
            .filter(|oldest| *oldest > last + 1)
            .map(|oldest| (last + 1, oldest - 1));
        (ring.iter().filter(|v| seq(v) > last).cloned().collect(), gap)
    }

    /// Whether a page that saw `last` can resume without a gap.
    pub fn holds(&self, last: u64) -> bool {
        self.after(last).1.is_none() && last <= self.newest()
    }
}
