// ███╗   ██╗███████╗████████╗██╗    ██╗ ██████╗ ██████╗ ██╗  ██╗
// ████╗  ██║██╔════╝╚══██╔══╝██║    ██║██╔═══██╗██╔══██╗██║ ██╔╝
// ██╔██╗ ██║█████╗     ██║   ██║ █╗ ██║██║   ██║██████╔╝█████╔╝
// ██║╚██╗██║██╔══╝     ██║   ██║███╗██║██║   ██║██╔══██╗██╔═██╗
// ██║ ╚████║███████╗   ██║   ╚███╔███╔╝╚██████╔╝██║  ██║██║  ██╗
// ╚═╝  ╚═══╝╚══════╝   ╚═╝    ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
// FILE: network

//! `Store Network "Rig"` names a data network the show uses; each station maps it to a NIC in its
//! row of the station table (`Set Network "Rig" "Interface" "en1"`, decisions §11).
//!
//! Owns:          the Network object: a data network the show names, Store / Delete (lines that use it refuse), its tree.
//! Doesn't own:   each station's NIC for it (wiring.rs NETWORK_SETTINGS), the lines on it (lumacore-cmd-protocols).
//! Entry points:  NETWORK, network_write, StoreNetwork, DeleteNetwork.
//! Decisions:     §11, §34 25.17.

use lumacore_objects::commands::registry::{Doc, Undo};
use lumacore_objects::commands::{info, list};
use lumacore_objects::errors::render_code;
use lumacore_objects::objects::protocols::network_id;
use lumacore_objects::objects::{ObjectDef, Verb, filter, out};
use lumacore_objects::props;
use lumacore_objects::text::render_success;
use lumacore_objects::vfs::{DynChild, NodeView, btree_to_children, object_view};
use lumacore_objects::{AppState, DispatchResult};
use lumacore_tree::{LineOps, Root, TreeNode};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

type Edit<'a> = &'a mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>;

// *************************************************************************************************
// Network
// *************************************************************************************************

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn network_store(state: &mut AppState, name: Option<String>) -> DispatchResult {
    let Some(name) = name else {
        return out(render_code("Usage: Store Network \"Rig\"", "E-CMD-006"));
    };
    let id = LineOps::ensure_network(&mut state.root.show_data, &name);
    out(render_success(&format!(
        "Network {id} \"{name}\" — map it on each station: Set Network \"{name}\" \"Interface\" <nic>"
    )))
}

fn network_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let Some(id) = args.first().and_then(|w| network_id(&state.root, w)) else {
        return out(render_code(
            "Usage: Delete Network <n|\"name\">",
            "E-CMD-006",
        ));
    };
    let protocols = &state.root.show_data.protocols;
    let name = protocols.networks[&id].name().unwrap_or("").to_string();
    let guid = protocols.networks[&id].meta.identity.guid;
    let used = protocols.all_lines().any(|(_, _, l)| l.network == guid);
    if used {
        return out(render_code(
            format!(
                "Network \"{name}\" is used by lines — move them first (Set Sacn <n> \"Network\" …)."
            ),
            "E-CMD-010",
        ));
    }
    state.root.show_data.protocols.networks.remove(&id);
    out(render_success(&format!("Deleted Network {id} \"{name}\"")))
}

/// A network lines use can't be deleted (refused before any ask).
fn network_in_use(state: &AppState, word: &str) -> Option<String> {
    let Some(id) = network_id(&state.root, word) else {
        return Some(render_code(
            format!("No Network {word} (List Network)."),
            "E-CMD-007",
        ));
    };
    let protocols = &state.root.show_data.protocols;
    let n = &protocols.networks[&id];
    let guid = n.meta.identity.guid;
    let used = protocols.all_lines().any(|(_, _, l)| l.network == guid);
    used.then(|| {
        render_code(
            format!(
                "Network \"{}\" is used by lines — move them first (Set Sacn <n> \"Network\" …).",
                n.name().unwrap_or("")
            ),
            "E-CMD-010",
        )
    })
}

fn networks_children(root: &Root) -> Vec<DynChild> {
    btree_to_children(&root.show_data.protocols.networks)
}
fn network_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.protocols.networks.get(&id).map(|t| object_view(id, t))
}
pub fn network_write(root: &mut Root, _dp: Option<u32>, id: u32, edit: Edit) -> Result<(), String> {
    let n = root
        .show_data
        .protocols
        .networks
        .get_mut(&id)
        .ok_or("no such network")?;
    props::edit_json(n, edit)
}
fn network_complete(state: &AppState, prefix: &str) -> Vec<String> {
    filter(
        state
            .root
            .show_data
            .protocols
            .networks
            .values()
            .filter_map(|n| n.name().map(|n| format!("\"{n}\""))),
        prefix,
    )
}

/// A network's name and notes are show data; which NIC it uses here
/// (`Set Network "Rig" "Interface" "en1"`) is this station's own setting.
fn network_scope(verb: Verb, args: &[&str]) -> lumacore_objects::objects::Scope {
    if verb == Verb::Set && args.iter().any(|a| a.eq_ignore_ascii_case("interface")) {
        lumacore_objects::objects::Scope::Station
    } else {
        lumacore_objects::objects::Scope::Master
    }
}

/// Deleting networks: dispatch's one ask names them; one in use is refused
/// before it.
static NETWORK_ROWS: lumacore_objects::commands::set_rows::RowDelete =
    lumacore_objects::commands::set_rows::RowDelete {
        keyword: "Network",
        noun: "network",
        numbers: |s| {
            s.root
                .show_data
                .protocols
                .networks
                .keys()
                .copied()
                .collect()
        },
        refused: network_in_use,
        affects: |_, _| None,
        own_ask: false,
    };

pub static NETWORK: ObjectDef = ObjectDef {
    decl: lumacore_objects::objects::ObjectDecl {
        keyword: "Network",
        lum: lumacore_objects::objects::LumShape::ids(1),
        default_verb: lumacore_objects::objects::Verb::Info,
        flags: lumacore_objects::objects::protocols::STORE_NETWORK_FLAGS,
        aliases: &["networks"],
        doc: Doc {
            display: "Network",
            category: lumacore_objects::commands::help::HelpCategory::Universe,
            also_applies_to: &[],
            syntax: "Store Network \"Rig\" | Set Network \"Rig\" \"Interface\" <nic|ip|subnet|none> | List Network",
            description: "A network the show sends or listens on, by name. Each station maps it to one \
                          of its NICs (station-local); a line on an unmapped network never sends. \
                          List Network shows the mapping and the interfaces this station has.",
            examples: &[
                "Store Network \"Rig\"",
                "Set Network \"Rig\" \"Interface\" \"en1\"",
                "Set Network \"Guest\" \"Interface\" \"10.1.0.0/24\"",
                "List Network",
            ],
            related: &["output", "input", "assign"],
            alias_depth: lumacore_objects::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: lumacore_objects::commands::registry::CommandClass::Commit,
        scope: network_scope,
        reach: lumacore_objects::objects::show_reach,
        rights: lumacore_objects::rights::setup,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (
            Verb::Store,
            lumacore_objects::commands::typed::typed::<StoreNetwork>,
        ),
        (
            Verb::Delete,
            lumacore_objects::commands::typed::typed::<DeleteNetwork>,
        ),
        (Verb::List, |s, a| out(list::networks(s, a))),
        // The station's network runtime: interfaces, listeners, connections.
        (Verb::Info, |s, _| out(info::info_network(s))),
    ],
    delete_ask: Some(|s, a| lumacore_objects::commands::set_rows::delete_ask(s, &NETWORK_ROWS, a)),
    complete: Some(network_complete),
    props: Some(&crate::wiring::NETWORK_SETTINGS),
    tree: Some(lumacore_objects::objects::TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: lumacore_objects::tree_path::Node::Networks,
        children: networks_children,
        object: network_view,
    }),
};

/// `Store Network "Rig"`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "Network")]
pub struct StoreNetwork {
    #[lum(target)]
    pub target: Option<lumacore_syntax::ast::Target>,
    #[lum(value)]
    pub name: Option<String>,
}

impl lumacore_objects::commands::typed::Command for StoreNetwork {
    fn run(self, state: &mut AppState) -> DispatchResult {
        match lumacore_objects::commands::set_rows::store_slot(
            state,
            &NETWORK,
            self.target.as_ref(),
            self.name,
        ) {
            Ok((id, name)) => {
                network_store(state, name.or_else(|| id.map(|n| format!("Network {n}"))))
            }
            Err(e) => out(e),
        }
    }
}

/// `Delete Network <target>` (a network lines use is refused).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Delete", object = "Network")]
pub struct DeleteNetwork {
    #[lum(target)]
    pub target: lumacore_syntax::ast::Target,
}

impl lumacore_objects::commands::typed::Command for DeleteNetwork {
    fn run(self, state: &mut AppState) -> DispatchResult {
        lumacore_objects::commands::set_rows::delete_typed(
            state,
            &NETWORK,
            &NETWORK_ROWS,
            &self.target,
            network_delete,
        )
    }
}
