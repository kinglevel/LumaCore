//! lumacore-playback: Playback data model types for LumaCore.
//!
//! Sequences → cues → parts (decisions §27: values and timing live on the
//! part; part 0 always exists), tracking, masters, park, effects and timing
//! presets — plus the running state of playbacks (`PlaybackStates`, saved
//! with the show, decisions §13).

use lumacore_base::{Identified, NodeMeta, Value, Walk, Walker};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// The show-wide playback objects. Sequences, Timing presets and Curve presets live in the
/// DataPools (decisions §32, §30 "12f": `Datapool::sequences`); Effects and Executors move there
/// with their own steps.
pub struct PlaybackData {
    #[serde(default)]
    pub effects: BTreeMap<u32, Effect>,
    #[serde(default)]
    pub executors: BTreeMap<u32, ExecutorObject>,
    /// Parked values: nothing overrides them (decisions §27).
    #[serde(default)]
    pub park: ParkData,
}

/// Parked values: per fixture attribute, and raw DMX channels (for unpatched
/// channels). Park is not the DMX tester.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParkData {
    #[serde(default)]
    pub fixtures: Vec<ParkedValue>,
    #[serde(default)]
    pub channels: Vec<ParkedChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkedValue {
    pub fixture_id: Uuid,
    pub attribute: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkedChannel {
    /// The universe (GUID link).
    pub universe: Uuid,
    pub address: u16,
    pub value: u8,
}

impl ParkData {
    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty() && self.channels.is_empty()
    }

    /// Park one attribute (replaces an earlier park of it).
    pub fn park(&mut self, fixture_id: Uuid, attribute: &str, value: Value) {
        self.fixtures
            .retain(|p| !(p.fixture_id == fixture_id && p.attribute == attribute));
        self.fixtures.push(ParkedValue {
            fixture_id,
            attribute: attribute.to_string(),
            value,
        });
    }

    /// Park one DMX channel (replaces an earlier park of it).
    pub fn park_channel(&mut self, universe: Uuid, address: u16, value: u8) {
        self.unpark_channel(universe, address);
        self.channels.push(ParkedChannel {
            universe,
            address,
            value,
        });
    }

    pub fn unpark_channel(&mut self, universe: Uuid, address: u16) -> bool {
        let before = self.channels.len();
        self.channels
            .retain(|c| !(c.universe == universe && c.address == address));
        before != self.channels.len()
    }

    /// Unpark a fixture (all attributes, or one). Returns how many went.
    pub fn unpark(&mut self, fixture_id: Uuid, attribute: Option<&str>) -> usize {
        let before = self.fixtures.len();
        self.fixtures.retain(|p| {
            p.fixture_id != fixture_id
                || attribute.is_some_and(|a| !p.attribute.eq_ignore_ascii_case(a))
        });
        before - self.fixtures.len()
    }

    pub fn get(&self, fixture_id: Uuid, attribute: &str) -> Option<&Value> {
        self.fixtures
            .iter()
            .find(|p| p.fixture_id == fixture_id && p.attribute == attribute)
            .map(|p| &p.value)
    }
}

/// An assignment linking a sequence to an executor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SequenceAssignment {
    pub meta: NodeMeta,
    /// The executor (GUID link).
    #[serde(default)]
    pub executor: Option<Uuid>,
}

/// A cue list. Cues are keyed by number × 1000 (`Cue 1.5` = 1500). Every sequence
/// has its fixed **CueZero** (the start state, `Cue 0`) and **OffCue** (the Off timing) — a type
/// invariant (decisions §32): they can't be deleted, moved or renumbered, and a file without them
/// is refused (no serde default).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceObject {
    /// Name, colour, icon … (a DataPool object like a Group; decisions §30 "12f").
    pub meta: lumacore_base::node::NodeMeta,
    /// What a run from the top starts from (CueZero): stored into like a cue, filled with
    /// defaults by `settings.cue_zero_mode` (decisions §30 "12f echoes").
    pub cue_zero: CueObject,
    /// The Off timing (its part 0's out fade / delay); releases everything (OffCue).
    pub off_cue: CueObject,
    #[serde(default)]
    pub cues: BTreeMap<u32, CueObject>,
    #[serde(default)]
    pub assignments: BTreeMap<u32, SequenceAssignment>,
    #[serde(default)]
    pub settings: SequenceSettings,
}

/// Cue number → key (`1.5` → 1500). None for a negative or malformed number
/// and for more than 3 decimals (`1.9999` is never rounded into Cue 2;
/// trailing zeros don't count): read from the typed digits, not the float.
pub fn cue_key(no: &str) -> Option<u32> {
    let (whole, frac) = no.split_once('.').unwrap_or((no, ""));
    let frac = frac.trim_end_matches('0');
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || !digits(frac) || frac.len() > 3 || (whole.is_empty() && frac.is_empty()) {
        return None;
    }
    let whole: u32 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
    let thousandths: u32 = if frac.is_empty() { 0 } else { format!("{frac:0<3}").parse().ok()? };
    let key = whole.checked_mul(1000)?.checked_add(thousandths)?;
    (key > 0 && key < 4_000_000_000).then_some(key)
}

/// Cue key → number as typed (`1500` → `1.5`, `2000` → `2`).
pub fn cue_label(key: u32) -> String {
    if key.is_multiple_of(1000) {
        (key / 1000).to_string()
    } else {
        let s = format!("{}.{:03}", key / 1000, key % 1000);
        s.trim_end_matches('0').to_string()
    }
}

/// A tracked value at a cue: the value and the cue (key) that stored it.
pub struct Tracked<'a> {
    /// Plain steps or a preset link (resolve a link with [`CueData::resolve`]).
    pub value: &'a CueData,
    pub cue: u32,
    /// The part of `cue` it is stored in (its timing).
    pub part: u32,
    /// Its place in that part's values (the plan compiler finds it again without a copy).
    pub index: usize,
}

impl Default for SequenceObject {
    fn default() -> Self {
        Self {
            meta: lumacore_base::node::NodeMeta::new(None),
            cue_zero: CueObject::new("CueZero"),
            off_cue: CueObject::new("OffCue"),
            cues: BTreeMap::new(),
            assignments: BTreeMap::new(),
            settings: SequenceSettings::default(),
        }
    }
}

/// CueZero's key (`Cue 0`): before every numbered cue.
pub const CUE_ZERO: u32 = 0;

impl SequenceObject {
    /// The cue at `key`: CueZero at 0, else a numbered cue.
    pub fn cue(&self, key: u32) -> Option<&CueObject> {
        if key == CUE_ZERO { Some(&self.cue_zero) } else { self.cues.get(&key) }
    }

    /// The cue after `key` (the first cue when `None`); wraps if the sequence
    /// wraps around.
    pub fn next_cue(&self, key: Option<u32>) -> Option<u32> {
        let next = match key {
            None => self.cues.keys().next().copied(),
            Some(k) => self.cues.range(k + 1..).next().map(|(&k, _)| k),
        };
        next.or_else(|| {
            (self.settings.wrap_around && key.is_some())
                .then(|| self.cues.keys().next().copied())
                .flatten()
        })
    }

    /// The cue before `key`.
    pub fn previous_cue(&self, key: u32) -> Option<u32> {
        self.cues.range(..key).next_back().map(|(&k, _)| k)
    }

    /// What the sequence outputs at cue `key` (manual `cue_tracking`): each
    /// cue stores only changes, a value tracks until a new value or a
    /// release. With tracking off only the cue's own values count. Releases
    /// are not returned (the attribute goes to lower sources).
    pub fn tracked(&self, key: u32) -> BTreeMap<(Uuid, &str), Tracked<'_>> {
        let mut out: BTreeMap<(Uuid, &str), Tracked<'_>> = BTreeMap::new();
        // A run starts from CueZero (decisions §32): its values track into every cue.
        let zero = (&CUE_ZERO, &self.cue_zero);
        let cues: Vec<(&u32, &CueObject)> = if key == CUE_ZERO {
            vec![zero]
        } else if self.settings.tracking {
            std::iter::once(zero).chain(self.cues.range(1..=key)).collect()
        } else {
            self.cues.get_key_value(&key).into_iter().collect()
        };
        for (&cue, obj) in cues {
            for (&part_no, part) in &obj.parts {
                for (index, v) in part.values.iter().enumerate() {
                    let k = (v.fixture_id, v.attribute.as_str());
                    if v.value.is_release() {
                        out.remove(&k);
                    } else {
                        out.insert(
                            k,
                            Tracked {
                                value: &v.value,
                                cue,
                                part: part_no,
                                index,
                            },
                        );
                    }
                }
            }
        }
        out
    }
}

/// Priority mode for playback output merging.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum PlaybackPriority {
    #[default]
    Ltp,
    Htp,
}

/// Determines which cue to jump to when a sequence is retriggered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum RestartMode {
    #[default]
    FirstCue,
    CurrentCue,
}

/// Per-sequence playback behaviour settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceSettings {
    /// Global tracking on/off for this sequence.
    #[serde(default = "default_true")]
    pub tracking: bool,
    /// **Planned** (saved, not used yet; R1-D8): the priority band.
    #[serde(default)]
    pub priority: PlaybackPriority,
    /// **Planned** (saved, not used yet; R1-D8): loop back to the first cue after the last.
    #[serde(default)]
    pub wrap_around: bool,
    /// **Planned** (saved, not used yet; R1-D8 — every field below until
    /// `kill_protect`): where a Go after Off starts.
    #[serde(default)]
    pub restart_mode: RestartMode,
    /// **Planned**: start when a value of the sequence is called.
    #[serde(default)]
    pub auto_start: bool,
    /// **Planned**: go off when every value is overridden.
    #[serde(default)]
    pub auto_stop: bool,
    /// **Planned**: fade-to-off instead of snap on release.
    #[serde(default)]
    pub soft_ltp: bool,
    /// **Planned**: crossfade between cues.
    #[serde(default)]
    pub xfade_mode: bool,
    /// **Planned**: Move-In-Black for this sequence.
    #[serde(default)]
    pub mib_enabled: bool,
    /// **Planned**: protected from Swop.
    #[serde(default)]
    pub swap_protect: bool,
    /// **Planned**: protected from Kill.
    #[serde(default)]
    pub kill_protect: bool,
    /// What CueZero is filled with (CueZeroMode; decisions §30 "12f echoes").
    #[serde(default)]
    pub cue_zero_mode: CueZeroMode,
    /// How many echoes may run at once (`Top On`, §32); 0 = Top refused. A full list refuses
    /// the newest Top On — an old echo is never dropped.
    #[serde(default = "default_max_echoes")]
    pub max_echoes: u32,
    /// Seconds an echo may run before its OffCue starts by itself (§32 EchoMax).
    #[serde(default = "default_echo_max")]
    pub echo_max_s: f64,
}

fn default_max_echoes() -> u32 {
    8
}

fn default_echo_max() -> f64 {
    30.0
}

/// What CueZero is filled with automatically: nothing (`Off`, the default), the
/// defaults of every attribute of the sequence's fixtures, or their dimmers only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CueZeroMode {
    #[default]
    Off,
    AllUsedAttributes,
    OnlyUsedDimmers,
}

impl CueZeroMode {
    pub const ALL: [CueZeroMode; 3] = [Self::Off, Self::AllUsedAttributes, Self::OnlyUsedDimmers];

    pub fn word(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::AllUsedAttributes => "AllUsedAttributes",
            Self::OnlyUsedDimmers => "OnlyUsedDimmers",
        }
    }
}

fn default_true() -> bool {
    true
}

impl Default for SequenceSettings {
    fn default() -> Self {
        Self {
            tracking: true,
            priority: PlaybackPriority::Ltp,
            wrap_around: false,
            restart_mode: RestartMode::FirstCue,
            auto_start: false,
            auto_stop: false,
            soft_ltp: false,
            xfade_mode: false,
            mib_enabled: false,
            swap_protect: false,
            kill_protect: false,
            cue_zero_mode: CueZeroMode::Off,
            max_echoes: default_max_echoes(),
            echo_max_s: default_echo_max(),
        }
    }
}

/// Move-In-Black preference for an individual cue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MibPreference {
    #[default]
    Normal,
    Forced,
    Inhibit,
}

/// Trigger mode for a cue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum CueTrigger {
    #[default]
    Go,
    Follow,
    Time(f64),
    Manual,
    Sound,
    Bpm(f64),
    Midi,
}

/// What a cue value holds (decisions 0293): plain steps, or a link to a preset. A link holds no
/// value of its own: the cue plays the preset as it is now (the fixture's own entry, else the
/// preset's universal one; none = no value), resolved where the plan is compiled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CueData {
    /// Plain steps (a release too). Its `preset` is always `None` here.
    Value(Value),
    /// The preset's GUID (a link, decisions §21 "6d").
    Link(Uuid),
}

impl CueData {
    /// What a programmer value becomes in a cue: a value that came from a preset is the link
    /// only, everything else its plain steps.
    pub fn from_programmer(v: &Value) -> Self {
        match v.preset {
            Some(g) if !v.release => Self::Link(g),
            _ => {
                let mut v = v.clone();
                v.preset = None;
                Self::Value(v)
            }
        }
    }

    /// The plain value, if it is one.
    pub fn plain(&self) -> Option<&Value> {
        match self {
            Self::Value(v) => Some(v),
            Self::Link(_) => None,
        }
    }

    /// The preset it links to, if it is a link.
    pub fn link(&self) -> Option<Uuid> {
        match self {
            Self::Link(g) => Some(*g),
            Self::Value(_) => None,
        }
    }

    /// A release (the attribute goes back to lower sources).
    pub fn is_release(&self) -> bool {
        matches!(self, Self::Value(v) if v.release)
    }

    /// The value it plays: its own, or the linked preset's for this fixture and attribute.
    pub fn resolve<'a, P: PresetValues + ?Sized>(&'a self, fixture: Uuid, attribute: &str, presets: &'a P) -> Option<&'a Value> {
        match self {
            Self::Value(v) => Some(v),
            Self::Link(g) => presets.preset_value(*g, fixture, attribute),
        }
    }
}

/// Where a cue's preset links are looked up (the show's preset pools; the plan compiler's own
/// per-compile index): the preset `preset`'s value for `fixture`'s `attribute` — the fixture's
/// own entry, else the universal one; None when the preset is gone or holds no such value.
pub trait PresetValues {
    fn preset_value(&self, preset: Uuid, fixture: Uuid, attribute: &str) -> Option<&Value>;
}

/// No presets (a link plays nothing): for readers that only look at plain values.
pub struct NoPresets;

impl PresetValues for NoPresets {
    fn preset_value(&self, _: Uuid, _: Uuid, _: &str) -> Option<&Value> {
        None
    }
}

/// One fixture attribute's value in a cue part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CueValue {
    pub fixture_id: Uuid,
    pub attribute: String,
    /// Plain steps or a preset link (decisions 0293).
    pub value: CueData,
    /// Its own Fade / Delay over the cue's (decisions §30 "12f" value timing).
    #[serde(default, skip_serializing_if = "lumacore_base::value::ValueTiming::is_cue")]
    pub timing: lumacore_base::value::ValueTiming,
}

/// A cue part: values and timing (part 0 always exists). Every part runs
/// its own timing from the same Go, like a cue stacked inside the cue
/// (decisions §29 "11b hardening"). Times in seconds. The usual rule
/// per attribute: a dimmer going up and every other attribute use the in
/// fade/delay, a dimmer going down the out fade/delay.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CuePart {
    pub meta: NodeMeta,
    #[serde(default)]
    pub fade_in: f64,
    #[serde(default)]
    pub fade_out: f64,
    #[serde(default)]
    pub delay_in: f64,
    #[serde(default)]
    pub delay_out: f64,
    #[serde(default)]
    pub values: Vec<CueValue>,
    /// Timing per show feature group (decisions §30 "12f-a"), keyed by the
    /// feature group's GUID: a set field overrides the part's own for the
    /// attributes of that group (precedence: cue → feature group →
    /// individual attribute → executor → rate).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub group_timing: BTreeMap<Uuid, GroupTiming>,
}

/// One feature group's timing in a cue part; `None` = the part's own.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct GroupTiming {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_in: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_out: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_in: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_out: Option<f64>,
    /// The shape of its fades, in and out (decisions §30 "12f" sequence window A); `None` = Linear.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<FadePath>,
}

/// How a fade travels from where a channel is to its new value (the sequence window's Path).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FadePath {
    #[default]
    Linear,
    /// Smooth start and end (smoothstep).
    SCurve,
    /// Slow start.
    EaseIn,
    /// Slow end.
    EaseOut,
    /// Jumps at the end of its delay.
    Snap,
}

impl FadePath {
    pub const ALL: [FadePath; 5] = [Self::Linear, Self::SCurve, Self::EaseIn, Self::EaseOut, Self::Snap];

    /// The word in a command line and in the sheet.
    pub fn word(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::SCurve => "SCurve",
            Self::EaseIn => "EaseIn",
            Self::EaseOut => "EaseOut",
            Self::Snap => "Snap",
        }
    }

    /// `S-curve`, `ease in`, `SCurve` … (case, spaces and dashes don't count).
    pub fn parse(s: &str) -> Option<Self> {
        let w: String = s.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        Self::ALL.into_iter().find(|p| p.word().eq_ignore_ascii_case(&w))
    }

    /// Where a fade is (0–1) at `t` (0–1) of its time.
    pub fn at(self, t: f64) -> f64 {
        match self {
            Self::Linear => t,
            Self::SCurve => t * t * (3.0 - 2.0 * t),
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::Snap => 1.0,
        }
    }
}

impl GroupTiming {
    pub fn is_empty(&self) -> bool {
        *self == GroupTiming::default()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CueObject {
    pub meta: NodeMeta,
    /// **Planned** (saved, not used yet; R1-D8): every cue runs on Go today.
    #[serde(default)]
    pub trigger: CueTrigger,
    #[serde(default)]
    pub parts: BTreeMap<u32, CuePart>,
    /// **Planned** (saved, not used yet; R1-D8): a blocking cue (no tracked
    /// values pass through). Blocked *values* are stored values equal to
    /// what tracks in (R1-31).
    #[serde(default)]
    pub blocked: bool,
    /// **Planned** (saved, not used yet; R1-D8).
    #[serde(default)]
    pub mib_preference: MibPreference,
}

impl CueObject {
    pub fn new(name: impl Into<String>) -> Self {
        let name: String = name.into();
        let mut cue = Self {
            meta: NodeMeta::new(Some(name).filter(|n| !n.is_empty())),
            ..Self::default()
        };
        cue.parts.insert(0, CuePart::default());
        cue
    }

    /// Part 0 (created if missing).
    pub fn part0(&mut self) -> &mut CuePart {
        self.parts.entry(0).or_default()
    }

    /// Every value of every part.
    pub fn values(&self) -> impl Iterator<Item = &CueValue> {
        self.parts.values().flat_map(|p| p.values.iter())
    }
}

fn default_level() -> f64 {
    1.0
}

fn default_rate() -> f64 {
    1.0
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Effect {
    pub meta: NodeMeta,
    #[serde(default)]
    pub effect_type: String,
    #[serde(default = "default_rate")]
    pub rate: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimingPreset {
    pub meta: NodeMeta,
    #[serde(default)]
    pub fade_in: f64,
    #[serde(default)]
    pub fade_out: f64,
    #[serde(default)]
    pub delay: f64,
}

fn default_curve_type() -> String {
    "Linear".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CurvePreset {
    pub meta: NodeMeta,
    #[serde(default = "default_curve_type")]
    pub curve_type: String,
}

fn default_fader_position() -> f64 {
    1.0
}

fn default_page() -> u32 {
    1
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutorObject {
    pub meta: NodeMeta,
    /// The sequence on it (GUID link).
    #[serde(default)]
    pub assigned_sequence: Option<Uuid>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_fader_position")]
    pub fader_position: f64,
}

// ── Running state (ShowData.Playbacks) ──────────────────────────────────────

/// What runs (`ShowData.Playbacks`, decisions §13, §32 "the Playbacks registry"): the playback
/// engine's one source of truth, split at the top by what is saved. `Live` is saved and resumed
/// snapped on load; `Echoes` (§32, part 2 of 12f) are replicated, never saved. Never undone.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaybackStates {
    #[serde(default)]
    pub live: LivePlaybacks,
    /// Replicated, never saved: the show file strips them and a load starts with none.
    #[serde(default)]
    pub echoes: EchoPlaybacks,
}

/// Echoes by engine kind (decisions §32): runs a `Top On` starts beside the live playback.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EchoPlaybacks {
    /// By the sequence's GUID, oldest first.
    #[serde(default)]
    pub sequences: BTreeMap<Uuid, Vec<Echo>>,
}

/// One echo (§32 "Top = the echo multi-caller"): a run from CueZero to the first cue with the
/// cue's timing; `Top Off` (or EchoMax, or its holder going away) plays its OffCue, and it is
/// gone when that has faded. Not a controllable playback: no Go, Goto or fader.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Echo {
    /// Its number within the sequence's echoes (for `List`).
    pub id: u32,
    /// The cue it stands on.
    pub cue: u32,
    /// When it started (session clock ms).
    pub started_ms: u64,
    /// Its LTP stamp: an echo is the newest run when it starts (`now_ms`).
    pub stamp: u64,
    /// The client holding it (a held pool cell / executor button), if any.
    #[serde(default)]
    pub holder: Option<String>,
    /// The user who started it (GUID link).
    pub started_by: Uuid,
    /// When its OffCue started (session clock ms); None = still running.
    #[serde(default)]
    pub off_ms: Option<u64>,
}

/// The saved part of the registry: one map per engine kind (effects join with §28) and the
/// masters (decisions §30 "12f against §32": `Playbacks.Live.Masters`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LivePlaybacks {
    /// Running sequences by the sequence's GUID.
    #[serde(default)]
    pub sequences: BTreeMap<Uuid, SequenceState>,
    #[serde(default)]
    pub masters: Masters,
}

/// The master groups (`ShowData.Masters`), ours numbered inside each group: 1 Selected
/// (planned) · 2 Grand · 3 Speed (planned, §28) · 4 Playback (planned, executors) · 5 Timing
/// (planned) — and the group masters. Saved, replicated, never undone (decisions §30 "12f").
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Masters {
    #[serde(default)]
    pub grand: GrandMasters,
    /// Group master levels by the group's GUID (decisions §22): the mode is on the group, the
    /// level lives here so moving it never fills undo.
    #[serde(default)]
    pub groups: BTreeMap<Uuid, MasterLevel>,
}

/// Master group 2, Grand: 2.1 Master (blackout = switched off, the level kept), 2.2 Highlight
/// and 2.4 Solo — caps on everybody's Highlight / Solo (each user's own masters scale theirs,
/// `Runtime.Users.<u>.Modes`). 2.3 Lowlight and 2.5 Blind are planned.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GrandMasters {
    /// 2.1, 0–1.
    pub master: f64,
    /// 2.1 switched on (off = blackout).
    pub on: bool,
    /// 2.2, 0–1.
    pub highlight: f64,
    /// 2.4, 0–1.
    pub solo: f64,
}

impl Default for GrandMasters {
    fn default() -> Self {
        Self { master: 1.0, on: true, highlight: 1.0, solo: 1.0 }
    }
}

impl PlaybackStates {
    /// The grand master's level (0–1) and whether it is on (off = blackout).
    pub fn grand(&self) -> (f64, bool) {
        let g = &self.live.masters.grand;
        (g.master, g.on)
    }
}

/// A master's level (0–1) and its fade: it moves from `from` to `to` over
/// `fade_ms`, starting at `start_ms` (session clock ms,
/// `lumacore_base::clock::session_ms`; decisions §24 — never wall clock).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MasterLevel {
    pub from: f64,
    pub to: f64,
    #[serde(default)]
    pub start_ms: u64,
    #[serde(default)]
    pub fade_ms: u64,
}

impl Default for MasterLevel {
    fn default() -> Self {
        Self::snap(1.0)
    }
}

impl MasterLevel {
    pub fn snap(level: f64) -> Self {
        Self {
            from: level,
            to: level,
            start_ms: 0,
            fade_ms: 0,
        }
    }

    /// The level at `now_ms`.
    pub fn at(&self, now_ms: u64) -> f64 {
        if self.fade_ms == 0 || now_ms >= self.start_ms.saturating_add(self.fade_ms) {
            return self.to;
        }
        let t = now_ms.saturating_sub(self.start_ms) as f64 / self.fade_ms as f64;
        self.from + (self.to - self.from) * t
    }

    /// Still moving at `now_ms`.
    pub fn fading(&self, now_ms: u64) -> bool {
        self.fade_ms > 0
            && now_ms < self.start_ms.saturating_add(self.fade_ms)
            && self.from != self.to
    }

    /// Where a saved master stands after a show load: its target, settled.
    /// `start_ms` is a session-clock time of the station that saved it and
    /// means nothing here (R1-03).
    pub fn settled(&self) -> Self {
        Self::snap(self.to)
    }

    /// Start moving to `to` over `fade_ms` from where it is at `now_ms`.
    pub fn go_to(&mut self, to: f64, fade_ms: u64, now_ms: u64) {
        *self = Self {
            from: self.at(now_ms),
            to,
            start_ms: now_ms,
            fade_ms,
        };
    }
}

/// Wall-clock ms (the one copy is the base's, decisions §34 25.14).
pub use lumacore_base::clock::now_ms;

/// A running sequence (its current cue and user).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceState {
    /// Cue key the sequence is in.
    pub current_cue: u32,
    /// The user who started it (GUID link).
    pub started_by: Uuid,
    /// When the sequence last asserted all its tracked values (start, Goto,
    /// show load). LTP stamp of values stored before the cues in `go_stamps`.
    pub asserted: u64,
    /// When each cue was reached by Go since the last assert (LTP stamps).
    #[serde(default)]
    pub go_stamps: BTreeMap<u32, u64>,
    #[serde(default = "default_level")]
    pub fader: f64,
    #[serde(default = "default_rate")]
    pub rate: f64,
    #[serde(default)]
    pub paused: bool,
}

impl SequenceState {
    /// Start (or Goto) at `cue`: everything asserted now.
    pub fn asserted_at(cue: u32, user: Uuid, stamp: u64) -> Self {
        Self {
            current_cue: cue,
            started_by: user,
            asserted: stamp,
            go_stamps: BTreeMap::new(),
            fader: 1.0,
            rate: 1.0,
            paused: false,
        }
    }

    /// Goto `cue` while running: every tracked value is asserted again;
    /// fader, rate, paused and who started it stay (R1-30).
    pub fn assert_at(&mut self, cue: u32, stamp: u64) {
        self.current_cue = cue;
        self.asserted = stamp;
        self.go_stamps.clear();
    }

    /// Go to `cue` from the current one (tracked values keep their stamps).
    pub fn go_to(&mut self, cue: u32, stamp: u64) {
        if cue <= self.current_cue {
            // Backwards or wrapping: the whole state is asserted again.
            self.asserted = stamp;
            self.go_stamps.clear();
        } else {
            self.go_stamps.insert(cue, stamp);
        }
        self.current_cue = cue;
    }

    /// The LTP stamp of a value stored in cue `stored_in`.
    pub fn stamp_of(&self, stored_in: u32) -> u64 {
        self.go_stamps
            .get(&stored_in)
            .copied()
            .unwrap_or(self.asserted)
    }
}

impl PlaybackData {
    /// The executor with this GUID: (number, executor).
    pub fn executor_by_guid(&self, guid: Uuid) -> Option<(u32, &ExecutorObject)> {
        self.executors
            .iter()
            .find(|(_, e)| e.meta.identity.guid == guid)
            .map(|(&n, e)| (n, e))
    }
}

// ── Identity (decisions §21 "6d") ───────────────────────────────────────────

impl Identified for SequenceObject {
    fn meta(&self) -> &lumacore_base::NodeMeta {
        &self.meta
    }
    fn kind(&self) -> &'static str {
        "Sequence"
    }
}

lumacore_base::meta_identified! {
    CueObject => "Cue",
    CuePart => "Part",
    SequenceAssignment => "Assignment",
    Effect => "Effect",
    TimingPreset => "TimingPreset",
    CurvePreset => "CurvePreset",
    ExecutorObject => "Executor",
}

impl Walk for PlaybackData {
    fn walk(&self, w: &mut Walker) {
        let Self {
            effects,
            executors,
            park,
        } = self;
        w.pool("Effects", effects);
        w.pool("Executors", executors);
        w.child("Park", park);
    }
}

impl Walk for ParkData {
    fn walk(&self, w: &mut Walker) {
        let Self { fixtures, channels } = self;
        for p in fixtures {
            let ParkedValue {
                fixture_id,
                attribute: _,
                value,
            } = p;
            w.link("Park.Fixture", "Fixture", *fixture_id);
            value.walk(w);
        }
        for c in channels {
            let ParkedChannel {
                universe,
                address: _,
                value: _,
            } = c;
            w.link("Park.Universe", "Universe", *universe);
        }
    }
}

impl Walk for SequenceObject {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            cue_zero,
            off_cue,
            cues,
            assignments,
            settings,
        } = self;
        meta.walk_links(w);
        w.child("CueZero", cue_zero);
        w.child("OffCue", off_cue);
        w.pool("Cues", cues);
        w.pool("Assignments", assignments);
        w.data(settings);
    }
}

impl Walk for SequenceAssignment {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            executor,
        } = self;
        meta.walk_links(w);
        w.link_opt("Executor", "Executor", *executor);
    }
}

impl Walk for CueObject {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            trigger,
            parts,
            blocked: _,
            mib_preference,
        } = self;
        meta.walk_links(w);
        w.data(trigger);
        w.data(mib_preference);
        w.pool("Parts", parts);
    }
}

impl Walk for CuePart {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            fade_in: _,
            fade_out: _,
            delay_in: _,
            delay_out: _,
            values,
            group_timing,
        } = self;
        meta.walk_links(w);
        for (g, t) in group_timing {
            w.link("GroupTiming", "FeatureGroup", *g);
            w.data(t);
        }
        for v in values {
            let CueValue {
                fixture_id,
                attribute: _,
                value,
                timing: _,
            } = v;
            w.link("Values.Fixture", "Fixture", *fixture_id);
            match value {
                CueData::Value(v) => v.walk(w),
                // A cue uses the preset it links to (decisions 0293): its "Used by".
                CueData::Link(g) => w.link("Values.Preset", "Preset", *g),
            }
        }
    }
}

impl Walk for Effect {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            effect_type: _,
            rate: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for TimingPreset {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            fade_in: _,
            fade_out: _,
            delay: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for CurvePreset {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            curve_type: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for ExecutorObject {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            assigned_sequence,
            page: _,
            fader_position: _,
        } = self;
        meta.walk_links(w);
        w.link_opt("Sequence", "Sequence", *assigned_sequence);
    }
}

impl Walk for PlaybackStates {
    fn walk(&self, w: &mut Walker) {
        let Self { live, echoes } = self;
        let EchoPlaybacks { sequences: echo_runs } = echoes;
        for (seq, runs) in echo_runs {
            w.link("EchoOf", "Sequence", *seq);
            for e in runs {
                w.link("StartedBy", "User", e.started_by);
            }
        }
        let LivePlaybacks { sequences, masters } = live;
        let Masters { grand: _, groups } = masters;
        for group in groups.keys() {
            w.link("GroupMaster", "Group", *group);
        }
        for (seq, st) in sequences {
            let SequenceState {
                current_cue: _,
                started_by,
                asserted: _,
                go_stamps: _,
                fader: _,
                rate: _,
                paused: _,
            } = st;
            w.link("Sequence", "Sequence", *seq);
            w.link("StartedBy", "User", *started_by);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_master_fade_near_the_end_of_time_never_overflows() {
        let mut m = MasterLevel::snap(1.0);
        m.go_to(0.5, u64::MAX, u64::MAX - 10);
        assert!(m.fading(u64::MAX - 5));
        assert!((0.5..=1.0).contains(&m.at(u64::MAX)));
        let late = MasterLevel {
            from: 0.0,
            to: 1.0,
            start_ms: u64::MAX,
            fade_ms: u64::MAX,
        };
        assert_eq!(late.at(0), 0.0);
        assert!(!late.fading(u64::MAX));
        assert_eq!(late.settled(), MasterLevel::snap(1.0));
    }

    fn seq_with(values: &[(u32, Uuid, &str, Value)]) -> SequenceObject {
        let mut seq = SequenceObject::default();
        for (cue, fixture, attr, value) in values {
            seq.cues
                .entry(*cue)
                .or_insert_with(|| CueObject::new(""))
                .part0()
                .values
                .push(CueValue {
                    fixture_id: *fixture,
                    attribute: attr.to_string(),
                    timing: Default::default(),
                    value: CueData::Value(value.clone()),
                });
        }
        seq
    }

    #[test]
    fn values_track_until_changed_or_released() {
        let f = Uuid::new_v4();
        let seq = seq_with(&[
            (1000, f, "Dimmer", Value::normalized(50.0)),
            (1000, f, "Pan", Value::normalized(10.0)),
            (2000, f, "Dimmer", Value::normalized(80.0)),
            (3000, f, "Pan", Value::release()),
        ]);
        let at2 = seq.tracked(2000);
        assert_eq!(at2[&(f, "Dimmer")].value.plain().and_then(Value::percent), Some(80.0));
        assert_eq!(at2[&(f, "Pan")].cue, 1000);
        let at3 = seq.tracked(3000);
        assert!(!at3.contains_key(&(f, "Pan")));
        // Tracking off: only the cue's own values.
        let mut flat = seq.clone();
        flat.settings.tracking = false;
        assert!(!flat.tracked(2000).contains_key(&(f, "Pan")));
    }

    #[test]
    fn cue_numbers_read_like_a_console() {
        assert_eq!(cue_key("1.5"), Some(1500));
        assert_eq!(cue_label(1500), "1.5");
        assert_eq!(cue_label(2000), "2");
        assert_eq!(cue_key("0"), None);
        assert_eq!(cue_key("2.25"), Some(2250));
        assert_eq!(cue_key(".5"), Some(500));
        assert_eq!(cue_key("1.500"), Some(1500));
        assert_eq!(cue_key("3"), Some(3000));
        // More than 3 decimals never rounds into another cue (R1-30).
        assert_eq!(cue_key("1.9999"), None);
        assert_eq!(cue_key("1.0004"), None);
        for bad in ["", ".", "-1", "+1", "1e3", "nan", "inf", "1.2.3", "4000000", "0.000"] {
            assert_eq!(cue_key(bad), None, "{bad}");
        }
    }

    #[test]
    fn go_stamps_keep_tracked_values_older() {
        let mut st = SequenceState::asserted_at(1000, Uuid::new_v4(), 10);
        st.go_to(2000, 20);
        assert_eq!(st.stamp_of(1000), 10);
        assert_eq!(st.stamp_of(2000), 20);
        st.go_to(1000, 30);
        assert_eq!(st.stamp_of(2000), 30);
    }
}
