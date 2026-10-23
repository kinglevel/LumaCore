// ██╗   ██╗██████╗ ██████╗  █████╗ ████████╗███████╗
// ██║   ██║██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝██╔════╝
// ██║   ██║██████╔╝██║  ██║███████║   ██║   █████╗
// ██║   ██║██╔═══╝ ██║  ██║██╔══██║   ██║   ██╔══╝
// ╚██████╔╝██║     ██████╔╝██║  ██║   ██║   ███████╗
//  ╚═════╝ ╚═╝     ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚══════╝
// FILE: update

//! `Update Sequence <m> [Cue <n>] [/Original | /Active] [/Shield]` (decisions 0293): your
//! programmer's plain values go back into a cue — where each value was stored (`/Original`, the
//! default: the cue it tracks from) or into this cue (`/Active`). Only values the cue already
//! plays change (a value that came from a preset never does: its cues follow the preset).
//! `/Shield` protects the next cues: the cue after this one keeps the old value where it only
//! tracked it. Without a cue: the sequence's current cue.
//!
//! Owns:          UpdateSequence, update_cue.
//! Doesn't own:   the Update ask (`objects::preset::update`), tracking (`SequenceObject::tracked`).
//! Entry points:  UpdateSequence, update_cue.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract.

use lumacore_tree::{CueData, CueValue, Value};
use uuid::Uuid;

use crate::errors::render_code;
use crate::objects::preset::links::{self, CueScope};
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
#[lum(verb = "Update", object = "Sequence")]
pub struct UpdateSequence {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
    #[lum(option = "Original")]
    pub original: bool,
    #[lum(option = "Active")]
    pub active: bool,
    #[lum(option = "Shield")]
    pub shield: bool,
}

impl crate::commands::typed::Command for UpdateSequence {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let mut run = || -> Result<String, String> {
            if self.original && self.active {
                return Err(render_code("Update a cue /Original or /Active, not both.", "E-CMD-006"));
            }
            let (dp, no, cue) = match links::cue_scope(state, &lumacore_syntax::ast::Value::Target(self.target.clone()))? {
                CueScope::Cue { dp, no, cue } => (dp, no, cue),
                CueScope::Sequence { dp, no } => {
                    let guid = state.root.show_data.datapools.get(&dp).and_then(|d| d.sequences.get(&no)).map(|s| s.meta.identity.guid);
                    let current = guid.and_then(|g| state.root.show_data.playbacks.live.sequences.get(&g)).map(|st| st.current_cue);
                    let cue = current.ok_or_else(|| render_code(format!("Sequence {no} isn't running: name its cue (Update Sequence {no} Cue 2)."), "E-PLAY-003"))?;
                    (dp, no, cue)
                }
                CueScope::All => return Err(render_code("Name a sequence's cue.", "E-CMD-006")),
            };
            update_cue(state, dp, no, cue, self.active, self.shield)
        };
        DispatchResult::Output(run().unwrap_or_else(|e| e).into())
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Write the acting user's plain values into the cue `cue` of sequence `no` (`active`: into this
/// cue, else where each was stored), shielding the next cue when asked.
pub fn update_cue(state: &mut AppState, dp: u32, no: u32, cue: u32, active: bool, shield: bool) -> Result<String, String> {
    let Some(prog) = state.current_programmer() else {
        return Err(render_code("No programmer available.", "E-CMD-011"));
    };
    let mine: Vec<(Uuid, String, Value)> = prog
        .values
        .iter()
        .filter(|(_, pv)| pv.active && !pv.value.release && pv.origin.is_none() && pv.value.preset.is_none())
        .map(|((f, a), pv)| (*f, a.clone(), links::plain(&pv.value)))
        .collect();
    let seq = state
        .root
        .show_data
        .datapools
        .get(&dp)
        .and_then(|d| d.sequences.get(&no))
        .ok_or_else(|| render_code(format!("Sequence {no} doesn't exist."), "E-CMD-007"))?;
    if cue != lumacore_tree::CUE_ZERO && !seq.cues.contains_key(&cue) {
        return Err(render_code(format!("Sequence {no} has no Cue {}.", lumacore_tree::cue_label(cue)), "E-PLAY-001"));
    }
    let tracked = seq.tracked(cue);
    let next = if cue == lumacore_tree::CUE_ZERO { seq.cues.keys().next().copied() } else { seq.cues.range(cue + 1..).next().map(|(k, _)| *k) };
    let after = next.filter(|_| shield && seq.settings.tracking).map(|n| seq.tracked(n));
    // (fixture, attribute, new value, the cue it goes to, the next cue's shield value)
    let mut writes: Vec<(Uuid, String, Value, u32, Option<CueData>)> = Vec::new();
    for (f, a, v) in mine {
        let Some(t) = tracked.get(&(f, a.as_str())) else { continue };
        let Some(stored) = t.value.plain() else { continue };
        if *stored == v {
            continue;
        }
        let into = if active { cue } else { t.cue };
        let guard = after.as_ref().and_then(|m| m.get(&(f, a.as_str()))).filter(|n| Some(n.cue) != next).map(|_| t.value.clone());
        writes.push((f, a, v, into, guard));
    }
    if writes.is_empty() {
        return Err(render_code(format!("Update Sequence {no} Cue {}: your programmer changed none of its plain values.", lumacore_tree::cue_label(cue)), "E-CMD-009"));
    }
    let count = writes.len();
    let mut shielded = 0;
    let mut homes: Vec<u32> = Vec::new();
    let seq = state.root.show_data.sequence_mut(dp, no).expect("looked up above");
    for (f, a, v, into, guard) in writes {
        if !homes.contains(&into) {
            homes.push(into);
        }
        put(seq, into, f, &a, CueData::Value(v));
        if let (Some(old), Some(n)) = (guard, next) {
            put(seq, n, f, &a, old);
            shielded += 1;
        }
    }
    let label = |k: u32| if k == lumacore_tree::CUE_ZERO { "CueZero".to_string() } else { format!("Cue {}", lumacore_tree::cue_label(k)) };
    let place = homes.iter().map(|k| label(*k)).collect::<Vec<_>>().join(", ");
    let shield_note = match (shielded, next) {
        (0, _) | (_, None) => String::new(),
        (n, Some(k)) => format!("; {n} kept in {} (/Shield)", label(k)),
    };
    Ok(render_success(&format!(
        "Updated Sequence {no} {} /{}: {count} value{} in {place}{shield_note}.",
        label(cue),
        if active { "Active" } else { "Original" },
        if count == 1 { "" } else { "s" }
    )))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// Put a value into cue `key`: over the cue's own value of the attribute (its timing kept), else
/// into part 0.
fn put(seq: &mut lumacore_tree::SequenceObject, key: u32, fixture: Uuid, attribute: &str, value: CueData) {
    let cue = if key == lumacore_tree::CUE_ZERO { &mut seq.cue_zero } else {
        match seq.cues.get_mut(&key) {
            Some(c) => c,
            None => return,
        }
    };
    for part in cue.parts.values_mut() {
        if let Some(v) = part.values.iter_mut().find(|v| v.fixture_id == fixture && v.attribute == attribute) {
            v.value = value;
            return;
        }
    }
    cue.part0().values.push(CueValue { fixture_id: fixture, attribute: attribute.to_string(), value, timing: Default::default() });
}
