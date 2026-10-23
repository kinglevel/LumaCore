//! IdPool: per-type sequential ID management with reuse after deletion.
//!
//! Industry convention: IDs start at 1, not 0.
//! Deleted IDs are returned to the pool and reused (lowest available first).

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Manages per-type sequential u32 IDs.
///
/// IDs start at 1 (industry convention). Deleted IDs are tracked in a BTreeSet
/// for O(log n) reuse, ensuring the lowest available ID is always returned.
///
/// # Example
/// ```
/// use lumacore_base::IdPool;
///
/// let mut pool = IdPool::new();
/// assert_eq!(pool.allocate(), Some(1));
/// assert_eq!(pool.allocate(), Some(2));
/// assert_eq!(pool.allocate(), Some(3));
/// pool.release(2);
/// assert_eq!(pool.allocate(), Some(2)); // reused
/// assert_eq!(pool.allocate(), Some(4));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdPool {
    /// One past the highest number ever used; past `u32::MAX` the pool is
    /// full (a number moved to the top leaves no new one above it).
    next_new: u64,
    freed: BTreeSet<u32>,
}

impl Default for IdPool {
    /// Returns an IdPool ready to allocate from 1 (industry convention).
    fn default() -> Self {
        Self::new()
    }
}

impl IdPool {
    /// Create a new empty IdPool. First call to `next()` returns 1.
    pub fn new() -> Self {
        Self {
            next_new: 1,
            freed: BTreeSet::new(),
        }
    }

    /// Allocate the next available ID: the lowest freed one, else one above
    /// the highest ever used. None when no number is left above it (R1-12).
    pub fn allocate(&mut self) -> Option<u32> {
        if let Some(reused) = self.freed.pop_first() {
            return Some(reused);
        }
        let id = u32::try_from(self.next_new).ok()?;
        self.next_new += 1;
        Some(id)
    }

    /// Release an ID back to the pool for reuse.
    ///
    /// The ID will be returned by a future call to `next()`.
    /// Releasing an ID that was never allocated is a no-op (no panic).
    pub fn release(&mut self, id: u32) {
        if id > 0 && u64::from(id) < self.next_new {
            self.freed.insert(id);
        }
    }

    /// Reserve a specific ID, removing it from the available pool.
    ///
    /// Used when loading from a file — the file contains explicit IDs that
    /// must be honoured. Advances `next_new` if necessary.
    pub fn reserve(&mut self, id: u32) {
        // u64: reserving u32::MAX leaves the pool full, never wrapped (R1-12).
        self.next_new = self.next_new.max(u64::from(id) + 1);
        self.freed.remove(&id);
    }

    /// Returns the total count of IDs currently in use (allocated - freed).
    pub fn active_count(&self) -> u32 {
        if self.next_new == 1 {
            return 0;
        }
        u32::try_from(self.next_new - 1 - self.freed.len() as u64).unwrap_or(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_one() {
        let mut pool = IdPool::new();
        assert_eq!(pool.allocate(), Some(1));
    }

    #[test]
    fn sequential_allocation() {
        let mut pool = IdPool::new();
        assert_eq!(pool.allocate(), Some(1));
        assert_eq!(pool.allocate(), Some(2));
        assert_eq!(pool.allocate(), Some(3));
    }

    #[test]
    fn reuses_freed_ids_lowest_first() {
        let mut pool = IdPool::new();
        pool.allocate(); // 1
        pool.allocate(); // 2
        pool.allocate(); // 3
        pool.release(2);
        pool.release(1);
        assert_eq!(pool.allocate(), Some(1)); // lowest freed first
        assert_eq!(pool.allocate(), Some(2));
        assert_eq!(pool.allocate(), Some(4)); // back to sequential
    }

    #[test]
    fn release_out_of_range_is_noop() {
        let mut pool = IdPool::new();
        pool.release(99); // never allocated — noop
        assert_eq!(pool.allocate(), Some(1)); // still starts at 1
    }

    #[test]
    fn reserve_specific_id() {
        let mut pool = IdPool::new();
        pool.reserve(5);
        // next_new should be 6 now
        let ids: Vec<u32> = (0..5).filter_map(|_| pool.allocate()).collect();
        assert!(!ids.contains(&5)); // 5 is reserved, should not be returned
    }

    #[test]
    fn active_count_tracks_usage() {
        let mut pool = IdPool::new();
        assert_eq!(pool.active_count(), 0);
        pool.allocate(); // 1
        pool.allocate(); // 2
        assert_eq!(pool.active_count(), 2);
        pool.release(1);
        assert_eq!(pool.active_count(), 1);
    }
}

#[cfg(test)]
mod full_tests {
    use super::*;

    /// R1 review 8: a number at the top leaves the pool full, never wrapped.
    #[test]
    fn a_full_pool_gives_none() {
        let mut pool = IdPool::new();
        pool.reserve(u32::MAX - 1);
        assert_eq!(pool.allocate(), Some(u32::MAX));
        assert_eq!(pool.allocate(), None, "nothing above u32::MAX");
        pool.reserve(u32::MAX);
        assert_eq!(pool.allocate(), None);
        pool.release(7);
        assert_eq!(pool.allocate(), Some(7), "a freed number still comes back");
    }
}
