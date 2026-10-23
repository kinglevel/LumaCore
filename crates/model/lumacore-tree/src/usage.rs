//! What a user uses (decisions §30 "12b2"): `ShowData.Users.{n}.Usage`, per
//! object GUID or command template, a count and the last use. It ranks the
//! command line's suggestions (after the grammar and the fuzzy score) and
//! goes with the show file so a station learns what its users do. Saved and
//! replicated, never in an undo frame (its own `ShowSubtree::Usage`, like
//! `Playbacks`): Oops never forgets what you used.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Half-life of a use: a use this long ago counts half.
pub const USAGE_HALF_LIFE_MS: u64 = 7 * 24 * 60 * 60 * 1000;

/// Entries kept per user; the least used go first.
pub const USAGE_MAX_ENTRIES: usize = 500;

/// One user's usage, by key: an object's GUID (`"6f1c…"`) or a command
/// template (`"word:Store"`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub entries: BTreeMap<String, UsageEntry>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageEntry {
    pub count: u32,
    /// Unix ms of the last use.
    pub last_ms: u64,
}

impl UsageEntry {
    /// The decayed rank at `now_ms`: the count, halved per half-life since
    /// the last use.
    pub fn rank(&self, now_ms: u64) -> f64 {
        let age = now_ms.saturating_sub(self.last_ms) as f64;
        f64::from(self.count) * 0.5f64.powf(age / USAGE_HALF_LIFE_MS as f64)
    }
}

impl Usage {
    /// Count one use of `key` at `now_ms`.
    pub fn record(&mut self, key: &str, now_ms: u64) {
        let entry = self.entries.entry(key.to_string()).or_default();
        entry.count = entry.count.saturating_add(1);
        entry.last_ms = entry.last_ms.max(now_ms);
        if self.entries.len() > USAGE_MAX_ENTRIES {
            self.prune(now_ms);
        }
    }

    /// The decayed rank of `key` (0 when never used).
    pub fn rank(&self, key: &str, now_ms: u64) -> f64 {
        self.entries.get(key).map_or(0.0, |e| e.rank(now_ms))
    }

    pub fn get(&self, key: &str) -> Option<&UsageEntry> {
        self.entries.get(key)
    }

    /// Drop the lowest-ranked entries down to [`USAGE_MAX_ENTRIES`]
    /// (deterministic: rank, then the older use, then the key).
    fn prune(&mut self, now_ms: u64) {
        let mut ranked: Vec<(f64, u64, String)> = self
            .entries
            .iter()
            .map(|(k, e)| (e.rank(now_ms), e.last_ms, k.clone()))
            .collect();
        ranked.sort_by(|a, b| {
            a.0.total_cmp(&b.0)
                .then(a.1.cmp(&b.1))
                .then(a.2.cmp(&b.2))
        });
        let excess = self.entries.len().saturating_sub(USAGE_MAX_ENTRIES);
        for (_, _, key) in ranked.into_iter().take(excess) {
            self.entries.remove(&key);
        }
    }
}

/// The usage key of a command or object word (a command template).
pub fn word_key(label: &str) -> String {
    format!("word:{label}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_decays_by_half_life() {
        let mut u = Usage::default();
        u.record("a", 0);
        u.record("a", 0);
        assert_eq!(u.rank("a", 0), 2.0);
        assert!((u.rank("a", USAGE_HALF_LIFE_MS) - 1.0).abs() < 1e-9);
        assert_eq!(u.rank("missing", 0), 0.0);
    }

    #[test]
    fn prune_keeps_the_most_used() {
        let mut u = Usage::default();
        u.record("keep", 10);
        u.record("keep", 10);
        for i in 0..USAGE_MAX_ENTRIES {
            u.record(&format!("k{i:04}"), 5);
        }
        assert_eq!(u.entries.len(), USAGE_MAX_ENTRIES);
        assert!(u.get("keep").is_some());
    }
}
