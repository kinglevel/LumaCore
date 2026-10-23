// ███████╗██╗  ██╗ ██████╗ ██╗    ██╗
// ██╔════╝██║  ██║██╔═══██╗██║    ██║
// ███████╗███████║██║   ██║██║ █╗ ██║
// ╚════██║██╔══██║██║   ██║██║███╗██║
// ███████║██║  ██║╚██████╔╝╚███╔███╔╝
// ╚══════╝╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝
// FILE: show

//! The Show object (decisions §33 46): List / Info / Dump / Delete / Import; its settings are
//! `Set Show …`. The timed Autosave writes a recovery copy, never the show file.
//!
//! Owns:          the Show object, List Show, the timed recovery copy, the show's status.
//! Doesn't own:   the show's settings (the kernel's settings_props), the file (lumacore-luma).
//! Entry points:  SHOW, list_shows, shows, autosave_in, autosave_tick, show_info.
//! Decisions:     §30 "12d", §33 46.

use lumacore_objects::commands::flags as F;
use std::time::{Duration, Instant};

use crate::ask::*;
use crate::words::*;
use lumacore_luma::backups::{SHOWS_DIR, list_show_files};
use lumacore_objects::AppState;
use lumacore_objects::DispatchResult;
use lumacore_objects::objects::{ObjectDef, Verb, filter, out as text_out, quote};
use lumacore_objects::text::render_success;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The Show object takes no options: saving and loading are SaveShow / LoadShow, its settings
/// `Set Show …` (decisions §33 46 — an option never says what a line does).
/// `/Drive "usb1"` on Delete and Import (decisions §38 5): the show on that drive.
pub(crate) const SHOW_FLAGS: &[F::FlagDef] = &[F::DRIVE.on(&[Verb::Delete, Verb::Import])];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `Show`'s levels (decisions §30 "12d"): reading is anyone's; importing parts is a load
/// (Setup); `Delete Show` and the rest are Program. Its settings are `Set Show …` (decision 46).
pub(crate) fn show_rights(verb: Verb, _: &[&str]) -> lumacore_objects::commands::registry::Rights {
    use lumacore_objects::commands::registry::Rights;
    match verb {
        Verb::List | Verb::Info | Verb::Dump => Rights::Guest,
        // Taking parts of another show is a load (decisions §30 "12d").
        Verb::Import => Rights::Setup,
        _ => Rights::Program,
    }
}

/// Seconds until the next recovery copy (the show's Autosave interval,
/// counted from the last save or copy), if an interval is set.
pub fn autosave_in(state: &AppState) -> Option<u64> {
    let minutes = state.root.show_data.show_settings.autosave_minutes;
    if minutes == 0 {
        return None;
    }
    let interval = Duration::from_secs(u64::from(minutes) * 60);
    let since = match state.saved.last_copy {
        Some(c) if c > state.last_save_at => c,
        _ => state.last_save_at,
    };
    Some(interval.saturating_sub(since.elapsed()).as_secs())
}

/// Called from the server loop: when the interval has passed and the show
/// changed, write a recovery copy (decisions §30 "12d": the timed autosave
/// never writes the show file — that changes only when someone saves).
pub fn autosave_tick(state: &mut AppState) {
    if !state.show_dirty || autosave_in(state) != Some(0) {
        return;
    }
    // Retry next interval, not every tick, whatever happened.
    state.saved.last_copy = Some(Instant::now());
    match lumacore_objects::recovery::recovery_copy(state, &state.root.show_data, "timed") {
        Some(path) => log::info!("recovery copy: {path}"),
        None => log::warn!("recovery copy failed"),
    }
}

/// The loaded show for status / the Backup menu:
/// `{name, dirty, description, autosave_minutes, autosave_in, startup}`.
pub fn show_info(state: &AppState) -> lumacore_wire::ShowStatus {
    let settings = &state.root.show_data.show_settings;
    lumacore_wire::ShowStatus {
        name: state.current_show.clone(),
        dirty: state.show_dirty,
        description: settings.description.clone(),
        autosave_minutes: settings.autosave_minutes,
        autosave_in: autosave_in(state),
        startup: state.local_settings.startup_show.clone(),
        format_version: settings.format_version,
        saved_with_commit: settings.saved_with_commit.clone(),
        show_id: settings.show_id.to_string(),
        merge_default: settings.merge_default.word().to_string(),
        merge_timeout: settings.merge_timeout,
        references: lumacore_objects::references::summary(state),
    }
}

/// `List Show`: the show files, newest first, the loaded one marked.
pub fn list_shows(state: &AppState) -> String {
    let shows = list_show_files();
    if shows.is_empty() {
        return render_success(&format!("No shows in {SHOWS_DIR}."));
    }
    let mut out = String::from("Shows:\n");
    for s in shows {
        let name = s["name"].as_str().unwrap_or_default();
        let kb = s["size"].as_u64().unwrap_or(0).div_ceil(1024);
        let mark = if state.current_show.as_deref() == Some(name) {
            if state.show_dirty {
                "  ◀ loaded (unsaved)"
            } else {
                "  ◀ loaded"
            }
        } else {
            ""
        };
        out.push_str(&format!("  {name:<40} {kb:>8} kB{mark}\n"));
    }
    out
}

// *************************************************************************************************
// Object: Show (see terminal/objects)
// *************************************************************************************************
pub(crate) fn show_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    cmd_delete_show(state, args)
}
pub(crate) fn show_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    text_out(shows(state, args))
}
pub(crate) fn show_info_h(state: &mut AppState, args: &[&str]) -> DispatchResult {
    match args.first() {
        Some(name) => text_out(lumacore_objects::commands::info::info_show_file(Some(name))),
        None => text_out(lumacore_objects::commands::info::info_loaded_show(state)),
    }
}
/// `Dump Show` — the loaded show's data.
pub(crate) fn show_dump(state: &mut AppState, args: &[&str]) -> DispatchResult {
    text_out(lumacore_objects::commands::show::dump_showdata(state, args))
}
pub(crate) fn show_complete(_state: &AppState, prefix: &str) -> Vec<String> {
    filter(
        list_show_files()
            .iter()
            .filter_map(|s| s["name"].as_str().map(quote)),
        prefix,
    )
}

pub static SHOW: ObjectDef = ObjectDef {
    decl: lumacore_objects::objects::ObjectDecl {
        keyword: "Show",
        lum: lumacore_objects::objects::LumShape::ids(1),
        default_verb: lumacore_objects::objects::Verb::Info,
        flags: SHOW_FLAGS,
        aliases: &["shows", "sw"],
        doc: lumacore_objects::commands::registry::Doc {
            display: "Show",
            category: lumacore_objects::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Show [\"name\"] | Set Show <setting> <value> | Import Show \"name\" [/Version <stamp>] <parts | objects> | Delete Show \"name\"",
            description: "Show info: the loaded show, or a show file by name (decisions §33 46). \
                          Saving and loading are SaveShow / LoadShow; its settings are `Set Show …`. \
                          Shows are `.luma` files in `library/shows/`; every save keeps the file it \
                          replaces as a version, deleting moves a show to the trash (30 days). \
                          `Import Show` takes parts (Data, Patch, Protocols, Users, Stations) or single \
                          objects of another show into this one by GUID, as one Oops step.",
            examples: &[
                "Show                  -- current show info",
                "Show \"demo\"           -- a show file's info",
                "Set Show \"Autosave\" 5 -- a setting",
                "Import Show \"Festival\" Patch Users          -- whole parts",
                "Import Show \"Festival\" Sequence 1 + 3 Group 2 -- single objects (the file's numbers)",
            ],
            related: &["saveshow", "loadshow", "list show", "version", "recovery"],
            alias_depth: lumacore_objects::commands::registry::AliasDepth::Full,
            hidden: false,
        },
        family: Some(lumacore_objects::commands::registry::CommandFamily::Inspection),
        // `Show "x"` reads; `Delete Show` / `Import Show` change: the change path runs (sent to
        // the master, replicated), no Oops step (issue 05).
        call_undo: lumacore_objects::commands::registry::Undo::Untracked,
        call_class: lumacore_objects::commands::registry::CommandClass::Commit,
        scope: lumacore_objects::objects::on_master,
        reach: lumacore_objects::objects::drive_reach,
        rights: show_rights,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Delete, show_delete),
        (Verb::List, show_list),
        (Verb::Info, show_info_h),
        (Verb::Dump, show_dump),
        (Verb::Import, |s, a| out(crate::import::run(s, a))),
    ],
    delete_ask: None,
    complete: Some(show_complete),
    props: Some(&lumacore_objects::settings_props::SHOW_SETTINGS),
    tree: None,
};

/// `List Show` lists show files; `List Show "name"` inspects one.
pub fn shows(state: &AppState, args: &[&str]) -> String {
    match lumacore_objects::commands::list::plain(args).first() {
        Some(name) => lumacore_objects::commands::list::list_show_file(
            name,
            lumacore_objects::commands::list::recursive(args),
        ),
        None => list_shows(state),
    }
}
