// ███████╗██╗██╗  ██╗████████╗██╗   ██╗██████╗ ███████╗
// ██╔════╝██║╚██╗██╔╝╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝
// █████╗  ██║ ╚███╔╝    ██║    ╚████╔╝ ██████╔╝█████╗
// ██╔══╝  ██║ ██╔██╗    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝
// ██║     ██║██╔╝ ██╗   ██║      ██║   ██║     ███████╗
// ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝      ╚═╝   ╚═╝     ╚══════╝
// FILE: fixtype

//! FixtureType, Revision, Mode, and ChannelFunction types.
//!
//! Models the GDTF fixture type hierarchy: FixtureType > Revision > Mode.
//! Re-importing a fixture creates a new Revision — users choose revision and
//! mode at patch time.
//!
//! GDTF native (decisions §29 "11b hardening"): the `.gdtf` file is the
//! source of truth (it travels in the show container, keyed by
//! `Revision::file_hash`); everything here is a view derived from it by
//! `lumacore_gdtf` — geometry tree, cells, every channel function with its
//! ranges and channel sets, highlight values, snap, wheels and physical
//! descriptions. Nothing here is edited by hand.
//!
//! Owns:          FixtureType, its revisions, modes, geometry, wheels, emitters, filters, channel functions.
//! Doesn't own:   parsing a .gdtf (lumacore-gdtf), patching (lumacore-patch).
//! Entry points:  FixtureType, Revision, Mode, ChannelFunction, AttributeType.
//! Decisions:     §21 "6d", §30 "12a2", §34 25.10.

use lumacore_base::node::{NodeMeta, TreeNode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The broad category of a fixture, derived from GDTF metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixtureCategory {
    Spot,
    Wash,
    Beam,
    Strobe,
    Effect,
    Led,
    Conventional,
    Other(String),
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Default for FixtureCategory {
    fn default() -> Self {
        FixtureCategory::Other("Unknown".to_string())
    }
}

/// The functional type of a DMX attribute.
///
/// Used for grouped-by-type channel listings and attribute-type filtering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum AttributeType {
    Dimmer,
    Color,
    ColorRed,
    ColorGreen,
    ColorBlue,
    ColorWhite,
    ColorAmber,
    Pan,
    Tilt,
    Zoom,
    Focus,
    Iris,
    Frost,
    Gobo,
    Shutter,
    Speed,
    Effect,
    Macro,
    Control,
    #[default]
    NoFeature,
    Unknown(String),
}

fn first_break() -> u8 {
    1
}

/// A GDTF channel set: a named DMX range of a channel function (a gobo
/// slot, "Open", "Strobe fast" …).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChannelSet {
    pub name: String,
    /// Start of the range as a fraction (0–1) of the channel's full range.
    pub dmx_from: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_from: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_to: Option<f64>,
    /// The wheel slot it selects (1-based), when the function names a wheel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wheel_slot: Option<i32>,
}

/// One GDTF ChannelFunction of a logical channel: a DMX range with its own
/// attribute, physical range and channel sets.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GdtfFunction {
    pub name: String,
    pub attribute: String,
    /// Start of the range as a fraction (0–1) of the channel's full range.
    pub dmx_from: f64,
    pub physical_from: f64,
    pub physical_to: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wheel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emitter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub channel_sets: Vec<ChannelSet>,
}

/// A cell of a multi-cell fixture (a GDTF `GeometryReference` instance of a
/// channel-bearing template). Cells are never selectable (§28): only the
/// effect generator expands them.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    /// Order in the mode (by DMX offset), from 1.
    pub index: u32,
    /// The GeometryReference's name.
    pub geometry: String,
    /// The template geometry it instances.
    pub template: String,
    /// Its first DMX slot (0-based offset from the fixture's address).
    pub dmx_offset: u16,
    /// Position composed from the geometry tree (metres, fixture space).
    pub position: [f64; 3],
}

/// A GDTF relation between two channels of a mode (`Multiply` = a master
/// dimmer, `Override`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    pub name: String,
    pub master: String,
    pub follower: String,
    pub kind: String,
}

/// A node of the GDTF geometry tree.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeometryNode {
    pub name: String,
    /// GDTF element: Geometry, Axis, Beam, FilterColor, GeometryReference …
    pub kind: String,
    /// Position matrix, row-major 4×4.
    pub position: Vec<f64>,
    /// A GeometryReference's template geometry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// A GeometryReference's breaks: (DMX break, DMX offset, 1-based).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breaks: Vec<(u8, u32)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<GeometryNode>,
}

/// A colour in CIE 1931 xyY.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Cie {
    pub x: f64,
    pub y: f64,
    #[serde(rename = "Y")]
    pub big_y: f64,
}

/// A slot of a wheel.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WheelSlot {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Cie>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    /// Gobo / media file inside the .gdtf.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<String>,
}

/// A GDTF wheel (colour, gobo, prism, animation).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Wheel {
    pub name: String,
    pub slots: Vec<WheelSlot>,
}

/// A GDTF emitter (an LED colour), for colour mixing.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Emitter {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Cie>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dominant_wavelength: Option<f64>,
}

/// A GDTF filter (a subtractive colour).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    pub name: String,
    pub color: Cie,
}

/// A single DMX channel within a mode (one per cell for a cell's channels).
///
/// Maps one DMX slot (or two for 16-bit) to a named attribute.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelFunction {
    /// The GDTF attribute name (e.g., "Dimmer", "Pan", "ColorRGB_R").
    pub attribute_name: String,
    /// The resolved attribute type for grouped display and filtering.
    pub attribute_type: AttributeType,
    /// Zero-indexed DMX offset from the fixture's start address.
    pub dmx_offset: u16,
    /// Number of bytes: 1 for 8-bit, 2 for 16-bit channels.
    pub byte_count: u8,
    /// Default DMX value (0-255 for 8-bit channels).
    pub default_value: u8,
    /// Default as a fraction (0–1) of the channel's full range, all bytes
    /// (GDTF `Default`); `None` in older shows → `default_value / 255`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_fraction: Option<f64>,
    /// Physical range of the channel (GDTF `PhysicalFrom`/`PhysicalTo`), for
    /// physical values. `None` → physical values can't be output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_range: Option<(f64, f64)>,
    /// True if this is a virtual channel (references a geometry, not a DMX slot).
    pub is_virtual: bool,
    /// True if the virtual reference was successfully resolved during import.
    /// False means the channel was skipped (broken reference).
    pub resolved: bool,
    /// The geometry it drives: the cell's GeometryReference for a cell
    /// channel, else the GDTF channel's own geometry.
    #[serde(default)]
    pub geometry: String,
    /// Which cell (`Mode::cells` index), None = shared by the fixture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell: Option<u32>,
    /// GDTF DMX break (1 = the fixture's first break).
    #[serde(default = "first_break")]
    pub dmx_break: u8,
    /// GDTF `DMXChannel@Highlight` as a fraction of the range (None: the
    /// channel has no highlight value — Highlight leaves it alone).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<f64>,
    /// GDTF `LogicalChannel@Snap`: the value jumps (at the end of its
    /// delay) instead of fading.
    #[serde(default)]
    pub snap: bool,
    /// Every ChannelFunction of the channel's (first) logical channel.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<GdtfFunction>,
}

/// A DMX mode — one specific channel layout within a revision.
///
/// The total footprint is the sum of all channel byte_counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode {
    pub meta: NodeMeta,
    /// All DMX channel functions in DMX offset order.
    pub channel_functions: Vec<ChannelFunction>,
    /// Total DMX footprint in channels (slots).
    pub dmx_footprint: u16,
    /// The mode's root geometry (`DMXMode@Geometry`).
    #[serde(default)]
    pub geometry: String,
    /// The cell layout (empty: a single-cell fixture).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cells: Vec<Cell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<Relation>,
}

impl TreeNode for Mode {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Mode"
    }
}

/// A revision of a fixture type — created each time the GDTF file is re-imported.
///
/// Users choose revision and mode per patched fixture instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub meta: NodeMeta,
    /// GDTF spec version string (e.g., "1.2").
    pub gdtf_version: String,
    /// SHA-256 hash of the original .gdtf file for re-import detection.
    pub file_hash: String,
    /// All DMX modes in this revision.
    pub modes: Vec<Mode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub geometries: Vec<GeometryNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wheels: Vec<Wheel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emitters: Vec<Emitter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<Filter>,
    /// What the fixture file says about its attributes (GDTF
    /// `<AttributeDefinitions>`): the attribute resolver reads it when the
    /// show meets an attribute it doesn't know (decisions §30 "12f-a").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<FileAttribute>,
}

/// One `<Attribute>` of a fixture file, as the file says it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileAttribute {
    pub name: String,
    #[serde(default)]
    pub pretty: String,
    /// `Feature="Group.Feature"`, as written (a hint, not an authority).
    #[serde(default)]
    pub feature: Option<String>,
    #[serde(default)]
    pub main_attribute: Option<String>,
    #[serde(default)]
    pub activation_group: Option<String>,
    /// The GDTF `PhysicalUnit` name.
    #[serde(default)]
    pub physical_unit: Option<String>,
}

impl TreeNode for Revision {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Revision"
    }
}

/// A fixture type, representing one GDTF profile in the library.
///
/// May have multiple revisions (one per import). Users choose a revision
/// and mode when patching a fixture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureType {
    pub meta: NodeMeta,
    /// Manufacturer name from GDTF metadata.
    pub manufacturer: String,
    /// Broad fixture category for filtering and display.
    pub category: FixtureCategory,
    /// All revisions of this fixture type, in import order.
    pub revisions: Vec<Revision>,
    /// The GDTF file's own `FixtureTypeID` (decisions §30 "12a2"): links use
    /// the fixture type's GUID; this matches a re-import (or a downloaded
    /// update) of the same type, which adds a [`Revision`].
    #[serde(default)]
    pub gdtf_fixture_type_id: Option<Uuid>,
}

impl FixtureType {
    pub fn new(
        manufacturer: String,
        name: Option<String>,
        category: FixtureCategory,
    ) -> Self {
        Self {
            meta: NodeMeta::new(name),
            manufacturer,
            category,
            revisions: Vec::new(),
            gdtf_fixture_type_id: None,
        }
    }

    /// Returns the most recently imported revision.
    pub fn latest_revision(&self) -> Option<&Revision> {
        self.revisions.last()
    }

    /// The revision and mode a fixture uses (its GUID links).
    pub fn mode_of(&self, revision: Uuid, mode: Uuid) -> Option<(&Revision, &Mode)> {
        let rev = self.revisions.iter().find(|r| r.meta.identity.guid == revision)?;
        let m = rev.modes.iter().find(|m| m.meta.identity.guid == mode)?;
        Some((rev, m))
    }

    /// The GUIDs of revision `rev` / mode `mode` by position (patch input).
    pub fn mode_guids(&self, rev: usize, mode: usize) -> Option<(Uuid, Uuid)> {
        let r = self.revisions.get(rev)?;
        Some((r.meta.identity.guid, r.modes.get(mode)?.meta.identity.guid))
    }
}

impl TreeNode for FixtureType {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "FixtureType"
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
