mod endpoint_tests;
mod engine_tests;
mod live_screen_tests;
mod state_tests;
mod static_file_tests;
mod websocket_tests;

use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;

use crate::{ConsoleBridge, DiagnosticsSnapshot, HttpState};

/// The login token the test console accepts (as Admin).
pub(crate) const TEST_TOKEN: &str = "test-token";
/// The device the test console says is DeskLocked.
pub(crate) const LOCKED_DEVICE: &str = "locked-tab";

/// A console that runs nothing: commands fail, data queries return empty JSON;
/// System Monitor history comes from the test's diagnostics snapshot.
struct NullConsole {
    diagnostics: Arc<ArcSwap<DiagnosticsSnapshot>>,
    /// A render engine's numbers the tests write frames into.
    render: Arc<lumacore_tree::runtime::RenderStats>,
    knobs: Knobs,
    /// When set, `execute` blocks until it is released (like a bridge
    /// waiting on a busy state thread).
    hold: Option<Arc<Hold>>,
}

/// What a test changes in the [`NullConsole`]: the station view and the
/// counters that wake the shared feed builds.
#[derive(Clone)]
pub(crate) struct Knobs {
    pub station: Arc<std::sync::Mutex<serde_json::Value>>,
    pub revisions: Arc<tokio::sync::watch::Sender<crate::Revisions>>,
    /// Tokens the console accepts besides [`TEST_TOKEN`]: token → user.
    pub tokens: Arc<std::sync::Mutex<Vec<(String, String)>>>,
}

impl Default for Knobs {
    fn default() -> Self {
        Self {
            station: Arc::default(),
            revisions: Arc::new(tokio::sync::watch::Sender::new(crate::Revisions::default())),
            tokens: Arc::default(),
        }
    }
}

impl Knobs {
    /// Change the station view and tell the feeds.
    pub fn set_station(&self, v: serde_json::Value) {
        *self.station.lock().unwrap() = v;
        self.revisions.send_modify(|r| r.station += 1);
    }
}

/// A gate `execute` waits at: `release` opens it.
#[derive(Default)]
pub(crate) struct Hold {
    open: std::sync::Mutex<bool>,
    opened: std::sync::Condvar,
}

impl Hold {
    pub(crate) fn release(&self) {
        *self.open.lock().unwrap() = true;
        self.opened.notify_all();
    }

    fn wait(&self) {
        let mut open = self.open.lock().unwrap();
        while !*open {
            open = self.opened.wait(open).unwrap();
        }
    }
}

impl ConsoleBridge for NullConsole {
    fn execute(&self, _user: &str, device: Option<&str>, _cmd: &str) -> Result<String, String> {
        if let Some(hold) = &self.hold {
            hold.wait();
            return Ok("held".to_string());
        }
        // Says which device the line named (R1-42 tests).
        Err(format!("no console in tests (device {})", device.unwrap_or("none")))
    }
    fn user_views(&self, _user: &str) -> serde_json::Value {
        serde_json::Value::Null
    }
    fn shows(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
    fn login(
        &self,
        _user: &str,
        _password: Option<&str>,
        _remote: bool,
        _from: &str,
    ) -> Result<(String, String), String> {
        Err("no console in tests".to_string())
    }
    fn verify_token(&self, token: &str) -> Option<String> {
        if token == TEST_TOKEN {
            return Some("Admin".to_string());
        }
        let tokens = self.knobs.tokens.lock().unwrap();
        tokens.iter().find(|(t, _)| t == token).map(|(_, u)| u.clone())
    }
    fn revisions(&self) -> tokio::sync::watch::Receiver<crate::Revisions> {
        self.knobs.revisions.subscribe()
    }
    fn logout(&self, _token: &str, _from: &str) {}
    fn device_lock_refusal(&self, device: &str) -> Result<Option<String>, String> {
        Ok((device == LOCKED_DEVICE)
            .then(|| "This device is locked (DeskLock): unlock it first. [E-LOCK-001]".to_string()))
    }
    fn station(&self) -> serde_json::Value {
        self.knobs.station.lock().unwrap().clone()
    }
    fn screen_configs(&self, _user: &str) -> serde_json::Value {
        serde_json::Value::Null
    }
    fn sysmon_history(&self) -> Vec<String> {
        self.diagnostics.load().sysmon_history.clone()
    }
    fn ui_keys(&self, _user: &str) -> serde_json::Value {
        serde_json::Value::Null
    }
    fn feedback_active(&self, _user: &str) -> Vec<String> {
        Vec::new()
    }
    fn render_stats(&self) -> Option<Arc<lumacore_tree::runtime::RenderStats>> {
        Some(self.render.clone())
    }
    fn feedback_settings(&self, _user: &str) -> serde_json::Value {
        serde_json::Value::Null
    }
    fn screen_layout(
        &self,
        _user: &str,
        config: Option<u32>,
        _screen: u32,
    ) -> (u32, Option<crate::ScreenLayout>) {
        (config.unwrap_or(1), None)
    }
}

/// An `HttpState` for router tests: the given diagnostics, sysmon channel and
/// web root, an empty show and screen store, and a [`NullConsole`].
pub(crate) fn test_http_state(
    diagnostics: Arc<ArcSwap<DiagnosticsSnapshot>>,
    sysmon_tx: tokio::sync::broadcast::Sender<String>,
    web_root: Option<std::path::PathBuf>,
) -> HttpState {
    test_http_state_with(diagnostics, sysmon_tx, web_root, Knobs::default())
}

/// [`test_http_state`] with the test's [`Knobs`].
pub(crate) fn test_http_state_with(
    diagnostics: Arc<ArcSwap<DiagnosticsSnapshot>>,
    sysmon_tx: tokio::sync::broadcast::Sender<String>,
    web_root: Option<std::path::PathBuf>,
    knobs: Knobs,
) -> HttpState {
    // A test engine never stops: keep the sender alive.
    let (stop_tx, stopping) = tokio::sync::watch::channel(false);
    std::mem::forget(stop_tx);
    HttpState {
        stopping,
        console: Arc::new(NullConsole {
            diagnostics: diagnostics.clone(),
            render: Arc::default(),
            knobs,
            hold: None,
        }),
        diagnostics,
        sysmon_tx,
        web_root,
        shell_port: 0,
        show: Arc::new(ArcSwap::from_pointee(lumacore_tree::ShowData::default())),
        screen_events: Arc::default(),
        hub: Arc::default(),
        live_screens: Arc::new(RwLock::new(Default::default())),
    }
}

/// [`test_http_state`] whose console's `execute` blocks until the returned
/// [`Hold`] is released.
pub(crate) fn held_http_state() -> (HttpState, Arc<Hold>) {
    let (tx, _) = tokio::sync::broadcast::channel(16);
    let diagnostics = Arc::new(ArcSwap::from_pointee(DiagnosticsSnapshot::default()));
    let mut state = test_http_state(diagnostics.clone(), tx, None);
    let hold = Arc::new(Hold::default());
    state.console = Arc::new(NullConsole {
        diagnostics,
        render: Arc::default(),
        knobs: Knobs::default(),
        hold: Some(hold.clone()),
    });
    (state, hold)
}
