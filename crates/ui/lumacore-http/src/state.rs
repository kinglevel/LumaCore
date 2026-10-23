//! Shared HTTP state: lock-free diagnostic snapshot via ArcSwap.

use serde::{Deserialize, Serialize};

/// Snapshot of server diagnostics, published lock-free via ArcSwap.
///
/// Updated every ~1s from the main server loop. HTTP handlers read this
/// without contending on the ServerContext mutex.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticsSnapshot {
    /// Every long-lived thread's health (`Root.Runtime.Health`).
    pub threads: Vec<lumacore_tree::runtime::ThreadHealthView>,

    /// Build info.
    pub version: String,
    pub profile: String,
    pub target_os: String,
    pub target_arch: String,
    pub git_commit: String,
    pub build_id: String,

    /// Runtime state.
    pub uptime_secs: u64,
    pub show_path: Option<String>,
    pub session_count: u32,
    pub fixture_count: usize,
    pub universe_count: usize,
    pub output_count: usize,
    /// The Dmx protocols whose drivers run, with their active lines.
    pub protocols_running: std::collections::BTreeMap<String, u32>,

    /// Sysmon history snapshot.
    pub sysmon_history: Vec<String>,
}
