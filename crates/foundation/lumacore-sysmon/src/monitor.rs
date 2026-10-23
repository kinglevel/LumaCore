// ███╗   ███╗ ██████╗ ███╗   ██╗██╗████████╗ ██████╗ ██████╗
// ████╗ ████║██╔═══██╗████╗  ██║██║╚══██╔══╝██╔═══██╗██╔══██╗
// ██╔████╔██║██║   ██║██╔██╗ ██║██║   ██║   ██║   ██║██████╔╝
// ██║╚██╔╝██║██║   ██║██║╚██╗██║██║   ██║   ██║   ██║██╔══██╗
// ██║ ╚═╝ ██║╚██████╔╝██║ ╚████║██║   ██║   ╚██████╔╝██║  ██║
// ╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝
// FILE: monitor

//! The System Monitor: the station's log of what happens — logins, commands of every user,
//! show changes, warnings and errors. Every line is a SysmonLine (time, level, subsystem, message)
//! in a ring of RING_SIZE lines; TCP monitor clients and --RootMonitor get its text form,
//! subscribers its structured lines (the sysmon feed, the daily log file). A message starting with
//! [ERROR] / [WARN] sets the level.
//!
//! Owns:          the log: SysmonLine, its ring, levels, subscribers, the JSON forward, the daily log file, echo.
//! Doesn't own:   feedback events (feedback.rs), panic reports (panic_report.rs), what a line says (its writer).
//! Entry points:  sysmon_note, sysmon_line, sysmon_early, subscribe, history, start_log_file, SysmonLine, Level.
//! Decisions:     §26, §30 "12b2", §34 25.10.

use std::collections::VecDeque;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

pub static SERVER_START: OnceLock<Instant> = OnceLock::new();

/// Lines kept in memory (the window's scrollback).
pub const RING_SIZE: usize = 5000;
/// Daily log files kept in `library/logs/`.
pub const LOG_DAYS: i64 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Info,
    Warn,
    Error,
    /// Critical events (an alert, red): shutdown, restart, power loss.
    Alert,
}

#[derive(Debug, Clone, Serialize)]
pub struct SysmonLine {
    /// Increasing per process; clients use it to join history and live lines.
    pub seq: u64,
    /// Wall clock, ms since the Unix epoch (the window shows local time).
    pub time_ms: u64,
    /// Since server start.
    pub elapsed_ms: u64,
    pub level: Level,
    pub subsystem: String,
    pub message: String,
    /// A feedback event (progress, flash, message, status) — see `crate::feedback`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feedback: Option<crate::feedback::Feedback>,
    /// Not kept in the ring or the log file (progress ticks); live only.
    #[serde(skip)]
    pub transient: bool,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl SysmonLine {
    /// The classic one-line form: `  0h00m01.234s        Session : message`.
    pub fn text(&self) -> String {
        let prefix = match self.level {
            Level::Info => "",
            Level::Warn => "[WARN] ",
            Level::Error => "[ERROR] ",
            Level::Alert => "[ALERT] ",
        };
        format!(
            "{} {:>14} : {prefix}{}",
            format_elapsed(self.elapsed_ms),
            self.subsystem,
            self.message
        )
    }

    /// A log file line: local date and time, then the classic form.
    fn file_line(&self) -> String {
        use chrono::TimeZone;
        let time = chrono::Local
            .timestamp_millis_opt(self.time_ms as i64)
            .single()
            .map(|t| t.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
            .unwrap_or_default();
        format!("{time} {}", self.text())
    }
}

struct Ring {
    lines: VecDeque<SysmonLine>,
    seq: u64,
    subscribers: Vec<SyncSender<SysmonLine>>,
}

static RING: Mutex<Ring> = Mutex::new(Ring {
    lines: VecDeque::new(),
    seq: 0,
    subscribers: Vec::new(),
});

/// `--RootMonitor`: echo every line on stderr (also lines emitted without a
/// server context, e.g. feedback from commands).
static ECHO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_echo(on: bool) {
    ECHO.store(on, std::sync::atomic::Ordering::Relaxed);
}

fn format_elapsed(ms: u64) -> String {
    let total_secs = ms / 1000;
    format!(
        "{:3}h{:02}m{:02}.{:03}s",
        total_secs / 3600,
        (total_secs % 3600) / 60,
        total_secs % 60,
        ms % 1000
    )
}

/// Format elapsed time since server start as `{h:3}h{m:02}m{s:02}.{ms:03}s`.
pub fn sysmon_elapsed() -> String {
    format_elapsed(elapsed_ms())
}

fn elapsed_ms() -> u64 {
    SERVER_START
        .get()
        .map(|s| s.elapsed().as_millis() as u64)
        .unwrap_or_default()
}

/// `[ERROR] …` / `[WARN] …` → level and the message without the prefix.
fn split_level(message: &str) -> (Level, &str) {
    let m = message.trim_start();
    for (prefix, level) in [
        ("[ALERT]", Level::Alert),
        ("[ERROR]", Level::Error),
        ("[WARN]", Level::Warn),
        ("[WARNING]", Level::Warn),
    ] {
        if let Some(rest) = m.strip_prefix(prefix) {
            return (level, rest.trim_start());
        }
    }
    (Level::Info, message)
}

/// Make a line, keep it in the ring and hand it to the subscribers.
fn record(subsystem: &str, message: &str) -> SysmonLine {
    let (level, message) = split_level(message);
    record_with(subsystem, level, message, None, false)
}

/// A feedback event as a sysmon line (`crate::feedback`). `transient` lines
/// (progress ticks) reach live subscribers only.
pub fn record_feedback(
    subsystem: &str,
    level: Level,
    message: &str,
    feedback: crate::feedback::Feedback,
    transient: bool,
) -> SysmonLine {
    let line = record_with(subsystem, level, message, Some(feedback), transient);
    if !transient {
        log::info!("{}", line.text());
        if ECHO.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!("{}", line.text());
        }
    }
    line
}

fn record_with(
    subsystem: &str,
    level: Level,
    message: &str,
    feedback: Option<crate::feedback::Feedback>,
    transient: bool,
) -> SysmonLine {
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default();
    let mut ring = RING.lock().unwrap_or_else(|e| e.into_inner());
    ring.seq += 1;
    let line = SysmonLine {
        seq: ring.seq,
        time_ms,
        elapsed_ms: elapsed_ms(),
        level,
        subsystem: subsystem.to_string(),
        message: message.to_string(),
        feedback,
        transient,
    };
    if !transient {
        if ring.lines.len() >= RING_SIZE {
            ring.lines.pop_front();
        }
        ring.lines.push_back(line.clone());
    }
    // A slow subscriber misses lines rather than blocking the server; a gone
    // one is dropped.
    ring.subscribers
        .retain(|s| !matches!(s.try_send(line.clone()), Err(TrySendError::Disconnected(_))));
    line
}

/// The ring so far and a receiver for every line after it (no gap, no
/// duplicate: both are taken under the same lock).
pub fn subscribe() -> (Vec<SysmonLine>, Receiver<SysmonLine>) {
    let (tx, rx) = mpsc::sync_channel(4096);
    let mut ring = RING.lock().unwrap_or_else(|e| e.into_inner());
    ring.subscribers.push(tx);
    (ring.lines.iter().cloned().collect(), rx)
}

/// Structured history (oldest first).
pub fn history() -> Vec<SysmonLine> {
    RING.lock()
        .map(|r| r.lines.iter().cloned().collect())
        .unwrap_or_default()
}

/// Error and alert lines after `time_ms` (Unix ms) still in the ring: a
/// System Monitor window's `Self.Errors` since its Clear mark (`Set Self
/// Since Now`, decisions §30 "12c").
pub fn errors_since(time_ms: u64) -> u64 {
    RING.lock()
        .map(|r| {
            r.lines
                .iter()
                .filter(|l| l.time_ms > time_ms && matches!(l.level, Level::Error | Level::Alert))
                .count() as u64
        })
        .unwrap_or(0)
}

/// Return a snapshot of all sysmon history lines (text form).
pub fn sysmon_history_snapshot() -> Vec<String> {
    history().iter().map(SysmonLine::text).collect()
}

/// A sysmon line, echoed to stderr when `echo` (`--RootMonitor`) or the
/// station's echo setting is on.
pub fn sysmon_line(subsystem: &str, message: impl AsRef<str>, echo: bool) {
    let line = record(subsystem, message.as_ref()).text();
    log::info!("{line}");
    if echo || ECHO.load(std::sync::atomic::Ordering::Relaxed) {
        eprintln!("{line}");
    }
}

/// A sysmon line from a thread that doesn't hold the server state (shell
/// clients, discovery, UDP): the same line, echoed when `--RootMonitor` is on.
pub fn sysmon_note(subsystem: &str, message: impl AsRef<str>) {
    let line = record(subsystem, message.as_ref()).text();
    log::info!("{line}");
    if ECHO.load(std::sync::atomic::Ordering::Relaxed) {
        eprintln!("{line}");
    }
}

/// Emit a sysmon line before ServerContext exists (log + stderr only).
/// Also records to history so late-connecting monitor clients see startup.
pub fn sysmon_early(subsystem: &str, message: impl AsRef<str>) {
    let line = record(subsystem, message.as_ref()).text();
    log::info!("{line}");
    eprintln!("{line}");
}

/// Hand every line (history first) as JSON to `send` on its own thread — the
/// HTTP engine's `sysmon` and `feedback` feeds. Stops when `send` returns false.
pub fn forward_json(send: impl Fn(String) -> bool + Send + 'static) {
    let (_history, rx) = subscribe();
    std::thread::Builder::new()
        .name("sysmon-forward".into())
        .spawn(move || {
            for line in rx {
                let Ok(json) = serde_json::to_string(&line) else {
                    continue;
                };
                if !send(json) {
                    break;
                }
            }
        })
        .ok();
}

/// History as JSON lines (a new `sysmon` feed subscription).
pub fn history_json() -> Vec<String> {
    history()
        .iter()
        .filter_map(|l| serde_json::to_string(l).ok())
        .collect()
}

// *************************************************************************************************
// Log file
// *************************************************************************************************
/// Append every line to `<dir>/sysmon-YYYY-MM-DD.log` (local date), rotating
/// daily and deleting files older than [`LOG_DAYS`]. Starts with the lines
/// recorded so far (startup).
pub fn start_log_file(dir: impl Into<PathBuf>) {
    let dir = dir.into();
    let (history, rx) = subscribe();
    std::thread::Builder::new()
        .name("sysmon-log".into())
        .spawn(move || {
            // tidy: drive-ok (the sysmon log writer thread, never a command)
            let _ = std::fs::create_dir_all(&dir);
            let mut current: Option<(String, std::fs::File)> = None;
            let write = |line: &SysmonLine, current: &mut Option<(String, std::fs::File)>| {
                let day = chrono::Local::now().format("%Y-%m-%d").to_string();
                if current.as_ref().is_none_or(|(d, _)| *d != day) {
                    prune_logs(&dir, &day);
                    let path = dir.join(format!("sysmon-{day}.log"));
                    // tidy: drive-ok (the sysmon log writer thread)
                    match std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&path)
                    {
                        Ok(f) => *current = Some((day, f)),
                        Err(e) => {
                            log::warn!("sysmon log {}: {e}", path.display());
                            return;
                        }
                    }
                }
                if let Some((_, f)) = current.as_mut() {
                    let _ = writeln!(f, "{}", line.file_line());
                }
            };
            for line in &history {
                write(line, &mut current);
            }
            loop {
                match rx.recv_timeout(Duration::from_secs(60)) {
                    Ok(line) if line.transient => {}
                    Ok(line) => write(&line, &mut current),
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        })
        .ok();
}

/// Delete `sysmon-YYYY-MM-DD.log` files older than [`LOG_DAYS`] before `today`.
fn prune_logs(dir: &std::path::Path, today: &str) {
    let Ok(today) = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d") else {
        return;
    };
    // tidy: drive-ok (pruning old logs on the writer thread)
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(date) = name
            .strip_prefix("sysmon-")
            .and_then(|n| n.strip_suffix(".log"))
        else {
            continue;
        };
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
            && (today - date).num_days() >= LOG_DAYS
        {
            // tidy: drive-ok (pruning old logs on the writer thread)
            let _ = std::fs::remove_file(entry.path());
        }
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
