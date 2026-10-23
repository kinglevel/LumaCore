//  ██████╗ ██████╗ ███╗   ██╗███████╗██╗ ██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗  ██║██╔════╝██║██╔════╝ ██╔════╝
// ██║     ██║   ██║██╔██╗ ██║█████╗  ██║██║  ███╗███████╗
// ██║     ██║   ██║██║╚██╗██║██╔══╝  ██║██║   ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚████║██║     ██║╚██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝     ╚═╝ ╚═════╝ ╚══════╝
// FILE: configs

//! A screen config of the user's profile: which screens a device shows.
//!
//! Owns:          the ScreenConfig object: call, store, delete, list, export / import.
//! Doesn't own:   the screen configs themselves (the kernel's screens).
//! Entry points:  SCREEN_CONFIG.
//! Decisions:     §30 "12c2".

use lumacore_objects::objects::{ObjectDef, Verb, filter, out};
use lumacore_objects::{AppState, DispatchResult};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn screenconfig_call(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(lumacore_objects::commands::screens::cmd_screenconfig(
        state, args,
    ))
}
fn screenconfig_store(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(lumacore_objects::commands::screens::cmd_store_screenconfig(
        state, args,
    ))
}
fn screenconfig_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(lumacore_objects::commands::screens::cmd_delete_screenconfig(state, args))
}
fn screenconfig_list(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    out(lumacore_objects::commands::screens::list_screenconfigs(
        state,
    ))
}
fn screenconfig_complete(state: &AppState, prefix: &str) -> Vec<String> {
    let nos = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map(|p| {
            p.content
                .screen_configs
                .keys()
                .map(u32::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    filter(nos, prefix)
}

pub static SCREEN_CONFIG: ObjectDef = ObjectDef {
    decl: lumacore_objects::objects::ObjectDecl {
        keyword: "ScreenConfig",
        lum: lumacore_objects::objects::LumShape::ids(1),
        default_verb: lumacore_objects::objects::Verb::Select,
        flags: &[lumacore_objects::commands::flags::OVERWRITE.on(&[Verb::Store])],
        aliases: &["screenconfigs"],
        doc: lumacore_objects::commands::registry::Doc {
            display: "ScreenConfig",
            category: lumacore_objects::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "ScreenConfig [n] | Store ScreenConfig n [\"name\"] | Label/Delete ScreenConfig n | Store Screen [n] | Delete Screen n | Export ScreenConfig n | Import ScreenConfig \"file\" [n]",
            description: "Screen configurations: named sets of screens with their windows, saved in the \
                          show on the user's profile (any number of screens — e.g. 1 on a laptop, 5 at \
                          a workstation). ScreenConfig n switches this client; which config a machine \
                          uses and which screens it shows are remembered by the client.",
            examples: &[
                "ScreenConfig                       -- list",
                "Store ScreenConfig 2 \"Workstation\" -- copy this config",
                "ScreenConfig 2                     -- use it here",
                "Store Screen                       -- add a screen",
                "Export ScreenConfig 1              -- library/ui/screenconfigs/<name>.json",
                "Import ScreenConfig \"Desk\" 3      -- as config 3",
            ],
            related: &["window", "view", "user"],
            alias_depth: lumacore_objects::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: lumacore_objects::commands::registry::Undo::ReadOnly,
        call_class: lumacore_objects::commands::registry::CommandClass::Commit,
        scope: lumacore_objects::objects::on_station,
        reach: lumacore_objects::commands::views::files_reach,
        rights: lumacore_objects::rights::screen,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Select, screenconfig_call),
        (Verb::Store, screenconfig_store),
        (Verb::Delete, screenconfig_delete),
        (Verb::List, screenconfig_list),
        (Verb::Export, |s, a| {
            out(lumacore_objects::commands::ui_files::export_screenconfig(
                s, a,
            ))
        }),
        (Verb::Import, |s, a| {
            out(lumacore_objects::commands::ui_files::import_screenconfig(
                s, a,
            ))
        }),
    ],
    delete_ask: None,
    complete: Some(screenconfig_complete),
    props: None,
    tree: None,
};
