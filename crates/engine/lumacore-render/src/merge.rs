// ███╗   ███╗███████╗██████╗  ██████╗ ███████╗
// ████╗ ████║██╔════╝██╔══██╗██╔════╝ ██╔════╝
// ██╔████╔██║█████╗  ██████╔╝██║  ███╗█████╗
// ██║╚██╔╝██║██╔══╝  ██╔══██╗██║   ██║██╔══╝
// ██║ ╚═╝ ██║███████╗██║  ██║╚██████╔╝███████╗
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝
// FILE: merge

//! The render engine's merge (decisions §27): show snapshot + per-user
//! modes → the winning value of every fixture attribute. The plan compiler
//! (`crate::plan`) runs it off the render thread; the tick (`crate::render`)
//! applies masters, Solo, park and DMX input from the compiled arrays.
//!
//! Merge, per fixture attribute:
//! 1. Park wins over everything.
//! 2. Frozen programmers (Freeze on) win over every playback (no Super
//!    playbacks exist yet).
//! 3. Otherwise programmers and running sequences merge LTP: the latest
//!    touch wins (decisions §4, §7). Blind programmers don't take part.
//! 4. Nothing → the fixture type's default.
//!
//! Then the output modifiers (in the tick): group masters (decisions §22:
//! Scaling multiplies, Negative / Positive limit — Negative wins where they
//! overlap —, Additive adds HTP), the grand master scales intensity
//! (switched off = blackout), Solo zeros the intensity of fixtures outside
//! every soloing user's selection. A fixture's Master React decides which
//! apply (None: no master and no Solo). Neither touches parked values. Raw
//! parked DMX channels are written last.
//!
//! Values are physical or normalized throughout; DMX bytes appear only when
//! a value is patched to its channel (16-bit and wider channels get every
//! byte).
//!
//! DMX input merges last, per DMX channel (decisions §11 "Step 4c
//! hardening"): each channel of our output has an `Owner`, each universe's
//! merge mode decides between that and the input layers (`Winner`). Park
//! always wins; masters and Solo never touch input.
//!
//! Owns:          the one merge: UserLayer, EngineInput, Owner, Winner, Resolved, resolve, releases, value timing.
//! Doesn't own:   the compiled plan (plan), the tick (render), running state (lumacore-tree Playbacks).
//! Entry points:  resolve, resolve_by, resolve_cached, releases, EngineInput, UserLayer, Winner.
//! Decisions:     §27, §29, §32.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

use lumacore_fixtures::fixture_type::ChannelFunction;
use lumacore_tree::tree::ShowData;
use lumacore_tree::{Absolute, Value};
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A user's modes as the engine needs them (from `Runtime.Users`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserLayer {
    /// The user (number) whose programmer this is.
    pub user_no: u32,
    pub freeze: bool,
    pub blind: bool,
    /// How strongly Solo acts (0 = off): the mode times its master
    /// (decisions §16, §29).
    pub solo: f64,
    /// How strongly Highlight acts on this user's selection (0 = off): the
    /// fixtures go towards their GDTF highlight values.
    pub highlight: f64,
}

/// Everything the engine renders from. Published by the state thread.
#[derive(Debug, Clone)]
pub struct EngineInput {
    pub show: Arc<ShowData>,
    /// Logged-in users' modes (users not listed have every mode off).
    pub users: Vec<UserLayer>,
    /// What every user identifies (decisions §34 round 3, "Identify, Probe and Find").
    pub identify: Vec<crate::identify::Identify>,
    /// Counts every publish: frames carry the generation they were
    /// rendered from (a command can wait for its own change).
    pub generation: u64,
    /// Changes when the patch changes: the compiler rebuilds the channel
    /// tier only then.
    pub patch_generation: u64,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl EngineInput {
    pub fn new(show: Arc<ShowData>) -> Self {
        Self {
            show,
            users: Vec::new(),
            identify: Vec::new(),
            generation: 0,
            patch_generation: 0,
        }
    }
}

/// Where a fixture attribute's output comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A user's programmer (user number).
    Programmer(u32),
    /// A running sequence.
    Sequence(u32),
    Park,
}

/// The winning value of a fixture attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    pub value: Value,
    pub source: Source,
    /// When it was touched (LTP against DMX input).
    pub stamp: u64,
    /// From a frozen programmer (above playbacks).
    pub frozen: bool,
    /// A cue value's part timing (programmers and park snap).
    pub timing: Option<Timing>,
    /// The newest stamp any running sequence offers this attribute, won or
    /// not (0 = none): a fade starts only when a Go / Goto / Go- brought a
    /// newer one (decisions §29, R1-27), never on a handover.
    pub sequence_stamp: u64,
}

/// A cue part's timing in ms (decisions §29: in for a dimmer going up and
/// every other attribute, out for a dimmer going down).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Timing {
    pub fade_in: u64,
    pub fade_out: u64,
    pub delay_in: u64,
    pub delay_out: u64,
    /// The shape of its fades (a feature group's Path).
    pub path: lumacore_tree::FadePath,
}

impl Timing {
    pub fn of_part(part: &lumacore_tree::CuePart) -> Self {
        let ms = |s: f64| (s.max(0.0) * 1000.0).round() as u64;
        Self {
            fade_in: ms(part.fade_in),
            fade_out: ms(part.fade_out),
            delay_in: ms(part.delay_in),
            delay_out: ms(part.delay_out),
            path: lumacore_tree::FadePath::Linear,
        }
    }

    /// The part's timing for one attribute: the part's own, overlaid by its
    /// show feature group's timing in the part (decisions §30 "12f-a").
    pub fn of_attribute(part: &lumacore_tree::CuePart, attribute: &str, show: &lumacore_tree::ShowData) -> Self {
        let mut t = Self::of_part(part);
        if part.group_timing.is_empty() {
            return t;
        }
        let Some(g) = show.attribute_defs.get(attribute).and_then(|d| d.feature_group) else {
            return t;
        };
        if let Some(gt) = part.group_timing.get(&g) {
            let ms = |s: f64| (s.max(0.0) * 1000.0).round() as u64;
            if let Some(v) = gt.fade_in {
                t.fade_in = ms(v);
            }
            if let Some(v) = gt.fade_out {
                t.fade_out = ms(v);
            }
            if let Some(v) = gt.delay_in {
                t.delay_in = ms(v);
            }
            if let Some(v) = gt.delay_out {
                t.delay_out = ms(v);
            }
            if let Some(p) = gt.path {
                t.path = p;
            }
        }
        t
    }

    /// A value's own timing over this (decisions §30 "12f" value timing): its Fade and Delay
    /// count In and Out alike (individual timing).
    pub fn with_value(mut self, t: &lumacore_base::value::ValueTiming) -> Self {
        let ms = |s: f64| (s.max(0.0) * 1000.0).round() as u64;
        if let Some(f) = t.fade {
            self.fade_in = ms(f);
            self.fade_out = ms(f);
        }
        if let Some(d) = t.delay {
            self.delay_in = ms(d);
            self.delay_out = ms(d);
        }
        self
    }

    /// (delay, fade) for a change: `down` = a dimmer going down.
    pub fn pick(&self, down: bool) -> (u64, u64) {
        if down {
            (self.delay_out, self.fade_out)
        } else {
            (self.delay_in, self.fade_in)
        }
    }
}

/// Who owns a channel of our own output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Owner {
    /// No fixture there.
    #[default]
    Unpatched,
    /// A fixture's default (nothing gives it a value).
    Default,
    /// A playback or an unfrozen programmer, touched at the stamp.
    Ours(u64),
    /// A frozen programmer, touched at the stamp.
    Frozen(u64),
    Park,
}

/// Who a channel's final value comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Winner {
    #[default]
    Unpatched,
    Default,
    /// Our playbacks or programmers.
    Ours,
    Park,
    /// DMX input from this input line.
    Input(u32),
}

type Key = (Uuid, String);

/// Keep the latest touch of an attribute.
type Offer<'a> = (u64, &'a Value, Source, Option<Timing>);

fn offer<'a, K: Hash + Eq>(
    map: &mut HashMap<K, Offer<'a>>,
    key: K,
    stamp: u64,
    value: &'a Value,
    source: Source,
    timing: Option<Timing>,
) {
    match map.entry(key) {
        Entry::Occupied(mut slot) => {
            if slot.get().0 < stamp {
                slot.insert((stamp, value, source, timing));
            }
        }
        Entry::Vacant(slot) => {
            slot.insert((stamp, value, source, timing));
        }
    }
}

/// What running sequences release in their current cue (R1-27, Q12): the
/// attribute goes back to lower sources, fading by that cue part's timing.
/// (stamp of the Go that brought the cue, timing.)
pub fn releases(input: &EngineInput) -> HashMap<Key, (u64, Timing)> {
    releases_by(input, |fixture, attribute| Some((fixture, attribute.to_string())))
}

/// [`releases`] keyed by `key_of` (the plan compiler: attribute indices); an attribute
/// `key_of` doesn't know is left out.
pub fn releases_by<K: Hash + Eq>(
    input: &EngineInput,
    key_of: impl Fn(Uuid, &str) -> Option<K>,
) -> HashMap<K, (u64, Timing)> {
    let show = &input.show;
    let mut out = HashMap::new();
    for (seq_guid, state) in &show.playbacks.live.sequences {
        let Some((_, seq)) = show.sequence_by_guid(*seq_guid).map(|(_, n, s)| (n, s)) else {
            continue;
        };
        let Some(cue) = seq.cues.get(&state.current_cue) else {
            continue;
        };
        let stamp = state.stamp_of(state.current_cue);
        for part in cue.parts.values() {
            for v in part.values.iter().filter(|v| v.value.is_release()) {
                let Some(key) = key_of(v.fixture_id, &v.attribute) else { continue };
                if out.get(&key).is_none_or(|(s, _): &(u64, Timing)| *s < stamp) {
                    out.insert(key, (stamp, Timing::of_attribute(part, &v.attribute, show)));
                }
            }
        }
    }
    out
}

/// The merge before output modifiers: the winning value of every fixture
/// attribute that has one (attributes not listed output their default).
pub fn resolve(input: &EngineInput) -> HashMap<Key, Resolved> {
    resolve_by(input, |fixture, attribute| Some((fixture, attribute.to_string())))
}

/// [`resolve`] keyed by `key_of` — the plan compiler keys by attribute index, so no
/// `(Uuid, String)` is built per value (decisions §31); an attribute `key_of` doesn't know
/// (not patched on the tier) is left out. The one merge: `resolve` is this.
pub fn resolve_by<K: Hash + Eq + Clone>(
    input: &EngineInput,
    key_of: impl Fn(Uuid, &str) -> Option<K>,
) -> HashMap<K, Resolved> {
    resolve_cached(input, key_of, |_| false, &mut SequenceCache::default())
}

/// What one running sequence offers at its current cue, by key: worked out once and kept by
/// the plan compiler while the sequence's `Arc` and its current cue stay the same — a
/// programmer change doesn't track 500 cues again (decisions §31 "Arc parts").
pub struct SequenceCache<K> {
    by_sequence: HashMap<Uuid, CachedSequence<K>>,
    /// Which feature group each attribute is in when the offers were worked out: a cue part's
    /// per-group timing (decisions §30 "12f-a") follows a regrouping.
    groups: u64,
}

impl<K> Default for SequenceCache<K> {
    fn default() -> Self {
        Self { by_sequence: HashMap::new(), groups: 0 }
    }
}

/// A fingerprint of the show's attribute → feature group links.
fn groups_fingerprint(show: &lumacore_tree::ShowData) -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for (name, def) in &show.attribute_defs {
        name.hash(&mut h);
        def.feature_group.hash(&mut h);
    }
    h.finish()
}

struct CachedSequence<K> {
    /// The sequence it was worked out from (kept alive: the same `Arc` = the same content).
    sequence: Arc<lumacore_tree::SequenceObject>,
    cue: u32,
    /// (key, the cue that brought it, its part, its place in the part's values, the part's
    /// timing): the value is read from `sequence`, never copied.
    offers: Vec<(K, u32, u32, usize, Option<Timing>)>,
}

impl<K> SequenceCache<K> {
    /// Forget everything (the keys changed: a new channel tier).
    pub fn clear(&mut self) {
        self.by_sequence.clear();
    }
}

/// The timing of the value at `index` of `part`: the part's, its feature group's, then the
/// value's own (decisions §30 "12f").
pub fn value_timing(part: &lumacore_tree::CuePart, index: usize, attr: &str, show: &lumacore_tree::ShowData) -> Timing {
    let t = Timing::of_attribute(part, attr, show);
    part.values.get(index).map_or(t, |v| t.with_value(&v.timing))
}

/// [`resolve_by`] with the compiler's [`SequenceCache`].
///
/// Echoes (decisions §32) offer as the newest runs, except intensity (`is_intensity`): that is
/// HTP in the plan (`plan::EchoLevel`). Readers that pass `|_| false` see an echo's intensity as
/// one more LTP offer.
pub fn resolve_cached<K: Hash + Eq + Clone>(
    input: &EngineInput,
    key_of: impl Fn(Uuid, &str) -> Option<K>,
    is_intensity: impl Fn(&K) -> bool,
    cache: &mut SequenceCache<K>,
) -> HashMap<K, Resolved> {
    let show = &input.show;
    // A cue's preset links play the preset as it is now (decisions 0293): looked up here, once
    // per compile, so the cached offers (positions in the sequence) never hold a stale copy and a
    // changed preset needs no invalidation.
    let presets = lumacore_tree::PresetIndex::new(show);
    // What each running sequence offers: from the cache, worked out again only for a changed
    // sequence or another cue.
    let groups = groups_fingerprint(show);
    if groups != cache.groups {
        cache.by_sequence.clear();
        cache.groups = groups;
    }
    cache.by_sequence.retain(|guid, _| show.playbacks.live.sequences.contains_key(guid));
    for (seq_guid, state) in &show.playbacks.live.sequences {
        let Some(sequence) = show.sequence_by_guid(*seq_guid).map(|(_, _, s)| s) else {
            continue;
        };
        let fresh = cache
            .by_sequence
            .get(seq_guid)
            .is_some_and(|c| Arc::ptr_eq(&c.sequence, sequence) && c.cue == state.current_cue);
        if fresh {
            continue;
        }
        let mut offers = Vec::new();
        for ((fixture, attr), tracked) in sequence.tracked(state.current_cue) {
            let Some(key) = key_of(fixture, attr) else { continue };
            let timing = sequence
                .cue(tracked.cue)
                .and_then(|c| c.parts.get(&tracked.part))
                .map(|p| value_timing(p, tracked.index, attr, show));
            offers.push((key, tracked.cue, tracked.part, tracked.index, timing));
        }
        cache.by_sequence.insert(
            *seq_guid,
            CachedSequence { sequence: sequence.clone(), cue: state.current_cue, offers },
        );
    }
    let mut ltp: HashMap<K, Offer> = HashMap::new();
    let mut frozen: HashMap<K, Offer> = HashMap::new();
    for (seq_guid, state) in &show.playbacks.live.sequences {
        let (Some((seq_id, _)), Some(cached)) =
            (show.sequence_by_guid(*seq_guid).map(|(_, n, s)| (n, s)), cache.by_sequence.get(seq_guid))
        else {
            continue;
        };
        for (key, cue, part, index, timing) in &cached.offers {
            let Some(value) = cached
                .sequence
                .cue(*cue)
                .and_then(|c| c.parts.get(part))
                .and_then(|p| p.values.get(*index))
            else {
                continue;
            };
            // A link to a preset without this value plays nothing.
            let Some(played) = presets.resolve(&value.value, value.fixture_id, &value.attribute) else {
                continue;
            };
            offer(&mut ltp, key.clone(), state.stamp_of(*cue), played, Source::Sequence(seq_id), *timing);
        }
    }
    // Echoes: the newest runs of their sequences (stamped at Top On); intensity is HTP elsewhere.
    for (seq_guid, runs) in &show.playbacks.echoes.sequences {
        let Some((seq_id, sequence)) = show.sequence_by_guid(*seq_guid).map(|(_, n, s)| (n, s)) else {
            continue;
        };
        for echo in runs {
            for ((fixture, attr), tracked) in sequence.tracked(echo.cue) {
                let Some(key) = key_of(fixture, attr) else { continue };
                if is_intensity(&key) {
                    continue;
                }
                let timing = sequence
                    .cue(tracked.cue)
                    .and_then(|c| c.parts.get(&tracked.part))
                    .map(|p| value_timing(p, tracked.index, attr, show));
                let Some(played) = presets.resolve(tracked.value, fixture, attr) else { continue };
                offer(&mut ltp, key, echo.stamp, played, Source::Sequence(seq_id), timing);
            }
        }
    }
    // Only sequences have offered so far: the newest sequence stamp per key.
    let sequence_stamps: HashMap<K, u64> = ltp.iter().map(|(k, o)| (k.clone(), o.0)).collect();
    for (&user_no, user) in &show.users {
        let layer = input.users.iter().find(|u| u.user_no == user_no);
        if layer.is_some_and(|l| l.blind) {
            continue;
        }
        let target = if layer.is_some_and(|l| l.freeze) {
            &mut frozen
        } else {
            &mut ltp
        };
        for ((fixture, attr), pv) in &user.programmer.values {
            if pv.value.release {
                continue;
            }
            let Some(key) = key_of(*fixture, attr) else { continue };
            offer(
                target,
                key,
                pv.stamp,
                &pv.value,
                Source::Programmer(user_no),
                None,
            );
        }
    }
    let frozen_keys: HashSet<K> = frozen.keys().cloned().collect();
    for (key, win) in frozen {
        ltp.insert(key, win);
    }
    let mut out: HashMap<K, Resolved> = ltp
        .into_iter()
        .map(|(k, (stamp, value, source, timing))| {
            let frozen = frozen_keys.contains(&k);
            let sequence_stamp = sequence_stamps.get(&k).copied().unwrap_or(0);
            (
                k,
                Resolved {
                    value: value.clone(),
                    source,
                    stamp,
                    frozen,
                    timing,
                    sequence_stamp,
                },
            )
        })
        .collect();
    for p in &show.playback.park.fixtures {
        let Some(key) = key_of(p.fixture_id, &p.attribute) else { continue };
        let sequence_stamp = sequence_stamps.get(&key).copied().unwrap_or(0);
        out.insert(
            key,
            Resolved {
                value: p.value.clone(),
                source: Source::Park,
                stamp: 0,
                frozen: false,
                timing: None,
                sequence_stamp,
            },
        );
    }
    out
}

/// A value as a fraction (0–1) of the channel's range; None if it can't be
/// output on this channel (a physical value without a physical range).
pub fn fraction(value: &Value, cf: &ChannelFunction) -> Option<f64> {
    let f = match value.absolute()? {
        Absolute::Normalized(p) => p / 100.0,
        Absolute::Physical(v) => {
            let (from, to) = cf.physical_range?;
            (v - from) / (to - from)
        }
    };
    Some(f.clamp(0.0, 1.0))
}

/// The channel's default as a fraction of its range.
pub fn default_fraction(cf: &ChannelFunction) -> f64 {
    cf.default_fraction
        .unwrap_or(f64::from(cf.default_value) / 255.0)
}

/// A fixture's channel functions (its fixture type's mode).
pub fn channels<'a>(show: &'a ShowData, fixture: &lumacore_fixtures::Fixture) -> &'a [ChannelFunction] {
    show.devices
        .dmx
        .patch
        .fixture_types
        .values()
        .find(|ft| ft.meta.guid() == fixture.mvr.fixture_type)
        .and_then(|ft| ft.mode_of(fixture.mvr.revision, fixture.mvr.mode))
        .map_or(&[], |(_, mode)| mode.channel_functions.as_slice())
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
