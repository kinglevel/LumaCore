// ██╗  ██╗ ██████╗  ██████╗ ██╗  ██╗███████╗
// ██║  ██║██╔═══██╗██╔═══██╗██║ ██╔╝██╔════╝
// ███████║██║   ██║██║   ██║█████╔╝ ███████╗
// ██╔══██║██║   ██║██║   ██║██╔═██╗ ╚════██║
// ██║  ██║╚██████╔╝╚██████╔╝██║  ██╗███████║
// ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: hooks

//! What a family adds to dispatch (decisions §34 25.13): each family pushes its hooks into the
//! catalog beside its rows, and dispatch runs them without naming a family — the special cases
//! that used to be written into `dispatch.rs`.
//!
//! Owns:          `Hook` and `hooks()`.
//! Doesn't own:   what each hook does (its family), when dispatch runs them (dispatch.rs).
//! Entry points:  Hook, hooks.
//! Decisions:     §34 25.1, 25.13.

use crate::objects::ObjectDef;
use crate::{AppState, DispatchResult};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One thing a family adds to how a line runs.
pub enum Hook {
    /// Before a line runs: a refusal stops it (a locked device runs only its lock keys).
    Gate(fn(&AppState, &str) -> Option<String>),
    /// Writes a line out as the Lum line it means before anything reads it; an error refuses it
    /// (the tree shell's `mv 3 7` on the show tree is `Move … At 7`).
    Rewrite(fn(&AppState, &str) -> Option<Result<String, String>>),
    /// Takes a line before its words run (`<line> /Watch`).
    Intercept(fn(&mut AppState, &[String], &str) -> Option<DispatchResult>),
    /// A user's show line went through (its words rank their suggestions: `Usage`).
    Used(fn(&mut AppState, &str)),
    /// After another show replaced the last (a load, a sync, a merge, NewShow): the key lines
    /// are checked.
    ShowReplaced(fn(&mut AppState)),
    /// After every change pass (windows following the profiles they show).
    Changed(fn(&mut AppState)),
    /// A patch draft was kept (`standard`, `mine`: the new attributes are cleared).
    PatchKept(fn(&mut AppState, bool, bool)),
    /// `Delete <object>` never asks in the station (it asks itself, or clearing is harmless).
    DeleteNeverAsks(&'static ObjectDef),
    /// `Set` on the object with these words is checked where it is written, not by the gate's
    /// property level (`Set Station "Tower" …`: the station that owns it checks).
    SetCheckedElsewhere(&'static ObjectDef, fn(&[&str]) -> bool),
    /// `Import Show "x" <Object> <ids>` takes single objects of this kind: the GUID of the one
    /// `id` names in the other show (`u32` = the selected DataPool's number).
    ImportByGuid(&'static ObjectDef, fn(&lumacore_tree::ShowData, u32, &str) -> Option<String>),
    /// `Set` / `Reset` on the object is never an Oops step (a grid's columns on your profile).
    SetUntracked(&'static ObjectDef),
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Every family's hooks, from the catalog.
pub fn hooks() -> impl Iterator<Item = &'static Hook> {
    lumacore_registry::get().rows::<Hook>()
}
