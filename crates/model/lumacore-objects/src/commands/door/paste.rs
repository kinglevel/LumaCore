// ██████╗  █████╗ ███████╗████████╗███████╗
// ██╔══██╗██╔══██╗██╔════╝╚══██╔══╝██╔════╝
// ██████╔╝███████║███████╗   ██║   █████╗
// ██╔═══╝ ██╔══██║╚════██║   ██║   ██╔══╝
// ██║     ██║  ██║███████║   ██║   ███████╗
// ╚═╝     ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝
// FILE: paste

//! `Paste [<target>] [/Merge | /Overwrite]` (decisions §35 9, 10, 13): finishes a `Copy` (new
//! objects) or a `Cut` (the same objects moved). The target is where they go; bare, a copy goes
//! to the first free numbers of its pool and a cut asks where. The objects are taken as they are
//! now — one deleted since is skipped and named.
//!
//! Owns:          paste, PasteBare.
//! Doesn't own:   the writes (`pools::door`), the clipboard's filling (`cut::put_aside`).
//! Entry points:  paste, PasteBare.
//! Decisions:     §35 9, 10, 13.

use lumacore_syntax::ast::{Target, Value};

use super::{datapool_of, destination, destinations, exists_ask, ops, owner_text, refused, reply, where_ask};
use crate::errors::render_code;
use crate::objects::ObjectDef;
use crate::pools::door as pool;
use crate::{AppState, DispatchResult};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A bare `Paste` (no object): where the clipboard's objects came from decides.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Paste")]
pub struct PasteBare {
    #[lum(option = "Merge")]
    pub merge: bool,
    #[lum(option = "Overwrite")]
    pub overwrite: bool,
}

/// `Cut`, `Insert` or `Compact` with no object: which objects take it (the registry's usage).
macro_rules! bare_usage {
    ($name:ident, $verb:literal, $v:ident) => {
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = $verb)]
        pub struct $name {}

        impl crate::commands::typed::Command for $name {
            fn run(self, _: &mut AppState) -> DispatchResult {
                crate::commands::run::objects_usage(crate::objects::Verb::$v)
            }
        }
    };
}

bare_usage!(CutBare, "Cut", Cut);
bare_usage!(InsertBare, "Insert", Insert);
bare_usage!(CompactBare, "Compact", Compact);

impl crate::commands::typed::Command for PasteBare {
    fn run(self, state: &mut AppState) -> DispatchResult {
        paste(state, None, None, super::on(self.merge, self.overwrite))
    }
}

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

pub fn paste(state: &mut AppState, object: Option<&'static ObjectDef>, target: Option<&Target>, on: pool::OnExisting) -> DispatchResult {
    let mut run = || -> Result<DispatchResult, String> {
        let user = state.effective_user_name().to_string();
        let clip = state.root.runtime.user_ref(&user).and_then(|u| u.clipboard.clone()).ok_or_else(|| render_code("Nothing on your clipboard — Copy or Cut first.", "E-CMD-009"))?;
        let kind = crate::objects::find(&clip.object).ok_or_else(|| render_code(format!("'{}' is no object any more.", clip.object), "E-CMD-007"))?;
        if let Some(o) = object.filter(|o| !std::ptr::eq(*o, kind)) {
            return Err(render_code(format!("Your clipboard holds {}s — a {} can't take them.", kind.keyword, o.keyword), "E-CMD-020"));
        }
        let ops = ops(kind)?;
        super::check_merge(ops, on)?;
        let from_dp = match clip.datapool {
            Some(g) => Some(datapool_of(state, Some(g)).ok_or_else(|| render_code("The DataPool they came from is gone.", "E-CMD-007"))?),
            None => None,
        };
        // Where each one is now (it may have moved), in clipboard order; the gone are named.
        let now: Vec<(u32, uuid::Uuid)> = ops.numbers(state, from_dp).into_iter().filter_map(|n| ops.guid(state, from_dp, n).map(|g| (n, g))).collect();
        let mut kept = Vec::new();
        let mut gone = Vec::new();
        for (was, guid) in &clip.rows {
            match now.iter().find(|(_, g)| g == guid) {
                Some((n, _)) => kept.push((*was, *n)),
                None => gone.push(format!("{} {was} is gone — not pasted.", kind.keyword)),
            }
        }
        if kept.is_empty() {
            return Err(render_code(format!("Nothing on your clipboard is still there: {}", gone.join(" ")), "E-CMD-007"));
        }
        let original: Vec<u32> = kept.iter().map(|(w, _)| *w).collect();
        let current: Vec<u32> = kept.iter().map(|(_, n)| *n).collect();
        let (to_dp, start) = match target {
            Some(t) => destination(state, kind, ops, from_dp, &Value::Target(t.clone()))?,
            None if clip.cut => {
                let line = format!("Paste {} {{to}}", kind.keyword);
                return Ok(where_ask("Paste", &format!("Paste the cut {}s where?", kind.keyword), kind.keyword, &line, &format!("Paste {} <number>", kind.keyword)));
            }
            None => (from_dp, None),
        };
        let dests = destinations(state, ops, to_dp, &original, start)?;
        let mut lines = if clip.cut {
            let lines = pool::relocate(ops, state, from_dp, &current, to_dp, &dests);
            state.root.runtime.user(&user).clipboard = None;
            lines
        } else {
            match pool::copy(ops, state, from_dp, &current, to_dp, &dests, on) {
                Ok(lines) => lines,
                Err(pool::Exists { taken }) => {
                    let line = format!("Paste {}{} {}", owner_text(state, ops, to_dp), kind.keyword, dests[0]);
                    return Ok(exists_ask(ops, &line, &taken));
                }
            }
        };
        lines.extend(gone);
        Ok(reply(lines))
    };
    run().unwrap_or_else(refused)
}
