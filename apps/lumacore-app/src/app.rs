//! The thin client (decisions §30 "12a3": "desktop apps/mobile apps are just
//! basic clients with server discovery and connection that gets lockscreens
//! etc that opens the http page"): the connect page (auto-start countdown,
//! server list, local server — bundled, `assets`), sessions (one server, with
//! one OS window per open screen of the screen config this machine uses) and
//! the menu bar. Login and lock happen on the station's own page (its
//! surface); the app only learns the login (`logged_in`) and opens the screens.
//!
//! A window is either the connect page (`lumacore://`) or a screen of a
//! session (the server's UI over HTTP, in an incognito web view so every window
//! keeps its own login). Pages talk to the app over IPC; the app answers
//! through `home.*` (connect page) or `lumacoreApp.*` / `LumaHost` (server UI).

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use serde_json::{Value, json};
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};
use tao::window::{Window, WindowBuilder, WindowId};
use wry::{WebView, WebViewBuilder};

use crate::discovery::{self, Server};
use crate::ipc::{Ipc, OverviewAct};
use crate::prefs::{self, LastConnection, Prefs, ScreenPrefs};
use crate::{assets, http, local_server};

const COUNTDOWN_SECS: u32 = 5;

// ── Model ────────────────────────────────────────────────────────────────────

/// A server the app talks to.
#[derive(Debug, Clone)]
struct Target {
    host: String,
    port: u16,
    id: String,
    name: String,
    /// The server on this machine.
    local: bool,
}

impl Target {
    fn new(host: &str, port: u16) -> Self {
        let local = matches!(host, "127.0.0.1" | "localhost" | "::1");
        Self {
            host: host.to_string(),
            port,
            id: String::new(),
            name: String::new(),
            local,
        }
    }

    fn local() -> Self {
        Self::new(local_server::HOST, local_server::PORT)
    }

    /// Prefs key: the stable station id, else the address.
    fn key(&self) -> String {
        if self.id.is_empty() {
            format!("{}:{}", self.host, self.port)
        } else {
            self.id.clone()
        }
    }

    fn base_url(&self) -> String {
        if self.host.contains(':') {
            format!("http://[{}]:{}", self.host, self.port)
        } else {
            format!("http://{}:{}", self.host, self.port)
        }
    }

    fn label(&self) -> &str {
        if self.name.is_empty() {
            &self.host
        } else {
            &self.name
        }
    }
}

#[derive(Debug, Clone)]
struct AutoLogin {
    user: String,
}

/// Auto-start: count down, then connect to the last server (or start the
/// local one) and log in as the last user.
#[derive(Debug, Clone)]
struct Plan {
    target: Target,
    user: String,
}

struct Session {
    target: Target,
    user: String,
    token: String,
    /// The screen config this machine shows.
    config: u32,
    /// `{default, configs: [{no, name, screens}]}` from `/api/me`.
    configs: Value,
    /// Screen number → its window.
    windows: BTreeMap<u32, WindowId>,
    /// The screen that shows the full lock surface (a tap on a blurred one
    /// moves it there); None = the lowest open screen.
    primary: Option<u32>,
    /// The user the station page logs in by itself (auto-start: the
    /// countdown already ran on the connect page).
    autologin: Option<String>,
}

impl Session {
    /// The screens of the shown config (screen 1 when it has none saved).
    fn screens(&self) -> Vec<u32> {
        let screens: Vec<u32> = self.configs["configs"]
            .as_array()
            .and_then(|cs| {
                cs.iter()
                    .find(|c| c["no"].as_u64() == Some(self.config as u64))
            })
            .and_then(|c| c["screens"].as_array())
            .map(|s| {
                s.iter()
                    .filter_map(|n| n.as_u64().map(|n| n as u32))
                    .collect()
            })
            .unwrap_or_default();
        if screens.is_empty() { vec![1] } else { screens }
    }
}

enum Role {
    Home,
    Screen { session: u64, screen: u32 },
    /// The session's Screens overview (decisions §30 "12a3").
    Overview { session: u64 },
}

struct AppWindow {
    window: Window,
    webview: Option<WebView>,
    role: Role,
    /// Home: the page has loaded (scripts before that are queued).
    ready: bool,
    queued: Vec<String>,
    /// Home: id of the request in flight; a newer one (or Cancel) supersedes it.
    req: u64,
    /// Home: the server on the users view and its `/api/station` info.
    station: Option<(Target, Value)>,
    /// Home: auto-start plan, waiting for the page.
    plan: Option<Plan>,
}

enum UserEvent {
    Ipc(WindowId, String),
    Menu(MenuId),
    Servers(Vec<Server>),
    LocalProbe(Option<Value>),
    LocalExited {
        pid: u32,
        code: Option<i32>,
    },
    /// The crashed child server was started again.
    LocalRestarted {
        pid: u32,
        info: Option<Value>,
    },
    LocalStarted {
        win: WindowId,
        req: u64,
        result: Result<(Value, Option<u32>), String>,
        auto: Option<AutoLogin>,
    },
    Station {
        win: WindowId,
        req: u64,
        target: Target,
        result: Result<Value, String>,
        auto: Option<AutoLogin>,
    },
    Configs {
        session: u64,
        result: Result<Value, String>,
    },
    ShowState {
        win: WindowId,
        show: Option<String>,
        dirty: bool,
    },
    CleanAsk {
        win: WindowId,
        running: bool,
        show: Option<String>,
        dirty: bool,
    },
    /// The local server running is another version: ask before using it.
    OtherServer {
        win: WindowId,
        req: u64,
        info: Value,
        other: local_server::OtherVersion,
        auto: Option<AutoLogin>,
        show: Option<String>,
        dirty: bool,
    },
    /// The local server is gone; go on with what the stop was for.
    StoppedThen { win: Option<WindowId>, then: AfterStop },
    /// A Shutdown didn't finish within `STOP_WAIT`: ask Force stop / Keep waiting.
    StopHung {
        win: Option<WindowId>,
        pid: Option<u32>,
        then: AfterStop,
    },
    Failed {
        win: Option<WindowId>,
        message: String,
    },
}

/// What a stop of the local server was for.
#[derive(Debug, Clone)]
enum AfterStop {
    /// Replace another version: start this app's server.
    StartMine(Option<AutoLogin>),
    /// Quit Server and Client.
    Quit,
    /// Only see it go (the screens go back to the server list by themselves).
    Nothing,
}

/// What a window shows after its session ends.
enum Next {
    Servers(Option<String>),
    Users(Target),
    Connect(Target, Option<AutoLogin>),
}

struct App {
    proxy: EventLoopProxy<UserEvent>,
    prefs: Prefs,
    prefs_dirty: bool,
    last_save: Instant,
    windows: HashMap<WindowId, AppWindow>,
    sessions: HashMap<u64, Session>,
    next_id: u64,
    servers: Vec<Server>,
    /// Station info of the server on this machine while it runs.
    local: Option<Value>,
    /// The local server is the app's child (else attached, or none).
    owned_pid: Option<u32>,
    restarted_once: bool,
    /// "Quit Server and Client" is in progress.
    quitting: bool,
    /// Another version of the local server, waiting for Replace / Use / Cancel.
    other: Option<(Value, Option<AutoLogin>)>,
    /// A hung stop, waiting for Force stop / Keep waiting.
    hung: Option<(Option<u32>, AfterStop)>,
    exit: bool,
    focused: Option<WindowId>,
    screens_menu: Submenu,
    /// The user's clipboard (a page's `copy`), opened on the first copy; None
    /// inside when this machine has none.
    clipboard: Option<Option<arboard::Clipboard>>,
}

// ── Entry ────────────────────────────────────────────────────────────────────

pub fn run() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let (menu, screens_menu) = build_menu();
    {
        let proxy = Mutex::new(proxy.clone());
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            if let Ok(p) = proxy.lock() {
                let _ = p.send_event(UserEvent::Menu(e.id));
            }
        }));
    }
    {
        let proxy = proxy.clone();
        discovery::start(move |list| {
            let _ = proxy.send_event(UserEvent::Servers(list));
        });
    }
    {
        // Watch the local server (started by us, another app or the CLI).
        let proxy = proxy.clone();
        std::thread::spawn(move || {
            let mut last: Option<Value> = None;
            loop {
                let now = local_server::probe();
                if now != last {
                    let _ = proxy.send_event(UserEvent::LocalProbe(now.clone()));
                    last = now;
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        });
    }

    let mut app = App {
        proxy,
        prefs: Prefs::load(),
        prefs_dirty: false,
        last_save: Instant::now(),
        windows: HashMap::new(),
        sessions: HashMap::new(),
        next_id: 1,
        servers: Vec::new(),
        local: None,
        owned_pid: None,
        restarted_once: false,
        quitting: false,
        other: None,
        hung: None,
        exit: false,
        focused: None,
        screens_menu,
        clipboard: None,
    };

    event_loop.run(move |event, target, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => {
                crate::platform::init_menu(&menu);
                let plan = app.autostart_plan();
                let win = app.open_home(target);
                if let Some(w) = app.windows.get_mut(&win) {
                    w.plan = plan;
                }
            }
            Event::UserEvent(e) => app.on_user_event(e, target),
            Event::WindowEvent {
                window_id, event, ..
            } => app.on_window_event(window_id, event, target),
            Event::MainEventsCleared => {
                if app.prefs_dirty && app.last_save.elapsed() >= Duration::from_secs(1) {
                    app.prefs.save();
                    app.prefs_dirty = false;
                    app.last_save = Instant::now();
                } else if app.prefs_dirty {
                    *control_flow = ControlFlow::WaitUntil(app.last_save + Duration::from_secs(1));
                }
            }
            _ => {}
        }
        if app.exit {
            app.prefs.save();
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn build_menu() -> (Menu, Submenu) {
    let menu = Menu::new();
    let cmd = |code| Some(Accelerator::new(Some(Modifiers::SUPER), code));
    let app_menu = Submenu::with_items(
        "LumaCore",
        true,
        &[
            &PredefinedMenuItem::about(None, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id("quit_client", "Quit Client", true, cmd(Code::KeyQ)),
            &MenuItem::with_id(
                "quit_all",
                "Quit Server and Client",
                true,
                Some(Accelerator::new(
                    Some(Modifiers::SUPER | Modifiers::ALT),
                    Code::KeyQ,
                )),
            ),
        ],
    )
    .expect("app menu");
    let file = Submenu::with_items(
        "File",
        true,
        &[
            &MenuItem::with_id("new_window", "New Window", true, cmd(Code::KeyN)),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::close_window(None),
        ],
    )
    .expect("file menu");
    // WKWebView needs these for the clipboard shortcuts.
    let edit = Submenu::with_items(
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::select_all(None),
        ],
    )
    .expect("edit menu");
    let screens = Submenu::new("Screens", true);
    let window = Submenu::with_items(
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id("overview", "Screens Overview…", true, None),
            &screens,
        ],
    )
    .expect("window menu");
    menu.append_items(&[&app_menu, &file, &edit, &window])
        .expect("menu bar");
    (menu, screens)
}

// ── Event handling ───────────────────────────────────────────────────────────

impl App {
    fn id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    fn autostart_plan(&self) -> Option<Plan> {
        if !self.prefs.auto_start {
            return None;
        }
        Some(match &self.prefs.last {
            Some(last) => {
                let mut target = if last.local {
                    Target::local()
                } else {
                    Target::new(&last.host, last.port)
                };
                target.id = last.station_id.clone();
                target.name = last.name.clone();
                Plan {
                    target,
                    user: last.user.clone(),
                }
            }
            None => Plan {
                target: Target::local(),
                user: "Admin".into(),
            },
        })
    }

    fn on_user_event(&mut self, event: UserEvent, target: &EventLoopWindowTarget<UserEvent>) {
        match event {
            UserEvent::Ipc(win, msg) => {
                let Some(msg) = Ipc::parse(&msg) else {
                    return;
                };
                match self.windows.get(&win).map(|w| matches!(w.role, Role::Home)) {
                    Some(true) => self.on_home_message(win, msg, target),
                    Some(false) => self.on_screen_message(win, msg, target),
                    None => {}
                }
            }
            UserEvent::Menu(id) => self.on_menu(id.as_ref(), target),
            UserEvent::Servers(list) => {
                self.servers = list;
                let state = json!({ "servers": self.servers });
                self.all_homes(&format!("home.state({state})"));
            }
            UserEvent::LocalProbe(info) => {
                self.local = info;
                let state = json!({ "local": self.local_json() });
                self.all_homes(&format!("home.state({state})"));
            }
            UserEvent::LocalExited { pid, code } => self.on_local_exited(pid, code, target),
            UserEvent::LocalRestarted { pid, info } => {
                self.owned_pid = Some(pid);
                self.local = info;
                let state = json!({ "local": self.local_json() });
                self.all_homes(&format!("home.state({state})"));
            }
            UserEvent::LocalStarted {
                win,
                req,
                result,
                auto,
            } => {
                if !self.current(win, req) {
                    return;
                }
                match result {
                    Ok((info, pid)) => self.attach_local(win, info, pid, auto),
                    Err(e) => self.home_js(
                        win,
                        &format!("home.state({})", json!({ "view": "servers", "error": e })),
                    ),
                }
            }
            UserEvent::Station {
                win,
                req,
                target: t,
                result,
                auto,
            } => {
                if !self.current(win, req) {
                    return;
                }
                match result {
                    Ok(info) => self.on_station(win, t, info, auto),
                    Err(e) => self.home_js(
                        win,
                        &format!("home.state({})", json!({ "view": "servers", "error": e })),
                    ),
                }
            }
            UserEvent::Configs { session, result } => match result {
                Ok(configs) => {
                    if let Some(s) = self.sessions.get_mut(&session) {
                        s.configs = configs;
                    }
                    self.layout_session(session, target);
                }
                Err(e) => log::warn!("screen configs: {e}"),
            },
            UserEvent::ShowState { win, show, dirty } => {
                self.home_js(
                    win,
                    &format!(
                        "home.confirmStop({})",
                        json!({ "show": show, "dirty": dirty })
                    ),
                );
            }
            UserEvent::CleanAsk {
                win,
                running,
                show,
                dirty,
            } => {
                let s = json!({ "running": running, "show": show, "dirty": dirty });
                self.home_js(win, &format!("home.confirmClean({s})"));
            }
            UserEvent::OtherServer {
                win,
                req,
                info,
                other,
                auto,
                show,
                dirty,
            } => {
                if !self.current(win, req) {
                    return;
                }
                let ask = json!({
                    "reason": other.reason, "build": other.build, "pid": other.pid,
                    "usable": other.usable, "show": show, "dirty": dirty,
                });
                self.other = Some((info, auto));
                self.home_js(win, &format!("home.confirmOther({ask})"));
            }
            UserEvent::StoppedThen { win, then } => match then {
                AfterStop::Quit => self.exit = true,
                AfterStop::Nothing => {}
                AfterStop::StartMine(auto) => {
                    if let Some(win) = win.or_else(|| self.any_home()) {
                        self.start_local(win, auto);
                    }
                }
            },
            UserEvent::StopHung { win, pid, then } => {
                let win = match win.filter(|w| self.windows.contains_key(w)).or_else(|| self.any_home()) {
                    Some(w) => w,
                    None => self.open_home(target),
                };
                let ask = json!({ "pid": pid });
                self.hung = Some((pid, then));
                self.home_js(win, &format!("home.confirmHung({ask})"));
            }
            UserEvent::Failed { win, message } => {
                self.quitting = false;
                match win {
                    Some(win) => {
                        self.home_js(win, &format!("home.state({})", json!({ "error": message })))
                    }
                    None => log::warn!("{message}"),
                }
            }
        }
    }

    /// Attach to the local server `info` (`pid`: ours when we started it).
    fn attach_local(
        &mut self,
        win: WindowId,
        info: Value,
        pid: Option<u32>,
        auto: Option<AutoLogin>,
    ) {
        if pid.is_some() {
            self.owned_pid = pid;
        }
        self.local = Some(info.clone());
        let state = json!({ "local": self.local_json() });
        self.all_homes(&format!("home.state({state})"));
        let mut t = Target::local();
        t.id = info["id"].as_str().unwrap_or_default().to_string();
        t.name = info["name"].as_str().unwrap_or_default().to_string();
        self.on_station(win, t, info, auto);
    }

    /// A home window, if one is open.
    fn any_home(&self) -> Option<WindowId> {
        self.windows
            .iter()
            .find(|(_, w)| matches!(w.role, Role::Home))
            .map(|(id, _)| *id)
    }

    /// Stop the local server (`save` as for Shutdown), then `then`; if it
    /// is still there after `STOP_WAIT`, ask Force stop / Keep waiting.
    /// `save: None` = the Shutdown was already sent (by a server UI page).
    fn stop_local_then(
        &self,
        win: Option<WindowId>,
        info: Option<&Value>,
        save: Option<Option<bool>>,
        then: AfterStop,
    ) {
        let proxy = self.proxy.clone();
        let pid = info
            .or(self.local.as_ref())
            .and_then(|l| l["pid"].as_u64())
            .map(|p| p as u32);
        std::thread::spawn(move || {
            let sent = match save {
                Some(save) => local_server::shutdown(save),
                None => Ok(()),
            };
            let event = match sent {
                Err(message) if local_server::probe().is_some() => UserEvent::Failed { win, message },
                _ if local_server::wait_down(local_server::STOP_WAIT) => {
                    UserEvent::StoppedThen { win, then }
                }
                _ => UserEvent::StopHung { win, pid, then },
            };
            let _ = proxy.send_event(event);
        });
    }

    /// Is `req` still the request this home window waits for?
    fn current(&self, win: WindowId, req: u64) -> bool {
        self.windows
            .get(&win)
            .is_some_and(|w| w.req == req && matches!(w.role, Role::Home))
    }

    fn local_json(&self) -> Value {
        json!({ "running": self.local.is_some(), "owned": self.owned_pid.is_some(), "station": self.local })
    }

    fn on_home_message(
        &mut self,
        win: WindowId,
        msg: Ipc,
        target: &EventLoopWindowTarget<UserEvent>,
    ) {
        match msg {
            Ipc::Ready => {
                let last_key = self.prefs.last.as_ref().map(|l| {
                    if l.station_id.is_empty() {
                        format!("{}:{}", l.host, l.port)
                    } else {
                        l.station_id.clone()
                    }
                });
                let state = json!({ "servers": self.servers, "local": self.local_json(), "last_key": last_key,
                                    "auto_start": self.prefs.auto_start, "kiosk": self.prefs.kiosk });
                let Some(w) = self.windows.get_mut(&win) else {
                    return;
                };
                w.ready = true;
                let queued = std::mem::take(&mut w.queued);
                let plan = w.plan.clone();
                if let Some(wv) = &w.webview {
                    let _ = wv.evaluate_script(&format!("home.state({state})"));
                    if let Some(plan) = &plan {
                        let p = json!({ "local": plan.target.local, "name": plan.target.name, "host": plan.target.host,
                                        "user": plan.user, "seconds": COUNTDOWN_SECS });
                        let _ = wv.evaluate_script(&format!("home.countdown({p})"));
                    } else if queued.is_empty() {
                        let _ = wv.evaluate_script("home.state({view: 'servers'})");
                    }
                    for js in queued {
                        let _ = wv.evaluate_script(&js);
                    }
                }
            }
            Ipc::AutostartGo => {
                let Some(plan) = self.windows.get_mut(&win).and_then(|w| w.plan.take()) else {
                    return;
                };
                let auto = Some(AutoLogin {
                    user: plan.user.clone()
                });
                if plan.target.local {
                    self.start_local(win, auto);
                } else {
                    // The server may have a new address: find it by station id.
                    let mut t = plan.target.clone();
                    if let Some(s) = self
                        .servers
                        .iter()
                        .find(|s| !t.id.is_empty() && s.id == t.id)
                    {
                        t.host = s.host.clone();
                        t.port = s.port;
                    }
                    self.open_server(win, t, auto);
                }
            }
            Ipc::AutostartCancel => {
                if let Some(w) = self.windows.get_mut(&win) {
                    w.plan = None;
                }
            }
            Ipc::StartLocal => self.start_local(win, None),
            Ipc::Open { host, port } => {
                let port = port.unwrap_or(local_server::PORT);
                let t = self.target_for(&host.unwrap_or_default(), port);
                if t.local && t.port == local_server::PORT {
                    self.start_local(win, None);
                } else {
                    self.open_server(win, t, None);
                }
            }
            Ipc::Servers | Ipc::Cancel => {
                let id = self.id();
                if let Some(w) = self.windows.get_mut(&win) {
                    w.req = id;
                }
            }
            Ipc::AutoStart { on } => {
                self.prefs.auto_start = on.unwrap_or(true);
                self.prefs_dirty = true;
            }
            Ipc::Kiosk { on } => {
                self.prefs.kiosk = on.unwrap_or(false);
                self.prefs_dirty = true;
            }
            Ipc::StopLocal { save } => {
                self.home_js(win, "home.busy('Shutting down the local server…')");
                self.stop_local_then(Some(win), None, Some(save), AfterStop::Quit);
            }
            // Another version of the local server (decisions §23 "App ↔
            // local server hardening"): replace it, use it or cancel.
            Ipc::OtherReplace { save } => {
                let Some((info, auto)) = self.other.take() else {
                    return;
                };
                self.home_js(win, "home.busy('Shutting down the other local server…')");
                self.stop_local_then(Some(win), Some(&info), Some(save), AfterStop::StartMine(auto));
            }
            Ipc::OtherUse => {
                if let Some((info, auto)) = self.other.take() {
                    self.attach_local(win, info, None, auto);
                }
            }
            Ipc::OtherCancel => {
                self.other = None;
                self.home_js(win, "home.state({view: 'servers'})");
            }
            // A stop that didn't finish: Force stop (on this click only) or wait.
            Ipc::ForceStop => {
                let Some((pid, then)) = self.hung.take() else {
                    return;
                };
                let Some(pid) = pid else {
                    return;
                };
                self.home_js(win, &format!("home.busy('Stopping process {pid}…')"));
                let proxy = self.proxy.clone();
                std::thread::spawn(move || {
                    let _ = proxy.send_event(match local_server::force_stop(pid) {
                        Ok(()) => UserEvent::StoppedThen { win: Some(win), then },
                        Err(message) => UserEvent::Failed { win: Some(win), message },
                    });
                });
            }
            Ipc::KeepWaiting => {
                let Some((pid, then)) = self.hung.take() else {
                    return;
                };
                self.home_js(win, "home.busy('Waiting for the local server to stop…')");
                let info = pid.map(|p| json!({ "pid": p }));
                self.stop_local_then(Some(win), info.as_ref(), None, then);
            }
            Ipc::HungCancel => {
                self.hung = None;
                self.quitting = false;
                self.home_js(win, "home.state({view: 'servers'})");
            }
            Ipc::CancelQuit => self.quitting = false,
            // CleanStart of the local server (a clean start), the way back
            // when nothing else works: ask first (with the show state).
            Ipc::CleanStart => {
                let proxy = self.proxy.clone();
                std::thread::spawn(move || {
                    let running = local_server::probe().is_some();
                    let (show, dirty) = if running {
                        local_server::show_state()
                    } else {
                        (None, false)
                    };
                    let _ = proxy.send_event(UserEvent::CleanAsk {
                        win,
                        running,
                        show,
                        dirty,
                    });
                });
            }
            Ipc::CleanStartGo { save } => {
                // This machine's UI prefs for the local server go too.
                let mut keys = vec![Target::local().key()];
                if let Some(id) = self.local.as_ref().and_then(|l| l["id"].as_str()) {
                    keys.push(id.to_string());
                }
                for k in &keys {
                    self.prefs.servers.remove(k);
                }
                if self.prefs.last.as_ref().is_some_and(|l| l.local) {
                    self.prefs.last = None;
                }
                self.prefs_dirty = true;
                self.start_local_with(
                    win,
                    Some(AutoLogin {
                        user: "Admin".into()
                    }),
                    Some(save),
                );
            }
            _ => {}
        }
        let _ = target;
    }

    fn on_screen_message(
        &mut self,
        win: WindowId,
        msg: Ipc,
        target: &EventLoopWindowTarget<UserEvent>,
    ) {
        if let Some(Role::Overview { session }) = self.windows.get(&win).map(|w| &w.role) {
            let session = *session;
            return self.on_overview_message(win, session, msg, target);
        }
        let Some((session, screen)) = self.windows.get(&win).and_then(|w| match w.role {
            Role::Screen { session, screen } => Some((session, screen)),
            _ => None,
        }) else {
            return;
        };
        // A Disconnect is a logout that goes back to the server list.
        let msg = match msg {
            Ipc::Disconnect { message } => Ipc::LoggedOut { message, disconnect: true },
            other => other,
        };
        match msg {
            // Logout → this server's users; Disconnect (or Cancel on the
            // reconnect overlay) → the server list.
            Ipc::LoggedOut { message, disconnect } => {
                let next = match self.sessions.get(&session) {
                    Some(s) if !disconnect => Next::Users(s.target.clone()),
                    _ => Next::Servers(message),
                };
                self.end_session(session, Some(win), next, target);
            }
            Ipc::UserChanged { user, token } => {
                let (Some(user), Some(token)) = (user, token) else {
                    return;
                };
                let Some(s) = self.sessions.get_mut(&session) else {
                    return;
                };
                s.user = user.clone();
                s.token = token;
                let key = s.target.key();
                s.config = self
                    .prefs
                    .user_ref(&key, &user)
                    .and_then(|u| u.config)
                    .unwrap_or(0);
                if let Some(last) = &mut self.prefs.last {
                    last.user = user;
                    self.prefs_dirty = true;
                }
                self.refresh_configs(session);
            }
            Ipc::ScreenConfig { no } => {
                let Some(no) = no else {
                    return;
                };
                let Some(s) = self.sessions.get_mut(&session) else {
                    return;
                };
                s.config = no;
                let (key, user) = (s.target.key(), s.user.clone());
                self.prefs.user(&key, &user).config = Some(no);
                self.prefs_dirty = true;
                self.refresh_configs(session);
            }
            Ipc::Connect { target: to, user } => {
                let t = self.resolve(&to.unwrap_or_default());
                let auto = user.map(|user| AutoLogin {
                    user,
                });
                self.end_session(session, Some(win), Next::Connect(t, auto), target);
            }
            Ipc::Pref { key, value } => {
                let Some(key) = key else { return };
                let Some(s) = self.sessions.get(&session) else {
                    return;
                };
                let (skey, user, config) = (s.target.key(), s.user.clone(), s.config);
                self.prefs
                    .user(&skey, &user)
                    .screens
                    .entry(prefs::screen_key(config, screen))
                    .or_default()
                    .page
                    .insert(key, value);
                self.prefs_dirty = true;
            }
            // The page confirmed a Shutdown: when quitting, wait for the
            // local server to go, then quit.
            Ipc::Power { command } => {
                let local = self.sessions.get(&session).is_some_and(|s| s.target.local);
                // A Shutdown of the local server: watch that it really goes
                // (then quit, when quitting).
                if local && command.is_some_and(|c| c.starts_with("Shutdown")) {
                    let then = if self.quitting { AfterStop::Quit } else { AfterStop::Nothing };
                    self.stop_local_then(None, None, None, then);
                }
            }
            Ipc::PowerCancel => self.quitting = false,
            // The station page's surface logged in (decisions §30 "12a3").
            Ipc::LoggedIn { user, token } => {
                if let (Some(user), Some(token)) = (user, token) {
                    self.logged_in(session, user, token);
                }
            }
            // Meow+F12: native full screen of this window on its display
            // (macOS: its own Space), never just a maximize.
            Ipc::Fullscreen { on } => {
                if let Some(w) = self.windows.get(&win) {
                    let on = on.unwrap_or(w.window.fullscreen().is_none());
                    w.window.set_fullscreen(on.then_some(tao::window::Fullscreen::Borderless(None)));
                }
            }
            // A tap on a blurred locked screen: the surface moves there.
            Ipc::FocusSurface => {
                if let Some(s) = self.sessions.get_mut(&session) {
                    s.primary = Some(screen);
                }
                self.push_primary(session);
            }
            // Menu › Screens: the session's overview window.
            Ipc::OpenOverview => self.open_overview(session, target),
            // Servers… on the station's surface: back to the connect page.
            Ipc::Servers => self.end_session(session, Some(win), Next::Servers(None), target),
            // A copy for the user's clipboard (the NAV inspector's copy keys): written
            // natively; the page says "Copied" or falls back to its own ways.
            Ipc::Copy { text } => {
                let ok = text.is_some_and(|t| self.copy_to_clipboard(&t));
                self.screen_js(win, &format!("window.lumacoreApp?.copied?.({ok})"));
            }
            _ => {}
        }
    }

    /// Which open screen shows the full lock surface: the chosen one, else
    /// the lowest (decisions §30 "12a3").
    fn primary_screen(&self, session: u64) -> Option<u32> {
        let s = self.sessions.get(&session)?;
        s.primary
            .filter(|p| s.windows.contains_key(p))
            .or_else(|| s.windows.keys().next().copied())
    }

    /// Tell every screen of `session` whether it is the primary one.
    fn push_primary(&self, session: u64) {
        let Some(s) = self.sessions.get(&session) else {
            return;
        };
        let primary = self.primary_screen(session);
        for (screen, win) in &s.windows {
            let js = format!(
                "if (window.LumaHost) {{ LumaHost.isPrimary = {}; window.lumacoreApp?.primaryChanged?.(); }}",
                primary == Some(*screen)
            );
            self.screen_js(*win, &js);
        }
    }

    fn on_window_event(
        &mut self,
        id: WindowId,
        event: WindowEvent,
        target: &EventLoopWindowTarget<UserEvent>,
    ) {
        match event {
            WindowEvent::CloseRequested => self.close_window(id),
            WindowEvent::Focused(true) => {
                self.focused = Some(id);
                self.update_screens_menu();
            }
            WindowEvent::Moved(_) | WindowEvent::Resized(_) => self.save_geometry(id),
            _ => {}
        }
        let _ = target;
    }

    fn on_menu(&mut self, id: &str, target: &EventLoopWindowTarget<UserEvent>) {
        match id {
            "quit_client" => self.exit = true,
            "quit_all" => self.quit_all(target),
            "new_window" => {
                self.open_home(target);
            }
            "overview" => {
                if let Some(session) = self.focused_session() {
                    self.open_overview(session, target);
                }
            }
            _ => {
                if let Some(n) = id
                    .strip_prefix("screen:")
                    .and_then(|n| n.parse::<u32>().ok())
                {
                    self.toggle_screen(n, target);
                }
            }
        }
    }

    // ── Servers, local server, login ─────────────────────────────────────────

    /// `Connect` target: a discovered station by name or id, else an address.
    fn resolve(&self, to: &str) -> Target {
        if let Some(s) = self
            .servers
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(to) || (!s.id.is_empty() && s.id == to))
        {
            let mut t = self.target_for(&s.host, s.port);
            t.id = s.id.clone();
            t.name = s.name.clone();
            return t;
        }
        if let Some(info) = &self.local
            && info["name"]
                .as_str()
                .is_some_and(|n| n.eq_ignore_ascii_case(to))
        {
            return Target::local();
        }
        let (host, port) = match to.rsplit_once(':') {
            Some((h, p)) if !h.contains(':') || h.starts_with('[') => (
                h.trim_matches(['[', ']']).to_string(),
                p.parse().unwrap_or(local_server::PORT),
            ),
            _ => (to.to_string(), local_server::PORT),
        };
        self.target_for(&host, port)
    }

    fn target_for(&self, host: &str, port: u16) -> Target {
        let mut t = Target::new(host, port);
        if let Some(s) = self
            .servers
            .iter()
            .find(|s| s.host == host && s.port == port)
        {
            t.id = s.id.clone();
            t.name = s.name.clone();
        }
        t
    }

    fn start_local(&mut self, win: WindowId, auto: Option<AutoLogin>) {
        self.start_local_with(win, auto, None);
    }

    /// Start or attach to the local server. `clean`: CleanStart it first
    /// (`Some(save)` — a running one over its command port, a new one with
    /// `--clean-start`).
    fn start_local_with(
        &mut self,
        win: WindowId,
        auto: Option<AutoLogin>,
        clean: Option<Option<bool>>,
    ) {
        let req = self.id();
        if let Some(w) = self.windows.get_mut(&win) {
            w.req = req;
        }
        let running = self.local.is_some();
        let busy = match (clean.is_some(), running) {
            (true, _) => "home.busy('Clean start of the local server…')",
            (false, true) => "home.busy('Connecting to the local server…')",
            (false, false) => "home.busy('Starting the local server…')",
        };
        self.home_js(win, busy);
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            let result = match local_server::probe() {
                // Already running (another app instance, the CLI, …): attach,
                // but never silently to another version (decisions §23 "App ↔
                // local server hardening").
                Some(info) => match (clean, local_server::other_version(&info)) {
                    (_, Some(other)) => {
                        let (show, dirty) = local_server::show_state();
                        let _ = proxy.send_event(UserEvent::OtherServer {
                            win,
                            req,
                            info,
                            other,
                            auto,
                            show,
                            dirty,
                        });
                        return;
                    }
                    (Some(save), None) => local_server::clean_start(save).and_then(|_| {
                        local_server::wait_ready(Duration::from_secs(10))
                            .map(|info| (info, None))
                            .ok_or_else(|| {
                                "The local server stopped answering after CleanStart.".to_string()
                            })
                    }),
                    (None, None) => Ok((info, None)),
                },
                None => {
                    let exit_proxy = proxy.clone();
                    let extra: &[&str] = if clean.is_some() {
                        &["--clean-start"]
                    } else {
                        &[]
                    };
                    local_server::spawn_with(extra, move |pid, code| {
                        let _ = exit_proxy.send_event(UserEvent::LocalExited { pid, code });
                    })
                    .and_then(|pid| {
                        local_server::wait_ready(Duration::from_secs(30))
                            .map(|info| (info, Some(pid)))
                            .ok_or_else(|| {
                                "The local server did not start (see its log).".to_string()
                            })
                    })
                }
            };
            let _ = proxy.send_event(UserEvent::LocalStarted {
                win,
                req,
                result,
                auto,
            });
        });
    }

    fn open_server(&mut self, win: WindowId, t: Target, auto: Option<AutoLogin>) {
        let req = self.id();
        if let Some(w) = self.windows.get_mut(&win) {
            w.req = req;
        }
        self.home_js(
            win,
            &format!(
                "home.busy({})",
                json!(format!("Connecting to {}…", t.label()))
            ),
        );
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            let result = match http::get(&t.host, t.port, "/api/station", None) {
                Ok((200, info)) => Ok(info),
                Ok((status, _)) => Err(format!("{}:{} answered {status}", t.host, t.port)),
                Err(e) => Err(format!("Can't reach {}: {e}", t.label())),
            };
            let _ = proxy.send_event(UserEvent::Station {
                win,
                req,
                target: t,
                result,
                auto,
            });
        });
    }

    /// A server's info arrived: open its page in this window, before any
    /// login — the station's surface logs in (decisions §30 "12a3"). An
    /// auto-start passes its user (the countdown ran here already).
    fn on_station(&mut self, win: WindowId, mut t: Target, info: Value, auto: Option<AutoLogin>) {
        t.id = info["id"].as_str().unwrap_or_default().to_string();
        t.name = info["name"].as_str().unwrap_or_default().to_string();
        if let Some(local) = &self.local
            && !t.id.is_empty()
            && local["id"].as_str() == Some(t.id.as_str())
        {
            t.local = true;
        }
        let id = self.id();
        self.sessions.insert(
            id,
            Session {
                target: t,
                user: String::new(),
                token: String::new(),
                config: 0,
                configs: Value::Null,
                windows: BTreeMap::new(),
                primary: None,
                autologin: auto.map(|a| a.user),
            },
        );
        if let Some(w) = self.windows.get_mut(&win) {
            w.station = None;
            w.plan = None;
        }
        self.show_screen(win, id, 1);
        self.update_screens_menu();
    }

    // ── Sessions and screens ─────────────────────────────────────────────────

    /// The station page logged in (its surface): remember the login, read
    /// the user's screen configs and open their screens with the token.
    fn logged_in(&mut self, session: u64, user: String, token: String) {
        let Some(s) = self.sessions.get_mut(&session) else {
            return;
        };
        s.user = user.clone();
        s.token = token;
        s.autologin = None;
        let key = s.target.key();
        s.config = self.prefs.user_ref(&key, &user).and_then(|u| u.config).unwrap_or(0);
        self.prefs.last = Some(LastConnection {
            local: s.target.local,
            station_id: s.target.id.clone(),
            name: s.target.name.clone(),
            host: s.target.host.clone(),
            port: s.target.port,
            user,
        });
        self.prefs_dirty = true;
        self.refresh_configs(session);
    }

    /// Re-read the session's screen configs (after Login / ScreenConfig),
    /// then lay its windows out again.
    fn refresh_configs(&mut self, session: u64) {
        let Some(s) = self.sessions.get(&session) else {
            return;
        };
        let (t, token) = (s.target.clone(), s.token.clone());
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            let result = match http::get(&t.host, t.port, "/api/me", Some(&token)) {
                Ok((200, me)) => Ok(me["screen_configs"].clone()),
                Ok((status, _)) => Err(format!("/api/me answered {status}")),
                Err(e) => Err(e),
            };
            let _ = proxy.send_event(UserEvent::Configs { session, result });
        });
    }

    /// One window per open screen of the session's config: reuse the
    /// session's windows in order, close extra ones, open missing ones.
    fn layout_session(&mut self, session: u64, target: &EventLoopWindowTarget<UserEvent>) {
        let Some(s) = self.sessions.get_mut(&session) else {
            return;
        };
        if s.config == 0 {
            s.config = s.configs["default"].as_u64().unwrap_or(1) as u32;
        }
        let key = s.target.key();
        let user = s.user.clone();
        let config = s.config;
        let all = s.screens();
        let mut wanted: Vec<u32> = all
            .iter()
            .copied()
            .filter(|n| {
                self.prefs
                    .user_ref(&key, &user)
                    .and_then(|u| u.screens.get(&prefs::screen_key(config, *n)))
                    .is_none_or(|p| p.open)
            })
            .collect();
        if wanted.is_empty() {
            wanted.push(all[0]);
        }
        let existing: Vec<WindowId> = std::mem::take(&mut s.windows).into_values().collect();

        let mut reuse = existing.into_iter();
        for screen in wanted {
            let win = match reuse.next() {
                Some(win) => win,
                None => self.new_window(target, "LumaCore"),
            };
            self.show_screen(win, session, screen);
        }
        for extra in reuse {
            self.windows.remove(&extra);
        }
        self.update_screens_menu();
        self.push_primary(session);
    }

    /// Load screen `screen` of `session` into window `win`.
    fn show_screen(&mut self, win: WindowId, session: u64, screen: u32) {
        let Some(s) = self.sessions.get_mut(&session) else {
            return;
        };
        s.windows.insert(screen, win);
        let skey = prefs::screen_key(s.config, screen);
        let saved: ScreenPrefs = self
            .prefs
            .user_ref(&s.target.key(), &s.user)
            .and_then(|u| u.screens.get(&skey))
            .cloned()
            .unwrap_or_default();
        let ctx = json!({
            "local": s.target.local,
            "station": s.target.name,
            "host": s.target.host,
            "user": s.user,
            "token": s.token,
            "config": s.config,
            "screen": screen,
            "page": saved.page,
            "device": self.prefs.device,
            "autologin": s.autologin,
        });
        // The app's side of the page (decisions §30 "12a3"): the device all
        // its windows share, Kiosk, whether this window shows the full lock
        // surface, native full screen, switch user, back to the server list.
        let primary = s.primary.or_else(|| s.windows.keys().next().copied()).is_none_or(|p| p == screen);
        let host = json!({ "device": self.prefs.device, "kiosk": self.prefs.kiosk, "isPrimary": primary });
        let host_js = format!(
            "window.LumaHost = Object.assign({host}, {{ \
               post(m) {{ window.ipc.postMessage(JSON.stringify(m)); }}, \
               fullscreen(on) {{ this.post({{ type: 'fullscreen', on: on ?? null }}); }}, \
               switchUser(user, token) {{ this.post({{ type: 'user_changed', user, token }}); }}, \
               loggedIn(user, token) {{ this.post({{ type: 'logged_in', user, token }}); }}, \
               focusSurface() {{ this.post({{ type: 'focus_surface' }}); }}, \
               showServers() {{ this.post({{ type: 'servers' }}); }}, \
               openOverview() {{ this.post({{ type: 'open_overview' }}); }}, \
               copy(text) {{ this.post({{ type: 'copy', text }}); }} }});"
        );
        let base = s.target.base_url();
        let url = format!("{base}/?screen={screen}");
        let title = if s.user.is_empty() {
            format!("LumaCore — {}", s.target.label())
        } else {
            format!("LumaCore — {} · {} · Screen {screen}", s.target.label(), s.user)
        };
        let Some(w) = self.windows.get_mut(&win) else {
            return;
        };
        w.role = Role::Screen { session, screen };
        w.ready = false;
        w.queued.clear();
        w.window.set_title(&title);
        apply_geometry(&w.window, &saved);
        w.webview = None; // drop the old page before the new one takes the window
        let proxy = self.proxy.clone();
        let built = WebViewBuilder::new()
            .with_incognito(true)
            .with_devtools(cfg!(debug_assertions))
            .with_initialization_script(&format!("window.LUMACORE_APP = {ctx};{host_js}"))
            .with_url(url)
            .with_navigation_handler(stay_on(base))
            .with_new_window_req_handler(no_new_windows)
            .with_ipc_handler(move |req| {
                let _ = proxy.send_event(UserEvent::Ipc(win, req.body().clone()));
            })
            .build(&w.window);
        match built {
            Ok(wv) => w.webview = Some(wv),
            Err(e) => log::error!("screen web view: {e}"),
        }
    }

    /// The session's Screens overview window (decisions §30 "12a3", the
    /// Overview board): the station page in overview mode, with the app's
    /// displays and window places.
    fn open_overview(&mut self, session: u64, target: &EventLoopWindowTarget<UserEvent>) {
        if let Some((id, _)) = self
            .windows
            .iter()
            .find(|(_, w)| matches!(w.role, Role::Overview { session: o } if o == session))
        {
            let id = *id;
            if let Some(w) = self.windows.get(&id) {
                w.window.set_focus();
            }
            return;
        }
        let Some(s) = self.sessions.get(&session) else {
            return;
        };
        let ctx = json!({ "local": s.target.local, "station": s.target.name, "host": s.target.host,
                          "user": s.user, "token": s.token, "config": s.config, "screen": 0, "page": {},
                          "device": self.prefs.device });
        let host_js = format!(
            "window.LumaHost = Object.assign({}, {{ \
               post(m) {{ window.ipc.postMessage(JSON.stringify(m)); }}, \
               overview() {{ this.post({{ type: 'overview_data' }}); }}, \
               overviewAction(act, no, display) {{ this.post({{ type: 'overview_action', act, no, display: display ?? null }}); }} }});",
            json!({ "device": self.prefs.device, "kiosk": self.prefs.kiosk, "isPrimary": false })
        );
        let base = s.target.base_url();
        let url = format!("{base}/?overview");
        let title = format!("LumaCore — {} · Screens", s.target.label());
        let win = self.new_window(target, &title);
        let proxy = self.proxy.clone();
        let Some(w) = self.windows.get_mut(&win) else {
            return;
        };
        w.role = Role::Overview { session };
        let built = WebViewBuilder::new()
            .with_incognito(true)
            .with_devtools(cfg!(debug_assertions))
            .with_initialization_script(&format!("window.LUMACORE_APP = {ctx};{host_js}"))
            .with_url(url)
            .with_navigation_handler(stay_on(base))
            .with_new_window_req_handler(no_new_windows)
            .with_ipc_handler(move |req| {
                let _ = proxy.send_event(UserEvent::Ipc(win, req.body().clone()));
            })
            .build(&w.window);
        match built {
            Ok(wv) => w.webview = Some(wv),
            Err(e) => log::error!("overview web view: {e}"),
        }
    }

    /// What the overview shows: this computer's displays and every screen of
    /// the session's config with its window's place.
    fn overview_data(&self, session: u64) -> Value {
        let Some(s) = self.sessions.get(&session) else {
            return json!({});
        };
        let any = s.windows.values().next().and_then(|id| self.windows.get(id));
        let monitors: Vec<tao::monitor::MonitorHandle> = self
            .windows
            .values()
            .next()
            .map(|w| w.window.available_monitors().collect())
            .unwrap_or_default();
        let main = any.and_then(|w| w.window.primary_monitor()).and_then(|m| m.name());
        let displays: Vec<Value> = monitors
            .iter()
            .map(|m| {
                let scale = m.scale_factor();
                let p = m.position().to_logical::<i32>(scale);
                let z = m.size().to_logical::<i32>(scale);
                let name = m.name().unwrap_or_else(|| "Display".into());
                json!({ "name": name, "x": p.x, "y": p.y, "w": z.width, "h": z.height, "main": m.name() == main })
            })
            .collect();
        let screens: Vec<Value> = s
            .screens()
            .into_iter()
            .map(|no| match s.windows.get(&no).and_then(|id| self.windows.get(id)) {
                Some(w) => {
                    let m = w.window.current_monitor();
                    let origin = m.as_ref().map(|m| m.position().to_logical::<i32>(m.scale_factor()));
                    let scale = w.window.scale_factor();
                    let p = w.window.outer_position().ok().map(|p| p.to_logical::<i32>(scale));
                    let z = w.window.inner_size().to_logical::<i32>(scale);
                    let (x, y) = match (p, origin) {
                        (Some(p), Some(o)) => (p.x - o.x, p.y - o.y),
                        (Some(p), None) => (p.x, p.y),
                        _ => (0, 0),
                    };
                    json!({ "no": no, "open": true, "display": m.and_then(|m| m.name()), "x": x, "y": y,
                            "w": z.width, "h": z.height, "fullscreen": w.window.fullscreen().is_some() })
                }
                None => json!({ "no": no, "open": false }),
            })
            .collect();
        json!({ "displays": displays, "screens": screens })
    }

    fn on_overview_message(
        &mut self,
        win: WindowId,
        session: u64,
        msg: Ipc,
        target: &EventLoopWindowTarget<UserEvent>,
    ) {
        match msg {
            Ipc::OverviewData => {}
            Ipc::OverviewAction { act, no, display } => {
                let no = no.unwrap_or(0);
                let screen_win = self.sessions.get(&session).and_then(|s| s.windows.get(&no).copied());
                match act {
                    Some(OverviewAct::Bring) => {
                        if let Some(w) = screen_win.and_then(|id| self.windows.get(&id)) {
                            w.window.set_minimized(false);
                            w.window.set_focus();
                        }
                    }
                    Some(act @ (OverviewAct::Toggle | OverviewAct::Popout)) => {
                        if act == OverviewAct::Toggle || screen_win.is_none() {
                            self.focused = screen_win.or(Some(win));
                            self.toggle_screen_of(session, no, target);
                        }
                    }
                    Some(OverviewAct::Send) => {
                        let name = display.unwrap_or_default();
                        if let Some(w) = screen_win.and_then(|id| self.windows.get(&id))
                            && let Some(m) = w.window.available_monitors().find(|m| m.name().as_deref() == Some(&name)) {
                                let scale = m.scale_factor();
                                let o = m.position().to_logical::<i32>(scale);
                                let a = m.size().to_logical::<i32>(scale);
                                let z = w.window.inner_size().to_logical::<i32>(w.window.scale_factor());
                                let (ww, hh) = (z.width.min(a.width), z.height.min(a.height));
                                w.window.set_inner_size(LogicalSize::new(ww as u32, hh as u32));
                                w.window.set_outer_position(LogicalPosition::new(o.x + (a.width - ww) / 2, o.y + (a.height - hh) / 2));
                            }
                        if let Some(id) = screen_win {
                            self.save_geometry(id);
                        }
                    }
                    _ => {}
                }
            }
            _ => return,
        }
        let data = self.overview_data(session);
        self.screen_js(win, &format!("window.lumacoreApp?.overview?.({data})"));
    }

    /// End a session: its windows close, except `keep` (else the first),
    /// which becomes a home window showing `next`.
    fn end_session(
        &mut self,
        session: u64,
        keep: Option<WindowId>,
        next: Next,
        target: &EventLoopWindowTarget<UserEvent>,
    ) {
        let Some(s) = self.sessions.remove(&session) else {
            return;
        };
        let keep = keep.or_else(|| s.windows.values().next().copied());
        for win in s.windows.values() {
            if Some(*win) != keep {
                self.windows.remove(win);
            }
        }
        self.windows.retain(|id, w| Some(*id) == keep || !matches!(w.role, Role::Overview { session: o } if o == session));
        if let Some(win) = keep {
            self.make_home(win);
            match next {
                Next::Servers(message) => {
                    let state = json!({ "view": "servers", "message": message });
                    self.home_js(win, &format!("home.state({state})"));
                }
                Next::Users(t) => self.open_server(win, t, None),
                Next::Connect(t, auto) => self.open_server(win, t, auto),
            }
        }
        if self.windows.is_empty() {
            self.open_home(target);
        }
        self.update_screens_menu();
    }

    fn toggle_screen(&mut self, screen: u32, target: &EventLoopWindowTarget<UserEvent>) {
        if let Some(session) = self.focused_session() {
            self.toggle_screen_of(session, screen, target);
        }
    }

    fn toggle_screen_of(&mut self, session: u64, screen: u32, target: &EventLoopWindowTarget<UserEvent>) {
        let Some(s) = self.sessions.get(&session) else {
            return;
        };
        let (key, user, config) = (s.target.key(), s.user.clone(), s.config);
        match s.windows.get(&screen).copied() {
            Some(win) => {
                if s.windows.len() == 1 {
                    return; // the last screen stays; Logout / Disconnect ends the session
                }
                self.save_geometry(win);
                self.windows.remove(&win);
                if let Some(s) = self.sessions.get_mut(&session) {
                    s.windows.remove(&screen);
                }
                self.prefs
                    .user(&key, &user)
                    .screens
                    .entry(prefs::screen_key(config, screen))
                    .or_default()
                    .open = false;
            }
            None => {
                self.prefs
                    .user(&key, &user)
                    .screens
                    .entry(prefs::screen_key(config, screen))
                    .or_default()
                    .open = true;
                let win = self.new_window(target, "LumaCore");
                self.show_screen(win, session, screen);
            }
        }
        self.prefs_dirty = true;
        self.update_screens_menu();
        self.push_primary(session);
    }

    fn focused_session(&self) -> Option<u64> {
        let focused = self.focused.and_then(|f| self.windows.get(&f))?;
        match focused.role {
            Role::Screen { session, .. } | Role::Overview { session } => Some(session),
            Role::Home => None,
        }
    }

    /// Window › Screens: the screens of the focused session's config.
    fn update_screens_menu(&self) {
        while self.screens_menu.remove_at(0).is_some() {}
        let Some(s) = self.focused_session().and_then(|id| self.sessions.get(&id)) else {
            let _ =
                self.screens_menu
                    .append(&MenuItem::new("No session in this window", false, None));
            return;
        };
        for screen in s.screens() {
            let open = s.windows.contains_key(&screen);
            let item = CheckMenuItem::with_id(
                format!("screen:{screen}"),
                format!("Screen {screen}"),
                true,
                open,
                None,
            );
            let _ = self.screens_menu.append(&item);
        }
    }

    // ── Windows ──────────────────────────────────────────────────────────────

    fn new_window(&mut self, target: &EventLoopWindowTarget<UserEvent>, title: &str) -> WindowId {
        // The app icon (images/brand/prompt-icon.svg) as 64 × 64 raw RGBA, made by
        // images/brand/make-icon-rgba.py.
        const ICON: &[u8] = include_bytes!("../assets/icon-64.rgba");
        let window = WindowBuilder::new()
            .with_title(title)
            .with_window_icon(tao::window::Icon::from_rgba(ICON.to_vec(), 64, 64).ok())
            .with_inner_size(LogicalSize::new(1280u32, 800u32))
            .with_min_inner_size(LogicalSize::new(640u32, 400u32))
            .build(target)
            .expect("failed to create window");
        let id = window.id();
        self.windows.insert(
            id,
            AppWindow {
                window,
                webview: None,
                role: Role::Home,
                ready: false,
                queued: Vec::new(),
                req: 0,
                station: None,
                plan: None,
            },
        );
        id
    }

    fn open_home(&mut self, target: &EventLoopWindowTarget<UserEvent>) -> WindowId {
        let win = self.new_window(target, "LumaCore");
        self.make_home(win);
        win
    }

    /// Show the home page in `win`.
    fn make_home(&mut self, win: WindowId) {
        let proxy = self.proxy.clone();
        let Some(w) = self.windows.get_mut(&win) else {
            return;
        };
        w.role = Role::Home;
        w.ready = false;
        w.queued.clear();
        w.station = None;
        w.window.set_title("LumaCore");
        w.webview = None;
        let built = WebViewBuilder::new()
            .with_devtools(cfg!(debug_assertions))
            .with_custom_protocol("lumacore".into(), |_id, req| {
                match assets::get(req.uri().path()) {
                    Some((bytes, mime)) => wry::http::Response::builder()
                        .header(wry::http::header::CONTENT_TYPE, mime)
                        .body(Cow::Borrowed(bytes))
                        .expect("asset response"),
                    None => wry::http::Response::builder()
                        .status(404)
                        .body(Cow::Borrowed(&b""[..]))
                        .expect("404 response"),
                }
            })
            .with_url("lumacore://localhost/")
            .with_navigation_handler(stay_on("lumacore://localhost".to_string()))
            .with_new_window_req_handler(no_new_windows)
            .with_ipc_handler(move |req| {
                let _ = proxy.send_event(UserEvent::Ipc(win, req.body().clone()));
            })
            .build(&w.window);
        match built {
            Ok(wv) => w.webview = Some(wv),
            Err(e) => log::error!("home web view: {e}"),
        }
    }

    /// Run `js` in a home window (queued until the page is ready).
    fn home_js(&mut self, win: WindowId, js: &str) {
        let Some(w) = self.windows.get_mut(&win) else {
            return;
        };
        if !matches!(w.role, Role::Home) {
            return;
        }
        match (&w.webview, w.ready) {
            (Some(wv), true) => {
                let _ = wv.evaluate_script(js);
            }
            _ => w.queued.push(js.to_string()),
        }
    }

    fn all_homes(&self, js: &str) {
        for w in self.windows.values() {
            if let (Role::Home, true, Some(wv)) = (&w.role, w.ready, &w.webview) {
                let _ = wv.evaluate_script(js);
            }
        }
    }

    /// Put `text` on the OS clipboard (the user's, never the station's object
    /// clipboard). False when this machine has none or it refused.
    fn copy_to_clipboard(&mut self, text: &str) -> bool {
        let clipboard = self.clipboard.get_or_insert_with(|| arboard::Clipboard::new().ok());
        match clipboard.as_mut().map(|c| c.set_text(text)) {
            Some(Ok(())) => true,
            Some(Err(e)) => {
                log::warn!("copy: the clipboard refused ({e})");
                false
            }
            None => false,
        }
    }

    fn screen_js(&self, win: WindowId, js: &str) {
        if let Some(wv) = self.windows.get(&win).and_then(|w| w.webview.as_ref()) {
            let _ = wv.evaluate_script(js);
        }
    }

    /// Red close button: a screen hides (remembered here), the last window of
    /// a session ends it, the last window of the app quits the client (the
    /// local server keeps running).
    fn close_window(&mut self, id: WindowId) {
        let role = self.windows.get(&id).map(|w| match w.role {
            Role::Screen { session, screen } => Some((session, screen)),
            Role::Home | Role::Overview { .. } => None,
        });
        match role {
            Some(Some((session, screen))) => {
                let count = self.sessions.get(&session).map_or(0, |s| s.windows.len());
                if count > 1 {
                    self.save_geometry(id);
                    if let Some(s) = self.sessions.get_mut(&session) {
                        s.windows.remove(&screen);
                        let (key, user, config) = (s.target.key(), s.user.clone(), s.config);
                        self.prefs
                            .user(&key, &user)
                            .screens
                            .entry(prefs::screen_key(config, screen))
                            .or_default()
                            .open = false;
                        self.prefs_dirty = true;
                    }
                } else {
                    self.sessions.remove(&session);
                }
                self.windows.remove(&id);
            }
            Some(None) => {
                self.windows.remove(&id);
            }
            None => return,
        }
        if self.windows.is_empty() {
            self.exit = true;
        }
        self.update_screens_menu();
    }

    fn save_geometry(&mut self, id: WindowId) {
        let Some(w) = self.windows.get(&id) else {
            return;
        };
        let Role::Screen { session, screen } = w.role else {
            return;
        };
        let Some(s) = self.sessions.get(&session) else {
            return;
        };
        if screen == 0 {
            return;
        }
        let scale = w.window.scale_factor();
        let maximized = w.window.is_maximized();
        // Where on which display (decisions §30 "12a3"): x / y relative to it.
        let monitor = w.window.current_monitor();
        let origin = monitor
            .as_ref()
            .map(|m| m.position().to_logical::<i32>(m.scale_factor()));
        let display = monitor.as_ref().and_then(|m| m.name());
        let pos = w
            .window
            .outer_position()
            .ok()
            .map(|p| p.to_logical::<i32>(scale))
            .map(|p| match origin {
                Some(o) => LogicalPosition::new(p.x - o.x, p.y - o.y),
                None => p,
            });
        let size = w.window.inner_size().to_logical::<u32>(scale);
        if w.window.fullscreen().is_some() {
            return;
        }
        let (key, user, config) = (s.target.key(), s.user.clone(), s.config);
        let sp = self
            .prefs
            .user(&key, &user)
            .screens
            .entry(prefs::screen_key(config, screen))
            .or_default();
        sp.maximized = maximized;
        sp.display = display;
        if !maximized {
            if let Some(p) = pos {
                sp.x = Some(p.x);
                sp.y = Some(p.y);
            }
            sp.w = Some(size.width);
            sp.h = Some(size.height);
        }
        self.prefs_dirty = true;
    }

    // ── Quit and the local server ────────────────────────────────────────────

    /// Quit Server and Client: the server's own Shutdown confirmation in a
    /// window logged in to it, else the app's (Save / Don't Save) on a home
    /// window. Without a local server it just quits.
    fn quit_all(&mut self, target: &EventLoopWindowTarget<UserEvent>) {
        if self.local.is_none() {
            self.exit = true;
            return;
        }
        self.quitting = true;
        let focused_session = self.focused_session();
        let on_local: Option<WindowId> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.target.local)
            .max_by_key(|(id, _)| Some(**id) == focused_session)
            .and_then(|(_, s)| s.windows.values().next().copied());
        if let Some(win) = on_local {
            if let Some(w) = self.windows.get(&win) {
                w.window.set_focus();
            }
            self.screen_js(win, "window.lumacoreApp && lumacoreApp.quitServer()");
            return;
        }
        let home = self
            .focused
            .filter(|f| {
                self.windows
                    .get(f)
                    .is_some_and(|w| matches!(w.role, Role::Home))
            })
            .or_else(|| {
                self.windows
                    .iter()
                    .find(|(_, w)| matches!(w.role, Role::Home))
                    .map(|(id, _)| *id)
            })
            .unwrap_or_else(|| self.open_home(target));
        if let Some(w) = self.windows.get(&home) {
            w.window.set_focus();
        }
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            let (show, dirty) = local_server::show_state();
            let _ = proxy.send_event(UserEvent::ShowState {
                win: home,
                show,
                dirty,
            });
        });
    }

    fn on_local_exited(
        &mut self,
        pid: u32,
        code: Option<i32>,
        target: &EventLoopWindowTarget<UserEvent>,
    ) {
        if self.owned_pid != Some(pid) {
            return;
        }
        self.owned_pid = None;
        self.local = None;
        if self.quitting {
            self.exit = true;
            return;
        }
        let local_sessions: Vec<u64> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.target.local)
            .map(|(id, _)| *id)
            .collect();
        if code == Some(0) {
            // Shutdown: back to the server list.
            for id in local_sessions {
                self.end_session(
                    id,
                    None,
                    Next::Servers(Some("The local server was shut down.".into())),
                    target,
                );
            }
            return;
        }
        // Crashed: restart it once; the screens reconnect by themselves.
        let notice = if self.restarted_once {
            format!(
                "The local server stopped (exit code {}).",
                code.map_or("signal".to_string(), |c| c.to_string())
            )
        } else {
            self.restarted_once = true;
            let proxy = self.proxy.clone();
            std::thread::spawn(move || {
                let exit_proxy = proxy.clone();
                match local_server::spawn(move |pid, code| {
                    let _ = exit_proxy.send_event(UserEvent::LocalExited { pid, code });
                }) {
                    Ok(pid) => {
                        let info = local_server::wait_ready(Duration::from_secs(30));
                        let _ = proxy.send_event(UserEvent::LocalRestarted { pid, info });
                    }
                    Err(e) => log::error!("{e}"),
                }
            });
            "The local server stopped unexpectedly — restarting it…".to_string()
        };
        for id in self
            .sessions
            .iter()
            .filter(|(_, s)| s.target.local)
            .flat_map(|(_, s)| s.windows.values())
        {
            self.screen_js(
                *id,
                &format!(
                    "window.lumacoreApp && lumacoreApp.notice({})",
                    json!(notice)
                ),
            );
        }
    }
}

/// Put a window where this screen was (decisions §30 "12a3"): on its display
/// when that display is there (x / y relative to it), else on the main
/// display, scaled to fit — it moves back when the display returns (the next
/// time the screen opens).
fn apply_geometry(window: &Window, p: &ScreenPrefs) {
    let monitors: Vec<_> = window.available_monitors().collect();
    let home = p
        .display
        .as_ref()
        .and_then(|name| monitors.iter().find(|m| m.name().as_deref() == Some(name.as_str())));
    let target = home.cloned().or_else(|| window.primary_monitor()).or_else(|| monitors.first().cloned());
    let Some(m) = target else {
        return;
    };
    let scale = m.scale_factor();
    let origin = m.position().to_logical::<i32>(scale);
    let area = m.size().to_logical::<i32>(scale);
    let (mut w, mut h) = (p.w.unwrap_or(1280) as i32, p.h.unwrap_or(800) as i32);
    if home.is_none() {
        // Scaled to fit the display it falls back to.
        let fit = f64::min(1.0, f64::min(area.width as f64 / w as f64, area.height as f64 / h as f64) * 0.95);
        w = (w as f64 * fit) as i32;
        h = (h as f64 * fit) as i32;
    }
    window.set_inner_size(LogicalSize::new(w.max(640) as u32, h.max(400) as u32));
    let (x, y) = match (home, p.x, p.y) {
        (Some(_), Some(x), Some(y)) => (origin.x + x, origin.y + y),
        _ => (origin.x + (area.width - w) / 2, origin.y + (area.height - h) / 2),
    };
    window.set_outer_position(LogicalPosition::new(x, y));
    if p.maximized {
        window.set_maximized(true);
    }
}

// ── Where a window may go (decisions §31 "Desktop app navigation") ──────────────────────────
//
// The station's page gets the user's login token through its initialization script, which runs
// on every page the web view loads: a window stays on its station (or, before connecting, the
// bundled connect page); anything else opens in the system browser and never sees the token.

/// Whether `url` is `base` itself or below it (`http://10.0.0.5:47656/…`).
fn within(url: &str, base: &str) -> bool {
    url == base || url.strip_prefix(base).is_some_and(|rest| rest.starts_with(['/', '?', '#']))
}

/// The navigation rule of a window showing `base`: its own pages load, `about:blank` too;
/// any other address opens in the system browser instead.
fn stay_on(base: String) -> impl Fn(String) -> bool + 'static {
    move |url: String| {
        if within(&url, &base) || url == "about:blank" {
            return true;
        }
        open_in_browser(&url);
        false
    }
}

/// A new window a page asks for: never one of ours; web links open in the system browser.
fn no_new_windows(url: String) -> bool {
    open_in_browser(&url);
    false
}

/// Open a web address in the system's browser (only http / https; nothing else is started).
fn open_in_browser(url: &str) {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        log::warn!("not opening {url}: only web addresses open in the browser");
        return;
    }
    let started = crate::platform::browser(url).spawn();
    if let Err(e) = started {
        log::warn!("could not open {url} in the browser: {e}");
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::within;

    #[test]
    fn a_window_stays_on_its_station() {
        let base = "http://10.0.0.5:47656";
        assert!(within("http://10.0.0.5:47656", base));
        assert!(within("http://10.0.0.5:47656/?screen=2", base));
        assert!(within("http://10.0.0.5:47656/api/station", base));
        assert!(!within("http://10.0.0.5:476560/", base), "another port");
        assert!(!within("http://10.0.0.5:47656.evil.example/", base), "another host");
        assert!(!within("https://example.com/", base));
    }
}
