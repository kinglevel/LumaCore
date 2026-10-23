// ███╗   ███╗██╗   ██╗██╗     ████████╗██╗██████╗  █████╗ ████████╗ ██████╗██╗  ██╗
// ████╗ ████║██║   ██║██║     ╚══██╔══╝██║██╔══██╗██╔══██╗╚══██╔══╝██╔════╝██║  ██║
// ██╔████╔██║██║   ██║██║        ██║   ██║██████╔╝███████║   ██║   ██║     ███████║
// ██║╚██╔╝██║██║   ██║██║        ██║   ██║██╔═══╝ ██╔══██║   ██║   ██║     ██╔══██║
// ██║ ╚═╝ ██║╚██████╔╝███████╗   ██║   ██║██║     ██║  ██║   ██║   ╚██████╗██║  ██║
// ╚═╝     ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   ╚═╝╚═╝     ╚═╝  ╚═╝   ╚═╝    ╚═════╝╚═╝  ╚═╝
// FILE: multipatch

//! `Fixture 201 Multipatch 1` (decisions §34 round 3 19; the copies live inside their
//! fixture, decided 2026-10-19): `Store Fixture 201 Thru 206 Multipatch 1 [Thru n] [At DMX …]`
//! adds copies (numbered by place, the next numbers only), `Assign Fixture 201 Multipatch 1 At DMX
//! …` addresses one, `Delete Fixture 201 Multipatch 1` removes it, `Info Fixture 201 Multipatch`
//! lists them; selecting one selects its fixture (it has no values of its own). The fixture words
//! (Store / Assign / Delete / Info / the bare call) hand a `… Multipatch …` target here.
//!
//! Owns:          MULTIPATCH, the copies' Store / Assign / Delete / Info.
//! Doesn't own:   the copies' data (`lumacore_fixtures::MultipatchCopy`), where they land
//!                (`address`), the mirrored output (the render plan).
//! Entry points:  MULTIPATCH, split, store, readdress, delete, info, fixtures_only.
//! Decisions:     §34 round 3 19.

use std::collections::BTreeMap;

use lumacore_syntax::ast::{Id, Ids, Item, Range, Target};

use super::address::{self, Item as Place, Spacing};
use crate::errors::render_code;
use crate::objects::patch::dmx::Dest;
use crate::objects::{Doc, LumShape, ObjectDecl, ObjectDef, Undo, Verb};
use crate::commands::help::HelpCategory;
use crate::commands::registry::CommandClass;
use crate::text::render_success;
use crate::AppState;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// `Multipatch`, a child of Fixture (owner first); never alone (a global number would change
/// with the patch).
pub static MULTIPATCH: ObjectDef = ObjectDef::new(ObjectDecl {
    keyword: "Multipatch",
    lum: LumShape::ids(1),
    default_verb: Verb::SelectFixtures,
    aliases: &[],
    doc: Doc {
        category: HelpCategory::Fixture,
        syntax: "Fixture <n> Multipatch <m> | Store Fixture <ns> Multipatch <m> [Thru <k>] [At DMX …] | Assign Fixture <n> Multipatch <m> At DMX … | Delete Fixture <n> Multipatch <m> | Info Fixture <n> Multipatch",
        examples: &[
            "Store Fixture 201 Multipatch 1 Thru 2 At DMX 5.001  -- two copies of 201",
            "Assign Fixture 201 Multipatch 1 At DMX 5.020",
            "Fixture 201 Multipatch 1                            -- selects 201",
            "Delete Fixture 201 Multipatch 1",
        ],
        related: &["fixture", "dmx"],
        ..Doc::new(
            "Multipatch",
            "A copy of a fixture at another DMX address: it outputs what its fixture does, has its own name and place, and no values of its own. Numbered per fixture: Fixture 201 Multipatch 1.",
        )
    },
    family: None,
    call_undo: Undo::ReadOnly,
    call_class: CommandClass::Commit,
    scope: crate::objects::on_master,
    reach: crate::objects::show_reach,
    rights: crate::rights::fixture,
    flags: &[],
    planned: &[],
})
// Alone it names no fixture: its words go through its owner (`Info Fixture 201 Multipatch`).
.verbs(&[(Verb::Info, crate::commands::typed::typed::<InfoMultipatch>)]);

/// `Info Multipatch …` alone names no fixture: a copy is read through its owner.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Info", object = "Multipatch")]
pub struct InfoMultipatch {
    #[lum(target)]
    pub target: Option<Target>,
}

impl crate::commands::typed::Command for InfoMultipatch {
    fn run(self, _: &mut AppState) -> crate::DispatchResult {
        let named = self.target.map_or(String::new(), |_| " (by its own number it names nothing)".into());
        crate::objects::out(render_code(format!("A multipatch belongs to its fixture: Info Fixture <n> Multipatch{named}."), "E-CMD-006"))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// A `Fixture … Multipatch …` target: the fixtures' part and the copies' ids (None = all).
pub fn split(target: &Target) -> Option<(Target, Option<Ids>)> {
    let [(None, Item::Path(segments))] = target.items.as_slice() else { return None };
    let [fixture, copy] = segments.as_slice() else { return None };
    let vocab = crate::lum::vocabulary();
    let is_copy = crate::objects::find(&vocab.word(copy.object.node).name).is_some_and(|o| std::ptr::eq(o, &MULTIPATCH));
    is_copy.then(|| (Target { items: vec![(None, Item::Path(vec![fixture.clone()]))] }, copy.ids.clone()))
}

/// `Store Fixture <ns> Multipatch [<m> [Thru <k>]] [At DMX …]`: new copies, the next numbers only;
/// a bare `Multipatch` is each fixture's next copy (decisions §34 round 3, the left-out items).
pub fn store(state: &mut AppState, fixtures: &Target, copies: Option<&Ids>, dest: Option<Dest>, every: Option<u16>) -> Result<String, String> {
    let fixtures = crate::resolve::fixtures(state, fixtures)?;
    let typed = copies.map(|c| numbers(Some(c))).transpose()?;
    let patch = &state.root.show_data.devices.dmx.patch;
    let mut wanted_of: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for no in &fixtures {
        let have = patch.fixture(no).expect("resolved").luma.multipatch.len() as u32;
        let wanted = typed.clone().unwrap_or_else(|| vec![have + 1]);
        if wanted.first() != Some(&(have + 1)) || wanted.windows(2).any(|w| w[1] != w[0] + 1) {
            return Err(render_code(format!("Fixture {no} has {have} multipatch: new ones are numbered from {}.", have + 1), "E-CMD-010"));
        }
        wanted_of.insert(*no, wanted);
    }
    // One place per copy, before anything changes.
    let mut who: BTreeMap<u32, (u32, usize)> = BTreeMap::new();
    let mut items = Vec::new();
    for no in &fixtures {
        let fp = address::footprint(&state.root.show_data, *no).ok_or_else(|| render_code(format!("Fixture {no} has no footprint."), "E-CMD-010"))?;
        for m in &wanted_of[no] {
            let key = u32::MAX - items.len() as u32;
            who.insert(key, (*no, *m as usize - 1));
            items.push(Place { no: key, footprint: fp, now: None });
        }
    }
    let plan = match dest {
        Some(d) => Some(address::plan(&state.root.show_data, &items, d, Spacing { every, keep: false })?),
        None => None,
    };
    for no in &fixtures {
        let f = state.root.show_data.devices.dmx.patch.fixture_mut(no).expect("resolved");
        for _ in &wanted_of[no] {
            f.luma.multipatch.push(lumacore_fixtures::MultipatchCopy { meta: lumacore_base::NodeMeta::new(None), breaks: Vec::new(), matrix: f.mvr.matrix });
        }
    }
    let made = match &typed {
        Some(w) => format!("Multipatch {}", address::numbers(w)),
        None => "Multipatch: each one's next".to_string(),
    };
    let mut lines = vec![render_success(&format!("Stored Fixture {} {made}", address::numbers(&fixtures)))];
    if let Some(plan) = plan {
        write(state, &plan, &who, &items);
        lines[0].push_str(&format!(" at {}", address::placed_text(&plan)));
        lines.extend(address::plan_notes(&plan));
    }
    Ok(lines.join("\n"))
}

/// `Assign Fixture <n> Multipatch <m> At DMX …`.
pub fn readdress(state: &mut AppState, fixtures: &Target, copies: Option<&Ids>, dest: Dest) -> Result<String, String> {
    let fixtures = crate::resolve::fixtures(state, fixtures)?;
    let mut who: BTreeMap<u32, (u32, usize)> = BTreeMap::new();
    let mut items = Vec::new();
    for no in &fixtures {
        let fp = address::footprint(&state.root.show_data, *no).unwrap_or(1);
        for i in existing(state, *no, copies)? {
            let key = u32::MAX - items.len() as u32;
            who.insert(key, (*no, i));
            items.push(Place { no: key, footprint: fp, now: None });
        }
    }
    let plan = address::plan(&state.root.show_data, &items, dest, Spacing::default())?;
    write(state, &plan, &who, &items);
    let mut lines = vec![render_success(&format!("Fixture {} Multipatch at {}", address::numbers(&fixtures), address::placed_text(&plan)))];
    lines.extend(address::plan_notes(&plan));
    Ok(lines.join("\n"))
}

/// `Delete Fixture <n> Multipatch <m>` (the copies after it move down one number).
pub fn delete(state: &mut AppState, fixtures: &Target, copies: Option<&Ids>) -> Result<String, String> {
    let fixtures = crate::resolve::fixtures(state, fixtures)?;
    let mut gone = 0;
    for no in &fixtures {
        let mut idx = existing(state, *no, copies)?;
        idx.sort_unstable();
        let f = state.root.show_data.devices.dmx.patch.fixture_mut(no).expect("resolved");
        for i in idx.into_iter().rev() {
            f.luma.multipatch.remove(i);
            gone += 1;
        }
    }
    state.root.show_data.rebuild_occupancy();
    Ok(render_success(&format!("Deleted {gone} multipatch of Fixture {}", address::numbers(&fixtures))))
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// `Info Fixture <n> Multipatch`: each copy, its name and address.
pub fn info(state: &AppState, fixtures: &[u32]) -> String {
    let show = &state.root.show_data;
    let mut lines = Vec::new();
    for no in fixtures {
        let Some(f) = show.devices.dmx.patch.fixture(no) else { continue };
        if f.luma.multipatch.is_empty() {
            lines.push(format!("Fixture {no}: no multipatch"));
        }
        for (i, c) in f.luma.multipatch.iter().enumerate() {
            let at = c.breaks.first().and_then(|b| show.universe_id_for_guid(b.universe_id).map(|u| format!("{u}.{:03}", b.start_address))).unwrap_or_else(|| "no address".into());
            let name = c.meta.identity.name.as_deref().map(|n| format!(" \"{n}\"")).unwrap_or_default();
            lines.push(format!("Fixture {no} Multipatch {}{name} · {at}", i + 1));
        }
    }
    lines.join("\n")
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// Raw words naming copies (`201 Multipatch 1`): the fixture part alone, for the selection (a
/// copy selects its fixture).
pub fn fixtures_only<'a>(args: &[&'a str]) -> Option<Vec<&'a str>> {
    let i = args.iter().position(|a| a.eq_ignore_ascii_case(MULTIPATCH.keyword))?;
    let mut rest = &args[i + 1..];
    while rest.first().is_some_and(|w| w.parse::<u32>().is_ok() || ["thru", "+", "-"].contains(&w.to_lowercase().as_str())) {
        rest = &rest[1..];
    }
    Some(args[..i].iter().chain(rest).copied().collect())
}

/// Write a copies' plan: each placed copy's first break.
fn write(state: &mut AppState, plan: &address::Plan, who: &BTreeMap<u32, (u32, usize)>, items: &[Place]) {
    let show = &mut state.root.show_data;
    for u in &plan.created {
        show.devices.dmx.patch.universes.insert(*u, lumacore_fixtures::Universe::new(None));
        show.devices.dmx.patch.universe_id_pool.reserve(*u);
    }
    for (key, u, start) in &plan.places {
        let (Some((no, i)), Some(guid)) = (who.get(key), show.universe_guid(*u)) else { continue };
        let fp = items.iter().find(|p| p.no == *key).map_or(1, |p| p.footprint);
        let Some(copy) = show.devices.dmx.patch.fixture_mut(no).and_then(|f| f.luma.multipatch.get_mut(*i)) else { continue };
        match copy.breaks.first_mut() {
            Some(b) => {
                b.universe_id = guid;
                b.start_address = *start;
            }
            None => copy.breaks.push(lumacore_fixtures::FixtureBreak::new(guid, *start, fp)),
        }
    }
    show.rebuild_occupancy();
}

/// The copies `ids` names on fixture `no` (indices), every one without ids.
fn existing(state: &AppState, no: u32, ids: Option<&Ids>) -> Result<Vec<usize>, String> {
    let have = state.root.show_data.devices.dmx.patch.fixture(no).map_or(0, |f| f.luma.multipatch.len());
    // A bare `Multipatch` names no copy here (round 3: only Store's bare one means "the next").
    let wanted: Vec<u32> = match ids {
        None => return Err(render_code(format!("Which copy of Fixture {no}? Name it: Multipatch 1 (it has {have})."), "E-CMD-006")),
        Some(ids) => numbers(Some(ids))?,
    };
    let missing: Vec<u32> = wanted.iter().copied().filter(|m| *m as usize > have || *m == 0).collect();
    if !missing.is_empty() || wanted.is_empty() {
        return Err(render_code(format!("Fixture {no} has {have} multipatch: no Multipatch {}.", address::numbers(&missing)), "E-CMD-007"));
    }
    Ok(wanted.into_iter().map(|m| m as usize - 1).collect())
}

/// The numbers typed (every one of a range).
fn numbers(ids: Option<&Ids>) -> Result<Vec<u32>, String> {
    let bad = || render_code("Multipatch takes numbers: Multipatch 1 Thru 2.", "E-CMD-008");
    let ids = ids.ok_or_else(bad)?;
    let whole = |id: &Option<Id>| match id {
        Some(Id::Number(n)) => n.node.parse::<u32>().ok().filter(|n| *n > 0).ok_or_else(bad),
        _ => Err(bad()),
    };
    let mut out = Vec::new();
    for (_, range) in &ids.ranges {
        match range {
            Range::One(id) => out.push(whole(&Some(id.clone()))?),
            Range::Thru { from, to, .. } => out.extend(whole(from)?..=whole(to)?),
        }
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
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
