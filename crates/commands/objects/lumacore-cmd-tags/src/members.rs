// ███╗   ███╗███████╗███╗   ███╗██████╗ ███████╗██████╗ ███████╗
// ████╗ ████║██╔════╝████╗ ████║██╔══██╗██╔════╝██╔══██╗██╔════╝
// ██╔████╔██║█████╗  ██╔████╔██║██████╔╝█████╗  ██████╔╝███████╗
// ██║╚██╔╝██║██╔══╝  ██║╚██╔╝██║██╔══██╗██╔══╝  ██╔══██╗╚════██║
// ██║ ╚═╝ ██║███████╗██║ ╚═╝ ██║██████╔╝███████╗██║  ██║███████║
// ╚═╝     ╚═╝╚══════╝╚═╝     ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝╚══════╝
// FILE: members

//! The verbs a tag passes on to what it carries (decisions §12 "Tags hardened" 4, lum.md "Tags"):
//! `Off Tag "All"`, `Go Tag 2.1`, `Tag 1.1 At 50`. They reach the objects tagged with the tag
//! (the current level); `/R` adds those of every tag under it. Each kind that takes the verb runs
//! its own handler (in its DataPool, by its own rights); the rest are skipped with a summary.
//! One Tag line is one statement, so one Oops step.
//!
//! Owns:          OnTag, OffTag, ToggleTag, GoTag, GotoTag, PauseTag, AtTag, pass_on.
//! Doesn't own:   what each kind does with the verb (its own handler).
//! Entry points:  the structs (TAG's verbs).
//! Decisions:     §12 "Tags hardened", lum.md "Tags", §30 (rights per object).

use lumacore_objects::errors::render_code;
use lumacore_objects::objects::{Verb, out};
use lumacore_objects::{AppState, DispatchResult};
use lumacore_syntax::ast::Value;

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

macro_rules! member_verb {
    ($name:ident, $word:literal, $verb:expr) => {
        #[doc = concat!("`", $word, " Tag <path> [/R]`: ", $word, " on what the tag carries.")]
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = $word, object = "Tag")]
        pub struct $name {
            #[lum(target)]
            pub target: lumacore_syntax::ast::Target,
            #[lum(option = "R")]
            pub recursive: bool,
        }

        impl lumacore_objects::commands::typed::Command for $name {
            fn run(self, state: &mut AppState) -> DispatchResult {
                pass_on(state, &self.target, self.recursive, $verb, &[])
            }
        }
    };
}

member_verb!(OnTag, "On", Verb::On);
member_verb!(OffTag, "Off", Verb::Off);
member_verb!(ToggleTag, "Toggle", Verb::Toggle);
member_verb!(GoTag, "Go", Verb::Go);
member_verb!(GotoTag, "Goto", Verb::Goto);
member_verb!(PauseTag, "Pause", Verb::Pause);

/// `Tag <path> At <value> [/R]`: the value on what the tag carries (a fixture's dimmer, a
/// sequence's fader, …: each kind's own At).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "At", object = "Tag")]
pub struct AtTag {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(at = "value")]
    pub value: Value,
    #[lum(option = "R")]
    pub recursive: bool,
}

impl lumacore_objects::commands::typed::Command for AtTag {
    fn run(self, state: &mut AppState) -> DispatchResult {
        match value_words(&self.value) {
            Ok(v) => {
                let tail: Vec<&str> = std::iter::once("At").chain(v.iter().map(String::as_str)).collect();
                pass_on(state, &self.target, self.recursive, Verb::At, &tail)
            }
            Err(e) => out(e),
        }
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Run `verb` on what the tags `target` names carry, kind by kind; `tail` follows the ids (`At 50`).
fn pass_on(state: &mut AppState, target: &lumacore_syntax::ast::Target, recursive: bool, verb: Verb, tail: &[&str]) -> DispatchResult {
    let tags = match crate::tag::targeted(state, target) {
        Ok(t) => t,
        Err(e) => return out(e),
    };
    let guids: Vec<uuid::Uuid> = tags.iter().map(|(_, g)| *g).collect();
    let reach = crate::tag::reach(state, &guids, recursive);
    let carriers = crate::tag::carriers(state, &reach);
    let mut ran: Vec<String> = Vec::new();
    let mut skipped: Vec<lumacore_tags::Member> = Vec::new();
    for (def, dp, numbers, members) in by_kind(state, &carriers) {
        let Some(handler) = def.handler(verb) else {
            skipped.extend(members);
            continue;
        };
        let ids = numbers.iter().map(u32::to_string).collect::<Vec<_>>().join(" + ");
        let mut args: Vec<&str> = ids.split(' ').collect();
        args.extend_from_slice(tail);
        let needed = (def.rights)(verb, &args);
        if let Err(e) = lumacore_objects::rights::require(state, needed, &format!("{verb:?} {}", def.keyword)) {
            return out(e);
        }
        let result = state.in_datapool(dp, |s| handler(s, &args));
        match result {
            DispatchResult::Output(r) => ran.push(String::from(r)),
            other => return other,
        }
    }
    if ran.is_empty() {
        let hint = if recursive || carriers.is_empty() { "" } else { " (add /R for the tags under it)" };
        let what = if carriers.is_empty() { "Nothing carries it".to_string() } else { format!("Nothing it carries takes {verb:?}") };
        return out(render_code(format!("{what}{hint}."), "E-CMD-009"));
    }
    if !skipped.is_empty() {
        ran.push(format!("skipped {} (no {verb:?})", crate::tag::kinds_text(&skipped)));
    }
    out(ran.join("\n"))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The carriers grouped by kind and DataPool, with each one's number there.
type KindGroup = (&'static lumacore_objects::objects::ObjectDef, Option<u32>, Vec<u32>, Vec<lumacore_tags::Member>);

fn by_kind(state: &AppState, carriers: &[lumacore_tags::Member]) -> Vec<KindGroup> {
    let root = &state.root;
    let mut out: Vec<KindGroup> = Vec::new();
    for m in carriers {
        let Some(def) = lumacore_objects::objects::find(m.kind) else { continue };
        let Some(tree) = def.tree.as_ref() else { continue };
        let pools: Vec<Option<u32>> = match tree.datapool {
            Some(_) => root.show_data.datapools.keys().map(|k| Some(*k)).collect(),
            None => vec![None],
        };
        let found = pools.into_iter().find_map(|dp| tree.children_in(root, dp).into_iter().find(|c| c.guid == Some(m.guid)).map(|c| (dp, c.id)));
        let Some((dp, no)) = found else { continue };
        match out.iter_mut().find(|(d, p, _, _)| std::ptr::eq(*d, def) && *p == dp) {
            Some((_, _, nos, ms)) => {
                nos.push(no);
                ms.push(m.clone());
            }
            None => out.push((def, dp, vec![no], vec![m.clone()])),
        }
    }
    out
}

/// A value as the words it was typed with (`50`, `+ 10`, `Full`, `"text"`).
fn value_words(v: &Value) -> Result<Vec<String>, String> {
    Ok(match v {
        Value::Number(n) => vec![n.node.clone()],
        Value::Relative { op, number } => vec![op.node.to_string(), number.node.clone()],
        Value::Word(w) => vec![lumacore_objects::lum::vocabulary().word(w.node).name.clone()],
        Value::Text(t) => vec![format!("\"{}\"", t.node)],
        _ => return Err(render_code("Tag … At takes a value: At 50, At Full, At + 10.", "E-CMD-008")),
    })
}
