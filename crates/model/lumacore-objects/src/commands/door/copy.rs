//  ██████╗ ██████╗ ██████╗ ██╗   ██╗
// ██╔════╝██╔═══██╗██╔══██╗╚██╗ ██╔╝
// ██║     ██║   ██║██████╔╝ ╚████╔╝
// ██║     ██║   ██║██╔═══╝   ╚██╔╝
// ╚██████╗╚██████╔╝██║        ██║
//  ╚═════╝ ╚═════╝ ╚═╝        ╚═╝
// FILE: copy

//! `Copy <target> At <where> [/Merge | /Overwrite]` and the bare `Copy <target>` that puts them on
//! the clipboard (decisions §35 3, 5, 6, 9): new GUIDs, links kept, the spacing kept.
//!
//! Owns:          copy.
//! Doesn't own:   the write (`pools::door::copy`), the clipboard (`cut::put_aside`).
//! Entry points:  copy.
//! Decisions:     §35 3, 5, 6, 9.

use lumacore_syntax::ast::{Target, Value};

use super::{destination, destinations, exists_ask, numbers_text, ops, owner_text, refused, reply, rows};
use crate::objects::ObjectDef;
use crate::pools::door as pool;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

pub fn copy(state: &mut AppState, object: &'static ObjectDef, target: &Target, to: Option<&Value>, on: pool::OnExisting) -> DispatchResult {
    let mut run = || -> Result<DispatchResult, String> {
        let ops = ops(object)?;
        super::check_merge(ops, on)?;
        let (dp, rows) = rows(state, object, target)?;
        let Some(at) = to else { return Ok(super::cut::put_aside(state, ops, dp, &rows, false)) };
        let (to_dp, start) = destination(state, object, ops, dp, at)?;
        let dests = destinations(state, ops, to_dp, &rows, start)?;
        Ok(match pool::copy(ops, state, dp, &rows, to_dp, &dests, on) {
            Ok(lines) => reply(lines),
            Err(pool::Exists { taken }) => {
                let at = if ops.datapool(state, to_dp) == ops.datapool(state, dp) { dests[0].to_string() } else { format!("{}{} {}", owner_text(state, ops, to_dp), object.keyword, dests[0]) };
                let line = format!("Copy {}{} {} At {at}", owner_text(state, ops, dp), object.keyword, numbers_text(&rows));
                exists_ask(ops, &line, &taken)
            }
        })
    };
    run().unwrap_or_else(refused)
}
