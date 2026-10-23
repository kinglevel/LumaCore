// ██████╗ ████████╗██████╗
// ██╔══██╗╚══██╔══╝██╔══██╗
// ██████╔╝   ██║   ██████╔╝
// ██╔═══╝    ██║   ██╔═══╝
// ██║        ██║   ██║
// ╚═╝        ╚═╝   ╚═╝
// FILE: ptp

//! Real IEEE 1588 PTP, read-only (decisions §24 "Step 9 hardening"): when
//! linuxptp runs on this machine, its state is read every 2 s through
//! `pmc` (ptp4l's management client, over its local socket) and shown next
//! to our own stream clock sync (`Runtime.Health.Clock`, the Realtime
//! window). LumaCore never configures PTP; elsewhere it is "not available".
//!
//! Owns:          reading linuxptp's state through pmc (read-only).
//! Doesn't own:   the session clock (lumacore-base clock, lumasession), where the status is shown (HealthBoard).
//! Entry points:  spawn_ptp_reader, parse.
//! Decisions:     §24 "Step 9 hardening", §34 20.6.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use lumacore_runtime::{HealthBoard, PtpStatus};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

const POLL: Duration = Duration::from_secs(2);
const PMC_TIMEOUT: Duration = Duration::from_secs(1);
/// ptp4l's management socket.
const PTP4L_SOCKET: &str = "/var/run/ptp4l";

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Poll linuxptp until `stop`; without it, say why once and idle.
pub fn spawn_ptp_reader(health: Arc<HealthBoard>, stop: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("ptp-reader".to_string())
        .spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                health.set_ptp(read());
                let until = Instant::now() + POLL;
                while Instant::now() < until && !stop.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        })
        .expect("should spawn ptp reader thread")
}

/// One reading: the status, or why there is none.
fn read() -> Result<PtpStatus, String> {
    if !cfg!(target_os = "linux") {
        return Err("not available on this OS (linuxptp is Linux only)".to_string());
    }
    if !std::path::Path::new(PTP4L_SOCKET).exists() {
        return Err("ptp4l is not running".to_string());
    }
    let mut child = Command::new("pmc")
        .args([
            "-u",
            "-b",
            "0",
            "GET TIME_STATUS_NP",
            "GET PORT_DATA_SET",
            "GET CURRENT_DATA_SET",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("pmc: {e}"))?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < PMC_TIMEOUT => thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("pmc did not answer".to_string());
            }
        }
    }
    let mut out = String::new();
    child
        .stdout
        .take()
        .ok_or("pmc: no output")?
        .read_to_string(&mut out)
        .map_err(|e| format!("pmc: {e}"))?;
    parse(&out).ok_or_else(|| "pmc: no PTP answer".to_string())
}

/// Read pmc's `key value` lines.
pub fn parse(out: &str) -> Option<PtpStatus> {
    let mut status = PtpStatus::default();
    let mut any = false;
    for line in out.lines() {
        let mut words = line.split_whitespace();
        let (Some(key), Some(value)) = (words.next(), words.next()) else {
            continue;
        };
        let number = || value.parse::<f64>().ok().map(|v| v as i64);
        match key {
            "master_offset" => {
                status.offset_ns = number()?;
                any = true;
            }
            "meanPathDelay" => status.path_delay_ns = number().unwrap_or(0),
            "gmIdentity" => status.grandmaster = value.to_string(),
            "gmPresent" => status.gm_present = value == "true",
            "portState" => status.port_state = value.to_string(),
            "stepsRemoved" => status.steps_removed = value.parse().unwrap_or(0),
            _ => {}
        }
    }
    any.then(|| {
        status.read_ms = lumacore_base::clock::now_ms();
        status
    })
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
