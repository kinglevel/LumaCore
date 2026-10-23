// ██████╗ ███████╗██╗     ███████╗████████╗███████╗
// ██╔══██╗██╔════╝██║     ██╔════╝╚══██╔══╝██╔════╝
// ██║  ██║█████╗  ██║     █████╗     ██║   █████╗
// ██║  ██║██╔══╝  ██║     ██╔══╝     ██║   ██╔══╝
// ██████╔╝███████╗███████╗███████╗   ██║   ███████╗
// ╚═════╝ ╚══════╝╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: delete

//! `Delete <target>` on the door (decisions §35 7): the one Delete ask names what uses each row;
//! after a Yes the rows go, a row its pool keeps is refused, and the links left behind are kept
//! broken and listed (Oops brings it all back).
//!
//! Owns:          delete, delete_ask.
//! Doesn't own:   the write (`pools::door::delete`), the ask's place in dispatch (`delete_first`).
//! Entry points:  delete, delete_ask.
//! Decisions:     §35 7, §30 "Merge of bugfixes".

use lumacore_syntax::ast::Target;
use lumacore_wire::ButtonKind;

use super::{numbers_text, ops, refused, reply, rows};
use crate::objects::{DeleteAsk, ObjectDef};
use crate::pools::door as pool;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

pub fn delete(state: &mut AppState, object: &'static ObjectDef, target: &Target) -> DispatchResult {
    let mut run = || -> Result<DispatchResult, String> {
        let ops = ops(object)?;
        let (dp, rows) = rows(state, object, target)?;
        Ok(reply(pool::delete(ops, state, dp, &rows)))
    };
    run().unwrap_or_else(refused)
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The station's one Delete question for a door object (`ObjectDef::delete_ask`): the rows, and
/// what uses them. `args` are what follows the object word, or the whole target when owners come
/// first (`DataPool 2 Group 3`).
pub fn delete_ask(state: &AppState, object: &'static ObjectDef, args: &[&str]) -> DeleteAsk {
    let words: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with('/')).collect();
    let owned = words.first().is_some_and(|w| !w.starts_with('"') && crate::objects::find(w).is_some());
    let target_text = if owned { words.join(" ") } else { format!("{} {}", object.keyword, words.join(" ")) };
    let target = match crate::resolve::target_of(&target_text) {
        Ok(t) => t,
        Err(e) => return DeleteAsk::Refuse(e),
    };
    let Ok(ops) = ops(object) else { return DeleteAsk::Plain };
    let (dp, rows) = match rows(state, object, &target) {
        Ok(r) => r,
        Err(e) => return DeleteAsk::Refuse(e),
    };
    let mut text = if rows.len() == 1 {
        let name = ops.name(state, dp, rows[0]).map(|n| format!(" \"{n}\"")).unwrap_or_default();
        format!("Delete {} {}{name}?", object.keyword, rows[0])
    } else {
        format!("Delete {} {}s ({})?", rows.len(), object.keyword, numbers_text(&rows))
    };
    // The answer names the rows found now (a pattern or a range never names others later).
    let line = format!("Delete {}{} {}", super::owner_text(state, ops, dp), object.keyword, numbers_text(&rows));
    let mut users: Vec<String> = rows.iter().filter_map(|&n| ops.guid(state, dp, n)).flat_map(|g| pool::users_of(&state.root.show_data, g)).collect();
    users.sort();
    users.dedup();
    if !users.is_empty() {
        let shown: Vec<&str> = users.iter().take(4).map(String::as_str).collect();
        let more = users.len().saturating_sub(4);
        text.push_str(&format!("\nUsed by: {}{} — left broken and listed.", shown.join(", "), if more > 0 { format!(" and {more} more") } else { String::new() }));
    }
    DeleteAsk::Ask(Box::new(
        crate::dialogs::Ask::new("Delete", text)
            .button("Delete", ButtonKind::Danger, format!("{line} /NoConfirmation"))
            .cancel("Cancel")
            .flags("/NoConfirmation"),
    ))
}
