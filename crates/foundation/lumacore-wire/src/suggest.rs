//! Suggestions for a command line (decisions §30 "12b2 hardening"): one
//! query / reply on the shell socket, the same typed items for the bottom
//! bar's list, the full-screen palette and the TUI. The station ranks; a
//! client only shows them and, on a pick, writes the command line (or runs an
//! item action's command line) — picking never runs code in the client.

use serde::{Deserialize, Serialize};

/// Where the list is shown: above the bottom command line bar, or the
/// full-screen palette (wider rows, sections, item actions).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    #[default]
    Bar,
    Palette,
}

/// What an item is. The order is the fixed tie-break of the ranking
/// (after the fuzzy score and usage).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// A line the user ran before that starts with what is typed.
    History,
    /// A command word (`Store`, `At`, `Go`).
    Verb,
    /// An object keyword (`Group`, `Sequence`) or another word of the grammar.
    Keyword,
    /// An object of the show (`Group 3 "Even"`).
    Object,
    /// A value (`Full`, `50`, an attribute name).
    Value,
    /// A node of the object tree (`ShowData.`).
    TreeNode,
    /// An option (`/Overwrite`, `--type`).
    Flag,
}

/// Whether the word works yet (decisions §29: planned words are discoverable).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    #[default]
    Ready,
    /// Published, not built: running it replies E-CMD-003.
    Planned,
}

/// Another thing an item can do, as a command line (the palette's
/// Meow+Enter and action list).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemAction {
    pub label: String,
    pub line: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuggestItem {
    pub kind: ItemKind,
    pub title: String,
    pub subtitle: String,
    /// Icon name from the UI kit's sprite.
    pub icon: String,
    /// The text that replaces [`Suggestions::replace`] when it is taken.
    pub insert: String,
    /// The object's GUID, when the item is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    /// What else it can do (palette), as command lines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<ItemAction>,
    #[serde(default)]
    pub status: ItemStatus,
    /// The acting user may not run it: the error code (E-RIGHTS-001); the
    /// subtitle says why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<String>,
    /// The section the palette lists it under (`Objects`, `History` …).
    pub section: String,
    /// Its rank on the station (higher first) — for tests and debugging; the
    /// list is already in order.
    pub score: i64,
}

/// The station's answer to a suggest query.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Suggestions {
    /// The query's sequence number; a client drops a reply older than its
    /// newest query.
    pub seq: u64,
    pub surface: Surface,
    /// The line and cursor (chars) the items were made for.
    pub line: String,
    pub cursor: u32,
    /// The span of `line` (chars, start..end) an item's `insert` replaces.
    pub replace: [u32; 2],
    /// Grey ghost text after the cursor: the top history match, else the
    /// first completion that extends the word.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// The syntax of the word the line starts with (`Store <object> …`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
    pub items: Vec<SuggestItem>,
    /// How many matched before the list was cut.
    pub total: u32,
    /// The user's profile setting: `Auto`, `TabOnly` or `Off`.
    pub mode: String,
}
