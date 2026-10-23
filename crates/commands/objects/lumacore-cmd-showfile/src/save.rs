// ███████╗ █████╗ ██╗   ██╗███████╗
// ██╔════╝██╔══██╗██║   ██║██╔════╝
// ███████╗███████║██║   ██║█████╗
// ╚════██║██╔══██║╚██╗ ██╔╝██╔══╝
// ███████║██║  ██║ ╚████╔╝ ███████╗
// ╚══════╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝
// FILE: save

//! Saving the show (decisions §30 "12d part 5"): every save keeps the file it replaces as a
//! version; over a newer-format file a copy is saved first (E-SHOW-004).
//!
//! Owns:          saving the show: the Admin user, the file a save replaces kept as a version, a newer format.
//! Doesn't own:   the `.luma` file and its versions (lumacore-luma show_file, backups).
//! Entry points:  save_show, save_show_as, write_show, ensure_admin_user.
//! Decisions:     §20, §30 "12d part 5".

use std::path::Path;
use std::time::Instant;

use crate::ask::*;
use lumacore_luma::backups::show_path;
use lumacore_objects::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Every show has an Admin user (created when missing).
pub fn ensure_admin_user(state: &mut AppState) {
    if state.root.show_data.ensure_admin() {
        log::info!("Created the missing Admin user");
    }
}

pub(crate) fn remember(state: &mut AppState, name: &str) {
    state.current_show = Some(name.to_string());
    state.show_dirty = false;
    state.last_save_at = Instant::now();
    state.local_settings.last_show = Some(name.to_string());
    state.local_settings.save();
}

/// Save the show as `name` (None: the current show, or a generated name).
///
/// Feedback: a "Saving showfile <name> : <MB>" progress pill for
/// everyone (a byte counter — the file is written in chunks), then the
/// "Show saved" flash; a failed save is a critical pop-up.
pub fn save_show(state: &mut AppState, name: Option<String>) -> Result<String, String> {
    save_show_as(state, name, false, "").map(Saved::message)
}

/// Why a save is written (its history row's reason): `quick` without a
/// name, `save` under the loaded show's name, `saveas` under another,
/// `restored` after opening a version or a copy.
pub(crate) fn save_reason(state: &AppState, name: Option<&str>) -> &'static str {
    let same = name.is_none_or(|n| state.current_show.as_deref() == Some(n));
    if state.saved.restored.is_some() && same {
        "restored"
    } else if name.is_none() {
        "quick"
    } else if same {
        "save"
    } else {
        "saveas"
    }
}

/// What a save did.
pub(crate) enum Saved {
    /// Written under the name asked for.
    Written(String),
    /// The file there is a newer format: written as a copy instead, which is
    /// now the loaded show. `original` may still be overwritten on request.
    Copied { message: String, original: String },
}

impl Saved {
    fn message(self) -> String {
        match self {
            Self::Written(m) | Self::Copied { message: m, .. } => m,
        }
    }
}

/// The format of the show file at `path`, if it is one.
pub(crate) fn file_format(path: &Path) -> Option<(u64, String)> {
    lumacore_luma::show_file::format_of(path)
}

/// Save, but never over a show file in a newer format than this build
/// writes unless `overwrite` (decisions §23 "App ↔ local server
/// hardening"): an older build would make it unloadable for the newer one.
/// Instead the show goes to `{name}_format{N}_{time}`.
pub(crate) fn save_show_as(
    state: &mut AppState,
    name: Option<String>,
    overwrite: bool,
    note: &str,
) -> Result<Saved, String> {
    let reason = save_reason(state, name.as_deref());
    let name = name
        .or_else(|| state.current_show.clone())
        .unwrap_or_else(generated_name);
    let newer = file_format(&show_path(&name))
        .filter(|(v, _)| *v > u64::from(lumacore_tree::FORMAT_VERSION));
    let Some((version, commit)) = newer.filter(|_| !overwrite) else {
        return write_show(state, name, reason, note).map(Saved::Written);
    };
    let stamp = generated_name();
    let copy = format!(
        "{name}_format{}_{}",
        lumacore_tree::FORMAT_VERSION,
        stamp.trim_start_matches("NewShow_")
    );
    let written = write_show(state, copy.clone(), "format", note)?;
    let message = lumacore_objects::errors::with_code(
        format!(
            "\"{name}\" was saved by a newer build (format {version}, {commit}); this build writes format {} \
             and won't replace it. {written} as a copy instead.",
            lumacore_tree::FORMAT_VERSION
        ),
        "E-SHOW-004",
    );
    state
        .sysmon_pending
        .push(("Show".to_string(), format!("[WARN] {message}")));
    Ok(Saved::Copied {
        message,
        original: name,
    })
}

pub(crate) fn write_show(
    state: &mut AppState,
    name: String,
    reason: &str,
    note: &str,
) -> Result<String, String> {
    use lumacore_sysmon::feedback::{self, Importance, Progress, category};
    // Live screens into the users' screen configs first.
    lumacore_objects::commands::screens::sync(state);
    let mut progress = Progress::start("Saving showfile", category::SHOW, Importance::Important);
    progress.text(&format!("{name} : 0.000 MB"));
    let result = (|| -> Result<(lumacore_luma::backups::Saved, bool), String> {
        let settings = &mut state.root.show_data.show_settings;
        settings.show_name = name.clone();
        if settings.created_with_version.is_empty() {
            settings.created_with_version = env!("CARGO_PKG_VERSION").to_string();
        }
        // The format and the build that wrote it (decisions §20).
        settings.format_version = lumacore_tree::FORMAT_VERSION;
        settings.saved_with_commit = lumacore_objects::build_info().git_commit;
        let path = show_path(&name);
        let dir = lumacore_luma::backups::shows_dir();
        lumacore_luma::drive::create_dir_all(&dir)
            .map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
        // The manifest's facts: who, where, why, the history row with what
        // changed since the last save (decisions §30 "12d part 5").
        let (facts, history, index) =
            lumacore_objects::recovery::facts(state, reason, note, &name, true);
        let (bytes, missing) =
            lumacore_luma::show_file::encode_with(&state.root.show_data, &facts)?;
        // The file it replaces becomes a version; back again if the write fails.
        let kept = lumacore_luma::backups::keep_version(&name, &path)?;
        if let Err(e) = lumacore_luma::show_file::write_bytes(&path, &bytes, |written| {
            progress.update(
                written as i64,
                &format!("{name} : {:.3} MB", written as f64 / 1_000_000.0),
            );
        }) {
            // The named file is untouched (an atomic write): the version
            // copied for this save goes again.
            if let Some(v) = &kept {
                let _ = lumacore_luma::drive::remove_file(v);
            }
            return Err(e);
        }
        for hash in missing {
            lumacore_sysmon::sysmon_note(
                "Show",
                format!(
                    "[WARN] fixture type file {hash}.gdtf isn't in this station's store; the show keeps its derived data only"
                ),
            );
        }
        let saved = lumacore_luma::backups::Saved {
            saved_at: facts["saved_at"].as_u64().unwrap_or(0),
            by: facts["user"].as_str().unwrap_or("").to_string(),
            station: facts["station"].as_str().unwrap_or("").to_string(),
            history,
            index,
            last_copy: None,
            restored: None,
        };
        Ok((saved, kept.is_some()))
    })();
    progress.stop();
    match result {
        Ok((saved, versioned)) => {
            state.saved = saved;
            remember(state, &name);
            lumacore_luma::backups::cleanup(&lumacore_objects::recovery::rules(state));
            feedback::flash("Show saved", Importance::Important);
            Ok(if versioned {
                format!("Show \"{name}\" saved (the previous file is kept as a version)")
            } else {
                format!("Show \"{name}\" saved")
            })
        }
        Err(e) => {
            feedback::message(
                lumacore_sysmon::Level::Error,
                category::SHOW,
                Importance::Critical,
                "Save failed",
                &e,
                None,
            );
            Err(e)
        }
    }
}
