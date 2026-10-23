//! The gesture channel (decisions §30 "12f hardening": not a `/Quiet` option — anyone could
//! hide any line with it). A continuous control (a fader, an encoder) sends a real command line
//! per move; it passes the same rights gate, dispatch and replication as a typed line, but quietly:
//! no echo, no command history, no Usage, no journal entry per move. Its end files ONE journal
//! entry and one System Monitor line ("Fader Master 2.1: 100 → 63.5 · Admin") under "Faders".
//! Only kit controls send it (the shell's `gesture` packet); the same line typed stays a normal
//! command. A member forwards the mark with the line, so the master stays quiet too.

use serde::{Deserialize, Serialize};

/// What the running line is to the channel (`AppState::gesture`). `id` names the gesture (one
/// drag), so its end finds where it began.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GestureLine {
    /// One move: nothing recorded, no Oops step.
    Move { id: String },
    /// The last line of a gesture: the one entry, `summary` saying what moved, and one Oops step
    /// from where the gesture began.
    End { id: String, summary: String },
}

/// How many gestures may be under way at once on a station (a lost end leaves one behind).
const MAX_OPEN: usize = 64;

/// Before a gesture's line runs (dispatch, on the station that runs it — the master for a
/// member): its end first puts back every subtree its moves changed, as they were before the
/// first move, so the end's Oops step spans the whole drag (decisions §30 "12f": a drag is one
/// Oops step — programmer values, cue timing, whatever the line changes). Masters are never undone.
pub fn before_line(state: &mut crate::AppState) {
    let Some(GestureLine::End { id, .. }) = state.gesture.clone() else { return };
    let user = state.effective_user_name().to_string();
    if let Some(start) = state.gesture_starts.remove(&(user, id)) {
        let _ = crate::oops::restore_subtrees(state, &start);
    }
}

/// After a gesture's move changed `changes`: keep each subtree as it was before the gesture's
/// first move (the end puts them back first).
pub fn remember(state: &mut crate::AppState, changes: &[crate::change::Change]) {
    let Some(GestureLine::Move { id }) = state.gesture.clone() else { return };
    if changes.is_empty() {
        return;
    }
    let key = (state.effective_user_name().to_string(), id);
    if !state.gesture_starts.contains_key(&key) && state.gesture_starts.len() >= MAX_OPEN {
        state.gesture_starts.clear();
    }
    let kept = state.gesture_starts.entry(key).or_default();
    for c in changes {
        if !kept.iter().any(|s| s.subtree == c.subtree) {
            kept.push(lumacore_tree::undo::SubtreeSnapshot { subtree: c.subtree.clone(), data: c.before.clone() });
        }
    }
}

/// The System Monitor category of a gesture's entry (a filter that can be hidden).
pub const SYSMON_CATEGORY: &str = "Faders";
