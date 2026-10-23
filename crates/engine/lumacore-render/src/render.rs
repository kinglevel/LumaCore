// ██████╗ ███████╗███╗   ██╗██████╗ ███████╗██████╗     ████████╗██╗ ██████╗██╗  ██╗
// ██╔══██╗██╔════╝████╗  ██║██╔══██╗██╔════╝██╔══██╗    ╚══██╔══╝██║██╔════╝██║ ██╔╝
// ██████╔╝█████╗  ██╔██╗ ██║██║  ██║█████╗  ██████╔╝       ██║   ██║██║     █████╔╝
// ██╔══██╗██╔══╝  ██║╚██╗██║██║  ██║██╔══╝  ██╔══██╗       ██║   ██║██║     ██╔═██╗
// ██║  ██║███████╗██║ ╚████║██████╔╝███████╗██║  ██║       ██║   ██║╚██████╗██║  ██╗
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝       ╚═╝   ╚═╝ ╚═════╝╚═╝  ╚═╝
// FILE: render tick

//! The render tick (decisions §24 "Step 9 hardening"): a compiled `Plan` +
//! the session clock + DMX input → one `RenderedFrame`.
//!
//! Audio-style rules: no allocation, no lock, no logging, no map or string
//! lookup — only the plan's arrays and a preallocated frame buffer (debug
//! builds check the allocation rule, `crate::engine`). Masters fade here,
//! every frame, from the session clock.
//!
//! Owns:          the tick: render_into, render_now, RenderedFrame, Stages.
//! Doesn't own:   the plan (plan), the thread that runs it (engine).
//! Entry points:  render_into, render_now, RenderedFrame, Stages, NoStages.
//! Decisions:     §24 "Step 9 hardening", §27.

use arrayvec::ArrayVec;
use lumacore_tree::{GroupMasterMode};
use lumacore_fixtures::{MasterReact};
use lumacore_protocols::{InputPriority, MergeMode};

use lumacore_protocols::input::InputLayer;
use crate::plan::{ChannelTier, Plan};
use crate::merge::{Owner, Winner};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Input layers merged per universe at most (more are ignored).
pub const MAX_LAYERS_PER_UNIVERSE: usize = 32;

/// One rendered frame: every universe of the plan, our output, the final
/// output (DMX input merged in) and who won each channel.
#[derive(Debug, Clone)]
pub struct RenderedFrame {
    /// Show generation of the plan it was rendered from.
    pub generation: u64,
    /// Frame counter of the engine (0 = not rendered by the engine).
    pub frame_no: u64,
    /// Session-clock time it was rendered for (ms).
    pub time_ms: u64,
    universes: Vec<u32>,
    ours: Vec<[u8; 512]>,
    out: Vec<[u8; 512]>,
    winners: Vec<[Winner; 512]>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Default for RenderedFrame {
    fn default() -> Self {
        Self::for_tier(&ChannelTier::default())
    }
}

impl RenderedFrame {
    /// A blank frame shaped for `tier`'s universes (allocates; the
    /// compiler builds frame pools with it).
    pub fn for_tier(tier: &ChannelTier) -> Self {
        let n = tier.universes.len();
        Self {
            generation: 0,
            frame_no: 0,
            time_ms: 0,
            universes: tier.universes.clone(),
            ours: vec![[0; 512]; n],
            out: vec![[0; 512]; n],
            winners: vec![[Winner::Unpatched; 512]; n],
        }
    }

    fn index(&self, universe: u32) -> Option<usize> {
        self.universes.binary_search(&universe).ok()
    }

    /// The final output of a universe.
    pub fn get(&self, universe: u32) -> Option<&[u8; 512]> {
        self.index(universe).map(|i| &self.out[i])
    }

    /// Our own output of a universe (before DMX input).
    pub fn ours(&self, universe: u32) -> Option<&[u8; 512]> {
        self.index(universe).map(|i| &self.ours[i])
    }

    /// Who won each channel of a universe.
    pub fn winners(&self, universe: u32) -> Option<&[Winner; 512]> {
        self.index(universe).map(|i| &self.winners[i])
    }

    pub fn contains(&self, universe: u32) -> bool {
        self.index(universe).is_some()
    }

    pub fn universes(&self) -> &[u32] {
        &self.universes
    }

    pub fn is_empty(&self) -> bool {
        self.universes.is_empty()
    }

    /// (universe, final output) in universe order.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &[u8; 512])> {
        self.universes.iter().copied().zip(self.out.iter())
    }

    /// Whether this buffer fits `tier` (same universes).
    pub fn fits(&self, tier: &ChannelTier) -> bool {
        self.universes == tier.universes
    }
}

// tidy: render-begin (decisions §24: the per-frame path)
/// What the group masters do to one fixture's intensity.
#[derive(Debug, Clone, Copy)]
struct Masters {
    positive: Option<f64>,
    negative: Option<f64>,
    scale: f64,
    additive: Option<f64>,
}

impl Masters {
    const NONE: Self = Self {
        positive: None,
        negative: None,
        scale: 1.0,
        additive: None,
    };

    /// Intensity `f` after the masters (`grand` already includes on/off).
    fn apply(&self, f: f64, grand: f64, react: MasterReact) -> f64 {
        let mut g = f * self.scale;
        // Negative has priority over Positive where both act.
        if let Some(limit) = self.negative.or(self.positive) {
            g = g.min(limit);
        }
        let add = self.additive.unwrap_or(0.0);
        match react {
            MasterReact::None => f,
            MasterReact::Grand => g.max(add) * grand,
            // Additive group masters skip the grand master.
            MasterReact::Group => (g * grand).max(add),
        }
    }
}

/// The masters acting on fixture `f` at `now_ms`.
fn masters_of(plan: &Plan, f: usize, now_ms: u64) -> Masters {
    let v = &plan.value;
    let Some(&(start, len)) = v.fixture_masters.get(f) else {
        return Masters::NONE;
    };
    let mut m = Masters::NONE;
    for &link in &v.master_links[start as usize..(start + len) as usize] {
        let slot = &v.masters[link as usize];
        let level = slot.level.at(now_ms).clamp(0.0, 1.0);
        match slot.mode {
            GroupMasterMode::Positive => m.positive = Some(m.positive.map_or(level, |p| p.max(level))),
            GroupMasterMode::Negative => m.negative = Some(m.negative.map_or(level, |n| n.min(level))),
            GroupMasterMode::Scaling => m.scale *= level,
            GroupMasterMode::Additive => m.additive = Some(m.additive.map_or(level, |a| a.max(level))),
            GroupMasterMode::None => {}
        }
    }
    m
}

/// Write `fraction` into `bytes` big-endian (coarse, fine, ultra …).
#[inline]
pub(crate) fn write_channel(buf: &mut [u8; 512], at: usize, bytes: usize, fraction: f64) {
    let max = (1u64 << (8 * bytes)) - 1;
    let v = (fraction.clamp(0.0, 1.0) * max as f64).round() as u64;
    for i in 0..bytes {
        if let Some(slot) = buf.get_mut(at + i) {
            *slot = (v >> (8 * (bytes - 1 - i))) as u8;
        }
    }
}

/// Stage boundaries reported to the caller (`crate::engine` times them).
pub trait Stages {
    fn stage(&mut self, index: usize);
}

/// No timing (tests, one-off renders).
pub struct NoStages;

impl Stages for NoStages {
    fn stage(&mut self, _: usize) {}
}

/// Stage indices (`lumacore_runtime::render_stats::STAGES`). Group masters
/// fade inside Channels (worked out once per fixture).
pub mod stage {
    pub const PLAN: usize = 0;
    pub const INPUTS: usize = 1;
    pub const CHANNELS: usize = 2;
    pub const PARK: usize = 3;
    pub const INPUT_MERGE: usize = 4;
    pub const PUBLISH: usize = 5;
}

/// Render `plan` at `now_ms` into `frame` (shaped for the plan's channel
/// tier), merging `layers` of DMX input. Allocates nothing.
pub fn render_into<'a>(
    frame: &mut RenderedFrame,
    plan: &Plan,
    now_ms: u64,
    layers: impl Iterator<Item = &'a InputLayer> + Clone,
    stages: &mut impl Stages,
) {
    let tier = &*plan.channel;
    let v = &plan.value;
    frame.generation = plan.generation;
    frame.time_ms = now_ms;
    for buf in frame.ours.iter_mut() {
        buf.fill(0);
    }

    stages.stage(stage::CHANNELS);
    // Channels come in fixture order: the masters are worked out once per
    // fixture.
    let mut current_fixture = u32::MAX;
    let mut masters = Masters::NONE;
    let mut react = MasterReact::None;
    let mut soloed_out = 0.0;
    // Echo intensity layers come sorted by channel (decisions §32): one cursor.
    let mut echo = 0usize;
    for (ci, (slot, value)) in tier.channels.iter().zip(&v.channels).enumerate() {
        let mut f = value.at(now_ms);
        while v.echoes.get(echo).is_some_and(|e| (e.channel as usize) < ci) {
            echo += 1;
        }
        if slot.dimmer && value.owner != Owner::Park {
            // HTP: the live run's (or the merge's) intensity against every echo's.
            while let Some(e) = v.echoes.get(echo).filter(|e| e.channel as usize == ci) {
                f = f.max(e.fade.at(e.to, now_ms));
                echo += 1;
            }
            if slot.fixture != current_fixture {
                current_fixture = slot.fixture;
                let fi = slot.fixture as usize;
                masters = masters_of(plan, fi, now_ms);
                react = tier.fixtures[fi].react;
                soloed_out = v.soloed_out.get(fi).copied().unwrap_or(0.0);
            }
            f = masters.apply(f, v.grand, react);
            f *= 1.0 - soloed_out;
        }
        if let Some(buf) = frame.ours.get_mut(slot.universe as usize) {
            write_channel(buf, slot.at as usize, slot.bytes as usize, f);
        }
    }

    stages.stage(stage::PARK);
    for &(u, at, value) in &v.raw_park {
        if let Some(slot) = frame.ours.get_mut(u as usize).and_then(|b| b.get_mut(at as usize)) {
            *slot = value;
        }
    }

    stages.stage(stage::INPUT_MERGE);
    for (i, &universe) in tier.universes.iter().enumerate() {
        let (Some(ours), Some(out), Some(won), Some(owners)) = (
            frame.ours.get(i),
            frame.out.get_mut(i),
            frame.winners.get_mut(i),
            v.owners.get(i),
        ) else {
            continue;
        };
        *out = *ours;
        for (w, o) in won.iter_mut().zip(owners.iter()) {
            *w = match o {
                Owner::Unpatched => Winner::Unpatched,
                Owner::Default => Winner::Default,
                Owner::Ours(_) | Owner::Frozen(_) => Winner::Ours,
                Owner::Park => Winner::Park,
            };
        }
        let mut here: ArrayVec<&InputLayer, MAX_LAYERS_PER_UNIVERSE> = ArrayVec::new();
        for l in layers.clone().filter(|l| l.universe == universe) {
            if here.try_push(l).is_err() {
                break;
            }
        }
        if here.is_empty() {
            continue;
        }
        let mode = tier.merge_modes.get(i).copied().unwrap_or_default();
        let Some(spans) = tier.spans.get(i) else {
            continue;
        };
        merge_universe(out, won, owners, spans, mode, &here);
    }
    // Identify: above the merged output, input included; Park's slots were left out (§34 round 3).
    crate::identify::apply(&mut frame.out, &v.identify, now_ms);
}

/// Whether input at `band` (changed at `stamp`) takes a channel owned by
/// `owner`, in a Prio universe. Until the Priority-pool bands exist the
/// bands map onto today's layers (decisions §11 "Step 4c hardening").
fn input_wins(band: InputPriority, stamp: u64, owner: Owner) -> bool {
    use InputPriority::*;
    match owner {
        Owner::Park => false,
        Owner::Unpatched | Owner::Default => true,
        Owner::Frozen(ours) => match band {
            Super => true,
            Prog => stamp > ours,
            _ => false,
        },
        Owner::Ours(ours) => match band {
            Super | Prog | Highest | High => true,
            Ltp => stamp > ours,
            Low | Lowest => false,
        },
    }
}

/// Merge one universe's input layers into our output, per channel (a
/// 16-bit channel's bytes come from one source: R1-51), by the universe's
/// merge mode.
fn merge_universe(
    out: &mut [u8; 512],
    won: &mut [Winner; 512],
    owners: &[Owner; 512],
    spans: &[u8; 512],
    mode: MergeMode,
    layers: &[&InputLayer],
) {
    // The highest band present (lowest enum value); inside a band, HTP and
    // the latest change between lines.
    let Some(band) = layers.iter().map(|l| l.band).min() else {
        return;
    };
    // A channel's bytes as one big-endian number (≤ 4 bytes).
    let value = |data: &[u8; 512], at: usize, n: usize| data[at..at + n].iter().fold(0u32, |v, b| v << 8 | u32::from(*b));
    let mut i = 0;
    while i < 512 {
        let n = usize::from(spans[i].max(1)).min(512 - i);
        // Park on any byte of the channel (a parked fine byte) keeps it all.
        let owner = if owners[i..i + n].contains(&Owner::Park) { Owner::Park } else { owners[i] };
        match mode {
            MergeMode::Off => {}
            MergeMode::Htp | MergeMode::LowTp => {
                if owner != Owner::Park {
                    let htp = mode == MergeMode::Htp;
                    for l in layers {
                        let (v, now) = (value(&l.data, i, n), value(out, i, n));
                        if (htp && v > now) || (!htp && v < now) {
                            out[i..i + n].copy_from_slice(&l.data[i..i + n]);
                            for w in &mut won[i..i + n] {
                                *w = Winner::Input(l.line);
                            }
                        }
                    }
                }
            }
            MergeMode::Prio => {
                let mut best: Option<&InputLayer> = None;
                let mut stamp = 0;
                for l in layers.iter().filter(|l| l.band == band) {
                    let l_stamp = l.stamps[i..i + n].iter().copied().max().unwrap_or(0);
                    stamp = stamp.max(l_stamp);
                    let key = (value(&l.data, i, n), l_stamp);
                    if best.is_none_or(|b| key > (value(&b.data, i, n), b.stamps[i..i + n].iter().copied().max().unwrap_or(0))) {
                        best = Some(l);
                    }
                }
                if let Some(best) = best
                    && input_wins(band, stamp, owner)
                {
                    out[i..i + n].copy_from_slice(&best.data[i..i + n]);
                    for w in &mut won[i..i + n] {
                        *w = Winner::Input(best.line);
                    }
                }
            }
        }
        i += n;
    }
}

// tidy: render-end
/// Render once, here and now, without an engine (tests, a station without
/// the render engine): compile `input` and render it at `now_ms` with no
/// DMX input.
pub fn render_now(input: &crate::merge::EngineInput, now_ms: u64) -> RenderedFrame {
    let plan = Plan::compile(input, None);
    let mut frame = RenderedFrame::for_tier(&plan.channel);
    render_into(&mut frame, &plan, now_ms, std::iter::empty(), &mut NoStages);
    frame
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
