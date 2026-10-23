//! The shell port (TCP, newline-delimited JSON): a client sends keys, sizes
//! and UI gestures; the station renders the session and sends ANSI frames, UI
//! events and status (decisions §23: stations run the terminals, clients only
//! stream them).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShellClientPacket {
    Hello {
        user: String,
        width: u16,
        height: u16,
        /// Password, when the user has one (or log in with `token`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        password: Option<String>,
        /// Login token from `POST /api/login`; replaces user + password.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token: Option<String>,
        /// The real client address when a bridge (the WebSocket proxy) connects
        /// on its behalf — decides whether this is a remote login.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_addr: Option<String>,
        /// [`crate::CLIENT_PROTOCOL`] of the client; 0 = a client older than
        /// the field. A mismatch is refused (E-WIRE-001).
        #[serde(default)]
        protocol: u32,
        /// The device this client belongs to (decisions §30 "12a3"): every
        /// window of an app shares one id, a browser tab has its own. The
        /// station keeps the device's lock and normal theme by it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        device: Option<String>,
    },
    /// A System Monitor client (sysmon lines only).
    MonitorHello {
        #[serde(default)]
        protocol: u32,
    },
    Key {
        key: RemoteShellKey,
    },
    Resize {
        width: u16,
        height: u16,
    },
    /// The client's terminal windows, focused first. Each gets its own frame
    /// at its own size, drawn from the same session (same command line,
    /// history and output). Empty = one frame at the session size.
    Viewports {
        viewports: Vec<Viewport>,
    },
    /// The client moved UI focus (terminal or a panel).
    Focus {
        focus: UiFocus,
        /// Number of the focused window in the client's screen; window
        /// commands without a placement act beside it.
        #[serde(default)]
        window: Option<u32>,
    },
    /// The client's screen (screen); view commands default to it.
    Screen {
        no: Option<u32>,
    },
    /// A window of the client's screen was scrolled: its first visible row
    /// (decisions §30 "12e hardening", R1 Q31). Quiet and station-local —
    /// no echo, no Oops step, never replicated; the window keeps it.
    Scroll {
        window: u32,
        row: f64,
    },
    /// A command issued by the client's UI (dialog OK …); echoed in the
    /// session's output like a typed command.
    Command {
        cmd: String,
        /// The window the line belongs to (decisions §30 "12c"): a window
        /// bar, window menu or settings-window line names its window here
        /// (a number in the client's screen), and `Self` in the line is that
        /// window. None = the focused window.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<u32>,
    },
    /// A continuous control's line (decisions §30 "12f": the gesture channel): a fader or an
    /// encoder sends a real command line per move and one at its end. It runs like a typed line
    /// (rights, dispatch, replication) but quietly — no echo, history, Usage or journal per move;
    /// the end files one entry ("Fader Master 2.1: 100 → 63.5"). While the station is busy only
    /// the newest move of a gesture waits; an end is never dropped.
    Gesture {
        /// The gesture (one per drag, chosen by the client).
        id: String,
        line: String,
        phase: GesturePhase,
        /// The window the control sits in (`Self`), as for `Command`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<u32>,
        /// What moves, for the log ("Fader Master 2.1").
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        /// Where it stood before the gesture (the first move carries it).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<String>,
        /// Where this line puts it ("63.5").
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    /// A UI key gesture (`press`, `release`, `hold`, `releaseafterhold`,
    /// `double`), with Meow held or latched; the server runs the key's command
    /// line (terminal/repl: `run_key_action`, decisions §17).
    KeyAction {
        key: String,
        gesture: String,
        #[serde(default)]
        meow: bool,
        /// A key placed in a window bar: its window (`Self`); None = the
        /// focused window (decisions §30 "12c").
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<u32>,
    },
    /// A keyboard key's later gesture (decisions §17, §29): `hold` while it
    /// is still down, `release` / `releaseafterhold` when it comes up. The
    /// press itself arrives as `Key`. The station runs the bound UI key's
    /// command line for that gesture.
    KeyGesture {
        key: RemoteShellKey,
        gesture: String,
    },
    /// A UI object was tapped, e.g. `object: "viewbutton", no: 4`. Completes a
    /// pending command (`Store` + tap) or runs the default action. `hold` =
    /// tap-and-hold (store).
    Tap {
        object: String,
        no: u32,
        #[serde(default)]
        hold: bool,
        /// A pool window's tap action (`{obj}` = the object, e.g. `Edit {obj}`;
        /// "" = only complete a pending command). None = call the object.
        #[serde(default)]
        action: Option<String>,
        /// The window tapped in: the line it makes is that window's (`Self`; a Presets window's
        /// lit tags go on what Store makes, decisions 0288).
        #[serde(default)]
        window: Option<u32>,
    },
    /// The answer to one of the station's dialogs (decisions §30 "12b"):
    /// the station closes it everywhere, then runs the button's line.
    DialogAnswer {
        #[serde(flatten)]
        answer: crate::dialog::DialogAnswer,
    },
    /// A touch or key on a dialog with a countdown: the station stops it.
    DialogPause {
        id: String,
    },
    /// Ask the station for suggestions (decisions §30 "12b2"): for `line` at
    /// `cursor` (chars) — or, with no line, for the session's own command
    /// line where it is now (the bottom bar). Answered by
    /// [`ShellServerPacket::Suggestions`] with the same `seq`.
    Suggest {
        seq: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        line: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cursor: Option<u32>,
        #[serde(default)]
        surface: crate::suggest::Surface,
    },
    /// Write the session's command line (taking a suggestion): chars
    /// `start..end` become `text`, the cursor goes after it. Nothing happens
    /// when the line is no longer `expect` (the user typed on meanwhile).
    LineEdit {
        start: u32,
        end: u32,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expect: Option<String>,
    },
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShellServerPacket {
    Frame {
        ansi: String,
        /// The viewport this frame is for; None = the session-size frame.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        view: Option<u32>,
    },
    /// Navigation event for the client's focused panel.
    Ui {
        event: UiEvent,
    },
    /// Status for the client's command line bar; sent when it changes.
    Status {
        status: Box<crate::Status>,
    },
    /// The answer to a [`ShellClientPacket::Suggest`].
    Suggestions {
        suggestions: Box<crate::suggest::Suggestions>,
    },
    Error {
        message: String,
    },
    Closed {
        message: String,
        exit_code: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteShellCode {
    Char(char),
    Enter,
    Tab,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    Esc,
    Insert,
    F(u8),
    /// Print Screen (takes screenshots; decisions §29 "11d").
    PrintScreen,
    /// Pause (desk lock, like F9; decisions §30 "12a3").
    Pause,
}

/// Modifier bits on the wire (the VT / xterm layout, decisions §30 "12a3"):
/// Shift 1, Alt 2, Ctrl 4. Clients send these; the station converts them
/// for its terminal (crossterm orders Ctrl and Alt the other way round).
pub mod modifiers {
    pub const SHIFT: u8 = 1;
    pub const ALT: u8 = 2;
    pub const CTRL: u8 = 4;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteShellKey {
    pub code: RemoteShellCode,
    pub modifiers: u8,
    /// Auto-repeat flag when the client knows it (browsers do, plain terminals
    /// don't). `None` falls back to timing heuristics.
    #[serde(default)]
    pub repeat: Option<bool>,
}

/// Where a session's navigation keys go. Typing always goes to the command line;
/// focus only decides who gets arrows, Enter (on an empty line), Backspace,
/// Delete and Esc.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "target", rename_all = "snake_case")]
pub enum UiFocus {
    #[default]
    Terminal,
    /// A UI panel, e.g. `kind: "groups"`.
    Panel { kind: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveDir {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Navigation intent routed to the focused panel. The panel decides what each
/// one means for its content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UiEvent {
    /// Arrow / page / home / end.
    Move { dir: MoveDir },
    /// Enter on an empty command line (KBD / PRG): act on the focused item.
    Activate,
    /// Enter on an empty command line (NAV): open the focused item's editor.
    Edit,
    /// Delete: remove the focused item (the panel confirms).
    Delete,
    /// Backspace in KBD: plain erase inside the panel.
    Erase,
    /// Backspace in NAV: up one level.
    Back,
    /// Backspace held in NAV: up to the top level.
    Root,
    /// Esc: hand focus back to the terminal.
    FocusTerminal,
    /// `Window <n>`: focus window `no` (activating its tab).
    FocusWindow { no: u32 },
    /// `ScreenConfig n`: this client switches to screen config `no` (its
    /// screens are opened from the show).
    ScreenConfig { no: u32 },
    /// This client now works as `user` (after `Login`): rejoin a screen of
    /// that user and reload their view bar.
    UserChanged {
        user: String,
        /// A login token for the new user (replaces the client's token).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token: Option<String>,
    },
    /// `Logout` / `Disconnect`: forget the login (the shell closes next).
    /// The app returns to this server's user list, or with `disconnect` to
    /// its server list.
    LoggedOut {
        #[serde(default)]
        disconnect: bool,
    },
    /// `Connect …`: the app switches this window to another server
    /// (`target` = station name or address, `host[:port]`), logging in as
    /// `user` when given.
    Connect {
        target: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        user: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    /// `EditSetting <object>` (decisions §29): open a
    /// settings pop-up for `title` with `fields`; each changed field runs its
    /// `line` with `{value}` filled in.
    EditSettings {
        title: String,
        fields: Vec<SettingField>,
        /// A line under the title (`from preset 2 "Big DMX" · changed`).
        #[serde(default, skip_serializing_if = "String::is_empty")]
        subtitle: String,
        /// What the settings belong to as a command word (`Window 1.4`):
        /// the window's own buttons (Load / Save preset, Reset, Delete) send
        /// their lines with it as the owner (decisions §30 "12c").
        #[serde(default, skip_serializing_if = "String::is_empty")]
        target: String,
    },
    /// `Menu ["name"] [/Tab x]` (decisions §30 "12d"): open a menu of the
    /// UI — `menu` is its MenuDef name (`Menu` = the ⚙ menu, `Shutdown`,
    /// `Settings`, `Network`, …).
    OpenMenu {
        menu: String,
        /// The tab to show: the one named with `/Tab`, else the profile's
        /// last tab of this menu; None = the first.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tab: Option<String>,
        /// The line named no tab: an open menu closes again (never a
        /// stacked copy).
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        toggle: bool,
        /// The user's grid states of this menu's grids (JSON object text
        /// keyed by grid key; empty = none stored).
        #[serde(default, skip_serializing_if = "String::is_empty")]
        grids: String,
        /// `/Open "<row> [<group> [<part>]]"`: the tab's row to drill into
        /// and where (decisions §30 "Users & Profiles follows storage").
        #[serde(default, skip_serializing_if = "String::is_empty")]
        open: String,
    },
    /// `Edit [UserProfile ["x"]] Attribute [/New] [/FixtureType]` (decisions
    /// §30 "12f-a"): open the attribute board of the show (`profile` 0) or a
    /// profile, fed by the `attributes` feed; `owner` starts its lines
    /// (`""`, `UserProfile`, `UserProfile "Bob"`), `filter` all / new /
    /// doubt / unused, `view` group / type.
    AttributeBoard {
        #[serde(default)]
        profile: u32,
        #[serde(default)]
        owner: String,
        filter: String,
        view: String,
    },
    /// `Edit Sequence <n>` (decisions §30 "12f" sequence window A): open the
    /// sequence sheet of `sequence` (its GUID) as a pop-up, fed by the
    /// `sequence` feed.
    SequenceSheet { sequence: String },
    /// Close open menus / pop-ups (a UI key's double press that replaces its
    /// press: Menu twice closes the menu again and quick-saves).
    CloseMenu,
    /// A short message for the user (`level`: info / error), e.g. the result
    /// of a UI key's double press ("Show \"x\" saved").
    Notice { text: String, level: String },
    /// Put `text` on the user's clipboard, the one of the computer this
    /// client runs on (the NAV inspector's copy keys), and say "Copied
    /// `label`". Never the station's object clipboard (`Copy` / `Paste`). A
    /// terminal writes OSC 52 ([`crate::clipboard::osc52`]); the page uses the
    /// browser's clipboard or hands it to the app.
    CopyText { label: String, text: String },
    /// The user's notification settings changed: the UI reloads them
    /// (`GET /api/feedback`).
    FeedbackSettings,
    /// `Sidebars` / `ViewBar` / `FullScreen` (decisions §30 "12a3"): the
    /// window the line came from shows or hides its sidebars (F12), its view
    /// bar (Meow+F10) or goes native full screen (Meow+F12). `on` None =
    /// toggle.
    Page {
        what: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        on: Option<bool>,
    },
}

/// One terminal window of a client: rendered separately at its own size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viewport {
    /// The client's id for it (the web UI uses the terminal's window number).
    pub view: u32,
    pub width: u16,
    pub height: u16,
    /// What it shows: the full terminal, only the command line (the screen's
    /// command line bar), or only the output (command line history).
    #[serde(default)]
    pub layers: ViewLayers,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewLayers {
    #[default]
    Full,
    Input,
    Output,
}

/// One field of the settings window (`EditSetting`, decisions §29; typed
/// since §30 "12b2": the kit draws the right cell for `input`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingField {
    pub name: String,
    /// The setting's stable key (`auto_scroll`, `window_bar_items`): pins and
    /// `data-id`s use it (decisions §30 "12c"); empty for non-window fields.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub key: String,
    /// The value differs from the default (the cell shows ↺, which runs
    /// `reset`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub changed: bool,
    /// The line that puts the default back (`Set Window 1.4 readout Default`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reset: String,
    /// The tab it sits on; empty = the window has one section (no tab strip).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub section: String,
    /// What it takes.
    pub input: FieldInput,
    /// A short explanation (what `Fit` means).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hint: String,
    /// The current value as text.
    pub value: String,
    /// The command line that sets it, with `{value}` for the new value.
    pub line: String,
    /// A link: the pick list's items (decisions §30 "12b"); `{value}` takes
    /// the picked item's number, or `none`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick: Option<crate::dialog::Pick>,
    /// Where it lives (decisions §30 "12d"): `station` (this station, never
    /// in the show) or `show` (in the show, every station); empty = not said.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub home: String,
    /// The unit a number shows (`Hz`, `min`, `s`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
    /// The level needed to change it (`Setup`); empty = no own level.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rights: String,
    /// Why it can't be changed here; empty = editable.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub readonly: String,
}

/// The input a settings field takes (decisions §30 "12b2": named choices,
/// never a magic number). `{value}` in the field's line takes the text typed
/// or picked: a choice's label, a named number's name, a link's number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldInput {
    Text {
        #[serde(default)]
        max_len: usize,
    },
    /// On / Off.
    Toggle,
    /// A whole number (the keypad); `named` are numbers with a name (`0` =
    /// `Fit`), offered as choices beside the keypad.
    Number {
        min: i64,
        max: i64,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        named: Vec<String>,
    },
    /// One of these labels (the swipe button).
    Choice { options: Vec<String> },
    /// A link: the field's `pick` lists the items.
    Link,
    /// A colour `#rrggbb`; empty = the theme's.
    Colour,
    /// An object's Paint (decisions 0294): drawn by the kit's paint picker (`<luma-paint>`, on the
    /// `paint` feed) — `#rrggbb`, a palette word, `Theme "<slot>"` or `none`.
    Paint,
    /// An ordered list of ids (the window bar's items): `items` are what may
    /// be in it; `{value}` takes the ids `"a,b,c"`.
    List { items: Vec<ListItem> },
    /// The tag tree as a picker (decisions §12 "Tags hardened" 8): a tick runs `assign`, an
    /// untick `unassign` (`{value}` = the tag's path); "New tag here" runs `store` under the
    /// highlighted tag or `store_top` with none (`{name}` = the new name, quoted).
    Tags { nodes: Vec<TagNode>, assign: String, unassign: String, store: String, store_top: String },
}

/// One tag of a picker tree, depth first (a parent before its children).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagNode {
    /// Its number, the path (`1.2.1`).
    pub path: String,
    pub name: String,
    /// `#rrggbb`, or empty.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub colour: String,
    /// What the edited objects carry: `on` (all of them), `part` (some), `off`.
    pub state: String,
    /// How many objects carry it directly (for the tree's count).
    #[serde(default)]
    pub count: usize,
}

/// One id a list field may hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListItem {
    pub id: String,
    pub label: String,
    /// `setting` or `action`.
    pub kind: String,
    /// An action's command line (empty for a setting).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub line: String,
}

impl FieldInput {
    /// What it takes, as a terminal shows it (`on/off`, `0–64 or Fit`).
    pub fn label(&self) -> String {
        match self {
            FieldInput::Text { .. } => "text".into(),
            FieldInput::Toggle => "on/off".into(),
            FieldInput::Number { min, max, named } if named.is_empty() => format!("{min}–{max}"),
            FieldInput::Number { min, max, named } => format!("{min}–{max} or {}", named.join("/")),
            FieldInput::Choice { options } => options.join("/"),
            FieldInput::Link => "name or number".into(),
            FieldInput::Colour => "#rrggbb or Theme".into(),
            FieldInput::Paint => "#rrggbb, a palette colour, Theme \"<slot>\" or none".into(),
            FieldInput::List { .. } => "\"a,b,c\"".into(),
            FieldInput::Tags { .. } => "tag paths".into(),
        }
    }
}

/// Where a gesture is (decisions §30 "12f").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GesturePhase {
    Move,
    End,
}
