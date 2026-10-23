//! LumaCore desktop app: the launcher and thin client — home window
//! (auto-start countdown, server discovery, Start local server, manual
//! address), login, one window per screen of the chosen screen config.
//!
//! The local server is the `lumacore-server` binary shipped next to this one,
//! started as a child process (decisions §23 "Step 8", the usual launcher
//! shape). The app compiles only `lumacore-wire`, never the
//! station.

mod app;
mod assets;
mod discovery;
mod http;
mod ipc;
mod local_server;
mod platform;
mod prefs;

fn main() {
    env_logger::init();
    app::run();
}
