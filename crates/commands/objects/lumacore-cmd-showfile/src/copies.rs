//  ██████╗ ██████╗ ██████╗ ██╗███████╗███████╗
// ██╔════╝██╔═══██╗██╔══██╗██║██╔════╝██╔════╝
// ██║     ██║   ██║██████╔╝██║█████╗  ███████╗
// ██║     ██║   ██║██╔═══╝ ██║██╔══╝  ╚════██║
// ╚██████╗╚██████╔╝██║     ██║███████╗███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚══════╝
// FILE: copies

//! A show's versions and this station's recovery copies (decisions §30 "12d part 5"): every save's
//! replaced file and every recovery copy, pinned or noted; deletes ask with the rows they find.
//!
//! Owns:          the Version and Recovery objects: list, info, delete (asked), pin, note.
//! Doesn't own:   the copies on disk and their cleanup (lumacore-luma backups).
//! Entry points:  VERSION, RECOVERY.
//! Decisions:     §30 "12d part 5".

use lumacore_objects::commands::flags as F;

use crate::ask::*;
use lumacore_objects::AppState;
use lumacore_objects::DispatchResult;
use lumacore_objects::dialogs::Ask;
use lumacore_objects::errors::render_code;
use lumacore_objects::objects::{ObjectDef, Verb, filter, out as text_out, quote};
use lumacore_objects::text::render_success;
use lumacore_wire::ButtonKind;

// *************************************************************************************************
// Objects: Version, Recovery (decisions §30 "12d part 5")
// *************************************************************************************************
use lumacore_luma::backups::{self, Copy, Store};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `"demo" 20261013-140203 + "20261013-131000"`: the show (a quoted name
/// that isn't a stamp; default the loaded show) and the stamps.
pub(crate) fn version_args(state: &AppState, args: &[&str]) -> (String, Vec<String>, Vec<String>) {
    let mut show = None;
    let mut ids = Vec::new();
    let mut rest = Vec::new();
    let mut words = true;
    for a in args {
        let v = a.trim_matches('"');
        if *a == "+" {
            continue;
        }
        if words && backups::stamp_ms(v).is_some() {
            ids.push(v.to_string());
        } else if words && show.is_none() && ids.is_empty() && !a.starts_with('/') {
            show = Some(v.to_string());
        } else {
            words = false;
            rest.push(v.to_string());
        }
    }
    let show = show
        .or_else(|| state.current_show.clone())
        .unwrap_or_default();
    (show, ids, rest)
}

/// Copies named by id (`+` between them).
pub(crate) fn copies(store: Store, show: &str, ids: &[String]) -> Result<Vec<Copy>, String> {
    ids.iter()
        .map(|id| {
            backups::find(store, show, id).ok_or_else(|| match store {
                Store::Version => {
                    format!("\"{show}\" has no version {id} (List Version \"{show}\")")
                }
                _ => format!("No recovery copy \"{id}\" (List Recovery)"),
            })
        })
        .collect()
}

pub(crate) fn no_such(e: String) -> DispatchResult {
    DispatchResult::Output(lumacore_objects::Reply::error(e, "E-CMD-007"))
}

/// The one Delete question for copies (dispatch's `delete_first`, decisions
/// §30 "Merge of bugfixes").
pub(crate) fn copies_ask(store: Store, list: &[Copy], line: &str) -> Ask {
    let what = match store {
        Store::Version => "version file",
        _ => "recovery copy",
    };
    let mut ask = Ask::new(
        "Delete",
        format!(
            "{} {what}{} go to .trash for 30 days{}.",
            list.len(),
            if list.len() == 1 { "" } else { "s" },
            if store == Store::Version {
                "; their history rows stay (marked pruned)"
            } else {
                ""
            }
        ),
    );
    for c in list.iter().take(8) {
        ask = ask.detail(c.id.clone(), None);
    }
    ask.button(
        "Delete",
        ButtonKind::Danger,
        format!("{line} /NoConfirmation"),
    )
    .cancel("Cancel")
    .flags("/NoConfirmation")
}

/// The copies a `Delete Version|Recovery …` names and the line naming them,
/// or the reply when it names none.
pub(crate) fn copies_to_delete(
    state: &AppState,
    store: Store,
    args: &[&str],
) -> Result<(Vec<Copy>, String), String> {
    let args: Vec<&str> = args
        .iter()
        .copied()
        .filter(|a| !a.starts_with('/'))
        .collect();
    match store {
        Store::Version => {
            let (show, ids, _) = version_args(state, &args);
            match copies(Store::Version, &show, &ids) {
                Ok(list) if !list.is_empty() => {
                    let line = format!(
                        "Delete Version {} {}",
                        quoted(&show),
                        list.iter()
                            .map(|c| quoted(&c.id))
                            .collect::<Vec<_>>()
                            .join(" + ")
                    );
                    Ok((list, line))
                }
                Ok(_) => Err(render_code(
                    "Name the version: Delete Version <stamp>",
                    "E-CMD-006",
                )),
                Err(e) => Err(render_code(e, "E-CMD-007")),
            }
        }
        _ => {
            let (ids, _) = recovery_ids(&args);
            match copies(Store::Recovery, "", &ids) {
                Ok(list) if !list.is_empty() => {
                    let line = format!(
                        "Delete Recovery {}",
                        list.iter()
                            .map(|c| quoted(&c.id))
                            .collect::<Vec<_>>()
                            .join(" + ")
                    );
                    Ok((list, line))
                }
                Ok(_) => Err(render_code(
                    "Name the copy (List Recovery), or Delete Recovery /Cleanup",
                    "E-CMD-006",
                )),
                Err(e) => Err(render_code(e, "E-CMD-007")),
            }
        }
    }
}

/// Dispatch's question before `Delete Version|Recovery …`.
pub(crate) fn copies_delete_ask(
    state: &AppState,
    store: Store,
    args: &[&str],
) -> lumacore_objects::objects::DeleteAsk {
    use lumacore_objects::objects::DeleteAsk;
    if store == Store::Recovery && F::has(args, &CLEANUP) {
        return DeleteAsk::Ask(Box::new(cleanup_ask(state)));
    }
    match copies_to_delete(state, store, args) {
        Ok((list, line)) => DeleteAsk::Ask(Box::new(copies_ask(store, &list, &line))),
        Err(reply) => DeleteAsk::Refuse(reply),
    }
}

/// Delete copies: to the trash (dispatch asked first).
pub(crate) fn delete_copies(list: &[Copy]) -> DispatchResult {
    let mut done = Vec::new();
    for c in list {
        match backups::to_trash(&c.path) {
            Ok(_) => done.push(c.id.clone()),
            Err(e) => return out(Err(e)),
        }
    }
    out(Ok(format!(
        "Moved to the trash (30 days): {}",
        done.join(", ")
    )))
}

pub(crate) fn history_text(show: &str, current: Option<&str>) -> String {
    let rows = backups::history_rows(show, current);
    if rows.is_empty() {
        return render_success(&format!("\"{show}\" has no saves with history yet."));
    }
    let mut text = format!("History of \"{show}\" (newest first):\n");
    for r in rows {
        let s = |k: &str| r[k].as_str().unwrap_or("").to_string();
        text.push_str(&format!(
            "  {:<16} {:<18} {:<10} {:<12} {:<11} {:<10} {}{}\n",
            s("id"),
            s("saved"),
            s("by"),
            s("station"),
            s("reason"),
            s("kept"),
            s("changes"),
            if s("note").is_empty() {
                String::new()
            } else {
                format!(" — {}", s("note"))
            }
        ));
    }
    text
}

pub(crate) fn version_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let (show, _, _) = version_args(state, args);
    if show.is_empty() {
        return DispatchResult::Output(lumacore_objects::Reply::error(
            "No show is loaded: List Version \"name\"",
            "E-CMD-009",
        ));
    }
    text_out(history_text(&show, state.current_show.as_deref()))
}

pub(crate) fn copy_info(c: &Copy) -> String {
    let now = backups::now_ms();
    let mut t = format!(
        "{} {} of \"{}\"\n  Saved     {}\n  Reason    {}\n  Station   {}\n  User      {}\n  Size      {}\n  Format    {}\n  File      library/shows/{}\n",
        match c.store {
            Store::Version => "Version",
            Store::Recovery => "Recovery copy",
            Store::Trash => "Deleted file",
        },
        c.id,
        c.show,
        backups::when_text(c.at, now),
        backups::reason_word(&c.reason),
        if c.station.is_empty() {
            "—"
        } else {
            &c.station
        },
        if c.user.is_empty() { "—" } else { &c.user },
        backups::size_text(c.size),
        c.format,
        c.rel
    );
    if let Some(n) = &c.pinned {
        t.push_str(&format!("  Pinned    ★ {n}\n"));
    }
    if !c.note.is_empty() {
        t.push_str(&format!("  Note      {}\n", c.note));
    }
    t
}

pub(crate) fn version_info(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let (show, ids, _) = version_args(state, args);
    match copies(Store::Version, &show, &ids) {
        Ok(list) if !list.is_empty() => {
            text_out(list.iter().map(copy_info).collect::<Vec<_>>().join("\n"))
        }
        Ok(_) => version_list(state, args),
        Err(e) => no_such(e),
    }
}

pub(crate) fn version_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    match copies_to_delete(state, Store::Version, args) {
        Ok((list, _)) => delete_copies(&list),
        Err(reply) => DispatchResult::Output(reply.into()),
    }
}

pub(crate) fn version_complete(state: &AppState, prefix: &str) -> Vec<String> {
    let show = state.current_show.clone().unwrap_or_default();
    filter(
        backups::versions(&show).into_iter().map(|c| quote(&c.id)),
        prefix,
    )
}

/// Rights of Version and Recovery: reading is anyone's, pinning and notes
/// Program, deleting and cleaning up Setup.
pub(crate) fn copy_rights(verb: Verb, _: &[&str]) -> lumacore_objects::commands::registry::Rights {
    use lumacore_objects::commands::registry::Rights;
    match verb {
        Verb::Set => Rights::Program,
        Verb::Delete => Rights::Setup,
        _ => Rights::Guest,
    }
}

// *************************************************************************************************
// A copy's properties (decisions §33 58): Pinned and Note
// *************************************************************************************************
pub(crate) fn copy_of(store: Store, state: &AppState, key: &str) -> Option<Copy> {
    let show = state.current_show.clone().unwrap_or_default();
    backups::find(store, &show, key)
}

pub(crate) fn copy_pinned(store: Store, state: &AppState, key: &str) -> String {
    if copy_of(store, state, key).is_some_and(|c| c.pinned.is_some()) {
        "Yes"
    } else {
        "No"
    }
    .to_string()
}

pub(crate) fn copy_set_pinned(
    store: Store,
    state: &mut AppState,
    key: &str,
    v: &str,
) -> Result<String, String> {
    let c = copy_of(store, state, key)
        .ok_or_else(|| render_code(format!("No copy {key}."), "E-CMD-007"))?;
    let on = v == "Yes";
    backups::set_pin(&c.rel, on.then(|| c.pinned.clone().unwrap_or_default()));
    Ok(format!(
        "{} {}",
        c.id,
        if on {
            "pinned (never cleaned up)"
        } else {
            "unpinned"
        }
    ))
}

pub(crate) fn copy_set_note(
    store: Store,
    state: &mut AppState,
    key: &str,
    v: &str,
) -> Result<String, String> {
    let c = copy_of(store, state, key)
        .ok_or_else(|| render_code(format!("No copy {key}."), "E-CMD-007"))?;
    backups::set_pin(&c.rel, Some(v.to_string()));
    Ok(format!("{} pinned with the note \"{v}\"", c.id))
}

macro_rules! copy_props {
    ($store:expr) => {
        [
            lumacore_objects::props::Property {
                name: "Pinned",
                kind: lumacore_objects::props::PropKind::OnOff,
                words: ("Yes", "No"),
                home: lumacore_objects::props::Home::Station,
                section: "Copy",
                doc: "never cleaned up",
                access: lumacore_objects::props::Access::Keyed {
                    get: |s, k| copy_pinned($store, s, k),
                    set: |s, k, v| copy_set_pinned($store, s, k, v),
                },
                ..lumacore_objects::props::Property::BASE
            },
            lumacore_objects::props::Property {
                name: "Note",
                home: lumacore_objects::props::Home::Station,
                section: "Copy",
                doc: "why it is kept (a note pins it)",
                access: lumacore_objects::props::Access::Keyed {
                    get: |s, k| {
                        copy_of($store, s, k)
                            .and_then(|c| c.pinned)
                            .unwrap_or_default()
                    },
                    set: |s, k, v| copy_set_note($store, s, k, v),
                },
                ..lumacore_objects::props::Property::BASE
            },
        ]
    };
}

pub(crate) static VERSION_PROPS: [lumacore_objects::props::Property; 2] =
    copy_props!(Store::Version);
pub(crate) static RECOVERY_PROPS: [lumacore_objects::props::Property; 2] =
    copy_props!(Store::Recovery);

/// The open show's versions by stamp (another show's are named owner first — §33 3d).
pub(crate) static VERSION_SETTINGS: lumacore_objects::props::Props =
    lumacore_objects::props::Props {
        lists: &[&VERSION_PROPS],
        keyed: Some(lumacore_objects::props::Keyed {
            rows: |s| {
                backups::versions(&s.current_show.clone().unwrap_or_default())
                    .into_iter()
                    .map(|c| c.id)
                    .collect()
            },
            row: |s, w| copy_of(Store::Version, s, w.trim_matches('"')).map(|c| c.id),
        }),
        label: Some(|_, k| format!("Version {k}")),
        ..lumacore_objects::props::Props::BASE
    };

pub(crate) static RECOVERY_SETTINGS: lumacore_objects::props::Props =
    lumacore_objects::props::Props {
        lists: &[&RECOVERY_PROPS],
        keyed: Some(lumacore_objects::props::Keyed {
            rows: |_| backups::recovery().into_iter().map(|c| c.id).collect(),
            row: |s, w| copy_of(Store::Recovery, s, w.trim_matches('"')).map(|c| c.id),
        }),
        label: Some(|_, k| format!("Recovery \"{k}\"")),
        ..lumacore_objects::props::Props::BASE
    };

pub static VERSION: ObjectDef = ObjectDef {
    decl: lumacore_objects::objects::ObjectDecl {
        keyword: "Version",
        lum: lumacore_objects::objects::LumShape::ids(1),
        default_verb: lumacore_objects::objects::Verb::Info,
        flags: &[CLEANUP],
        aliases: &["versions"],
        doc: lumacore_objects::commands::registry::Doc {
            display: "Version",
            category: lumacore_objects::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "List Version [\"show\"] | Info Version [\"show\"] <stamp> | Set Version \"<stamp>\" \"Pinned\" Yes|No | \"Note\" \"text\" | Delete Version [\"show\"] <stamp> [+ <stamp>]",
            description: "A show's history and its versions: every save keeps the file it replaces in \
                          `library/shows/.versions/<show>/` (on this station), named by when it was saved. \
                          `List Version` is the show's history (one row per save: by, station, reason, \
                          changes, kept). A pinned version (a note pins it too) is never cleaned up. \
                          `LoadShow \"x\" /Version <stamp>` opens one as unsaved changes; `Import Show \"x\" \
                          /Version <stamp> …` takes parts of it.",
            examples: &[
                "List Version                             -- the loaded show's history",
                "List Version \"Festival\"                 -- another show's, without loading it",
                "Set Version \"20261013-113000\" \"Note\" \"before rehearsal\"",
                "LoadShow \"demo\" /Version 20261013-113000",
            ],
            related: &["show", "recovery", "saveshow", "loadshow"],
            alias_depth: lumacore_objects::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: lumacore_objects::commands::registry::Undo::ReadOnly,
        call_class: lumacore_objects::commands::registry::CommandClass::Commit,
        scope: lumacore_objects::objects::on_station,
        reach: lumacore_objects::objects::drive_reach,
        rights: copy_rights,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::List, version_list),
        (Verb::Info, version_info),
        (Verb::Delete, version_delete),
    ],
    delete_ask: Some(|s, a| copies_delete_ask(s, Store::Version, a)),
    complete: Some(version_complete),
    props: Some(&VERSION_SETTINGS),
    tree: None,
};

pub(crate) fn recovery_ids(args: &[&str]) -> (Vec<String>, Vec<String>) {
    let mut ids = Vec::new();
    let mut rest = Vec::new();
    for a in args {
        if *a == "+" {
            continue;
        }
        if rest.is_empty() && (a.starts_with('"') || a.contains('_')) {
            ids.push(a.trim_matches('"').to_string());
        } else {
            rest.push(a.trim_matches('"').to_string());
        }
    }
    (ids, rest)
}

pub(crate) fn recovery_list(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    let list = backups::recovery();
    if list.is_empty() {
        return text_out(render_success("No recovery copies on this station."));
    }
    let rules = lumacore_objects::recovery::rules(state);
    let used = backups::usage();
    let now = backups::now_ms();
    let mut t = format!(
        "Recovery copies (library/shows/.autosave on this station; versions + copies use {} of {} MB, kept {} days):\n",
        backups::size_text(used),
        rules.cap_mb,
        rules.keep_days
    );
    for c in list {
        t.push_str(&format!(
            "  {:<52} {:<18} {:<12} {:<18} {:>8}{}\n",
            c.id,
            backups::when_text(c.at, now),
            backups::reason_word(&c.reason),
            format!(
                "{} · {}",
                if c.station.is_empty() {
                    "—"
                } else {
                    &c.station
                },
                if c.user.is_empty() { "—" } else { &c.user }
            ),
            backups::size_text(c.size),
            if c.pinned.is_some() { "  ★" } else { "" }
        ));
    }
    text_out(t)
}

pub(crate) fn recovery_info(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let (ids, _) = recovery_ids(args);
    match copies(Store::Recovery, "", &ids) {
        Ok(list) if !list.is_empty() => {
            text_out(list.iter().map(copy_info).collect::<Vec<_>>().join("\n"))
        }
        Ok(_) => recovery_list(state, args),
        Err(e) => no_such(e),
    }
}

/// The question before `Delete Recovery /Cleanup`: what goes by the rules.
pub(crate) fn cleanup_ask(state: &AppState) -> Ask {
    let rules = lumacore_objects::recovery::rules(state);
    let goes = backups::would_clean(&rules);
    let size: u64 = goes.iter().map(|c| c.size).sum();
    let text = if goes.is_empty() {
        "Nothing is past the cleanup rules; only the trash's files older than 30 days go."
            .to_string()
    } else {
        format!(
            "{} old versions and copies ({}) go for good, by the rules: kept {} days, at most {} MB.",
            goes.len(),
            backups::size_text(size),
            rules.keep_days,
            rules.cap_mb
        )
    };
    let mut ask = Ask::new("Clean Up", text)
        .button(
            "Clean up",
            ButtonKind::Danger,
            "Delete Recovery /Cleanup /NoConfirmation",
        )
        .cancel("Cancel")
        .flags("/NoConfirmation");
    for c in goes.iter().take(8) {
        ask = ask.detail(format!("{} · {}", c.rel, backups::size_text(c.size)), None);
    }
    ask
}

/// `Delete Recovery "<copy>" [+ …]` (to the trash) or `Delete Recovery
/// /Cleanup` (the station's rules now). Dispatch asked first, with what goes.
pub(crate) fn recovery_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    if F::has(args, &CLEANUP) {
        let removed = backups::cleanup(&lumacore_objects::recovery::rules(state));
        return out(Ok(format!("Cleanup removed {} files", removed.len())));
    }
    match copies_to_delete(state, Store::Recovery, args) {
        Ok((list, _)) => delete_copies(&list),
        Err(reply) => DispatchResult::Output(reply.into()),
    }
}

pub(crate) fn recovery_complete(_state: &AppState, prefix: &str) -> Vec<String> {
    filter(
        backups::recovery().into_iter().map(|c| quote(&c.id)),
        prefix,
    )
}

pub static RECOVERY: ObjectDef = ObjectDef {
    decl: lumacore_objects::objects::ObjectDecl {
        keyword: "Recovery",
        lum: lumacore_objects::objects::LumShape::ids(1),
        default_verb: lumacore_objects::objects::Verb::Info,
        flags: &[CLEANUP],
        aliases: &["recoveries"],
        doc: lumacore_objects::commands::registry::Doc {
            display: "Recovery",
            category: lumacore_objects::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "List Recovery | Info Recovery \"<copy>\" | Set Recovery \"<copy>\" \"Pinned\" Yes|No | \"Note\" \"text\" | Delete Recovery \"<copy>\" | Delete Recovery /Cleanup",
            description: "Recovery copies on this station (`library/shows/.autosave/`): the timed copy \
                          every Autosave minutes of a changed show, the copy before a load replaces \
                          unsaved work, and the copies kept on a stop signal or a lost merge. \
                          `LoadShow \"x\" /Recovery \"<copy>\"` opens one as unsaved changes. Cleanup \
                          follows the station's RecoveryKeepDays and RecoveryCap; pinned copies stay; \
                          deleted ones go to the trash for 30 days. After an unclean stop the station \
                          asks once when a copy is newer than the show file.",
            examples: &[
                "List Recovery",
                "LoadShow \"demo\" /Recovery \"demo_timed_FOH_20261013-154700\"",
                "Set Recovery \"demo_timed_FOH_20261013-154700\" \"Pinned\" Yes",
                "Delete Recovery /Cleanup     -- the cleanup rules now (asks)",
            ],
            related: &["version", "show", "loadshow"],
            alias_depth: lumacore_objects::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: lumacore_objects::commands::registry::Undo::ReadOnly,
        call_class: lumacore_objects::commands::registry::CommandClass::Commit,
        scope: lumacore_objects::objects::on_station,
        reach: lumacore_objects::objects::drive_reach,
        rights: copy_rights,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::List, recovery_list),
        (Verb::Info, recovery_info),
        (Verb::Delete, recovery_delete),
    ],
    delete_ask: Some(|s, a| copies_delete_ask(s, Store::Recovery, a)),
    complete: Some(recovery_complete),
    props: Some(&RECOVERY_SETTINGS),
    tree: None,
};
