//  ██████╗ ██████╗ ███╗   ███╗██████╗  █████╗  ██████╗████████╗
// ██╔════╝██╔═══██╗████╗ ████║██╔══██╗██╔══██╗██╔════╝╚══██╔══╝
// ██║     ██║   ██║██╔████╔██║██████╔╝███████║██║        ██║
// ██║     ██║   ██║██║╚██╔╝██║██╔═══╝ ██╔══██║██║        ██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║     ██║  ██║╚██████╗   ██║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝  ╚═╝ ╚═════╝   ╚═╝
// FILE: compact

//! `Compact <object> [<rows>] [/By <order>]` (decisions §35 16): renumber a pool one after
//! another — the whole pool from 1, or the rows named from their first — keeping GUIDs; `/By
//! Name` or a pool's own order (`/By Address`) instead of the numbers' order.
//!
//! Owns:          compact.
//! Doesn't own:   the write (`pools::door::compact`), `Sort` (an object's contents).
//! Entry points:  compact.
//! Decisions:     §35 16.

use lumacore_syntax::ast::{Target, Value};

use super::{ops, refused, reply};
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

pub fn compact(state: &mut AppState, object: &'static ObjectDef, target: &Target, by: Option<&Value>) -> DispatchResult {
    let mut run = || -> Result<DispatchResult, String> {
        let ops = ops(object)?;
        let resolved = crate::resolve::resolve_one(state, target, object)?;
        let rows: Option<Vec<u32>> = resolved.named.then(|| resolved.members.iter().map(|m| m.no).collect());
        let order = match by {
            None => pool::Order::Number,
            Some(Value::Word(w)) => order_of(&crate::lum::vocabulary().word(w.node).name),
            Some(Value::Text(t)) => order_of(&t.node),
            Some(_) => return Err(render_code(format!("/By {}.", ops.orders().join(", ")), "E-CMD-008")),
        };
        let lines = pool::compact(ops, state, resolved.datapool, rows.as_deref(), &order).map_err(|e| render_code(e, "E-CMD-010"))?;
        Ok(if lines.is_empty() { DispatchResult::Output(crate::Reply::ok(format!("{}s: nothing to renumber.", object.keyword))) } else { reply(lines) })
    };
    run().unwrap_or_else(refused)
}

fn order_of(name: &str) -> pool::Order {
    match name.to_ascii_lowercase().as_str() {
        "number" => pool::Order::Number,
        "name" => pool::Order::Name,
        _ => pool::Order::By(name.to_string()),
    }
}
