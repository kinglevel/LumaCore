//! lumacore-timecode: Timecode data model types for LumaCore.
//!
//! Provides timecode items, tracks, clips, regions, markers,
//! events, tempo/signature entries, and loop regions.

use lumacore_base::node::{NodeMeta, TreeNode};
use lumacore_base::{Identified, Walk, Walker};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// The authored timing base for a timecode timeline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TimeBase {
    #[default]
    Frames,
    Beats,
    Ticks,
    Smpte,
}

/// A single timed event inside a timecode track.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimecodeEvent {
    pub tick: u64,
    pub event_type: String,
    pub data: Vec<u8>,
}

/// Typed event collections inside a timecode track.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackEvents {
    #[serde(default)]
    pub note_events: Vec<TimecodeEvent>,
    #[serde(default)]
    pub control_change_events: Vec<TimecodeEvent>,
    #[serde(default)]
    pub pitch_bend_events: Vec<TimecodeEvent>,
    #[serde(default)]
    pub program_change_events: Vec<TimecodeEvent>,
    #[serde(default)]
    pub meta_events: Vec<TimecodeEvent>,
}

/// A reusable timed content container inside a track.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimecodeClip {
    pub meta: NodeMeta,
    pub start_tick: u64,
    pub end_tick: u64,
}

/// A placed timed span inside a track.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimecodeRegion {
    pub meta: NodeMeta,
    pub start_tick: u64,
    pub end_tick: u64,
}

/// An ordered timed lane inside a Timecode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimecodeTrack {
    pub meta: NodeMeta,
    #[serde(default)]
    pub events: TrackEvents,
    #[serde(default)]
    pub clips: Vec<TimecodeClip>,
    #[serde(default)]
    pub regions: Vec<TimecodeRegion>,
}

/// A marker on the timeline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimecodeMarker {
    pub meta: NodeMeta,
    pub tick: u64,
}

/// A tempo change on the timeline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TempoEntry {
    pub tick: u64,
    pub bpm: f64,
}

/// A time signature change on the timeline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureEntry {
    pub tick: u64,
    pub numerator: u8,
    pub denominator: u8,
}

/// The authored loop span of a timecode timeline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoopRegion {
    pub start_tick: u64,
    pub end_tick: u64,
    #[serde(default)]
    pub enabled: bool,
}

/// A stored timecode object — authored timeline structure in ShowData.
///
/// Contains tracks, markers, tempo/signature data, loop region, duration,
/// and time base. Runtime transport state belongs to the Timecode Engine,
/// not here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimecodeItem {
    pub meta: NodeMeta,
    #[serde(default)]
    pub tracks: BTreeMap<u32, TimecodeTrack>,
    #[serde(default)]
    pub marker_track: Vec<TimecodeMarker>,
    #[serde(default)]
    pub tempo_track: Vec<TempoEntry>,
    #[serde(default)]
    pub signature_track: Vec<SignatureEntry>,
    #[serde(default)]
    pub loop_region: LoopRegion,
    #[serde(default)]
    pub duration_ticks: u64,
    #[serde(default)]
    pub time_base: TimeBase,
}

impl TreeNode for TimecodeItem {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "Timecode"
    }
}

// ── Identity (decisions §21 "6d") ───────────────────────────────────────────

lumacore_base::meta_identified! {
    TimecodeTrack => "TimecodeTrack",
    TimecodeClip => "TimecodeClip",
    TimecodeRegion => "TimecodeRegion",
    TimecodeMarker => "TimecodeMarker",
}

impl Identified for TimecodeItem {
    fn meta(&self) -> &lumacore_base::NodeMeta {
        &self.meta
    }
    fn kind(&self) -> &'static str {
        "Timecode"
    }
}

impl Walk for TimecodeItem {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            tracks,
            marker_track,
            tempo_track,
            signature_track,
            loop_region,
            duration_ticks: _,
            time_base,
        } = self;
        meta.walk_links(w);
        w.pool("Tracks", tracks);
        w.list("Markers", marker_track);
        w.data(tempo_track);
        w.data(signature_track);
        w.data(loop_region);
        w.data(time_base);
    }
}

impl Walk for TimecodeTrack {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            events,
            clips,
            regions,
        } = self;
        meta.walk_links(w);
        w.data(events);
        w.list("Clips", clips);
        w.list("Regions", regions);
    }
}

macro_rules! leaf_walk {
    ($($t:ident { $($f:ident),* }),* $(,)?) => {$(
        impl Walk for $t {
            fn walk(&self, w: &mut Walker) {
                let Self { meta, $($f: _),* } = self;
                meta.walk_links(w);
            }
        }
    )*};
}

leaf_walk! {
    TimecodeClip { start_tick, end_tick },
    TimecodeRegion { start_tick, end_tick },
    TimecodeMarker { tick },
}
