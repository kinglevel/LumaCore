// ██╗   ██╗███████╗██████╗ ██████╗
// ██║   ██║██╔════╝██╔══██╗██╔══██╗
// ██║   ██║█████╗  ██████╔╝██████╔╝
// ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══██╗
//  ╚████╔╝ ███████╗██║  ██║██████╔╝
//   ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═════╝
// FILE: verb
//! What can be done to an object (decisions §15, §33): the verbs, one namespace with the commands.
//! Pure data, so every crate's rows can name them (decisions §34 21).
//!
//! Owns:          Verb and its label
//! Doesn't own:   which object takes which verb (each object's row), the handlers (dispatch)
//! Entry points:  Verb, Verb::label
//! Decisions:     §15, §33, §34 21

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What can be done to an object. A bare object runs its [`ObjectDef::default_verb`]
/// (`Group 3` = `SelectFixtures Group 3`, decisions §33 6, 40–48); `Call` is the verb that
/// brings a stored thing back (`Call View 3`, decision 35).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Call,
    /// Select fixtures (`SelectFixtures Group 3`, decision 26).
    SelectFixtures,
    /// Make it your selected one (`Select Sequence 3`, `Select Window 1.4`).
    Select,
    /// `Goto Cue 3`.
    Goto,
    /// `Go Macro 2`.
    Go,
    /// `<target> At <value>` (decision 43): `Fixture 1 At 50`, `Master 2.1 At 80`.
    At,
    Store,
    Delete,
    Label,
    Assign,
    Set,
    Edit,
    Copy,
    Move,
    Update,
    /// Remove a route (`Unassign Universe 1 At Output 2`).
    Unassign,
    List,
    Info,
    Dump,
    /// Switch on / knock in (`On Blind`, `On Fixture 2`).
    On,
    /// Switch off / knock out (`Off Blind`, `Off Fixture 2`).
    Off,
    /// On or Off, from the object's state (`Toggle Freeze`).
    Toggle,
    /// `Join LumaSession "Festival"`.
    Join,
    /// `Leave LumaSession`.
    Leave,
    /// `Invite Station "FOH-2"`.
    Invite,
    /// `Dismiss Station "FOH-2"`.
    Dismiss,
    /// `Claim LumaSession`: take the master role (§21 "6c").
    Claim,
    /// `Pause LumaSession`: stop a decision's countdown (§21 "6d").
    Pause,
    /// `Import Image "stage.jpg"`, `Import UserProfile "Bob"` (§30 "12a2").
    Import,
    /// `Export UserProfile "Bob"`, `Export Image 3`.
    Export,
    /// Back to the defaults (`Reset Self`, `Reset Window 1.4`; decisions §30
    /// "12c"). A bare `Reset` is the programmer's four-stage reset.
    Reset,
    /// One-shot reorder (`Sort FeatureGroup "Beam" /By Family`, decisions §33 88).
    Sort,
    /// One-shot split into groups (`Split FeatureGroup "Effects" /By Family`, §33 88).
    Split,
    /// Put objects aside to be moved by `Paste` (`Cut Group 1`, decisions §35 14).
    Cut,
    /// Finish a `Copy` / `Cut`: the target is where they go (`Paste Group 10`, §35 13).
    Paste,
    /// A move with shift (`Insert Group 5 At 9`, §35 15).
    Insert,
    /// Renumber a pool, holes closed (`Compact Group /By Name`, §35 16).
    Compact,
    /// An object's settings (`EditSetting Theme`; a one-of-a-kind settings object typed alone,
    /// decisions §33 63).
    EditSetting,
    /// Bring a preset's attributes into the cues that link it (decisions 0293).
    Recast,
    /// Turn preset links into plain values (decisions 0293).
    Extract,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Verb {
    pub fn label(self) -> &'static str {
        match self {
            Self::Call => "Call",
            Self::SelectFixtures => "SelectFixtures",
            Self::Select => "Select",
            Self::Goto => "Goto",
            Self::Go => "Go",
            Self::At => "At",
            Self::Store => "Store",
            Self::Delete => "Delete",
            Self::Label => "Label",
            Self::Assign => "Assign",
            Self::Set => "Set",
            Self::Edit => "Edit",
            Self::Copy => "Copy",
            Self::Move => "Move",
            Self::Update => "Update",
            Self::Unassign => "Unassign",
            Self::List => "List",
            Self::Info => "Info",
            Self::Dump => "Dump",
            Self::On => "On",
            Self::Off => "Off",
            Self::Toggle => "Toggle",
            Self::Join => "Join",
            Self::Leave => "Leave",
            Self::Invite => "Invite",
            Self::Dismiss => "Dismiss",
            Self::Claim => "Claim",
            Self::Pause => "Pause",
            Self::Import => "Import",
            Self::Export => "Export",
            Self::Reset => "Reset",
            Self::Sort => "Sort",
            Self::Split => "Split",
            Self::Cut => "Cut",
            Self::Paste => "Paste",
            Self::Insert => "Insert",
            Self::Compact => "Compact",
            Self::EditSetting => "EditSetting",
            Self::Recast => "Recast",
            Self::Extract => "Extract",
        }
    }
}
