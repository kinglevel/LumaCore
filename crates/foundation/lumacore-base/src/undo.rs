//! Undo/redo system for LumaCore.
//!
//! Subtree-granular undo: each frame records only the affected subtrees of
//! `ShowData`, not the entire tree.  This enables per-user isolation and
//! multi-user conflict detection.
//!
//! Stacks live in `Runtime.Users.{User}.Undo`: replicated, never saved
//! (decisions §21 "Step 6 hardening").

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Default memory budget: 10 MB (subtree deltas are much smaller than full snapshots).
pub const DEFAULT_MAX_BYTES: usize = 10 * 1024 * 1024;

/// Default maximum number of undo frames (a common console limit, ~100).
pub const DEFAULT_MAX_FRAMES: usize = 100;

/// One subtree of `ShowData`, snapshotted on its own by the change path (undo
/// frames, replication ops, journal). One per ShowData field; pools are one
/// per entry **by GUID** (decisions §21 "6d": distribution is strict about
/// identity — an op always means the same object, wherever its slot is; a
/// move is a change of that one subtree), where a missing entry means
/// "absent" (so creating and deleting one is undone and replicated too). The list of a show's subtrees is an
/// exhaustive destructure of `ShowData` (`lumacore_objects::change::subtrees`): a new
/// field doesn't compile until it is listed here (decisions §21).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShowSubtree {
    /// `devices`: the patch (fixtures, types, universes), FID namespaces.
    Devices,
    /// One `datapools` entry (groups, presets, …), by GUID.
    DataPool(uuid::Uuid),
    DatapoolPriorityOrders,
    ShowSettings,
    Spatial3D,
    /// `playback`: sequences, cues, executors.
    Playback,
    /// Running playback state (`ShowData.Playbacks`): saved and replicated,
    /// never undone — Oops never undoes a Go (decisions §13).
    Playbacks,
    /// `ShowData.LumaSession`: patch version and known stations (decisions
    /// §21 "6c"). Replicated, never undone.
    Session,
    /// One `user_profiles` entry: views, screens, keys, settings (by GUID).
    Profile(uuid::Uuid),
    /// One `users` entry (login, role, profile link) without its
    /// programmer, by the user's GUID.
    User(uuid::Uuid),
    /// The programmer values of the user with this GUID (§10).
    Programmer(uuid::Uuid),
    /// The selection of the user with this GUID: the fixture selection (with
    /// its grid) and the one-only DataPool / Sequence / Timecode selections.
    /// Apart from `Programmer` so a selection step and a programmer step
    /// restore independently (decisions §29 "11c hardening").
    Selection(uuid::Uuid),
    /// What the user with this GUID uses (decisions §30 "12b2"): saved and
    /// replicated, never undone — like `Playbacks`.
    Usage(uuid::Uuid),
    Protocols,
    EffectEngines,
    Variables,
    /// `images`: the show's picture pool (decisions §30 "12a2").
    Images,
    /// `assets`: asset roots and large files (§8).
    Assets,
    AttributeDefs,
    FeatureGroups,
    ActivationGroups,
    /// `attribute_rules`: the resolver's rules (decisions §30 "12f-a").
    AttributeRules,
    /// `stations`: the station table — each station's rig wiring (decisions
    /// §30 "12d part 4"). Undone like any show data.
    Stations,
    /// `tags`: the tag tree (decisions §12 "Tags hardened"). Undone like any show data.
    Tags,
}

impl ShowSubtree {
    /// Whether an undo frame records this subtree.
    pub fn is_undoable(&self) -> bool {
        !matches!(
            self,
            ShowSubtree::Playbacks | ShowSubtree::Session | ShowSubtree::Usage(_)
        )
    }
}

impl std::fmt::Display for ShowSubtree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShowSubtree::DataPool(g) => write!(f, "DataPool({})", short(g)),
            ShowSubtree::Profile(g) => write!(f, "Profile({})", short(g)),
            ShowSubtree::User(g) => write!(f, "User({})", short(g)),
            ShowSubtree::Programmer(g) => write!(f, "Programmer(user {})", short(g)),
            ShowSubtree::Selection(g) => write!(f, "Selection(user {})", short(g)),
            ShowSubtree::Usage(g) => write!(f, "Usage(user {})", short(g)),
            other => write!(f, "{other:?}"),
        }
    }
}

/// The first 8 hex digits of a GUID (logs and journal lines).
fn short(g: &uuid::Uuid) -> String {
    g.simple().to_string()[..8].to_string()
}

/// The category of an Oops step (decisions §29 "11c hardening"): a label
/// only — its colour and the `List Undo /<category>` filter. Oops always
/// undoes the newest step, whatever its category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OopsCategory {
    /// Values: At, Full, Zero, Park, Clear, Reset, Meow.
    Programmer,
    /// The fixture selection and the one-only selections.
    Selection,
    /// Pool objects: Store, Delete, Copy, Move, Label, Edit.
    Show,
    /// Fixtures, types, universes, networks, lines, routes, Keep Patch.
    Patch,
    /// Windows, views, screens, watches.
    UI,
    /// Users and their login, role and profile.
    Users,
    /// Show settings and profile settings.
    Settings,
    /// Key bindings and UI keys.
    Keys,
    /// Station-local settings (interfaces, network → NIC, Online, name,
    /// priority): restored on the station where they changed.
    Station,
}

impl OopsCategory {
    pub const ALL: [OopsCategory; 9] = [
        OopsCategory::Programmer,
        OopsCategory::Selection,
        OopsCategory::Show,
        OopsCategory::Patch,
        OopsCategory::UI,
        OopsCategory::Users,
        OopsCategory::Settings,
        OopsCategory::Keys,
        OopsCategory::Station,
    ];

    pub fn label(self) -> &'static str {
        match self {
            OopsCategory::Programmer => "Programmer",
            OopsCategory::Selection => "Selection",
            OopsCategory::Show => "Show",
            OopsCategory::Patch => "Patch",
            OopsCategory::UI => "UI",
            OopsCategory::Users => "Users",
            OopsCategory::Settings => "Settings",
            OopsCategory::Keys => "Keys",
            OopsCategory::Station => "Station",
        }
    }

    /// A category by its label (any case).
    pub fn parse(name: &str) -> Option<OopsCategory> {
        Self::ALL
            .into_iter()
            .find(|c| c.label().eq_ignore_ascii_case(name))
    }
}

/// How a command files its Oops step: the category, and whether it is a
/// **Main** action (Oops Confirmation "Main": create, delete, move,
/// insert, file actions, layout).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OopsKind {
    pub category: OopsCategory,
    pub main: bool,
}

/// State outside `ShowData` that an Oops step also puts back, only on the
/// station it belongs to (decisions §29 "11c hardening").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalSubtree {
    /// The station-local settings (interfaces, network → NIC, Online, name,
    /// priority, …).
    Station,
    /// One user's watches on that station.
    Watches(String),
}

/// A [`LocalSubtree`] before and after the step, on the station `station_id`.
/// Oops puts `before` back, Redo `after`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSnapshot {
    pub what: LocalSubtree,
    pub station_id: String,
    #[serde(with = "crate::embedded_json")]
    pub before: Vec<u8>,
    #[serde(with = "crate::embedded_json")]
    pub after: Vec<u8>,
}

impl LocalSnapshot {
    fn size(&self) -> usize {
        self.before.len() + self.after.len()
    }
}

/// A single subtree snapshot within an undo frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtreeSnapshot {
    /// Which subtree this snapshot captures.
    pub subtree: ShowSubtree,
    /// JSON bytes of just this subtree (carried as the JSON they are, `embedded_json`).
    #[serde(with = "crate::embedded_json")]
    pub data: Vec<u8>,
}

/// A single undo frame — records which subtrees changed.
#[derive(Debug, Clone, Serialize)]
pub struct UndoFrame {
    /// Human-readable description of the action (e.g., "Fixture 1 At 50").
    pub description: String,
    /// The category, for filtering.
    pub category: OopsCategory,
    /// The user who performed the action.
    pub user: String,
    /// Monotonic sequence ID for conflict detection.
    pub sequence_id: u64,
    /// Pre-mutation snapshots of affected subtrees.
    pub snapshots: Vec<SubtreeSnapshot>,
    /// The station the command came from and when (unix ms) — the Oops
    /// session time / in-my-session (§21 "6c").
    #[serde(default)]
    pub station: String,
    #[serde(default)]
    pub time_ms: u64,
    /// A Main action (Oops Confirmation "Main" asks before undoing it).
    #[serde(default)]
    pub main: bool,
    /// Station-local state the step changed (station settings, watches).
    #[serde(default)]
    pub local: Vec<LocalSnapshot>,
    /// Total byte size of all snapshot data — recomputed on deserialise.
    #[serde(skip)]
    pub size_bytes: usize,
}

/// Custom `Deserialize` that recomputes `size_bytes` from snapshot data lengths.
impl<'de> Deserialize<'de> for UndoFrame {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            description: String,
            category: OopsCategory,
            user: String,
            sequence_id: u64,
            snapshots: Vec<SubtreeSnapshot>,
            #[serde(default)]
            station: String,
            #[serde(default)]
            time_ms: u64,
            #[serde(default)]
            main: bool,
            #[serde(default)]
            local: Vec<LocalSnapshot>,
        }
        let raw = Raw::deserialize(de)?;
        let size: usize = raw.snapshots.iter().map(|s| s.data.len()).sum::<usize>()
            + raw.local.iter().map(LocalSnapshot::size).sum::<usize>();
        Ok(UndoFrame {
            description: raw.description,
            category: raw.category,
            user: raw.user,
            sequence_id: raw.sequence_id,
            snapshots: raw.snapshots,
            station: raw.station,
            time_ms: raw.time_ms,
            main: raw.main,
            local: raw.local,
            size_bytes: size,
        })
    }
}

impl UndoFrame {
    /// The same step holding other snapshots (the redo side of an Oops, the
    /// undo side of a Redo): category, Main, station and local state stay.
    pub fn with_snapshots(&self, snapshots: Vec<SubtreeSnapshot>, description: &str) -> UndoFrame {
        let size = snapshots.iter().map(|s| s.data.len()).sum::<usize>()
            + self.local.iter().map(LocalSnapshot::size).sum::<usize>();
        UndoFrame {
            description: if description.is_empty() {
                self.description.clone()
            } else {
                description.to_string()
            },
            category: self.category,
            user: self.user.clone(),
            sequence_id: self.sequence_id,
            snapshots,
            station: self.station.clone(),
            time_ms: self.time_ms,
            main: self.main,
            local: self.local.clone(),
            size_bytes: size,
        }
    }
}

/// What a command did to one user's undo stack, as the master records it and
/// members replay it ([`UndoStack::apply_step`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoStep {
    /// A new frame was filed.
    Push(UndoFrame),
    /// Oops: the newest frame went, `redo` was pushed onto the redo stack.
    Undo { redo: UndoFrame },
    /// Redo: the newest redo frame went, `undo` was pushed back as undo.
    Redo { undo: UndoFrame },
}

/// An [`UndoStep`] on a named user's stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUndoStep {
    pub user: String,
    pub step: UndoStep,
}

/// A memory-aware stack of undo/redo frames (one per user, in
/// `Runtime.Users.{User}.Undo`).
///
/// ### Semantics
/// - `push` saves pre-mutation subtree snapshots and clears the redo stack.
/// - `pop_undo` returns the previous subtree snapshots and saves current ones for redo.
/// - `pop_redo` replays the redone subtree snapshots and saves current ones for undo.
/// - The undo deque is newest-at-back; `pop` takes from the back.
/// - Trimmed by both byte budget AND frame count (~100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoStack {
    /// Undo frames — oldest at front, newest at back.
    pub frames: VecDeque<UndoFrame>,
    /// Redo frames — oldest at front, newest at back.
    pub redo_frames: VecDeque<UndoFrame>,
    /// Total bytes consumed by undo frames (redo frames not counted).
    pub total_bytes: usize,
    /// Maximum bytes before the oldest undo frames are trimmed.
    pub max_bytes: usize,
    /// Maximum number of undo frames.
    pub max_frames: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_BYTES)
    }
}

impl UndoStack {
    /// Create a new `UndoStack` with the given memory budget.
    pub fn new(max_bytes: usize) -> Self {
        Self {
            frames: VecDeque::new(),
            redo_frames: VecDeque::new(),
            total_bytes: 0,
            max_bytes,
            max_frames: DEFAULT_MAX_FRAMES,
        }
    }

    /// Push a pre-mutation undo frame onto the stack.
    ///
    /// - Clears the redo stack (a new action invalidates redo history).
    /// - Trims oldest frames when total memory or frame count exceed budget.
    pub fn push_frame(&mut self, frame: UndoFrame) {
        let size = frame.size_bytes;

        // New action invalidates redo history.
        self.redo_frames.clear();

        self.frames.push_back(frame);
        self.total_bytes += size;

        // Trim oldest frames until under budget (both byte and count).
        while self.total_bytes > self.max_bytes || self.frames.len() > self.max_frames {
            if let Some(old) = self.frames.pop_front() {
                self.total_bytes = self.total_bytes.saturating_sub(old.size_bytes);
            } else {
                break;
            }
        }
    }

    /// Undo the last action.
    ///
    /// Pops the newest undo frame, pushes `current_snapshots` onto the redo
    /// stack (so the user can redo), and returns the frame to restore.
    ///
    /// Returns `None` if there is nothing to undo.
    pub fn pop_undo(
        &mut self,
        current_snapshots: Vec<SubtreeSnapshot>,
        current_desc: &str,
    ) -> Option<UndoFrame> {
        let frame = self.frames.pop_back()?;
        self.total_bytes = self.total_bytes.saturating_sub(frame.size_bytes);

        // Save current state as a redo frame.
        self.redo_frames
            .push_back(frame.with_snapshots(current_snapshots, current_desc));

        Some(frame)
    }

    /// Redo the last undone action.
    ///
    /// Pops the newest redo frame, pushes `current_snapshots` back onto the
    /// undo stack, and returns the frame to restore.
    ///
    /// Returns `None` if there is nothing to redo.
    pub fn pop_redo(
        &mut self,
        current_snapshots: Vec<SubtreeSnapshot>,
        current_desc: &str,
    ) -> Option<UndoFrame> {
        let frame = self.redo_frames.pop_back()?;

        // Re-push current state as an undo frame.
        let undo_frame = frame.with_snapshots(current_snapshots, current_desc);
        self.total_bytes += undo_frame.size_bytes;
        self.frames.push_back(undo_frame);

        Some(frame)
    }

    /// Replay what the master did to this stack (decisions §21 "6b"), so a
    /// member's copy matches the master's frame for frame.
    pub fn apply_step(&mut self, step: UndoStep) {
        match step {
            UndoStep::Push(frame) => self.push_frame(frame),
            UndoStep::Undo { redo } => {
                if let Some(frame) = self.frames.pop_back() {
                    self.total_bytes = self.total_bytes.saturating_sub(frame.size_bytes);
                }
                self.redo_frames.push_back(redo);
            }
            UndoStep::Redo { undo } => {
                self.redo_frames.pop_back();
                self.total_bytes += undo.size_bytes;
                self.frames.push_back(undo);
            }
        }
    }

    /// Clear all undo and redo frames.
    pub fn clear_all(&mut self) {
        self.frames.clear();
        self.redo_frames.clear();
        self.total_bytes = 0;
    }

    /// Return frame descriptions, newest first.
    pub fn list(&self) -> Vec<&str> {
        self.frames
            .iter()
            .rev()
            .map(|f| f.description.as_str())
            .collect()
    }

    /// Number of undo frames available.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns `true` if there are no undo frames.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Total memory used by undo frames in bytes.
    pub fn memory_usage(&self) -> usize {
        self.total_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(desc: &str) -> UndoFrame {
        UndoFrame {
            description: desc.to_string(),
            category: OopsCategory::Show,
            user: "Admin".to_string(),
            sequence_id: 0,
            snapshots: make_snapshots(desc),
            station: String::new(),
            time_ms: 0,
            main: false,
            local: Vec::new(),
            size_bytes: desc.len(),
        }
    }

    #[test]
    fn replayed_steps_leave_a_mirror_like_the_original() {
        // The master: push two, undo one, redo it.
        let mut master = UndoStack::default();
        let mut steps = Vec::new();
        master.push_frame(frame("a"));
        steps.push(UndoStep::Push(master.frames.back().unwrap().clone()));
        master.push_frame(frame("b"));
        steps.push(UndoStep::Push(master.frames.back().unwrap().clone()));
        master.pop_undo(make_snapshots("now"), "current").unwrap();
        steps.push(UndoStep::Undo {
            redo: master.redo_frames.back().unwrap().clone(),
        });
        master.pop_redo(make_snapshots("then"), "current").unwrap();
        steps.push(UndoStep::Redo {
            undo: master.frames.back().unwrap().clone(),
        });

        let mut member = UndoStack::default();
        for step in steps {
            member.apply_step(step);
        }
        assert_eq!(member.list(), master.list());
        assert_eq!(member.redo_frames.len(), master.redo_frames.len());
        assert_eq!(member.memory_usage(), master.memory_usage());
    }

    fn make_snapshots(content: &str) -> Vec<SubtreeSnapshot> {
        vec![SubtreeSnapshot {
            subtree: ShowSubtree::Programmer(uuid::Uuid::from_u128(1)),
            data: content.as_bytes().to_vec(),
        }]
    }

    fn make_frame(desc: &str, content: &str) -> UndoFrame {
        let data = content.as_bytes().to_vec();
        let size = data.len();
        UndoFrame {
            description: desc.to_string(),
            category: OopsCategory::Programmer,
            user: "Admin".to_string(),
            sequence_id: 0,
            snapshots: vec![SubtreeSnapshot {
                subtree: ShowSubtree::Programmer(uuid::Uuid::from_u128(1)),
                data,
            }],
            station: String::new(),
            time_ms: 0,
            main: false,
            local: Vec::new(),
            size_bytes: size,
        }
    }

    // ── Construction ──────────────────────────────────────────────────────────

    #[test]
    fn undo_new_is_empty() {
        let stack = UndoStack::new(1024);
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.memory_usage(), 0);
    }

    #[test]
    fn undo_default_has_10mb_budget() {
        let stack = UndoStack::default();
        assert_eq!(stack.max_bytes, DEFAULT_MAX_BYTES);
        assert_eq!(stack.max_frames, DEFAULT_MAX_FRAMES);
    }

    // ── Push ──────────────────────────────────────────────────────────────────

    #[test]
    fn undo_push_adds_frame_and_tracks_bytes() {
        let mut stack = UndoStack::new(1024 * 1024);
        let frame = make_frame("Fixture 1 At 50", "state_a");
        let size = frame.size_bytes;
        stack.push_frame(frame);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.memory_usage(), size);
    }

    #[test]
    fn undo_push_clears_redo_stack() {
        let mut stack = UndoStack::new(1024 * 1024);
        stack.push_frame(make_frame("a", "snap_a"));
        // Manually manufacture a redo frame to test it gets cleared.
        stack.redo_frames.push_back(make_frame("redo", "redo_snap"));
        assert!(!stack.redo_frames.is_empty());

        // A new push must clear redo.
        stack.push_frame(make_frame("b", "snap_b"));
        assert!(stack.redo_frames.is_empty());
    }

    // ── Pop undo ──────────────────────────────────────────────────────────────

    #[test]
    fn undo_pop_undo_returns_frame() {
        let mut stack = UndoStack::new(1024 * 1024);
        stack.push_frame(make_frame("Fixture 1 At 50", "before_command"));

        let current = make_snapshots("after_command");
        let restored = stack.pop_undo(current, "current");
        assert!(restored.is_some());
        assert_eq!(restored.unwrap().snapshots[0].data, b"before_command");
    }

    #[test]
    fn undo_pop_undo_empty_returns_none() {
        let mut stack = UndoStack::new(1024 * 1024);
        let result = stack.pop_undo(make_snapshots("cur"), "current");
        assert!(result.is_none());
    }

    #[test]
    fn undo_pop_undo_moves_current_to_redo() {
        let mut stack = UndoStack::new(1024 * 1024);
        stack.push_frame(make_frame("cmd", "snap_a"));

        let current = make_snapshots("current_state");
        stack.pop_undo(current, "current");

        assert_eq!(stack.redo_frames.len(), 1);
        assert_eq!(
            stack.redo_frames.back().unwrap().snapshots[0].data,
            b"current_state"
        );
    }

    #[test]
    fn undo_three_pushes_three_pops_in_reverse_order() {
        let mut stack = UndoStack::new(1024 * 1024);
        let snaps = ["snap_a", "snap_b", "snap_c"];
        for (i, s) in snaps.iter().enumerate() {
            stack.push_frame(make_frame(&format!("cmd_{i}"), s));
        }
        assert_eq!(stack.len(), 3);

        // Pop returns newest first.
        let r1 = stack.pop_undo(make_snapshots("cur"), "c1").unwrap();
        assert_eq!(r1.snapshots[0].data, b"snap_c");

        let r2 = stack.pop_undo(make_snapshots("cur"), "c2").unwrap();
        assert_eq!(r2.snapshots[0].data, b"snap_b");

        let r3 = stack.pop_undo(make_snapshots("cur"), "c3").unwrap();
        assert_eq!(r3.snapshots[0].data, b"snap_a");

        // 4th pop returns None.
        assert!(stack.pop_undo(make_snapshots("cur"), "c4").is_none());
    }

    // ── Pop redo ──────────────────────────────────────────────────────────────

    #[test]
    fn undo_pop_redo_empty_returns_none() {
        let mut stack = UndoStack::new(1024 * 1024);
        assert!(stack.pop_redo(make_snapshots("cur"), "c").is_none());
    }

    #[test]
    fn undo_redo_roundtrip() {
        let mut stack = UndoStack::new(1024 * 1024);
        stack.push_frame(make_frame("At 50", "before"));

        // Undo: back to 'before'.
        let after = make_snapshots("after");
        let restored = stack.pop_undo(after, "after").unwrap();
        assert_eq!(restored.snapshots[0].data, b"before");

        // Redo: back to 'after'.
        let before = make_snapshots("before");
        let redone = stack.pop_redo(before, "before").unwrap();
        assert_eq!(redone.snapshots[0].data, b"after");
    }

    // ── Memory trimming ───────────────────────────────────────────────────────

    #[test]
    fn undo_trims_oldest_when_over_budget() {
        // Budget of 20 bytes; each snapshot is 10 bytes.
        let mut stack = UndoStack::new(20);
        stack.max_frames = 1000; // don't let frame limit interfere
        stack.push_frame(make_frame("cmd1", "0123456789")); // 10 bytes
        stack.push_frame(make_frame("cmd2", "abcdefghij")); // 10 bytes
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.memory_usage(), 20);

        // Adding a 3rd frame (10 bytes) must trim the oldest.
        stack.push_frame(make_frame("cmd3", "klmnopqrst"));
        assert!(stack.memory_usage() <= 20);
        assert_eq!(stack.len(), 2); // oldest dropped
        // The remaining frames should be cmd2 and cmd3 (newest two).
        let descs = stack.list();
        assert!(descs.contains(&"cmd3"));
        assert!(descs.contains(&"cmd2"));
        assert!(!descs.contains(&"cmd1"));
    }

    #[test]
    fn undo_trims_when_over_frame_count() {
        let mut stack = UndoStack::new(1024 * 1024);
        stack.max_frames = 3;
        for i in 0..5 {
            stack.push_frame(make_frame(&format!("cmd{i}"), "x"));
        }
        assert_eq!(stack.len(), 3);
        let descs = stack.list();
        assert!(descs.contains(&"cmd4"));
        assert!(descs.contains(&"cmd3"));
        assert!(descs.contains(&"cmd2"));
    }

    // ── clear_all ─────────────────────────────────────────────────────────────

    #[test]
    fn undo_clear_all_empties_everything() {
        let mut stack = UndoStack::new(1024 * 1024);
        stack.push_frame(make_frame("a", "snap_a"));
        stack.push_frame(make_frame("b", "snap_b"));
        stack.redo_frames.push_back(make_frame("r", "redo"));
        stack.clear_all();
        assert!(stack.is_empty());
        assert!(stack.redo_frames.is_empty());
        assert_eq!(stack.total_bytes, 0);
    }

    // ── list() ────────────────────────────────────────────────────────────────

    #[test]
    fn undo_list_returns_newest_first() {
        let mut stack = UndoStack::new(1024 * 1024);
        stack.push_frame(make_frame("first", "a"));
        stack.push_frame(make_frame("second", "b"));
        stack.push_frame(make_frame("third", "c"));

        let list = stack.list();
        assert_eq!(list, vec!["third", "second", "first"]);
    }

    // ── Serde roundtrip ───────────────────────────────────────────────────────

    #[test]
    fn undo_serde_roundtrip_recomputes_size_bytes() {
        let mut stack = UndoStack::new(1024 * 1024);
        let frame = make_frame("cmd", "snapshot_data_here");
        let expected_size = frame.size_bytes;
        stack.push_frame(frame);

        // Serialise and deserialise.
        let json = serde_json::to_string(&stack).expect("serialize");
        let restored: UndoStack = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.frames.len(), 1);
        assert_eq!(restored.frames[0].size_bytes, expected_size);
    }

    // ── ShowSubtree display ───────────────────────────────────────────────────

    #[test]
    fn show_subtree_display() {
        assert_eq!(ShowSubtree::Devices.to_string(), "Devices");
        assert_eq!(
            ShowSubtree::Programmer(uuid::Uuid::from_u128(0xabcdef12 << 96)).to_string(),
            "Programmer(user abcdef12)"
        );
        assert_eq!(
            ShowSubtree::DataPool(uuid::Uuid::from_u128(0x12345678 << 96)).to_string(),
            "DataPool(12345678)"
        );
    }
}
