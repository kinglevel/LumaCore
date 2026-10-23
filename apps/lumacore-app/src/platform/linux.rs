// ██╗     ██╗███╗   ██╗██╗   ██╗██╗  ██╗
// ██║     ██║████╗  ██║██║   ██║╚██╗██╔╝
// ██║     ██║██╔██╗ ██║██║   ██║ ╚███╔╝
// ██║     ██║██║╚██╗██║██║   ██║ ██╔██╗
// ███████╗██║██║ ╚████║╚██████╔╝██╔╝ ██╗
// ╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝
// FILE: linux

//! Linux: prefs under `~/.config`, Unix processes; no app-wide menu bar.
//!
//! Owns:          the Linux side of `platform`.
//! Doesn't own:   the shared Unix process handling (platform/unix.rs).
//! Entry points:  init_menu, prefs_path, SERVER_EXE, detach, process_name, terminate, kill, browser.
//! Decisions:     §36 4.

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

use std::path::PathBuf;
use std::process::Command;

pub use super::unix::{detach, kill, process_name, terminate};

/// The local server's binary, next to the app's.
pub const SERVER_EXE: &str = "lumacore-server";

/// A Linux menu bar belongs to each GTK window; the app has none yet.
pub fn init_menu(_menu: &muda::Menu) {}

/// `~/.config/lumacore/client.json`.
pub fn prefs_path() -> PathBuf {
    super::unix::home().join(".config/lumacore/client.json")
}

/// The command that opens a web address in the system's browser (`xdg-open`).
pub fn browser(url: &str) -> Command {
    let mut cmd = Command::new("xdg-open");
    cmd.arg(url);
    cmd
}
