//! Root, ShowData, StationSettings — the top-level tree containers.
//!
//! Root is the single owner of the entire object tree.
//! ShowData is the show file (serialized to .luma).
//! StationSettings is machine-specific config (persists across shows).

use lumacore_fixtures::attribute_def::{ActivationGroup, AttributeDefinition, FeatureGroup};
use crate::datapool::{Datapool, DatapoolPriorityOrder};
use crate::events::{TreeEvent, TreeObserver};
use crate::node::NodeMeta;
use lumacore_fixtures::fixture::{Fixture, Slot};
use lumacore_fixtures::fixture_type::FixtureType;
use crate::id_pool::IdPool;
use crate::keybindings::{KeyBindingMap, debug_keybindings, vanilla_keybindings};
use lumacore_protocols::Line;
use crate::programmer::UserProfile;
use crate::runtime::{RuntimeState, StationPriority};
use lumacore_fixtures::universe::Universe;
pub use lumacore_base::variables::{VariableStore, VariableValue};
pub use lumacore_effects::{
    EffectEngineConfig, EffectEnginesData, EffectGenerator, EffectGraphNode, EffectParameter,
};
pub use lumacore_playback::{
    CueData, CueObject, CuePart, CueTrigger, CueValue, CurvePreset, FadePath, Effect, ExecutorObject, GroupTiming, NoPresets, PresetValues,
    MibPreference, ParkData, ParkedChannel, ParkedValue, PlaybackData,
    CueZeroMode, CUE_ZERO, Echo, EchoPlaybacks, GrandMasters, LivePlaybacks, MasterLevel, Masters, PlaybackPriority, PlaybackStates, RestartMode, now_ms, SequenceAssignment, SequenceObject,
    SequenceSettings, SequenceState, TimingPreset, Tracked, cue_key, cue_label,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use uuid::Uuid;

/// The show format this build writes and reads (decisions §20). Only
/// forward: no migrations; bump it when the file format changes.
pub const FORMAT_VERSION: u32 = 36;

/// The show's LumaSession memory (`ShowData.LumaSession`, the
/// stations that output).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionData {
    /// Rises on every change to the live patch (Keep Patch, Live Patch Set,
    /// Import, undo of those). Remembered at a split, so a rejoin can tell
    /// whether both sides changed the patch (6d blocks Merge All then).
    #[serde(default)]
    pub patch_version: u64,
    /// Every station that ever joined, by station id: re-invited when it
    /// comes back with Invite on; removed with `Delete Station`.
    #[serde(default)]
    pub stations: BTreeMap<String, KnownStation>,
}

/// A station the session knows (`ShowData.LumaSession.Stations`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct KnownStation {
    pub meta: NodeMeta,
    pub station_id: String,
    /// Last address it was seen on, and its stream port.
    pub ip: String,
    #[serde(default)]
    pub stream_port: u16,
    #[serde(default)]
    pub priority: crate::StationPriority,
    /// When it first joined (unix ms).
    pub joined_unix_ms: u64,
}

/// Show-level configuration stored in ShowData.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowSettings {
    pub show_name: String,
    pub description: String,
    /// LumaCore version that created this show.
    pub created_with_version: String,
    /// Show format of the file (decisions §20): set on save. A file with a
    /// higher version than this build's [`FORMAT_VERSION`] is refused.
    #[serde(default)]
    pub format_version: u32,
    /// Commit of the build that last saved the show (`-dirty` = built with
    /// uncommitted changes), to trace issues.
    #[serde(default)]
    pub saved_with_commit: String,
    /// Autosave interval in minutes, 0 = off (AutosaveInterval).
    #[serde(default = "default_autosave_minutes")]
    pub autosave_minutes: u32,
    /// The show's identity (decisions §21 "6d"): made on New Show, kept on
    /// Save, new on Save As. Only the same show can Merge on rejoin.
    pub show_id: uuid::Uuid,
    /// What the merge prompt picks when nobody answers (Data Merge
    /// Default Mode). Admin.
    #[serde(default)]
    pub merge_default: MergeChoice,
    /// Seconds the merge prompt waits (Data Merge Default Timeout):
    /// 0 = no prompt, the default applies at once; None = wait for the
    /// operator. Admin.
    #[serde(default = "default_merge_timeout")]
    pub merge_timeout: Option<u32>,
    /// The attribute template the show's grouping started from (decisions
    /// §30 "12f-a": "LumaCore FX" or "GDTF standard"); `Set ShowData
    /// AttributeTemplate "x"` regroups (asks).
    #[serde(default)]
    pub attribute_template: String,
}

/// The four answers when a station rejoins with a diverged show
/// (`DataNegotiationMode`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeChoice {
    /// Both sides' changes in one show (by GUID; conflicts: the master's).
    #[default]
    Merge,
    /// Take the session's show (mine autosaved).
    UseMaster,
    /// My show wins (the others autosave theirs).
    UseMine,
    /// Stay alone: IdleMaster, no output.
    Cancel,
}

impl MergeChoice {
    pub const ALL: [MergeChoice; 4] = [
        MergeChoice::Merge,
        MergeChoice::UseMaster,
        MergeChoice::UseMine,
        MergeChoice::Cancel,
    ];

    /// The word in prompts, sysmon and `Set` (the usual console wording).
    pub fn word(self) -> &'static str {
        match self {
            MergeChoice::Merge => "Merge",
            MergeChoice::UseMaster => "Use Master",
            MergeChoice::UseMine => "Use Mine",
            MergeChoice::Cancel => "Cancel",
        }
    }

    /// Parse `Merge`, `UseMaster` / `Use Master`, `UseMine`, `Cancel`.
    pub fn parse(s: &str) -> Option<Self> {
        let k: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        Self::ALL
            .into_iter()
            .find(|c| c.word().replace(' ', "").eq_ignore_ascii_case(&k))
    }
}

fn default_merge_timeout() -> Option<u32> {
    Some(60)
}

fn default_autosave_minutes() -> u32 {
    15
}

impl Default for ShowSettings {
    fn default() -> Self {
        Self {
            show_name: String::new(),
            description: String::new(),
            created_with_version: String::new(),
            format_version: FORMAT_VERSION,
            saved_with_commit: String::new(),
            autosave_minutes: default_autosave_minutes(),
            show_id: uuid::Uuid::new_v4(),
            merge_default: MergeChoice::default(),
            merge_timeout: default_merge_timeout(),
            attribute_template: String::new(),
        }
    }
}

/// `Root.ShowData.Devices.Dmx.Patch`: fixture types, fixtures and universes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatchData {
    /// All imported fixture type profiles.
    #[serde(default)]
    /// Shared per type (decisions §31 "Arc parts"): GDTF trees are the biggest part of a show
    /// and rarely change; a published copy shares them. Change one with `fixture_type_mut`.
    pub fixture_types: BTreeMap<u32, std::sync::Arc<FixtureType>>,
    /// All patched fixture instances, by slot (decisions §40 5): the key is given at insert and
    /// means nothing — a line's `Fixture 849` is the fixture whose `mvr.fid` is 849
    /// (`fixture_by_fid`), `Fixture Media 7003` the one with that second id (`fixture_by_id`).
    #[serde(default)]
    pub fixtures: BTreeMap<Slot, Fixture>,
    /// The fixture numbers in use (`mvr.fid`), for the next free one. Skipped on serialization —
    /// rebuilt on load.
    #[serde(skip, default = "IdPool::new")]
    pub fixture_id_pool: IdPool,
    /// Number → slot, filled as numbers are looked up and checked on every hit (a fixture may
    /// have moved or been renumbered behind it), so `Fixture n` on a rig of a thousand is a
    /// map lookup, not a scan (decisions §40 "40f built"). Never saved, never trusted blindly.
    #[serde(skip)]
    pub fid_index: FidIndex,
    /// All universes.
    #[serde(default)]
    pub universes: BTreeMap<u32, Universe>,
    /// ID pool for universe allocation. Skipped on serialization — rebuilt on load.
    #[serde(skip, default = "IdPool::new")]
    pub universe_id_pool: IdPool,
}

impl PatchData {
    /// Fixture type `n` to change: copied first when a published show still shares it
    /// (`Arc::make_mut`, decisions §31 "Arc parts").
    pub fn fixture_type_mut(&mut self, n: u32) -> Option<&mut FixtureType> {
        self.fixture_types.get_mut(&n).map(std::sync::Arc::make_mut)
    }

    // ----- The one door to a fixture (decisions §40 5): by number, second id, GUID or slot -----

    /// `Fixture n`: the fixture whose number is `n`.
    pub fn fixture_by_fid(&self, n: impl std::borrow::Borrow<u32>) -> Option<(Slot, &Fixture)> {
        let slot = self.slot_of_fid(*n.borrow())?;
        self.fixtures.get(&slot).map(|f| (slot, f))
    }

    pub fn fixture_by_fid_mut(&mut self, n: impl std::borrow::Borrow<u32>) -> Option<(Slot, &mut Fixture)> {
        let slot = self.slot_of_fid(*n.borrow())?;
        self.fixtures.get_mut(&slot).map(|f| (slot, f))
    }

    /// The slot of `Fixture n`: the index's answer when it still holds (the slot's fixture is
    /// still numbered `n`), else a scan that teaches the index.
    fn slot_of_fid(&self, n: u32) -> Option<Slot> {
        if let Some(slot) = self.fid_index.get(n)
            && self.fixtures.get(&slot).is_some_and(|f| f.mvr.fid == Some(n))
        {
            return Some(slot);
        }
        let found = self.fixtures.iter().find(|(_, f)| f.mvr.fid == Some(n)).map(|(s, _)| *s);
        match found {
            Some(slot) => self.fid_index.set(n, slot),
            None => self.fid_index.forget(n),
        }
        found
    }

    /// The index built whole (a load, `after_load`): every lookup a hit from the start.
    pub fn rebuild_fid_index(&mut self) {
        self.fid_index.rebuild(self.fixtures.iter().filter_map(|(s, f)| f.mvr.fid.map(|n| (n, *s))));
    }

    /// `Fixture Media 7003`: the fixture whose second id is (`ty`, `n`).
    pub fn fixture_by_id(&self, ty: lumacore_fixtures::IdType, n: u32) -> Option<(Slot, &Fixture)> {
        self.fixtures.iter().find(|(_, f)| f.mvr.id_type == ty && f.mvr.cid == Some(n)).map(|(s, f)| (*s, f))
    }

    pub fn fixture_by_guid(&self, guid: Uuid) -> Option<(Slot, &Fixture)> {
        self.fixtures.iter().find(|(_, f)| f.meta.identity.guid == guid).map(|(s, f)| (*s, f))
    }

    pub fn fixture_by_guid_mut(&mut self, guid: Uuid) -> Option<(Slot, &mut Fixture)> {
        self.fixtures.iter_mut().find(|(_, f)| f.meta.identity.guid == guid).map(|(s, f)| (*s, f))
    }

    /// The slot a GUID sits in.
    pub fn slot_of(&self, guid: Uuid) -> Option<Slot> {
        self.fixture_by_guid(guid).map(|(s, _)| s)
    }

    /// The number of the fixture at `slot`.
    pub fn fid_at(&self, slot: Slot) -> Option<u32> {
        self.fixtures.get(&slot).and_then(|f| f.mvr.fid)
    }

    /// The next free slot: one past the highest (slots are never reused in a session, so a
    /// slot a line raced to still means what it meant).
    pub fn next_slot(&self) -> Slot {
        Slot(self.fixtures.keys().last().map_or(0, |s| s.0) + 1)
    }

    /// The next order under `parent` (decisions §40 6): ten past the last child's.
    pub fn next_order(&self, parent: Option<Uuid>) -> u32 {
        self.fixtures.values().filter(|f| f.mvr.parent == parent).map(|f| f.mvr.order).max().map_or(10, |o| o + 10)
    }

    /// Puts a fixture in: a new slot, the next order under its parent when it has none, its
    /// number reserved. The slot it got.
    pub fn insert_fixture(&mut self, mut fixture: Fixture) -> Slot {
        if fixture.mvr.order == 0 {
            fixture.mvr.order = self.next_order(fixture.mvr.parent);
        }
        if let Some(n) = fixture.mvr.fid {
            self.fixture_id_pool.reserve(n);
        }
        let slot = self.next_slot();
        self.fixtures.insert(slot, fixture);
        slot
    }

    /// Takes a fixture out, its number released.
    pub fn remove_fixture(&mut self, slot: Slot) -> Option<Fixture> {
        let f = self.fixtures.remove(&slot)?;
        if let Some(n) = f.mvr.fid {
            self.fixture_id_pool.release(n);
        }
        Some(f)
    }

    /// Every numbered fixture, by number: (number, slot).
    pub fn fids(&self) -> Vec<(u32, Slot)> {
        let mut out: Vec<(u32, Slot)> = self.fixtures.iter().filter_map(|(s, f)| f.mvr.fid.map(|n| (n, *s))).collect();
        out.sort_unstable();
        out
    }

    /// The numbered fixtures as a map by number (a view built per call; the old selection
    /// resolver and the walk read by number and range).
    pub fn by_number(&self) -> BTreeMap<u32, &Fixture> {
        self.fixtures.values().filter_map(|f| f.mvr.fid.map(|n| (n, f))).collect()
    }

    /// Whether number `n` is taken.
    pub fn has_fid(&self, n: impl std::borrow::Borrow<u32>) -> bool {
        self.fixture_by_fid(n).is_some()
    }

    /// `Fixture n`, the fixture alone.
    pub fn fixture(&self, n: impl std::borrow::Borrow<u32>) -> Option<&Fixture> {
        self.fixture_by_fid(n).map(|(_, f)| f)
    }

    pub fn fixture_mut(&mut self, n: impl std::borrow::Borrow<u32>) -> Option<&mut Fixture> {
        self.fixture_by_fid_mut(n).map(|(_, f)| f)
    }

    /// Takes `Fixture n` out (its number released); what it was.
    pub fn remove_fid(&mut self, n: impl std::borrow::Borrow<u32>) -> Option<Fixture> {
        let slot = self.fixture_by_fid(*n.borrow())?.0;
        self.remove_fixture(slot)
    }

    /// Puts a fixture in as `Fixture n` (decisions §40 5: the slot is the pool's).
    pub fn insert_fid(&mut self, n: impl std::borrow::Borrow<u32>, mut fixture: Fixture) -> Slot {
        fixture.mvr.fid = Some(*n.borrow());
        self.insert_fixture(fixture)
    }
}

/// `PatchData`'s number → slot index (above): a lock around a map, so a read (`&self`) can
/// teach it; cloned with the data, empty after a load until the first lookups or `rebuild`.
#[derive(Debug, Default)]
pub struct FidIndex(std::sync::Mutex<BTreeMap<u32, Slot>>);

impl Clone for FidIndex {
    fn clone(&self) -> Self {
        FidIndex(std::sync::Mutex::new(self.0.lock().map(|m| m.clone()).unwrap_or_default()))
    }
}

impl FidIndex {
    fn get(&self, n: u32) -> Option<Slot> {
        self.0.lock().ok()?.get(&n).copied()
    }

    fn set(&self, n: u32, slot: Slot) {
        if let Ok(mut m) = self.0.lock() {
            m.insert(n, slot);
        }
    }

    fn forget(&self, n: u32) {
        if let Ok(mut m) = self.0.lock() {
            m.remove(&n);
        }
    }

    fn rebuild(&self, entries: impl Iterator<Item = (u32, Slot)>) {
        if let Ok(mut m) = self.0.lock() {
            *m = entries.collect();
        }
    }
}

/// `Root.ShowData.Devices` (decisions §34 "34g4 patch hardened" 2, 7): a node per device kind —
/// `Dmx` today; later kinds on other base protocols get their own — and the `Metadata` they share.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevicesData {
    #[serde(default)]
    pub dmx: DmxDevices,
    #[serde(default)]
    pub metadata: DevicesMetadata,
}

/// `Root.ShowData.Devices.Dmx`: the DMX devices.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DmxDevices {
    #[serde(default)]
    pub patch: PatchData,
}

/// `Root.ShowData.Devices.Metadata`: what the device kinds and 3DSpatial share.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DevicesMetadata {
    /// `Metadata.MVR`: what MVR files bring besides fixtures, by UUID.
    #[serde(default)]
    pub mvr: lumacore_fixtures::MvrMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolEndpointConfig {
    pub meta: NodeMeta,
    pub protocol: String,
}

/// `ShowData.Protocols` (decisions §11, §34 "34f7c hardened" 5): the networks the show uses and
/// each protocol's node by its row's name, in the group of its kind —
/// `Protocols.DMXProtocols.Sacn`, `Protocols.DataProtocols.Osc`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolsData {
    #[serde(default)]
    pub networks: BTreeMap<u32, lumacore_protocols::Network>,
    /// The protocols whose lines carry universes, by row name.
    #[serde(default)]
    pub dmx_protocols: BTreeMap<String, ProtocolNode>,
    /// The protocols whose messages become command lines, by row name.
    #[serde(default)]
    pub data_protocols: BTreeMap<String, ProtocolNode>,
    #[serde(default)]
    pub bidirectional: BTreeMap<u32, ProtocolEndpointConfig>,
}

/// One protocol in the show: what the whole protocol has (its declared protocol settings) and
/// its lines by number (`Sacn 1`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolNode {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub settings: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub lines: BTreeMap<u32, Line>,
    /// Rebuilt on load, never saved.
    #[serde(skip, default = "IdPool::new")]
    pub id_pool: IdPool,
}

impl ProtocolsData {
    /// The group a protocol's node sits in (its row's kind; a name this build lacks is looked for
    /// in both, so a load can refuse it).
    fn group_of(&self, protocol: &str) -> &BTreeMap<String, ProtocolNode> {
        match lumacore_protocols::protocol::find(protocol).map(|p| p.kind) {
            Some(lumacore_protocols::Kind::Data) => &self.data_protocols,
            Some(lumacore_protocols::Kind::Dmx) => &self.dmx_protocols,
            None if self.data_protocols.keys().any(|k| k.eq_ignore_ascii_case(protocol)) => &self.data_protocols,
            None => &self.dmx_protocols,
        }
    }

    /// The node of protocol `protocol` (any case), if the show has one.
    pub fn node(&self, protocol: &str) -> Option<&ProtocolNode> {
        self.group_of(protocol).iter().find(|(k, _)| k.eq_ignore_ascii_case(protocol)).map(|(_, n)| n)
    }

    /// The node of a protocol of this build, made when the show has none yet (in its kind's group,
    /// under the row's own name). None for a name this build doesn't have.
    pub fn node_mut(&mut self, protocol: &str) -> Option<&mut ProtocolNode> {
        let row = lumacore_protocols::protocol::find(protocol)?;
        let group = match row.kind {
            lumacore_protocols::Kind::Dmx => &mut self.dmx_protocols,
            lumacore_protocols::Kind::Data => &mut self.data_protocols,
        };
        Some(group.entry(row.name.to_string()).or_default())
    }

    /// A protocol's lines (none when the show has no node for it).
    pub fn lines(&self, protocol: &str) -> Option<&BTreeMap<u32, Line>> {
        self.node(protocol).map(|n| &n.lines)
    }

    /// A protocol's lines to change (its node made if missing).
    pub fn lines_mut(&mut self, protocol: &str) -> Option<&mut BTreeMap<u32, Line>> {
        self.node_mut(protocol).map(|n| &mut n.lines)
    }

    /// Line `no` of `protocol`.
    pub fn line(&self, protocol: &str, no: u32) -> Option<&Line> {
        self.lines(protocol)?.get(&no)
    }

    /// Every line of every protocol: (protocol, number, line), DMX protocols first.
    pub fn all_lines(&self) -> impl Iterator<Item = (&str, u32, &Line)> {
        self.dmx_protocols
            .iter()
            .chain(self.data_protocols.iter())
            .flat_map(|(name, node)| node.lines.iter().map(move |(&no, l)| (name.as_str(), no, l)))
    }

    /// Every protocol node the show has: (name, node), DMX protocols first.
    pub fn nodes(&self) -> impl Iterator<Item = (&str, &ProtocolNode)> {
        self.dmx_protocols.iter().chain(self.data_protocols.iter()).map(|(k, n)| (k.as_str(), n))
    }

    /// The name of the network with this GUID ("?" when it is missing).
    pub fn network_name(&self, guid: Uuid) -> &str {
        self.networks
            .values()
            .find(|n| n.meta.identity.guid == guid)
            .and_then(|n| n.meta.identity.name.as_deref())
            .unwrap_or("?")
    }

    /// The network called `name`: (number, GUID).
    pub fn network_named(&self, name: &str) -> Option<(u32, Uuid)> {
        self.networks
            .iter()
            .find(|(_, n)| {
                n.meta
                    .name()
                    .is_some_and(|n| n.eq_ignore_ascii_case(name))
            })
            .map(|(&id, n)| (id, n.meta.identity.guid))
    }

    /// The line with this GUID: (protocol, number, line).
    pub fn line_by_guid(&self, guid: Uuid) -> Option<(&str, u32, &Line)> {
        self.all_lines().find(|(_, _, l)| l.meta.identity.guid == guid)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpatialSceneObject {
    pub meta: NodeMeta,
    #[serde(default)]
    pub resource_ref: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpatialPatchObject {
    pub meta: NodeMeta,
    /// The fixture (GUID link).
    pub fixture: Uuid,
    #[serde(default)]
    pub object_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Spatial3DData {
    #[serde(default)]
    pub world: BTreeMap<u32, SpatialSceneObject>,
    #[serde(default)]
    pub stages: BTreeMap<u32, SpatialSceneObject>,
    #[serde(default)]
    pub layers: BTreeMap<u32, SpatialSceneObject>,
    #[serde(default)]
    pub cameras: BTreeMap<u32, SpatialSceneObject>,
    #[serde(default)]
    pub patch: BTreeMap<u32, SpatialPatchObject>,
    #[serde(default)]
    pub tracking: BTreeMap<u32, SpatialSceneObject>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UIWorkspaceState {
    pub name: String,
}

/// A live UI engine (e.g. the HTTP/WebSocket engine serving the web UI),
/// published at `Root.UIEngines.<no>`. Runtime-only, never saved.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UIEngineState {
    pub name: String,
    /// Engine kind, e.g. "http".
    pub kind: String,
    /// "Running" / "Stopped".
    pub status: String,
    /// Bind address, e.g. "0.0.0.0:47656".
    pub address: String,
    pub port: Option<u16>,
    /// Where the WebUI comes from: the `--web-root` folder, or "built-in" (decisions §36 2).
    pub web_root: Option<String>,
    pub workspace: UIWorkspaceState,
    pub sessions: BTreeMap<String, String>,
    pub viewports: BTreeMap<String, String>,
    pub render_state: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct UIEnginesRoot {
    pub engines: BTreeMap<u32, UIEngineState>,
}

impl UIEnginesRoot {
    /// Publish an engine under the next free number; returns that number.
    pub fn publish(&mut self, engine: UIEngineState) -> u32 {
        let no = self.engines.keys().next_back().map_or(1, |n| n + 1);
        self.engines.insert(no, engine);
        no
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourceEntry {
    pub name: String,
    pub resource_type: String,
}

#[derive(Debug, Clone)]
pub struct ResourceRoot {
    pub entries: BTreeMap<String, ResourceEntry>,
    /// Built-in window types and their settings schemas (`Root.Resources.WindowTypes`).
    pub window_types: BTreeMap<u32, crate::window_types::WindowType>,
    /// Built-in UI keys, locked (`Root.Resources.UIKeys`).
    pub ui_keys: BTreeMap<u32, crate::ui_keys::UiKey>,
}

impl Default for ResourceRoot {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            window_types: lumacore_registry::get().window_types().clone(),
            ui_keys: crate::ui_keys::builtin_ui_keys(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildInfo {
    pub version: String,
    pub profile: String,
    pub target_os: String,
    pub target_arch: String,
    pub git_commit: String,
    pub build_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct DiagnosticRoot {
    pub build_info: BuildInfo,
    pub message_center: Vec<String>,
    pub status_center: Vec<String>,
}

/// The complete show data — everything in the .luma file.
///
/// BTreeMap is used for all collections so iteration order is deterministic
/// (sorted by integer ID), which simplifies serialization and diffing.
///
/// IdPool instances track per-type ID allocation with safe reuse after deletion.
/// They are serialized alongside their collections so IDs are stable across
/// save/load cycles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShowData {
    /// Root.ShowData.Devices — authored and resolved device/control structure.
    #[serde(default)]
    pub devices: DevicesData,
    /// Named datapools (e.g., "songname", "busk1") containing Groups, PresetPools.
    #[serde(default)]
    pub datapools: BTreeMap<u32, Datapool>,
    /// Pool-level stacking rules. Higher-priority pools win over lower ones.
    #[serde(default)]
    pub datapool_priority_orders: BTreeMap<u32, DatapoolPriorityOrder>,
    /// Show-level settings (name, description, etc.).
    pub show_settings: ShowSettings,
    /// Spatial world data for stages, layers, cameras, and auto-generated patch objects.
    #[serde(default)]
    pub spatial_3d: Spatial3DData,
    /// Playback-oriented show objects.
    #[serde(default)]
    pub playback: PlaybackData,
    /// `ShowData.Playbacks`: what is running (saved, resumed on load,
    /// replicated, never undone — decisions §13).
    #[serde(default)]
    pub playbacks: PlaybackStates,
    /// `ShowData.LumaSession`: what the show remembers of its LumaSession —
    /// the patch version and every station that ever joined (decisions §21
    /// "6c hardening"). Replicated, never undone.
    #[serde(default)]
    pub session: SessionData,
    /// User profiles: how users work (views, screen configs, keys, settings).
    /// Shared per profile (decisions §31 "Arc parts"): a profile carries images, screens and
    /// views; a published copy shares every one nobody changed. Change one with `profile_mut`.
    pub user_profiles: BTreeMap<u32, std::sync::Arc<UserProfile>>,
    /// Users (ShowData.Users): login, role, profile, and their own
    /// programmer and selection.
    #[serde(default)]
    pub users: BTreeMap<u32, crate::programmer::User>,
    /// Root.ShowData.Protocols — protocol assignments and outputs.
    #[serde(default)]
    pub protocols: ProtocolsData,
    /// Saved effect engine configurations.
    #[serde(default)]
    pub effect_engines: EffectEnginesData,
    /// Show-level variables that travel with the show file.
    #[serde(default)]
    pub variables: VariableStore,
    /// `ShowData.Images`: the show's own pictures, for show data (pool
    /// objects, layouts). Icons are never in the show: the standard set ships
    /// with LumaCore and a profile carries its own (decisions §30 "12a2").
    #[serde(default)]
    pub images: BTreeMap<u32, crate::media::MediaFile>,
    /// `ShowData.Assets`: asset roots and the large files on them (§8).
    #[serde(default)]
    pub assets: crate::media::AssetsData,
    /// Global attribute definitions, seeded from the GDTF standard attribute table.
    ///
    /// Keyed by canonical GDTF attribute name (e.g. "Dimmer", "Pan"). The DMX
    /// render path reads `ChannelFunction` for math; this table drives display,
    /// encoder grouping, and preset routing.
    #[serde(default)]
    pub attribute_defs: BTreeMap<String, AttributeDefinition>,
    /// Feature groups that determine default encoder page grouping.
    #[serde(default)]
    pub feature_groups: BTreeMap<u32, FeatureGroup>,
    /// Activation groups that link mutually-exclusive attributes.
    #[serde(default)]
    pub activation_groups: BTreeMap<u32, ActivationGroup>,
    /// The resolver's rules, in order (decisions §30 "12f-a"):
    /// `Root.ShowData.AttributeRules`.
    #[serde(default)]
    pub attribute_rules: BTreeMap<u32, lumacore_fixtures::attribute_def::AttributeRule>,
    /// `ShowData.Stations`: the station table (decisions §30 "12d part 4",
    /// `OutputStations`): each station's rig wiring — its NIC for
    /// each of the show's networks and what each NIC serves — one row per
    /// station, pre-configurable while a station is offline. Replicated and
    /// undone like any show data; each station applies only its own row
    /// (and keeps a last-applied copy in its station settings). Identity
    /// (name, location, priority, …) stays on the station.
    #[serde(default)]
    pub stations: BTreeMap<u32, StationWiring>,
    /// `ShowData.Tags`: the tag tree (decisions §12 "Tags hardened"); objects link their tags
    /// by GUID in their `NodeMeta.tags`.
    #[serde(default)]
    pub tags: lumacore_tags::TagTree,
}

impl ShowData {
    /// The user named `name` (case-insensitive).
    pub fn user(&self, name: &str) -> Option<&crate::programmer::User> {
        self.users
            .values()
            .find(|u| u.login.eq_ignore_ascii_case(name))
    }

    pub fn user_mut(&mut self, name: &str) -> Option<&mut crate::programmer::User> {
        self.users
            .values_mut()
            .find(|u| u.login.eq_ignore_ascii_case(name))
    }

    /// The profile a user works in (only through the user: a profile has no
    /// user of its own, decisions §30 "12c2").
    pub fn profile_id_for_user(&self, name: &str) -> Option<u32> {
        self.user(name).and_then(|u| self.profile_no(u.profile))
    }

    pub fn profile_for_user(&self, name: &str) -> Option<&UserProfile> {
        self.profile_id_for_user(name)
            .and_then(|id| self.user_profiles.get(&id))
            .map(|p| p.as_ref())
    }

    pub fn profile_for_user_mut(&mut self, name: &str) -> Option<&mut UserProfile> {
        self.profile_id_for_user(name).and_then(|id| self.profile_mut(id))
    }

    /// Profile `n` to change: copied first when a published show still shares it
    /// (`Arc::make_mut`, decisions §31 "Arc parts").
    /// Every sequence of every DataPool: (datapool, number, sequence).
    pub fn sequences(&self) -> impl Iterator<Item = (u32, u32, &std::sync::Arc<SequenceObject>)> {
        self.datapools.iter().flat_map(|(&dp, d)| d.sequences.iter().map(move |(&n, s)| (dp, n, s)))
    }

    /// The sequence with this GUID, in whichever DataPool it is: (datapool, number, sequence).
    pub fn sequence_by_guid(&self, guid: Uuid) -> Option<(u32, u32, &std::sync::Arc<SequenceObject>)> {
        self.sequences().find(|(_, _, s)| s.meta.identity.guid == guid)
    }

    /// Sequence `n` of DataPool `dp` to change: copied first when a published show still shares
    /// it (`Arc::make_mut`, decisions §31 "Arc parts").
    pub fn sequence_mut(&mut self, dp: u32, n: u32) -> Option<&mut SequenceObject> {
        self.datapools.get_mut(&dp)?.sequences.get_mut(&n).map(std::sync::Arc::make_mut)
    }

    pub fn profile_mut(&mut self, n: u32) -> Option<&mut UserProfile> {
        self.user_profiles.get_mut(&n).map(std::sync::Arc::make_mut)
    }

    /// The user's rights level — only ever from the User (decisions §30
    /// "12c2": a deleted user has none, whatever profiles are named like them).
    pub fn rights_of(&self, name: &str) -> Option<crate::programmer::UserRights> {
        self.user(name).map(|u| u.rights)
    }

    /// The locked template profile "Default": (number, profile).
    pub fn default_profile(&self) -> Option<(u32, &UserProfile)> {
        self.user_profiles
            .iter()
            .find(|(_, p)| p.locked)
            .map(|(&n, p)| (n, p.as_ref()))
    }

    /// Whether profile `no` is the locked template.
    pub fn is_default_profile(&self, no: u32) -> bool {
        self.user_profiles.get(&no).is_some_and(|p| p.locked)
    }

    /// Every show has the template profile "Default" (number 1 when free).
    pub fn ensure_default_profile(&mut self) -> bool {
        if self.default_profile().is_some() {
            return false;
        }
        let id = if self.user_profiles.contains_key(&1) {
            self.user_profiles.keys().next_back().map_or(1, |n| n + 1)
        } else {
            1
        };
        self.user_profiles.insert(id, std::sync::Arc::new(UserProfile::new_default()));
        true
    }

    /// Make a freshly deserialized (or partly replaced) show usable: a show
    /// always has an Admin, and every ID pool is rebuilt from the objects
    /// that exist. Every load path calls this — file, join, sync, undo restore,
    /// replication — so a pool never hands out an ID that is already taken.
    /// No migrations: shows load as written (decisions §20).
    pub fn after_load(&mut self) {
        self.ensure_default_profile();
        self.ensure_admin();
        self.rebuild_id_pools();
        self.rebuild_occupancy();
        self.observe_stamps();
    }

    /// Drop saved windows (screen configs and views of every profile) whose
    /// window type this build doesn't know (R1-57, Q32). One line per
    /// dropped window, for the System Monitor.
    pub fn drop_unknown_windows(&mut self) -> Vec<String> {
        self.user_profiles.values_mut().map(std::sync::Arc::make_mut).flat_map(|p| p.drop_unknown_windows()).collect()
    }

    /// Raise the LTP stamp floor to every stamp in the show (R1-01): a show
    /// loaded, synced or merged from a station whose clock was ahead must
    /// not win over the touches made here afterwards.
    fn observe_stamps(&self) {
        use lumacore_base::value::observe_stamp;
        for state in self.playbacks.live.sequences.values() {
            observe_stamp(state.asserted);
            for &stamp in state.go_stamps.values() {
                observe_stamp(stamp);
            }
        }
        for user in self.users.values() {
            for v in user.programmer.values.values() {
                observe_stamp(v.stamp);
            }
        }
    }

    /// The number of the profile with this GUID.
    pub fn profile_no(&self, guid: Uuid) -> Option<u32> {
        self.user_profiles
            .iter()
            .find(|(_, p)| p.meta.identity.guid == guid)
            .map(|(&n, _)| n)
    }

    /// A user's profile: (number, profile).
    pub fn profile_of(&self, user: &crate::programmer::User) -> Option<(u32, &UserProfile)> {
        self.user_profiles
            .iter()
            .find(|(_, p)| p.meta.identity.guid == user.profile)
            .map(|(&n, p)| (n, p.as_ref()))
    }

    /// The name of the user with this GUID.
    pub fn user_name(&self, guid: Uuid) -> Option<&str> {
        self.user_by_guid(guid).map(|u| u.login.as_str())
    }

    /// The user with this GUID.
    pub fn user_by_guid(&self, guid: Uuid) -> Option<&crate::programmer::User> {
        self.users.values().find(|u| u.meta.identity.guid == guid)
    }

    /// The number of the user named `name`.
    pub fn user_no(&self, name: &str) -> Option<u32> {
        self.users.iter().find(|(_, u)| u.login.eq_ignore_ascii_case(name)).map(|(no, _)| *no)
    }

    /// The programmer (and selection) of the user named `name`.
    pub fn programmer_of(&self, name: &str) -> Option<&crate::programmer::Programmer> {
        self.user(name).map(|u| &u.programmer)
    }

    pub fn programmer_of_mut(&mut self, name: &str) -> Option<&mut crate::programmer::Programmer> {
        self.user_mut(name).map(|u| &mut u.programmer)
    }

    /// Rebuild every universe's (unsaved) slot occupancy from the patched
    /// fixtures, so a loaded show can't double-patch an address.
    pub fn rebuild_occupancy(&mut self) {
        let patch = &mut self.devices.dmx.patch;
        for u in patch.universes.values_mut() {
            u.mark_free(1, lumacore_fixtures::universe::UNIVERSE_CAPACITY as u16);
        }
        for f in patch.fixtures.values() {
            // A multipatch copy's slots are its fixture's too.
            for brk in f.mvr.breaks.iter().chain(f.luma.multipatch.iter().flat_map(|c| &c.breaks)) {
                if let Some(u) = patch
                    .universes
                    .values_mut()
                    .find(|u| u.meta.identity.guid == brk.universe_id)
                {
                    u.mark_occupied(brk.start_address, brk.channel_count, f.meta.identity.guid);
                }
            }
        }
    }

    /// Rebuild the (unsaved) ID pools from the objects in the show.
    pub fn rebuild_id_pools(&mut self) {
        fn pool<'a>(ids: impl Iterator<Item = &'a u32>) -> IdPool {
            let mut pool = IdPool::new();
            for &id in ids {
                pool.reserve(id);
            }
            pool
        }
        let patch = &mut self.devices.dmx.patch;
        patch.fixture_id_pool = pool(patch.fixtures.values().filter_map(|f| f.mvr.fid).collect::<Vec<_>>().iter());
        patch.rebuild_fid_index();
        patch.universe_id_pool = pool(patch.universes.keys());
        // A line's protocol is the node it lives in.
        for (name, node) in self.protocols.dmx_protocols.iter_mut().chain(self.protocols.data_protocols.iter_mut()) {
            node.id_pool = pool(node.lines.keys());
            for line in node.lines.values_mut() {
                line.protocol.clone_from(name);
            }
        }
    }

    /// A show always has an Admin: an Admin profile and an Admin user (a
    /// rule, not a migration). Returns whether anything was created.
    pub fn ensure_admin(&mut self) -> bool {
        use crate::TreeNode;
        if self.user("Admin").is_some() {
            return false;
        }
        let profile = match self
            .user_profiles
            .iter()
            .find(|(_, p)| p.name() == Some("Admin"))
        {
            Some((&id, _)) => id,
            None => {
                let id = self.user_profiles.keys().next_back().map_or(1, |n| n + 1);
                self.user_profiles
                    .insert(id, std::sync::Arc::new(UserProfile::new_admin(Some("Admin".to_string()))));
                id
            }
        };
        let no = self.users.keys().next_back().map_or(1, |n| n + 1);
        self.users.insert(
            no,
            crate::programmer::User::new(
                no,
                "Admin",
                crate::programmer::UserRights::Admin,
                self.user_profiles[&profile].meta.identity.guid,
            ),
        );
        true
    }

    pub fn new() -> Self {
        let mut sd = Self::default();
        // Create the non-deletable "Default" datapool (ID 1)
        sd.datapools
            .insert(1, Datapool::new(Some("Default".to_string())));
        // The locked template "Default" (ID 1, decisions §30 "12c2"): new
        // profiles start as a copy of it; no user holds it.
        sd.user_profiles.insert(1, std::sync::Arc::new(UserProfile::new_default()));
        // Admin's own profile (ID 2).
        sd.user_profiles
            .insert(2, std::sync::Arc::new(UserProfile::new_admin(Some("Admin".to_string()))));
        // The Debug profile (ID 3) with Ctrl-D → "debug" keybind
        let mut debug_user = UserProfile::new_admin(Some("Debug".to_string()));
        debug_user.bind_command("Ctrl-D", "debug", "Toggle debug mode");
        debug_user.bind_command("Ctrl-Q", "shutdown", "Quick quit");
        sd.user_profiles.insert(3, std::sync::Arc::new(debug_user));
        use crate::programmer::{User, UserRights};
        let (admin, debug) = (sd.user_profiles[&2].meta.identity.guid, sd.user_profiles[&3].meta.identity.guid);
        sd.users.insert(1, User::new(1, "Admin", UserRights::Admin, admin));
        sd.users.insert(2, User::new(2, "Debug", UserRights::Admin, debug));
        // The show's attribute grouping starts from the default template
        // (decisions §30 "12f-a").
        let _ = crate::attribute_resolver::seed(&mut sd, crate::attribute_resolver::DEFAULT_TEMPLATE);
        sd
    }

    // ── GUID ↔ integer ID lookup helpers ─────────────────────────────────

    /// Resolve a user-facing fixture ID to its GUID.
    pub fn fixture_guid(&self, id: u32) -> Option<Uuid> {
        self.devices.dmx.patch.fixture_by_fid(id).map(|(_, f)| f.meta.identity.guid)
    }

    /// Resolve a fixture GUID back to its user-facing integer ID.
    pub fn fixture_id_for_guid(&self, guid: Uuid) -> Option<u32> {
        self.devices.dmx.patch.fixture_by_guid(guid).and_then(|(_, f)| f.mvr.fid)
    }

    /// Resolve a user-facing universe ID to its GUID.
    pub fn universe_guid(&self, id: u32) -> Option<Uuid> {
        self.devices.dmx.patch.universes.get(&id).map(|u| u.meta.identity.guid)
    }

    /// Resolve a universe GUID back to its user-facing integer ID.
    pub fn universe_id_for_guid(&self, guid: Uuid) -> Option<u32> {
        self.devices
            .dmx
            .patch
            .universes
            .iter()
            .find(|(_, u)| u.meta.identity.guid == guid)
            .map(|(&id, _)| id)
    }

    /// Resolve multiple user-facing fixture IDs to GUIDs (preserving order, skipping missing).
    pub fn fixture_guids(&self, ids: &[u32]) -> Vec<Uuid> {
        ids.iter().filter_map(|&id| self.fixture_guid(id)).collect()
    }

    /// Resolve multiple fixture GUIDs back to user-facing integer IDs (preserving order).
    pub fn fixture_ids_for_guids(&self, guids: &[Uuid]) -> Vec<u32> {
        // One map, not a scan per GUID: 35k fixtures stay linear (R1-71).
        let ids: std::collections::HashMap<Uuid, u32> =
            self.devices.dmx.patch.fixtures.values().filter_map(|f| f.mvr.fid.map(|n| (f.meta.identity.guid, n))).collect();
        guids.iter().filter_map(|g| ids.get(g).copied()).collect()
    }
}

/// Machine-specific settings that persist across shows.
///
/// StationSettings is NOT part of the .luma file. It lives in a separate
/// per-machine config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationSettings {
    /// The sACN Component Identifier (CID) for this station.
    /// Stable per machine — regenerated only if the config file is deleted.
    pub sacn_cid: Uuid,
    /// Network interface addresses available for sACN output.
    pub nic_addresses: Vec<Ipv4Addr>,
    /// Root data directory (e.g., ~/lumacore_library/).
    pub data_dir: PathBuf,
    /// Station display name (shown in multi-station setups).
    pub station_name: String,
    /// Priority for authority negotiation in multi-station sessions.
    #[serde(default)]
    pub station_priority: StationPriority,
    /// Which NIC each of the show's networks uses on this station: network
    /// name (lowercase) → interface name, address or subnet (decisions §11).
    /// A network not listed here never sends.
    #[serde(default)]
    pub networks: BTreeMap<String, String>,
    /// Where each of the show's asset roots is on this station: root name
    /// (lowercase) → folder or drive (decisions §8, §30 "12a2"). A root not
    /// listed here has no files on this station.
    #[serde(default)]
    pub asset_roots: BTreeMap<String, String>,
    /// `Set Station ThemeOverride Dark`: every device of this station shows
    /// the normal theme with this preset (decisions §30 "12a3"); None = off.
    #[serde(default)]
    pub theme_override: Option<String>,
    /// Send output after losing the session (IdleMaster, §10). Off.
    #[serde(default)]
    pub output_when_alone: bool,
    /// The LumaSession this station belongs to (named, found by discovery).
    #[serde(default = "default_lumasession")]
    pub lumasession: String,
    /// Where the station is (a label in the LumaSessions grid).
    #[serde(default)]
    pub location: String,
    /// Other masters may invite this station (EnableInvite).
    #[serde(default = "yes")]
    pub invite: bool,
    /// LumaSession on/off (EnableNetwork): off stops everything
    /// networked — LumaSessions, discovery, Shell/HTTP on NICs, DMX
    /// protocols. Localhost keeps working.
    #[serde(default = "yes")]
    pub online: bool,
    /// Which services each NIC serves (interface name → services); a NIC not
    /// listed serves everything.
    #[serde(default)]
    pub services: BTreeMap<String, NicServices>,
    /// DMX output held off after leaving or being dismissed from a
    /// LumaSession, until the operator resumes it.
    #[serde(default)]
    pub output_held: bool,
    /// Discovery multicast group (Multicast Base): Alternative when
    /// something else on the rig uses the default group (§21 "6c").
    #[serde(default)]
    pub multicast_alternative: bool,
    /// This station's drives (decisions §38): id (`usb1`, `ssda`, `folder1`) → what it is bound
    /// to, so the same stick keeps its id; added folders carry their path.
    #[serde(default)]
    pub drives: BTreeMap<String, DriveMemory>,
}

/// What kind of drive an id names (decisions §38): the prefix of its id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriveKind {
    /// The station's own `library/` (Drive 1).
    LocalLibrary,
    /// A fixed disk beyond the system one (letters: `ssda`, `ssdb`).
    Ssd,
    Hdd,
    /// A removable volume on USB (`usb1`).
    Usb,
    /// A folder added by hand (`folder1`).
    Folder,
    /// LumaCore's own network connections (planned): `smb1`, `ftp1`, `sftp1`.
    Smb,
    Ftp,
    Sftp,
}

impl DriveKind {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::LocalLibrary => "local-library",
            Self::Ssd => "ssd",
            Self::Hdd => "hdd",
            Self::Usb => "usb",
            Self::Folder => "folder",
            Self::Smb => "smb",
            Self::Ftp => "ftp",
            Self::Sftp => "sftp",
        }
    }

    /// The word a setting shows and takes (`Set Drive "ssdb" "Kind" "USB"`).
    pub fn word(self) -> &'static str {
        match self {
            Self::LocalLibrary => "Internal",
            Self::Ssd => "SSD",
            Self::Hdd => "HDD",
            Self::Usb => "USB",
            Self::Folder => "Folder",
            Self::Smb => "SMB",
            Self::Ftp => "FTP",
            Self::Sftp => "SFTP",
        }
    }

    pub fn from_word(w: &str) -> Option<Self> {
        [Self::Ssd, Self::Hdd, Self::Usb, Self::Folder, Self::Smb, Self::Ftp, Self::Sftp]
            .into_iter()
            .find(|k| k.word().eq_ignore_ascii_case(w))
    }
}

/// A remembered drive: the volume it is bound to (its key) or the folder it was added from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveMemory {
    /// The volume's key (`uuid:…`, `probe:…`, `label:…`) — empty for an added folder.
    #[serde(default)]
    pub key: String,
    pub kind: DriveKind,
    /// The display name: the volume's label, or what was set.
    #[serde(default)]
    pub label: String,
    /// An added folder's path (None for a found volume).
    #[serde(default)]
    pub path: Option<String>,
    /// When it was last seen (unix ms).
    #[serde(default)]
    pub last_seen: u64,
}

/// What one NIC serves (Menu → Network → Interfaces).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NicServices {
    /// Discovery, the command port and replication.
    pub lumasession: bool,
    pub shell: bool,
    pub http: bool,
}

impl Default for NicServices {
    fn default() -> Self {
        Self {
            lumasession: true,
            shell: true,
            http: true,
        }
    }
}

/// One row of the station table (`ShowData.Stations.<n>`, decisions §30
/// "12d part 4"): a station's wiring in this show.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StationWiring {
    pub meta: NodeMeta,
    /// The station id (`station_settings.json`) this row belongs to; empty
    /// = made in advance, not yet claimed by a station.
    #[serde(default)]
    pub station_id: String,
    /// The station's NIC for each of the show's networks: network GUID →
    /// interface name, address or subnet. A network not listed never sends
    /// from this station.
    #[serde(default)]
    pub networks: BTreeMap<uuid::Uuid, String>,
    /// What each NIC serves (interface name, lowercase → services); a NIC
    /// not listed serves everything.
    #[serde(default)]
    pub services: BTreeMap<String, NicServices>,
}

impl StationWiring {
    /// The GUID of the row station `station_id` makes for itself (derived,
    /// so two sides of a split make the same object; never the station's
    /// `KnownStation` GUID).
    pub fn guid_for(station_id: &str) -> uuid::Uuid {
        lumacore_base::Guid::named("StationWiring", station_id).0
    }

    /// What NIC `nic` serves in this row.
    pub fn services_of(&self, nic: &str) -> NicServices {
        self.services.get(&nic.to_lowercase()).copied().unwrap_or_default()
    }
}

fn default_lumasession() -> String {
    "LumaCore".to_string()
}

fn yes() -> bool {
    true
}

impl StationSettings {
    /// What `nic` serves.
    pub fn services_of(&self, nic: &str) -> NicServices {
        self.services
            .get(&nic.to_lowercase())
            .copied()
            .unwrap_or_default()
    }
}

impl Default for StationSettings {
    fn default() -> Self {
        let station_name = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "LumaCore".to_string());
        Self {
            sacn_cid: Uuid::new_v4(),
            nic_addresses: Vec::new(),
            data_dir: PathBuf::from("~/.local/share/lumacore"),
            station_name,
            station_priority: StationPriority::default(),
            networks: BTreeMap::new(),
            asset_roots: BTreeMap::new(),
            theme_override: None,
            output_when_alone: false,
            lumasession: default_lumasession(),
            location: String::new(),
            invite: true,
            online: true,
            services: BTreeMap::new(),
            output_held: false,
            multicast_alternative: false,
            drives: BTreeMap::new(),
        }
    }
}

impl StationSettings {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Root-level library branch — tracks the `library/` directory.
#[derive(Debug, Clone, Default)]
pub struct LibraryRoot {
    /// Path to the library directory.
    pub path: Option<PathBuf>,
}

/// The root of the LumaCore object tree.
///
/// Root owns all tree data and the observer list. All tree mutations go
/// through Root methods, which emit events to registered observers.
///
/// Observers are not serialized (they are runtime-only closures).
pub struct Root {
    pub show_data: ShowData,
    pub station_settings: StationSettings,
    /// Runtime-only state (never saved). Sessions, engines, network, diagnostics.
    pub runtime: RuntimeState,
    /// Live UI engines which render saved user views into workspaces.
    pub ui_engines: UIEnginesRoot,
    /// Shared resolved assets available to engines.
    pub resources: ResourceRoot,
    /// Top-level diagnostics and status.
    pub diagnostics: DiagnosticRoot,
    /// Library root — fixture libraries, shows, logs, imported resources.
    pub library: LibraryRoot,
    /// Vanilla (factory default) key bindings. Not serialized — rebuilt on startup.
    /// User overrides live in UserProfile (inside ShowData).
    pub default_keybindings: KeyBindingMap,
    /// Debug-only key bindings. Override everything when debug mode is active.
    pub debug_keybindings: KeyBindingMap,
    /// Runtime-only observer list. Not serialized.
    observers: Vec<TreeObserver>,
}

impl Root {
    /// Create a new empty Root with default show data and station settings.
    ///
    /// - ShowData is empty except for the default Admin UserProfile.
    /// - StationSettings gets a fresh random CID.
    pub fn new() -> Self {
        Self {
            show_data: ShowData::new(),
            station_settings: StationSettings::new(),
            runtime: RuntimeState::default(),
            ui_engines: UIEnginesRoot::default(),
            resources: ResourceRoot::default(),
            diagnostics: DiagnosticRoot::default(),
            library: LibraryRoot::default(),
            default_keybindings: vanilla_keybindings(),
            debug_keybindings: debug_keybindings(),
            observers: Vec::new(),
        }
    }

    /// Register an observer to receive tree events.
    ///
    /// Observers are called synchronously in registration order.
    pub fn add_observer(&mut self, observer: TreeObserver) {
        self.observers.push(observer);
    }

    /// Emit a tree event to all registered observers.
    pub fn emit(&self, event: &TreeEvent) {
        for observer in &self.observers {
            observer(event);
        }
    }
}

impl Default for Root {
    fn default() -> Self {
        Self::new()
    }
}

impl Root {
    /// Resolve effective keybindings for a user with three-layer priority.
    ///
    /// Priority: debug (if active) > user > vanilla.
    pub fn resolve_keybindings(&self, user_id: u32, debug_active: bool) -> KeyBindingMap {
        let user_bindings = self
            .show_data
            .user_profiles
            .get(&user_id)
            .map(|p| &p.content.keybindings)
            .cloned()
            .unwrap_or_default();
        crate::keybindings::resolve_bindings(
            &self.default_keybindings,
            &user_bindings,
            &self.debug_keybindings,
            debug_active,
        )
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn factory_binds_are_user_keys() {
        let show = ShowData::new();
        assert_eq!(
            show.profile_for_user("Debug").unwrap().content.keybindings["Ctrl-D"].ui_key,
            "user.ctrl-d"
        );
    }

    use super::*;
    use crate::node::TreeNode;
    use std::sync::{Arc, Mutex};

    /// R1-57 (Q32): a saved window whose type this build doesn't know is
    /// dropped (its pane closes) and named; known windows stay.
    #[test]
    fn saved_windows_of_an_unknown_type_are_dropped() {
        use crate::runtime::{LayoutNode, Window};
        let mut show = ShowData::new();
        show.ensure_admin();
        let profile = show.user_profiles.values_mut().map(std::sync::Arc::make_mut).next().unwrap();
        let mut windows = BTreeMap::new();
        windows.insert(1, Window::new(1, "terminal"));
        windows.insert(2, Window::new(2, "no-such-type"));
        let content = crate::programmer::ScreenContent {
            meta: Default::default(),
            root: LayoutNode::Leaf { windows: vec![1, 2], active: 0 },
            windows,
            encoder_bar: Default::default(),
        };
        let mut config = crate::programmer::ScreenConfig {
            meta: NodeMeta::new(Some("Screens".into())),
            screens: BTreeMap::new(),
        };
        config.screens.insert(1, content);
        profile.content.screen_configs.insert(1, config);
        let notes = show.drop_unknown_windows();
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(notes[0].contains("no-such-type"));
        let content = &show.user_profiles.values().next().unwrap().content.screen_configs[&1].screens[&1];
        assert_eq!(content.windows.keys().copied().collect::<Vec<_>>(), vec![1]);
        assert!(matches!(&content.root, LayoutNode::Leaf { windows, .. } if windows == &vec![1]));
    }

    #[test]
    fn a_loaded_show_with_stamps_ahead_floors_new_touches() {
        // A show saved on a station whose clock was an hour ahead (R1-01).
        let ahead = lumacore_base::value::next_stamp() + 3_600_000_000;
        let mut show = ShowData::new();
        show.ensure_admin();
        let fixture = Uuid::new_v4();
        let user = show.users.values_mut().next().unwrap();
        user.programmer
            .set_value(fixture, "Dimmer", crate::Value::normalized(50.0));
        user.programmer
            .values
            .get_mut(&(fixture, "Dimmer".to_string()))
            .unwrap()
            .stamp = ahead;
        let mut loaded: ShowData =
            serde_json::from_str(&serde_json::to_string(&show).unwrap()).unwrap();
        loaded.after_load();
        // A touch made after the load wins over the loaded one.
        let user = loaded.users.values_mut().next().unwrap();
        user.programmer
            .set_value(Uuid::new_v4(), "Dimmer", crate::Value::normalized(10.0));
        let newest = user.programmer.values.values().map(|v| v.stamp).max().unwrap();
        assert!(newest > ahead);
    }

    #[test]
    fn root_new_creates_admin_user_profile() {
        let root = Root::new();
        let admin = root.show_data.profile_for_user("Admin").unwrap();
        assert_eq!(admin.name(), Some("Admin"));
        assert!(!admin.locked);
    }

    #[test]
    fn root_show_data_is_empty_except_admin() {
        let root = Root::new();
        assert!(root.show_data.devices.dmx.patch.fixture_types.is_empty());
        assert!(root.show_data.devices.dmx.patch.fixtures.is_empty());
        assert!(root.show_data.devices.dmx.patch.universes.is_empty());
        assert_eq!(root.show_data.protocols.all_lines().count(), 0);
    }

    #[test]
    fn root_station_settings_has_valid_cid() {
        let root = Root::new();
        assert_eq!(
            root.station_settings.sacn_cid.get_version(),
            Some(uuid::Version::Random)
        );
    }

    #[test]
    fn root_emit_notifies_observers() {
        let mut root = Root::new();
        let received = Arc::new(Mutex::new(0u32));
        let counter = received.clone();

        root.add_observer(Box::new(move |_event| {
            *counter.lock().unwrap() += 1;
        }));

        root.emit(&TreeEvent::UniverseCreated { universe_id: 1 });
        root.emit(&TreeEvent::UniverseCreated { universe_id: 2 });

        assert_eq!(*received.lock().unwrap(), 2);
    }

    #[test]
    fn show_data_new_has_admin_profile() {
        let sd = ShowData::new();
        assert_eq!(sd.user_profiles.len(), 3);
        // 1 = the locked template Default (decisions §30 "12c2"), 2 Admin, 3 Debug.
        assert_eq!(sd.default_profile().map(|(n, p)| (n, p.name())), Some((1, Some("Default"))));
        assert_eq!(sd.profile_id_for_user("Admin"), Some(2));
        assert_eq!(sd.profile_id_for_user("Debug"), Some(3));
        assert!(sd.users.values().all(|u| u.profile != sd.user_profiles[&1].meta.identity.guid), "nobody holds Default");
    }

    #[test]
    fn rights_and_profiles_come_only_from_users() {
        let mut sd = ShowData::new();
        // A profile named like a user that doesn't exist gives nothing.
        sd.user_profiles.insert(9, std::sync::Arc::new(UserProfile::new_admin(Some("Ghost".into()))));
        assert_eq!(sd.rights_of("Ghost"), None);
        assert_eq!(sd.profile_id_for_user("Ghost"), None);
    }

    #[test]
    fn a_profile_copy_gets_new_guids_and_keeps_its_links() {
        let mut p = UserProfile::new(Some("Bob".into()));
        p.bind_command("Ctrl-R", "Go", "Go");
        let view = crate::programmer::StoredView {
            meta: NodeMeta::new(Some("Main".into())),
            screens: Default::default(),
        };
        let button = crate::programmer::ViewButton { kind: crate::programmer::ButtonKind::View, guid: view.meta.identity.guid };
        p.content.view_buttons.entry(1).or_default().insert(1, button);
        p.content.views.insert(1, view);
        p.locked = true;
        let copy = p.copy_renewed(Some("Bob 2".into()));
        assert_eq!((copy.name(), copy.locked), (Some("Bob 2"), false));
        assert_ne!(copy.meta.identity.guid, p.meta.identity.guid);
        assert_ne!(copy.content.views[&1].meta.identity.guid, p.content.views[&1].meta.identity.guid);
        assert_ne!(copy.content.ui_keys[&1].meta.identity.guid, p.content.ui_keys[&1].meta.identity.guid);
        assert_eq!(copy.content.view_buttons[&1][&1].guid, copy.content.views[&1].meta.identity.guid, "the view button follows the copy");
        let mut sd = ShowData::new();
        sd.user_profiles.insert(4, std::sync::Arc::new(p));
        sd.user_profiles.insert(5, std::sync::Arc::new(copy));
        assert!(sd.identity().duplicates.is_empty());
    }
}
