// ██████╗ ███████╗██╗      ██████╗  ██████╗ █████╗ ████████╗███████╗
// ██╔══██╗██╔════╝██║     ██╔═══██╗██╔════╝██╔══██╗╚══██╔══╝██╔════╝
// ██████╔╝█████╗  ██║     ██║   ██║██║     ███████║   ██║   █████╗
// ██╔══██╗██╔══╝  ██║     ██║   ██║██║     ██╔══██║   ██║   ██╔══╝
// ██║  ██║███████╗███████╗╚██████╔╝╚██████╗██║  ██║   ██║   ███████╗
// ╚═╝  ╚═╝╚══════╝╚══════╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚══════╝
// FILE: relocate

//! `Move <target> At <where>` (decisions §35 4, 6): the same objects (GUIDs kept) at new numbers;
//! onto a taken number the two swap, a range slot by slot; between owners of one kind too. A bare
//! `Move <target>` asks where.
//!
//! Owns:          relocate.
//! Doesn't own:   the write (`pools::door::relocate`).
//! Entry points:  relocate.
//! Decisions:     §35 4, 6, 14.

use lumacore_syntax::ast::{Target, Value};

use super::{destination, destinations, numbers_text, ops, owner_text, refused, reply, rows, where_ask};
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

/// How `Move … At n` numbers them (decisions §34 round 3 19): by default the spacing is kept and
/// taken numbers swap; `/Skip` jumps over taken numbers, `/By <column>` numbers them in that
/// column's order — both consecutive from n.
#[derive(Clone, Debug, Default)]
pub struct Renumber {
    pub skip: bool,
    pub by: Option<String>,
}

impl Renumber {
    pub fn of(skip: bool, by: Option<&Value>) -> Self {
        let by = by.map(|v| match v {
            Value::Text(t) | Value::Number(t) => t.node.clone(),
            Value::Word(w) => crate::lum::vocabulary().word(w.node).name.clone(),
            // `/By FixtureType`: an object word stands as the column's name.
            Value::Target(t) => match t.items.as_slice() {
                [(None, lumacore_syntax::ast::Item::Path(segs))] if segs.len() == 1 => crate::lum::vocabulary().word(segs[0].object.node).name.clone(),
                _ => String::new(),
            },
            _ => String::new(),
        });
        Renumber { skip, by }
    }
}

pub fn relocate(state: &mut AppState, object: &'static ObjectDef, target: &Target, to: Option<&Value>, how: Renumber) -> DispatchResult {
    let mut run = || -> Result<DispatchResult, String> {
        let ops = ops(object)?;
        let (dp, mut rows) = rows(state, object, target)?;
        if let Some(by) = &how.by {
            if !ops.orders().iter().any(|o| o.eq_ignore_ascii_case(by)) {
                return Err(crate::errors::render_code(format!("{}s number by {}, not '{by}'.", object.keyword, ops.orders().join(", ")), "E-CMD-008"));
            }
            let keys: std::collections::BTreeMap<u32, String> = rows.iter().map(|&n| (n, ops.order_key(state, dp, n, by).unwrap_or_default())).collect();
            rows.sort_by(|a, b| keys[a].cmp(&keys[b]).then(a.cmp(b)));
        }
        let Some(at) = to else {
            let what = format!("{}{} {}", owner_text(state, ops, dp), object.keyword, numbers_text(&rows));
            return Ok(where_ask(&format!("Move {what}"), &format!("Move {what} to another number."), object.keyword, &format!("Move {what} At {{to}}"), &format!("At <number>: Move {what} At <n>")));
        };
        let (to_dp, start) = destination(state, object, ops, dp, at)?;
        let dests = if how.skip || how.by.is_some() {
            consecutive(state, ops, (dp, to_dp), &rows, start, how.skip)?
        } else {
            destinations(state, ops, to_dp, &rows, start)?
        };
        Ok(reply(pool::relocate(ops, state, dp, &rows, to_dp, &dests)))
    };
    run().unwrap_or_else(refused)
}

/// Consecutive numbers from `start` for `rows` in their order; `skip` jumps over numbers rows that
/// aren't moving hold (else they swap, as Move does).
fn consecutive(state: &AppState, ops: &dyn crate::pools::door::PoolOps, (dp, to_dp): (Option<u32>, Option<u32>), rows: &[u32], start: Option<u32>, skip: bool) -> Result<Vec<u32>, String> {
    let taken: std::collections::BTreeSet<u32> = ops.numbers(state, to_dp).into_iter().filter(|n| !(dp == to_dp && rows.contains(n))).collect();
    let mut n = match start {
        Some(s) => s,
        None => pool::next_free(&taken.iter().copied().collect::<Vec<_>>()),
    };
    let mut out = Vec::new();
    for _ in rows {
        while skip && taken.contains(&n) {
            n += 1;
        }
        if n > ops.max_no() {
            return Err(crate::errors::render_code(format!("{} numbers are 1–{}: no room from {} — nothing done.", ops.object(), ops.max_no(), out.first().unwrap_or(&n)), "E-CMD-008"));
        }
        out.push(n);
        n += 1;
    }
    Ok(out)
}
