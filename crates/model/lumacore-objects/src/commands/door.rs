// ██████╗  ██████╗  ██████╗ ██████╗
// ██╔══██╗██╔═══██╗██╔═══██╗██╔══██╗
// ██║  ██║██║   ██║██║   ██║██████╔╝
// ██║  ██║██║   ██║██║   ██║██╔══██╗
// ██████╔╝╚██████╔╝╚██████╔╝██║  ██║
// ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝
// FILE: door

//! The verbs on the object door (decisions §35): Copy, Cut, Paste, Move, Insert, Compact and
//! Delete, the same for every object whose `TreeDef` names a door. The rows come from the one
//! resolver, the writes go through `pools::door`, and this layer adds only what a line needs —
//! where `At` points, the clipboard, the asks and the replies. An object takes them with
//! `door_verbs!` (its typed structs, one per verb) and lists them on its `ObjectDef`.
//!
//! Owns:          door_verbs!, the destination of `At`, the clipboard, the existing-object and
//!                where-to asks, the Delete ask that names users, the one nesting ask, numbers_text.
//! Doesn't own:   the writes (`pools::door`), which rows a line names (`resolve`).
//! Entry points:  door_verbs!, delete_ask, nest_ask, PasteBare.
//! Decisions:     §35 4–10, 13–16; §34 "34g4 part 3c hardened" 3 (nesting).

use lumacore_syntax::ast::{Item, Target, Value};
use lumacore_wire::{ButtonKind, InputKind};
use uuid::Uuid;

use crate::errors::render_code;
use crate::objects::ObjectDef;
use crate::pools::door::{self as pool, PoolOps};
use crate::{AppState, DispatchResult};

mod compact;
mod copy;
mod cut;
mod delete;
mod insert;
mod paste;
mod relocate;

pub use compact::compact;
pub use copy::copy;
pub use cut::cut;
pub use delete::{delete, delete_ask};
pub use insert::insert;
pub use paste::{CompactBare, CutBare, InsertBare, PasteBare, paste};
pub use relocate::{Renumber, relocate};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// An object's typed door verbs (`door_verbs!(group_door, "Group", GROUP)`): a module with
/// `Copy`, `Cut`, `Paste`, `Move`, `Insert`, `Compact` and `Delete`, each `run: typed::<…>`.
#[macro_export]
macro_rules! door_verbs {
    ($module:ident, $keyword:literal, $object:path) => {
        pub mod $module {
            use lumacore_syntax::ast::{Target, Value};
            use $crate::commands::door as door;
            use $crate::{AppState, DispatchResult};

            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Copy", object = $keyword)]
            pub struct Copy {
                #[lum(target)]
                pub target: Target,
                #[lum(at = "destination")]
                pub to: Option<Value>,
                #[lum(option = "Merge")]
                pub merge: bool,
                #[lum(option = "Overwrite")]
                pub overwrite: bool,
            }
            impl $crate::commands::typed::Command for Copy {
                fn run(self, state: &mut AppState) -> DispatchResult {
                    door::copy(state, &$object, &self.target, self.to.as_ref(), door::on(self.merge, self.overwrite))
                }
            }

            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Cut", object = $keyword)]
            pub struct Cut {
                #[lum(target)]
                pub target: Target,
            }
            impl $crate::commands::typed::Command for Cut {
                fn run(self, state: &mut AppState) -> DispatchResult {
                    door::cut(state, &$object, &self.target)
                }
            }

            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Paste", object = $keyword)]
            pub struct Paste {
                #[lum(target)]
                pub target: Option<Target>,
                #[lum(option = "Merge")]
                pub merge: bool,
                #[lum(option = "Overwrite")]
                pub overwrite: bool,
            }
            impl $crate::commands::typed::Command for Paste {
                fn run(self, state: &mut AppState) -> DispatchResult {
                    door::paste(state, Some(&$object), self.target.as_ref(), door::on(self.merge, self.overwrite))
                }
            }

            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Move", object = $keyword)]
            pub struct Move {
                #[lum(target)]
                pub target: Target,
                #[lum(at = "destination")]
                pub to: Option<Value>,
                #[lum(option = "Skip")]
                pub skip: bool,
                #[lum(option = "By")]
                pub by: Option<Value>,
            }
            impl $crate::commands::typed::Command for Move {
                fn run(self, state: &mut AppState) -> DispatchResult {
                    let how = door::Renumber::of(self.skip, self.by.as_ref());
                    door::relocate(state, &$object, &self.target, self.to.as_ref(), how)
                }
            }

            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Insert", object = $keyword)]
            pub struct Insert {
                #[lum(target)]
                pub target: Target,
                #[lum(at = "destination")]
                pub to: Option<Value>,
            }
            impl $crate::commands::typed::Command for Insert {
                fn run(self, state: &mut AppState) -> DispatchResult {
                    door::insert(state, &$object, &self.target, self.to.as_ref())
                }
            }

            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Compact", object = $keyword)]
            pub struct Compact {
                #[lum(target)]
                pub target: Target,
                #[lum(option = "By")]
                pub by: Option<Value>,
            }
            impl $crate::commands::typed::Command for Compact {
                fn run(self, state: &mut AppState) -> DispatchResult {
                    door::compact(state, &$object, &self.target, self.by.as_ref())
                }
            }

            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Delete", object = $keyword)]
            pub struct Delete {
                #[lum(target)]
                pub target: Target,
            }
            impl $crate::commands::typed::Command for Delete {
                fn run(self, state: &mut AppState) -> DispatchResult {
                    door::delete(state, &$object, &self.target)
                }
            }
        }
    };
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The flags' answer to an existing row (decisions §35 5).
pub fn on(merge: bool, overwrite: bool) -> pool::OnExisting {
    match (merge, overwrite) {
        (true, _) => pool::OnExisting::Merge,
        (_, true) => pool::OnExisting::Overwrite,
        _ => pool::OnExisting::Ask,
    }
}

/// `/Merge` where the pool can't merge is refused, never an overwrite (decisions §35 5).
pub(crate) fn check_merge(ops: &dyn PoolOps, on: pool::OnExisting) -> Result<(), String> {
    if on == pool::OnExisting::Merge && !ops.can_merge() {
        return Err(render_code(format!("A {} can't merge — /Overwrite replaces it.", ops.object()), "E-CMD-010"));
    }
    Ok(())
}

/// The object's door, or why it has none yet.
pub(crate) fn ops(object: &'static ObjectDef) -> Result<&'static dyn PoolOps, String> {
    object.tree.as_ref().and_then(|t| t.door).ok_or_else(|| render_code(format!("A {} isn't on the object door yet.", object.keyword), "E-CMD-010"))
}

/// The rows a target names, with their DataPool (`DataPool 2 Group 3`, else the selected one).
pub(crate) fn rows(state: &AppState, object: &'static ObjectDef, target: &Target) -> Result<(Option<u32>, Vec<u32>), String> {
    let resolved = crate::resolve::resolve_one(state, target, object)?;
    let numbers: Vec<u32> = resolved.members.iter().map(|m| m.no).collect();
    if numbers.is_empty() {
        return Err(render_code(format!("No such {} (not found).", object.keyword), "E-CMD-007"));
    }
    Ok((resolved.datapool, numbers))
}

/// Where `At` points (decisions §35 6): a number in the rows' own pool, or an object of the same
/// kind in another owner (`At DataPool 2 Group 8`; the kind alone = its next free number).
/// Another kind is refused (E-CMD-020, Unix's EXDEV).
pub(crate) fn destination(state: &AppState, object: &'static ObjectDef, ops: &dyn PoolOps, from_dp: Option<u32>, at: &Value) -> Result<(Option<u32>, Option<u32>), String> {
    match at {
        Value::Number(n) => match n.node.parse::<u32>() {
            Ok(0) | Err(_) => Err(render_code(format!("At '{}': a {} number from 1.", n.node, object.keyword), "E-CMD-008")),
            Ok(n) => Ok((from_dp, Some(n))),
        },
        Value::Target(t) => {
            if let Some(other) = kind_of(t).filter(|o| !std::ptr::eq(*o, object)) {
                return Err(render_code(format!("A {} can't go into a {} — another kind (Import / Export carry objects across).", object.keyword, other.keyword), "E-CMD-020"));
            }
            let (slot, dp) = crate::resolve::slot(state, Some(t), object)?;
            let dp = dp.or(from_dp).filter(|_| ops.datapool(state, None).is_some());
            match slot {
                crate::resolve::Slot::Number(n) => Ok((dp, Some(n))),
                crate::resolve::Slot::Next => Ok((dp, None)),
                _ => Err(render_code(format!("At a {} number.", object.keyword), "E-CMD-008")),
            }
        }
        _ => Err(render_code(format!("At a {} number, or an owner's (At DataPool 2 {} 8).", object.keyword, object.keyword), "E-CMD-008")),
    }
}

/// The object a target names last (`DataPool 2 Group 8` → Group).
fn kind_of(target: &Target) -> Option<&'static ObjectDef> {
    let [(None, Item::Path(segments))] = target.items.as_slice() else { return None };
    let last = segments.last()?;
    crate::objects::find(&crate::lum::vocabulary().word(last.object.node).name)
}

/// The destinations of `rows` from `start` (their spacing kept), or the first free block in that
/// pool when `start` is None; a new number above the pool's cap is refused.
pub(crate) fn destinations(state: &AppState, ops: &dyn PoolOps, to_dp: Option<u32>, rows: &[u32], start: Option<u32>) -> Result<Vec<u32>, String> {
    let taken = ops.numbers(state, to_dp);
    let start = match start {
        Some(s) => s,
        None => pool::next_free_block(&taken, &pool::offsets(rows)).ok_or_else(|| render_code(format!("No room for them among the {}s.", ops.object()), "E-CMD-010"))?,
    };
    let dests = pool::spacing(rows, start).map_err(|e| render_code(e, "E-CMD-008"))?;
    if let Some(d) = dests.iter().find(|d| **d > ops.max_no() && !taken.contains(d)) {
        return Err(render_code(format!("{} numbers are 1–{}: {d} is above — nothing done.", ops.object(), ops.max_no()), "E-CMD-008"));
    }
    Ok(dests)
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// Numbers as a line writes them: `1 Thru 3 + 5`.
pub fn numbers_text(numbers: &[u32]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < numbers.len() {
        let mut j = i;
        while j + 1 < numbers.len() && numbers[j + 1] == numbers[j] + 1 {
            j += 1;
        }
        out.push(if j > i { format!("{} Thru {}", numbers[i], numbers[j]) } else { numbers[i].to_string() });
        i = j + 1;
    }
    out.join(" + ")
}

/// `DataPool 2 ` in front of a line's object when the rows aren't in the user's selected one.
pub(crate) fn owner_text(state: &AppState, ops: &dyn PoolOps, dp: Option<u32>) -> String {
    match ops.datapool(state, dp) {
        Some(d) if d != state.selected_datapool() => format!("DataPool {d} "),
        _ => String::new(),
    }
}

/// The one question for a write onto rows that exist (decisions §35 5): Merge where the pool can,
/// Overwrite, Cancel; the flags answer it, and a stored line has to.
pub(crate) fn exists_ask(ops: &dyn PoolOps, line: &str, taken: &[u32]) -> DispatchResult {
    let object = ops.object();
    let mut ask = crate::dialogs::Ask::new(line.to_string(), format!("{object} {} exists.", numbers_text(taken)));
    if ops.can_merge() {
        ask = ask.button("Merge", ButtonKind::Primary, format!("{line} /Merge"));
    }
    let flags = if ops.can_merge() { "/Merge or /Overwrite" } else { "/Overwrite" };
    DispatchResult::Ask(Box::new(ask.button("Overwrite", ButtonKind::Danger, format!("{line} /Overwrite")).cancel("Cancel").flags(flags)))
}

/// Where to? — the keypad (a bare Move / Insert, a bare Paste of a cut): `{to}` in `line`.
pub(crate) fn where_ask(title: &str, text: &str, object: &str, line: &str, flag: &str) -> DispatchResult {
    DispatchResult::Ask(Box::new(
        crate::dialogs::Ask::new(title.to_string(), text.to_string())
            .input("to", &format!("To ({object} number)"), "", InputKind::Number { min: Some(1.0), max: None, unit: None })
            .button("Please", ButtonKind::Please, line.to_string())
            .cancel("Cancel")
            .flags(flag.to_string()),
    ))
}

/// The one nesting question (decisions §34 "34g4 part 3c hardened" 3): `Move <node> At <node of
/// a nesting kind>` makes it a child, asked first — "MvrGroup 3 'Truss 2' becomes a child of
/// MvrGroup 2 'Truss 1' (2 groups, 88 fixtures)?". `line` is the Move as typed, with the rows
/// found; `/NoConfirmation` answers it (stored lines, a drag).
pub fn nest_ask(what: &str, under: &str, counts: &str, line: &str) -> DispatchResult {
    DispatchResult::Ask(Box::new(
        crate::dialogs::Ask::new("Move", format!("{what} becomes a child of {under} ({counts})?"))
            .button("Nest", ButtonKind::Primary, format!("{line} /NoConfirmation"))
            .cancel("Cancel")
            .flags("/NoConfirmation"),
    ))
}

/// A DataPool's GUID by number and back (the clipboard keeps GUIDs, §21 "6d").
pub(crate) fn datapool_guid(state: &AppState, dp: Option<u32>) -> Option<Uuid> {
    state.root.show_data.datapools.get(&dp?).map(|d| d.meta.identity.guid)
}

pub(crate) fn datapool_of(state: &AppState, guid: Option<Uuid>) -> Option<u32> {
    let guid = guid?;
    state.root.show_data.datapools.iter().find(|(_, d)| d.meta.identity.guid == guid).map(|(n, _)| *n)
}

/// A reply of lines; an error line keeps its code.
pub(crate) fn reply(lines: Vec<String>) -> DispatchResult {
    DispatchResult::Output(crate::Reply::joined(lines.into_iter().map(crate::Reply::from)))
}

pub(crate) fn refused(e: String) -> DispatchResult {
    DispatchResult::Output(e.into())
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
