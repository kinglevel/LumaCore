//! `Root.Runtime.Journal` — the change log (`docs/meta/decisions.md` §26).
//!
//! Every command that changes (or tries to change) the show leaves an entry:
//! who, from where, what, which subtrees changed and how it ended. The last
//! [`JOURNAL_CAPACITY`] entries are kept; nothing is saved with the show (the
//! sysmon log on disk keeps the full history).

use std::collections::VecDeque;

use serde::Serialize;

/// Entries kept in memory.
pub const JOURNAL_CAPACITY: usize = 1000;

/// How a journaled command ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum JournalResult {
    Ok,
    /// It reported an error with this code (`E-PATCH-002`), or none.
    Error(Option<String>),
    /// It panicked; the log has the backtrace under this report number.
    Internal {
        report: u64,
    },
}

impl std::fmt::Display for JournalResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => f.write_str("ok"),
            Self::Error(Some(code)) => write!(f, "error {code}"),
            Self::Error(None) => f.write_str("error"),
            Self::Internal { report } => write!(f, "internal error, report #{report}"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct JournalEntry {
    /// Unix time in ms.
    pub time_ms: u64,
    /// Session epoch (0 until master election exists, §10).
    pub epoch: u64,
    /// Position in this journal (never reused while the server runs).
    pub seq: u64,
    pub user: String,
    pub station: String,
    pub command: String,
    /// The show subtrees the command changed (`Patch`, `Datapool(1)`, …).
    pub changed: Vec<String>,
    pub result: JournalResult,
}

#[derive(Debug, Default)]
pub struct Journal {
    entries: VecDeque<JournalEntry>,
    next_seq: u64,
}

impl Journal {
    /// Append an entry (its `seq` is assigned here); the oldest drops off past
    /// the capacity. Returns the assigned seq.
    pub fn record(&mut self, mut entry: JournalEntry) -> u64 {
        self.next_seq += 1;
        entry.seq = self.next_seq;
        if self.entries.len() == JOURNAL_CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
        self.next_seq
    }

    /// Oldest first.
    pub fn entries(&self) -> impl DoubleEndedIterator<Item = &JournalEntry> + ExactSizeIterator {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries after `seq` (for `Watch Events`).
    pub fn since(&self, seq: u64) -> impl Iterator<Item = &JournalEntry> {
        self.entries.iter().filter(move |e| e.seq > seq)
    }

    pub fn last_seq(&self) -> u64 {
        self.next_seq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(command: &str) -> JournalEntry {
        JournalEntry {
            time_ms: 0,
            epoch: 0,
            seq: 0,
            user: "Admin".into(),
            station: "S".into(),
            command: command.into(),
            changed: vec![],
            result: JournalResult::Ok,
        }
    }

    #[test]
    fn keeps_the_last_entries_with_increasing_seq() {
        let mut journal = Journal::default();
        for i in 0..JOURNAL_CAPACITY + 5 {
            journal.record(entry(&format!("cmd {i}")));
        }
        assert_eq!(journal.len(), JOURNAL_CAPACITY);
        let first = journal.entries().next().unwrap();
        assert_eq!(first.seq, 6);
        assert_eq!(first.command, "cmd 5");
        assert_eq!(journal.last_seq(), (JOURNAL_CAPACITY + 5) as u64);
        assert_eq!(journal.since(journal.last_seq() - 2).count(), 2);
    }
}
