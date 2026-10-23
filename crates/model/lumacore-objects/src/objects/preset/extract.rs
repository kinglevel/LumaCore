// ███████╗██╗  ██╗████████╗██████╗  █████╗  ██████╗████████╗
// ██╔════╝╚██╗██╔╝╚══██╔══╝██╔══██╗██╔══██╗██╔════╝╚══██╔══╝
// █████╗   ╚███╔╝    ██║   ██████╔╝███████║██║        ██║
// ██╔══╝   ██╔██╗    ██║   ██╔══██╗██╔══██║██║        ██║
// ███████╗██╔╝ ██╗   ██║   ██║  ██║██║  ██║╚██████╗   ██║
// ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝   ╚═╝
// FILE: extract

//! `Extract Preset <ids> [Sequence <m> [Cue <n>]]` (decisions 0293): links to a preset become the
//! preset's values as they are now — in your programmer (no sequence named), or in the cues of a
//! sequence or one cue. `Extract Sequence <m> [Cue <n>]` (the Sequence object's) does it for every
//! preset linked there. Deleting a used preset extracts its cues first.
//!
//! Owns:          ExtractPreset.
//! Doesn't own:   the writes (`links::extract_cues`, `links::extract_programmer`), the Sequence
//!                form (`commands::playback::extract`).
//! Entry points:  ExtractPreset.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract.

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
#[lum(verb = "Extract", object = "Preset")]
pub struct ExtractPreset {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    /// In these cues instead of the programmer: `Sequence 4`, `Sequence 4 Cue 9`.
    #[lum(value)]
    pub scope: Option<lumacore_syntax::ast::Value>,
}

/// A bare `Extract`: what it takes.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Extract")]
pub struct ExtractWhat {}

impl crate::commands::typed::Command for ExtractWhat {
    fn run(self, _: &mut AppState) -> DispatchResult {
        crate::commands::run::objects_usage(crate::objects::Verb::Extract)
    }
}

impl crate::commands::typed::Command for ExtractPreset {
    fn run(self, state: &mut AppState) -> DispatchResult {
        DispatchResult::Output(extract(state, &self).unwrap_or_else(|e| e).into())
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn extract(state: &mut AppState, line: &ExtractPreset) -> Result<String, String> {
    let scope = line.scope.as_ref().map(|v| links::cue_scope(state, v)).transpose()?;
    let picked = crate::resolve::resolve_one(state, &line.target, &super::super::PRESET)?;
    let dp = picked.datapool.unwrap_or_else(|| state.selected_datapool());
    let guids: Vec<(u32, uuid::Uuid)> = picked
        .members
        .iter()
        .filter_map(|m| state.root.show_data.datapools.get(&dp).and_then(|d| d.presets.get(&m.no)).map(|p| (m.no, p.meta.identity.guid)))
        .collect();
    if guids.is_empty() {
        return Err(render_code("No such preset.", "E-CMD-007"));
    }
    let shown = guids.iter().map(|(n, _)| format!("Preset {n}")).collect::<Vec<_>>().join(" + ");
    let count = match scope {
        None => links::extract_programmer(state, &guids.iter().map(|(_, g)| *g).collect::<Vec<_>>()),
        Some(scope) => guids.iter().map(|(_, g)| links::extract_cues(&mut state.root.show_data, Some(*g), scope)).sum(),
    };
    let place = match scope {
        None => "your programmer".to_string(),
        Some(CueScope::All) => "every cue".to_string(),
        Some(s) => super::recast::scope_text(state, s),
    };
    if count == 0 {
        return Err(render_code(format!("Extract {shown}: nothing in {place} links it."), "E-CMD-009"));
    }
    Ok(render_success(&format!("Extract {shown}: {count} value{} in {place} plain now.", if count == 1 { "" } else { "s" })))
}
