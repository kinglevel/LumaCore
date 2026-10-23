// ██╗██████╗ ███████╗███╗   ██╗████████╗██╗███████╗██╗   ██╗
// ██║██╔══██╗██╔════╝████╗  ██║╚══██╔══╝██║██╔════╝╚██╗ ██╔╝
// ██║██║  ██║█████╗  ██╔██╗ ██║   ██║   ██║█████╗   ╚████╔╝
// ██║██║  ██║██╔══╝  ██║╚██╗██║   ██║   ██║██╔══╝    ╚██╔╝
// ██║██████╔╝███████╗██║ ╚████║   ██║   ██║██║        ██║
// ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝╚═╝        ╚═╝
// FILE: identify

//! Identify's output (decisions §34 round 3, "Identify, Probe and Find hardened"): what an
//! identified fixture or a probed address sends. The compiler turns the published targets into
//! slots — each with its wave — and the tick writes them over the merged output, after DMX input:
//! above programmers, playback, masters (a blackout too), Solo and input; never on a parked
//! channel. A fixture shows its GDTF defaults, its dimmer a sine 15 % → 100 % every 2 s, its
//! colour round the hue circle every 6 s through its mixing (RGB / CMY), a wheel-only fixture
//! steps its wheel's slots, 1 s each; a raw probe sends 38 → 255 in the 2 s sine, or, `/As` a
//! fixture, that fixture's shape. A fixture parked by a conflict still flashes on its own slots.
//!
//! Owns:          Identify (the published target), IdentifySlot, Wave, compile, apply.
//! Doesn't own:   who identifies what (the state's Identify object), the merge (`merge`), Park.
//! Entry points:  compile (compiler thread), apply (the tick).
//! Decisions:     §34 round 3, "Identify, Probe and Find hardened".

use crate::merge::{Owner, channels, default_fraction};
use crate::plan::ChannelTier;
use lumacore_fixtures::fixture_type::{AttributeType, ChannelFunction};
use lumacore_tree::tree::ShowData;
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a user identifies, as the state publishes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identify {
    /// A fixture: every break and every multipatch copy, in its own shape.
    Fixture(Uuid),
    /// A run of slots (universe number, first slot, how many): a raw sine, or fixture
    /// `as_fixture`'s shape from `start` (Find).
    Dmx { universe: u32, start: u16, count: u16, as_fixture: Option<Uuid> },
}

/// How an identified channel moves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wave {
    /// Its GDTF default, held.
    Fixed(f64),
    /// The dimmer's sine, 15 % → 100 % every 2 s.
    Dimmer,
    /// One primary of the hue circle (0 red, 1 green, 2 blue), every 6 s; `sub` = its CMY
    /// complement.
    Hue { primary: u8, sub: bool },
    /// A wheel stepping its slots, 1 s each: `count` values from `from` in `IdentifyTier::wheel`.
    Wheel { from: u32, count: u32 },
    /// A raw probe: 38 → 255 in the 2 s sine.
    Raw,
}

/// One identified channel: where it is written and how it moves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IdentifySlot {
    /// Index into the tier's universes.
    pub universe: u16,
    pub at: u16,
    pub bytes: u8,
    pub wave: Wave,
}

/// The identified channels of a plan.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IdentifyTier {
    pub slots: Vec<IdentifySlot>,
    /// The wheels' slot values (fractions), each wheel a run.
    pub wheel: Vec<f64>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The identified channels of `targets`, on the compiler thread. A slot a Park holds
/// (`owners` = Park) is left out: Park wins.
pub fn compile(show: &ShowData, tier: &ChannelTier, owners: &[[Owner; 512]], targets: &[Identify]) -> IdentifyTier {
    let mut out = IdentifyTier::default();
    if targets.is_empty() {
        return out;
    }
    let patch = &show.devices.dmx.patch;
    let fixture_of = |g: &Uuid| patch.fixtures.values().find(|f| f.meta.identity.guid == *g);
    for t in targets {
        match t {
            Identify::Fixture(g) => {
                let Some(f) = fixture_of(g) else { continue };
                let cfs = channels(show, f);
                let copies = f.luma.multipatch.iter().flat_map(|c| c.breaks.iter().enumerate());
                for (break_no, b) in f.mvr.breaks.iter().enumerate().chain(copies) {
                    let Some(&u) = tier.universe_index.get(&b.universe_id) else { continue };
                    shape(&mut out, cfs, break_no, u, b.start_address, b.channel_count);
                }
            }
            Identify::Dmx { universe, start, count, as_fixture } => {
                let Some(u) = tier.universe_no(*universe) else { continue };
                match as_fixture.and_then(|g| fixture_of(&g)) {
                    Some(f) => shape(&mut out, channels(show, f), 0, u as u16, *start, *count),
                    None => {
                        for s in *start..start.saturating_add(*count) {
                            if (1..=512).contains(&s) {
                                out.slots.push(IdentifySlot { universe: u as u16, at: s - 1, bytes: 1, wave: Wave::Raw });
                            }
                        }
                    }
                }
            }
        }
    }
    out.slots.retain(|s| owners.get(s.universe as usize).is_none_or(|o| (0..s.bytes as usize).all(|i| o.get(s.at as usize + i) != Some(&Owner::Park))));
    out
}

/// A fixture's break `break_no` (0 = the first) laid out from `start` (1-indexed) for `count`
/// slots: each channel its wave.
fn shape(out: &mut IdentifyTier, cfs: &[ChannelFunction], break_no: usize, universe: u16, start: u16, count: u16) {
    let Some(base) = (start as usize).checked_sub(1) else { return };
    // Mixing wins over a wheel: a wheel moves only on a fixture with no mixing.
    let mixes = cfs.iter().any(|cf| primary(&cf.attribute_name).is_some());
    for cf in cfs {
        if cf.is_virtual || usize::from(cf.dmx_break.max(1)) != break_no + 1 || cf.dmx_offset >= count {
            continue;
        }
        let at = base + cf.dmx_offset as usize;
        if at >= 512 {
            continue;
        }
        let wave = if cf.attribute_type == AttributeType::Dimmer {
            Wave::Dimmer
        } else if let Some((primary, sub)) = primary(&cf.attribute_name) {
            Wave::Hue { primary, sub }
        } else if let Some(slots) = (!mixes).then(|| wheel_slots(cf)).filter(|s| s.len() > 1) {
            let from = out.wheel.len() as u32;
            out.wheel.extend(&slots);
            Wave::Wheel { from, count: slots.len() as u32 }
        } else {
            Wave::Fixed(default_fraction(cf))
        };
        let bytes = usize::from(cf.byte_count.clamp(1, 4)).min(512 - at) as u8;
        out.slots.push(IdentifySlot { universe, at: at as u16, bytes, wave });
    }
}

/// A colour mixing channel's primary (GDTF ColorAdd / ColorRGB / ColorSub): 0 red, 1 green,
/// 2 blue, and whether it subtracts (CMY). White, amber, UV … stay at their defaults.
fn primary(attribute: &str) -> Option<(u8, bool)> {
    let (kind, part) = attribute.split_once('_')?;
    let sub = match kind {
        "ColorAdd" | "ColorRGB" => false,
        "ColorSub" => true,
        _ => return None,
    };
    let p = match part {
        "R" | "Red" | "C" | "Cyan" => 0,
        "G" | "Green" | "M" | "Magenta" => 1,
        "B" | "Blue" | "Y" | "Yellow" => 2,
        _ => return None,
    };
    Some((p, sub))
}

/// A colour wheel's slots as fractions of the channel (its channel sets naming a wheel slot).
fn wheel_slots(cf: &ChannelFunction) -> Vec<f64> {
    if !cf.attribute_name.starts_with("Color") {
        return Vec::new();
    }
    cf.functions
        .iter()
        .filter(|f| f.wheel.is_some())
        .flat_map(|f| f.channel_sets.iter().filter(|s| s.wheel_slot.is_some()).map(|s| s.dmx_from))
        .collect()
}

// tidy: render-begin (decisions §24: the tick)
/// Write the identified channels over `out` at `now_ms` (after the input merge; allocates
/// nothing).
pub fn apply(out: &mut [[u8; 512]], tier: &IdentifyTier, now_ms: u64) {
    for s in &tier.slots {
        let f = level(s.wave, &tier.wheel, now_ms);
        if let Some(buf) = out.get_mut(s.universe as usize) {
            crate::render::write_channel(buf, s.at as usize, s.bytes as usize, f);
        }
    }
}

/// A wave's level (0–1) at `now_ms`.
pub fn level(wave: Wave, wheel: &[f64], now_ms: u64) -> f64 {
    // 0 → 1 → 0 over `period` ms.
    let sine = |period: u64| 0.5 - 0.5 * (std::f64::consts::TAU * (now_ms % period) as f64 / period as f64).cos();
    match wave {
        Wave::Fixed(f) => f,
        Wave::Dimmer => 0.15 + 0.85 * sine(2000),
        Wave::Raw => (38.0 + 217.0 * sine(2000)) / 255.0,
        Wave::Hue { primary, sub } => {
            let v = hue_part((now_ms % 6000) as f64 / 6000.0, primary);
            if sub { 1.0 - v } else { v }
        }
        Wave::Wheel { from, count } => {
            let i = from as usize + ((now_ms / 1000) % u64::from(count.max(1))) as usize;
            wheel.get(i).copied().unwrap_or(0.0)
        }
    }
}

/// One primary of a fully saturated colour at hue `h` (0–1).
fn hue_part(h: f64, primary: u8) -> f64 {
    // HSV with S = V = 1: each primary is a trapezoid, shifted by a third.
    let x = (h * 6.0 + [0.0, 4.0, 2.0][primary as usize % 3]) % 6.0;
    match x {
        x if x < 1.0 => 1.0,
        x if x < 2.0 => 2.0 - x,
        x if x < 4.0 => 0.0,
        x if x < 5.0 => x - 4.0,
        _ => 1.0,
    }
}
// tidy: render-end

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
