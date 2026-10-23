// ███████╗██╗██╗  ██╗████████╗██╗   ██╗██████╗ ███████╗
// ██╔════╝██║╚██╗██╔╝╚══██╔══╝██║   ██║██╔══██╗██╔════╝
// █████╗  ██║ ╚███╔╝    ██║   ██║   ██║██████╔╝█████╗
// ██╔══╝  ██║ ██╔██╗    ██║   ██║   ██║██╔══██╗██╔══╝
// ██║     ██║██╔╝ ██╗   ██║   ╚██████╔╝██║  ██║███████╗
// ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: fixture

//! Fixture: a patched fixture instance in ShowData.
//!
//! A Fixture is a patched instance of a FixtureType revision+mode, in two parts: what MVR says
//! (`fixture_mvr.rs`) and what only Luma has. Its breaks (universe + start address + channel count) are
//! MVR's `<Addresses>`.
//!
//! Owns:          Fixture, LumaFixture, FixtureBreak, MasterReact.
//! Doesn't own:   the patch's rules (lumacore-patch), selections (the programmer).
//! Entry points:  Fixture, FixtureBreak, MasterReact.
//! Decisions:     §11, §21 "6d", §34 25.10, §34 "34g4 patch hardened".

use lumacore_base::node::{NodeMeta, TreeNode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fixture_mvr::MvrFixture;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A single break assignment for a fixture.
///
/// Phase 1 fixtures always have exactly one break (len=1), but the Vec
/// design accommodates multi-universe fixtures (footprints >512 channels)
/// from day one per CONTEXT.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureBreak {
    /// The universe GUID this break is patched to.
    pub universe_id: Uuid,
    /// The 1-indexed DMX start address (1-512).
    pub start_address: u16,
    /// Number of DMX channels consumed by this break.
    pub channel_count: u16,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl FixtureBreak {
    pub fn new(universe_id: Uuid, start_address: u16, channel_count: u16) -> Self {
        Self {
            universe_id,
            start_address,
            channel_count,
        }
    }

    /// Returns the last occupied address (1-indexed, inclusive).
    pub fn end_address(&self) -> u16 {
        self.start_address + self.channel_count - 1
    }
}

/// A patched fixture instance (decisions §34 "34g4 patch hardened" 2): its meta (the MVR uuid
/// as GUID, the name, the note), what MVR says about it, and what only Luma has. Its ids are
/// three (decisions §40 7): `mvr.fid` (the number, may be none), `mvr.id_type` + `mvr.cid`
/// (the second id). The pool's key is a [`Slot`], never a number a line names. A fixture may
/// hold fixtures (`mvr.parent` a fixture, §40 3); it never has sub-fixtures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub meta: NodeMeta,
    /// Every MVR 1.6 `<Fixture>` field (type, revision, mode and breaks among them).
    pub mvr: MvrFixture,
    /// What MVR lacks.
    #[serde(default)]
    pub luma: LumaFixture,
}

/// The fields of a fixture MVR has no place for.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LumaFixture {
    /// Which masters act on this fixture (Master React, decisions §22).
    #[serde(default)]
    pub master_react: MasterReact,
    /// Its multipatch copies (decisions §34 round 3 19, where they live decided 2026-10-19: inside
    /// their fixture): each mirrors the fixture's channels at its own address, numbered by place
    /// (`Fixture 201 Multipatch 1`). MVR import / export maps each to MVR's separate fixture with
    /// `multipatch` set. Empty is not saved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub multipatch: Vec<MultipatchCopy>,
}

/// One multipatch copy of a fixture: its own GUID and name, address and place; no values of its
/// own (it follows its fixture).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipatchCopy {
    pub meta: NodeMeta,
    #[serde(default)]
    pub breaks: Vec<FixtureBreak>,
    #[serde(default)]
    pub matrix: crate::Matrix,
}

/// Which masters act on a fixture's intensity (Master React).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MasterReact {
    /// Group masters and the grand master.
    #[default]
    Grand,
    /// Group masters; Additive group masters skip the grand master.
    Group,
    /// No master and no Solo (house lights).
    None,
}

impl MasterReact {
    pub const NAMES: &'static [&'static str] = &["Grand", "Group", "None"];

    pub fn parse(word: &str) -> Option<Self> {
        Some(match word.to_ascii_lowercase().as_str() {
            "grand" => Self::Grand,
            "group" => Self::Group,
            "none" => Self::None,
            _ => return None,
        })
    }
}

/// The patch's key for a fixture (decisions §40 5): given at insert, stable, never shown and
/// never on a line — `Fixture 849` is the fixture whose `fid` is 849, found through
/// `PatchData::fixture_by_fid`. A newtype so no site can mistake it for the number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Slot(pub u32);
// No `Display` on purpose: a slot that reached a line or a cell would read like a number. Where a
// file or a log needs it, `slot.0` says so.

impl Fixture {
    /// Create a new single-break fixture (Phase 1 standard).
    #[allow(
        clippy::too_many_arguments,
        reason = "a fixture's identity and first break are set together; a builder comes with the change path"
    )]
    pub fn new(
        name: Option<String>,
        fixture_type_guid: Uuid,
        revision: Uuid,
        mode: Uuid,
        universe_guid: Uuid,
        start_address: u16,
        channel_count: u16,
    ) -> Self {
        Self {
            meta: NodeMeta::new(name),
            mvr: MvrFixture::new(
                fixture_type_guid,
                revision,
                mode,
                vec![FixtureBreak::new(universe_guid, start_address, channel_count)],
            ),
            luma: LumaFixture::default(),
        }
    }

    /// Returns the primary break (first break — Phase 1 standard).
    pub fn primary_break(&self) -> Option<&FixtureBreak> {
        self.mvr.breaks.first()
    }

    /// Every second id it answers to: MVR's (when it has one), then the more ids.
    pub fn second_ids(&self) -> impl Iterator<Item = (crate::IdType, u32)> + '_ {
        self.mvr.cid.filter(|_| self.mvr.id_type != crate::IdType::None).map(|cid| (self.mvr.id_type, cid)).into_iter()
    }

    /// Its number, `Fixture 849`; a folder has none.
    pub fn fid(&self) -> Option<u32> {
        self.mvr.fid
    }

    /// How a line names it: `Fixture 849`, `Fixture Media 7003`, else its name.
    pub fn handle(&self) -> String {
        match (self.mvr.fid, self.second_ids().next()) {
            (Some(n), _) => format!("Fixture {n}"),
            (None, Some((t, n))) => format!("Fixture {} {n}", t.name()),
            (None, None) => format!("Fixture \"{}\"", self.meta.name().unwrap_or("")),
        }
    }

    /// Total channel count across all breaks.
    pub fn total_channels(&self) -> u16 {
        self.mvr.breaks.iter().map(|b| b.channel_count).sum()
    }
}

impl TreeNode for Fixture {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Fixture"
    }
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
