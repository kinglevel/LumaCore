// ███╗   ███╗██╗   ██╗██████╗     ██████╗ ███████╗██╗   ██╗██╗███████╗██╗    ██╗
// ████╗ ████║██║   ██║██╔══██╗    ██╔══██╗██╔════╝██║   ██║██║██╔════╝██║    ██║
// ██╔████╔██║██║   ██║██████╔╝    ██████╔╝█████╗  ██║   ██║██║█████╗  ██║ █╗ ██║
// ██║╚██╔╝██║╚██╗ ██╔╝██╔══██╗    ██╔══██╗██╔══╝  ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║
// ██║ ╚═╝ ██║ ╚████╔╝ ██║  ██║    ██║  ██║███████╗ ╚████╔╝ ██║███████╗╚███╔███╔╝
// ╚═╝     ╚═╝  ╚═══╝  ╚═╝  ╚═╝    ╚═╝  ╚═╝╚══════╝  ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝
// FILE: mvr_review

//! The MVR import review (decisions §34 "Import MVR hardened" 3): what `Import Mvr "file"` found
//! against the holder's Devices draft — one row per file fixture with its status, match key and
//! the file's value per field beside the live one —, kept on the draft until Apply or Cancel and
//! published in the `patch` feed as the grid's review mode. Nothing is written while it stands.
//! The rows' numbers are the fixtures they match or make; the grid's selection is the ticks.
//!
//! Owns:          ImportReview, ReviewRow, ReviewCell, RowStatus, ReviewSettings, ReviewCounts.
//! Doesn't own:   building it (`lumacore-objects` patch/mvr/review), the file (`lumacore-mvr`).
//! Entry points:  ImportReview::counts, summary.
//! Decisions:     §34 "Import MVR hardened" 2–4.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lumacore_tree::MvrTake;
// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES


/// The review of one file against the draft.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportReview {
    /// The file as named on the line, and its drive.
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drive: Option<String>,
    /// The scope as typed (`MvrGroup 3`, `Tag "Truss"`); None = the whole patch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub settings: ReviewSettings,
    pub rows: Vec<ReviewRow>,
    /// The placement answers so far (decision 4): missing group uuid → Recreate | Parent | None.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub answers: std::collections::BTreeMap<Uuid, String>,
}

/// The settings the review was built with (the preset with the line's overrides).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewSettings {
    pub match_key: String,
    pub lost: String,
    pub rotation: String,
    pub missing: String,
    /// The id type an untyped CustomId gets (Generic by default).
    #[serde(default)]
    pub id_type: String,
    pub addresses: MvrTake,
    pub ids: MvrTake,
    pub names: MvrTake,
    pub position: MvrTake,
    pub revisions: MvrTake,
    pub new_fixtures: MvrTake,
    pub fid_offset: i64,
    pub universe_offset: i64,
    pub offset_mm: [i64; 3],
    pub rotate_deg: i64,
}

/// What a row is against the draft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RowStatus {
    /// Matched, nothing differs.
    Same,
    /// Matched, cells differ.
    Changed,
    /// Not in the draft: a fixture the file adds.
    New,
    /// In the draft's scope, not in the file.
    Lost,
    /// Matched a fixture outside the scope.
    Outside,
    /// Its address or number is another fixture's: never taken, blocks Apply while ticked.
    Clash,
}

impl RowStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Same => "Same",
            Self::Changed => "Changed",
            Self::New => "New",
            Self::Lost => "Lost",
            Self::Outside => "Outside",
            Self::Clash => "Clash",
        }
    }
}

/// One file fixture (or a lost draft fixture) in the review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewRow {
    /// The fixture it matches, or the number a new one takes (0 = none: a folder, decisions §40 7).
    pub no: u32,
    /// How a line will name it when it has no number: its second id (`Media 7003`), else "".
    #[serde(default)]
    pub id: String,
    /// Its second id as it will be written: the type (`""` = none) and the number.
    #[serde(default)]
    pub id_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid: Option<u32>,
    /// The file fixture it sits in (MVR's `ChildList`, decisions §40 3): a nested fixture's parent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_fixture: Option<Uuid>,
    /// Its place among its siblings in the file (decisions §40 6).
    #[serde(default)]
    pub order: u32,
    /// The file fixture's uuid (a lost row: the draft fixture's GUID).
    pub uuid: Uuid,
    pub status: RowStatus,
    /// The draft fixture it matched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched: Option<Uuid>,
    /// `UUID`, `FID`, `Name` or "".
    pub match_key: String,
    pub name: String,
    pub gdtf_spec: String,
    pub gdtf_mode: String,
    /// The zip holds the GDTF it names.
    pub gdtf_embedded: bool,
    /// The show has a type for it (a revision with the embedded file's hash).
    pub type_known: bool,
    /// Its MVR group in the file (None: straight under the layer), and the names for the grid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<Uuid>,
    pub group_name: String,
    pub layer: Uuid,
    pub layer_name: String,
    /// The group is gone from the draft (a placement ask at Apply, decision 4).
    pub group_missing: bool,
    pub cells: Vec<ReviewCell>,
    /// Pre-selected in the grid (the ticks).
    pub taken: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clash: Option<String>,
}

/// One field the file may change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewCell {
    /// The grid's field (`name`, `address`, `pos`, `rot`, `type`, `fid`, `unit`, `custom_id`).
    pub field: String,
    /// Its group (`Addresses`, `Ids`, `Names`, `Position`, `Revisions`).
    pub group: String,
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live: Option<String>,
    pub differs: bool,
    pub mode: MvrTake,
    /// Taken into the draft at Apply (the group's Take, or a cell's own `/Take`).
    pub taken: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clash: Option<String>,
}

/// How many rows of each status, and how many are ticked.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewCounts {
    pub same: usize,
    pub changed: usize,
    pub new: usize,
    pub lost: usize,
    pub outside: usize,
    pub clash: usize,
    pub taken: usize,
}
// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC


impl ImportReview {
    pub fn counts(&self) -> ReviewCounts {
        let mut c = ReviewCounts::default();
        for r in &self.rows {
            match r.status {
                RowStatus::Same => c.same += 1,
                RowStatus::Changed => c.changed += 1,
                RowStatus::New => c.new += 1,
                RowStatus::Lost => c.lost += 1,
                RowStatus::Outside => c.outside += 1,
                RowStatus::Clash => c.clash += 1,
            }
            if r.taken {
                c.taken += 1;
            }
        }
        c
    }

    /// One line: `"Rig v3.mvr": 148 fixtures — 140 same, 6 changed, 8 new, 2 lost, 1 clash · 14 ticked`.
    pub fn summary(&self) -> String {
        let c = self.counts();
        let mut parts = Vec::new();
        for (n, word) in [(c.same, "same"), (c.changed, "changed"), (c.new, "new"), (c.lost, "lost"), (c.outside, "outside the scope"), (c.clash, "clash")] {
            if n > 0 {
                parts.push(format!("{n} {word}"));
            }
        }
        format!("\"{}\": {} rows — {} · {} ticked", self.file, self.rows.len(), if parts.is_empty() { "nothing".to_string() } else { parts.join(", ") }, c.taken)
    }

    /// The rows by number.
    pub fn row(&self, no: u32) -> Option<&ReviewRow> {
        self.rows.iter().find(|r| r.no == no)
    }
}
