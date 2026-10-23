// ███╗   ███╗ █████╗  ██████╗ ██████╗ ███████╗
// ████╗ ████║██╔══██╗██╔════╝██╔═══██╗██╔════╝
// ██╔████╔██║███████║██║     ██║   ██║███████╗
// ██║╚██╔╝██║██╔══██║██║     ██║   ██║╚════██║
// ██║ ╚═╝ ██║██║  ██║╚██████╗╚██████╔╝███████║
// ╚═╝     ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚══════╝
// FILE: macos

//! macOS: the menu bar is the app's (NSApp), prefs under Application Support, Unix processes.
//!
//! Owns:          the macOS side of `platform`.
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

/// The menu bar belongs to the application on macOS.
pub fn init_menu(menu: &muda::Menu) {
    menu.init_for_nsapp();
}

/// `~/Library/Application Support/LumaCore/client.json`.
pub fn prefs_path() -> PathBuf {
    super::unix::home().join("Library/Application Support/LumaCore/client.json")
}

/// The command that opens a web address in the system's browser (`open`).
pub fn browser(url: &str) -> Command {
    let mut cmd = Command::new("open");
    cmd.arg(url);
    cmd
}
