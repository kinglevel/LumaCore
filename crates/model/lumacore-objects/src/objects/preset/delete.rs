// ██████╗ ███████╗██╗     ███████╗████████╗███████╗
// ██╔══██╗██╔════╝██║     ██╔════╝╚══██╔══╝██╔════╝
// ██║  ██║█████╗  ██║     █████╗     ██║   █████╗
// ██║  ██║██╔══╝  ██║     ██╔══╝     ██║   ██╔══╝
// ██████╔╝███████╗███████╗███████╗   ██║   ███████╗
// ╚═════╝ ╚══════╝╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: delete

//! Deleting a preset that cues use (decisions 0293): the delete ask names those cues and offers
//! "Delete, keep the values" — each linking cue gets the preset's values first (an Extract), then
//! the preset goes, all in the one line: Oops brings back the preset and every link. A preset no
//! cue uses asks the door's plain question.
//!
//! Owns:          DeletePreset, delete_ask.
//! Doesn't own:   the door's delete (`commands::door`), the Extract (`links::extract_cues`).
//! Entry points:  DeletePreset, delete_ask.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract, §35 7.

use lumacore_tree::PresetUses;
use lumacore_wire::{ButtonKind, Tone};

use super::links::{self, CueScope};
use crate::objects::DeleteAsk;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Delete", object = "Preset")]
pub struct DeletePreset {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
}

impl crate::commands::typed::Command for DeletePreset {
    fn run(self, state: &mut AppState) -> DispatchResult {
        // The cues that link them keep the values first (one line: one Oops step).
        if let Ok(picked) = crate::resolve::resolve_one(state, &self.target, &super::super::PRESET) {
            let dp = picked.datapool.unwrap_or_else(|| state.selected_datapool());
            let guids: Vec<uuid::Uuid> = picked
                .members
                .iter()
                .filter_map(|m| state.root.show_data.datapools.get(&dp).and_then(|d| d.presets.get(&m.no)).map(|p| p.meta.identity.guid))
                .collect();
            for g in guids {
                links::extract_cues(&mut state.root.show_data, Some(g), CueScope::All);
            }
        }
        crate::commands::door::delete(state, &super::super::PRESET, &self.target)
    }
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The delete ask: the door's, or — when cues link the presets — one naming those cues with
/// "Delete, keep the values".
pub fn delete_ask(state: &AppState, args: &[&str]) -> DeleteAsk {
    let plain = crate::commands::door::delete_ask(state, &super::super::PRESET, args);
    let DeleteAsk::Ask(door) = &plain else { return plain };
    let Some(line) = door.line_of("Delete") else { return plain };
    let words: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with('/')).collect();
    let owned = words.first().is_some_and(|w| !w.starts_with('"') && crate::objects::find(w).is_some());
    let text = if owned { words.join(" ") } else { format!("Preset {}", words.join(" ")) };
    let Ok(target) = crate::resolve::target_of(&text) else { return plain };
    let Ok(picked) = crate::resolve::resolve_one(state, &target, &super::super::PRESET) else { return plain };
    let dp = picked.datapool.unwrap_or_else(|| state.selected_datapool());
    let show = &state.root.show_data;
    let uses = PresetUses::build(show);
    let mut cues: Vec<String> = Vec::new();
    let mut title = String::new();
    for m in &picked.members {
        let Some(p) = show.datapools.get(&dp).and_then(|d| d.presets.get(&m.no)) else { continue };
        if title.is_empty() {
            title = format!("Delete Preset {}{}", m.no, p.meta.name().map(|n| format!(" \"{n}\"")).unwrap_or_default());
        }
        for u in uses.cues(p.meta.identity.guid) {
            let named = if u.name.is_empty() { String::new() } else { format!(" \"{}\"", u.name) };
            let c = format!("{}{named}", links::cue_line(state, u.datapool, u.sequence, u.cue));
            if !cues.contains(&c) {
                cues.push(c);
            }
        }
    }
    if cues.is_empty() {
        return plain;
    }
    if picked.members.len() > 1 {
        title = format!("Delete {} presets", picked.members.len());
    }
    let n = cues.len();
    let mut ask = crate::dialogs::Ask::new(
        title,
        "These cues keep the preset's current values as plain values, and their links are dropped. Oops puts the preset and every link back.",
    )
    .meta(if n == 1 { "1 cue uses it".to_string() } else { format!("{n} cues use it") }, Some(Tone::Warn));
    for c in cues {
        ask = ask.detail(c, None);
    }
    DeleteAsk::Ask(Box::new(ask.cancel("Cancel").button("Delete, keep the values", ButtonKind::Danger, line).flags("/NoConfirmation")))
}
