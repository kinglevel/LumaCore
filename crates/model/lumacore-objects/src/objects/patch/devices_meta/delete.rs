// ██████╗ ███████╗██╗     ███████╗████████╗███████╗
// ██╔══██╗██╔════╝██║     ██╔════╝╚══██╔══╝██╔════╝
// ██║  ██║█████╗  ██║     █████╗     ██║   █████╗
// ██║  ██║██╔══╝  ██║     ██╔══╝     ██║   ██╔══╝
// ██████╔╝███████╗███████╗███████╗   ██║   ███████╗
// ╚═════╝ ╚══════╝╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: delete

//! `Delete MvrLayer <n>` / `Delete MvrGroup <n>` (decisions §34 round 3 18, 19): a node that holds
//! something asks once — Keep the fixtures (the default, `/KeepChildren`: its groups and fixtures
//! move to its parent; under a layer the fixtures go to No MVR group, its groups with it) or
//! Delete them too (`/WithChildren`, naming what uses them) — the station's one Delete ask
//! (`ObjectDef::delete_ask`).
//!
//! Owns:          ask (the two words' Delete question), DeleteMvrLayer, DeleteMvrGroup.
//! Doesn't own:   the unpatch (`commands::patch::unpatch_ids`), the question's place (dispatch).
//! Entry points:  ask, DeleteMvrLayer, DeleteMvrGroup.
//! Decisions:     §34 "34g4 patch hardened" 12, §30 "Merge of bugfixes" (one Delete ask).

use lumacore_syntax::ast::Target;
use lumacore_wire::ButtonKind;
use uuid::Uuid;

use super::{Kind, counts_text, fixtures_under, groups_under, label, targeted};
use crate::objects::{DeleteAsk, out, patch_claimed};
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Delete MvrLayer <n> [/KeepChildren | /WithChildren]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Delete", object = "MvrLayer")]
pub struct DeleteMvrLayer {
    #[lum(target)]
    pub target: Target,
    #[lum(option = "KeepChildren")]
    pub keep_children: bool,
    #[lum(option = "WithChildren")]
    pub with_children: bool,
}

impl crate::commands::typed::Command for DeleteMvrLayer {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        if self.keep_children && self.with_children {
            return out(crate::errors::render_code("/KeepChildren or /WithChildren, not both.", "E-CMD-006"));
        }
        out(delete(state, &self.target, Kind::Layer, self.with_children).unwrap_or_else(|e| e))
    }
}

/// `Delete MvrGroup <n> [/KeepChildren | /WithChildren]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Delete", object = "MvrGroup")]
pub struct DeleteMvrGroup {
    #[lum(target)]
    pub target: Target,
    #[lum(option = "KeepChildren")]
    pub keep_children: bool,
    #[lum(option = "WithChildren")]
    pub with_children: bool,
}

impl crate::commands::typed::Command for DeleteMvrGroup {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        if self.keep_children && self.with_children {
            return out(crate::errors::render_code("/KeepChildren or /WithChildren, not both.", "E-CMD-006"));
        }
        out(delete(state, &self.target, Kind::Group, self.with_children).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `with_children`: the groups under the nodes and their fixtures go too; else (the default,
/// `/KeepChildren`) they move up: a group's children to its parent, a layer's fixtures to No MVR
/// group (its groups have no layer to stay in, so they go, empty).
fn delete(state: &mut AppState, target: &Target, kind: Kind, with_children: bool) -> Result<String, String> {
    let nodes = targeted(state, target, kind)?;
    let (groups, fixtures) = reach(state, &nodes);
    let names: Vec<String> = nodes.iter().map(|(_, g)| label(&state.root.show_data.devices.metadata.mvr, *g)).collect();
    let gone: Vec<Uuid> = nodes.iter().map(|(_, g)| *g).collect();
    let mut lines = Vec::new();
    if with_children {
        lines.push(format!("Deleted {} ({})", names.join(", "), counts_text(groups.len(), fixtures.len())));
        if !fixtures.is_empty() {
            lines.push(crate::commands::patch::unpatch_ids(state, &fixtures));
        }
    } else {
        // Up one level: each node's parent, past the nodes going too; a layer has none.
        let md = &state.root.show_data.devices.metadata.mvr;
        let up = |mut g: Uuid| -> Option<Uuid> {
            while gone.contains(&g) {
                g = md.groups.values().find(|x| x.meta.identity.guid == g)?.parent;
            }
            Some(g)
        };
        let moves: Vec<(Uuid, Option<Uuid>)> = gone.iter().map(|g| (*g, up(*g))).collect();
        let layer_groups: Vec<Uuid> = if kind == Kind::Layer { groups.clone() } else { Vec::new() };
        let md = &mut state.root.show_data.devices.metadata.mvr;
        for g in md.groups.values_mut() {
            if let Some((_, Some(to))) = moves.iter().find(|(from, _)| *from == g.parent) {
                g.parent = *to;
            }
        }
        md.groups.retain(|_, g| !layer_groups.contains(&g.meta.identity.guid));
        let lost = [gone.as_slice(), layer_groups.as_slice()].concat();
        let mut kept = 0;
        for f in state.root.show_data.devices.dmx.patch.fixtures.values_mut() {
            if let Some(p) = f.mvr.parent.filter(|p| lost.contains(p)) {
                f.mvr.parent = if kind == Kind::Layer { None } else { moves.iter().find(|(from, _)| *from == p).and_then(|(_, to)| *to) };
                kept += 1;
            }
        }
        let kept = if kept == 0 { String::new() } else { format!(" · {kept} fixture{} kept, one level up", if kept == 1 { "" } else { "s" }) };
        lines.push(format!("Deleted {}{kept}", names.join(", ")));
    }
    let md = &mut state.root.show_data.devices.metadata.mvr;
    if with_children {
        md.groups.retain(|_, g| !groups.contains(&g.meta.identity.guid));
    }
    md.groups.retain(|_, g| !gone.contains(&g.meta.identity.guid));
    md.layers.retain(|_, l| !gone.contains(&l.meta.identity.guid));
    state.root.show_data.rebuild_id_pools();
    Ok(lines.join("\n"))
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The Delete question: the nodes, the groups and fixtures that go with them, what uses those.
pub fn ask(state: &AppState, kind: Kind, args: &[&str]) -> DeleteAsk {
    let word = kind.object().keyword;
    let words: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with('/')).collect();
    let nodes = match crate::resolve::target_of(&format!("{word} {}", words.join(" "))).and_then(|t| targeted(state, &t, kind)) {
        Ok(n) => n,
        Err(e) => return DeleteAsk::Refuse(e),
    };
    let md = &state.root.show_data.devices.metadata.mvr;
    let (groups, fixtures) = reach(state, &nodes);
    let mut text = match nodes.as_slice() {
        [(_, g)] => format!("Delete {}?", label(md, *g)),
        _ => format!("Delete {} {word}s?", nodes.len()),
    };
    let holds = !groups.is_empty() || !fixtures.is_empty();
    if holds {
        text.push_str(&format!("\nIt holds {}.", counts_text(groups.len(), fixtures.len())));
    }
    let fixture_guids: Vec<Uuid> = fixtures.iter().filter_map(|n| state.root.show_data.devices.dmx.patch.fixture(n)).map(|f| f.meta.identity.guid).collect();
    let mut users: Vec<String> = fixture_guids.iter().flat_map(|g| crate::pools::door::users_of(&state.root.show_data, *g)).collect();
    users.sort();
    users.dedup();
    if !users.is_empty() {
        let shown: Vec<&str> = users.iter().take(4).map(String::as_str).collect();
        let more = users.len().saturating_sub(4);
        text.push_str(&format!("\nDeleting the fixtures too leaves {}{} broken and listed.", shown.join(", "), if more > 0 { format!(" and {more} more") } else { String::new() }));
    }
    let numbers: Vec<u32> = nodes.iter().map(|(n, _)| *n).collect();
    let line = format!("Delete {word} {}", crate::commands::door::numbers_text(&numbers));
    let ask = crate::dialogs::Ask::new("Delete", text);
    let ask = if holds {
        // Keep the fixtures is the default (decisions §34 round 3 18).
        ask.button("Keep the fixtures", ButtonKind::Primary, format!("{line} /KeepChildren /NoConfirmation"))
            .button("Delete the fixtures too", ButtonKind::Danger, format!("{line} /WithChildren /NoConfirmation"))
    } else {
        ask.button("Delete", ButtonKind::Danger, format!("{line} /NoConfirmation"))
    };
    DeleteAsk::Ask(Box::new(ask.cancel("Cancel").flags("/NoConfirmation /KeepChildren /WithChildren")))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The groups under the nodes (not the nodes) and the fixtures in all of them.
fn reach(state: &AppState, nodes: &[(u32, Uuid)]) -> (Vec<Uuid>, Vec<u32>) {
    let devices = &state.root.show_data.devices;
    let mut groups: Vec<Uuid> = Vec::new();
    let mut fixtures: Vec<u32> = Vec::new();
    for (_, g) in nodes {
        for s in groups_under(&devices.metadata.mvr, *g) {
            if !groups.contains(&s) && !nodes.iter().any(|(_, n)| *n == s) {
                groups.push(s);
            }
        }
        for f in fixtures_under(devices, *g) {
            if !fixtures.contains(&f) {
                fixtures.push(f);
            }
        }
    }
    fixtures.sort_unstable();
    (groups, fixtures)
}
