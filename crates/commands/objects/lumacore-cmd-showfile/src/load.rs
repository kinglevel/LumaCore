// ██╗      ██████╗  █████╗ ██████╗
// ██║     ██╔═══██╗██╔══██╗██╔══██╗
// ██║     ██║   ██║███████║██║  ██║
// ██║     ██║   ██║██╔══██║██║  ██║
// ███████╗╚██████╔╝██║  ██║██████╔╝
// ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝
// FILE: load

//! Loading a show (decisions §30 "12d"): a version or recovery copy opens as unsaved changes of
//! its show; the show file stays as it is until someone saves.
//!
//! Owns:          loading a show, a version or a recovery copy (as unsaved changes of its show).
//! Doesn't own:   the file's read (lumacore-luma show_file), what follows another show (the kernel's objects).
//! Entry points:  load_show, load_from.
//! Decisions:     §20, §30 "12d part 5".

use crate::save::*;
use lumacore_luma::backups::{SHOWS_DIR, show_path};
use lumacore_objects::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Load show `name`. Returns the message (plus sACN start, as before).
pub fn load_show(state: &mut AppState, name: &str) -> Result<String, String> {
    load_from(state, name, None)
}

/// Load show `name`, or one of its versions / recovery copies (`copy`): a
/// copy opens as unsaved changes of `name` (decisions §30 "12d": the show
/// file stays as it is until someone saves, and then becomes a version).
pub(crate) fn load_from(
    state: &mut AppState,
    name: &str,
    copy: Option<&lumacore_luma::backups::Copy>,
) -> Result<String, String> {
    use lumacore_sysmon::feedback::{self, Importance, Progress, category};
    let path = copy.map_or_else(|| show_path(name), |c| c.path.clone());
    // The demo is built from its sources when it is missing or stale, as a start with it does.
    if copy.is_none()
        && let Some(note) = lumacore_objects::demo::ensure_if_demo(&path)
    {
        lumacore_sysmon::sysmon_note("Show", note);
    }
    if lumacore_luma::drive::metadata(&path).is_err() {
        return Err(format!("No show \"{name}\" in {SHOWS_DIR}"));
    }
    let mut progress = Progress::start("Loading showfile", category::SHOW, Importance::Important);
    progress.text(name);
    let loaded = lumacore_objects::objects::load_show(state, &path.to_string_lossy());
    progress.stop();
    if let Err(e) = loaded {
        feedback::message(
            lumacore_sysmon::Level::Error,
            category::SHOW,
            Importance::Important,
            "Load failed",
            &e,
            None,
        );
        return Err(e);
    }
    feedback::flash("Show loaded", Importance::Important);
    ensure_admin_user(state);
    lumacore_objects::commands::screens::restore_all(state);
    remember(state, name);
    // An older show opens upgraded, as unsaved changes: its file stays until a save, which keeps
    // it as a version (decisions 0289).
    let upgraded = state.upgraded.take();
    if let Some(u) = &upgraded {
        state.show_dirty = true;
        lumacore_sysmon::sysmon_note("Show", format!("{name}: {}", u.line()));
        for note in &u.notes {
            lumacore_sysmon::sysmon_note("Show", format!("  {note}"));
        }
    }
    let Some(copy) = copy else {
        return Ok(match upgraded {
            Some(u) => format!("Show \"{name}\" loaded — {}; unsaved until SaveShow, which keeps the old file as a version", u.line()),
            None => format!("Show \"{name}\" loaded"),
        });
    };
    // The copy is unsaved work on the show: its file's history goes on.
    state.saved = lumacore_luma::backups::manifest(&show_path(name))
        .map(|(m, _, _)| lumacore_luma::backups::Saved::from_manifest(&m))
        .unwrap_or_default();
    state.saved.restored = Some(copy.id.clone());
    state.root.show_data.show_settings.show_name = name.to_string();
    state.show_dirty = true;
    let what = match copy.store {
        lumacore_luma::backups::Store::Version => "version",
        _ => "recovery copy",
    };
    Ok(format!(
        "Opened the {what} {} of \"{name}\" ({}) as unsaved changes — SaveShow makes it the show (the file now becomes a version)",
        copy.id,
        lumacore_luma::backups::when_text(copy.at, lumacore_luma::backups::now_ms())
    ))
}
