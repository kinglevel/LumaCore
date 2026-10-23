// ██╗   ██╗███╗   ██╗██╗██╗  ██╗
// ██║   ██║████╗  ██║██║╚██╗██╔╝
// ██║   ██║██╔██╗ ██║██║ ╚███╔╝
// ██║   ██║██║╚██╗██║██║ ██╔██╗
// ╚██████╔╝██║ ╚████║██║██╔╝ ██╗
//  ╚═════╝ ╚═╝  ╚═══╝╚═╝╚═╝  ╚═╝
// FILE: unix

//! What macOS and Linux share: the local server in its own process group, and finding and
//! signalling it by pid with `ps` / `kill`.
//!
//! Owns:          detach, process_name, terminate, kill for Unix.
//! Doesn't own:   when to stop (local_server::force_stop).
//! Entry points:  detach, process_name, terminate, kill.
//! Decisions:     §36 4, §23 "App ↔ local server hardening".

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

use std::path::Path;
use std::process::Command;

/// `$HOME` (`.` without one).
pub fn home() -> std::path::PathBuf {
    std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_else(|| ".".into())
}

/// The local server gets its own process group, so a Ctrl-C to the app doesn't reach it.
pub fn detach(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
}

/// The executable name of process `pid` (`None` = no such process).
pub fn process_name(pid: u32) -> Result<Option<String>, String> {
    let out = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .map_err(|e| e.to_string())?;
    let comm = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok((!comm.is_empty()).then(|| {
        Path::new(&comm)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or(comm)
    }))
}

fn signal(pid: u32, signal: &str) -> Result<(), String> {
    Command::new("kill")
        .args([signal, &pid.to_string()])
        .status()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Ask process `pid` to stop (SIGTERM: the server still says goodbye and keeps an autosave).
pub fn terminate(pid: u32) -> Result<(), String> {
    signal(pid, "-TERM")
}

/// Stop process `pid` at once (SIGKILL).
pub fn kill(pid: u32) -> Result<(), String> {
    signal(pid, "-KILL")
}
