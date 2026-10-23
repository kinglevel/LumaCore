// ██████╗ ██╗      █████╗ ███╗   ██╗
// ██╔══██╗██║     ██╔══██╗████╗  ██║
// ██████╔╝██║     ███████║██╔██╗ ██║
// ██╔═══╝ ██║     ██╔══██║██║╚██╗██║
// ██║     ███████╗██║  ██║██║ ╚████║
// ╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝
// FILE: plan

//! The render plan (decisions §24 "Step 9 hardening"): the show compiled
//! into flat arrays so the 60 Hz tick never touches a map or a string.
//!
//! Two tiers, built on the compiler thread (`crate::compiler`):
//!
//! - **Channel tier** — the patch: universes, fixtures, fixture attributes
//!   and every DMX channel function as an index-addressed slot (universe
//!   index, address, bytes, default, physical range). Rebuilt only when the
//!   patch changes (`EngineInput::patch_generation`), like a live
//!   patch's realtime channels.
//! - **Value tier** — what the channels show: the merge of programmers,
//!   running cues, park and Freeze/Blind (`playback::resolve`) as one
//!   fraction + owner per channel, the group masters acting on each fixture
//!   (levels fade in the tick), the grand master, Solo and raw parked
//!   channels. Rebuilt on every change.
//!
//! The tick (`crate::render`) only walks these arrays.
//!
//! Owns:          Plan and its two tiers: ChannelTier (the patch as slots), ValueTier (values, fades, masters, echoes).
//! Doesn't own:   the merge (merge), the compiler thread (compiler), the tick (render).
//! Entry points:  Plan, ChannelTier, ValueTier, Fade, index_map.
//! Decisions:     §24 "Step 9 hardening", §29, §30 "12f".

use std::collections::HashMap;
use std::sync::Arc;

use lumacore_fixtures::fixture_type::AttributeType;
use lumacore_tree::tree::ShowData;
use lumacore_tree::{GroupMasterMode, Value};
use lumacore_fixtures::{MasterReact};
use lumacore_protocols::{MergeMode};
use uuid::Uuid;

use crate::merge::{EngineInput, Owner, Resolved, Source, channels, default_fraction, fraction};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A fixture in the plan.
#[derive(Debug, Clone)]
pub struct FixtureSlot {
    pub guid: Uuid,
    pub react: MasterReact,
}

/// One DMX channel function of a patched fixture.
#[derive(Debug, Clone, Copy)]
pub struct ChannelSlot {
    /// Index into `ChannelTier::attrs`.
    pub attr: u32,
    /// Index into `ChannelTier::fixtures`.
    pub fixture: u32,
    /// Index into `ChannelTier::universes`.
    pub universe: u16,
    /// 0-based slot of the first byte.
    pub at: u16,
    /// 1–4 bytes, big-endian.
    pub bytes: u8,
    /// Intensity: masters and Solo act on it.
    pub dimmer: bool,
}

/// The patch, compiled.
#[derive(Debug, Default, Clone)]
pub struct ChannelTier {
    pub patch_generation: u64,
    /// Universe numbers, sorted (index = universe index).
    pub universes: Vec<u32>,
    /// Each universe's merge mode for DMX input.
    pub merge_modes: Vec<MergeMode>,
    pub fixtures: Vec<FixtureSlot>,
    /// Fixture attributes: (fixture GUID, attribute name).
    pub attrs: Vec<(Uuid, String)>,
    /// Channel functions in fixture order.
    pub channels: Vec<ChannelSlot>,
    /// Lookups for the compiler and readers (never the tick).
    pub attr_index: HashMap<(Uuid, String), u32>,
    /// Per fixture (by `fixture_index`), its attributes' range in `attrs` (contiguous): the
    /// merge finds an attribute without building a `(Uuid, String)` key (decisions §31).
    pub fixture_attrs: Vec<std::ops::Range<u32>>,
    pub fixture_index: HashMap<Uuid, u32>,
    pub universe_index: HashMap<Uuid, u16>,
    /// The channel function of every channel slot (fractions, defaults).
    pub functions: Vec<lumacore_fixtures::fixture_type::ChannelFunction>,
    /// Per universe and slot: how many bytes the channel starting there has
    /// (1 for an unpatched slot, 2–4 for a 16/24/32-bit channel), 0 for a
    /// channel's later bytes — DMX input merges whole channels (R1-51).
    pub spans: Vec<[u8; 512]>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl ChannelTier {
    /// Compile the patch of `show`.
    pub fn build(show: &ShowData, patch_generation: u64) -> Self {
        let patch = &show.devices.dmx.patch;
        let universes: Vec<u32> = patch.universes.keys().copied().collect();
        let merge_modes = patch.universes.values().map(|u| u.merge_mode).collect();
        let universe_index: HashMap<Uuid, u16> = patch
            .universes
            .values()
            .enumerate()
            .map(|(i, u)| (u.meta.identity.guid, i as u16))
            .collect();
        let mut tier = Self {
            patch_generation,
            spans: vec![[1u8; 512]; universes.len()],
            universes,
            merge_modes,
            universe_index,
            ..Default::default()
        };
        // An overlap or a run past 512 parks its fixtures: they send nothing until resolved
        // (decisions §34 round 3, 18; their values and programming stay).
        let slots = lumacore_tree::dmx_slots::SlotMap::of(patch);
        for fixture in patch.fixtures.values() {
            let guid = fixture.meta.identity.guid;
            let fixture_no = tier.fixtures.len() as u32;
            tier.fixtures.push(FixtureSlot {
                guid,
                react: fixture.luma.master_react,
            });
            tier.fixture_index.insert(guid, fixture_no);
            let attrs_from = tier.attrs.len() as u32;
            let cfs = channels(show, fixture);
            // A multipatch copy mirrors the fixture's channels at its own address (round 3 19).
            let copies = fixture.luma.multipatch.iter().flat_map(|c| c.breaks.iter().enumerate());
            let runs = fixture.mvr.breaks.iter().enumerate().chain(copies).filter(|_| !slots.is_parked(guid));
            for (break_no, brk) in runs {
                let Some(&universe) = tier.universe_index.get(&brk.universe_id) else {
                    continue;
                };
                // DMX addresses are 1-indexed; an invalid 0 is skipped.
                let Some(base) = (brk.start_address as usize).checked_sub(1) else {
                    continue;
                };
                for cf in cfs {
                    // A channel belongs to its GDTF break (1 = the first).
                    if cf.is_virtual
                        || usize::from(cf.dmx_break.max(1)) != break_no + 1
                        || cf.dmx_offset >= brk.channel_count
                    {
                        continue;
                    }
                    let at = base + cf.dmx_offset as usize;
                    if at >= 512 {
                        continue;
                    }
                    let key = (guid, cf.attribute_name.clone());
                    let attr = match tier.attr_index.get(&key) {
                        Some(&a) => a,
                        None => {
                            let a = tier.attrs.len() as u32;
                            tier.attrs.push(key.clone());
                            tier.attr_index.insert(key, a);
                            a
                        }
                    };
                    let bytes = cf.byte_count.clamp(1, 4);
                    if let Some(span) = tier.spans.get_mut(universe as usize) {
                        let n = usize::from(bytes).min(512 - at);
                        span[at] = n as u8;
                        for b in &mut span[at + 1..at + n] {
                            *b = 0;
                        }
                    }
                    tier.channels.push(ChannelSlot {
                        attr,
                        fixture: fixture_no,
                        universe,
                        at: at as u16,
                        bytes,
                        dimmer: cf.attribute_type == AttributeType::Dimmer,
                    });
                    tier.functions.push(cf.clone());
                }
            }
            tier.fixture_attrs.push(attrs_from..tier.attrs.len() as u32);
        }
        tier
    }

    /// The index in `attrs` of fixture `fixture`'s `attribute`, when it outputs on this tier
    /// (what `attr_index` answers, without a `(Uuid, String)` key).
    pub fn attr_of(&self, fixture: Uuid, attribute: &str) -> Option<u32> {
        let f = *self.fixture_index.get(&fixture)? as usize;
        self.fixture_attrs.get(f)?.clone().find(|&a| self.attrs[a as usize].1 == attribute)
    }

    /// Index of a universe number.
    pub fn universe_no(&self, universe: u32) -> Option<usize> {
        self.universes.binary_search(&universe).ok()
    }
}

/// A channel's value before masters: what the merge gives it.
#[derive(Debug, Clone, Copy)]
pub struct ChannelValue {
    /// 0–1 of the channel's range: where it ends up.
    pub fraction: f64,
    pub owner: Owner,
    /// A cue fade towards `fraction` (decisions §29), evaluated in the tick.
    pub fade: Option<Fade>,
    /// Highlight (decisions §16, §29): the channel's GDTF highlight value
    /// and how strongly it acts (the highlighting users' master).
    pub highlight: Option<(f64, f64)>,
    /// The newest stamp a running sequence offers this channel's attribute
    /// (`Resolved::sequence_stamp`): the next plan fades only for a newer
    /// one (a Go), never on a handover.
    pub sequence_stamp: u64,
}

impl ChannelValue {
    /// The value at `now_ms` (session clock) before Highlight: where a
    /// fade starts from.
    // tidy: render-begin (decisions §24: called by the tick)
    pub fn base_at(&self, now_ms: u64) -> f64 {
        match &self.fade {
            Some(f) => f.at(self.fraction, now_ms),
            None => self.fraction,
        }
    }

    /// The value at `now_ms` (session clock).
    pub fn at(&self, now_ms: u64) -> f64 {
        let f = self.base_at(now_ms);
        match self.highlight {
            Some((target, strength)) if self.owner != Owner::Park => f + (target - f) * strength,
            _ => f,
        }
    }
    // tidy: render-end
}

/// One echo's intensity on one channel (decisions §32, §30 "12f echoes": intensity HTP across
/// a sequence's live run and its echoes). Worked out from the echo alone — its start, cue and
/// OffCue — so the tick needs nothing else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EchoLevel {
    /// Index into `ValueTier::channels`.
    pub channel: u32,
    /// Where it ends up (0 once its OffCue plays).
    pub to: f64,
    pub fade: Fade,
}

/// A running cue fade of one channel: from `from` towards the channel's
/// fraction, starting after `delay_ms`, over `fade_ms`. A `snap` channel
/// (GDTF `LogicalChannel@Snap`) jumps at the end of its delay.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fade {
    pub from: f64,
    pub start_ms: u64,
    pub delay_ms: u64,
    pub fade_ms: u64,
    pub snap: bool,
    /// Its shape (the feature group's Path; Linear by default).
    pub path: lumacore_tree::FadePath,
}

impl Fade {
    // tidy: render-begin (decisions §24: called by the tick)
    pub fn at(&self, to: f64, now_ms: u64) -> f64 {
        let begin = self.begin_ms();
        if now_ms < begin {
            return self.from;
        }
        if self.snaps() || self.fade_ms == 0 || now_ms >= begin.saturating_add(self.fade_ms) {
            return to;
        }
        let t = (now_ms - begin) as f64 / self.fade_ms as f64;
        self.from + (to - self.from) * self.path.at(t)
    }

    /// Still moving at `now_ms`.
    pub fn running(&self, now_ms: u64) -> bool {
        now_ms < self.end_ms()
    }

    /// When the fade starts moving (after its delay); saturating, so a
    /// huge time never overflows in the tick.
    pub fn begin_ms(&self) -> u64 {
        self.start_ms.saturating_add(self.delay_ms)
    }

    /// When the channel arrives (a snap at the end of its delay).
    pub fn end_ms(&self) -> u64 {
        self.begin_ms().saturating_add(if self.snaps() { 0 } else { self.fade_ms })
    }

    /// A GDTF snap channel or a Snap path: it jumps at the end of its delay.
    fn snaps(&self) -> bool {
        self.snap || self.path == lumacore_tree::FadePath::Snap
    }
    // tidy: render-end
}

/// A group master acting on fixtures; its level fades in the tick.
#[derive(Debug, Clone, Copy)]
pub struct MasterSlot {
    pub mode: GroupMasterMode,
    pub level: lumacore_tree::MasterLevel,
}

/// The values, compiled against a channel tier.
#[derive(Debug, Default)]
pub struct ValueTier {
    /// One per channel slot.
    pub channels: Vec<ChannelValue>,
    /// The winning value of each fixture attribute (index = attr), for
    /// readers (`Park` at the current output, `Info`). None = its default.
    pub attrs: Vec<Option<Resolved>>,
    /// Who owns every DMX slot of our output, per universe.
    pub owners: Vec<[Owner; 512]>,
    /// Grand master level (0 when switched off).
    pub grand: f64,
    pub masters: Vec<MasterSlot>,
    /// Per fixture: the range of `master_links` acting on it.
    pub fixture_masters: Vec<(u32, u32)>,
    /// Indices into `masters`.
    pub master_links: Vec<u32>,
    /// Per fixture: how strongly Solo takes its intensity out (0 = not).
    pub soloed_out: Vec<f64>,
    /// Raw parked DMX channels: (universe index, slot, value).
    pub raw_park: Vec<(u16, u16, u8)>,
    /// Echo intensity layers, sorted by channel: the tick takes the highest (HTP).
    pub echoes: Vec<EchoLevel>,
    /// Identified channels, written over the merged output (decisions §34 round 3).
    pub identify: crate::identify::IdentifyTier,
}

impl ValueTier {
    /// Compile the values of `input` against `tier`. With the plan that
    /// renders now (`previous`; its channels mapped onto `tier` when the
    /// patch changed), a cue's newer value (a Go) fades from where its
    /// channel is at `now_ms` by its part's timing; a fade still running
    /// goes on (retargeted when its value changed); a handover snaps.
    pub fn build(
        input: &EngineInput,
        tier: &ChannelTier,
        previous: Option<&Plan>,
        now_ms: u64,
        cache: &mut crate::merge::SequenceCache<u32>,
    ) -> Self {
        // A rebuilt channel tier (a Devices change): the old→new index map
        // carries running fades across (decisions §24, R1-26).
        let carried: Option<Vec<Option<u32>>> = previous
            .filter(|p| !std::ptr::eq(p.channel.as_ref(), tier))
            .map(|p| index_map(&p.channel, tier));
        let show = &input.show;
        // The merge, keyed by this tier's attribute index (no per-value String key).
        let key_of = |fixture: Uuid, attribute: &str| tier.attr_of(fixture, attribute);
        let released = crate::merge::releases_by(input, key_of);
        // Which attributes are intensity (an echo's are HTP, not LTP).
        let mut intensity = vec![false; tier.attrs.len()];
        for slot in tier.channels.iter().filter(|s| s.dimmer) {
            intensity[slot.attr as usize] = true;
        }
        let mut attrs: Vec<Option<Resolved>> = vec![None; tier.attrs.len()];
        for (i, r) in crate::merge::resolve_cached(input, key_of, |a| intensity[*a as usize], cache) {
            attrs[i as usize] = Some(r);
        }

        // Highlight: each highlighting user's selection, at their strength.
        let mut highlight = vec![0.0f64; tier.fixtures.len()];
        for layer in input.users.iter().filter(|u| u.highlight > 0.0) {
            let Some(user) = show.users.get(&layer.user_no) else {
                continue;
            };
            for g in &user.programmer.selected_fixtures {
                if let Some(&f) = tier.fixture_index.get(g) {
                    let h = &mut highlight[f as usize];
                    *h = h.max(layer.highlight);
                }
            }
        }

        let mut owners = vec![[Owner::Unpatched; 512]; tier.universes.len()];
        let mut channels = Vec::with_capacity(tier.channels.len());
        for (slot, cf) in tier.channels.iter().zip(&tier.functions) {
            let won = attrs[slot.attr as usize].as_ref();
            let owner = match won {
                None => Owner::Default,
                Some(r) if r.source == Source::Park => Owner::Park,
                Some(r) if r.frozen => Owner::Frozen(r.stamp),
                Some(r) => Owner::Ours(r.stamp),
            };
            let fraction = won
                .and_then(|r| fraction(&r.value, cf))
                .unwrap_or_else(|| default_fraction(cf));
            let own = &mut owners[slot.universe as usize];
            for o in own.iter_mut().skip(slot.at as usize).take(slot.bytes as usize) {
                *o = owner;
            }
            let i = channels.len();
            let old_index = match &carried {
                Some(map) => map[i],
                None => Some(i as u32),
            };
            let old = previous
                .zip(old_index)
                .and_then(|(p, o)| p.value.channels.get(o as usize));
            // A release by a Go (the attribute fell back to a lower source
            // than that Go): it fades by the releasing cue's timing (Q12).
            let release = released
                .get(&slot.attr)
                .filter(|(stamp, _)| won.is_none_or(|r| r.stamp < *stamp));
            let sequence_stamp = won.map_or(0, |r| r.sequence_stamp).max(release.map_or(0, |r| r.0));
            let fade = old.and_then(|old| {
                // Where the channel is now, without Highlight.
                let was = old.base_at(now_ms);
                if old.owner == owner {
                    // The same touch: a running fade goes on …
                    let f = old.fade.filter(|f| f.running(now_ms))?;
                    if old.fraction == fraction {
                        return Some(f);
                    }
                    // … retargeted (a Store into the running cue): from
                    // where it is over the time it had left.
                    if (was - fraction).abs() < f64::EPSILON {
                        return None;
                    }
                    let begin = f.begin_ms();
                    return Some(Fade {
                        from: was,
                        start_ms: now_ms,
                        delay_ms: begin.saturating_sub(now_ms),
                        fade_ms: f.end_ms().saturating_sub(now_ms.max(begin)),
                        snap: f.snap,
                        path: f.path,
                    });
                }
                // Only a Go / Goto / Go- of the winning sequence (a stamp
                // newer than any the old plan saw) fades; a handover
                // (Clear, Unpark, Off Freeze, Blind, Off Sequence) snaps.
                let timing = match won.filter(|r| matches!(r.source, Source::Sequence(_)) && r.stamp > old.sequence_stamp) {
                    Some(r) => r.timing?,
                    None => release.filter(|(stamp, _)| *stamp > old.sequence_stamp)?.1,
                };
                if (was - fraction).abs() < f64::EPSILON {
                    return None;
                }
                let (delay_ms, fade_ms) = timing.pick(slot.dimmer && fraction < was);
                (delay_ms > 0 || fade_ms > 0).then_some(Fade {
                    from: was,
                    start_ms: now_ms,
                    delay_ms,
                    fade_ms,
                    snap: cf.snap,
                    path: timing.path,
                })
            });
            // Only GDTF highlight values: a channel without one stays.
            let strength = highlight[slot.fixture as usize];
            let highlight = cf.highlight.filter(|_| strength > 0.0).map(|h| (h, strength));
            channels.push(ChannelValue {
                fraction,
                owner,
                fade,
                highlight,
                sequence_stamp,
            });
        }

        let (grand_level, grand_on) = show.playbacks.grand();
        let grand = if grand_on { grand_level } else { 0.0 };

        // Group masters: one slot per mastering group, linked per fixture.
        let mut masters = Vec::new();
        let mut per_fixture: Vec<Vec<u32>> = vec![Vec::new(); tier.fixtures.len()];
        for group in show.datapools.values().flat_map(|dp| dp.groups.values()) {
            if group.master == GroupMasterMode::None {
                continue;
            }
            let level = show
                .playbacks
                .live.masters.groups
                .get(&group.meta.identity.guid)
                .copied()
                .unwrap_or_default();
            let m = masters.len() as u32;
            masters.push(MasterSlot {
                mode: group.master,
                level,
            });
            for member in &group.members {
                if let Some(&f) = tier.fixture_index.get(&member.fixture) {
                    per_fixture[f as usize].push(m);
                }
            }
        }
        let mut fixture_masters = Vec::with_capacity(per_fixture.len());
        let mut master_links = Vec::new();
        for links in per_fixture {
            fixture_masters.push((master_links.len() as u32, links.len() as u32));
            master_links.extend(links);
        }

        let soloing: Vec<&crate::merge::UserLayer> = input.users.iter().filter(|u| u.solo > 0.0).collect();
        let soloed_out = if soloing.is_empty() {
            vec![0.0; tier.fixtures.len()]
        } else {
            let strength = soloing.iter().map(|u| u.solo).fold(0.0, f64::max);
            let selected: std::collections::HashSet<Uuid> = soloing
                .iter()
                .filter_map(|u| show.users.get(&u.user_no))
                .flat_map(|u| u.programmer.selected_fixtures.iter().copied())
                .collect();
            tier.fixtures
                .iter()
                .map(|f| {
                    if f.react != MasterReact::None && !selected.contains(&f.guid) {
                        strength
                    } else {
                        0.0
                    }
                })
                .collect()
        };

        let mut raw_park = Vec::new();
        for ch in &show.playback.park.channels {
            if let (Some(&u), Some(slot)) = (
                tier.universe_index.get(&ch.universe),
                (ch.address as usize).checked_sub(1),
            ) && slot < 512
            {
                raw_park.push((u, slot as u16, ch.value));
                owners[u as usize][slot] = Owner::Park;
            }
        }

        let echoes = echo_levels(show, tier, &intensity);
        let identify = crate::identify::compile(show, tier, &owners, &input.identify);

        Self {
            channels,
            attrs,
            owners,
            grand,
            masters,
            fixture_masters,
            master_links,
            soloed_out,
            raw_park,
            echoes,
            identify,
        }
    }

    /// Whether a master is fading at `now_ms` (for readers; the tick
    /// evaluates every level every frame).
    pub fn fading(&self, now_ms: u64) -> bool {
        self.masters.iter().any(|m| m.level.fading(now_ms))
    }
}

/// For each channel slot of `new`, the slot of `old` that held the same
/// channel: keyed by (fixture GUID, attribute, ordinal of that attribute's
/// slots). Built on the compiler thread when the channel tier is rebuilt, so
/// running fades survive a Devices change (decisions §24).
pub fn index_map(old: &ChannelTier, new: &ChannelTier) -> Vec<Option<u32>> {
    let mut old_slots: Vec<Vec<u32>> = vec![Vec::new(); old.attrs.len()];
    for (i, c) in old.channels.iter().enumerate() {
        old_slots[c.attr as usize].push(i as u32);
    }
    let mut seen = vec![0usize; new.attrs.len()];
    new.channels
        .iter()
        .map(|c| {
            let ordinal = seen[c.attr as usize];
            seen[c.attr as usize] += 1;
            let old_attr = *old.attr_index.get(&new.attrs[c.attr as usize])?;
            old_slots[old_attr as usize].get(ordinal).copied()
        })
        .collect()
}

/// Everything the tick renders from.
#[derive(Debug)]
pub struct Plan {
    /// The show generation this plan was compiled from.
    pub generation: u64,
    pub channel: Arc<ChannelTier>,
    pub value: ValueTier,
    /// How long compiling took (µs): channel tier (0 when reused) + value
    /// tier.
    pub compile_us: (u64, u64),
}

impl Plan {
    /// Compile `input`, reusing `previous`'s channel tier when the patch is
    /// the same.
    pub fn compile(input: &EngineInput, previous: Option<&Arc<ChannelTier>>) -> Self {
        Self::compile_after(input, previous, None, 0)
    }

    /// Compile `input` after `plan` (the one rendering now): cue values
    /// fade from where the channels are at `now_ms` (session clock).
    pub fn compile_after(
        input: &EngineInput,
        previous: Option<&Arc<ChannelTier>>,
        plan: Option<&Plan>,
        now_ms: u64,
    ) -> Self {
        Self::compile_cached(input, previous, plan, now_ms, &mut crate::merge::SequenceCache::default())
    }

    /// [`Self::compile_after`] with the compiler thread's [`SequenceCache`]: what each running
    /// sequence offers is kept while its `Arc` and current cue stay (decisions §31). A rebuilt
    /// channel tier clears it (its attribute indices are the keys).
    ///
    /// [`SequenceCache`]: crate::merge::SequenceCache
    pub fn compile_cached(
        input: &EngineInput,
        previous: Option<&Arc<ChannelTier>>,
        plan: Option<&Plan>,
        now_ms: u64,
        cache: &mut crate::merge::SequenceCache<u32>,
    ) -> Self {
        let started = std::time::Instant::now();
        let channel = match previous {
            Some(p) if p.patch_generation == input.patch_generation => p.clone(),
            _ => {
                cache.clear();
                Arc::new(ChannelTier::build(&input.show, input.patch_generation))
            }
        };
        let channel_us = started.elapsed().as_micros() as u64;
        let value_started = std::time::Instant::now();
        let value = ValueTier::build(input, &channel, plan, now_ms, cache);
        Self {
            generation: input.generation,
            channel,
            value,
            compile_us: (channel_us, value_started.elapsed().as_micros() as u64),
        }
    }

    /// An empty plan (no universes).
    pub fn empty() -> Self {
        Self {
            generation: 0,
            channel: Arc::new(ChannelTier::default()),
            value: ValueTier::default(),
            compile_us: (0, 0),
        }
    }

    /// The winning value of a fixture attribute (None = its default or not
    /// patched). Readers only.
    pub fn resolved(&self, fixture: Uuid, attribute: &str) -> Option<&Resolved> {
        let i = *self.channel.attr_index.get(&(fixture, attribute.to_string()))?;
        self.value.attrs.get(i as usize)?.as_ref()
    }

    /// The value a fixture attribute outputs before masters: the winner, or
    /// the channel's default.
    pub fn current_value(&self, fixture: Uuid, attribute: &str) -> Option<Value> {
        if let Some(r) = self.resolved(fixture, attribute) {
            return Some(r.value.clone());
        }
        let i = *self.channel.attr_index.get(&(fixture, attribute.to_string()))?;
        let at = self.channel.channels.iter().position(|c| c.attr == i)?;
        Some(Value::normalized(default_fraction(&self.channel.functions[at]) * 100.0))
    }
}

/// Every echo's intensity layers (decisions §32, §30 "12f echoes"), sorted by channel. A running
/// echo fades in from CueZero's level (else nothing) to its cue by the cue part's In timing; one
/// whose OffCue plays fades from where it stood to 0 by the OffCue's Out timing.
fn echo_levels(show: &ShowData, tier: &ChannelTier, intensity: &[bool]) -> Vec<EchoLevel> {
    let echoes = &show.playbacks.echoes.sequences;
    if echoes.is_empty() {
        return Vec::new();
    }
    // The channels of each intensity attribute (only built when an echo runs).
    let mut by_attr: HashMap<u32, Vec<u32>> = HashMap::new();
    for (ci, slot) in tier.channels.iter().enumerate() {
        if slot.dimmer {
            by_attr.entry(slot.attr).or_default().push(ci as u32);
        }
    }
    let mut out = Vec::new();
    // Preset links play the preset as it is now (decisions 0293).
    let presets = lumacore_tree::PresetIndex::new(show);
    for (guid, runs) in echoes {
        let Some((_, seq)) = show.sequence_by_guid(*guid).map(|(_, n, s)| (n, s)) else { continue };
        let start = seq.tracked(lumacore_tree::CUE_ZERO);
        let (off_delay, off_fade) = seq.off_cue.parts.get(&0).map_or((0, 0), |p| {
            let ms = |s: f64| (s.max(0.0) * 1000.0).round() as u64;
            (ms(p.delay_out), ms(p.fade_out))
        });
        for echo in runs {
            for ((fixture, attr), t) in seq.tracked(echo.cue) {
                let Some(a) = tier.attr_of(fixture, attr) else { continue };
                if !intensity.get(a as usize).copied().unwrap_or(false) {
                    continue;
                }
                let timing = seq
                    .cue(t.cue)
                    .and_then(|c| c.parts.get(&t.part))
                    .map(|p| crate::merge::value_timing(p, t.index, attr, show))
                    .unwrap_or_default();
                for &ci in by_attr.get(&a).map(Vec::as_slice).unwrap_or_default() {
                    let cf = &tier.functions[ci as usize];
                    let to = presets.resolve(t.value, fixture, attr).and_then(|v| fraction(v, cf)).unwrap_or(0.0);
                    let from = start
                        .get(&(fixture, attr))
                        .and_then(|z| presets.resolve(z.value, fixture, attr))
                        .and_then(|v| fraction(v, cf))
                        .unwrap_or(0.0);
                    let rising = Fade { from, start_ms: echo.started_ms, delay_ms: timing.delay_in, fade_ms: timing.fade_in, snap: false, path: timing.path };
                    out.push(match echo.off_ms {
                        None => EchoLevel { channel: ci, to, fade: rising },
                        Some(off) => EchoLevel {
                            channel: ci,
                            to: 0.0,
                            fade: Fade { from: rising.at(to, off), start_ms: off, delay_ms: off_delay, fade_ms: off_fade, snap: false, path: timing.path },
                        },
                    });
                }
            }
        }
    }
    out.sort_by_key(|l| l.channel);
    out
}
