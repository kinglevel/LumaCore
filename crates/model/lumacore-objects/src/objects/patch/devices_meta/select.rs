// ███████╗███████╗██╗     ███████╗ ██████╗████████╗
// ██╔════╝██╔════╝██║     ██╔════╝██╔════╝╚══██╔══╝
// ███████╗█████╗  ██║     █████╗  ██║        ██║
// ╚════██║██╔══╝  ██║     ██╔══╝  ██║        ██║
// ███████║███████╗███████╗███████╗╚██████╗   ██║
// ╚══════╝╚══════╝╚══════╝╚══════╝ ╚═════╝   ╚═╝
// FILE: select

//! A bare `MvrGroup 3` / `MvrLayer 1` / `MvrClass 2` / `MvrPosition 1` (their default verb,
//! decisions §34 "34g4 part 3c hardened" 1) selects the fixtures in it — a layer or a group with
//! every group under it — by the one selection rule; `List` / `Info` read the pools.
//!
//! Owns:          Select / List / Info of the four MVR words.
//! Doesn't own:   the selection rule (AppState::select_fixtures), the tree (devices_meta.rs).
//! Entry points:  SelectMvrLayer, …, ListMvrLayer, …, InfoMvrLayer, ….
//! Decisions:     §34 "34g4 part 3c hardened" 1.

use lumacore_syntax::ast::Target;
use uuid::Uuid;

use super::{Kind, counts_text, fixtures_under, groups_under, label, targeted};
use crate::errors::render_code;
use crate::objects::out;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// The three read verbs of one word.
macro_rules! read_verbs {
    ($select:ident, $list:ident, $info:ident, $word:literal, $kind:expr) => {
        #[doc = concat!("`", $word, " <n>` = `SelectFixtures ", $word, " <n>`.")]
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = "SelectFixtures", object = $word)]
        pub struct $select {
            #[lum(target)]
            pub target: Target,
        }
        impl crate::commands::typed::Command for $select {
            fn run(self, state: &mut AppState) -> DispatchResult {
                out(select(state, &self.target, $kind).unwrap_or_else(|e| e))
            }
        }

        #[doc = concat!("`List ", $word, " [<n>]`.")]
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = "List", object = $word)]
        pub struct $list {
            #[lum(target)]
            pub target: Option<Target>,
        }
        impl crate::commands::typed::Command for $list {
            fn run(self, state: &mut AppState) -> DispatchResult {
                out(list(state, self.target.as_ref(), $kind).unwrap_or_else(|e| e))
            }
        }

        #[doc = concat!("`Info ", $word, " [<n>]`.")]
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = "Info", object = $word)]
        pub struct $info {
            #[lum(target)]
            pub target: Option<Target>,
        }
        impl crate::commands::typed::Command for $info {
            fn run(self, state: &mut AppState) -> DispatchResult {
                out(list(state, self.target.as_ref(), $kind).unwrap_or_else(|e| e))
            }
        }
    };
}

read_verbs!(SelectMvrLayer, ListMvrLayer, InfoMvrLayer, "MvrLayer", Kind::Layer);
read_verbs!(SelectMvrGroup, ListMvrGroup, InfoMvrGroup, "MvrGroup", Kind::Group);
read_verbs!(SelectMvrClass, ListMvrClass, InfoMvrClass, "MvrClass", Kind::Class);
read_verbs!(SelectMvrPosition, ListMvrPosition, InfoMvrPosition, "MvrPosition", Kind::Position);

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn select(state: &mut AppState, target: &Target, kind: Kind) -> Result<String, String> {
    let nodes = targeted(state, target, kind)?;
    let mut fixtures: Vec<u32> = Vec::new();
    for (_, g) in &nodes {
        for n in members(state, kind, *g) {
            if !fixtures.contains(&n) {
                fixtures.push(n);
            }
        }
    }
    if fixtures.is_empty() {
        return Err(render_code(format!("No fixtures in {}.", names(state, kind, &nodes)), "E-CMD-009"));
    }
    let n = state.select_fixtures(&fixtures, lumacore_tree::programmer::SelectMode::Plain)?;
    Ok(format!("{} fixture{} from {} · {n} selected", fixtures.len(), if fixtures.len() == 1 { "" } else { "s" }, names(state, kind, &nodes)))
}

/// One line per node: where it is and what is under it.
fn list(state: &AppState, target: Option<&Target>, kind: Kind) -> Result<String, String> {
    let md = &state.root.show_data.devices.metadata.mvr;
    let rows: Vec<(u32, Uuid)> = match target {
        Some(t) => targeted(state, t, kind)?,
        None => kind.rows(md).into_iter().map(|(n, _, g)| (n, g)).collect(),
    };
    if rows.is_empty() {
        return Ok(format!("No {} yet.", kind.object().keyword));
    }
    let devices = &state.root.show_data.devices;
    let lines: Vec<String> = rows
        .iter()
        .map(|(n, g)| {
            let here = match kind {
                Kind::Layer | Kind::Group => label(md, *g),
                _ => one_label(state, kind, *n),
            };
            let place = match kind {
                Kind::Group => md.groups.get(n).map(|grp| format!(" · in {}", label(md, grp.parent))).unwrap_or_default(),
                _ => String::new(),
            };
            let under = match kind {
                Kind::Layer | Kind::Group => counts_text(groups_under(md, *g).len(), fixtures_under(devices, *g).len()),
                _ => {
                    let n = members(state, kind, *g).len();
                    format!("{n} fixture{}", if n == 1 { "" } else { "s" })
                }
            };
            format!("{here}{place} · {under}")
        })
        .collect();
    Ok(lines.join("\n"))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The fixtures a node reaches: in it and under it (a layer, a group), of it (a class, a position).
fn members(state: &AppState, kind: Kind, node: Uuid) -> Vec<u32> {
    let devices = &state.root.show_data.devices;
    let of = |pick: fn(&lumacore_fixtures::MvrFixture) -> Option<Uuid>| {
        devices.dmx.patch.fixtures.values().filter(|f| pick(&f.mvr) == Some(node)).filter_map(|f| f.mvr.fid).collect()
    };
    match kind {
        Kind::Layer | Kind::Group => fixtures_under(devices, node),
        Kind::Class => of(|m| m.class),
        Kind::Position => of(|m| m.position),
    }
}

fn one_label(state: &AppState, kind: Kind, n: u32) -> String {
    let word = kind.object().keyword;
    match kind.rows(&state.root.show_data.devices.metadata.mvr).into_iter().find(|(m, _, _)| *m == n).and_then(|(_, name, _)| name) {
        Some(s) => format!("{word} {n} \"{s}\""),
        None => format!("{word} {n}"),
    }
}

fn names(state: &AppState, kind: Kind, nodes: &[(u32, Uuid)]) -> String {
    let md = &state.root.show_data.devices.metadata.mvr;
    nodes
        .iter()
        .map(|(n, g)| match kind {
            Kind::Layer | Kind::Group => label(md, *g),
            _ => one_label(state, kind, *n),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
