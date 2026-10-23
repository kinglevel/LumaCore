// ██████╗ ███╗   ███╗██╗  ██╗
// ██╔══██╗████╗ ████║╚██╗██╔╝
// ██║  ██║██╔████╔██║ ╚███╔╝
// ██║  ██║██║╚██╔╝██║ ██╔██╗
// ██████╔╝██║ ╚═╝ ██║██╔╝ ██╗
// ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝
// FILE: dmx

//! `DMX` (decisions §33 48, §34 round 3 19): a universe's slots, the object. `DMX 2` is universe
//! 2's slots, `DMX 2.001` one address — one part reads as the universe (DMX only; no absolute
//! form). Bare it is Info: what the engine outputs, `/Input` what the input lines bring in; `DMX
//! 1.001 At 255` (the tester) is planned. As a destination, `At DMX …` is where fixtures go
//! ([`Dest`]); `Universe` stays the object (name, merge mode, lines).
//!
//! Owns:          the DMX object, its readout, reading `DMX …` as a destination.
//! Doesn't own:   the frames (the render engine, `AppState::published_frame`), where fixtures land
//!                (`fixture::address`), Park DMX (control), the DMX window's feed (lumacore-http).
//! Entry points:  DMX, Dest, dest_of.
//! Decisions:     §33 48, §34 round 3 19, §11 "Step 4c".

use lumacore_syntax::ast::{Id, Item, Range, Target, Value};

use super::super::{Doc, LumShape, ObjectDecl, ObjectDef, Undo, Verb, out};
use crate::commands::flags::FlagDef;
use crate::commands::help::HelpCategory;
use crate::commands::registry::CommandClass;
use crate::commands::typed::typed;
use crate::errors::render_code;
use crate::{AppState, DispatchResult};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a `DMX …` names: a bare `DMX` any slots, `DMX 2` a universe, `DMX 2.001` an address;
/// owner first, fixtures' own addresses (`Fixture 301 Thru 304 DMX`, break n with `DMX n`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dest {
    Any,
    Universe(u32),
    Exact(u32, u16),
    Of { fixtures: Vec<u32>, brk: Option<u16> },
}

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

const INPUT: FlagDef = FlagDef::new("Input", "What the input lines bring in, before it merges").on(&[Verb::Info]);

/// `DMX` (decisions §33 48): ids of two parts, one part = the universe.
pub static DMX: ObjectDef = ObjectDef::new(ObjectDecl {
    keyword: "DMX",
    lum: LumShape::ids(2),
    default_verb: Verb::Info,
    aliases: &[],
    doc: Doc {
        category: HelpCategory::Universe,
        syntax: "DMX <universe>[.<address>] [/Input] | DMX <u>.<a> At <value> (planned) | Park DMX <u>.<a> | … At DMX [<u>[.<a>]]",
        examples: &[
            "DMX 1                            -- what universe 1 outputs",
            "DMX 1.005                        -- one channel",
            "DMX 1 /Input                     -- what the input lines bring in",
            "Assign Fixture 201 At DMX 2.001  -- a fixture's address",
            "Assign Fixture 201 Thru 206 At DMX 2   -- the first free run in universe 2",
        ],
        related: &["universe", "park", "assign"],
        ..Doc::new(
            "DMX",
            "A universe's DMX slots: DMX 2 is universe 2, DMX 2.001 one address (never the absolute number: DMX 513 is universe 513). Alone it shows what the engine outputs; /Input what the input lines bring in. After At it is where fixtures go: an address, the first free run in a universe, or a bare DMX for the first free run anywhere.",
        )
    },
    family: None,
    call_undo: Undo::ReadOnly,
    call_class: CommandClass::Query,
    scope: super::super::on_master,
    reach: super::super::show_reach,
    rights: crate::rights::anyone,
    flags: &[INPUT],
    planned: &[Verb::At],
})
.verbs(&[(Verb::Info, typed::<InfoDmx>)]);

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `DMX <u>[.<a>] [/Input]` (and `Info DMX …`): the output sheet of a universe or one channel.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Info", object = "DMX")]
pub struct InfoDmx {
    #[lum(target)]
    pub target: Target,
    #[lum(option = "Input")]
    pub input: bool,
}

impl crate::commands::typed::Command for InfoDmx {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let shown = (|| {
            let [(None, Item::Path(segments))] = self.target.items.as_slice() else {
                return Err(render_code("DMX <universe>[.<address>]: one universe or one address.", "E-CMD-006"));
            };
            let value = Value::Target(self.target.clone());
            match dest_of(state, &value).unwrap_or_else(|| ids_of(segments.last().and_then(|s| s.ids.as_ref())))? {
                Dest::Universe(u) => Ok(readout(state, u, None, self.input)),
                Dest::Exact(u, a) => Ok(readout(state, u, Some(a), self.input)),
                Dest::Of { fixtures, brk } => Ok(addresses(state, &fixtures, brk)),
                Dest::Any => Err(render_code("DMX <universe>[.<address>]: which universe?", "E-CMD-006")),
            }
        })();
        out(shown.unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `value` as a DMX destination: None when it doesn't name DMX at all (another object, a word).
pub fn dest_of(state: &AppState, value: &Value) -> Option<Result<Dest, String>> {
    let Value::Target(target) = value else { return None };
    let [(None, Item::Path(segments))] = target.items.as_slice() else { return None };
    let vocab = crate::lum::vocabulary();
    let is_dmx = |s: &lumacore_syntax::ast::Segment| crate::objects::find(&vocab.word(s.object.node).name).is_some_and(|o| std::ptr::eq(o, &DMX));
    match segments.as_slice() {
        [segment] if is_dmx(segment) => Some(ids_of(segment.ids.as_ref())),
        // `Fixture 301 Thru 304 DMX [n]`: their addresses (break n).
        [owner, segment] if is_dmx(segment) => Some(owned(state, owner, segment.ids.as_ref())),
        _ => None,
    }
}

/// The fixtures an owner segment names and the break a `DMX n` after it picks.
fn owned(state: &AppState, owner: &lumacore_syntax::ast::Segment, ids: Option<&lumacore_syntax::ast::Ids>) -> Result<Dest, String> {
    let fixtures = crate::resolve::fixtures(state, &Target { items: vec![(None, Item::Path(vec![owner.clone()]))] })?;
    let brk = match ids_of(ids)? {
        Dest::Any => None,
        Dest::Universe(n) => Some(u16::try_from(n).map_err(|_| render_code("A fixture's break is 1–8.", "E-CMD-008"))?),
        _ => return Err(render_code("Fixture n DMX m: m is the break (1 = the first).", "E-CMD-008")),
    };
    Ok(Dest::Of { fixtures, brk })
}

/// A DMX id: none = any, `2` = universe 2, `2.001` = an address. One destination, numbers only.
fn ids_of(ids: Option<&lumacore_syntax::ast::Ids>) -> Result<Dest, String> {
    let Some(ids) = ids else { return Ok(Dest::Any) };
    let [(None, Range::One(Id::Number(n)))] = ids.ranges.as_slice() else {
        return Err(render_code("DMX names one universe or one address: DMX 2, DMX 2.001.", "E-CMD-008"));
    };
    let bad = || render_code(format!("DMX {}: a universe (2) or an address (2.001, 1–512).", n.node), "E-CMD-008");
    match n.node.split_once('.') {
        None => n.node.parse::<u32>().ok().filter(|u| *u > 0).map(Dest::Universe).ok_or_else(bad),
        Some((u, a)) => {
            let u = u.parse::<u32>().ok().filter(|u| *u > 0).ok_or_else(bad)?;
            let a = a.parse::<u16>().ok().filter(|a| (1..=512).contains(a)).ok_or_else(bad)?;
            Ok(Dest::Exact(u, a))
        }
    }
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// `Info Fixture 3 DMX`: each fixture's breaks and where they are.
pub fn addresses(state: &AppState, fixtures: &[u32], brk: Option<u16>) -> String {
    let show = &state.root.show_data;
    let lines: Vec<String> = fixtures
        .iter()
        .filter_map(|no| show.devices.dmx.patch.fixture(no).map(|f| (no, f)))
        .map(|(no, f)| {
            let breaks: Vec<String> = f
                .mvr
                .breaks
                .iter()
                .enumerate()
                .filter(|(i, _)| brk.is_none_or(|b| *i + 1 == b as usize))
                .map(|(i, b)| {
                    let u = show.universe_id_for_guid(b.universe_id).unwrap_or(0);
                    format!("DMX {} {u}.{:03}–{u}.{:03}", i + 1, b.start_address, b.start_address + b.channel_count.max(1) - 1)
                })
                .collect();
            format!("Fixture {no}: {}", if breaks.is_empty() { "no address".into() } else { breaks.join(" · ") })
        })
        .collect();
    lines.join("\n")
}

/// The engine's output of universe `u` (its channels that aren't 0, eight per row) or one
/// channel; `input`: what the DMX input lines bring in instead, before it merges.
fn readout(state: &AppState, u: u32, only: Option<u16>, input: bool) -> String {
    if !state.root.show_data.devices.dmx.patch.universes.contains_key(&u) {
        return render_code(format!("Universe {u} does not exist."), "E-UNIVERSE-001");
    }
    let frame = if input {
        match state.published_input(u) {
            Some(f) => f,
            None => return format!("DMX {u} /Input: nothing received (no input line routed here has a source)"),
        }
    } else {
        state.published_frame(u).unwrap_or([0; 512])
    };
    let label = if input { "DMX In" } else { "DMX" };
    if let Some(a) = only {
        return format!("{label} {u}.{a:03} = {}", frame[a as usize - 1]);
    }
    let mut lines = vec![format!("{label} universe {u} (non-zero channels)")];
    let mut row = String::new();
    let mut n = 0;
    for (i, v) in frame.iter().enumerate().filter(|(_, v)| **v != 0) {
        row.push_str(&format!("{:>3}:{v:<4}", i + 1));
        n += 1;
        if n % 8 == 0 {
            lines.push(std::mem::take(&mut row));
        }
    }
    if !row.is_empty() {
        lines.push(row);
    }
    if n == 0 {
        lines.push("  (all 0)".to_string());
    }
    lines.join("\n")
}
