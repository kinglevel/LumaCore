// ███╗   ██╗███████╗███████╗████████╗
// ████╗  ██║██╔════╝██╔════╝╚══██╔══╝
// ██╔██╗ ██║█████╗  ███████╗   ██║
// ██║╚██╗██║██╔══╝  ╚════██║   ██║
// ██║ ╚████║███████╗███████║   ██║
// ╚═╝  ╚═══╝╚══════╝╚══════╝   ╚═╝
// FILE: nest

//! `Move` on the MVR tree (decisions §34 "34g4 part 3c hardened" 3, the one nesting rule):
//! `Move MvrGroup 3 At MvrGroup 2` / `At MvrLayer 1` makes it a child there, asked first (the
//! door's `nest_ask`; `/NoConfirmation` answers it); `Move … At <number>` is the door's
//! renumber (swap). A node never goes under itself or its own groups, and a layer is the top of
//! the tree: it only renumbers. GUIDs stay, so the fixtures in it go with it.
//!
//! Owns:          MoveMvrLayer, MoveMvrGroup.
//! Doesn't own:   the renumber (`commands::door::relocate`), the question (`door::nest_ask`).
//! Entry points:  MoveMvrLayer, MoveMvrGroup.
//! Decisions:     §34 "34g4 part 3c hardened" 3, 4.

use lumacore_syntax::ast::{Target, Value};
use uuid::Uuid;

use super::{Kind, counts_text, fixtures_under, groups_under, label, place_of, targeted};
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

/// `Move MvrLayer <n> At <n>`: renumber only (a layer nests nowhere).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Move", object = "MvrLayer")]
pub struct MoveMvrLayer {
    #[lum(target)]
    pub target: Target,
    #[lum(at = "destination")]
    pub to: Option<Value>,
}

impl crate::commands::typed::Command for MoveMvrLayer {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        if matches!(self.to, Some(Value::Target(_))) {
            return out(render_code("An MvrLayer is the top of the MVR tree: it nests nowhere (Move MvrLayer <n> At <number> renumbers it).", "E-PATCH-011"));
        }
        crate::commands::door::relocate(state, &super::MVR_LAYER, &self.target, self.to.as_ref(), Default::default())
    }
}

/// `Move MvrGroup <n> At MvrLayer <n> | MvrGroup <n> | <n> [/NoConfirmation]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Move", object = "MvrGroup")]
pub struct MoveMvrGroup {
    #[lum(target)]
    pub target: Target,
    #[lum(at = "destination")]
    pub to: Option<Value>,
    #[lum(option = "NoConfirmation")]
    pub confirmed: bool,
}

impl crate::commands::typed::Command for MoveMvrGroup {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        match &self.to {
            Some(Value::Target(to)) => nest(state, &self.target, to, self.confirmed).unwrap_or_else(out),
            _ => crate::commands::door::relocate(state, &super::MVR_GROUP, &self.target, self.to.as_ref(), Default::default()),
        }
    }
}

/// `Move Fixture <ids> At <node> | <number> [/Skip | /By n] [/NoConfirmation]` (decisions §40
/// "40d hardened" 3–4): At a node — a layer, a group, a fixture — nests the fixtures (a folder
/// with everything under it) after the one nesting question; At a bare number renumbers, as
/// the door has it; a folder has no number to move.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Move", object = "Fixture")]
pub struct MoveFixture {
    #[lum(target)]
    pub target: Target,
    #[lum(at = "destination")]
    pub to: Option<Value>,
    #[lum(option = "Skip")]
    pub skip: bool,
    #[lum(option = "By")]
    pub by: Option<Value>,
    #[lum(option = "NoConfirmation")]
    pub confirmed: bool,
}

impl crate::commands::typed::Command for MoveFixture {
    fn run(self, state: &mut AppState) -> DispatchResult {
        match &self.to {
            // Nesting changes the tree: the draft's. A renumber is the door's, as before.
            Some(Value::Target(to)) => match patch_claimed(state) {
                Err(e) => e,
                Ok(()) => nest_fixtures(state, &self.target, to, self.confirmed).unwrap_or_else(out),
            },
            _ => {
                // A folder has no number: it moves only into a node.
                match crate::resolve::fixture_guids(state, &self.target) {
                    Ok(guids) if guids.iter().any(|g| state.root.show_data.devices.dmx.patch.fixture_by_guid(*g).is_some_and(|(_, f)| f.mvr.fid.is_none())) => {
                        out(render_code("A folder has no number: Move it At a node (At MvrGroup <n>, At MvrLayer <n>, At Fixture <id>).", "E-PATCH-011"))
                    }
                    _ => {
                        let how = crate::commands::door::Renumber::of(self.skip, self.by.as_ref());
                        crate::commands::door::relocate(state, &crate::objects::FIXTURE, &self.target, self.to.as_ref(), how)
                    }
                }
            }
        }
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Fixtures (folders among them) into a node: the question first, then the parents set.
fn nest_fixtures(state: &mut AppState, target: &Target, to: &Target, confirmed: bool) -> Result<DispatchResult, String> {
    use super::nodes;
    let moved = crate::resolve::fixture_guids(state, target)?;
    let under = nodes::node_target(state, to)?.ok_or_else(|| render_code("Move Fixture … At names one node.", "E-CMD-008"))?;
    let devices = &state.root.show_data.devices;
    for g in &moved {
        if *g == under || nodes::is_under(devices, under, *g) {
            return Err(render_code(format!("{} can't go under itself or one of its own fixtures.", nodes::node_label(devices, *g)), "E-PATCH-011"));
        }
    }
    let handles: Vec<String> = moved.iter().map(|g| devices.dmx.patch.fixture_by_guid(*g).map(|(_, f)| f.handle()).unwrap_or_default()).collect();
    let line = format!("Move {} At {}", handles.join(" + "), nodes::node_word(devices, under));
    if !confirmed {
        let what = match moved.as_slice() {
            [g] => nodes::node_label(devices, *g),
            _ => format!("{} fixtures", moved.len()),
        };
        let folders: usize = moved.iter().map(|g| nodes::folders_under(devices, *g)).sum::<usize>()
            + moved.iter().filter(|g| devices.dmx.patch.fixture_by_guid(**g).is_some_and(|(_, f)| f.mvr.fid.is_none())).count();
        let fixtures: usize = moved.iter().map(|g| nodes::fixtures_under(devices, *g).len()).sum::<usize>()
            + moved.iter().filter(|g| devices.dmx.patch.fixture_by_guid(**g).is_some_and(|(_, f)| f.mvr.fid.is_some())).count();
        let s = |n: usize| if n == 1 { "" } else { "s" };
        let counts = format!("{folders} folder{}, {fixtures} fixture{}", s(folders), s(fixtures));
        return Ok(crate::commands::door::nest_ask(&what, &nodes::node_label(devices, under), &counts, &line));
    }
    let to_label = nodes::node_label(devices, under);
    let mut order = devices.dmx.patch.next_order(Some(under));
    let patch = &mut state.root.show_data.devices.dmx.patch;
    let mut done = 0;
    for g in &moved {
        if let Some((_, f)) = patch.fixture_by_guid_mut(*g) {
            f.mvr.parent = Some(under);
            f.mvr.order = order;
            order += 10;
            done += 1;
        }
    }
    Ok(out(format!("Moved {done} into {to_label}")))
}

fn nest(state: &mut AppState, target: &Target, to: &Target, confirmed: bool) -> Result<DispatchResult, String> {
    let moved = targeted(state, target, Kind::Group)?;
    let under = destination(state, to)?;
    let md = &state.root.show_data.devices.metadata.mvr;
    for (_, g) in &moved {
        if *g == under || groups_under(md, *g).contains(&under) {
            return Err(render_code(format!("{} can't go under itself or one of its own groups.", label(md, *g)), "E-PATCH-011"));
        }
    }
    if !confirmed {
        let what = match moved.as_slice() {
            [(_, g)] => label(md, *g),
            _ => format!("{} MvrGroups", moved.len()),
        };
        let devices = &state.root.show_data.devices;
        let mut groups: Vec<Uuid> = moved.iter().map(|(_, g)| *g).collect();
        for (_, g) in &moved {
            groups.extend(groups_under(md, *g));
        }
        groups.sort();
        groups.dedup();
        let fixtures: usize = moved.iter().map(|(_, g)| fixtures_under(devices, *g).len()).sum();
        let numbers: Vec<u32> = moved.iter().map(|(n, _)| *n).collect();
        let line = format!("Move MvrGroup {} At {}", crate::commands::door::numbers_text(&numbers), word_of(md, under));
        return Ok(crate::commands::door::nest_ask(&what, &label(md, under), &counts_text(groups.len(), fixtures), &line));
    }
    let to_label = label(md, under);
    let md = &mut state.root.show_data.devices.metadata.mvr;
    let mut done: Vec<String> = Vec::new();
    for (n, _) in &moved {
        if let Some(g) = md.groups.get_mut(n) {
            g.parent = under;
            done.push(n.to_string());
        }
    }
    Ok(out(format!("Moved MvrGroup {} into {to_label}", done.join(", "))))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The one layer or group `At` names.
fn destination(state: &AppState, to: &Target) -> Result<Uuid, String> {
    let usage = || render_code("Move MvrGroup … At MvrLayer <n>, At MvrGroup <n> or At <number>.", "E-CMD-008");
    let items = crate::resolve::resolve(state, to, &super::MVR_GROUP)?;
    let [(None, r)] = items.as_slice() else { return Err(usage()) };
    if !std::ptr::eq(r.object, &super::MVR_GROUP) && !std::ptr::eq(r.object, &super::MVR_LAYER) {
        return Err(usage());
    }
    let [m] = r.members.as_slice() else { return Err(render_code("Move MvrGroup … At names one layer or group.", "E-CMD-008")) };
    m.guid.ok_or_else(|| render_code(format!("No such {}.", r.object.keyword), "E-CMD-007"))
}

/// `MvrGroup 2` / `MvrLayer 1`, as a line names it.
fn word_of(md: &lumacore_fixtures::MvrMetadata, node: Uuid) -> String {
    match place_of(md, node) {
        Some(super::Place::Layer(n)) => format!("MvrLayer {n}"),
        Some(super::Place::Group(n)) => format!("MvrGroup {n}"),
        None => String::new(),
    }
}
