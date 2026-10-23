// ██████╗ ███████╗ ██████╗ ██████╗ ██╗   ██╗███████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝██╔═══██╗██║   ██║██╔════╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║     ██║   ██║██║   ██║█████╗  ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║     ██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╗╚██████╔╝ ╚████╔╝ ███████╗██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝   ╚═╝
// FILE: recovery

//! Backups that read the station (decisions §30 "12d part 5 built", §34 25.10): who and why a
//! save or recovery copy is written (`facts`), the station's cleanup rules, the startup recovery
//! check and its ask. Everything on disk is `lumacore_luma::backups`.
//!
//! Owns:          the backups that read the station: a save's facts, recovery copies, the cleanup rules, the startup recovery ask.
//! Doesn't own:   the files on disk (lumacore-luma backups.rs).
//! Entry points:  facts, recovery_copy, try_recovery_copy, rules, startup, recovery_ask, station_facts.
//! Decisions:     §30 "12d part 5 built", §34 25.10.

use lumacore_luma::backups::*;
use std::path::Path;

use lumacore_tree::ShowData;
use serde_json::{Value, json};

use crate::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Who and where: the user running the line, this station.
fn who(state: &AppState) -> (String, String) {
    let user = state.effective_user_name();
    let user = if user == crate::INTERNAL_ROOT_USER { "—" } else { user };
    (user.to_string(), state.root.station_settings.station_name.clone())
}

/// The manifest facts of a save (`row` = a history row is added, the copy's
/// own facts otherwise): `(extra manifest fields, the new history, index)`.
pub fn facts(state: &AppState, reason: &str, note: &str, name: &str, row: bool) -> (Value, Vec<Value>, Value) {
    let index = index(&state.root.show_data);
    let at = now_ms();
    let (user, station) = who(state);
    let mut history = state.saved.history.clone();
    if row {
        let (changes, detail) = diff(&state.saved.index, &index);
        history.insert(
            0,
            json!({
                "id": format!("{}.{:03}", stamp(at), at % 1000), "saved_at": at, "by": user, "station": station, "reason": reason,
                "note": note, "name": name, "format": lumacore_tree::FORMAT_VERSION,
                "changes": changes, "detail": detail,
            }),
        );
        history.truncate(HISTORY_ROWS);
    }
    let extra = json!({
        "saved_at": at, "reason": reason, "station": station, "user": user, "note": note,
        "history": history,
    });
    (extra, history, index)
}

/// A recovery copy of the loaded show (`reason`: timed, before-load,
/// signal, use-master, …). Untitled shows too. Returns its path.
pub fn recovery_copy(state: &AppState, show: &ShowData, reason: &str) -> Option<String> {
    try_recovery_copy(state, show, reason).ok().flatten()
}

/// [`recovery_copy`] that says why a copy couldn't be written (R1-16: the
/// caller must then not replace the show). `Ok(None)` = unit tests without a
/// test library (nothing written, but a closed door is still met).
pub fn try_recovery_copy(state: &AppState, show: &ShowData, reason: &str) -> Result<Option<String>, String> {
    let name = state.current_show.clone().unwrap_or_else(|| "untitled".into());
    let dir = shows_dir().join(AUTOSAVE);
    let station = sane(&state.root.station_settings.station_name);
    let base = format!("{}_{reason}_{station}_{}", sane(&name), stamp(now_ms()));
    let mut path = dir.join(format!("{base}.luma"));
    let failed = |path: &Path, e: std::io::Error| format!("{}: {e}", path.display());
    if !writes_allowed() {
        // Unit tests leave no files behind, but still meet a closed door.
        lumacore_luma::drive::may_write(&path).map_err(|e| failed(&path, e))?;
        return Ok(None);
    }
    let (mut extra, _, _) = facts(state, reason, "", &name, false);
    extra["show_name"] = json!(name);
    let (bytes, _) = lumacore_luma::show_file::encode_with(show, &extra).map_err(|e| format!("{}: {e}", path.display()))?;
    lumacore_luma::drive::create_dir_all(&dir).map_err(|e| failed(&dir, e))?;
    let mut n = 2;
    while lumacore_luma::drive::metadata(&path).is_ok() {
        path = dir.join(format!("{base}-{n}.luma"));
        n += 1;
    }
    lumacore_luma::drive::write_atomic(&path, bytes).map_err(|e| failed(&path, e))?;
    let shown = rel(&path);
    lumacore_sysmon::sysmon_note("Show", format!("Recovery copy ({}): {shown}", reason_word(reason)));
    cleanup(&rules(state));
    Ok(Some(shown))
}

pub fn rules(state: &AppState) -> Rules {
    let d = Rules::default();
    Rules {
        keep_days: state.local_settings.recovery_keep_days.unwrap_or(d.keep_days),
        cap_mb: state.local_settings.recovery_cap_mb.unwrap_or(d.cap_mb),
    }
}

/// At start (after the startup show): when the station didn't stop cleanly
/// last time and a recovery copy of the show is newer than its file, ask
/// once — Open copy / Open saved show / Show Recovery… (decisions §30 "12d
/// hardening": never loads by itself, never asks twice: the marker is
/// written again whatever is chosen).
pub fn startup(state: &mut AppState) {
    // A start that died while loading its show (`.starting`, E-SHOW-006) is
    // an unclean stop too: the show was skipped first, now the copy is
    // offered (decisions §30 "Merge of bugfixes").
    let crashed = lumacore_luma::drive::metadata(marker()).is_ok() || state.skipped_startup_show.is_some();
    let _ = lumacore_luma::drive::create_dir_all(lumacore_luma::station_config::dir());
    let _ = lumacore_luma::drive::write(marker(), format!("{}\n", std::process::id()));
    if !crashed {
        return;
    }
    if let Some(ask) = recovery_ask(state) {
        crate::dialogs::open_for(state, ask, crate::dialogs::Audience::Station, "Admin");
    }
}

/// The ask after an unclean stop, when a newer recovery copy exists.
pub fn recovery_ask(state: &AppState) -> Option<crate::dialogs::Ask> {
    use lumacore_wire::ButtonKind;
    // A skipped startup show (nothing loaded): its file's own save time.
    let skipped = state.current_show.is_none().then(|| state.skipped_startup_show.clone()).flatten();
    let (show, saved_at) = match &skipped {
        Some(name) => {
            let path = show_path(name);
            let at = manifest(&path).and_then(|(m, _, mtime)| m["saved_at"].as_u64().or(Some(mtime))).unwrap_or(0);
            (name.clone(), at)
        }
        None => (state.current_show.clone().unwrap_or_else(|| "untitled".into()), state.saved.saved_at),
    };
    let copy = recovery()
        .into_iter()
        .find(|c| c.show == show && c.at > saved_at && c.format == u64::from(lumacore_tree::FORMAT_VERSION))?;
    let now = now_ms();
    let saved = if saved_at == 0 { "never saved".to_string() } else { format!("saved {}", when_text(saved_at, now)) };
    let why = if skipped.is_some() {
        format!("The last start died while loading \"{show}\", so it wasn't loaded (E-SHOW-006).")
    } else {
        "LumaCore stopped unexpectedly.".to_string()
    };
    Some(
        crate::dialogs::Ask::new(
            "Recovery",
            format!(
                "{why} A recovery copy of \"{show}\" from {} is newer than the show file ({saved}).",
                when_text(copy.at, now)
            ),
        )
        .row("Copy", format!("{} · {}", reason_word(&copy.reason), copy.id))
        .row("Written by", format!("{} · {}", if copy.station.is_empty() { "—" } else { &copy.station }, if copy.user.is_empty() { "—" } else { &copy.user }))
        .button(
            "Open copy",
            ButtonKind::Primary,
            format!("LoadShow \"{show}\" /Recovery \"{}\" /NoConfirmation", copy.id),
        )
        .button("Show Recovery…", ButtonKind::Plain, "Menu \"Backup\" /Tab Recovery")
        .cancel(if skipped.is_some() { "Not now" } else { "Open saved show" })
        .note("The copy opens as unsaved changes of the show; the show file stays as it is until you save."),
    )
}

/// What the Backup menu needs of the state thread (published with the
/// station view): the cleanup rules and the loaded file's facts.
pub fn station_facts(state: &AppState) -> Value {
    let r = rules(state);
    json!({
        "keep_days": r.keep_days, "cap_mb": r.cap_mb,
        "saved_at": state.saved.saved_at, "by": state.saved.by, "station": state.saved.station,
        "restored": state.saved.restored,
        "counts": state.saved.index.is_object().then(|| counts(&state.saved.index)),
    })
}
