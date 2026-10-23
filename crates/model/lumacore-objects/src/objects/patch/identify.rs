// ██╗██████╗ ███████╗███╗   ██╗████████╗██╗███████╗██╗   ██╗
// ██║██╔══██╗██╔════╝████╗  ██║╚══██╔══╝██║██╔════╝╚██╗ ██╔╝
// ██║██║  ██║█████╗  ██╔██╗ ██║   ██║   ██║█████╗   ╚████╔╝
// ██║██║  ██║██╔══╝  ██║╚██╗██║   ██║   ██║██╔══╝    ╚██╔╝
// ██║██████╔╝███████╗██║ ╚████║   ██║   ██║██║        ██║
// ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝╚═╝        ╚═╝
// FILE: identify

//! `Identify` (decisions §34 round 3, "Identify, Probe and Find hardened"): fixtures or addresses
//! show themselves on the rig. One word, an object owning what it flashes — `Identify Fixture
//! 104` runs its default verb On, a bare `Identify` takes your selection, `Identify DMX 1.150`
//! probes an address or a run (`/As Fixture n`: in that fixture's shape, Find), `Off Identify
//! [target]` ends yours or part of it. One per user (a new line replaces yours); station output
//! only — never stored, never an Oops step, run on the station that sends; Setup and up. What is
//! left on ends by itself (logout, a lost holder, 5 minutes, Unclaim of the patch) and is logged.
//!
//! Owns:          IDENTIFY, its On / Off, tick (the ends), end_left_on.
//! Doesn't own:   the output (`lumacore_render::identify`), the state (`identifying`).
//! Entry points:  IDENTIFY, tick, end_left_on.
//! Decisions:     §34 round 3, "Identify, Probe and Find hardened".

use lumacore_render::identify::Identify;
use lumacore_syntax::ast::{Id, Item, Range, Segment, Target, Value};

use super::super::{Doc, LumShape, ObjectDecl, ObjectDef, Undo, Verb, out};
use crate::commands::flags::FlagDef;
use crate::commands::help::HelpCategory;
use crate::commands::registry::{CommandClass, Rights};
use crate::commands::typed::typed;
use crate::errors::render_code;
use crate::identifying::{Identifying, MAX_MS};
use crate::{AppState, DispatchResult};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

const AS: FlagDef = FlagDef::new("As", "Probe the run the way this fixture would show (Find)").value("fixture").on(&[Verb::On]);

/// What Identify owns: the fixtures and the addresses it flashes.
static CHILDREN: [&ObjectDecl; 2] = [&super::FIXTURE.decl, &super::dmx::DMX.decl];

/// Identify needs Setup, the patching level (§34 round 3, 18); reading is anyone's.
fn rights(verb: Verb, _: &[&str]) -> Rights {
    match verb {
        Verb::List | Verb::Info | Verb::Dump => Rights::Guest,
        _ => Rights::Setup,
    }
}

pub static IDENTIFY: ObjectDef = ObjectDef::new(ObjectDecl {
    keyword: "Identify",
    lum: LumShape::ONE.children(&CHILDREN),
    default_verb: Verb::On,
    aliases: &[],
    doc: Doc {
        category: HelpCategory::Universe,
        syntax: "Identify [Fixture <ids> | DMX <u.a> [Thru <u.a>]] [/As Fixture <n>] | Off Identify [Fixture <ids> | DMX …]",
        examples: &[
            "Identify Fixture 101 Thru 106     -- they flash on the rig",
            "Identify                          -- your selection",
            "Identify DMX 1.150                -- probe an address, patched or not",
            "Identify DMX 1.043 Thru 1.056 /As Fixture 104   -- Find: the run in 104's shape",
            "Off Identify                      -- end yours",
        ],
        related: &["dmx", "fixture", "park", "highlight"],
        ..Doc::new(
            "Identify",
            "Make fixtures or addresses show themselves on the rig: their GDTF defaults, the dimmer in a sine (15–100 %, 2 s), the colour round the hue circle (6 s; a wheel steps its slots). A probed address sends a 38–255 sine. Above programmers, playback, masters, Solo and input; Park wins. One per user; it ends with Off Identify, at logout and after 5 minutes. Station output only: never stored, never an Oops step.",
        )
    },
    family: None,
    call_undo: Undo::ReadOnly,
    call_class: CommandClass::Commit,
    scope: super::super::on_master,
    reach: super::super::show_reach,
    rights,
    flags: &[AS],
    planned: &[],
})
.verbs(&[(Verb::On, typed::<OnIdentify>), (Verb::Off, typed::<OffIdentify>)]);

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Identify [target] [/As Fixture n]` (= `On Identify …`): yours, replacing what you had.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "On", object = "Identify")]
pub struct OnIdentify {
    #[lum(target)]
    pub target: Target,
    #[lum(option = "As")]
    pub as_fixture: Option<Value>,
}

impl crate::commands::typed::Command for OnIdentify {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(start(state, &self.target, self.as_fixture.as_ref()).unwrap_or_else(|e| e))
    }
}

/// `Off Identify [target]`: all of yours, or the fixtures / addresses named.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Off", object = "Identify")]
pub struct OffIdentify {
    #[lum(target)]
    pub target: Target,
}

impl crate::commands::typed::Command for OffIdentify {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(stop(state, &self.target).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// What a line names under Identify: fixtures (their numbers) and DMX runs.
#[derive(Default)]
struct Named {
    fixtures: Vec<u32>,
    /// (universe, first slot, count); `None` = every run (`DMX Thru`).
    runs: Vec<Option<(u32, u16, u16)>>,
}

fn named(state: &AppState, target: &Target) -> Result<Named, String> {
    let vocab = crate::lum::vocabulary();
    let is = |seg: &Segment, obj: &ObjectDef| crate::objects::find(&vocab.word(seg.object.node).name).is_some_and(|o| std::ptr::eq(o, obj));
    let mut n = Named::default();
    for (_, item) in &target.items {
        let Item::Path(segments) = item else {
            return Err(render_code("Identify Fixture … | Identify DMX ….", "E-CMD-006"));
        };
        // The leading `Identify` (owner first), then what it owns.
        let rest: Vec<Segment> = segments.iter().skip_while(|s| is(s, &IDENTIFY)).cloned().collect();
        let Some(first) = rest.first() else { continue };
        if is(first, &super::dmx::DMX) {
            let ids = first.ids.as_ref().ok_or_else(|| render_code("Identify DMX <universe>[.<address>]: which slots?", "E-CMD-006"))?;
            for (_, r) in &ids.ranges {
                n.runs.push(run_of(r)?);
            }
        } else {
            n.fixtures.extend(crate::resolve::fixtures(state, &Target { items: vec![(None, Item::Path(rest))] })?);
        }
    }
    Ok(n)
}

/// One DMX range as a run: `2` a universe, `2.150` an address, `2.043 Thru 2.056` a run inside
/// one universe, a bare `Thru` every run (only Off takes it).
fn run_of(r: &Range) -> Result<Option<(u32, u16, u16)>, String> {
    let span = |u: u32, a: u16, b: u16| Ok(Some((u, a.min(b), a.max(b) - a.min(b) + 1)));
    match r {
        Range::One(id) => match address(id)? {
            (u, None) => span(u, 1, 512),
            (u, Some(a)) => span(u, a, a),
        },
        Range::Thru { from: None, to: None, .. } => Ok(None),
        Range::Thru { from, to, .. } => {
            let from = from.as_ref().map(address).transpose()?;
            let to = to.as_ref().map(address).transpose()?;
            let u = from.or(to).map_or(1, |(u, _)| u);
            if to.is_some_and(|(v, _)| v != u) {
                return Err(render_code("Identify DMX: a run stays in one universe (2.001 Thru 2.040).", "E-CMD-008"));
            }
            span(u, from.and_then(|(_, a)| a).unwrap_or(1), to.and_then(|(_, a)| a).unwrap_or(512))
        }
    }
}

/// A DMX id: (universe, address if one is named).
fn address(id: &Id) -> Result<(u32, Option<u16>), String> {
    let Id::Number(n) = id else {
        return Err(render_code("DMX takes numbers: 2, 2.150.", "E-CMD-008"));
    };
    let bad = || render_code(format!("DMX {}: a universe (2) or an address (2.150, 1–512).", n.node), "E-CMD-008");
    match n.node.split_once('.') {
        None => Ok((n.node.parse::<u32>().ok().filter(|u| *u > 0).ok_or_else(bad)?, None)),
        Some((u, a)) => Ok((
            u.parse::<u32>().ok().filter(|u| *u > 0).ok_or_else(bad)?,
            Some(a.parse::<u16>().ok().filter(|a| (1..=512).contains(a)).ok_or_else(bad)?),
        )),
    }
}

fn start(state: &mut AppState, target: &Target, as_fixture: Option<&Value>) -> Result<String, String> {
    let mut n = named(state, target)?;
    let me = state.effective_user_name().to_string();
    let patch = &state.root.show_data.devices.dmx.patch;
    // A bare `Identify`: your selection.
    if n.fixtures.is_empty() && n.runs.is_empty() {
        let selected = state.root.show_data.programmer_of(&me).map(|p| p.selected_fixtures.clone()).unwrap_or_default();
        n.fixtures = patch.fixtures.values().filter(|f| selected.contains(&f.meta.identity.guid)).filter_map(|f| f.mvr.fid).collect();
        if n.fixtures.is_empty() {
            return Err(render_code("Nothing selected to identify: Identify Fixture … or select fixtures first.", "E-CMD-009"));
        }
    }
    let as_guid = match as_fixture {
        None => None,
        Some(Value::Target(t)) => {
            let nos = crate::resolve::fixtures(state, t)?;
            let [no] = nos.as_slice() else {
                return Err(render_code("/As names one fixture: /As Fixture 104.", "E-CMD-008"));
            };
            Some(patch.fixture(no).expect("resolved").meta.identity.guid)
        }
        Some(_) => return Err(render_code("/As names a fixture: /As Fixture 104.", "E-CMD-008")),
    };
    let mut targets: Vec<Identify> = n.fixtures.iter().filter_map(|no| patch.fixture(no)).map(|f| Identify::Fixture(f.meta.identity.guid)).collect();
    let mut words = Vec::new();
    if !n.fixtures.is_empty() {
        words.push(format!("Fixture {}", super::fixture::address::numbers(&n.fixtures)));
    }
    for run in &n.runs {
        let Some((u, a, c)) = run else {
            return Err(render_code("Identify DMX: which slots? (DMX 2, DMX 2.150 Thru 2.160)", "E-CMD-006"));
        };
        targets.push(Identify::Dmx { universe: *u, start: *a, count: *c, as_fixture: as_guid });
        words.push(if *c == 1 { format!("DMX {u}.{a:03}") } else { format!("DMX {u}.{a:03}–{u}.{:03}", a + c - 1) });
    }
    let what = words.join(" + ");
    state.identify.insert(me, Identifying { targets, what: what.clone(), started_ms: lumacore_base::clock::session_ms() });
    Ok(format!("Identify {what}: flashing on the live rig (Off Identify ends it; 5 minutes at most)."))
}

fn stop(state: &mut AppState, target: &Target) -> Result<String, String> {
    let n = named(state, target)?;
    let me = state.effective_user_name().to_string();
    let guids: Vec<uuid::Uuid> = {
        let patch = &state.root.show_data.devices.dmx.patch;
        n.fixtures.iter().filter_map(|no| patch.fixture(no)).map(|f| f.meta.identity.guid).collect()
    };
    let Some(mine) = state.identify.get_mut(&me) else {
        return Ok("Nothing of yours is identified.".into());
    };
    if !(n.fixtures.is_empty() && n.runs.is_empty()) {
        mine.targets.retain(|t| match t {
            Identify::Fixture(g) => !guids.contains(g),
            Identify::Dmx { universe, start, count, .. } => !n.runs.iter().any(|r| match r {
                None => true,
                Some((u, a, c)) => u == universe && *a < start + count && *start < a + c,
            }),
        });
        if !mine.targets.is_empty() {
            return Ok("Identify: the rest goes on.".into());
        }
    }
    state.identify.remove(&me);
    Ok("Identify off.".into())
}

/// End what is left of `user`'s Identify and log it (`why`: Unclaim, logged out, 5 minutes).
pub fn end_left_on(state: &mut AppState, user: &str, why: &str) {
    let key = state.identify.keys().find(|k| k.eq_ignore_ascii_case(user)).cloned();
    if let Some(gone) = key.and_then(|k| state.identify.remove(&k)) {
        state.sysmon_pending.push(("Patch".into(), format!("Identify left on ({user}: {}) — ended ({why})", gone.what)));
    }
}

/// The station's check, on the state thread: an Identify whose user is no longer here
/// (`present`: the users logged in on the LumaSession's stations), or that ran 5 minutes, ends and
/// is logged.
pub fn tick(state: &mut AppState, present: &std::collections::HashSet<String>) {
    if state.identify.is_empty() {
        return;
    }
    let now = lumacore_base::clock::session_ms();
    let ends: Vec<(String, &str)> = state
        .identify
        .iter()
        .filter_map(|(u, i)| {
            if !present.iter().any(|h| h.eq_ignore_ascii_case(u)) {
                Some((u.clone(), "logged out"))
            } else if now.saturating_sub(i.started_ms) >= MAX_MS {
                Some((u.clone(), "5 minutes"))
            } else {
                None
            }
        })
        .collect();
    for (user, why) in ends {
        end_left_on(state, &user, why);
    }
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
