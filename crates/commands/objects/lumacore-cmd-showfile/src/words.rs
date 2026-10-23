// ██╗    ██╗ ██████╗ ██████╗ ██████╗ ███████╗
// ██║    ██║██╔═══██╗██╔══██╗██╔══██╗██╔════╝
// ██║ █╗ ██║██║   ██║██████╔╝██║  ██║███████╗
// ██║███╗██║██║   ██║██╔══██╗██║  ██║╚════██║
// ╚███╔███╔╝╚██████╔╝██║  ██║██████╔╝███████║
//  ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚══════╝
// FILE: words

//! The show-file words, the Backup and Shutdown menus' buttons: each echoes on the command line.
//!
//! Owns:          SaveShow, LoadShow, NewShow, CleanStart, DeleteShow, Restart / Shutdown.
//! Doesn't own:   the options and the asks (ask.rs), saving and loading (save.rs, load.rs).
//! Entry points:  cmd_save_show, cmd_load_show, cmd_new_show, cmd_clean_start, clean_start, cmd_delete_show, cmd_power.
//! Decisions:     §30 "12d", "12d part 5".

use crate::ask::*;
use crate::load::*;
use crate::save::*;
use lumacore_luma::backups::{SHOWS_DIR, show_path};
use lumacore_objects::AppState;
use lumacore_objects::DispatchResult;
use lumacore_objects::dialogs::Ask;
use lumacore_wire::ButtonKind;
use lumacore_wire::StartupShow;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `SaveShow ["name"] [/Overwrite]` — a quick save without a name.
/// Over a newer-format file it saves a copy and then asks whether to
/// overwrite that file anyway (`/Overwrite` does it at once).
/// `/Drive "usb1"` (decisions §38): the rest of the line works in that drive's shows folder —
/// its versions, recovery copies and trash beside it; a write makes the library tree first.
pub(crate) fn drive_guard(state: &AppState, drive: Option<&str>, write: bool) -> Result<Option<lumacore_luma::backups::DirGuard>, String> {
    let Some(d) = drive else { return Ok(None) };
    let lib = if write { state.ensure_library(d)? } else { state.drive_library(Some(d))? };
    Ok(Some(lumacore_luma::backups::DirGuard::enter(lib.join("shows"))))
}

pub fn cmd_save_show(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let (opts, name) = match parse(args).and_then(|o| show_name(&o.rest).map(|n| (o, n))) {
        Ok(v) => v,
        Err(e) => return out(Err(e)),
    };
    let note = opts.note.clone().unwrap_or_default();
    let _dir = match drive_guard(state, opts.drive.as_deref(), true) {
        Ok(g) => g,
        Err(e) => return out(Err(e)),
    };
    let (message, original) = match save_show_as(state, name, opts.overwrite, &note) {
        Ok(Saved::Written(m)) => return out(Ok(m)),
        Ok(Saved::Copied { message, original }) => (message, original),
        Err(e) => return out(Err(e)),
    };
    let opts = Opts {
        no_save: true,
        ..opts
    };
    let asked = confirm(
        state,
        &opts,
        "Overwrite Show",
        &format!(
            "{message}\nReally overwrite \"{original}\"? Only format {} builds can load it afterwards; \
             the newer build refuses it.",
            lumacore_tree::FORMAT_VERSION
        ),
        format!("SaveShow {} /Overwrite", quoted(&original)),
        "Overwrite",
        false,
    );
    match asked {
        Some(ask) => ask,
        // /NoConfirmation keeps the copy.
        None => out(Err(message)),
    }
}

/// `LoadShow "name" [/Version <stamp> | /Recovery "<copy>"] [/Save|/NoSave]
/// [/NoConfirmation]`.
pub fn cmd_load_show(state: &mut AppState, args: &[&str]) -> DispatchResult {
    use lumacore_luma::backups::{self, Store};
    let (opts, name) = match parse(args).and_then(|o| show_name(&o.rest).map(|n| (o, n))) {
        Ok((o, Some(n))) => (o, n),
        // A recovery copy names its own show.
        Ok((o, None)) if o.recovery.is_some() => match o
            .recovery
            .as_deref()
            .and_then(|id| backups::find(Store::Recovery, "", id))
        {
            Some(c) => {
                let n = c.show.clone();
                (o, n)
            }
            None => {
                return DispatchResult::Output(lumacore_objects::Reply::error(
                    format!(
                        "No recovery copy \"{}\" (List Recovery)",
                        o.recovery.as_deref().unwrap_or("")
                    ),
                    "E-CMD-007",
                ));
            }
        },
        Ok((_, None)) => {
            return out(Err(
                "Usage: LoadShow \"name\" [/Version <stamp> | /Recovery \"copy\"]".into(),
            ));
        }
        Err(e) => return out(Err(e)),
    };
    let _dir = match drive_guard(state, opts.drive.as_deref(), false) {
        Ok(g) => g,
        Err(e) => return out(Err(e)),
    };
    let copy = match (&opts.version, &opts.recovery) {
        (Some(_), Some(_)) => return out(Err("/Version and /Recovery exclude each other".into())),
        (Some(id), None) => match backups::find(Store::Version, &name, id) {
            Some(c) => Some(c),
            None => {
                return DispatchResult::Output(lumacore_objects::Reply::error(
                    format!("\"{name}\" has no version {id} (List Version \"{name}\")"),
                    "E-CMD-007",
                ));
            }
        },
        (None, Some(id)) => match backups::find(Store::Recovery, &name, id) {
            Some(c) => Some(c),
            None => {
                return DispatchResult::Output(lumacore_objects::Reply::error(
                    format!("No recovery copy \"{id}\" (List Recovery)"),
                    "E-CMD-007",
                ));
            }
        },
        (None, None) => None,
    };
    if copy.is_none() && !show_path(&name).exists() {
        return out(Err(format!("No show \"{name}\" in {SHOWS_DIR}")));
    }
    if let Some(c) = &copy
        && let Err((why, code)) = lumacore_luma::show_file::upgrade::readable(c.format)
    {
        return DispatchResult::Output(lumacore_objects::Reply::error(format!("{} was {why}.", c.id), code));
    }
    let (title, what, line) = match &copy {
        Some(c) if c.store == Store::Version => (
            "Open Version",
            format!(
                "Open the version {} of \"{name}\" as unsaved changes?",
                c.id
            ),
            format!("LoadShow {} /Version {}", quoted(&name), c.id),
        ),
        Some(c) => (
            "Open Recovery Copy",
            format!(
                "Open the recovery copy {} as unsaved changes of \"{name}\"?",
                c.id
            ),
            format!("LoadShow {} /Recovery {}", quoted(&name), quoted(&c.id)),
        ),
        None => (
            "Load Show",
            format!("Load show \"{name}\"?"),
            format!("LoadShow {}", quoted(&name)),
        ),
    };
    if let Some(asked) = confirm(state, &opts, title, &what, line, "Load", false) {
        return asked;
    }
    out(save_first(state, &opts, "before-load").and_then(|saved| {
        let loaded = load_from(state, &name, copy.as_ref())?;
        Ok(saved.map_or(loaded.clone(), |s| format!("{s}\n{loaded}")))
    }))
}

/// `NewShow "name" [/Save|/NoSave] [/NoConfirmation]` — an empty show, saved
/// right away as "name" (or a generated name).
pub fn cmd_new_show(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let (opts, name) = match parse(args).and_then(|o| show_name(&o.rest).map(|n| (o, n))) {
        Ok((o, n)) => (o, n.unwrap_or_else(generated_name)),
        Err(e) => return out(Err(e)),
    };
    if show_path(&name).exists() {
        return out(Err(format!("Show \"{name}\" already exists")));
    }
    if let Some(asked) = confirm(
        state,
        &opts,
        "New Show",
        &format!("Start the new show \"{name}\"?"),
        format!("NewShow {}", quoted(&name)),
        "New Show",
        false,
    ) {
        return asked;
    }
    out(save_first(state, &opts, "before-load").and_then(|saved| {
        lumacore_objects::objects::replace_show(state, lumacore_tree::ShowData::new());
        state.clear_all_undo_stacks();
        ensure_admin_user(state);
        lumacore_objects::commands::screens::restore_all(state);
        state.saved = Default::default();
        let created = write_show(state, name.clone(), "new", "")?;
        lumacore_sysmon::feedback::flash(
            "New show",
            lumacore_sysmon::feedback::Importance::Important,
        );
        let msg = format!("New show \"{name}\" — {created}");
        Ok(saved.map_or(msg.clone(), |s| format!("{s}\n{msg}")))
    }))
}

/// `CleanStart [/Save|/NoSave] [/NoConfirmation]` — a clean start, the way
/// back when the show or the UI is unusable: an empty show (Admin, no
/// password, default profile and screens), startup show None so a broken show
/// doesn't load again, station settings to defaults (the station id and the
/// login secret stay, so discovery and logins keep working). The show file on
/// disk is not touched. Admin only (declared on the word).
pub fn cmd_clean_start(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let opts = match parse(args) {
        Ok(o) => o,
        Err(e) => return out(Err(e)),
    };
    if let Some(asked) = confirm(
        state,
        &opts,
        "CleanStart",
        "Start clean: an empty show, no startup show and default station settings? The show files stay on disk.",
        "CleanStart".to_string(),
        "CleanStart",
        false,
    ) {
        return asked;
    }
    out(save_first(state, &opts, "before-load").map(|saved| {
        clean_start(state);
        state.saved = Default::default();
        let user = state.effective_user_name().to_string();
        lumacore_sysmon::feedback::message(
            lumacore_sysmon::Level::Warn,
            lumacore_sysmon::feedback::category::SHOW,
            lumacore_sysmon::feedback::Importance::Important,
            "CleanStart",
            &format!("by {user}: empty show, default station settings"),
            None,
        );
        state.sysmon_pending.push((
            "Show".to_string(),
            format!("[WARN] CleanStart by '{user}': empty show, default station settings"),
        ));
        let msg = "CleanStart: empty show, startup show None, station settings reset.".to_string();
        saved.map_or(msg.clone(), |s| format!("{s}\n{msg}"))
    }))
}

/// The CleanStart state change (also used at server start with
/// `--clean-start`).
pub fn clean_start(state: &mut AppState) {
    lumacore_objects::objects::replace_show(state, lumacore_tree::ShowData::new());
    state.clear_all_undo_stacks();
    state.seen_names.clear();
    ensure_admin_user(state);
    lumacore_objects::commands::screens::restore_all(state);
    state.current_show = None;
    state.show_dirty = false;
    let keep = &state.local_settings;
    state.local_settings = lumacore_luma::station_config::LocalSettings {
        station_id: keep.station_id.clone(),
        token_secret: keep.token_secret.clone(),
        revoked_tokens: keep.revoked_tokens.clone(),
        startup_show: StartupShow::None,
        // An unreadable settings file stays untouched (R1-14).
        unreadable: keep.unreadable.clone(),
        ..Default::default()
    };
    state.local_settings.save();
}

/// `DeleteShow "name" [/NoConfirmation]` — not the loaded show; the file
/// goes to the trash for 30 days, its versions stay.
pub fn cmd_delete_show(state: &mut AppState, args: &[&str]) -> DispatchResult {
    // Several shows (`"A" + "B"`, the Library's selection): one ask, then each.
    if args.contains(&"+") {
        let opts = match parse(args) {
            Ok(o) => o,
            Err(e) => return out(Err(e)),
        };
        let names: Vec<&str> = opts.rest.iter().map(String::as_str).filter(|a| *a != "+").collect();
        let drive = opts.drive_word();
        if !opts.no_confirm {
            let line = format!("DeleteShow {}{drive}", names.join(" + "));
            let ask = Ask::new(
                "Delete Shows",
                format!(
                    "{} shows go to .trash for 30 days; their versions stay.",
                    names.len()
                ),
            )
            .button(
                "Delete",
                ButtonKind::Danger,
                format!("{line} /NoConfirmation"),
            )
            .cancel("Cancel");
            return DispatchResult::Ask(Box::new(ask));
        }
        let mut lines = Vec::new();
        let drive_args: Vec<&str> = opts.drive.as_deref().map_or(Vec::new(), |d| vec!["/Drive", d]);
        for n in names {
            let mut one = vec![n, "/NoConfirmation"];
            one.extend(drive_args.iter().copied());
            if let DispatchResult::Output(o) = cmd_delete_show(state, &one) {
                lines.push(o);
            }
        }
        return DispatchResult::Output(lumacore_objects::Reply::joined(lines));
    }
    let (opts, name) = match parse(args).and_then(|o| show_name(&o.rest).map(|n| (o, n))) {
        Ok((o, Some(n))) => (o, n),
        Ok((_, None)) => return out(Err("Usage: DeleteShow \"name\"".into())),
        Err(e) => return out(Err(e)),
    };
    let _dir = match drive_guard(state, opts.drive.as_deref(), false) {
        Ok(g) => g,
        Err(e) => return out(Err(e)),
    };
    if state.current_show.as_deref() == Some(name.as_str()) {
        return out(Err(format!("\"{name}\" is the loaded show")));
    }
    let path = show_path(&name);
    if !path.exists() {
        return out(Err(format!("No show \"{name}\" in {SHOWS_DIR}")));
    }
    // Deleting never needs the save question.
    let opts = Opts {
        no_save: true,
        ..opts
    };
    if let Some(asked) = confirm(
        state,
        &opts,
        "Delete Show",
        &format!("Delete show \"{name}\"? The file goes to .trash for 30 days; its versions stay."),
        format!("DeleteShow {}", quoted(&name)),
        "Delete",
        false,
    ) {
        return asked;
    }
    out(lumacore_luma::backups::to_trash(&path)
        .map(|_| format!("Show \"{name}\" moved to the trash (30 days)")))
}

/// `Shutdown` / `Restart` `[/Save|/NoSave] [/NoAutoClose] [/NoConfirmation]`.
pub fn cmd_power(state: &mut AppState, args: &[&str], restart: bool) -> DispatchResult {
    let opts = match parse(args) {
        Ok(o) => o,
        Err(e) => return out(Err(e)),
    };
    let (title, message, command) = if restart {
        ("Restart", "Restart LumaCore?", "Restart")
    } else {
        ("Shutdown", "Shut down LumaCore?", "Shutdown")
    };
    if let Some(asked) = confirm(
        state,
        &opts,
        title,
        message,
        command.to_string(),
        title,
        true,
    ) {
        return asked;
    }
    // A member's save is its own copy: what the save stamps into the show
    // (ShowSettings, profiles) stays in the file and never reaches the
    // session's show through the change path.
    let member_copy = state.link.is_some().then(|| state.root.show_data.clone());
    let saved = save_first(state, &opts, "shutdown");
    if let Some(show) = member_copy {
        state.root.show_data = show;
    }
    if let Err(e) = saved {
        return out(Err(format!("{e} — {} cancelled", title.to_lowercase())));
    }
    // Critical: everyone connected gets the pop-up (their UIs then show the
    // reconnect overlay; a Restart comes back by itself).
    let user = state.effective_user_name().to_string();
    lumacore_sysmon::feedback::message(
        lumacore_sysmon::Level::Alert,
        lumacore_sysmon::feedback::category::POWER,
        lumacore_sysmon::feedback::Importance::Critical,
        if restart {
            "Server restarting"
        } else {
            "Server shutting down"
        },
        &format!("by {user}"),
        None,
    );
    state.restart_requested = restart;
    DispatchResult::Exit
}
