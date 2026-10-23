//  █████╗ ███████╗███████╗██╗ ██████╗ ███╗   ██╗
// ██╔══██╗██╔════╝██╔════╝██║██╔════╝ ████╗  ██║
// ███████║███████╗███████╗██║██║  ███╗██╔██╗ ██║
// ██╔══██║╚════██║╚════██║██║██║   ██║██║╚██╗██║
// ██║  ██║███████║███████║██║╚██████╔╝██║ ╚████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: assign

//! `Assign Fixture … At MvrGroup <n>` / `At MvrLayer <n>` puts fixtures in a node of the MVR tree
//! (decisions §34 "34g4 part 3c hardened" 2; lum.md: putting one object into another is Assign).
//! A fixture has one place (MVR's one parent), so it leaves its old group. `Assign Fixture … At
//! DMX …` gives them addresses (§34 round 3 19, the placement rule in `fixture::address`).
//! `Unassign` always names the kind it takes away: `At DMX [n[.aaa]]` addresses, `At MvrGroup
//! [n]` / `At MvrLayer [n]` the place (a bare kind = whichever it has); a fixture alone is refused.
//!
//! Owns:          AssignFixture, UnassignFixture.
//! Doesn't own:   the tree (devices_meta.rs), the claim (`patch_claimed`), where fixtures land
//!                (`fixture::address`).
//! Entry points:  AssignFixture, UnassignFixture.
//! Decisions:     §34 "34g4 part 3c hardened" 2, round 3 19.

use lumacore_syntax::ast::{Target, Value};
use uuid::Uuid;

use super::label;
use crate::objects::patch::dmx;
use crate::objects::patch::fixture::address;
use crate::errors::render_code;
use crate::objects::{out, patch_claimed};
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Assign Fixture <ids> At MvrGroup <n> | MvrLayer <n> | MvrClass <n> | DMX [<u>[.<a>]] [/Every n | /KeepSpacing]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Assign", object = "Fixture")]
pub struct AssignFixture {
    #[lum(target)]
    pub target: Target,
    #[lum(at = "destination")]
    pub to: Value,
    #[lum(option = "Every")]
    pub every: Option<Value>,
    #[lum(option = "KeepSpacing")]
    pub keep_spacing: bool,
    #[lum(option = "Swap")]
    pub swap: bool,
}

impl crate::commands::typed::Command for AssignFixture {
    fn run(self, state: &mut AppState) -> DispatchResult {
        // An address is a live column (decisions §34 round 3, 18: Patch to… and Next free work in
        // Live): `At DMX` needs no claim; under another's draft it runs live (`live_draft`).
        let dest = dmx::dest_of(state, &self.to);
        if dest.is_none()
            && let Err(e) = patch_claimed(state)
        {
            return e;
        }
        if let Some(dest) = dest {
            if let Some((fixtures, copies)) = crate::objects::patch::fixture::multipatch::split(&self.target) {
                return out(dest.and_then(|d| crate::objects::patch::fixture::multipatch::readdress(state, &fixtures, copies.as_ref(), d)).unwrap_or_else(|e| e));
            }
            return out(dest.and_then(|d| if self.swap { swap(state, &self.target, d) } else { readdress(state, &self, d) }).unwrap_or_else(|e| e));
        }
        if self.every.is_some() || self.keep_spacing || self.swap {
            return out(render_code("/Every and /KeepSpacing space addresses: Assign Fixture … At DMX ….", "E-CMD-006"));
        }
        // A fixture's one MVR class (round 3, the left-out items), like its MVR group.
        if let Some(class) = class_of(state, &self.to) {
            return out(class.and_then(|c| set_class(state, &self.target, c)).unwrap_or_else(|e| e));
        }
        out(place(state, &self.target, Some(&self.to)).unwrap_or_else(|e| e))
    }
}

/// `Unassign Fixture <ids> At DMX [<u>[.<a>]] | MvrGroup [<n>] | MvrLayer [<n>]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Unassign", object = "Fixture")]
pub struct UnassignFixture {
    #[lum(target)]
    pub target: Target,
    #[lum(at = "destination")]
    pub to: Option<Value>,
}

impl crate::commands::typed::Command for UnassignFixture {
    fn run(self, state: &mut AppState) -> DispatchResult {
        // Tags are a live column: no claim (decisions §34 "34g4 patch hardened" 8).
        if let Some(done) = self.to.as_ref().and_then(|to| untag(state, &self.target, to)) {
            return out(done.unwrap_or_else(|e| e));
        }
        // Its addresses are a live column: `At DMX` needs no claim (round 3, 18).
        if !self.to.as_ref().is_some_and(|to| dmx::dest_of(state, to).is_some())
            && let Err(e) = patch_claimed(state)
        {
            return e;
        }
        // Unassign names what it takes away (decisions §34 round 3 19).
        let Some(to) = &self.to else {
            return out(render_code(
                "Unassign Fixture … At DMX (its addresses) | At MvrGroup / MvrLayer (its place): say what it loses.",
                "E-CMD-006",
            ));
        };
        if let Some(dest) = dmx::dest_of(state, to) {
            return out(dest.and_then(|d| unaddress(state, &self.target, d)).unwrap_or_else(|e| e));
        }
        if class_of(state, to).is_some() {
            return out(set_class(state, &self.target, None).unwrap_or_else(|e| e));
        }
        out(unplace(state, &self.target, Some(to)).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn place(state: &mut AppState, target: &Target, to: Option<&Value>) -> Result<String, String> {
    // By GUID: a folder (no number) goes into a node like any fixture (decisions §40 "40d hardened" 3).
    let guids = crate::resolve::fixture_guids(state, target)?;
    let node = node_of(state, to)?.ok_or_else(|| render_code("Assign Fixture … At MvrGroup <n>, At MvrLayer <n> or At Fixture <id>.", "E-CMD-008"))?;
    // Never under itself or something it holds (a fixture holding fixtures is a node too).
    let devices = &state.root.show_data.devices;
    for g in &guids {
        if *g == node || super::nodes::is_under(devices, node, *g) {
            return Err(render_code(format!("{} can't go under itself or one of its own fixtures.", super::nodes::node_label(devices, *g)), "E-PATCH-011"));
        }
    }
    let handles: Vec<String> = guids.iter().filter_map(|g| devices.dmx.patch.fixture_by_guid(*g).map(|(_, f)| f.handle())).collect();
    let numbers: Vec<u32> = guids.iter().filter_map(|g| devices.dmx.patch.fixture_by_guid(*g).and_then(|(_, f)| f.mvr.fid)).collect();
    let next_order = devices.dmx.patch.next_order(Some(node));
    let all = &mut state.root.show_data.devices.dmx.patch;
    let mut moved = 0;
    for g in &guids {
        if let Some((_, f)) = all.fixture_by_guid_mut(*g)
            && f.mvr.parent != Some(node)
        {
            f.mvr.parent = Some(node);
            // Last among the node's children, in the typed order (decisions §40 6).
            f.mvr.order = next_order + 10 * moved as u32;
            moved += 1;
        }
    }
    let into = super::nodes::node_label(&state.root.show_data.devices, node);
    let what = if numbers.len() == guids.len() { fixtures_text(&numbers) } else { handles.join(" + ") };
    Ok(format!("{what} into {into}{}", if moved < guids.len() { format!(" ({} already there)", guids.len() - moved) } else { String::new() }))
}

/// `Assign Fixture … At DMX …`: the placement rule, then the addresses written.
fn readdress(state: &mut AppState, cmd: &AssignFixture, dest: dmx::Dest) -> Result<String, String> {
    let fixtures = super::super::numbers(state, &cmd.target, &crate::objects::FIXTURE)?;
    let every = match &cmd.every {
        Some(Value::Number(n)) => Some(n.node.parse::<u16>().ok().filter(|n| (1..=512).contains(n)).ok_or_else(|| render_code("/Every takes slots, 1–512.", "E-CMD-008"))?),
        Some(_) => return Err(render_code("/Every takes slots, 1–512.", "E-CMD-008")),
        None => None,
    };
    let show = &state.root.show_data;
    let items = address::items(show, &fixtures)?;
    let plan = address::plan(show, &items, dest, address::Spacing { every, keep: cmd.keep_spacing })?;
    let footprints = items.iter().map(|i| (i.no, i.footprint)).collect();
    address::apply(&mut state.root.show_data, &plan, &footprints);
    let mut lines = vec![crate::text::render_success(&format!("{} at {}", fixtures_text(&fixtures), address::placed_text(&plan)))];
    lines.extend(address::plan_notes(&plan));
    Ok(lines.join("\n"))
}

/// `Assign Fixture A At Fixture B DMX /Swap`: the pairs trade addresses (round 3 19).
fn swap(state: &mut AppState, target: &Target, dest: dmx::Dest) -> Result<String, String> {
    let dmx::Dest::Of { fixtures: others, brk } = dest else {
        return Err(render_code("/Swap trades with fixtures: Assign Fixture … At Fixture … DMX /Swap.", "E-CMD-006"));
    };
    let fixtures = super::super::numbers(state, target, &crate::objects::FIXTURE)?;
    let pairs = address::swap(&mut state.root.show_data, &fixtures, &others, brk);
    if pairs.is_empty() {
        return Err(render_code("No pair to trade.", "E-CMD-009"));
    }
    let text: Vec<String> = pairs.iter().map(|(a, b)| format!("{a} ↔ {b}")).collect();
    Ok(crate::text::render_success(&format!("Addresses traded: {}", text.join(" · "))))
}

/// `Unassign Fixture … At DMX …`: every address, those in a universe, or the one at an address.
fn unaddress(state: &mut AppState, target: &Target, dest: dmx::Dest) -> Result<String, String> {
    let fixtures = super::super::numbers(state, target, &crate::objects::FIXTURE)?;
    let lost = address::unassign(&mut state.root.show_data, &fixtures, dest);
    if lost.is_empty() {
        return Err(render_code("None of them has an address there.", "E-CMD-009"));
    }
    Ok(format!("{} lost {} address", fixtures_text(&lost), if lost.len() == 1 { "its" } else { "their" }))
}

/// `Unassign Fixture … At Tag [path]`: that tag off them, every tag with the bare kind (round 3
/// 19). The Tag object is known by its tree node (its crate sits above this one). None = not a tag.
fn untag(state: &mut AppState, target: &Target, to: &Value) -> Option<Result<String, String>> {
    let Value::Target(t) = to else { return None };
    let [(None, lumacore_syntax::ast::Item::Path(segs))] = t.items.as_slice() else { return None };
    let vocab = crate::lum::vocabulary();
    let object = crate::objects::find(&vocab.word(segs.last()?.object.node).name)?;
    if !object.tree.as_ref().is_some_and(|tr| tr.node == crate::tree_path::Node::Tags) {
        return None;
    }
    Some((|| {
        let fixtures = super::super::numbers(state, target, &crate::objects::FIXTURE)?;
        let only: Option<Vec<Uuid>> = match segs.last().and_then(|s| s.ids.as_ref()) {
            None => None,
            Some(_) => Some(crate::resolve::resolve_one(state, t, object)?.members.iter().filter_map(|m| m.guid).collect()),
        };
        let mut lost = Vec::new();
        for no in &fixtures {
            let Some(f) = state.root.show_data.devices.dmx.patch.fixture_mut(no) else { continue };
            let tags = &mut f.meta.annotation.tags;
            let before = tags.len();
            tags.retain(|g| only.as_ref().is_some_and(|o| !o.contains(g)));
            if tags.len() < before {
                lost.push(*no);
            }
        }
        if lost.is_empty() {
            return Err(render_code("None of them carries that tag.", "E-CMD-009"));
        }
        Ok(format!("{} lost the tag{}", fixtures_text(&lost), if only.is_none() { "s" } else { "" }))
    })())
}

fn unplace(state: &mut AppState, target: &Target, to: Option<&Value>) -> Result<String, String> {
    let fixtures = super::super::numbers(state, target, &crate::objects::FIXTURE)?;
    let only = node_of(state, to)?;
    let all = &mut state.root.show_data.devices.dmx.patch;
    let mut out: Vec<u32> = Vec::new();
    for n in &fixtures {
        if let Some(f) = all.fixture_mut(n)
            && f.mvr.parent.is_some()
            && (only.is_none() || f.mvr.parent == only)
        {
            f.mvr.parent = None;
            out.push(*n);
        }
    }
    if out.is_empty() {
        let from = only.map(|g| format!(" in {}", label(&state.root.show_data.devices.metadata.mvr, g))).unwrap_or_else(|| " in an MVR group".into());
        return Err(render_code(format!("None of them is{from}."), "E-CMD-009"));
    }
    Ok(format!("{} now in No MVR group", fixtures_text(&out)))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// `At MvrClass [n]`: Some when `At` names the class kind — the class (None for a bare
/// `MvrClass`, which Unassign takes away), or why it can't be one.
fn class_of(state: &AppState, to: &Value) -> Option<Result<Option<Uuid>, String>> {
    let Value::Target(t) = to else { return None };
    let items = crate::resolve::resolve(state, t, &super::MVR_CLASS).ok()?;
    let [(None, r)] = items.as_slice() else { return None };
    if !std::ptr::eq(r.object, &super::MVR_CLASS) {
        return None;
    }
    if !r.named {
        return Some(Ok(None));
    }
    Some(match super::targeted(state, t, super::Kind::Class) {
        Ok(found) => match found.as_slice() {
            [(_, g)] => Ok(Some(*g)),
            _ => Err(render_code("At names one class.", "E-CMD-008")),
        },
        Err(e) => Err(e),
    })
}

/// Puts the fixtures in `class` (None: no class).
fn set_class(state: &mut AppState, target: &Target, class: Option<Uuid>) -> Result<String, String> {
    let fixtures = super::super::numbers(state, target, &crate::objects::FIXTURE)?;
    if class.is_none() && !fixtures.iter().any(|n| state.root.show_data.devices.dmx.patch.fixture(n).is_some_and(|f| f.mvr.class.is_some())) {
        return Err(render_code("None of them has an MVR class.", "E-CMD-009"));
    }
    let named = class.and_then(|g| state.root.show_data.devices.metadata.mvr.classes.values().find(|c| c.meta.identity.guid == g)).map(|c| c.meta.identity.name.clone().unwrap_or_default());
    for n in &fixtures {
        if let Some(f) = state.root.show_data.devices.dmx.patch.fixture_mut(n) {
            f.mvr.class = class;
        }
    }
    Ok(match named {
        Some(name) => format!("{} in class {name}", fixtures_text(&fixtures)),
        None => format!("{} without an MVR class", fixtures_text(&fixtures)),
    })
}

/// The one node `At` names — a layer, a group or a fixture that holds fixtures (decisions §40
/// "40d hardened" 3); None without `At` or for a bare kind (`At MvrGroup`: whichever the
/// fixture is in, decisions §34 round 3 19).
fn node_of(state: &AppState, to: Option<&Value>) -> Result<Option<Uuid>, String> {
    let Some(to) = to else { return Ok(None) };
    let Value::Target(t) = to else {
        return Err(render_code("At MvrGroup <n>, At MvrLayer <n> or At Fixture <id>: a fixture goes in a node of the tree.", "E-PATCH-011"));
    };
    super::nodes::node_target(state, t)
}

fn fixtures_text(nos: &[u32]) -> String {
    match nos {
        [one] => format!("Fixture {one}"),
        _ => format!("{} fixtures ({})", nos.len(), crate::commands::door::numbers_text(nos)),
    }
}
