// ██████╗ ███████╗██████╗ ██╗     ██╗ ██████╗ █████╗ ████████╗██╗ ██████╗ ███╗   ██╗
// ██╔══██╗██╔════╝██╔══██╗██║     ██║██╔════╝██╔══██╗╚══██╔══╝██║██╔═══██╗████╗  ██║
// ██████╔╝█████╗  ██████╔╝██║     ██║██║     ███████║   ██║   ██║██║   ██║██╔██╗ ██║
// ██╔══██╗██╔══╝  ██╔═══╝ ██║     ██║██║     ██╔══██║   ██║   ██║██║   ██║██║╚██╗██║
// ██║  ██║███████╗██║     ███████╗██║╚██████╗██║  ██║   ██║   ██║╚██████╔╝██║ ╚████║
// ╚═╝  ╚═╝╚══════╝╚═╝     ╚══════╝╚═╝ ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: replication

//! Replication: apply a result from the master to this station.
//!
//! Uses the same `restore_subtrees()` as undo to apply the post-change
//! subtree snapshots (idempotent and convergent), then replays what the
//! command did to undo stacks, so a member's copy matches the master's
//! (decisions §21 "6b").
//!
//! Owns:          applying a replicated op on a member (subtree snapshots, undo steps, the session runtime).
//! Doesn't own:   the journal (lumacore-tree), sending ops (link).
//! Entry points:  apply_replicated_op.
//! Decisions:     §10, §21 "6b", §34 19.3.

use lumacore_tree::ReplicationOp;
use lumacore_tree::runtime::{JournalEntry, JournalResult};

use lumacore_objects::AppState;
use lumacore_objects::commands::oops;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Apply a replicated operation to local state.
///
/// Sets `replaying_remote_op` so nothing is filed again (no undo frame, no
/// outbox), updates `subtree_owners` for conflict detection, records the op
/// in `Runtime.Journal` and remembers it as the last one applied.
pub fn apply_replicated_op(state: &mut AppState, op: &ReplicationOp) -> Result<(), String> {
    state.replaying_remote_op = true;
    let restored = if op.post_snapshots.is_empty() {
        Ok(())
    } else {
        oops::restore_subtrees(state, &op.post_snapshots)
    };
    if let Err(e) = restored {
        state.replaying_remote_op = false;
        return Err(e);
    }
    lumacore_objects::commands::views::follow_touched_profiles(state);
    for step in &op.undo {
        // An Oops / Redo of a step with station-local state: this station
        // puts its own part back (decisions §29 "11c").
        match &step.step {
            lumacore_tree::UndoStep::Undo { redo } => {
                oops::restore_local(state, &redo.local, oops::Side::Before)
            }
            lumacore_tree::UndoStep::Redo { undo } => {
                oops::restore_local(state, &undo.local, oops::Side::After)
            }
            lumacore_tree::UndoStep::Push(_) => {}
        }
        state.undo_stack_mut(&step.user).apply_step(step.step.clone());
    }
    state.replaying_remote_op = false;

    for snap in &op.post_snapshots {
        state
            .subtree_owners
            .insert(snap.subtree.clone(), (op.user.clone(), op.seq));
    }
    if !op.command.is_empty() {
        state.root.runtime.journal.record(JournalEntry {
            time_ms: op.authority_timestamp_ms,
            epoch: op.epoch,
            seq: 0,
            user: op.user.clone(),
            station: op.station.clone(),
            command: op.command.clone(),
            changed: op.affected_subtrees.iter().map(|s| s.to_string()).collect(),
            result: JournalResult::Ok,
        });
    }
    state.last_applied = Some((op.journal, op.seq));
    Ok(())
}
