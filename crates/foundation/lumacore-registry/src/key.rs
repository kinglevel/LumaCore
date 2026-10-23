// ██╗  ██╗███████╗██╗   ██╗
// ██║ ██╔╝██╔════╝╚██╗ ██╔╝
// █████╔╝ █████╗   ╚████╔╝
// ██╔═██╗ ██╔══╝    ╚██╔╝
// ██║  ██╗███████╗   ██║
// ╚═╝  ╚═╝╚══════╝   ╚═╝
// FILE: key
//! A built-in UI key (decisions §17, §30 "12a3"): its id, label, icon, tooltip, a command line
//! per gesture, its Meow lines, and the state it lights by. Pure data: the show tree turns the
//! catalog's keys into its locked `UiKey`s at `Root.Resources.UIKeys` (decisions §34 21).
//!
//! Owns:          KeyDecl
//! Doesn't own:   the show's keys (`lumacore_tree::UiKey`), binding keyboard keys (base)
//! Entry points:  KeyDecl
//! Decisions:     §17, §30 "12a3", "12f", §34 21

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One built-in key as the catalog holds it (built rows: owned text).
#[derive(Debug, Clone, Default)]
pub struct KeyDecl {
    /// Stable id: `menu`, `desklock`.
    pub id: String,
    pub label: String,
    pub icon: String,
    pub tooltip: String,
    pub press: String,
    pub double_press: String,
    pub hold: String,
    pub release: String,
    pub release_after_hold: String,
    /// What a press runs while Meow is held.
    pub meow_press: String,
    /// What the release after a hold runs while Meow is held.
    pub meow_release_after_hold: String,
    /// The double press stands in for the press (the UI undoes what the first press opened).
    pub double_replaces_press: bool,
    /// What lights it (`Runtime.Users.{User}.Modes.Blind`, `Device.Locked`).
    pub state: String,
}
