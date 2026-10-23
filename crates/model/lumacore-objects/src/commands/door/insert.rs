// ██╗███╗   ██╗███████╗███████╗██████╗ ████████╗
// ██║████╗  ██║██╔════╝██╔════╝██╔══██╗╚══██╔══╝
// ██║██╔██╗ ██║███████╗█████╗  ██████╔╝   ██║
// ██║██║╚██╗██║╚════██║██╔══╝  ██╔══██╗   ██║
// ██║██║ ╚████║███████║███████╗██║  ██║   ██║
// ╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝╚═╝  ╚═╝   ╚═╝
// FILE: insert

//! `Insert <target> At <where>` (decisions §35 15): a move with shift — the objects land at
//! `At`, and the ones in the way move up to the first free number. GUIDs kept. A bare `Insert
//! <target>` asks where.
//!
//! Owns:          insert.
//! Doesn't own:   the write (`pools::door::insert`).
//! Entry points:  insert.
//! Decisions:     §35 15.

use lumacore_syntax::ast::{Target, Value};

use super::{destination, numbers_text, ops, owner_text, refused, reply, rows, where_ask};
use crate::errors::render_code;
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

pub fn insert(state: &mut AppState, object: &'static ObjectDef, target: &Target, to: Option<&Value>) -> DispatchResult {
    let mut run = || -> Result<DispatchResult, String> {
        let ops = ops(object)?;
        let (dp, rows) = rows(state, object, target)?;
        let Some(at) = to else {
            let what = format!("{}{} {}", owner_text(state, ops, dp), object.keyword, numbers_text(&rows));
            return Ok(where_ask(&format!("Insert {what}"), &format!("Insert {what} at a number; the ones there move up."), object.keyword, &format!("Insert {what} At {{to}}"), &format!("At <number>: Insert {what} At <n>")));
        };
        let (to_dp, start) = destination(state, object, ops, dp, at)?;
        let start = start.unwrap_or_else(|| pool::next_free(&ops.numbers(state, to_dp)));
        Ok(reply(pool::insert(ops, state, dp, &rows, to_dp, start).map_err(|e| render_code(e, "E-CMD-010"))?))
    };
    run().unwrap_or_else(refused)
}
