// ██╗    ██╗██╗███╗   ██╗██████╗  ██████╗ ██╗    ██╗███████╗
// ██║    ██║██║████╗  ██║██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██║ █╗ ██║██║██╔██╗ ██║██║  ██║██║   ██║██║ █╗ ██║███████╗
// ██║███╗██║██║██║╚██╗██║██║  ██║██║   ██║██║███╗██║╚════██║
// ╚███╔███╔╝██║██║ ╚████║██████╔╝╚██████╔╝╚███╔███╔╝███████║
//  ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚═════╝  ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: windows

//! Windows: prefs under `%APPDATA%`, `lumacore-server.exe`; Force stop is left to the Task
//! Manager; no app-wide menu bar.
//!
//! Owns:          the Windows side of `platform`.
//! Doesn't own:   anything shared.
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

/// The local server's binary, next to the app's.
pub const SERVER_EXE: &str = "lumacore-server.exe";

/// A Windows menu bar belongs to each window (HWND); the app has none yet.
pub fn init_menu(_menu: &muda::Menu) {}

/// `%APPDATA%\LumaCore\client.json`.
pub fn prefs_path() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LumaCore")
        .join("client.json")
}

/// The local server stays in the app's console group.
pub fn detach(_cmd: &mut Command) {}

/// Force stop isn't available here.
pub fn process_name(pid: u32) -> Result<Option<String>, String> {
    Err(format!("Force stop isn't available here: end process {pid} in the Task Manager."))
}

/// Never reached: `process_name` refuses first.
pub fn terminate(pid: u32) -> Result<(), String> {
    process_name(pid).map(|_| ())
}

/// Never reached: `process_name` refuses first.
pub fn kill(pid: u32) -> Result<(), String> {
    process_name(pid).map(|_| ())
}

/// The command that opens a web address in the system's browser (`rundll32 url.dll`).
pub fn browser(url: &str) -> Command {
    // Not `cmd /C start`: cmd would read `&` in the address as a command separator.
    let mut cmd = Command::new("rundll32");
    cmd.args(["url.dll,FileProtocolHandler", url]);
    cmd
}
