//! Replication types for LumaSession synchronization (decisions §10, §21 "6b").
//!
//! Operations flow: originator -> master -> all stations. The master is the
//! single source of truth: it executes commands and sends the **results**
//! (post-change subtree snapshots, never command replay), each with
//! `(epoch, seq)`, plus what the command did to undo stacks and the
//! session-wide runtime. This reuses the `SubtreeSnapshot` / `ShowSubtree`
//! types from the undo system.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::undo::{ShowSubtree, SubtreeSnapshot, UndoStep, UserUndoStep};

/// Operations a master keeps for catching up (decisions §21 "6b").
pub const DEFAULT_MAX_ENTRIES: usize = 1000;
/// Bytes of snapshots a master keeps for catching up (decisions §31): every op carries whole
/// subtrees, so 1000 ops of a big Playback could otherwise reach a gigabyte. A member behind
/// the trimmed range gets a full snapshot.
pub const DEFAULT_MAX_BYTES: usize = 64 * 1024 * 1024;

/// A change waiting for the master to sequence it.
///
/// Created by the change path after a mutating command succeeds; the master
/// commits it to its [`ReplicationJournal`] after the job.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingReplicationOp {
    /// The user who performed the command.
    pub user: String,
    /// The command text (passwords masked).
    pub command: String,
    /// Which subtrees this command touched.
    pub affected_subtrees: Vec<ShowSubtree>,
    /// Post-mutation snapshots of the affected subtrees.
    pub post_snapshots: Vec<SubtreeSnapshot>,
    /// What the command did to undo stacks (members replay it).
    #[serde(default)]
    pub undo: Vec<UserUndoStep>,
    /// The session-wide runtime after the command, when it changed (modes,
    /// claims, variables, members), as JSON.
    #[serde(default)]
    pub runtime: Option<serde_json::Value>,
}

/// A sequenced, master-committed operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationOp {
    /// The journal that numbered it: a new one starts with every master
    /// and every show load, so a station can tell "just behind" from "start
    /// over".
    pub journal: Uuid,
    /// The master's epoch (6c bumps it on every election; 1 until then).
    pub epoch: u64,
    /// Monotonic sequence number within the journal.
    pub seq: u64,
    /// The station whose client sent the command.
    #[serde(default)]
    pub station: String,
    /// The user who performed the command.
    pub user: String,
    /// The command text (passwords masked).
    pub command: String,
    /// Which subtrees this command touched.
    pub affected_subtrees: Vec<ShowSubtree>,
    /// Post-mutation snapshots of the affected subtrees.
    pub post_snapshots: Vec<SubtreeSnapshot>,
    /// What the command did to undo stacks.
    #[serde(default)]
    pub undo: Vec<UserUndoStep>,
    /// The session-wide runtime after the command, when it changed.
    #[serde(default)]
    pub runtime: Option<serde_json::Value>,
    /// Master wall-clock timestamp (unix millis).
    pub authority_timestamp_ms: u64,
}

impl ReplicationOp {
    /// Bytes this op holds: its snapshots, the undo frames it carries and its text (the
    /// runtime JSON is small and not counted).
    pub fn size_bytes(&self) -> usize {
        let snapshots: usize = self.post_snapshots.iter().map(|s| s.data.len()).sum();
        let undo: usize = self
            .undo
            .iter()
            .map(|u| match &u.step {
                UndoStep::Push(frame) | UndoStep::Undo { redo: frame } | UndoStep::Redo { undo: frame } => {
                    frame.size_bytes
                }
            })
            .sum();
        snapshots + undo + self.command.len() + self.user.len() + self.station.len()
    }
}

/// Ring-buffer journal of committed operations on the master.
///
/// Stores the most recent operations so that stations can catch up after a
/// brief disconnection without a full ShowData snapshot.
#[derive(Debug, Clone)]
pub struct ReplicationJournal {
    /// This journal's identity (see [`ReplicationOp::journal`]).
    id: Uuid,
    /// The master's epoch.
    epoch: u64,
    /// Next sequence number to assign.
    next_seq: u64,
    /// Ring buffer of committed operations (oldest at front).
    log: VecDeque<ReplicationOp>,
    /// Per-station last-acknowledged sequence number.
    station_acks: BTreeMap<String, u64>,
    /// Maximum number of entries to retain.
    max_entries: usize,
    /// Bytes held by `log` ([`ReplicationOp::size_bytes`]).
    bytes: usize,
    /// Maximum bytes to retain (the newest op always stays).
    max_bytes: usize,
}

impl Default for ReplicationJournal {
    fn default() -> Self {
        Self::with_limits(DEFAULT_MAX_ENTRIES, DEFAULT_MAX_BYTES)
    }
}

impl ReplicationJournal {
    /// Create a new journal keeping at most `max_entries` ops (and [`DEFAULT_MAX_BYTES`]).
    pub fn new(max_entries: usize) -> Self {
        Self::with_limits(max_entries, DEFAULT_MAX_BYTES)
    }

    /// Create a new journal trimmed at whichever limit is reached first.
    pub fn with_limits(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            epoch: 1,
            next_seq: 1,
            log: VecDeque::new(),
            station_acks: BTreeMap::new(),
            max_entries,
            bytes: 0,
            max_bytes,
        }
    }

    /// Start over (a show was loaded): a new identity, nothing to catch up
    /// from, so every station takes a full snapshot.
    pub fn restart(&mut self) {
        *self = Self {
            epoch: self.epoch,
            ..Self::with_limits(self.max_entries, self.max_bytes)
        };
    }

    /// Commit a pending operation from `station`: assign a sequence number
    /// and append to the journal.
    pub fn commit(
        &mut self,
        pending: PendingReplicationOp,
        station: &str,
        timestamp_ms: u64,
    ) -> ReplicationOp {
        let seq = self.next_seq;
        self.next_seq += 1;

        let op = ReplicationOp {
            journal: self.id,
            epoch: self.epoch,
            seq,
            station: station.to_string(),
            user: pending.user,
            command: pending.command,
            affected_subtrees: pending.affected_subtrees,
            post_snapshots: pending.post_snapshots,
            undo: pending.undo,
            runtime: pending.runtime,
            authority_timestamp_ms: timestamp_ms,
        };

        self.bytes += op.size_bytes();
        self.log.push_back(op.clone());
        self.trim();

        op
    }

    /// Trim oldest entries while the journal is over either limit; the newest op stays even
    /// when it alone is over the byte limit.
    fn trim(&mut self) {
        while self.log.len() > self.max_entries || (self.bytes > self.max_bytes && self.log.len() > 1) {
            match self.log.pop_front() {
                Some(old) => self.bytes = self.bytes.saturating_sub(old.size_bytes()),
                None => break,
            }
        }
    }

    /// Record a station's acknowledgement of a sequence number.
    pub fn ack(&mut self, station: &str, acked_seq: u64) {
        let entry = self.station_acks.entry(station.to_string()).or_insert(0);
        if acked_seq > *entry {
            *entry = acked_seq;
        }
    }

    /// Get operations from a given sequence number (inclusive) onward.
    ///
    /// Returns `None` if the requested range has been trimmed from the journal.
    pub fn ops_from(&self, from_seq: u64) -> Option<Vec<&ReplicationOp>> {
        if from_seq > self.next_seq {
            return None;
        }
        if self.log.is_empty() {
            return (from_seq == self.next_seq).then(Vec::new);
        }

        let oldest_seq = self.log.front().map(|op| op.seq).unwrap_or(0);
        if from_seq < oldest_seq {
            // Requested range has been trimmed — caller needs full snapshot
            return None;
        }

        let ops: Vec<&ReplicationOp> = self.log.iter().filter(|op| op.seq >= from_seq).collect();
        Some(ops)
    }

    /// A new master's epoch (6c: every election or handover raises it).
    pub fn set_epoch(&mut self, epoch: u64) {
        self.epoch = epoch;
    }

    /// This journal's identity.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// The master's epoch.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// The next sequence number that will be assigned.
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// The most recently committed sequence number, or 0 if empty.
    pub fn tip_seq(&self) -> u64 {
        self.next_seq.saturating_sub(1)
    }

    /// Number of operations currently in the journal.
    pub fn len(&self) -> usize {
        self.log.len()
    }

    /// Whether the journal is empty.
    pub fn is_empty(&self) -> bool {
        self.log.is_empty()
    }

    /// Bytes the journal holds now.
    pub fn size_bytes(&self) -> usize {
        self.bytes
    }

    /// Get the per-station ack map (read-only).
    pub fn station_acks(&self) -> &BTreeMap<String, u64> {
        &self.station_acks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pending(user: &str, cmd: &str) -> PendingReplicationOp {
        PendingReplicationOp {
            user: user.to_string(),
            command: cmd.to_string(),
            affected_subtrees: vec![ShowSubtree::Devices],
            post_snapshots: vec![SubtreeSnapshot {
                subtree: ShowSubtree::Devices,
                data: b"snapshot_data".to_vec(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn journal_commit_assigns_sequential_ids() {
        let mut journal = ReplicationJournal::new(100);
        let op1 = journal.commit(make_pending("Admin", "Label Fixture 1 Spot"), "FOH", 1000);
        let op2 = journal.commit(make_pending("Admin", "Label Fixture 2 Wash"), "FOH", 1001);
        assert_eq!(op1.seq, 1);
        assert_eq!(op2.seq, 2);
        assert_eq!(journal.next_seq(), 3);
        assert_eq!(journal.tip_seq(), 2);
        assert_eq!(journal.len(), 2);
    }

    #[test]
    fn journal_trims_oldest_when_full() {
        let mut journal = ReplicationJournal::new(3);
        for i in 0..5 {
            journal.commit(make_pending("Admin", &format!("cmd{i}")), "FOH", 1000 + i);
        }
        assert_eq!(journal.len(), 3);
        // Oldest surviving should be seq 3 (cmds: cmd2, cmd3, cmd4)
        let oldest = journal.log.front().unwrap();
        assert_eq!(oldest.seq, 3);
        assert_eq!(oldest.command, "cmd2");
    }

    #[test]
    fn journal_ops_from_returns_range() {
        let mut journal = ReplicationJournal::new(100);
        for i in 0..5 {
            journal.commit(make_pending("Admin", &format!("cmd{i}")), "FOH", 1000 + i);
        }
        let ops = journal.ops_from(3).unwrap();
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0].seq, 3);
        assert_eq!(ops[1].seq, 4);
        assert_eq!(ops[2].seq, 5);
    }

    #[test]
    fn journal_ops_from_returns_none_when_trimmed() {
        let mut journal = ReplicationJournal::new(3);
        for i in 0..5 {
            journal.commit(make_pending("Admin", &format!("cmd{i}")), "FOH", 1000 + i);
        }
        // Seq 1 was trimmed
        assert!(journal.ops_from(1).is_none());
        // Seq 3 (oldest surviving) should work
        assert!(journal.ops_from(3).is_some());
    }

    #[test]
    fn journal_ack_tracks_per_station() {
        let mut journal = ReplicationJournal::new(100);
        journal.ack("StationA", 5);
        journal.ack("StationB", 3);
        journal.ack("StationA", 7);
        // Should not go backwards
        journal.ack("StationB", 1);

        assert_eq!(journal.station_acks()["StationA"], 7);
        assert_eq!(journal.station_acks()["StationB"], 3);
    }

    #[test]
    fn journal_empty_ops_from_returns_empty_vec() {
        let journal = ReplicationJournal::new(100);
        let ops = journal.ops_from(1).unwrap();
        assert!(ops.is_empty());
        // Asking past the tip can't be served from this journal.
        assert!(journal.ops_from(5).is_none());
    }

    #[test]
    fn ops_carry_journal_epoch_and_station_and_restart_changes_identity() {
        let mut journal = ReplicationJournal::new(10);
        let first = journal.id();
        let op = journal.commit(make_pending("Admin", "x"), "FOH-2", 5);
        assert_eq!((op.journal, op.epoch, op.seq), (first, 1, 1));
        assert_eq!(op.station, "FOH-2");
        journal.restart();
        assert_ne!(journal.id(), first);
        assert_eq!(journal.tip_seq(), 0);
        assert!(journal.ops_from(1).unwrap().is_empty());
    }

    #[test]
    fn journal_default_has_1000_capacity() {
        let journal = ReplicationJournal::default();
        assert_eq!(journal.max_entries, 1000);
        assert_eq!(journal.max_bytes, DEFAULT_MAX_BYTES);
        assert!(journal.is_empty());
        assert_eq!(journal.tip_seq(), 0);
    }

    fn pending_of_size(bytes: usize) -> PendingReplicationOp {
        PendingReplicationOp {
            user: "A".to_string(),
            command: "x".to_string(),
            affected_subtrees: vec![ShowSubtree::Playback],
            post_snapshots: vec![SubtreeSnapshot { subtree: ShowSubtree::Playback, data: vec![b'0'; bytes] }],
            ..Default::default()
        }
    }

    #[test]
    fn journal_trims_by_bytes_before_count() {
        // Each op is 1000 snapshot bytes + "x" + "A" + "S"; 3500 bytes hold three of them.
        let mut journal = ReplicationJournal::with_limits(100, 3500);
        for _ in 0..5 {
            journal.commit(pending_of_size(1000), "S", 0);
        }
        assert_eq!(journal.len(), 3);
        assert_eq!(journal.size_bytes(), 3 * 1003);
        assert!(journal.ops_from(2).is_none(), "trimmed by size: a full snapshot instead");
        assert_eq!(journal.ops_from(3).unwrap().len(), 3);
    }

    #[test]
    fn an_op_over_the_byte_limit_alone_still_stays() {
        let mut journal = ReplicationJournal::with_limits(100, 500);
        journal.commit(pending_of_size(100), "S", 0);
        journal.commit(pending_of_size(10_000), "S", 0);
        assert_eq!(journal.len(), 1);
        assert_eq!(journal.ops_from(2).unwrap().len(), 1);
        journal.restart();
        assert_eq!(journal.size_bytes(), 0);
        assert_eq!(journal.max_bytes, 500, "a restart keeps the limits");
    }
}
