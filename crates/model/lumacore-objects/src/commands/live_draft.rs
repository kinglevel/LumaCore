// ██╗     ██╗██╗   ██╗███████╗    ██████╗ ██████╗  █████╗ ███████╗████████╗
// ██║     ██║██║   ██║██╔════╝    ██╔══██╗██╔══██╗██╔══██╗██╔════╝╚══██╔══╝
// ██║     ██║██║   ██║█████╗      ██║  ██║██████╔╝███████║█████╗     ██║
// ██║     ██║╚██╗ ██╔╝██╔══╝      ██║  ██║██╔══██╗██╔══██║██╔══╝     ██║
// ███████╗██║ ╚████╔╝ ███████╗    ██████╔╝██║  ██║██║  ██║██║        ██║
// ╚══════╝╚═╝  ╚═══╝  ╚══════╝    ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝        ╚═╝
// FILE: live_draft

//! Live lines under the Devices draft (decisions §34 "34g4 patch hardened" 8, the Live Patch):
//! while someone holds the draft, another user's `Set` on a declared live column
//! (`Props::live`), Assign / Unassign `… At DMX` (an address is live, round 3 18) and every Oops /
//! Redo run against the live devices — the normal change path
//! records them (undone, replicated) — and the draft then follows cell by cell, unless the holder
//! changed that cell (refused, naming them).
//!
//! Owns:          runs_live, rebase_draft.
//! Doesn't own:   the draft (`lumacore_state::patch_draft`), the swap around the line (dispatch).
//! Entry points:  runs_live, rebase_draft.
//! Decisions:     §34 "34g4 patch hardened" 8, §11 "Issue 11 hardening".

use crate::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Whether `line` (one statement) runs against the live devices: a draft is open and it is an
/// Oops / Redo (the draft files no steps, so a Devices step is a live one) or another user's
/// `Set` on a live column.
pub fn runs_live(state: &AppState, line: &str) -> bool {
    // Only the outer statement swaps (a stored line's statements run inside it).
    if state.patch_draft.is_none() || state.dispatch_depth > 0 {
        return false;
    }
    let tokens = super::tokenize(line);
    let Some(first) = tokens.first() else { return false };
    if super::registry::is_oops(first) || super::registry::is_command(first, "redo") {
        return true;
    }
    if state.patch_draft_of_other().is_none() {
        return false;
    }
    // A fixture's addresses are a live column: Assign / Unassign … At DMX (round 3, 18).
    if super::registry::is_command(first, "assign") || super::registry::is_command(first, "unassign") {
        let dmx = crate::objects::patch::dmx::DMX.keyword;
        let at = tokens.iter().position(|t| super::registry::is_command(t, "at"));
        return at.is_some_and(|i| tokens[i + 1..].iter().any(|t| t.eq_ignore_ascii_case(dmx)));
    }
    if !super::registry::is_command(first, "set") {
        return false;
    }
    let args: Vec<&str> = tokens[1..].iter().map(String::as_str).collect();
    matches!(super::set::named_object(state, &args), Some((obj, Some(prop))) if obj.props.is_some_and(|p| p.is_live(prop.name)))
}

/// Which patch-row cells of which fixtures a live line changed, by comparing the live devices
/// before (`base`) and after it: the fields the grid shows (`name`, `note`, `address`, the
/// position, `master_react`).
fn live_cells(base: &serde_json::Value, after: &lumacore_tree::DevicesData) -> Vec<(uuid::Uuid, Vec<&'static str>)> {
    let Ok(before) = serde_json::from_value::<lumacore_tree::DevicesData>(base.clone()) else { return Vec::new() };
    let was: std::collections::BTreeMap<uuid::Uuid, &lumacore_tree::fixtures::Fixture> =
        before.dmx.patch.fixtures.values().map(|f| (f.meta.identity.guid, f)).collect();
    after
        .dmx
        .patch
        .fixtures
        .values()
        .filter_map(|f| {
            let old = was.get(&f.meta.identity.guid)?;
            let mut cells = Vec::new();
            if f.meta.identity.name != old.meta.identity.name {
                cells.push("name");
            }
            if f.meta.annotation.note != old.meta.annotation.note {
                cells.push("note");
            }
            if json(&f.mvr.breaks) != json(&old.mvr.breaks) {
                cells.push("address");
            }
            if json(&f.mvr.matrix) != json(&old.mvr.matrix) {
                cells.extend(["pos_x", "pos_y", "pos_z", "rot_x", "rot_y", "rot_z"]);
            }
            if json(&f.luma.master_react) != json(&old.luma.master_react) {
                cells.push("master_react");
            }
            (!cells.is_empty()).then_some((f.meta.identity.guid, cells))
        })
        .collect()
}

/// A show value as JSON, to compare two of them (no `PartialEq` needed on every type).
fn json<T: serde::Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or_default()
}

/// After a live line (the live devices still in the show): the draft — kept aside — follows
/// what changed from `base`. Err = a cell the holder changed too: the refusal to reply.
pub fn rebase_draft(state: &mut AppState, base: &serde_json::Value) -> Result<(), String> {
    let theirs = serde_json::to_value(&state.root.show_data.devices).unwrap_or_default();
    let changed = live_cells(base, &state.root.show_data.devices);
    let who = state.effective_user_name().to_string();
    let Some(draft) = state.patch_draft.as_mut() else { return Ok(()) };
    let ours = serde_json::to_value(&draft.live).unwrap_or_default();
    match crate::patch_draft::rebase(base, &ours, &theirs) {
        Ok(v) => {
            match serde_json::from_value(v) {
                Ok(devices) => {
                    draft.live = devices;
                    // The violet dot: these cells came from someone's live edit.
                    for (guid, fields) in changed {
                        let cells = draft.marks.entry(guid).or_default();
                        for f in fields {
                            cells.insert(f.to_string(), who.clone());
                        }
                    }
                }
                // A rebase that doesn't read back as devices is refused like a conflict.
                Err(e) => return Err(crate::errors::render_code(format!("The draft can't follow that change ({e})."), "E-PATCH-008")),
            }
            Ok(())
        }
        Err(cells) => Err(crate::errors::render_code(
            format!(
                "{} changed {} in the Devices draft: that cell waits until they keep or discard it.",
                draft.owner,
                cells.first().map_or("it", String::as_str)
            ),
            "E-PATCH-008",
        )),
    }
}
