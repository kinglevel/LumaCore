//! Datapool: named container for groups, presets, layouts, distribution presets,
//! dynamic states, plugins, timecodes, and related reusable show content.
//!
//! A show can have multiple datapools (e.g., "songname", "busk1") each
//! containing its own Groups and other data.

use crate::node::{NodeMeta, TreeNode};
pub use lumacore_timecode::{
    LoopRegion, SignatureEntry, TempoEntry, TimeBase, TimecodeClip, TimecodeEvent, TimecodeItem,
    TimecodeMarker, TimecodeRegion, TimecodeTrack, TrackEvents,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatapoolPriorityOrder {
    pub meta: NodeMeta,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutItem {
    pub meta: NodeMeta,
}

impl TreeNode for LayoutItem {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Layout"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterItem {
    pub meta: NodeMeta,
}

impl TreeNode for FilterItem {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Filter"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldItem {
    pub meta: NodeMeta,
}

impl TreeNode for WorldItem {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "World"
    }
}

/// How a macro line waits before proceeding.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MacroWait {
    /// Wait a fixed number of seconds.
    Time(f64),
    /// Proceed as soon as the previous line finishes.
    #[default]
    Follow,
    /// Wait for an explicit Go trigger.
    Go,
}

/// Whether a macro line auto-executes or is merely placed on the command line.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MacroExecute {
    #[default]
    Yes,
    No,
}

/// A single line inside a Macro.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroLine {
    pub meta: NodeMeta,
    pub command: String,
    #[serde(default)]
    pub wait: MacroWait,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub execute: MacroExecute,
}

impl Default for MacroLine {
    fn default() -> Self {
        Self {
            meta: NodeMeta::default(),
            command: String::new(),
            wait: MacroWait::default(),
            enabled: true,
            execute: MacroExecute::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroItem {
    pub meta: NodeMeta,
    #[serde(default)]
    pub lines: Vec<MacroLine>,
}

impl TreeNode for MacroItem {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Macro"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributePreset {
    pub meta: NodeMeta,
    #[serde(default)]
    pub context: String,
}

impl TreeNode for DistributePreset {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "DistributePreset"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicState {
    pub meta: NodeMeta,
    #[serde(default)]
    pub description: String,
}

impl TreeNode for DynamicState {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "DynamicState"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginItem {
    pub meta: NodeMeta,
    /// From the plugin's `plugin.toml` manifest (`[plugin] id`); namespaces its
    /// keys (`plugin.<id>.<button>`).
    #[serde(default)]
    pub plugin_id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    /// Lua files (`[[script]] file`).
    #[serde(default)]
    pub scripts: Vec<String>,
    /// Buttons (`[[button]]`): global, locked UI keys.
    #[serde(default)]
    pub buttons: Vec<crate::ui_keys::UiKey>,
}

impl TreeNode for PluginItem {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Plugin"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityItem {
    pub meta: NodeMeta,
    #[serde(default)]
    pub level: i32,
}

impl TreeNode for PriorityItem {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Priority"
    }
}

/// A named datapool containing Groups and other programmable data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Datapool {
    pub meta: NodeMeta,
    pub groups: BTreeMap<u32, Group>,
    /// The one preset pool (decisions 0288): `Preset <n>`, whatever feature groups its values
    /// touch; a window filters it by the programmer's feature group and the presets' tags.
    #[serde(default)]
    pub presets: BTreeMap<u32, Preset>,
    #[serde(default)]
    pub layouts: BTreeMap<u32, LayoutItem>,
    #[serde(default)]
    pub filters: BTreeMap<u32, FilterItem>,
    #[serde(default)]
    pub worlds: BTreeMap<u32, WorldItem>,
    #[serde(default)]
    pub macros: BTreeMap<u32, MacroItem>,
    #[serde(default)]
    pub distribute_presets: BTreeMap<u32, DistributePreset>,
    #[serde(default)]
    pub dynamic_states: BTreeMap<u32, DynamicState>,
    #[serde(default)]
    pub plugins: BTreeMap<u32, PluginItem>,
    #[serde(default)]
    pub timecodes: BTreeMap<u32, TimecodeItem>,
    #[serde(default)]
    pub priorities: BTreeMap<u32, PriorityItem>,
    /// Cue lists (decisions §32, §30 "12f": DataPool objects like Groups). Shared per sequence
    /// (decisions §31 "Arc parts"): a published copy shares every sequence nobody changed, and
    /// the plan compiler keeps what it worked out while its `Arc` stays. Change one with
    /// `ShowData::sequence_mut`.
    #[serde(default)]
    pub sequences: BTreeMap<u32, std::sync::Arc<lumacore_playback::SequenceObject>>,
    #[serde(default)]
    pub timing_presets: BTreeMap<u32, lumacore_playback::TimingPreset>,
    #[serde(default)]
    pub curve_presets: BTreeMap<u32, lumacore_playback::CurvePreset>,
}

impl TreeNode for Datapool {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Datapool"
    }
}

impl Datapool {
    pub fn new(name: Option<String>) -> Self {
        Self {
            meta: NodeMeta::new(name),
            groups: BTreeMap::new(),
            presets: BTreeMap::new(),
            layouts: BTreeMap::new(),
            filters: BTreeMap::new(),
            worlds: BTreeMap::new(),
            macros: BTreeMap::new(),
            distribute_presets: BTreeMap::new(),
            dynamic_states: BTreeMap::new(),
            plugins: BTreeMap::new(),
            timecodes: BTreeMap::new(),
            priorities: BTreeMap::new(),
            sequences: BTreeMap::new(),
            timing_presets: BTreeMap::new(),
            curve_presets: BTreeMap::new(),
        }
    }
}

/// A Group inside a Datapool (decisions §22): a stored selection — its
/// members in selection order, each with its own grid spot. Like a
/// `SELECTIONDATA`: order and grid are independent, no values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub meta: NodeMeta,
    /// Members in selection order (fixture GUID links + grid spots).
    #[serde(default)]
    pub members: Vec<GroupMember>,
    /// The group master's mode (its live level is in `ShowData.Playbacks`).
    #[serde(default)]
    pub master: GroupMasterMode,
    /// Recall keeps the stored grid gaps and offset (PreserveGridPositions).
    #[serde(default = "yes")]
    pub preserve_grid: bool,
    /// Where the user's grid cursor goes after the group is selected.
    #[serde(default)]
    pub move_grid_cursor: GridCursorMove,
    /// Reserved for the group's Pawtern (decisions §3); always None today.
    #[serde(default)]
    pub pawtern: Option<Uuid>,
}

fn yes() -> bool {
    true
}

/// One group member: the fixture (GUID link) and its grid spot.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GroupMember {
    pub fixture: Uuid,
    #[serde(flatten)]
    pub cell: GridCell,
}

/// A spot in the selection grid (grid x/y/z), with the member's
/// rotation and mirroring (decisions §28 grid layouts honour them).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub z: i32,
    /// Degrees, clockwise.
    #[serde(default)]
    pub rotation: f32,
    #[serde(default)]
    pub mirror_x: bool,
    #[serde(default)]
    pub mirror_y: bool,
}

/// `GroupMasterMode`: how the group's master acts on its members'
/// intensity (applied after the merge, after the grand master).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupMasterMode {
    /// No master.
    #[default]
    None,
    /// Limits (HTP between positive masters).
    Positive,
    /// Limits (lowest wins; beats Positive where they overlap).
    Negative,
    /// Multiplies.
    Scaling,
    /// Adds its level, HTP with the output.
    Additive,
}

impl GroupMasterMode {
    pub const NAMES: &'static [&'static str] = &["None", "Positive", "Negative", "Scaling", "Additive"];

    pub fn parse(word: &str) -> Option<Self> {
        Some(match word.to_ascii_lowercase().as_str() {
            "none" => Self::None,
            "positive" => Self::Positive,
            "negative" => Self::Negative,
            "scaling" => Self::Scaling,
            "additive" => Self::Additive,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        Self::NAMES[self as usize]
    }
}

/// Where a selection moves the grid cursor (Move Grid Cursor).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridCursorMove {
    /// Use the user's own setting (`<Linked>`).
    #[default]
    Linked,
    /// The cursor stays.
    None,
    /// The cursor moves to the next free X.
    AppendX,
    /// The cursor moves to X 0 of the next free line.
    NewLine,
}

impl GridCursorMove {
    pub const NAMES: &'static [&'static str] = &["Linked", "None", "AppendX", "NewLine"];

    pub fn parse(word: &str) -> Option<Self> {
        Some(match word.to_ascii_lowercase().as_str() {
            "linked" => Self::Linked,
            "none" => Self::None,
            "appendx" => Self::AppendX,
            "newline" => Self::NewLine,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        Self::NAMES[self as usize]
    }
}

impl TreeNode for Group {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Group"
    }
}

impl Group {
    pub fn new(name: Option<String>) -> Self {
        Self {
            meta: NodeMeta::new(name),
            members: Vec::new(),
            master: GroupMasterMode::None,
            preserve_grid: true,
            move_grid_cursor: GridCursorMove::Linked,
            pawtern: None,
        }
    }

    /// The members' fixture GUIDs, in selection order.
    pub fn fixtures(&self) -> Vec<Uuid> {
        self.members.iter().map(|m| m.fixture).collect()
    }

    pub fn contains(&self, fixture: Uuid) -> bool {
        self.members.iter().any(|m| m.fixture == fixture)
    }

    /// Members in a row from x 0 (a selection without grid spots).
    pub fn set_row(&mut self, fixtures: &[Uuid]) {
        self.members = fixtures
            .iter()
            .enumerate()
            .map(|(i, &fixture)| GroupMember {
                fixture,
                cell: GridCell {
                    x: i as i32,
                    ..GridCell::default()
                },
            })
            .collect();
    }

    /// Add members not already in the group. `append_x`: their grid spots
    /// are moved after the group's last X (GridMergeMode AppendX), else
    /// they keep their own. Returns how many were added.
    pub fn merge_members(&mut self, new: &[GroupMember], append_x: bool) -> usize {
        let fresh: Vec<GroupMember> = new
            .iter()
            .filter(|m| !self.contains(m.fixture))
            .copied()
            .collect();
        let shift = match (append_x, fresh.iter().map(|m| m.cell.x).min()) {
            (true, Some(min)) => {
                self.members.iter().map(|m| m.cell.x + 1).max().unwrap_or(0) - min
            }
            _ => 0,
        };
        for mut m in fresh.iter().copied() {
            m.cell.x += shift;
            self.members.push(m);
        }
        fresh.len()
    }

    /// Remove these fixtures. Returns how many were removed.
    pub fn remove_fixtures(&mut self, fixtures: &[Uuid]) -> usize {
        let before = self.members.len();
        self.members.retain(|m| !fixtures.contains(&m.fixture));
        before - self.members.len()
    }
}

/// How a preset applies to fixtures.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PresetMode {
    /// Values stored per-fixture.
    #[default]
    Selective,
    /// One set of values applied to all fixtures in the pool.
    Global,
    /// One set of values applied universally regardless of fixture selection.
    Universal,
}

/// A single attribute value entry inside a Preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetEntry {
    /// `None` = Universal or Global; `Some` = Selective (per-fixture).
    #[serde(default)]
    pub fixture_id: Option<Uuid>,
    /// GDTF canonical attribute name, e.g. "Dimmer", "Pan".
    pub attribute: String,
    /// The one value type (decisions §27); older shows' `Percent`/`Physical`/
    /// `Release` read into it.
    pub value: lumacore_base::Value,
}

/// A stored preset of the DataPool's preset pool (decisions 0288). Its tags are its NodeMeta's.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub meta: NodeMeta,
    #[serde(default)]
    pub mode: PresetMode,
    #[serde(default)]
    pub values: Vec<PresetEntry>,
}

impl TreeNode for Preset {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Preset"
    }
}

impl Preset {
    /// The fixtures it holds values for (a Universal value names none).
    pub fn fixtures(&self) -> impl Iterator<Item = Uuid> + '_ {
        let mut seen = std::collections::BTreeSet::new();
        self.values.iter().filter_map(|e| e.fixture_id).filter(move |g| seen.insert(*g))
    }

    /// The attributes it holds values for, in first-stored order, each once.
    pub fn attributes(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for e in &self.values {
            if !out.contains(&e.attribute.as_str()) {
                out.push(&e.attribute);
            }
        }
        out
    }
}
