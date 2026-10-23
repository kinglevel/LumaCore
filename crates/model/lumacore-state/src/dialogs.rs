//! The standard dialog on the station (decisions §30 "12b hardening"): every
//! question the station asks is one dialog it holds until it is answered or
//! its countdown ends. A command asks by returning [`DispatchResult::Ask`]
//! with an [`Ask`]; a shell opens it on the board for the client that ran
//! the line ([`open`]); one-shot paths (UDP, HTTP, `--cmd`) refuse it and
//! name the flag ([`Ask::refusal`]). Each client's `Status` carries the
//! dialogs meant for it ([`for_client`]) — the line's device or client, a
//! claim ask's owner, the station's merge decision. An answer closes the
//! dialog on every client first, then the button's command line runs as the
//! answering user ([`answer`]). A dialog without a countdown waits.
//!
//! [`DispatchResult::Ask`]: crate::DispatchResult::Ask

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use lumacore_wire::{
    ButtonKind, Countdown, Dialog, DialogAnswer, DialogButton, DialogClosed, DialogInput,
    DialogRow, DialogSelector, DialogToggle, InputKind, Tone,
};
use uuid::Uuid;

use crate::AppState;

/// How long an answered dialog's notice stays in the statuses.
const CLOSED_FOR: Duration = Duration::from_secs(4);
/// A dialog whose client is gone this long counts as cancelled (a reload
/// comes back sooner and finds it still open).
const GONE_AFTER: Duration = Duration::from_secs(30);

use lumacore_base::clock::now_ms;

mod topic;
pub use topic::{close_topic, holds_topic};

// ── Asking ──────────────────────────────────────────────────────────

/// A question a command asks: the dialog as clients see it, plus what each
/// answer does (command-line templates filled from the inputs by name).
#[derive(Debug, Clone)]
pub struct Ask {
    view: Dialog,
    /// Per button: the line it runs (`{name}` = an input, toggle or
    /// selector); None = only close (Cancel).
    lines: Vec<Option<String>>,
    /// Per button: the line it runs instead when every input is empty (an
    /// empty Label clears it: `Label Group 4 /Clear`).
    empty_lines: Vec<Option<String>>,
    /// Per toggle: the text `{name}` becomes when it is on.
    toggle_flags: Vec<String>,
    /// Per selector: the text `{name}` becomes for each choice.
    selector_texts: Vec<Vec<String>>,
    /// Input values that stand for other text: (input, value, text) — a pick
    /// list row "Own — a copy of Default" (no 0) becomes `New`, a file row
    /// its file name. Such text may name other inputs (`{name}`).
    value_texts: Vec<(String, String, String)>,
    countdown_s: Option<(u32, usize)>,
    /// What a one-shot command adds instead of answering (`/NoConfirmation`).
    flags: String,
}

/// How long an owner has to answer a takeover (the countdown).
pub const TAKEOVER: std::time::Duration = std::time::Duration::from_secs(10);

impl Ask {
    pub fn new(title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            view: Dialog {
                title: title.into(),
                text: text.into(),
                ..Dialog::default()
            },
            lines: Vec::new(),
            empty_lines: Vec::new(),
            toggle_flags: Vec::new(),
            selector_texts: Vec::new(),
            value_texts: Vec::new(),
            countdown_s: None,
            flags: "/NoConfirmation".to_string(),
        }
    }

    pub fn meta(mut self, text: impl Into<String>, tone: Option<Tone>) -> Self {
        self.view.meta = Some(text.into());
        self.view.meta_tone = tone;
        self
    }

    pub fn row(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.view.rows.push(DialogRow { k: k.into(), v: v.into(), tone: None });
        self
    }

    pub fn detail(mut self, k: impl Into<String>, tone: Option<Tone>) -> Self {
        self.view.details.push(DialogRow { k: k.into(), v: String::new(), tone });
        self
    }

    /// A file of the library to pick (`{name}` = `"file" [/Drive "usb1"]`): the kit's files
    /// element on the kind's folder (decisions §38 5, the station's "which file?" ask).
    pub fn file(self, name: &str, label: &str, kind: &str) -> Self {
        self.input(name, label, "", InputKind::File { files: kind.to_string() })
    }

    pub fn input(mut self, name: &str, label: &str, value: impl Into<String>, kind: InputKind) -> Self {
        self.view.inputs.push(DialogInput {
            name: name.to_string(),
            label: label.to_string(),
            value: value.into(),
            kind,
        });
        self
    }

    /// A switch; `{name}` in a line becomes `flag` when it is on, else "".
    pub fn toggle(mut self, name: &str, label: &str, on: bool, flag: &str) -> Self {
        self.view.toggles.push(DialogToggle {
            name: name.to_string(),
            label: label.to_string(),
            on,
        });
        self.toggle_flags.push(flag.to_string());
        self
    }

    /// A choice of `(label, text)`; `{name}` in a line becomes the text.
    pub fn selector(mut self, name: &str, label: &str, choices: &[(&str, &str)], value: &str) -> Self {
        self.view.selectors.push(DialogSelector {
            name: name.to_string(),
            label: label.to_string(),
            choices: choices.iter().map(|(l, _)| l.to_string()).collect(),
            value: value.to_string(),
        });
        self.selector_texts
            .push(choices.iter().map(|(_, t)| t.to_string()).collect());
        self
    }

    /// When input `name` holds `value`, `{name}` becomes `text` (taken as it
    /// is; it may name other inputs).
    pub fn value_text(mut self, name: &str, value: impl Into<String>, text: impl Into<String>) -> Self {
        self.value_texts.push((name.to_string(), value.into(), text.into()));
        self
    }

    /// A button running `line`. The first Primary / Please button is the
    /// one Enter presses.
    pub fn button(mut self, label: &str, kind: ButtonKind, line: impl Into<String>) -> Self {
        let line = line.into();
        let first_primary = !self
            .view
            .buttons
            .iter()
            .any(|b| matches!(b.kind, ButtonKind::Primary | ButtonKind::Please));
        if first_primary && matches!(kind, ButtonKind::Primary | ButtonKind::Please) {
            self.view.primary = self.view.buttons.len();
        }
        self.view.buttons.push(DialogButton {
            label: label.to_string(),
            kind,
            disabled: None,
        });
        self.lines.push(Some(line));
        self.empty_lines.push(None);
        self
    }

    /// The last button runs `line` instead when every input is empty.
    pub fn or_when_empty(mut self, line: impl Into<String>) -> Self {
        if let Some(slot) = self.empty_lines.last_mut() {
            *slot = Some(line.into());
        }
        self
    }

    /// The last button is greyed out, with the reason.
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        if let Some(b) = self.view.buttons.last_mut() {
            b.disabled = Some(reason.into());
        }
        self
    }

    /// A button that only closes the dialog (Esc presses it).
    pub fn cancel(mut self, label: &str) -> Self {
        self.view.cancel = Some(self.view.buttons.len());
        self.view.buttons.push(DialogButton {
            label: label.to_string(),
            kind: ButtonKind::Plain,
            disabled: None,
        });
        self.lines.push(None);
        self.empty_lines.push(None);
        self
    }

    /// The station presses button `default` after `seconds`.
    pub fn countdown(mut self, seconds: u32, default: usize) -> Self {
        self.countdown_s = Some((seconds, default));
        self
    }

    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.view.topic = Some(topic.into());
        self
    }

    pub fn note(mut self, text: impl Into<String>) -> Self {
        self.view.note = Some(text.into());
        self
    }

    /// The flags a one-shot command adds instead of answering.
    pub fn flags(mut self, flags: impl Into<String>) -> Self {
        self.flags = flags.into();
        self
    }

    pub fn title(&self) -> &str {
        &self.view.title
    }

    /// The dialog as clients see it (without an id).
    pub fn view(&self) -> &Dialog {
        &self.view
    }

    /// For a path that can't answer (UDP, HTTP, `--cmd`): what it asks and
    /// which flag answers it.
    pub fn refusal(&self) -> String {
        let text = self.view.text.lines().next().unwrap_or_default();
        crate::errors::render_code(
            format!(
                "{}: {text} This line asks a question a one-shot command can't answer — add {}.",
                self.view.title, self.flags
            ),
            "E-CMD-010",
        )
    }

    /// The line the primary button runs with the shown values (the sweep
    /// answers every question this way).
    /// The rest of a line that waits for this answer (a statement in the
    /// middle of `a; b; c` asked): every answering button runs it after
    /// its own line.
    pub fn then_run(mut self, rest: &str) -> Self {
        let add = |l: &mut Option<String>| {
            if let Some(line) = l {
                line.push_str("; ");
                line.push_str(rest);
            }
        };
        self.lines.iter_mut().for_each(add);
        self.empty_lines.iter_mut().for_each(add);
        self
    }

    /// Rewrite every button's line (an owner-first line puts its owner back
    /// into the lines its handler asked with).
    pub fn map_lines(mut self, f: impl Fn(&str) -> String) -> Self {
        let map = |l: &mut Option<String>| {
            if let Some(line) = l {
                *line = f(line);
            }
        };
        self.lines.iter_mut().for_each(map);
        self.empty_lines.iter_mut().for_each(map);
        self
    }

    pub fn primary_line(&self) -> Option<String> {
        self.line_for(self.view.primary, &DialogAnswer::default()).ok().flatten()
    }

    /// The line the button labelled `label` runs with the shown values.
    /// Every line a button may run (its line and its empty-input line).
    pub fn button_lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().chain(self.empty_lines.iter()).flatten().map(String::as_str)
    }

    pub fn line_of(&self, label: &str) -> Option<String> {
        let i = self.view.buttons.iter().position(|b| b.label == label)?;
        self.line_for(i, &DialogAnswer::default()).ok().flatten()
    }

    /// The line button `i` runs with `answer`'s values, or None (Cancel).
    pub fn line_for(&self, i: usize, answer: &DialogAnswer) -> Result<Option<String>, String> {
        let Some(template) = self.lines.get(i).ok_or("No such button.")? else {
            return Ok(None);
        };
        let all_empty = !self.view.inputs.is_empty()
            && self.view.inputs.iter().all(|input| {
                answer
                    .inputs
                    .get(&input.name)
                    .unwrap_or(&input.value)
                    .trim()
                    .is_empty()
            });
        if all_empty && let Some(Some(line)) = self.empty_lines.get(i) {
            return Ok(Some(line.clone()));
        }
        let mut line = template.clone();
        // Selectors first: a choice's text may be a whole line naming inputs
        // (New profile: From Default / Copy of… / Import file…).
        for (s, texts) in self.view.selectors.iter().zip(&self.selector_texts) {
            let chosen = answer.selectors.get(&s.name).unwrap_or(&s.value);
            let text = s
                .choices
                .iter()
                .position(|c| c == chosen)
                .and_then(|i| texts.get(i))
                .ok_or_else(|| format!("{} isn't a choice of {}.", chosen, s.label))?;
            line = line.replace(&format!("{{{}}}", s.name), text);
        }
        // Values that stand for text next: their text may name other inputs.
        for input in &self.view.inputs {
            let raw = answer.inputs.get(&input.name).unwrap_or(&input.value);
            if let Some((_, _, text)) = self
                .value_texts
                .iter()
                .find(|(n, v, _)| *n == input.name && v.trim() == raw.trim())
            {
                line = line.replace(&format!("{{{}}}", input.name), text);
            }
        }
        for input in &self.view.inputs {
            let raw = answer.inputs.get(&input.name).unwrap_or(&input.value);
            let value = clean_input(&input.kind, raw)?;
            line = line.replace(&format!("{{{}}}", input.name), &value);
        }
        for (t, flag) in self.view.toggles.iter().zip(&self.toggle_flags) {
            let on = answer.toggles.get(&t.name).copied().unwrap_or(t.on);
            line = line.replace(&format!("{{{}}}", t.name), if on { flag } else { "" });
        }
        Ok(Some(line.split_whitespace().collect::<Vec<_>>().join(" ")))
    }
}

/// An input's value made safe for a command line: text loses its quotes and
/// statement separators; a number is digits and `+ - * / .` only (the
/// station works the expression out, decisions §30 "12b").
fn clean_input(kind: &InputKind, raw: &str) -> Result<String, String> {
    match kind {
        InputKind::Number { .. } => {
            let ok = raw
                .chars()
                .all(|c| c.is_ascii_digit() || " .+-*/".contains(c));
            if ok && !raw.trim().is_empty() {
                Ok(raw.trim().to_string())
            } else {
                Err(crate::errors::render_code(
                    format!("\"{raw}\" isn't a number."),
                    "E-CMD-008",
                ))
            }
        }
        // A file pick: `<drive>:<file>` → `"file"`, plus the drive off the local library
        // (decisions §38 5). A file name never holds `:` (`drive::library_file`).
        InputKind::File { .. } => {
            let raw = raw.trim().replace(['"', ';'], "");
            let (drive, file) = raw.split_once(':').unwrap_or((crate::drives::LOCAL, raw.as_str()));
            if file.trim().is_empty() {
                return Err(crate::errors::render_code("Which file?", "E-CMD-006"));
            }
            Ok(if drive.eq_ignore_ascii_case(crate::drives::LOCAL) || drive.is_empty() {
                format!("\"{file}\"")
            } else {
                format!("\"{file}\" /Drive \"{drive}\"")
            })
        }
        _ => Ok(raw.replace(['"', ';'], "")),
    }
}

// ── The board ───────────────────────────────────────────────────────

/// Who a dialog is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Audience {
    /// Every window of an app, or one browser tab (decisions §30 "12a3").
    Device(String),
    /// A shell client without a device (the CLI).
    Client(Uuid),
    /// The station's own console.
    Local,
    /// Every client of a user.
    User(String),
    /// Every client of the station (the merge decision).
    Station,
}

/// A client asking what it should see: its user, device and shell client.
#[derive(Debug, Clone, Default)]
pub struct Who {
    pub user: String,
    pub device: Option<String>,
    pub client: Option<Uuid>,
}

impl Who {
    /// The client the state is executing for.
    pub fn current(state: &AppState) -> Self {
        Self {
            user: state.effective_user_name().to_string(),
            device: state.current_device.clone(),
            client: state.current_client,
        }
    }

    fn sees(&self, to: &Audience) -> bool {
        match to {
            Audience::Device(d) => self.device.as_deref() == Some(d.as_str()),
            Audience::Client(c) => self.device.is_none() && self.client == Some(*c),
            Audience::Local => self.device.is_none() && self.client.is_none(),
            Audience::User(u) => self.user.eq_ignore_ascii_case(u),
            Audience::Station => true,
        }
    }

    fn label(&self) -> String {
        self.user.clone()
    }
}

/// A board dialog is seen by its audience while the user is the one who
/// was asked (another user logging in on the same device doesn't get it).
fn may_see(who: &Who, o: &Open) -> bool {
    who.sees(&o.to)
        && (matches!(o.to, Audience::User(_) | Audience::Station)
            || who.user.eq_ignore_ascii_case(&o.user))
}

#[derive(Debug, Clone)]
struct Open {
    ask: Ask,
    to: Audience,
    user: String,
    deadline_ms: Option<u64>,
    paused: bool,
    /// Since when no client of `to` was attached (see [`forget_gone`]).
    gone_since: Option<Instant>,
}

/// The station's open dialogs (station-local, never replicated or saved).
#[derive(Debug, Default)]
pub struct Board {
    open: BTreeMap<u64, Open>,
    next: u64,
    closed: Vec<(Audience, DialogClosed, Instant)>,
}

impl Board {
    pub fn is_empty(&self) -> bool {
        self.open.is_empty()
    }

    fn close(&mut self, to: Audience, notice: DialogClosed) {
        let now = Instant::now();
        self.closed.retain(|(_, _, at)| now.duration_since(*at) < CLOSED_FOR);
        self.closed.push((to, notice, now));
    }
}

/// Open `ask` for the client the state executes for (a shell, a UI client
/// or the station's console). The reply is empty: the dialog shows it.
pub fn open(state: &mut AppState, ask: Ask) -> crate::DispatchResult {
    let to = match (&state.current_device, state.current_client) {
        (Some(d), _) => Audience::Device(d.clone()),
        (None, Some(c)) => Audience::Client(c),
        (None, None) => Audience::Local,
    };
    let user = state.effective_user_name().to_string();
    open_for(state, ask, to, &user);
    crate::DispatchResult::Output((String::new()).into())
}

/// Open `ask` for `to`, answered by default as `user` when it times out.
pub fn open_for(state: &mut AppState, ask: Ask, to: Audience, user: &str) -> String {
    let board = &mut state.dialogs;
    board.next += 1;
    let id = board.next;
    let deadline_ms = ask
        .countdown_s
        .map(|(s, _)| now_ms() + u64::from(s) * 1000);
    board.open.insert(
        id,
        Open {
            ask,
            to,
            user: user.to_string(),
            deadline_ms,
            paused: false,
            gone_since: None,
        },
    );
    format!("d{id}")
}

/// Every dialog `who` should see, oldest first, and the notices of ones it
/// had that were just answered.
pub fn for_client(state: &AppState, who: &Who) -> (Vec<Dialog>, Vec<DialogClosed>) {
    let mut out = Vec::new();
    // The station's merge decision (§21 "6c" / "6d"): every client.
    if let Some(d) = &state.decision {
        out.push(merge_view(d));
    }
    // Someone asks for one of this user's objects (§21 "6d").
    for a in &state.root.runtime.claim_asks {
        if a.owner.eq_ignore_ascii_case(&who.user) {
            out.push(claim_view(a));
        }
    }
    for (id, o) in &state.dialogs.open {
        if may_see(who, o) {
            let mut view = o.ask.view.clone();
            view.id = format!("d{id}");
            view.countdown = o.ask.countdown_s.map(|(total_s, default)| Countdown {
                deadline_ms: o.deadline_ms.unwrap_or(0),
                total_s,
                default,
                paused: o.paused,
            });
            out.push(view);
        }
    }
    // A login's Meow License comes before anything else it may see (decisions 0290).
    out.sort_by_key(|d| d.topic.as_deref() != Some(crate::consent::TOPIC));
    let now = Instant::now();
    let closed = state
        .dialogs
        .closed
        .iter()
        .filter(|(to, _, at)| now.duration_since(*at) < CLOSED_FOR && who.sees(to))
        .map(|(_, n, _)| n.clone())
        .collect();
    (out, closed)
}

fn merge_view(d: &lumacore_wire::Decision) -> Dialog {
    let buttons = d
        .options
        .iter()
        .map(|(label, cmd)| DialogButton {
            label: label.clone(),
            kind: match label.as_str() {
                l if l == d.default => ButtonKind::Primary,
                "Use Mine" => ButtonKind::Danger,
                _ => ButtonKind::Plain,
            },
            // Cancel without a line only closes the question.
            disabled: (cmd.is_empty() && label != "Cancel").then(|| {
                d.merge_blocked
                    .clone()
                    .unwrap_or_else(|| "not possible now".to_string())
            }),
        })
        .collect();
    let default = d
        .options
        .iter()
        .position(|(l, _)| *l == d.default)
        .unwrap_or(0);
    Dialog {
        id: "merge".to_string(),
        title: d.title.clone(),
        meta: d.merge_blocked.as_ref().map(|_| "Merge blocked".to_string()),
        meta_tone: d.merge_blocked.as_ref().map(|_| Tone::Warn),
        text: d.message.clone(),
        details: d
            .details
            .iter()
            .map(|line| DialogRow {
                k: line.clone(),
                v: String::new(),
                tone: detail_tone(line),
            })
            .collect(),
        buttons,
        primary: default,
        cancel: d.options.iter().position(|(l, _)| l == "Cancel"),
        countdown: d.deadline_ms.map(|deadline_ms| Countdown {
            deadline_ms,
            total_s: 0,
            default,
            paused: d.paused,
        }),
        ..Dialog::default()
    }
}

/// A merge difference's colour from its wording (§21 "6d" previews).
fn detail_tone(line: &str) -> Option<Tone> {
    let l = line.to_ascii_lowercase();
    if l.contains("both") || l.contains("blocks") {
        Some(Tone::Warn)
    } else if l.contains("delet") || l.contains("removed") {
        Some(Tone::Alert)
    } else if l.contains("new") || l.contains("added") {
        Some(Tone::Ok)
    } else {
        None
    }
}

fn claim_view(a: &lumacore_tree::ClaimAsk) -> Dialog {
    Dialog {
        id: format!("claim:{}", a.object),
        title: format!("{} asks for {}", a.from, a.label),
        text: format!("{} wants to change {}, which you have claimed.", a.from, a.label),
        toggles: vec![DialogToggle {
            name: "remember".to_string(),
            label: "Remember for this session".to_string(),
            on: false,
        }],
        buttons: vec![
            DialogButton {
                label: "Hand over".to_string(),
                kind: ButtonKind::Primary,
                disabled: None,
            },
            DialogButton {
                label: "Keep".to_string(),
                kind: ButtonKind::Plain,
                disabled: None,
            },
        ],
        primary: 0,
        cancel: Some(1),
        countdown: Some(Countdown {
            deadline_ms: a.deadline_ms,
            total_s: TAKEOVER.as_secs() as u32,
            default: 0,
            paused: false,
        }),
        ..Dialog::default()
    }
}

/// `who` answers dialog `answer.id`: it closes on every client first, then
/// the returned line (None = Cancel, nothing to run) runs as `who`.
pub fn answer(state: &mut AppState, who: &Who, answer: &DialogAnswer) -> Result<Option<String>, String> {
    let gone = || {
        crate::errors::render_code(
            "That question was already answered or has ended.",
            "E-CMD-010",
        )
    };
    if answer.id == "merge" {
        let d = state.decision.as_ref().ok_or_else(gone)?;
        let (label, line) = d.options.get(answer.button).cloned().ok_or_else(gone)?;
        if line.is_empty() && label != "Cancel" {
            return Err(crate::errors::render_code(
                d.merge_blocked.clone().unwrap_or_else(|| format!("{label} isn't possible now.")),
                "E-CMD-010",
            ));
        }
        let title = d.title.clone();
        // An answer always closes the decision (R1 review 19), whatever
        // its line then says: a refused line never leaves it hanging.
        state.decision = None;
        state.dialogs.close(
            Audience::Station,
            DialogClosed { id: answer.id.clone(), title, by: who.label(), answer: label },
        );
        return Ok((!line.is_empty()).then_some(line));
    }
    if let Some(guid) = answer.id.strip_prefix("claim:") {
        let ask = state
            .root
            .runtime
            .claim_asks
            .iter()
            .find(|a| a.object.to_string() == guid && a.owner.eq_ignore_ascii_case(&who.user))
            .cloned()
            .ok_or_else(gone)?;
        let remember = answer.toggles.get("remember").copied().unwrap_or(false);
        let flag = if remember { " /Remember" } else { "" };
        let (label, line) = match answer.button {
            0 => ("Hand over", format!("Unclaim {}{flag}", ask.label)),
            _ => ("Keep", format!("Claim {} /Keep{flag}", ask.label)),
        };
        state.dialogs.close(
            Audience::User(who.user.clone()),
            DialogClosed {
                id: answer.id.clone(),
                title: format!("{} asks for {}", ask.from, ask.label),
                by: who.label(),
                answer: label.to_string(),
            },
        );
        return Ok(Some(line));
    }
    let id: u64 = answer
        .id
        .strip_prefix('d')
        .and_then(|n| n.parse().ok())
        .ok_or_else(gone)?;
    let open = state.dialogs.open.get(&id).ok_or_else(gone)?;
    if !may_see(who, open) {
        return Err(gone());
    }
    if let Some(reason) = open
        .ask
        .view
        .buttons
        .get(answer.button)
        .and_then(|b| b.disabled.clone())
    {
        return Err(crate::errors::render_code(&reason, "E-CMD-010"));
    }
    // Check the inputs before closing: a bad number keeps the dialog open.
    let line = open.ask.line_for(answer.button, answer)?;
    let open = state.dialogs.open.remove(&id).ok_or_else(gone)?;
    let label = open.ask.view.buttons[answer.button].label.clone();
    state.dialogs.close(
        open.to,
        DialogClosed {
            id: answer.id.clone(),
            title: open.ask.view.title.clone(),
            by: who.label(),
            answer: label,
        },
    );
    Ok(line)
}

/// A touch on a dialog with a countdown stops it. The merge decision pauses
/// through its own word (returned as the line to run).
pub fn pause(state: &mut AppState, who: &Who, id: &str) -> Option<String> {
    if id == "merge" {
        return state
            .decision
            .as_ref()
            .filter(|d| d.deadline_ms.is_some() && !d.paused)
            .map(|_| "Pause LumaSession".to_string());
    }
    let n: u64 = id.strip_prefix('d')?.parse().ok()?;
    if let Some(o) = state.dialogs.open.get_mut(&n)
        && may_see(who, o)
    {
        o.paused = true;
    }
    None
}

/// A dialog whose countdown ended: the station presses its default button.
#[derive(Debug, Clone)]
pub struct Due {
    pub to: Audience,
    pub user: String,
    pub line: Option<String>,
}

/// Take every board dialog whose countdown ended (housekeeping, ~50 ms).
pub fn take_due(state: &mut AppState) -> Vec<Due> {
    let now = now_ms();
    let ids: Vec<u64> = state
        .dialogs
        .open
        .iter()
        .filter(|(_, o)| !o.paused && o.deadline_ms.is_some_and(|d| d <= now))
        .map(|(id, _)| *id)
        .collect();
    let mut due = Vec::new();
    for id in ids {
        let Some(o) = state.dialogs.open.remove(&id) else {
            continue;
        };
        let default = o.ask.countdown_s.map_or(0, |(_, d)| d);
        let line = o.ask.line_for(default, &DialogAnswer::default()).ok().flatten();
        state.dialogs.close(
            o.to.clone(),
            DialogClosed {
                id: format!("d{id}"),
                title: o.ask.view.title.clone(),
                by: "time".to_string(),
                answer: o.ask.view.buttons[default].label.clone(),
            },
        );
        due.push(Due { to: o.to, user: o.user, line });
    }
    due
}

/// Drop dialogs whose client has been gone [`GONE_AFTER`] (logged out,
/// closed): they count as cancelled. `attached` says whether a client of an
/// audience is attached now.
pub fn forget_gone(state: &mut AppState, attached: impl Fn(&Audience) -> bool) {
    let now = Instant::now();
    state.dialogs.open.retain(|_, o| {
        if matches!(o.to, Audience::Local | Audience::Station) || attached(&o.to) {
            o.gone_since = None;
            return true;
        }
        let since = *o.gone_since.get_or_insert(now);
        now.duration_since(since) < GONE_AFTER
    });
}

