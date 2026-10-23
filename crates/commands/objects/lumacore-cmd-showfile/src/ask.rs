//  █████╗ ███████╗██╗  ██╗
// ██╔══██╗██╔════╝██║ ██╔╝
// ███████║███████╗█████╔╝
// ██╔══██║╚════██║██╔═██╗
// ██║  ██║███████║██║  ██╗
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
// FILE: ask

//! What every show-file word shares (decisions §30 "12d"): `/Save` · `/NoSave` · `/NoAutoClose` ·
//! `/Note`, the show's name, and the question before replacing or leaving the show — "Don't save"
//! still keeps a recovery copy, never loses work silently.
//!
//! Owns:          the show-file words' options, a show's name, "save first?" and the confirm pop-up.
//! Doesn't own:   the dialog (the kernel's dialogs), the recovery copy (lumacore-state recovery).
//! Entry points:  SHOW_FILE_FLAGS, Opts, parse, show_name, generated_name, save_first, confirm, out.
//! Decisions:     §30 "12b", "12d", R1-16.

use lumacore_objects::commands::flags as F;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::save::*;
use lumacore_objects::AppState;
use lumacore_objects::DispatchResult;
use lumacore_objects::dialogs::Ask;
use lumacore_objects::text::render_success;
use lumacore_wire::ButtonKind;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Seconds before a Shutdown / Restart pop-up confirms by itself (10 s).
pub(crate) const POWER_COUNTDOWN: u32 = 10;

#[derive(Default)]
pub(crate) struct Opts {
    pub(crate) no_confirm: bool,
    pub(crate) save: bool,
    pub(crate) no_save: bool,
    pub(crate) no_auto_close: bool,
    /// SaveShow over a newer-format file.
    pub(crate) overwrite: bool,
    /// `/Note "x"`: the save's note (its history row).
    pub(crate) note: Option<String>,
    /// `/Version <stamp>` / `/Recovery "<copy>"`: open that file instead.
    pub(crate) version: Option<String>,
    pub(crate) recovery: Option<String>,
    /// `/Drive "usb1"`: the drive whose library the line reads or writes (decisions §38).
    pub(crate) drive: Option<String>,
    pub(crate) rest: Vec<String>,
}

impl Opts {
    /// ` /Drive "usb1"` for a question's answer line (the drive rides with the line), or "".
    pub(crate) fn drive_word(&self) -> String {
        self.drive.as_deref().map_or(String::new(), |d| format!(" /Drive \"{d}\""))
    }
}

// The show-file commands' options (decisions §31): LoadShow, SaveShow, NewShow, CleanStart,
// DeleteShow, Restart and Shutdown read this one set.
/// `Delete Version|Recovery /Cleanup`: the station's cleanup rules now (decisions §31).
pub(crate) const CLEANUP: F::FlagDef =
    F::FlagDef::new("Cleanup", "Apply the station's cleanup rules now");
pub const NO_AUTO_CLOSE: F::FlagDef =
    F::FlagDef::new("NoAutoClose", "Keep the show-file dialog open");
pub const NOTE: F::FlagDef =
    F::FlagDef::new("Note", "A note saved with this version").value("text");
pub const SHOW_FILE_FLAGS: &[F::FlagDef] = &[
    F::SAVE,
    F::NO_SAVE,
    F::OVERWRITE,
    F::VERSION,
    F::RECOVERY,
    F::DRIVE,
    NO_AUTO_CLOSE,
    NOTE,
];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

pub(crate) fn parse(args: &[&str]) -> Result<Opts, String> {
    let value = |def: &F::FlagDef, what: &str| -> Result<Option<String>, String> {
        match (F::has(args, def), F::value(args, def)) {
            (false, _) => Ok(None),
            (true, Some(v)) => Ok(Some(v)),
            (true, None) => Err(format!("/{} needs {what}", def.name)),
        }
    };
    let o = Opts {
        no_confirm: F::has(args, &F::NO_CONFIRMATION),
        save: F::has(args, &F::SAVE),
        no_save: F::has(args, &F::NO_SAVE),
        no_auto_close: F::has(args, &NO_AUTO_CLOSE),
        overwrite: F::has(args, &F::OVERWRITE),
        note: value(&NOTE, "a note")?,
        version: value(&F::VERSION, "a version (its stamp, List Version)")?,
        recovery: value(&F::RECOVERY, "a recovery copy (List Recovery)")?,
        drive: value(&F::DRIVE, "a drive (List Drive)")?.map(|d| d.trim_matches('"').to_string()),
        rest: F::positional(args, &[&NOTE, &F::VERSION, &F::RECOVERY, &F::DRIVE])
            .into_iter()
            .map(str::to_string)
            .collect(),
    };
    if o.save && o.no_save {
        return Err("/Save and /NoSave exclude each other".into());
    }
    Ok(o)
}

/// A show name from command arguments: unquoted, without `.luma`, no paths.
pub(crate) fn show_name(parts: &[String]) -> Result<Option<String>, String> {
    // Only the quotes go: an apostrophe is part of a name (`Rock'n'`, R1-71).
    let name = parts.join(" ").trim().trim_matches('"').trim().to_string();
    let name = name
        .strip_suffix(".luma")
        .unwrap_or(&name)
        .trim()
        .to_string();
    if name.is_empty() {
        return Ok(None);
    }
    // A name any station can store (R1-71): one file-name rule, `drive`'s.
    lumacore_luma::drive::check_file_name(&name)
        .map_err(|e| format!("Invalid show name '{name}': {e}"))?;
    Ok(Some(name))
}

/// The name of an unnamed show: `NewShow_2026.10.10_14.03.07UTC`.
pub(crate) fn generated_name() -> String {
    format!("NewShow_{}", utc_stamp())
}

/// `2026.10.10_14.03.07UTC` (the date stamp in file names).
pub(crate) fn utc_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}.{month:02}.{day:02}_{:02}.{:02}.{:02}UTC",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Before replacing or leaving the show: `/Save` saves it first; without
/// it, unsaved changes are kept as a before-load recovery copy (decisions
/// §30 "12d": "Don't save" never loses work silently).
pub(crate) fn save_first(
    state: &mut AppState,
    opts: &Opts,
    reason: &str,
) -> Result<Option<String>, String> {
    if opts.save && state.show_dirty {
        return save_show(state, None).map(Some);
    }
    if state.show_dirty {
        // A copy that can't be written stops the line: nothing is replaced
        // (R1-16).
        return match lumacore_objects::recovery::try_recovery_copy(
            state,
            &state.root.show_data,
            reason,
        ) {
            Ok(kept) => Ok(kept.map(|p| format!("Unsaved changes kept as a recovery copy: {p}"))),
            Err(e) => Err(lumacore_objects::errors::with_code(
                format!("Cannot keep your unsaved changes ({e}); nothing was replaced."),
                "E-CMD-012",
            )),
        };
    }
    Ok(None)
}

/// Ask before running `command` unless `/NoConfirmation` (the standard
/// dialog, decisions §30 "12b"). Returns Some(the ask) when it asks; the
/// answer runs `command` again with `/NoConfirmation` (and `/Save` or
/// `/NoSave` when the show has unsaved changes).
pub(crate) fn confirm(
    state: &mut AppState,
    opts: &Opts,
    title: &str,
    message: &str,
    command: String,
    ok: &str,
    countdown: bool,
) -> Option<DispatchResult> {
    if opts.no_confirm {
        return None;
    }
    // The answer runs on the same drive (decisions §38 5).
    let command = format!("{command}{}", opts.drive_word());
    let unsaved = state.show_dirty && !opts.save && !opts.no_save;
    let mut ask = if unsaved {
        Ask::new(title, format!("{message} The show has unsaved changes."))
            .button(
                "Save",
                ButtonKind::Primary,
                format!("{command} /Save /NoConfirmation"),
            )
            .button(
                "Don't save",
                ButtonKind::Danger,
                format!("{command} /NoSave /NoConfirmation"),
            )
            .cancel("Cancel")
            .note("Don't save still keeps a recovery copy (Backup › Recovery).")
            .flags("/NoConfirmation with /Save or /NoSave")
    } else {
        Ask::new(title, message)
            .button(
                ok,
                ButtonKind::Primary,
                format!("{command} /NoConfirmation"),
            )
            .cancel("Cancel")
    };
    if countdown && !unsaved && !opts.no_auto_close {
        ask = ask.countdown(POWER_COUNTDOWN, 0);
    }
    if countdown {
        // Shutdown / Restart: say whose machine it is when the client is
        // elsewhere.
        if state.remote_client {
            let station = state.root.station_settings.station_name.clone();
            ask = ask.note(format!("This is the station {station}, not this computer."));
        }
        ask = ask.topic(format!(
            "power:{}",
            command.split_whitespace().next().unwrap_or_default()
        ));
    }
    Some(DispatchResult::Ask(Box::new(ask)))
}

pub(crate) fn quoted(name: &str) -> String {
    format!("\"{name}\"")
}

pub(crate) fn out(result: Result<String, String>) -> DispatchResult {
    // Show load errors arrive already coded (E-SHOW-…); others get E-CMD-012.
    DispatchResult::Output(result.map_or_else(
        |e| lumacore_objects::errors::reply_error(&e, "E-CMD-012"),
        |m| lumacore_objects::Reply::ok(render_success(&m)),
    ))
}
