//  █████╗ ████████╗████████╗██████╗ ██╗██████╗ ██╗   ██╗████████╗███████╗███████╗
// ██╔══██╗╚══██╔══╝╚══██╔══╝██╔══██╗██║██╔══██╗██║   ██║╚══██╔══╝██╔════╝██╔════╝
// ███████║   ██║      ██║   ██████╔╝██║██████╔╝██║   ██║   ██║   █████╗  ███████╗
// ██╔══██║   ██║      ██║   ██╔══██╗██║██╔══██╗██║   ██║   ██║   ██╔══╝  ╚════██║
// ██║  ██║   ██║      ██║   ██║  ██║██║██████╔╝╚██████╔╝   ██║   ███████╗███████║
// ╚═╝  ╚═╝   ╚═╝      ╚═╝   ╚═╝  ╚═╝╚═╝╚═════╝  ╚═════╝    ╚═╝   ╚══════╝╚══════╝
// FILE: attributes

//! Attribute definitions — two layers that never mix (decisions §30 "12f-a").
//!
//! The show's layer (`AttributeDefinition`, `FeatureGroup`, `ActivationGroup`,
//! `AttributeRule` on `ShowData`) is what cue timing per feature group reads.
//! Each user profile has its own (`ProfileAttributes`): its own groups, rules
//! and knowledge of every attribute it ever placed, its readout / resolution
//! per attribute, and its encoder bar layouts. Both are keyed by the GDTF
//! attribute name.
//!
//! Owns:          the show's attribute definitions, feature groups, activation groups, rules, a profile's placement and encoder bars.
//! Doesn't own:   the attribute board's views (lumacore-views), the programmer's values.
//! Entry points:  AttributeDefinition, FeatureGroup, AttributeRule, ProfileAttributes, EncoderBarLayout.
//! Decisions:     §30 "12f-a", §34 25.10.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use lumacore_base::NodeMeta;
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Physical unit for an attribute — determines display and physical↔percent conversion.
///
/// Mirrors the GDTF `PhysicalUnit` enum (GDTF 1.2 §A.3.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PhysicalUnit {
    #[default]
    None,
    Percent,
    Length,              // m
    Mass,                // kg
    Time,                // s
    Temperature,         // K
    LuminousIntensity,   // cd
    Angle,               // deg
    Force,               // N
    Frequency,           // Hz
    Current,             // A
    Voltage,             // V
    Power,               // W
    Energy,              // J
    Area,                // m²
    Volume,              // m³
    Speed,               // m/s
    Acceleration,        // m/s²
    AngularSpeed,        // deg/s
    AngularAcceleration, // deg/s²
    WaveLength,          // nm
    ColorComponent,      // normalized 0–1
}

/// How this attribute's value is naturally displayed to the operator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NaturalReadout {
    /// Show as a percentage (0–100%).
    #[default]
    Percent,
    /// Show in the attribute's native physical unit (e.g. degrees for Pan/Tilt).
    Physical,
}

/// Encoder wheel resolution for this attribute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EncoderResolution {
    /// Standard coarse step.
    #[default]
    Coarse,
    /// Fine adjustment step.
    Fine,
    /// Ultra-fine step (sub-step precision).
    Ultra,
    /// Discrete button-style selection (no continuous range).
    Button,
}

/// The show's definition of one attribute (decisions §30 "12f-a"): which
/// feature group it is in (cue timing per feature group reads it), and what
/// the standard table / the fixture file say about it. Written when a
/// patched fixture brings an attribute the show doesn't know
/// (`attribute_resolver::sync`); DMX math stays on `ChannelFunction`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    pub meta: NodeMeta,
    /// Canonical GDTF name, e.g. "Dimmer", "Pan", "ColorRGB_R".
    /// Also used as the map key on `ShowData.attribute_defs`.
    pub name: String,
    /// Short display label, e.g. "Dim", "P", "R".
    pub pretty: String,
    /// Physical unit — authoritative source for display formatting.
    pub physical_unit: PhysicalUnit,
    /// How the value is naturally shown to the operator.
    #[serde(default)]
    pub natural_readout: NaturalReadout,
    /// ID into `ShowData.feature_groups`, if any.
    #[serde(default)]
    pub feature_group: Option<Uuid>,
    /// ID into `ShowData.activation_groups`, if any.
    #[serde(default)]
    pub activation_group: Option<Uuid>,
    /// Self-reference by canonical name to the parent attribute.
    /// `None` = this is a root attribute (e.g. "Pan" is root, "Pan_Fine" references "Pan").
    #[serde(default)]
    pub main_attribute: Option<String>,
    /// Global default encoder resolution.
    #[serde(default)]
    pub encoder_resolution: EncoderResolution,
    /// Relative rendering weight (0.0–1.0), `INTENSITY`.
    #[serde(default)]
    pub intensity: f32,
    /// Whether Move-In-Black applies to this attribute (`MIB`).
    #[serde(default)]
    pub mib: bool,
    /// The feature inside its group (e.g. "RGB" in Color), if any.
    #[serde(default)]
    pub feature: Option<String>,
    /// What the attribute means to the 3D view / tools (the standard
    /// table's `Special`, e.g. "Gobo", "Zoom"); never changes with the group.
    #[serde(default)]
    pub special: Option<String>,
    /// Order inside its feature group (the column / strip order).
    #[serde(default)]
    pub position: u32,
    /// How it got its group (the resolver's step, or the user).
    #[serde(default)]
    pub placement: Placement,
    /// Brought by a patch and not yet answered by the person patching (the
    /// board's New filter). Cleared when the patch is kept.
    #[serde(default)]
    pub new: bool,
}

/// How an attribute got its feature group — the resolver's step, in order of
/// confidence (decisions §30 "12f-a": exact · family · parent · context ·
/// hint · guess), or placed by a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlaceHow {
    /// A name of the standard table.
    #[default]
    Exact,
    /// A family rule (`Shutter{n}*`).
    Family,
    /// Its main attribute's group.
    Parent,
    /// Its activation group's members.
    Context,
    /// A keyword or the fixture file's Feature.
    Hint,
    /// Nothing matched: the fallback group.
    Guess,
    /// Placed by a user.
    User,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl PlaceHow {
    pub fn word(self) -> &'static str {
        match self {
            PlaceHow::Exact => "exact",
            PlaceHow::Family => "family",
            PlaceHow::Parent => "parent",
            PlaceHow::Context => "context",
            PlaceHow::Hint => "hint",
            PlaceHow::Guess => "guess",
            PlaceHow::User => "user",
        }
    }
}

/// The resolver's record of a placement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Placement {
    pub how: PlaceHow,
    /// Why, in words ("family Shutter{n}*", "keyword \"invert\"").
    #[serde(default)]
    pub why: String,
    /// What the fixture file said (`Feature="Beam.Beam"`), kept for Info.
    #[serde(default)]
    pub file_feature: Option<String>,
}

/// One rule of the resolver (decisions §30 "12f-a"): a name pattern and the
/// group (and feature) it places into. `*` = any characters, `{n}` = digits,
/// case-insensitive, whole name. Rules are tried in order (the pool number);
/// family rules place with confidence, keyword rules only hint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeRule {
    pub meta: NodeMeta,
    pub pattern: String,
    /// The feature group it places into (a GUID link).
    pub group: Uuid,
    #[serde(default)]
    pub feature: Option<String>,
    #[serde(default)]
    pub kind: RuleKind,
    /// The template class it came from ("shutter", "gobo" …); the exact step
    /// finds a standard name's group through it. `None` for learned rules.
    #[serde(default)]
    pub class: Option<String>,
    /// Made by "Apply to similar names?" (a user's move), not by a template.
    #[serde(default)]
    pub learned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RuleKind {
    #[default]
    Family,
    Keyword,
}

/// A feature group (decisions §30 "12f-a"): the show's grouping of
/// attributes that cue timing per feature group reads; a profile has its own
/// for its encoder bar. Freely named ("Dimmer", "FX", "Shutter" …).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureGroup {
    pub meta: NodeMeta,
    /// The features inside it, in order (e.g. Color: RGB, Color, HSB …).
    #[serde(default)]
    pub features: Vec<String>,
}

/// An activation group: links mutually-exclusive attributes.
///
/// When one attribute in the group is activated, others are deactivated.
/// Examples: "PanTilt" (Pan + Tilt), "ColorRGB" (R + G + B).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationGroup {
    pub meta: NodeMeta,
    /// Display name, e.g. "PanTilt", "ColorRGB".
    pub name: String,
}

/// A user profile's own attribute layer (decisions §30 "12f-a"): its groups
/// (by GUID, its own), its resolver rules and template, every attribute it
/// ever placed (kept when no patched fixture uses it any more — the profile
/// gets smarter), and its encoder bar layouts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileAttributes {
    #[serde(default)]
    pub template: String,
    #[serde(default)]
    pub groups: BTreeMap<u32, FeatureGroup>,
    #[serde(default)]
    pub rules: BTreeMap<u32, AttributeRule>,
    /// Keyed by attribute name.
    #[serde(default)]
    pub placed: BTreeMap<String, PlacedAttribute>,
    /// The encoder bar layouts (a pool; each screen shows one).
    #[serde(default)]
    pub encoder_bars: BTreeMap<u32, EncoderBarLayout>,
}

impl ProfileAttributes {
    /// The profile has a layer of its own yet.
    pub fn is_seeded(&self) -> bool {
        !self.groups.is_empty()
    }
    /// Attributes waiting for the user's answer (placed at the suggestion, marked new).
    pub fn waiting(&self) -> Vec<&str> {
        self.placed.iter().filter(|(_, p)| p.new).map(|(n, _)| n.as_str()).collect()
    }
}

/// Where a profile put one attribute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedAttribute {
    /// One of the profile's own groups.
    pub group: Uuid,
    #[serde(default)]
    pub feature: Option<String>,
    #[serde(default)]
    pub position: u32,
    #[serde(default)]
    pub placement: Placement,
    /// Waiting for the user's answer (the new-attribute dialog).
    #[serde(default)]
    pub new: bool,
    /// The profile's readout / encoder resolution for it (None = the show's).
    #[serde(default)]
    pub readout: Option<NaturalReadout>,
    #[serde(default)]
    pub resolution: Option<EncoderResolution>,
}

/// One encoder bar layout of a profile (an EncoderBarPool): strip
/// buttons, each one of the profile's groups with its 4-slot pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderBarLayout {
    pub meta: NodeMeta,
    #[serde(default)]
    pub buttons: Vec<EncoderButton>,
}

/// A strip button: a group of the profile and its slots (four per page;
/// `None` = an empty slot).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncoderButton {
    pub group: Uuid,
    #[serde(default)]
    pub slots: Vec<Option<String>>,
}
