//! Export / import of the UI floor plan as JSON (decisions §30 "12c":
//! "anything should be able to be exported and imported as json"):
//!
//! ```text
//! Export View 3 ["file"]            → library/ui/views/<name>.json
//! Import View "file" [n]            ← into your profile's views (n or the next free)
//! Export ScreenConfig 1 ["file"]    → library/ui/screenconfigs/<name>.json
//! Import ScreenConfig "file" [n]
//! Export WindowPreset 2 ["file"]    → library/ui/windowpresets/<name>.json
//! Import WindowPreset "file" [n]    ← into the preset's window type
//! ```
//!
//! Each file is `{kind, format, name, guid, …}`: only the floor plan (which
//! window, where, its type, label, colour, scroll and settings) — never how a
//! window is built (that ships in the registry). The format is the show's
//! (forward only: another format is refused). An import is the same object
//! when its GUIDs are free in this show, else a copy with new GUIDs (strict
//! identity, decisions §21 "6d"). Files go only through `lumacore_luma::drive`.

use serde_json::{Value, json};

use lumacore_tree::{ScreenConfig, StoredView, WindowPreset};

use crate::errors::{render_code, reply};
use crate::text::render_success;
use crate::AppState;

/// `library/ui/<kind>/` (tests keep theirs out of the repo).
pub fn dir(kind: &str) -> String {
    let base = if cfg!(test) {
        std::env::temp_dir().join("lumacore-test-ui").to_string_lossy().into_owned()
    } else {
        "library/ui".to_string()
    };
    format!("{base}/{kind}")
}

/// A file name from an object's name (no path, no odd characters).
fn file_name(name: &str) -> String {
    let n: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ') { c } else { '_' })
        .collect();
    let n = n.trim().to_string();
    if n.is_empty() { "unnamed".into() } else { n }
}

fn words(args: &[&str]) -> Vec<String> {
    args.iter().filter(|a| !a.starts_with('/')).map(|a| a.trim_matches('"').to_string()).collect()
}

fn own_profile(state: &AppState) -> Result<u32, String> {
    state
        .root
        .show_data
        .profile_id_for_user(state.effective_user_name())
        .ok_or_else(|| render_code("You have no profile.", "E-CMD-011"))
}

/// Write `value` to `library/ui/<kind>/<name>.json`.
fn write(kind: &str, name: &str, value: &Value) -> Result<String, String> {
    let dir = dir(kind);
    let path = format!("{dir}/{}.json", file_name(name));
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    lumacore_luma::drive::create_dir_all(&dir)
        .and_then(|_| lumacore_luma::drive::write_atomic(&path, &bytes))
        .map_err(|e| render_code(format!("Cannot write {path}: {e}"), "E-CMD-012"))?;
    Ok(path)
}

/// Read `library/ui/<dir>/<file>.json` and check its kind and format.
fn read(dir_kind: &str, kind: &str, file: &str) -> Result<(Value, String), String> {
    if file.contains("..") || file.contains('/') || file.contains('\\') {
        return Err(render_code(format!("'{file}' is not a file in library/ui/{dir_kind}/."), "E-CMD-012"));
    }
    let path = format!("{}/{}.json", dir(dir_kind), file.trim_end_matches(".json"));
    let bytes = lumacore_luma::drive::read(&path).map_err(|e| render_code(format!("Cannot read {path}: {e}"), "E-CMD-012"))?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| render_code(format!("{path}: {e}"), "E-CMD-012"))?;
    if v["kind"].as_str() != Some(kind) {
        return Err(render_code(format!("{path} is a {}, not a {kind}.", v["kind"].as_str().unwrap_or("?")), "E-CMD-012"));
    }
    // A view or window preset kept its shape since the oldest readable format (decisions 0289);
    // a step that changes one also upgrades these files.
    let format = v["format"].as_u64().unwrap_or(0);
    if let Err((why, code)) = lumacore_luma::show_file::upgrade::readable(format) {
        return Err(render_code(format!("{path} was {why}."), code));
    }
    Ok((v, path))
}

/// `{kind, format, …object}`.
fn wrap<T: serde::Serialize>(kind: &str, obj: &T) -> Value {
    let mut v = json!({ "kind": kind, "format": lumacore_tree::FORMAT_VERSION });
    if let (Value::Object(head), Ok(Value::Object(body))) = (&mut v, serde_json::to_value(obj)) {
        head.extend(body);
    }
    v
}

/// The object goes in as-is when its GUIDs are free in the show, else as a
/// copy with new ones (`renew` gives them). Returns whether it was copied.
fn place(state: &mut AppState, insert: impl Fn(&mut lumacore_tree::ShowData, bool)) -> Result<bool, String> {
    let mut trial = state.root.show_data.clone();
    insert(&mut trial, false);
    let copy = !trial.identity().duplicates.is_empty();
    if copy {
        let mut trial = state.root.show_data.clone();
        insert(&mut trial, true);
        if !trial.identity().duplicates.is_empty() {
            return Err(render_code("The file shares GUIDs with this show even as a copy.", "E-SHOW-003"));
        }
    }
    insert(&mut state.root.show_data, copy);
    state.show_dirty = true;
    Ok(copy)
}

fn number(args: &[&str], what: &str) -> Result<u32, String> {
    words(args)
        .first()
        .and_then(|w| w.parse::<u32>().ok())
        .filter(|n| *n > 0)
        .ok_or_else(|| render_code(format!("Which {what}? Give its number."), "E-CMD-006"))
}

/// `Import … "file" [n]`: the file and the number to put it at.
fn file_and_no(args: &[&str], usage: &str) -> Result<(String, Option<u32>), String> {
    let w = words(args);
    let file = w.first().cloned().ok_or_else(|| render_code(usage, "E-CMD-006"))?;
    Ok((file, w.get(1).and_then(|n| n.parse().ok()).filter(|n: &u32| *n > 0)))
}

// ── View ─────────────────────────────────────────────────────────────────────

pub fn export_view(state: &mut AppState, args: &[&str]) -> String {
    let no = match number(args, "view") {
        Ok(n) => n,
        Err(e) => return e,
    };
    let Some(view) = state.current_user_profile_mut().and_then(|p| p.content.views.get(&no)).cloned() else {
        return render_code(format!("View {no} is empty."), "E-CMD-007");
    };
    let name = words(args).get(1).cloned().unwrap_or_else(|| view.meta.name().unwrap_or_default().to_string());
    match write("views", &name, &wrap("View", &view)) {
        Ok(path) => render_success(&format!("View {no} \"{}\" exported to {path}", view.meta.name().unwrap_or_default())),
        Err(e) => e,
    }
}

pub fn import_view(state: &mut AppState, args: &[&str]) -> String {
    let (file, at) = match file_and_no(args, "Usage: Import View \"file\" [n] (a file in library/ui/views/)") {
        Ok(x) => x,
        Err(e) => return e,
    };
    let (v, path) = match read("views", "View", &file) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let view: StoredView = match serde_json::from_value(v) {
        Ok(v) => v,
        Err(e) => return render_code(format!("{path}: {e}"), "E-CMD-012"),
    };
    let profile = match own_profile(state) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let views = &state.root.show_data.user_profiles[&profile].content.views;
    let no = at.unwrap_or_else(|| views.keys().next_back().map_or(1, |n| n + 1));
    let name = view.meta.name().unwrap_or_default().to_string();
    let placed = place(state, |show, copy| {
        let mut view = view.clone();
        if copy {
            view.meta.identity.guid = uuid::Uuid::new_v4();
            view.screens.values_mut().for_each(|s| s.meta.identity.guid = uuid::Uuid::new_v4());
        }
        if let Some(p) = show.profile_mut(profile) {
            p.content.views.insert(no, view);
        }
    });
    // The view bar hears it from the change path (`views::profiles_changed`).
    match placed {
        Ok(copy) => render_success(&format!("View {no} \"{name}\" imported from {path}{}", if copy { " (a copy: new GUIDs)" } else { "" })),
        Err(e) => e,
    }
}

// ── ScreenConfig ─────────────────────────────────────────────────────────────

pub fn export_screenconfig(state: &mut AppState, args: &[&str]) -> String {
    let no = match number(args, "screen config") {
        Ok(n) => n,
        Err(e) => return e,
    };
    let Some(config) = state.current_user_profile_mut().and_then(|p| p.content.screen_configs.get(&no)).cloned() else {
        return render_code(format!("No screen config {no}."), "E-CMD-007");
    };
    let name = words(args).get(1).cloned().unwrap_or_else(|| config.meta.name().unwrap_or_default().to_string());
    match write("screenconfigs", &name, &wrap("ScreenConfig", &config)) {
        Ok(path) => render_success(&format!("Screen config {no} \"{}\" exported to {path}", config.meta.name().unwrap_or_default())),
        Err(e) => e,
    }
}

pub fn import_screenconfig(state: &mut AppState, args: &[&str]) -> String {
    let (file, at) = match file_and_no(args, "Usage: Import ScreenConfig \"file\" [n] (a file in library/ui/screenconfigs/)") {
        Ok(x) => x,
        Err(e) => return e,
    };
    let (v, path) = match read("screenconfigs", "ScreenConfig", &file) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let config: ScreenConfig = match serde_json::from_value(v) {
        Ok(c) => c,
        Err(e) => return render_code(format!("{path}: {e}"), "E-CMD-012"),
    };
    let profile = match own_profile(state) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let configs = &state.root.show_data.user_profiles[&profile].content.screen_configs;
    let no = at.unwrap_or_else(|| configs.keys().next_back().map_or(1, |n| n + 1));
    if configs.contains_key(&no) {
        return render_code(format!("Screen config {no} exists — import at a free number (Import ScreenConfig \"{file}\" <n>)."), "E-CMD-010");
    }
    let name = config.meta.name().unwrap_or_default().to_string();
    let placed = place(state, |show, copy| {
        let mut config = config.clone();
        if copy {
            config.meta.identity.guid = uuid::Uuid::new_v4();
            config.screens.values_mut().for_each(|s| s.meta.identity.guid = uuid::Uuid::new_v4());
        }
        if let Some(p) = show.profile_mut(profile) {
            p.content.screen_configs.insert(no, config);
        }
    });
    match placed {
        Ok(copy) => render_success(&format!("Screen config {no} \"{name}\" imported from {path}{}", if copy { " (a copy: new GUIDs)" } else { "" })),
        Err(e) => e,
    }
}

// ── WindowPreset ─────────────────────────────────────────────────────────────

pub fn export_preset(state: &mut AppState, profile: Option<u32>, args: &[&str]) -> String {
    // The preset is named like Label / Delete name it (Self's type or /Type).
    let (_, ty, no, preset, rest) = match super::window_presets::named(state, profile, args) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let name = rest.first().cloned().unwrap_or_else(|| preset.meta.name().unwrap_or_default().to_string());
    match write("windowpresets", &name, &wrap("WindowPreset", &preset)) {
        Ok(path) => render_success(&format!("{} preset {no} \"{}\" exported to {path}", ty.label, preset.meta.name().unwrap_or_default())),
        Err(e) => e,
    }
}

pub fn import_preset(state: &mut AppState, profile: Option<u32>, args: &[&str]) -> String {
    let (file, at) = match file_and_no(args, "Usage: Import WindowPreset \"file\" [n] (a file in library/ui/windowpresets/)") {
        Ok(x) => x,
        Err(e) => return e,
    };
    let (v, path) = match read("windowpresets", "WindowPreset", &file) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let preset: WindowPreset = match serde_json::from_value(v) {
        Ok(p) => p,
        Err(e) => return render_code(format!("{path}: {e}"), "E-CMD-012"),
    };
    let Some(ty) = lumacore_registry::get().window_types().values().find(|t| t.name == preset.kind && !t.hidden).cloned() else {
        return render_code(format!("{path}: unknown window type '{}'", preset.kind), "E-CMD-012");
    };
    let profile = match profile.map(Ok).unwrap_or_else(|| own_profile(state)) {
        Ok(p) => p,
        Err(e) => return e,
    };
    if let Err(e) = crate::commands::media::may_change_profile(state, profile) {
        return reply(e, "E-RIGHTS-001");
    }
    let pool = state.root.show_data.user_profiles[&profile].content.window_presets.get(&ty.name).cloned().unwrap_or_default();
    let no = at.unwrap_or_else(|| pool.keys().next_back().map_or(1, |n| n + 1));
    if pool.contains_key(&no) {
        return render_code(format!("{} preset {no} exists — import at a free number.", ty.label), "E-CMD-010");
    }
    let name = preset.meta.name().unwrap_or_default().to_string();
    // Settings this type doesn't have are dropped (unknown bar ids too).
    let settings = ty.resolve_settings(&preset.settings);
    let placed = place(state, |show, copy| {
        let mut preset = preset.clone();
        preset.settings = settings.clone();
        if copy {
            preset.meta.identity.guid = uuid::Uuid::new_v4();
        }
        if let Some(p) = show.profile_mut(profile) {
            p.content.window_presets.entry(preset.kind.clone()).or_default().insert(no, preset);
        }
    });
    match placed {
        Ok(copy) => render_success(&format!(
            "{} preset {no} \"{name}\" imported from {path}{}",
            ty.label,
            if copy { " (a copy: new GUIDs)" } else { "" }
        )),
        Err(e) => e,
    }
}
