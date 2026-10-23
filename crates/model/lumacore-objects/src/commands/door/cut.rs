//  ██████╗██╗   ██╗████████╗
// ██╔════╝██║   ██║╚══██╔══╝
// ██║     ██║   ██║   ██║
// ██║     ██║   ██║   ██║
// ╚██████╗╚██████╔╝   ██║
//  ╚═════╝ ╚═════╝    ╚═╝
// FILE: cut

//! `Cut <target>` (decisions §35 9, 14): the objects go on your clipboard to be moved by `Paste`;
//! a bare `Copy` puts them there to be copied. The clipboard holds GUIDs (§35 10).
//!
//! Owns:          cut, put_aside.
//! Doesn't own:   Paste (`paste`).
//! Entry points:  cut, put_aside.
//! Decisions:     §35 9, 10, 14.

use lumacore_syntax::ast::Target;

use super::{datapool_guid, numbers_text, ops, refused, rows};
use crate::objects::ObjectDef;
use crate::pools::door::PoolOps;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

pub fn cut(state: &mut AppState, object: &'static ObjectDef, target: &Target) -> DispatchResult {
    let mut run = || -> Result<DispatchResult, String> {
        let ops = ops(object)?;
        let (dp, rows) = rows(state, object, target)?;
        Ok(put_aside(state, ops, dp, &rows, true))
    };
    run().unwrap_or_else(refused)
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The rows on the acting user's clipboard, cut or copied.
pub(crate) fn put_aside(state: &mut AppState, ops: &dyn PoolOps, dp: Option<u32>, rows: &[u32], cut: bool) -> DispatchResult {
    let dp = ops.datapool(state, dp);
    let clipboard = lumacore_tree::Clipboard {
        cut,
        object: ops.object().to_string(),
        datapool: datapool_guid(state, dp),
        rows: rows.iter().filter_map(|&n| ops.guid(state, dp, n).map(|g| (n, g))).collect(),
    };
    let user = state.effective_user_name().to_string();
    state.root.runtime.user(&user).clipboard = Some(clipboard);
    let what = format!("{} {}", ops.object(), numbers_text(rows));
    DispatchResult::Output(crate::Reply::ok(format!(
        "{what} {} — Paste {} <number> puts {}.",
        if cut { "cut, on your clipboard" } else { "on your clipboard" },
        ops.object(),
        if cut { "them there" } else { "copies there" }
    )))
}
