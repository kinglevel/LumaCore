//  ██████╗ ██████╗      ██╗███████╗ ██████╗████████╗███████╗
// ██╔═══██╗██╔══██╗     ██║██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║   ██║██████╔╝     ██║█████╗  ██║        ██║   ███████╗
// ██║   ██║██╔══██╗██   ██║██╔══╝  ██║        ██║   ╚════██║
// ╚██████╔╝██████╔╝╚█████╔╝███████╗╚██████╗   ██║   ███████║
//  ╚═════╝ ╚═════╝  ╚════╝ ╚══════╝ ╚═════╝   ╚═╝   ╚══════╝
// FILE: objects

//! What this running station shows about itself: the server, the shells and clients, the session
//! clock, the journal of changes, health, the UI engines.
//!
//! Owns:          the station's objects: Server, Shell, Client, Clock, Journal, Health, UIEngine.
//! Doesn't own:   Station and Interface (the kernel's), their listings (the kernel's list, info).
//! Entry points:  SERVER, SHELL, CLIENT, CLOCK, JOURNAL, HEALTH, UI_ENGINE.
//! Decisions:     §24, §26, §30.

use lumacore_objects::commands::help::HelpCategory;
use lumacore_objects::commands::registry::{AliasDepth, CommandClass, CommandFamily, Doc, Undo};
use lumacore_objects::objects::{ObjectDef, Verb, out};
use lumacore_objects::vfs::{DynChild, NodeView};
use lumacore_objects::{AppState, DispatchResult};
use lumacore_tree::Root;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

lumacore_objects::listed_object!(
props &lumacore_objects::settings_props::ABOUT_SETTINGS,
default Info,
station SERVER,
"Server",
[],
"This LumaCore server",
[
    List => |s, _| out(lumacore_objects::commands::info::info_server(s)),
    Info => |s, _| out(lumacore_objects::commands::info::info_server(s)),
]);

lumacore_objects::listed_object!(
lum lumacore_objects::objects::LumShape::ids(1),
default Info,
station SHELL,
"Shell",
["shells"],
"Logged-in users and their shell clients (terminals, the app, web)",
[
    List => |s, a| out(lumacore_objects::commands::list::shells(s, a)),
    Info => |s, a| out(lumacore_objects::commands::info::info_session(s, a.first().copied())),
]);

lumacore_objects::listed_object!(
lum lumacore_objects::objects::LumShape::ids(1),
default Info,
station CLIENT,
"Client",
["clients"],
"Connected clients",
[
    List => |s, a| out(lumacore_objects::commands::list::clients(s, a)),
    Info => |s, a| out(lumacore_objects::commands::info::info_client(s, a.first().copied())),
]);

lumacore_objects::listed_object!(
default Info,
    station CLOCK,
    "Clock",
    ["timing"],
    "The session clock and its sync",
    [
        Info => |s, _| out(lumacore_objects::commands::info::info_clock(s)),
    ],
    extra: (Doc {
        display: "Clock",
        category: HelpCategory::System,
        also_applies_to: &[],
        syntax: "Clock [/watch[=<ms>]]",
        description: "Display the current wall-clock time (HH:MM:SS.mmm). \
                      Add `/watch` to live-refresh. \
                      Use `Info Clock` for the session clock: its offset to the \
                      LumaSession master, round trip, jitter, drift and PTP.",
        examples: &[
            "clock           -- show current time",
            "clock /watch    -- live clock (200ms)",
        ],
        related: &["watch"],
        alias_depth: AliasDepth::Base,
        hidden: false,
    }, Some(CommandFamily::Inspection), Undo::ReadOnly, CommandClass::Realtime));

lumacore_objects::listed_object!(
default List,
    station JOURNAL,
    "Journal",
    ["events"],
    "The change log: the last commands that changed the show",
    [
        List => |s, a| out(lumacore_objects::commands::list::journal(s, a)),
    ],
    extra: (Doc {
        display: "Journal",
        category: HelpCategory::System,
        also_applies_to: &[],
        syntax: "List Journal [count] | Watch Events",
        description: "Every command that changes the show leaves an entry in \
                      Runtime.Journal: time, user, station, command, the show subtrees \
                      it changed and how it ended (ok, error code, or internal error \
                      report). The last 1000 are kept; nothing is saved with the show.",
        examples: &[
            "list journal       -- the last 20 changes",
            "list journal 100   -- the last 100",
            "watch events       -- changes as they happen",
        ],
        related: &["undo", "health", "error"],
        alias_depth: AliasDepth::Base,
        hidden: false,
    }, None, Undo::ReadOnly, CommandClass::Commit)
);

lumacore_objects::listed_object!(
default Info,
    station HEALTH,
    "Health",
    [],
    "The render engine stage by stage, the output lines' rates, the session clock and every thread (List Health / Info Health Render|Lines|Clock|Threads)",
    [
        List => |s, a| out(lumacore_objects::commands::list::health(s, a)),
        Info => |s, a| out(lumacore_objects::commands::list::health(s, a)),
    ]
);

fn ui_engines_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(lumacore_objects::commands::list::tree(
        state,
        lumacore_objects::tree_path::Node::UIEngines,
        args,
    ))
}
fn ui_engines_children(root: &Root) -> Vec<DynChild> {
    root.ui_engines
        .engines
        .iter()
        .map(|(&id, e)| DynChild {
            display_name: format!("{} \"{}\"  ({}, {})", id, e.name, e.status, e.address),
            id,
            node_type: "UIEngine",
            is_leaf: true,
            child_count: 0,
            name: None,
            guid: None,
        })
        .collect()
}
/// An engine renders screens; it doesn't own them (they live in
/// Root.Runtime.Users). Today the HTTP engine serves them all.
fn ui_engine_view(root: &Root, id: u32) -> Option<NodeView> {
    let e = root.ui_engines.engines.get(&id)?;
    let mut json = serde_json::to_value(e).unwrap_or(serde_json::Value::Null);
    if let Ok(store) = root.runtime.live_screens.read() {
        json["live_screens"] = store.all().map(|s| s.no).collect::<Vec<_>>().into();
    }
    Some(NodeView::Object {
        json,
        node_type: "UIEngine",
        id,
        guid: String::new(),
        name: Some(e.name.clone()),
    })
}

pub static UI_ENGINE: ObjectDef = ObjectDef {
    decl: lumacore_objects::objects::ObjectDecl {
        keyword: "UIEngine",
        lum: lumacore_objects::objects::LumShape::ONE,
        default_verb: lumacore_objects::objects::Verb::List,
        flags: &[],
        aliases: &["uiengines"],
        doc: lumacore_objects::commands::registry::Doc::new(
            "UIEngine",
            "A UI engine rendering the screens (HTTP / web)",
        ),
        family: None,
        call_undo: lumacore_objects::commands::registry::Undo::ReadOnly,
        call_class: lumacore_objects::commands::registry::CommandClass::Commit,
        scope: lumacore_objects::objects::on_master,
        reach: lumacore_objects::objects::show_reach,
        rights: lumacore_objects::rights::anyone,
        planned: &[],
    },
    pool: None,
    verbs: &[(Verb::List, ui_engines_list)],
    delete_ask: None,
    complete: None,
    props: None,
    tree: Some(lumacore_objects::objects::TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: lumacore_objects::tree_path::Node::UIEngines,
        children: ui_engines_children,
        object: ui_engine_view,
    }),
};
