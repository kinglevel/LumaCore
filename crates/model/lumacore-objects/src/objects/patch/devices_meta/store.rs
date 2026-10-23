// ███████╗████████╗ ██████╗ ██████╗ ███████╗
// ██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝
// ███████╗   ██║   ██║   ██║██████╔╝█████╗
// ╚════██║   ██║   ██║   ██║██╔══██╗██╔══╝
// ███████║   ██║   ╚██████╔╝██║  ██║███████╗
// ╚══════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: store

//! `Store MvrLayer [<n>] ["name"]` and `Store MvrGroup [<n>] ["name"]` (decisions §34 "34g4 part
//! 3c hardened"): a new, empty node at the slot or the next free number, in the Devices draft. A
//! new group sits in the first layer; `Move MvrGroup … At …` nests it elsewhere.
//!
//! Owns:          StoreMvrLayer, StoreMvrGroup.
//! Doesn't own:   the slot rule (`resolve::slot`), the claim (`patch_claimed`).
//! Entry points:  StoreMvrLayer, StoreMvrGroup.
//! Decisions:     §34 "34g4 part 3c hardened".

use lumacore_base::NodeMeta;
use lumacore_fixtures::{MvrGroup, MvrLayer};
use lumacore_syntax::ast::Target;

use super::Kind;
use crate::errors::render_code;
use crate::objects::{out, patch_claimed};
use crate::resolve::Slot;
use crate::{AppState, DispatchResult};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `Store MvrLayer [<n>] ["name"]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "MvrLayer")]
pub struct StoreMvrLayer {
    #[lum(target)]
    pub target: Option<Target>,
    #[lum(value)]
    pub name: Option<String>,
}

impl crate::commands::typed::Command for StoreMvrLayer {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        out(store(state, self.target.as_ref(), self.name, Kind::Layer).unwrap_or_else(|e| e))
    }
}

/// `Store MvrGroup [<n>] ["name"]`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "MvrGroup")]
pub struct StoreMvrGroup {
    #[lum(target)]
    pub target: Option<Target>,
    #[lum(value)]
    pub name: Option<String>,
}

impl crate::commands::typed::Command for StoreMvrGroup {
    fn run(self, state: &mut AppState) -> DispatchResult {
        if let Err(e) = patch_claimed(state) {
            return e;
        }
        out(store(state, self.target.as_ref(), self.name, Kind::Group).unwrap_or_else(|e| e))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn store(state: &mut AppState, target: Option<&Target>, name: Option<String>, kind: Kind) -> Result<String, String> {
    let word = kind.object().keyword;
    let taken: Vec<u32> = kind.rows(&state.root.show_data.devices.metadata.mvr).iter().map(|(n, _, _)| *n).collect();
    let next = crate::pools::door::next_free(&taken);
    let (n, name) = match crate::resolve::slot(state, target, kind.object())?.0 {
        Slot::Number(n) => (n, name),
        Slot::NewName(s) => (next, Some(s)),
        Slot::Next => (next, name),
        Slot::Key(k) => return Err(render_code(format!("'{k}' isn't an {word} number."), "E-CMD-008")),
    };
    if taken.contains(&n) {
        return Err(render_code(format!("{word} {n} exists: Label {word} {n} renames it."), "E-CMD-010"));
    }
    let meta = NodeMeta::new(name.filter(|s| !s.is_empty()));
    let md = &mut state.root.show_data.devices.metadata.mvr;
    let place = match kind {
        Kind::Layer => {
            md.layers.insert(n, MvrLayer { meta, matrix: Default::default(), order: 0 });
            String::new()
        }
        _ => {
            let Some(layer) = md.layers.values().next().map(|l| l.meta.identity.guid) else {
                return Err(render_code("Store an MvrLayer first: a group sits in a layer or a group.", "E-CMD-010"));
            };
            md.groups.insert(n, MvrGroup { meta, parent: layer, matrix: Default::default(), order: 0 });
            format!(" in {}", super::label(md, layer))
        }
    };
    state.root.show_data.rebuild_id_pools();
    Ok(format!("Stored {}{place}", super::label(&state.root.show_data.devices.metadata.mvr, guid_at(state, kind, n))))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

fn guid_at(state: &AppState, kind: Kind, n: u32) -> uuid::Uuid {
    kind.rows(&state.root.show_data.devices.metadata.mvr).into_iter().find(|(m, _, _)| *m == n).map(|(_, _, g)| g).unwrap_or_default()
}
