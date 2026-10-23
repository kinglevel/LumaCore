// ██████╗ ███████╗ ██████╗ █████╗ ███████╗████████╗
// ██╔══██╗██╔════╝██╔════╝██╔══██╗██╔════╝╚══██╔══╝
// ██████╔╝█████╗  ██║     ███████║███████╗   ██║
// ██╔══██╗██╔══╝  ██║     ██╔══██║╚════██║   ██║
// ██║  ██║███████╗╚██████╗██║  ██║███████║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝  ╚═╝╚══════╝   ╚═╝
// FILE: recast

//! `Recast Preset <ids> [Sequence <m> [Cue <n>]]` (decisions 0293): the cues that link a preset
//! get links for the attributes it holds now and lose the links to the ones it dropped. It asks
//! first, naming each cue and what it gains and loses (`/NoConfirmation` answers); the Update ask
//! offers it as a switch.
//!
//! Owns:          RecastPreset, its ask.
//! Doesn't own:   the plan and the write (`links::recast_plan`, `links::recast`).
//! Entry points:  RecastPreset.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract.

use lumacore_wire::ButtonKind;

use super::links::{self, CueScope};
use crate::errors::render_code;
use crate::text::render_success;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Recast", object = "Preset")]
pub struct RecastPreset {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    /// Only these cues: `Sequence 1`, `Sequence 1 Cue 2`.
    #[lum(value)]
    pub scope: Option<lumacore_syntax::ast::Value>,
    #[lum(option = "NoConfirmation")]
    pub confirmed: bool,
}

/// A bare `Recast`: what it takes.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Recast")]
pub struct RecastWhat {}

impl crate::commands::typed::Command for RecastWhat {
    fn run(self, _: &mut AppState) -> DispatchResult {
        crate::commands::run::objects_usage(crate::objects::Verb::Recast)
    }
}

impl crate::commands::typed::Command for RecastPreset {
    fn run(self, state: &mut AppState) -> DispatchResult {
        match recast(state, &self) {
            Ok(r) | Err(r) => r,
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

fn recast(state: &mut AppState, line: &RecastPreset) -> Result<DispatchResult, DispatchResult> {
    let err = |e: String| DispatchResult::Output(e.into());
    let scope = match &line.scope {
        Some(v) => links::cue_scope(state, v).map_err(err)?,
        None => CueScope::All,
    };
    let picked = crate::resolve::resolve_one(state, &line.target, &super::super::PRESET).map_err(err)?;
    let dp = picked.datapool.unwrap_or_else(|| state.selected_datapool());
    let mut plans = Vec::new();
    for m in &picked.members {
        let Some(p) = state.root.show_data.datapools.get(&dp).and_then(|d| d.presets.get(&m.no)) else { continue };
        let guid = p.meta.identity.guid;
        let name = p.meta.name().map(|n| format!(" \"{n}\"")).unwrap_or_default();
        plans.push((m.no, name, guid, links::recast_plan(&state.root.show_data, guid, scope)));
    }
    if plans.is_empty() {
        return Err(err(render_code("No such preset.", "E-CMD-007")));
    }
    let cues: usize = plans.iter().map(|(.., p)| p.len()).sum();
    let shown = plans.iter().map(|(no, name, ..)| format!("Preset {no}{name}")).collect::<Vec<_>>().join(", ");
    if cues == 0 {
        return Ok(DispatchResult::Output(render_success(&format!("Recast {shown}: the cues that follow already match it.")).into()));
    }
    if !line.confirmed {
        let mut gains: Vec<String> = Vec::new();
        let mut loses: Vec<String> = Vec::new();
        for (.., plan) in &plans {
            for r in plan {
                gains.extend(r.gains.iter().filter(|a| !gains.contains(a)).cloned().collect::<Vec<_>>());
                loses.extend(r.loses.iter().filter(|a| !loses.contains(a)).cloned().collect::<Vec<_>>());
            }
        }
        let mut meta = Vec::new();
        if !gains.is_empty() {
            meta.push(format!("{} added", gains.join(", ")));
        }
        if !loses.is_empty() {
            meta.push(format!("{} removed", loses.join(", ")));
        }
        let mut ask = crate::dialogs::Ask::new(
            format!("Recast {shown}"),
            "The cues that use this preset get links for its new attributes and lose the links to the removed ones.",
        )
        .meta(meta.join(" · "), None);
        for (.., plan) in &plans {
            for r in plan {
                let mut what: Vec<String> = r.gains.iter().map(|a| format!("+ {a}")).collect();
                what.extend(r.loses.iter().map(|a| format!("− {a}")));
                let named = if r.name.is_empty() { String::new() } else { format!(" \"{}\"", r.name) };
                ask = ask.row(format!("{}{named}", links::cue_line(state, r.dp, r.sequence, r.cue)), what.join(" · "));
            }
        }
        let numbers = crate::commands::door::numbers_text(&plans.iter().map(|(n, ..)| *n).collect::<Vec<_>>());
        let within = line.scope.as_ref().map(|_| format!(" {}", scope_text(state, scope))).unwrap_or_default();
        let typed = format!("Recast {}Preset {numbers}{within}", links::owner(state, dp));
        ask = ask
            .cancel("Cancel")
            .button(&format!("Recast {}", if cues == 1 { "1 cue".to_string() } else { format!("{cues} cues") }), ButtonKind::Primary, format!("{typed} /NoConfirmation"))
            .flags("/NoConfirmation");
        return Ok(DispatchResult::Ask(Box::new(ask)));
    }
    for (_, _, guid, plan) in &plans {
        links::recast(&mut state.root.show_data, *guid, plan);
    }
    Ok(DispatchResult::Output(render_success(&format!("Recast {shown}: {cues} cue{} changed.", if cues == 1 { "" } else { "s" })).into()))
}

/// The scope as the line names it again (`Sequence 1`, `Sequence 1 Cue 2`).
pub fn scope_text(state: &AppState, scope: CueScope) -> String {
    match scope {
        CueScope::All => String::new(),
        CueScope::Sequence { dp, no } => links::cue_line(state, dp, no, 0).split(" Cue ").next().unwrap_or_default().to_string(),
        CueScope::Cue { dp, no, cue } => links::cue_line(state, dp, no, cue),
    }
}
