//! Session log: crash-safe command/output logging to library/logs/.
//!
//! Opens a timestamped log file before the first REPL prompt.
//! Flushes after every write so the log is readable even after a crash.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// A session log that writes commands and output to a timestamped file.
pub struct SessionLog {
    writer: BufWriter<File>,
    pub path: PathBuf,
}

impl SessionLog {
    /// Open a new session log file in `library/logs/`.
    ///
    /// The filename is `session_YYYY-MM-DD_HHMMSS.log`.
    /// Creates the directory if needed.
    /// Returns `None` if the file cannot be opened (e.g., permission error) —
    /// the REPL continues without logging.
    pub fn open() -> Option<Self> {
        let logs_dir = PathBuf::from("library/logs");

        if let Err(e) = lumacore_luma::drive::create_dir_all(&logs_dir) {
            log::warn!("could not create session log directory {logs_dir:?}: {e}");
            return None;
        }

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Format as human-readable datetime: session_YYYY-MM-DD_HHMMSS.log
        let datetime = format_datetime(ts);
        let filename = format!("session_{datetime}.log");
        let path = logs_dir.join(filename);

        match lumacore_luma::drive::create(&path) {
            Ok(file) => {
                log::info!("session log: {path:?}");
                Some(Self {
                    writer: BufWriter::new(file),
                    path,
                })
            }
            Err(e) => {
                log::warn!("could not open session log {path:?}: {e}");
                None
            }
        }
    }

    /// Log a command line (prefixed with `> `).
    pub fn log_command(&mut self, cmd: &str) {
        let _ = writeln!(self.writer, "> {cmd}");
        let _ = self.writer.flush();
    }

    /// Log output text.
    pub fn log_output(&mut self, output: &str) {
        // Strip ANSI escape codes when writing to log file
        let clean = crate::text::strip_ansi(output);
        let _ = writeln!(self.writer, "{clean}");
        let _ = self.writer.flush();
    }
}

/// Format a Unix timestamp as YYYY-MM-DD_HHMMSS (UTC).
///
/// Uses pure arithmetic — no external dependencies.
fn format_datetime(secs: u64) -> String {
    // Days since Unix epoch
    let days = secs / 86400;
    let time_of_day = secs % 86400;

    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    // Convert days since epoch to year/month/day (Gregorian calendar)
    let (year, month, day) = days_to_ymd(days);

    format!("{year:04}-{month:02}-{day:02}_{h:02}{m:02}{s:02}")
}

/// Convert days since Unix epoch (1970-01-01) to (year, month, day).
fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    // Algorithm: civil calendar from http://howardhinnant.github.io/date_algorithms.html
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m as u32, d as u32)
}
