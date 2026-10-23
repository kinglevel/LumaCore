//! The standard dialog (decisions §30 "12b hardening", the usual message box shape):
//! every question a station asks — a confirm, a text prompt, a claim ask, the
//! merge decision — is one [`Dialog`] held by the station and sent in each
//! client's [`crate::Status`] that should see it. A client answers with
//! [`crate::ShellClientPacket::DialogAnswer`]; the station closes the dialog
//! on every client first, then runs the button's command line as that user.
//! Answers are named, never positions in a list of values.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Dialog {
    /// The station's id for it (`d7`, `claim:<guid>`, `merge`).
    pub id: String,
    pub title: String,
    /// Small text right of the title ("same GUID", "E-WIRE-001").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_tone: Option<Tone>,
    pub text: String,
    /// Facts as label / value rows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<DialogRow>,
    /// A list (e.g. the merge's differences), coloured by tone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<DialogRow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<DialogInput>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub toggles: Vec<DialogToggle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selectors: Vec<DialogSelector>,
    /// A tree to tick (decisions 0293, the Update ask): drawn by the kit's tree in pick mode,
    /// each tickable node answered like a toggle (by its `id`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<DialogTree>,
    pub buttons: Vec<DialogButton>,
    /// The button Enter presses.
    #[serde(default)]
    pub primary: usize,
    /// The button Esc presses (None: Esc does nothing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel: Option<usize>,
    /// Answered by the station at `deadline_ms` (unix ms) with `default`,
    /// unless paused. Clients only show it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub countdown: Option<Countdown>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// What it is about, for a client's own follow-up (`power:Shutdown`:
    /// the app hears a Cancel of its quit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Countdown {
    pub deadline_ms: u64,
    /// Seconds it started with (for a bar).
    pub total_s: u32,
    /// The button the station presses at the deadline.
    pub default: usize,
    #[serde(default)]
    pub paused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Dim,
    Info,
    Ok,
    Warn,
    Alert,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogRow {
    pub k: String,
    #[serde(default)]
    pub v: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogInput {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub value: String,
    pub kind: InputKind,
}

/// What edits an input: the keyboard, the keypad or the pick list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InputKind {
    Text,
    /// Masked; never echoed.
    Password,
    /// The keypad: range, unit, relative input (`+ 3`) worked out by the
    /// station.
    Number {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit: Option<String>,
    },
    /// The pick list.
    Link { pick: Pick },
    /// A file of the library (decisions §38 5): the kit's `<luma-files>` with the kind's folder
    /// on the drives rail; the value is `<drive>:<file>` (`usb1:Robe/Spiider.gdtf`), rendered
    /// into the line as `"file" /Drive "usb1"` by the station.
    File { files: String },
}

/// What a link can point at (the pick list, decisions §30 "12b", one
/// shared pop-up): filter, None for an optional link, the items, Add New.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pick {
    /// The object kind (`Network`, `User`, `Image`).
    pub kind: String,
    pub items: Vec<PickItem>,
    /// None is offered (an optional link).
    #[serde(default)]
    pub optional: bool,
    /// A first row that follows the user's selection (`Selected DataPool`,
    /// decisions §30 "12b2"); picking it fills `{value}` with `selected`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow: Option<String>,
    /// The link points at something gone: shown as a top row until changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing: Option<String>,
    /// Add New: the line that stores a new one (`Store Network 4`) — the
    /// pick list then picks `add_no`. None = the user can't add one here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add_no: Option<u32>,
    /// The page answers with the picked item's GUID, not its number (a tag: its number is a
    /// path, decisions §12 "Tags hardened").
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub by_guid: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PickItem {
    pub no: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    #[serde(default)]
    pub guid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogToggle {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub on: bool,
    /// The text `{name}` becomes in a line when it is on (a client's preview of the lines).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub flag: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogSelector {
    pub name: String,
    pub label: String,
    pub choices: Vec<String>,
    #[serde(default)]
    pub value: String,
    /// The text `{name}` becomes for each choice (a client's preview of the lines).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub texts: Vec<String>,
}

/// A tree in a dialog (decisions 0293): nodes in order, each at its depth. A tickable node is
/// answered like a toggle (`DialogAnswer.toggles[id]`); what it adds to the answer's line is its
/// `on` or `off` statements. The station builds the line; a client builds the same list only to
/// preview it ([`DialogTree::statements`]).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DialogTree {
    pub nodes: Vec<DialogTreeNode>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DialogTreeNode {
    /// Its name in the answer (unique in the dialog).
    pub id: String,
    pub depth: u32,
    /// What it is (`preset`, `cue`, `plaincue`, `dir`, `loop`): the client's icon.
    pub kind: String,
    /// Its number as shown (`P3`, `Cue 2`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub no: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// A short badge (`S` / `G` / `U`, `Seq 1`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub badge: String,
    /// The small text on the right while ticked (`3 cues`, `follows`) and while not
    /// (`keeps its old look`; empty = the same).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note_off: String,
    #[serde(default)]
    pub on: bool,
    /// Whether it can be ticked (a heading or a circular reference can't).
    #[serde(default)]
    pub tickable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
    /// The statements it adds while ticked / not ticked.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub when_on: Vec<TreeStatement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub when_off: Vec<TreeStatement>,
}

/// A statement a tree node adds to the answer's line: statements run lane by lane (lane 0 first),
/// in tree order within a lane. A statement that starts with `{?name}` runs only while the
/// toggle or node `name` is on; `{?name!=text}` only while the selector `name` isn't on the
/// choice whose text is `text`. Guards stack.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TreeStatement {
    #[serde(default)]
    pub lane: u8,
    pub text: String,
}

impl DialogTree {
    /// The statements the tree adds with these ticks (`on(id)`: a node's or a toggle's state;
    /// `text(name)`: a selector's chosen text), lane by lane, the guards worked out; placeholders
    /// (`{content}`) stay for the dialog's line to fill.
    pub fn statements(&self, on: impl Fn(&str) -> Option<bool>, text: impl Fn(&str) -> Option<String>) -> Vec<String> {
        let state = |id: &str| match id.split_once("!=") {
            Some((name, not)) => text(name).is_some_and(|t| t != not),
            None => on(id).or_else(|| self.nodes.iter().find(|n| n.id == id).map(|n| n.on)).unwrap_or(false),
        };
        let mut picked: Vec<(u8, usize, String)> = Vec::new();
        for (i, n) in self.nodes.iter().enumerate() {
            let list = if !n.tickable || state(&n.id) { &n.when_on } else { &n.when_off };
            for s in list {
                let mut text = s.text.as_str();
                let mut keep = true;
                while let Some(rest) = text.strip_prefix("{?") {
                    let Some((name, after)) = rest.split_once('}') else { break };
                    keep &= state(name);
                    text = after;
                }
                if keep && !text.trim().is_empty() {
                    picked.push((s.lane, i, text.trim().to_string()));
                }
            }
        }
        picked.sort_by_key(|(lane, i, _)| (*lane, *i));
        picked.into_iter().map(|(_, _, t)| t).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogButton {
    pub label: String,
    #[serde(default)]
    pub kind: ButtonKind,
    /// Greyed out, with the reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonKind {
    #[default]
    Plain,
    Primary,
    Danger,
    /// Green, like the keypad's Please.
    Please,
}

/// A dialog this client saw was answered (here or elsewhere): shown briefly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogClosed {
    pub id: String,
    pub title: String,
    /// Who answered ("Bob"); "time" when the countdown ran out.
    pub by: String,
    /// The button's label.
    pub answer: String,
}

/// A client's answer: the button and everything entered, by name.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DialogAnswer {
    pub id: String,
    pub button: usize,
    #[serde(default)]
    pub inputs: BTreeMap<String, String>,
    #[serde(default)]
    pub toggles: BTreeMap<String, bool>,
    #[serde(default)]
    pub selectors: BTreeMap<String, String>,
}
