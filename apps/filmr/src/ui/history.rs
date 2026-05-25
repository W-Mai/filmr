//! Generic undo/redo history with idle-based commit.
//!
//! Strategy:
//! - Record() observes the current value every frame.
//! - When the value differs from the last committed entry, it becomes
//!   "pending". If the same pending value is seen for DEBOUNCE_MS
//!   continuously (user stopped sliding), it gets committed.
//! - This means rapid slider drags collapse into a single history entry,
//!   committed when the user releases.

use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::time::Instant;

const DEBOUNCE_MS: u128 = 350;
const MAX_HISTORY: usize = 100;

fn hash_value<T: Serialize>(t: &T) -> u64 {
    let bytes = serde_json::to_vec(t).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    hasher.write(&bytes);
    hasher.finish()
}

pub struct History<T: Clone + Serialize> {
    past: Vec<T>,
    past_hashes: Vec<u64>,
    future: Vec<T>,
    pending_hash: Option<u64>,
    pending_since: Option<Instant>,
}

impl<T: Clone + Serialize> Default for History<T> {
    fn default() -> Self {
        Self {
            past: Vec::new(),
            past_hashes: Vec::new(),
            future: Vec::new(),
            pending_hash: None,
            pending_since: None,
        }
    }
}

impl<T: Clone + Serialize> History<T> {
    /// Observe current state. Cheap to call every frame.
    /// Commits a snapshot only when the value has been stable (same as last
    /// observed) for DEBOUNCE_MS, indicating the user paused editing.
    pub fn record(&mut self, current: &T) {
        let h = hash_value(current);

        // Initialize baseline on first call
        if self.past.is_empty() {
            self.past.push(current.clone());
            self.past_hashes.push(h);
            return;
        }

        let committed_hash = *self.past_hashes.last().unwrap();

        // Value matches last committed → no pending change
        if h == committed_hash {
            self.pending_hash = None;
            self.pending_since = None;
            return;
        }

        let now = Instant::now();

        if self.pending_hash == Some(h) {
            // Same pending value seen — check if held long enough to commit
            if let Some(since) = self.pending_since {
                if now.duration_since(since).as_millis() >= DEBOUNCE_MS {
                    // Commit
                    self.past.push(current.clone());
                    self.past_hashes.push(h);
                    if self.past.len() > MAX_HISTORY {
                        self.past.remove(0);
                        self.past_hashes.remove(0);
                    }
                    self.future.clear();
                    self.pending_hash = None;
                    self.pending_since = None;
                }
            }
        } else {
            // New pending value
            self.pending_hash = Some(h);
            self.pending_since = Some(now);
        }
    }

    pub fn can_undo(&self) -> bool {
        self.past.len() >= 2
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn has_pending(&self) -> bool {
        self.pending_hash.is_some()
    }

    /// Undo: revert to previous snapshot. Returns true if changed.
    pub fn undo(&mut self, current: &mut T) -> bool {
        if self.past.len() < 2 {
            return false;
        }
        let popped = self.past.pop().unwrap();
        self.past_hashes.pop();
        self.future.push(popped);
        if let Some(prev) = self.past.last() {
            *current = prev.clone();
            self.pending_hash = None;
            self.pending_since = None;
            return true;
        }
        false
    }

    /// Redo: re-apply a previously undone snapshot. Returns true if changed.
    pub fn redo(&mut self, current: &mut T) -> bool {
        if let Some(snap) = self.future.pop() {
            let h = hash_value(&snap);
            self.past.push(snap.clone());
            self.past_hashes.push(h);
            *current = snap;
            self.pending_hash = None;
            self.pending_since = None;
            return true;
        }
        false
    }

    pub fn clear(&mut self) {
        self.past.clear();
        self.past_hashes.clear();
        self.future.clear();
        self.pending_hash = None;
        self.pending_since = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_change_no_record() {
        let mut h: History<i32> = History::default();
        let v = 1;
        for _ in 0..10 {
            h.record(&v);
        }
        assert_eq!(h.past.len(), 1);
        assert!(!h.can_undo());
    }

    #[test]
    fn idle_commits_pending() {
        let mut h: History<i32> = History::default();
        let mut v = 1;
        h.record(&v);
        // Change value
        v = 2;
        h.record(&v);
        // Same value, but not yet idle long enough
        h.record(&v);
        assert_eq!(h.past.len(), 1, "shouldn't commit yet");
        // Wait past debounce
        std::thread::sleep(std::time::Duration::from_millis(400));
        h.record(&v);
        assert_eq!(h.past.len(), 2, "should commit after idle");
        assert!(h.can_undo());
    }

    #[test]
    fn intermediate_values_collapse() {
        let mut h: History<i32> = History::default();
        let mut v = 1;
        h.record(&v);
        // Rapidly change v
        for x in 2..10 {
            v = x;
            h.record(&v);
        }
        // v=9 not stable yet → no commit
        assert_eq!(h.past.len(), 1);
        // Wait past debounce, hold v=9
        std::thread::sleep(std::time::Duration::from_millis(400));
        h.record(&v);
        assert_eq!(h.past.len(), 2);
        // History should be just [1, 9] — intermediate values collapsed
    }

    #[test]
    fn undo_redo() {
        let mut h: History<i32> = History::default();
        let mut v = 1;
        h.record(&v);
        v = 2;
        h.record(&v);
        std::thread::sleep(std::time::Duration::from_millis(400));
        h.record(&v); // commits 2
        v = 3;
        h.record(&v);
        std::thread::sleep(std::time::Duration::from_millis(400));
        h.record(&v); // commits 3

        assert_eq!(h.past.len(), 3);
        h.undo(&mut v);
        assert_eq!(v, 2);
        h.undo(&mut v);
        assert_eq!(v, 1);
        h.redo(&mut v);
        assert_eq!(v, 2);
    }
}
