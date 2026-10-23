//! The local server: the `lumacore-server` binary next to the app, started as
//! a child process (its own process group, so it survives "Quit Client" and a
//! terminal Ctrl-C), or an already running server on this machine that the
//! app attaches to (it then isn't the app's to watch).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::http;

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = lumacore_wire::REMOTE_HTTP_PORT;

/// Station info of the server running on this machine, if any.
pub fn probe() -> Option<Value> {
    match http::get(HOST, PORT, "/api/station", None) {
        Ok((200, v)) => Some(v),
        _ => None,
    }
}

/// Start the child server. Returns its pid; `on_exit(pid, code)` runs when it
/// ends (`None` = killed by a signal). A Restart re-execs in place, so the pid
/// and this watch stay valid.
pub fn spawn(on_exit: impl FnOnce(u32, Option<i32>) + Send + 'static) -> Result<u32, String> {
    spawn_with(&[], on_exit)
}

/// `spawn` with extra server arguments (`--clean-start`).
pub fn spawn_with(
    extra: &[&str],
    on_exit: impl FnOnce(u32, Option<i32>) + Send + 'static,
) -> Result<u32, String> {
    let exe = server_binary()?;
    let mut cmd = Command::new(exe);
    // No --web-root: the server serves its built-in WebUI (decisions §36 2).
    cmd.args(extra).stdin(Stdio::null());
    crate::platform::detach(&mut cmd);
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("could not start the local server: {e}"))?;
    let pid = child.id();
    std::thread::spawn(move || {
        let code = child.wait().ok().and_then(|s| s.code());
        on_exit(pid, code);
    });
    Ok(pid)
}

/// `lumacore-server`, shipped next to the app binary (`Contents/MacOS` in a
/// .app bundle, `target/<profile>/` in development).
fn server_binary() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let server = exe.parent().unwrap_or(Path::new(".")).join(crate::platform::SERVER_EXE);
    if server.exists() {
        Ok(server)
    } else {
        Err(format!(
            "could not start the local server: {} is missing (build it with `cargo build -p lumacore-station --features http`)",
            server.display()
        ))
    }
}

/// A local server that isn't this app's own build (decisions §23 "App ↔
/// local server hardening").
#[derive(Debug, Clone)]
pub struct OtherVersion {
    /// Why it doesn't count as ours.
    pub reason: String,
    /// Its build id ("unknown" from a server older than the field).
    pub build: String,
    /// Its pid, when it says (servers since step 9b).
    pub pid: Option<u32>,
    /// The client protocol matches: the app could still use it.
    pub usable: bool,
}

/// Is the running server `info` (`/api/station`) the build next to this app?
/// Another client protocol (missing = before step 8), another build id, or
/// the same binary rebuilt after it started (the id doesn't change between
/// uncommitted rebuilds) all say no.
pub fn other_version(info: &Value) -> Option<OtherVersion> {
    other_version_of(info, server_binary().ok().as_deref())
}

/// `other_version` against the server binary `mine`.
fn other_version_of(info: &Value, mine: Option<&Path>) -> Option<OtherVersion> {
    let build = info["build"].as_str().unwrap_or("unknown").to_string();
    let pid = info["pid"].as_u64().map(|p| p as u32);
    let protocol = info["client_protocol"].as_u64();
    let other = |reason: String, usable: bool| {
        Some(OtherVersion {
            reason,
            build: build.clone(),
            pid,
            usable,
        })
    };
    if protocol != Some(u64::from(lumacore_wire::CLIENT_PROTOCOL)) {
        return other(
            match protocol {
                Some(p) => format!(
                    "it speaks client protocol {p}, this app {}",
                    lumacore_wire::CLIENT_PROTOCOL
                ),
                None => "it is from an older LumaCore (no client protocol)".to_string(),
            },
            false,
        );
    }
    let mine = mine?;
    if let Some(own) = binary_build(mine)
        && own != build
    {
        return other(format!("it is build {build}, this app's server is {own}"), true);
    }
    let same_file = info["exe"]
        .as_str()
        .and_then(|e| std::fs::canonicalize(e).ok())
        .zip(std::fs::canonicalize(mine).ok())
        .is_some_and(|(a, b)| a == b);
    let rebuilt_ms = std::fs::metadata(mine)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64);
    if same_file
        && let (Some(rebuilt), Some(started)) = (rebuilt_ms, info["started_ms"].as_u64())
        && rebuilt > started
    {
        return other(
            "its binary was rebuilt after it started".to_string(),
            true,
        );
    }
    None
}

/// `lumacore-server --version` → its build id.
fn binary_build(exe: &Path) -> Option<String> {
    let out = Command::new(exe).arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.trim()
        .strip_prefix("lumacore-server ")
        .map(str::to_string)
}

/// Stop the local server process `pid` the hard way, after the operator
/// asked (Force stop): SIGTERM — the server still says goodbye and keeps an
/// autosave — then SIGKILL if it is still there after 5 s. Only a pid the
/// server reported itself, and only when that process is a LumaCore server.
pub fn force_stop(pid: u32) -> Result<(), String> {
    let Some(name) = crate::platform::process_name(pid)? else {
        return wait_down(Duration::from_secs(5))
            .then_some(())
            .ok_or_else(|| format!("Process {pid} is gone, but something still answers on port {PORT}."));
    };
    if !name.starts_with("lumacore") {
        return Err(format!("Process {pid} is \"{name}\", not a LumaCore server — not stopped."));
    }
    crate::platform::terminate(pid)?;
    if wait_down(Duration::from_secs(5)) {
        return Ok(());
    }
    crate::platform::kill(pid)?;
    if wait_down(Duration::from_secs(3)) {
        Ok(())
    } else {
        Err(format!("The local server (pid {pid}) did not stop."))
    }
}

/// How long the app waits for a Shutdown before it offers Force stop: the
/// server's shutdown deadline (5 s) plus a margin.
pub const STOP_WAIT: Duration = Duration::from_secs(10);

/// Wait until the local server answers (a fresh one loads its show first).
pub fn wait_ready(timeout: Duration) -> Option<Value> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(v) = probe() {
            return Some(v);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    None
}

/// Wait until the local server is gone (after a Shutdown).
pub fn wait_down(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if probe().is_none() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    false
}

/// Shutdown of the local server from the app (no session window open on
/// it): `Shutdown /NoConfirmation /Save|/NoSave` over the local command port.
pub fn shutdown(save: Option<bool>) -> Result<(), String> {
    command("Shutdown", save)
}

/// CleanStart of the running local server (home screen).
pub fn clean_start(save: Option<bool>) -> Result<(), String> {
    command("CleanStart", save)
}

/// `<verb> /NoConfirmation [/Save|/NoSave]` as Admin over the local command port.
fn command(verb: &str, save: Option<bool>) -> Result<(), String> {
    let flags = match save {
        Some(true) => " /Save",
        Some(false) => " /NoSave",
        None => "",
    };
    let request = lumacore_wire::RemoteRequest::Command {
        user: "Admin".to_string(),
        cmd: format!("{verb} /NoConfirmation{flags}"),
    };
    let reply = lumacore_wire::send_request_to(
        HOST,
        lumacore_wire::REMOTE_CMD_PORT,
        &request,
        Duration::from_secs(5),
    )?;
    if reply.ok { Ok(()) } else { Err(reply.message) }
}

/// The show state for the stop confirmation: `(show name, unsaved)`.
pub fn show_state() -> (Option<String>, bool) {
    match http::get(HOST, PORT, "/api/shows", None) {
        Ok((200, v)) => (
            v["current"]["name"].as_str().map(str::to_string),
            v["current"]["dirty"].as_bool().unwrap_or(false),
        ),
        _ => (None, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn server() -> Option<PathBuf> {
        let p = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/lumacore-server"));
        p.exists().then_some(p)
    }

    #[test]
    fn another_protocol_or_none_is_another_version_and_not_usable() {
        let old = other_version_of(&json!({ "build": "x" }), None).expect("no protocol");
        assert!(!old.usable && old.reason.contains("older LumaCore"), "{old:?}");
        let other = other_version_of(&json!({ "client_protocol": 999 }), None).expect("protocol 999");
        assert!(!other.usable && other.reason.contains("999"), "{other:?}");
    }

    /// Needs `target/debug/lumacore-server` (built by the station's tests).
    #[test]
    fn the_same_build_is_ours_until_its_binary_is_rebuilt() {
        let Some(exe) = server() else {
            eprintln!("skipped: target/debug/lumacore-server isn't built");
            return;
        };
        let build = binary_build(&exe).expect("--version prints the build");
        let protocol = lumacore_wire::CLIENT_PROTOCOL;
        let info = |started_ms: u64| {
            json!({ "build": build, "client_protocol": protocol, "exe": exe, "pid": 1, "started_ms": started_ms })
        };
        let future = u64::MAX / 2;
        assert!(other_version_of(&info(future), Some(&exe)).is_none());
        let rebuilt = other_version_of(&info(0), Some(&exe)).expect("started before the binary");
        assert!(rebuilt.usable && rebuilt.reason.contains("rebuilt"), "{rebuilt:?}");
        let mut older = info(future);
        older["build"] = "0.0.1/debug/x/abc".into();
        let other = other_version_of(&older, Some(&exe)).expect("another build");
        assert!(other.usable && other.reason.contains("0.0.1"), "{other:?}");
    }
}
