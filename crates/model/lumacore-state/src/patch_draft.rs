// ██████╗  █████╗ ████████╗ ██████╗██╗  ██╗    ██████╗ ██████╗  █████╗ ███████╗████████╗
// ██╔══██╗██╔══██╗╚══██╔══╝██╔════╝██║  ██║    ██╔══██╗██╔══██╗██╔══██╗██╔════╝╚══██╔══╝
// ██████╔╝███████║   ██║   ██║     ███████║    ██║  ██║██████╔╝███████║█████╗     ██║
// ██╔═══╝ ██╔══██║   ██║   ██║     ██╔══██║    ██║  ██║██╔══██╗██╔══██║██╔══╝     ██║
// ██║     ██║  ██║   ██║   ╚██████╗██║  ██║    ██████╔╝██║  ██║██║  ██║██║        ██║
// ╚═╝     ╚═╝  ╚═╝   ╚═╝    ╚═════╝╚═╝  ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝        ╚═╝
// FILE: patch_draft

//! The staged patch (decisions §11 "Menus, LumaSession and staged patch", §34 "34g4 patch
//! hardened" 8; Patch vs Live Patch).
//!
//! Claiming Devices takes a **draft**: from then on `ShowData.devices` is the draft and the live
//! devices are kept here. The rig keeps running on the live devices (the state thread publishes
//! them in place of the draft), the holder's edits are neither undone nor replicated one by one,
//! and other users' device edits are refused — except on the declared live columns
//! (`Props::live`): such a line runs against the live devices ([`AppState::swap_live_devices`],
//! the normal change path: undone, replicated) and the draft follows it cell by cell
//! ([`rebase`]) unless the holder changed that cell. Leaving asks Keep / Discard / Cancel: Keep
//! files the whole draft as one change (one undo step, one replication op), Discard puts the
//! live devices back.
//!
//! Owns:          PatchDraft, the draft's begin / keep-check / discard / swap, live_view, rebase.
//! Doesn't own:   the claim (`lumacore-objects` claims), which lines are live (`Props::live`, dispatch).
//! Entry points:  PatchDraft, AppState::begin_patch_draft, swap_live_devices, live_view, rebase.
//! Decisions:     §11 "Issue 11 hardening", §34 "34g4 patch hardened" 8.


use lumacore_tree::{DevicesData, ShowData};
use serde_json::Value;

use super::AppState;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The live devices while someone edits a draft of them.
#[derive(Debug, Clone)]
pub struct PatchDraft {
    pub owner: String,
    pub live: DevicesData,
    /// Cells another user's live edit brought into the draft (decisions §34 round 3, the violet
    /// dot): fixture GUID → the patch row's field (`name`, `address`, `pos_x` …) → who; kept until
    /// the holder saves or discards.
    pub marks: std::collections::BTreeMap<uuid::Uuid, std::collections::BTreeMap<String, String>>,
    /// The open MVR import review (decisions §34 "Import MVR hardened" 3): what a file would
    /// change, nothing written; Apply writes the ticked rows, Cancel drops it, Keep is refused
    /// while it stands.
    pub review: Option<crate::mvr_review::ImportReview>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl AppState {
    /// Start a draft (claiming Devices).
    pub fn begin_patch_draft(&mut self) {
        if self.patch_draft.is_none() {
            self.patch_draft = Some(PatchDraft {
                owner: self.effective_user_name().to_string(),
                live: self.root.show_data.devices.clone(),
                marks: Default::default(),
                review: None,
            });
            // Nothing in the show changed, but what readers see did: the draft is published
            // for its holder (the Patch dialog's `patch` feed).
            self.show_generation += 1;
        }
    }

    /// Whether the draft differs from the live devices — an open MVR review counts (decisions §34
    /// "Import MVR hardened" 3: leaving asks, Keep is refused until Apply or Cancel).
    pub fn patch_draft_changed(&self) -> bool {
        self.patch_draft
            .as_ref()
            .is_some_and(|d| d.review.is_some() || !same(&d.live, &self.root.show_data.devices))
    }

    /// The holder of the open draft, if it isn't the current user.
    pub fn patch_draft_of_other(&self) -> Option<&str> {
        self.patch_draft
            .as_ref()
            .map(|d| d.owner.as_str())
            .filter(|o| !o.eq_ignore_ascii_case(self.effective_user_name()))
    }

    /// Drop the draft: the live devices come back.
    pub fn discard_patch_draft(&mut self) {
        if let Some(d) = self.patch_draft.take() {
            self.root.show_data.devices = d.live;
            self.root.show_data.after_load();
            self.show_generation += 1;
            self.patch_generation += 1;
        }
    }

    /// End the draft without touching the show (a new show was loaded).
    pub fn forget_patch_draft(&mut self) {
        if self.patch_draft.take().is_some() {
            // Its holder sees the live devices again (the published draft goes).
            self.show_generation += 1;
        }
    }

    /// Swap the draft and the live devices: `ShowData.devices` is the live one until the next
    /// swap (a live line runs in between). No-op without a draft.
    pub fn swap_live_devices(&mut self) {
        if let Some(d) = self.patch_draft.as_mut() {
            std::mem::swap(&mut self.root.show_data.devices, &mut d.live);
        }
    }
}

/// The show as readers and the engine see it: the live devices while a draft is open.
pub fn live_view(state: &AppState, mut show: ShowData) -> ShowData {
    if let Some(d) = &state.patch_draft {
        show.devices = d.live.clone();
    }
    show
}

/// The draft after the live devices moved from `base` to `theirs` (decisions §34 "34g4 patch
/// hardened" 8): every cell the holder left as it was (`ours` == `base`) follows the live one;
/// a cell both changed differently is a conflict (its path). Objects merge key by key; anything
/// else (numbers, text, lists) is one cell.
pub fn rebase(base: &Value, ours: &Value, theirs: &Value) -> Result<Value, Vec<String>> {
    let mut conflicts = Vec::new();
    let out = cell(base, ours, theirs, "", &mut conflicts);
    if conflicts.is_empty() { Ok(out.unwrap_or(Value::Null)) } else { Err(conflicts) }
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

fn same(a: &DevicesData, b: &DevicesData) -> bool {
    serde_json::to_vec(a).ok() == serde_json::to_vec(b).ok()
}

/// One cell (None = gone).
fn cell(base: &Value, ours: &Value, theirs: &Value, path: &str, conflicts: &mut Vec<String>) -> Option<Value> {
    if ours == base {
        return Some(theirs.clone());
    }
    if theirs == base || theirs == ours {
        return Some(ours.clone());
    }
    let (Value::Object(b), Value::Object(o), Value::Object(t)) = (base, ours, theirs) else {
        conflicts.push(path.to_string());
        return Some(ours.clone());
    };
    let mut out = serde_json::Map::new();
    let keys: std::collections::BTreeSet<&String> = b.keys().chain(o.keys()).chain(t.keys()).collect();
    for k in keys {
        let at = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
        let side = |m: &serde_json::Map<String, Value>| m.get(k).cloned().unwrap_or(Value::Null);
        if let Some(v) = cell(&side(b), &side(o), &side(t), &at, conflicts).filter(|v| !v.is_null() || o.contains_key(k)) {
            out.insert(k.clone(), v);
        }
    }
    Some(Value::Object(out))
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
