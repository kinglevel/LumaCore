//! View-sessions: the live windows of one screen.
//!
//! A user can have several screens (desktop, tablet, phone…), each with
//! its own split-tree layout of numbered windows. Published at
//! `Root.Runtime.Users.<user>.Screens.<n>.Windows.<no>`; this is the
//! canonical, editable state — UI engines only render it.
//!
//! Modelled on ScreenContent → ViewWidget (settings, appearance,
//! scroll positions per window), with a split tree instead of a grid.
//!
//! The store is shared (`SharedScreens`) between the command thread and UI
//! engines; listeners are notified after every change so engines can push it to
//! their clients. Window settings are stored as plain JSON here — validation
//! against the window-type schema happens in the callers (lumacore-tree).

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub type SharedScreens = Arc<RwLock<ScreenStore>>;

/// Window kind of an empty pane: a placeholder the UI shows as "Add window".
/// Storing a window into it replaces it.
pub const EMPTY_WINDOW: &str = "empty";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitDir {
    /// Children side by side.
    H,
    /// Children stacked.
    V,
}

/// Split-tree node. Leaves are panels holding one or more windows as tabs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LayoutNode {
    Split {
        dir: SplitDir,
        /// Fraction of space given to the first child (0.0–1.0).
        ratio: f64,
        children: Vec<LayoutNode>,
    },
    Leaf {
        /// Window numbers shown as tabs, in header order.
        windows: Vec<u32>,
        /// Index of the active tab.
        #[serde(default)]
        active: usize,
    },
}

impl LayoutNode {
    /// The same layout with every pane on its first tab (comparing layouts
    /// without the tab each pane shows).
    pub fn without_active(&self) -> Self {
        match self {
            Self::Split {
                dir,
                ratio,
                children,
            } => Self::Split {
                dir: *dir,
                ratio: *ratio,
                children: children.iter().map(Self::without_active).collect(),
            },
            Self::Leaf { windows, .. } => Self::Leaf {
                windows: windows.clone(),
                active: 0,
            },
        }
    }

    fn collect_windows(&self, out: &mut Vec<u32>) {
        match self {
            Self::Split { children, .. } => children.iter().for_each(|c| c.collect_windows(out)),
            Self::Leaf { windows, .. } => out.extend(windows),
        }
    }

    /// The leaf showing `window`, or the first leaf when `window` is None.
    fn leaf_mut(&mut self, window: Option<u32>) -> Option<(&mut Vec<u32>, &mut usize)> {
        match self {
            Self::Split { children, .. } => children.iter_mut().find_map(|c| c.leaf_mut(window)),
            Self::Leaf { windows, active } => match window {
                Some(w) if !windows.contains(&w) => None,
                _ => Some((windows, active)),
            },
        }
    }

    /// Drop `gone` from every leaf; empty leaves disappear and their splits
    /// collapse into the sibling. None when nothing is left.
    pub fn without(self, gone: &[u32]) -> Option<Self> {
        match self {
            Self::Split {
                dir,
                ratio,
                children,
            } => {
                let mut kept: Vec<Self> = children
                    .into_iter()
                    .filter_map(|c| c.without(gone))
                    .collect();
                match kept.len() {
                    0 => None,
                    1 => kept.pop(),
                    _ => Some(Self::Split {
                        dir,
                        ratio,
                        children: kept,
                    }),
                }
            }
            Self::Leaf { windows, active } => {
                let current = windows.get(active).copied();
                let windows: Vec<u32> = windows.into_iter().filter(|w| !gone.contains(w)).collect();
                if windows.is_empty() {
                    return None;
                }
                let active = current
                    .and_then(|c| windows.iter().position(|w| *w == c))
                    .unwrap_or(active.min(windows.len() - 1));
                Some(Self::Leaf { windows, active })
            }
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowAppearance {
    /// Optional accent color override ("#rrggbb").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WindowScroll {
    #[serde(default)]
    pub h: f64,
    #[serde(default)]
    pub v: f64,
}

/// One live window (a ViewWidget).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub no: u32,
    /// Window type name, e.g. "groups" (see `Root.Resources.WindowTypes`).
    pub kind: String,
    /// Tab title; the window type's label when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub settings: Map<String, Value>,
    #[serde(default)]
    pub appearance: WindowAppearance,
    #[serde(default)]
    pub scroll: WindowScroll,
    /// The `WindowPreset` it was last loaded from (decisions §30 "12c"): a
    /// copy, not a live link — only so the Load list can preselect it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screen {
    /// Who it is, with its display name: kept in the station's files (decisions §12 "Tags part
    /// 1b hardened" 6).
    #[serde(default)]
    pub meta: lumacore_base::Identity,
    pub no: u32,
    pub user: String,
    /// Which screen of the user's profile this is: screen configuration and
    /// screen number within it. Its layout is kept in
    /// `UserProfile.screen_configs[config].screens[screen]` (show data).
    #[serde(default = "one")]
    pub config: u32,
    #[serde(default = "one")]
    pub screen: u32,
    pub root: LayoutNode,
    pub windows: BTreeMap<u32, Window>,
    /// The window held full screen on this screen (decisions §30 "12c2":
    /// a phone's hold-full-screen with the profile's `HoldFollows` on —
    /// every client of the screen shows it maximised). Runtime only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximized: Option<u32>,
    /// What this screen's encoder bar shows (decisions §30 "12f"): mode, strip button, page,
    /// layer. Runtime only — switching is not an Oops step, like a tab.
    #[serde(default)]
    pub encoder: EncoderState,
    /// Bumped on every change; clients use it to skip their own echoes.
    pub revision: u64,
}

impl Window {
    pub fn new(no: u32, kind: &str) -> Self {
        Self {
            no,
            kind: kind.into(),
            label: None,
            settings: Map::new(),
            appearance: WindowAppearance::default(),
            scroll: WindowScroll::default(),
            preset: None,
        }
    }

    /// Every setting and the bar back to the type's defaults (`Reset
    /// Self`); label, colour and scroll stay.
    pub fn reset_settings(&mut self) {
        self.settings.clear();
        self.preset = None;
    }
}

fn one() -> u32 {
    1
}

/// The layout a new screen starts with: one panel with a Terminal and a
/// Groups window.
pub fn default_layout() -> (LayoutNode, BTreeMap<u32, Window>) {
    let s = Screen::new(0, "");
    (s.root, s.windows)
}

impl Screen {
    fn new(no: u32, user: &str) -> Self {
        let window = Window::new;
        Self {
            no,
            user: user.into(),
            config: 1,
            screen: 1,
            meta: lumacore_base::Identity::new(Some(format!("Screen {no}"))),
            root: LayoutNode::Leaf {
                windows: vec![1, 2],
                active: 0,
            },
            windows: [(1, window(1, "terminal")), (2, window(2, "groups"))].into(),
            maximized: None,
            encoder: EncoderState::default(),
            revision: 1,
        }
    }
}

type Listener = Box<dyn Fn(&Screen) + Send + Sync>;
type ViewsListener = Box<dyn Fn(&str) + Send + Sync>;
type ReloadListener = Box<dyn Fn(Option<&[u32]>) + Send + Sync>;
type CaptureListener = Box<dyn Fn(&CaptureRequest) + Send + Sync>;

/// A screenshot asked of the clients showing some live screens (decisions
/// §29 "11d"): each of them renders its screen as seen and uploads it with
/// this `id` (`POST /api/screenshots`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureRequest {
    pub id: u64,
    /// The live screens to capture.
    pub screens: Vec<u32>,
}

/// A capture still waiting for uploads.
#[derive(Debug, Clone)]
struct PendingCapture {
    /// Who asked (their notices tell where the files went).
    asked_by: String,
    /// Live screen → uploads still expected.
    expected: BTreeMap<u32, u32>,
    asked: std::time::Instant,
}

/// A capture that didn't arrive in time (`ScreenStore::overdue`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverdueCapture {
    pub id: u64,
    pub asked_by: String,
    /// Live screens with missing uploads.
    pub screens: Vec<u32>,
}

#[derive(Default)]
pub struct ScreenStore {
    sessions: BTreeMap<u32, Screen>,
    /// Sessions changed since the last `take_changed` — copied into the
    /// users' screen configs (show data) by the server loop.
    changed: std::collections::BTreeSet<u32>,
    listeners: Vec<Listener>,
    views_listeners: Vec<ViewsListener>,
    reload_listeners: Vec<ReloadListener>,
    /// Clients viewing each live screen (UI clients count themselves in
    /// while their event stream is open).
    viewers: BTreeMap<u32, u32>,
    capture_listeners: Vec<CaptureListener>,
    captures: BTreeMap<u64, PendingCapture>,
    next_capture: u64,
}

impl fmt::Debug for ScreenStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScreenStore")
            .field("sessions", &self.sessions)
            .field(
                "listeners",
                &(self.listeners.len()
                    + self.views_listeners.len()
                    + self.reload_listeners.len()
                    + self.capture_listeners.len()),
            )
            .finish()
    }
}

impl ScreenStore {
    pub fn shared() -> SharedScreens {
        Arc::new(RwLock::new(Self::default()))
    }

    /// Called after every change with the updated screen.
    pub fn subscribe(&mut self, listener: impl Fn(&Screen) + Send + Sync + 'static) {
        self.listeners.push(Box::new(listener));
    }

    /// Called with the user name whenever that user's stored views or view
    /// buttons change (they live in ShowData; this is only the notification).
    pub fn subscribe_views(&mut self, listener: impl Fn(&str) + Send + Sync + 'static) {
        self.views_listeners.push(Box::new(listener));
    }

    pub fn views_changed(&self, user: &str) {
        for listener in &self.views_listeners {
            listener(user);
        }
    }

    /// Called by `ReloadUI` with the screens to reload (None = every UI client).
    pub fn subscribe_reload(&mut self, listener: impl Fn(Option<&[u32]>) + Send + Sync + 'static) {
        self.reload_listeners.push(Box::new(listener));
    }

    /// Ask UI engines to reload the given screens' UI (None = all of them).
    pub fn reload_ui(&self, screens: Option<&[u32]>) {
        for listener in &self.reload_listeners {
            listener(screens);
        }
    }

    pub fn get(&self, no: u32) -> Option<&Screen> {
        self.sessions.get(&no)
    }

    // ── Screenshots (decisions §29 "11d") ───────────────────────────────

    /// A UI client started (true) or stopped viewing live screen `no`.
    pub fn viewer(&mut self, no: u32, joined: bool) {
        let count = self.viewers.entry(no).or_default();
        if joined {
            *count += 1;
        } else {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.viewers.remove(&no);
            }
        }
    }

    /// How many UI clients view live screen `no`.
    pub fn viewers(&self, no: u32) -> u32 {
        self.viewers.get(&no).copied().unwrap_or(0)
    }

    /// Called with every capture request (UI engines pass it to their clients).
    pub fn subscribe_capture(&mut self, listener: impl Fn(&CaptureRequest) + Send + Sync + 'static) {
        self.capture_listeners.push(Box::new(listener));
    }

    /// Ask every client viewing `screens` for a screenshot. Returns the
    /// request id and how many clients were asked (0: nobody views them —
    /// nothing is asked).
    pub fn ask_capture(&mut self, asked_by: &str, screens: &[u32]) -> (u64, u32) {
        let expected: BTreeMap<u32, u32> = screens
            .iter()
            .map(|&no| (no, self.viewers(no)))
            .filter(|(_, n)| *n > 0)
            .collect();
        let clients = expected.values().sum();
        if clients == 0 {
            return (0, 0);
        }
        self.next_capture += 1;
        let id = self.next_capture;
        let request = CaptureRequest {
            id,
            screens: expected.keys().copied().collect(),
        };
        self.captures.insert(
            id,
            PendingCapture {
                asked_by: asked_by.to_string(),
                expected,
                asked: std::time::Instant::now(),
            },
        );
        for listener in &self.capture_listeners {
            listener(&request);
        }
        (id, clients)
    }

    /// An upload for capture `id` of live screen `screen` arrived: who asked
    /// for it, or None when it wasn't asked for (or already complete).
    pub fn capture_arrived(&mut self, id: u64, screen: u32) -> Option<String> {
        let pending = self.captures.get_mut(&id)?;
        let left = pending.expected.get_mut(&screen).filter(|n| **n > 0)?;
        *left -= 1;
        let asked_by = pending.asked_by.clone();
        if pending.expected.values().all(|n| *n == 0) {
            self.captures.remove(&id);
        }
        Some(asked_by)
    }

    /// Captures older than `after` that still miss uploads (removed).
    pub fn overdue(&mut self, after: std::time::Duration) -> Vec<OverdueCapture> {
        let late: Vec<u64> = self
            .captures
            .iter()
            .filter(|(_, p)| p.asked.elapsed() >= after)
            .map(|(id, _)| *id)
            .collect();
        late.into_iter()
            .filter_map(|id| {
                let p = self.captures.remove(&id)?;
                Some(OverdueCapture {
                    id,
                    asked_by: p.asked_by,
                    screens: p
                        .expected
                        .into_iter()
                        .filter(|(_, n)| *n > 0)
                        .map(|(no, _)| no)
                        .collect(),
                })
            })
            .collect()
    }

    pub fn all(&self) -> impl Iterator<Item = &Screen> {
        self.sessions.values()
    }

    pub fn for_user<'a>(&'a self, user: &'a str) -> impl Iterator<Item = &'a Screen> {
        self.sessions
            .values()
            .filter(move |s| s.user.eq_ignore_ascii_case(user))
    }

    /// The live screen showing `user`'s screen `screen` of config
    /// `config`, if any.
    pub fn find_screen(&self, user: &str, config: u32, screen: u32) -> Option<u32> {
        self.sessions
            .values()
            .find(|s| s.user.eq_ignore_ascii_case(user) && s.config == config && s.screen == screen)
            .map(|s| s.no)
    }

    /// Open `user`'s screen `screen` of config `config`: the live screen
    /// if one exists, else a new one laid out from `saved` (the screen's
    /// content from the show) or the default layout. Returns its number.
    pub fn open_screen(
        &mut self,
        user: &str,
        config: u32,
        screen: u32,
        saved: Option<(LayoutNode, BTreeMap<u32, Window>)>,
    ) -> u32 {
        if let Some(no) = self.find_screen(user, config, screen) {
            return no;
        }
        let no = self.sessions.keys().next_back().map_or(1, |n| n + 1);
        let mut session = Screen::new(no, user);
        session.config = config;
        session.screen = screen;
        session.meta.name = Some(format!("Screen {screen}"));
        if let Some((root, windows)) = saved {
            session.root = root;
            session.windows = windows;
        }
        self.sessions.insert(no, session);
        self.notify(no);
        // Opening restores show data — not a change to save back.
        self.changed.remove(&no);
        no
    }

    /// Sessions changed since the last call (for copying into show data).
    pub fn take_changed(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.changed).into_iter().collect()
    }

    /// Lay a session out again from show data (show load). Not recorded as
    /// a change.
    pub fn restore(&mut self, no: u32, root: LayoutNode, windows: BTreeMap<u32, Window>) {
        if self.replace_layout(no, root, windows).is_ok() {
            self.changed.remove(&no);
        }
    }

    /// Join `join` if it exists and belongs to `user`, otherwise create a new
    /// screen with the default layout. Returns its number.
    ///
    /// A requested number that is free is reused, so a screen keeps its number
    /// across server restarts and stored multi-screen views find it again.
    pub fn join_or_create(&mut self, user: &str, join: Option<u32>) -> u32 {
        if let Some(no) = join
            && self
                .sessions
                .get(&no)
                .is_some_and(|s| s.user.eq_ignore_ascii_case(user))
        {
            return no;
        }
        let no = match join {
            Some(n) if n > 0 && !self.sessions.contains_key(&n) => n,
            _ => self.sessions.keys().next_back().map_or(1, |n| n + 1),
        };
        let mut session = Screen::new(no, user);
        // Next free screen of the user's first config.
        session.screen = (1..)
            .find(|n| self.find_screen(user, 1, *n).is_none())
            .unwrap_or(1);
        session.meta.name = Some(format!("Screen {}", session.screen));
        self.sessions.insert(no, session);
        self.notify(no);
        no
    }

    /// Replace a screen's layout and windows. Windows not referenced by
    /// the tree are dropped; tree references without a window are an error.
    pub fn replace_layout(
        &mut self,
        no: u32,
        root: LayoutNode,
        mut windows: BTreeMap<u32, Window>,
    ) -> Result<u64, String> {
        let session = self
            .sessions
            .get_mut(&no)
            .ok_or_else(|| format!("no screen {no}"))?;
        let mut used = Vec::new();
        root.collect_windows(&mut used);
        if let Some(missing) = used.iter().find(|n| !windows.contains_key(n)) {
            return Err(format!("layout references unknown window {missing}"));
        }
        windows.retain(|n, _| used.contains(n));
        for (n, w) in windows.iter_mut() {
            w.no = *n;
        }
        if session.maximized.is_some_and(|m| !windows.contains_key(&m)) {
            session.maximized = None;
        }
        session.root = root;
        session.windows = windows;
        session.revision += 1;
        let revision = session.revision;
        self.notify(no);
        Ok(revision)
    }

    /// Edit one window in place (e.g. a setting changed from the command line).
    pub fn update_window(
        &mut self,
        no: u32,
        window: u32,
        f: impl FnOnce(&mut Window) -> Result<(), String>,
    ) -> Result<u64, String> {
        let session = self
            .sessions
            .get_mut(&no)
            .ok_or_else(|| format!("no screen {no}"))?;
        let w = session
            .windows
            .get_mut(&window)
            .ok_or_else(|| format!("no window {window} in screen {no}"))?;
        f(w)?;
        session.revision += 1;
        let revision = session.revision;
        self.notify(no);
        Ok(revision)
    }

    /// A window's scroll as the page reports it (§30 "12e hardening"): quiet —
    /// no revision, no listeners (a scroll is no layout change and must not
    /// make the page redraw or a layout PUT conflict).
    pub fn set_scroll(&mut self, no: u32, window: u32, scroll: WindowScroll) -> Result<(), String> {
        let w = self
            .sessions
            .get_mut(&no)
            .ok_or_else(|| format!("no screen {no}"))?
            .windows
            .get_mut(&window)
            .ok_or_else(|| format!("no window {window} in screen {no}"))?;
        w.scroll = scroll;
        Ok(())
    }

    /// Add a `kind` window as a tab beside `beside` (in the same pane, right
    /// after it), or in the first pane when `beside` is None. `window` picks
    /// its number, else the next free one. Returns `(window, revision)`.
    pub fn add_window(
        &mut self,
        no: u32,
        kind: &str,
        window: Option<u32>,
        beside: Option<u32>,
    ) -> Result<(u32, u64), String> {
        let session = self
            .sessions
            .get_mut(&no)
            .ok_or_else(|| format!("no screen {no}"))?;
        let win = match window {
            Some(w) if session.windows.contains_key(&w) => {
                return Err(format!("window {w} already exists"));
            }
            Some(0) => return Err("window numbers start at 1".into()),
            Some(w) => w,
            None => session.windows.keys().next_back().map_or(1, |n| n + 1),
        };
        let (windows, active) = session.root.leaf_mut(beside).ok_or_else(|| {
            format!(
                "no window {} in screen {no}",
                beside.unwrap_or_default()
            )
        })?;
        let at = beside
            .and_then(|b| windows.iter().position(|w| *w == b))
            .map_or(windows.len(), |i| i + 1);
        windows.insert(at, win);
        *active = at;
        session.windows.insert(win, Window::new(win, kind));
        session.revision += 1;
        let revision = session.revision;
        self.notify(no);
        Ok((win, revision))
    }

    /// Change a window's type in place: same number and tab position, the new
    /// type's default settings, no label.
    pub fn set_window_kind(&mut self, no: u32, window: u32, kind: &str) -> Result<u64, String> {
        self.update_window(no, window, |w| {
            *w = Window::new(window, kind);
            Ok(())
        })
    }

    /// Remove windows; emptied panes close. Removing everything leaves one
    /// empty pane.
    pub fn remove_windows(&mut self, no: u32, gone: &[u32]) -> Result<u64, String> {
        let session = self
            .sessions
            .get_mut(&no)
            .ok_or_else(|| format!("no screen {no}"))?;
        if let Some(missing) = gone.iter().find(|w| !session.windows.contains_key(w)) {
            return Err(format!("no window {missing} in screen {no}"));
        }
        let placeholder = LayoutNode::Leaf {
            windows: Vec::new(),
            active: 0,
        };
        let root = std::mem::replace(&mut session.root, placeholder);
        session.windows.retain(|w, _| !gone.contains(w));
        if session.maximized.is_some_and(|m| gone.contains(&m)) {
            session.maximized = None;
        }
        session.root = match root.without(gone) {
            Some(root) => root,
            None => {
                let win = gone.iter().copied().min().unwrap_or(1);
                session.windows.insert(win, Window::new(win, EMPTY_WINDOW));
                LayoutNode::Leaf {
                    windows: vec![win],
                    active: 0,
                }
            }
        };
        session.revision += 1;
        let revision = session.revision;
        self.notify(no);
        Ok(revision)
    }

    /// Hold `window` full screen on screen `no` (None = none). Not a layout
    /// change: nothing is copied into the profile's screen configs.
    pub fn set_maximized(&mut self, no: u32, window: Option<u32>) -> Result<u64, String> {
        let session = self
            .sessions
            .get_mut(&no)
            .ok_or_else(|| format!("no screen {no}"))?;
        if let Some(w) = window
            && !session.windows.contains_key(&w)
        {
            return Err(format!("no window {w} in screen {no}"));
        }
        session.maximized = window;
        session.revision += 1;
        let revision = session.revision;
        for listener in &self.listeners {
            listener(session);
        }
        Ok(revision)
    }

    /// Change what the screen's encoder bar shows (`Set Screen EncoderMode …`).
    pub fn set_encoder(&mut self, no: u32, f: impl FnOnce(&mut EncoderState)) -> Result<u64, String> {
        let session = self.sessions.get_mut(&no).ok_or_else(|| format!("no screen {no}"))?;
        f(&mut session.encoder);
        session.revision += 1;
        let revision = session.revision;
        for listener in &self.listeners {
            listener(session);
        }
        Ok(revision)
    }

    fn notify(&mut self, no: u32) {
        self.changed.insert(no);
        if let Some(session) = self.sessions.get(&no) {
            for listener in &self.listeners {
                listener(session);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_capture_asks_the_viewers_and_counts_their_uploads() {
        let mut store = ScreenStore::default();
        let asked = Arc::new(RwLock::new(Vec::new()));
        let seen = asked.clone();
        store.subscribe_capture(move |r| seen.write().unwrap().push(r.clone()));
        assert_eq!(store.ask_capture("Admin", &[1]), (0, 0), "nobody views screen 1");
        store.viewer(1, true);
        store.viewer(1, true);
        store.viewer(2, true);
        let (id, clients) = store.ask_capture("Admin", &[1, 2, 3]);
        assert_eq!(clients, 3);
        assert_eq!(asked.read().unwrap()[0].screens, vec![1, 2]);
        assert_eq!(store.capture_arrived(id, 1).as_deref(), Some("Admin"));
        assert_eq!(store.capture_arrived(id, 3), None, "not asked");
        assert_eq!(store.capture_arrived(id, 2).as_deref(), Some("Admin"));
        let late = store.overdue(std::time::Duration::ZERO);
        assert_eq!(late, vec![OverdueCapture { id, asked_by: "Admin".into(), screens: vec![1] }]);
        assert_eq!(store.capture_arrived(id, 1), None, "gone once overdue");
        store.viewer(2, false);
        assert_eq!(store.viewers(2), 0);
    }

    /// A scroll report is quiet (§30 "12e hardening"): the window keeps it,
    /// but no revision moves and no listener hears of it.
    #[test]
    fn a_scroll_report_is_kept_quietly() {
        let mut store = ScreenStore::default();
        let no = store.join_or_create("Admin", None);
        let w = *store.get(no).unwrap().windows.keys().next().unwrap();
        let heard = Arc::new(RwLock::new(0));
        let h = heard.clone();
        store.subscribe(move |_| *h.write().unwrap() += 1);
        let rev = store.get(no).unwrap().revision;
        store.set_scroll(no, w, WindowScroll { h: 0.0, v: 12.0 }).unwrap();
        assert_eq!(store.get(no).unwrap().windows[&w].scroll.v, 12.0);
        assert_eq!(store.get(no).unwrap().revision, rev);
        assert_eq!(*heard.read().unwrap(), 0);
        assert!(store.set_scroll(no, 999, WindowScroll::default()).is_err());
    }

    #[test]
    fn join_existing_or_create() {
        let mut store = ScreenStore::default();
        let a = store.join_or_create("Admin", None);
        assert_eq!(store.join_or_create("Admin", Some(a)), a);
        assert_ne!(
            store.join_or_create("Other", Some(a)),
            a,
            "can't join another user's session"
        );
    }

    #[test]
    fn add_beside_and_remove_collapses() {
        let mut store = ScreenStore::default();
        let no = store.join_or_create("Admin", None);
        // Default: one pane [1 terminal, 2 groups].
        let (w, _) = store.add_window(no, "groups", None, Some(1)).unwrap();
        assert_eq!(w, 3);
        let s = store.get(no).unwrap();
        assert!(
            matches!(&s.root, LayoutNode::Leaf { windows, active: 1 } if windows == &vec![1, 3, 2])
        );
        assert!(
            store.add_window(no, "groups", Some(3), None).is_err(),
            "number taken"
        );

        store.remove_windows(no, &[1, 2, 3]).unwrap();
        let s = store.get(no).unwrap();
        assert_eq!(s.windows.len(), 1, "an empty pane is left");
        assert_eq!(s.windows[&1].kind, EMPTY_WINDOW);
    }

    #[test]
    fn replace_layout_drops_orphans_and_rejects_dangling() {
        let mut store = ScreenStore::default();
        let no = store.join_or_create("Admin", None);
        let session = store.get(no).unwrap().clone();
        let leaf = LayoutNode::Leaf {
            windows: vec![1],
            active: 0,
        };
        store
            .replace_layout(no, leaf, session.windows.clone())
            .unwrap();
        assert_eq!(store.get(no).unwrap().windows.len(), 1);

        let dangling = LayoutNode::Leaf {
            windows: vec![9],
            active: 0,
        };
        assert!(store.replace_layout(no, dangling, session.windows).is_err());
    }
}

/// What a screen's encoder bar shows (decisions §30 "12f": Mode above Layer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncoderState {
    pub mode: EncoderMode,
    /// The strip button (1-based) of the layout (Attributes) or the cue (Cue timing).
    pub group: u32,
    /// The page of four encoders (1-based).
    pub page: u32,
    pub layer: EncoderLayer,
}

impl Default for EncoderState {
    fn default() -> Self {
        Self { mode: EncoderMode::Attributes, group: 1, page: 1, layer: EncoderLayer::Value }
    }
}

/// What the whole bar is for (round 12). Effects shows as planned until §28.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EncoderMode {
    #[default]
    Attributes,
    CueTimingVertical,
    CueTimingHorizontal,
    Masters,
    Effects,
}

impl EncoderMode {
    pub const ALL: [EncoderMode; 5] =
        [Self::Attributes, Self::CueTimingVertical, Self::CueTimingHorizontal, Self::Masters, Self::Effects];
    /// The words `Set Screen "EncoderMode"` takes.
    pub const WORDS: &'static [&'static str] = &["Attributes", "CueTimingVertical", "CueTimingHorizontal", "Masters", "Effects"];

    pub fn word(self) -> &'static str {
        match self {
            Self::Attributes => "Attributes",
            Self::CueTimingVertical => "CueTimingVertical",
            Self::CueTimingHorizontal => "CueTimingHorizontal",
            Self::Masters => "Masters",
            Self::Effects => "Effects",
        }
    }

    pub fn parse(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.word().eq_ignore_ascii_case(word))
    }
}

/// Which fader of each attribute is the wide one (Value · Fade · Delay, round 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EncoderLayer {
    #[default]
    Value,
    Fade,
    Delay,
}

impl EncoderLayer {
    pub const ALL: [EncoderLayer; 3] = [Self::Value, Self::Fade, Self::Delay];
    /// The words `Set Screen "EncoderLayer"` takes.
    pub const WORDS: &'static [&'static str] = &["Value", "Fade", "Delay"];

    pub fn word(self) -> &'static str {
        match self {
            Self::Value => "Value",
            Self::Fade => "Fade",
            Self::Delay => "Delay",
        }
    }

    pub fn parse(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.word().eq_ignore_ascii_case(word))
    }
}
