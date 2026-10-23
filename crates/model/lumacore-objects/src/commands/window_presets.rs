//! Window presets on the profile (decisions §30 "12c": `WindowPreset`, not
//! "Preset" — the lighting word; on the **profile**, like a console's
//! `StorePreferences.Views`). A preset holds everything of a window —
//! settings, the window bar, label, colour, scroll — numbered per window
//! type. Loading copies it into a window (no live link; the window remembers
//! which preset it came from so the Load list can preselect it).
//!
//! ```text
//! Store WindowPreset ["name"]          Self's window as a new preset of its type
//! Store WindowPreset 2 ["name"]        into preset 2 (asks before overwriting; /Overwrite)
//! Store WindowPreset                   asks: name, new or which to overwrite
//! WindowPreset 2                       load preset 2 into Self
//! WindowPreset                         asks: which one (the pick list)
//! Label WindowPreset 2 "Patch check"   ·  Delete WindowPreset 3
//! List WindowPreset [/Type DMX]        ·  Info WindowPreset 2
//! UserProfile "Bob" WindowPreset 1     another profile's (owner first; bare = your own)
//! Export WindowPreset 2 · Import WindowPreset "Big DMX"   library/ui/windowpresets/ (JSON)
//! ```
//!
//! The window is `Self` (the line's owner), or `/Window [screen.]n`; the
//! window type is the window's, or `/Type <type>`.

use crate::commands::flags as F;
use std::collections::BTreeMap;

use lumacore_tree::runtime::Window;
use lumacore_tree::{WindowPreset, WindowType};
use serde_json::{Map, Value};

use crate::commands::registry::{Reach, Undo};
use crate::errors::{render_code, reply};
use crate::objects::{Doc, ObjectDef, Verb, filter, out, quote};
use crate::text::render_success;
use crate::{AppState, DispatchResult};

/// What a line names: the profile, the window (if any) and the window type.
#[derive(Debug, Clone)]
pub struct Target {
    pub profile: u32,
    pub window: Option<(u32, u32)>,
    pub ty: Option<WindowType>,
}

/// Positional words and options of a WindowPreset line.
#[derive(Debug, Default)]
struct Parsed {
    words: Vec<String>,
    window: Option<String>,
    ty: Option<String>,
    overwrite: bool,
}

/// `WindowPreset … /Window <n>` (decisions §31: declared on the WindowPreset object).
const WINDOW: F::FlagDef = F::FlagDef::new("Window", "The window it comes from or goes to").alias(&["w"]).value("window");

fn parse(args: &[&str]) -> Result<Parsed, String> {
    let value = |def: &F::FlagDef| -> Result<Option<String>, String> {
        match (F::has(args, def), F::value(args, def)) {
            (false, _) => Ok(None),
            (true, Some(v)) => Ok(Some(v)),
            (true, None) => Err(format!("/{} needs a value", def.name)),
        }
    };
    Ok(Parsed {
        window: value(&WINDOW)?,
        ty: value(&F::TYPE)?,
        overwrite: F::has(args, &F::OVERWRITE),
        words: F::positional(args, &[&WINDOW, &F::TYPE]).iter().map(|w| w.trim_matches('"').to_string()).collect(),
    })
}

/// The profile the acting user works in.
fn own_profile(state: &AppState) -> Option<u32> {
    state.root.show_data.profile_id_for_user(state.effective_user_name())
}

/// A profile by number or name.
pub fn profile_no(state: &AppState, word: &str) -> Option<u32> {
    let word = word.trim().trim_matches('"');
    let profiles = &state.root.show_data.user_profiles;
    if let Ok(no) = word.parse::<u32>() {
        return profiles.contains_key(&no).then_some(no);
    }
    profiles
        .iter()
        .find(|(_, p)| p.meta.identity.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(word)))
        .map(|(&no, _)| no)
}

fn find_type(query: &str) -> Result<WindowType, String> {
    let types = lumacore_registry::get().window_types();
    types
        .values()
        .find(|t| !t.hidden && t.matches(query))
        .cloned()
        .ok_or_else(|| {
            let names: Vec<&str> = types.values().filter(|t| !t.hidden).map(|t| t.label.as_str()).collect();
            format!("Unknown window type '{query}'. Types: {}", names.join(", "))
        })
}

/// `[screen.]n` → (live screen, window), in the line's screen by default.
fn window_address(state: &AppState, text: &str) -> Result<(u32, u32), String> {
    let (screen, no) = match text.split_once('.') {
        Some((s, n)) => (s.parse::<u32>().ok(), n),
        None => (None, text),
    };
    let no: u32 = no.parse().map_err(|_| format!("'{text}' is not a window ([screen.]n)"))?;
    let screen = match screen {
        Some(s) => s,
        None => {
            let user = state.effective_user_name().to_string();
            super::views::target_screens(state, &user, &[])?[0]
        }
    };
    window_of(state, screen, no).map(|_| (screen, no)).ok_or_else(|| format!("No window {screen}.{no}"))
}

fn window_of(state: &AppState, screen: u32, no: u32) -> Option<Window> {
    let store = state.root.runtime.live_screens.read().ok()?;
    store.get(screen)?.windows.get(&no).cloned()
}

/// Profile, window and type of a line. `need_window`: the window must be
/// known (store / load); else `/Type` may stand in for it.
fn target(state: &AppState, profile: Option<u32>, p: &Parsed, need_window: bool) -> Result<Target, String> {
    let profile = match profile {
        Some(no) => no,
        None => own_profile(state).ok_or_else(|| render_code("You have no profile.", "E-CMD-011"))?,
    };
    let window = match &p.window {
        Some(w) => Some(window_address(state, w).map_err(|e| render_code(e, "E-CMD-007"))?),
        None if need_window || p.ty.is_none() => match crate::self_ref::owner_window(state) {
            Ok(w) => Some(w),
            Err(e) if need_window => return Err(e),
            Err(_) => None,
        },
        None => None,
    };
    let ty = match (&p.ty, window) {
        (Some(t), _) => Some(find_type(t).map_err(|e| render_code(e, "E-CMD-008"))?),
        (None, Some((s, n))) => {
            let kind = window_of(state, s, n).map(|w| w.kind).unwrap_or_default();
            lumacore_registry::get().window_types().values().find(|t| t.name == kind && !t.hidden).cloned()
        }
        (None, None) => None,
    };
    if need_window && ty.is_none() {
        return Err(render_code("This window has no presets (an empty pane).", "E-CMD-010"));
    }
    Ok(Target { profile, window, ty })
}

fn presets<'a>(state: &'a AppState, profile: u32, kind: &str) -> Option<&'a BTreeMap<u32, WindowPreset>> {
    state.root.show_data.user_profiles.get(&profile)?.content.window_presets.get(kind)
}

fn profile_label(state: &AppState, no: u32) -> String {
    state
        .root
        .show_data
        .user_profiles
        .get(&no)
        .and_then(|p| p.meta.identity.name.clone())
        .map_or(format!("UserProfile {no}"), |n| format!("UserProfile {}", quote(&n)))
}

/// The window is as the preset left it (the settings window's "changed").
pub fn matches_preset(win: &Window, p: &WindowPreset) -> bool {
    let Some(ty) = lumacore_registry::get().window_types().values().find(|t| t.name == win.kind).cloned() else {
        return false;
    };
    ty.resolve_settings(&win.settings) == ty.resolve_settings(&p.settings)
        && win.label == p.label
        && win.appearance.color == p.colour
        && win.scroll == p.scroll
}

/// A window as a preset's content (every setting explicit: it holds everything).
fn content_of(win: &Window, ty: &WindowType) -> (Map<String, Value>, Option<String>, Option<String>, lumacore_tree::runtime::WindowScroll) {
    (
        ty.resolve_settings(&win.settings),
        win.label.clone(),
        win.appearance.color.clone(),
        win.scroll.clone(),
    )
}

/// Copy a preset into a window.
pub fn load_into(win: &mut Window, p: &WindowPreset) {
    win.settings = p.settings.clone();
    win.label = p.label.clone();
    win.appearance.color = p.colour.clone();
    win.scroll = p.scroll.clone();
    win.preset = Some(p.meta.identity.guid);
}

// ── Verbs ────────────────────────────────────────────────────────────────────

/// `Store WindowPreset [n] ["name"]`.
pub fn store(state: &mut AppState, profile: Option<u32>, args: &[&str]) -> String {
    let p = match parse(args) {
        Ok(p) => p,
        Err(e) => return render_code(e, "E-CMD-006"),
    };
    let t = match target(state, profile, &p, true) {
        Ok(t) => t,
        Err(e) => return e,
    };
    if let Err(e) = crate::commands::media::may_change_profile(state, t.profile) {
        return reply(e, "E-RIGHTS-001");
    }
    let ty = t.ty.expect("need_window gives a type");
    let (screen, wno) = t.window.expect("need_window gives a window");
    let Some(win) = window_of(state, screen, wno) else {
        return render_code(format!("No window {screen}.{wno}"), "E-CMD-007");
    };
    // `none` comes from the save dialog's pick list (no preset picked = new).
    let mut words: Vec<String> = p.words.iter().filter(|w| !w.eq_ignore_ascii_case("none")).cloned().collect();
    let number = words.first().and_then(|w| w.parse::<u32>().ok()).filter(|n| *n > 0);
    if number.is_some() {
        words.remove(0);
    }
    let name = (!words.is_empty()).then(|| words.join(" ")).filter(|n| !n.trim().is_empty());
    let pool = presets(state, t.profile, &ty.name).cloned().unwrap_or_default();
    let window_word = format!("/Window {screen}.{wno}");

    // Bare: ask for a name, new or which to overwrite (decisions §30 "12c").
    if number.is_none() && name.is_none() {
        let next = pool.keys().next_back().map_or(1, |n| n + 1);
        let pick = lumacore_wire::Pick {
            kind: "WindowPreset".into(),
            items: pool
                .iter()
                .map(|(no, p)| lumacore_wire::PickItem { no: *no, name: p.meta.name().unwrap_or_default().to_string(), note: "overwrite…".into(), guid: p.meta.identity.guid.to_string() })
                .collect(),
            optional: true,
            ..Default::default()
        };
        let default_name = win.label.clone().unwrap_or_else(|| format!("{} {next}", ty.label));
        state.asking = Some(
            crate::dialogs::Ask::new(
                format!("Save preset · {}", ty.label),
                format!("Window {screen}.{wno} as a {} preset of {}: everything — settings, bar, label, colour, scroll.", ty.label, profile_label(state, t.profile)),
            )
            .input("name", "Name", default_name, lumacore_wire::InputKind::Text)
            .input("preset", &format!("Overwrite (none = new preset {next})"), "none", lumacore_wire::InputKind::Link { pick })
            .button("Save", lumacore_wire::ButtonKind::Primary, format!("Store WindowPreset {{preset}} \"{{name}}\" {window_word}"))
            .cancel("Cancel")
            .flags("a name: Store WindowPreset \"name\""),
        );
        return render_success(&format!("Save preset · {}: asking for a name", ty.label));
    }

    let existing = number.and_then(|n| pool.get(&n)).cloned();
    if let (Some(old), Some(old_no)) = (&existing, number)
        && !p.overwrite
    {
        let old_name = old.meta.name().unwrap_or_default();
        let name = name.clone().unwrap_or_else(|| old_name.to_string());
        state.asking = Some(
            crate::dialogs::Ask::new(
                format!("Overwrite preset {old_no} \"{old_name}\"?"),
                format!("{} preset {old_no} gets Window {screen}.{wno} — everything: settings, bar, label, colour, scroll. Windows loaded from it keep what they have.", ty.label),
            )
            .button(
                "Overwrite",
                lumacore_wire::ButtonKind::Danger,
                format!("Store WindowPreset {old_no} \"{name}\" /Overwrite {window_word}"),
            )
            .cancel("Cancel")
            .flags("/Overwrite"),
        );
        return render_code(
            format!("{} preset {old_no} \"{old_name}\" exists. Add /Overwrite to replace it.", ty.label),
            "E-CMD-010",
        );
    }
    let no = number.unwrap_or_else(|| pool.keys().next_back().map_or(1, |n| n + 1));
    let (settings, label, colour, scroll) = content_of(&win, &ty);
    let preset = WindowPreset {
        // An overwrite keeps the preset's identity.
        meta: lumacore_base::NodeMeta::with_guid(
            existing.as_ref().map_or_else(uuid::Uuid::new_v4, |e| e.meta.identity.guid),
            Some(
                name.or_else(|| existing.as_ref().and_then(|e| e.meta.name().map(str::to_string)))
                    .unwrap_or_else(|| label.clone().unwrap_or_else(|| format!("{} {no}", ty.label))),
            ),
        ),
        kind: ty.name.clone(),
        label,
        colour,
        scroll,
        settings,
    };
    let guid = preset.meta.identity.guid;
    let shown = preset.meta.name().unwrap_or_default().to_string();
    let Some(prof) = state.root.show_data.profile_mut(t.profile) else {
        return render_code("No such profile.", "E-CMD-011");
    };
    prof.content.window_presets.entry(ty.name.clone()).or_default().insert(no, preset);
    state.show_dirty = true;
    // The window now is this preset's copy.
    if let Ok(mut store) = state.root.runtime.live_screens.write() {
        let _ = store.update_window(screen, wno, |w| {
            w.preset = Some(guid);
            Ok(())
        });
    }
    render_success(&format!(
        "{} preset {no} \"{shown}\" {} from Window {screen}.{wno}",
        ty.label,
        if existing.is_some() { "overwritten" } else { "stored" },
    ))
}

/// `WindowPreset n` loads into Self; bare asks which.
pub fn call(state: &mut AppState, profile: Option<u32>, args: &[&str]) -> String {
    let p = match parse(args) {
        Ok(p) => p,
        Err(e) => return render_code(e, "E-CMD-006"),
    };
    let t = match target(state, profile, &p, true) {
        Ok(t) => t,
        Err(e) => return e,
    };
    let ty = t.ty.clone().expect("a type");
    let (screen, wno) = t.window.expect("a window");
    let pool = presets(state, t.profile, &ty.name).cloned().unwrap_or_default();
    let number = p.words.first().and_then(|w| w.parse::<u32>().ok());
    let Some(number) = number else {
        if pool.is_empty() {
            return render_code(
                format!("No {} presets yet — Store WindowPreset saves this window as one.", ty.label),
                "E-CMD-009",
            );
        }
        let current = window_of(state, screen, wno).and_then(|w| w.preset);
        let pick = lumacore_wire::Pick {
            kind: "WindowPreset".into(),
            items: pool
                .iter()
                .map(|(no, p)| lumacore_wire::PickItem {
                    no: *no,
                    name: p.meta.name().unwrap_or_default().to_string(),
                    note: if Some(p.meta.identity.guid) == current { "this window came from it".into() } else { String::new() },
                    guid: p.meta.identity.guid.to_string(),
                })
                .collect(),
            ..Default::default()
        };
        let first = current
            .and_then(|g| pool.iter().find(|(_, p)| p.meta.identity.guid == g))
            .or_else(|| pool.iter().next())
            .map(|(no, _)| no.to_string())
            .unwrap_or_default();
        state.asking = Some(
            crate::dialogs::Ask::new(
                format!("Load preset · {}", ty.label),
                format!("Copies everything into Window {screen}.{wno} (settings, bar, label, colour, scroll); later changes don't touch the preset."),
            )
            .input("preset", "Preset", first, lumacore_wire::InputKind::Link { pick })
            .button("Load", lumacore_wire::ButtonKind::Primary, format!("WindowPreset {{preset}} /Window {screen}.{wno}"))
            .cancel("Cancel")
            .flags("a number: WindowPreset <n>"),
        );
        return render_success(&format!("Load preset · {}: asking which", ty.label));
    };
    let Some(preset) = pool.get(&number).cloned() else {
        return render_code(format!("No {} preset {number} in {}.", ty.label, profile_label(state, t.profile)), "E-CMD-007");
    };
    let result = state
        .root
        .runtime
        .live_screens
        .write()
        .map_err(|e| e.to_string())
        .and_then(|mut store| store.update_window(screen, wno, |w| {
            load_into(w, &preset);
            Ok(())
        }));
    match result {
        Ok(_) => render_success(&format!(
            "{} preset {number} \"{}\" → Window {screen}.{wno} (copied in)",
            ty.label, preset.meta.name().unwrap_or_default()
        )),
        Err(e) => render_code(e, "E-CMD-007"),
    }
}

/// The preset a Label / Delete / Info / Export line names.
pub fn named(state: &AppState, profile: Option<u32>, args: &[&str]) -> Result<(Target, WindowType, u32, WindowPreset, Vec<String>), String> {
    let p = parse(args).map_err(|e| render_code(e, "E-CMD-006"))?;
    let t = target(state, profile, &p, false)?;
    let ty = t.ty.clone().ok_or_else(|| {
        render_code("Which window type? Name it with /Type (e.g. /Type DMX) or run it from a window (Self).", "E-CMD-006")
    })?;
    let number = p
        .words
        .first()
        .and_then(|w| w.parse::<u32>().ok())
        .ok_or_else(|| render_code("Which preset? Give its number.", "E-CMD-006"))?;
    let preset = presets(state, t.profile, &ty.name)
        .and_then(|pool| pool.get(&number))
        .cloned()
        .ok_or_else(|| render_code(format!("No {} preset {number} in {}.", ty.label, profile_label(state, t.profile)), "E-CMD-007"))?;
    Ok((t, ty, number, preset, p.words[1..].to_vec()))
}

pub fn label(state: &mut AppState, profile: Option<u32>, args: &[&str]) -> String {
    let (t, ty, no, _, rest) = match named(state, profile, args) {
        Ok(x) => x,
        Err(e) => return e,
    };
    if let Err(e) = crate::commands::media::may_change_profile(state, t.profile) {
        return reply(e, "E-RIGHTS-001");
    }
    let name = rest.join(" ");
    if name.trim().is_empty() {
        return render_code("Usage: Label WindowPreset n \"name\"", "E-CMD-006");
    }
    if let Some(p) = state
        .root
        .show_data
        .profile_mut(t.profile)
        .and_then(|p| p.content.window_presets.get_mut(&ty.name))
        .and_then(|pool| pool.get_mut(&no))
    {
        p.meta.identity.name = Some(name.clone());
    }
    state.show_dirty = true;
    render_success(&format!("{} preset {no} labelled \"{name}\"", ty.label))
}

pub fn delete(state: &mut AppState, profile: Option<u32>, args: &[&str]) -> String {
    let (t, ty, no, preset, _) = match named(state, profile, args) {
        Ok(x) => x,
        Err(e) => return e,
    };
    if let Err(e) = crate::commands::media::may_change_profile(state, t.profile) {
        return reply(e, "E-RIGHTS-001");
    }
    if let Some(prof) = state.root.show_data.profile_mut(t.profile)
        && let Some(pool) = prof.content.window_presets.get_mut(&ty.name)
    {
        pool.remove(&no);
        if pool.is_empty() {
            prof.content.window_presets.remove(&ty.name);
        }
    }
    state.show_dirty = true;
    render_success(&format!("Deleted {} preset {no} \"{}\"", ty.label, preset.meta.name().unwrap_or_default()))
}

pub fn info(state: &mut AppState, profile: Option<u32>, args: &[&str]) -> String {
    let (t, ty, no, preset, _) = match named(state, profile, args) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let mut out = format!(
        "{} preset {no} \"{}\" ({}, GUID {})\n",
        ty.label,
        preset.meta.name().unwrap_or_default(),
        profile_label(state, t.profile),
        preset.meta.identity.guid
    );
    if let Some(l) = &preset.label {
        out.push_str(&format!("  label   {l}\n"));
    }
    if let Some(c) = &preset.colour {
        out.push_str(&format!("  colour  {c}\n"));
    }
    for def in &ty.settings {
        if let Some(v) = preset.settings.get(&def.key) {
            out.push_str(&format!("  {:<18} {}\n", def.key, def.label_of(v)));
        }
    }
    out.trim_end().to_string()
}

pub fn list(state: &mut AppState, profile: Option<u32>, args: &[&str]) -> String {
    let p = match parse(args) {
        Ok(p) => p,
        Err(e) => return render_code(e, "E-CMD-006"),
    };
    let t = match target(state, profile, &p, false) {
        Ok(t) => t,
        Err(e) => return e,
    };
    let Some(prof) = state.root.show_data.user_profiles.get(&t.profile) else {
        return render_code("No such profile.", "E-CMD-011");
    };
    let types = lumacore_registry::get().window_types();
    let label_of = |kind: &str| types.values().find(|t| t.name == kind).map_or(kind.to_string(), |t| t.label.clone());
    let mut out = format!("Window presets of {}:\n", profile_label(state, t.profile));
    let mut any = false;
    for (kind, pool) in &prof.content.window_presets {
        if t.ty.as_ref().is_some_and(|ty| ty.name != *kind) {
            continue;
        }
        for (no, p) in pool {
            any = true;
            out.push_str(&format!("  {:<16} {no:>3}  {}\n", label_of(kind), p.meta.name().unwrap_or_default()));
        }
    }
    if !any {
        return render_success(&format!(
            "No window presets{} in {} — Store WindowPreset saves a window.",
            t.ty.map(|t| format!(" of {}", t.label)).unwrap_or_default(),
            profile_label(state, t.profile)
        ));
    }
    out.trim_end().to_string()
}

fn complete(state: &AppState, prefix: &str) -> Vec<String> {
    let Some(prof) = state.root.show_data.profile_for_user(state.effective_user_name()) else {
        return Vec::new();
    };
    let kind = crate::self_ref::owner_window(state)
        .ok()
        .and_then(|(s, n)| window_of(state, s, n))
        .map(|w| w.kind);
    let nos: Vec<String> = prof
        .content.window_presets
        .iter()
        .filter(|(k, _)| kind.as_ref().is_none_or(|kind| kind == *k))
        .flat_map(|(_, pool)| pool.keys().map(u32::to_string))
        .collect();
    filter(nos, prefix)
}

fn reach(verb: Verb) -> Reach {
    match verb {
        Verb::Import | Verb::Export => Reach::Drive,
        _ => Reach::Show,
    }
}

/// `<verb> WindowPreset …` of your own profile.
fn own(verb: Verb) -> fn(&mut AppState, &[&str]) -> DispatchResult {
    match verb {
        Verb::Call => |s, a| out(call(s, None, a)),
        Verb::Store => |s, a| out(store(s, None, a)),
        Verb::Label => |s, a| out(label(s, None, a)),
        Verb::Delete => |s, a| out(delete(s, None, a)),
        Verb::Info => |s, a| out(info(s, None, a)),
        Verb::Export => |s, a| out(super::ui_files::export_preset(s, None, a)),
        Verb::Import => |s, a| out(super::ui_files::import_preset(s, None, a)),
        _ => |s, a| out(list(s, None, a)),
    }
}

/// `<verb> UserProfile [<n>|"name"] WindowPreset …` (owner first): Some when
/// the line names a WindowPreset after the profile.
pub fn on_profile(state: &mut AppState, verb: Verb, args: &[&str]) -> Option<DispatchResult> {
    let (profile, rest) = match args {
        [first, rest @ ..] if WINDOW_PRESET.matches(first) => (None, rest),
        [owner, second, rest @ ..] if WINDOW_PRESET.matches(second) => match profile_no(state, owner) {
            Some(no) => (Some(no), rest),
            None => return Some(out(render_code(format!("No UserProfile {owner}."), "E-CMD-007"))),
        },
        _ => return None,
    };
    Some(out(match verb {
        Verb::Call => call(state, profile, rest),
        Verb::Store => store(state, profile, rest),
        Verb::Label => label(state, profile, rest),
        Verb::Delete => delete(state, profile, rest),
        Verb::Info => info(state, profile, rest),
        Verb::Export => super::ui_files::export_preset(state, profile, rest),
        Verb::Import => super::ui_files::import_preset(state, profile, rest),
        _ => list(state, profile, rest),
    }))
}

pub static WINDOW_PRESET: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "WindowPreset",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Call,
        flags: &[WINDOW, F::TYPE, F::OVERWRITE.on(&[Verb::Store])],
        aliases: &["windowpresets"],
        doc: Doc {
            display: "WindowPreset",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Store WindowPreset [n] [\"name\"] [/Overwrite] | WindowPreset [n] | Label|Delete|Info|Export WindowPreset n | Import WindowPreset \"file\" | List WindowPreset [/Type t] | UserProfile [<n>|\"name\"] WindowPreset …",
            description: "A window's look and settings kept on your profile, per window type: everything \
                          of a window — its settings, window bar, label, colour and scroll. Store saves \
                          Self's window (the window the line belongs to; /Window [screen.]n names one), \
                          WindowPreset n copies one into it (no live link: the window only remembers \
                          where it came from). A bare Store WindowPreset / WindowPreset asks. Presets \
                          travel with Export UserProfile; Export / Import WindowPreset use JSON in \
                          library/ui/windowpresets/.",
            examples: &[
                "Store WindowPreset \"Big DMX\"        -- this window as a new preset",
                "WindowPreset 2                      -- load preset 2 into this window",
                "Store WindowPreset 2 /Overwrite      -- replace preset 2",
                "Label WindowPreset 2 \"Patch check\"",
                "List WindowPreset /Type DMX",
                "UserProfile \"Bob\" WindowPreset 1    -- load Bob's preset 1",
                "Export WindowPreset 2",
            ],
            related: &["self", "window", "userprofile"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        // Runs where its window lives (a member's profile writes go to its master).
        scope: crate::objects::on_station,
        reach,
        rights: crate::rights::screen,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Call, |s, a| own(Verb::Call)(s, a)),
        (Verb::Store, |s, a| own(Verb::Store)(s, a)),
        (Verb::Label, |s, a| own(Verb::Label)(s, a)),
        (Verb::Delete, |s, a| own(Verb::Delete)(s, a)),
        (Verb::List, |s, a| own(Verb::List)(s, a)),
        (Verb::Info, |s, a| own(Verb::Info)(s, a)),
        (Verb::Export, |s, a| own(Verb::Export)(s, a)),
        (Verb::Import, |s, a| own(Verb::Import)(s, a)),
    ],
    delete_ask: None,
    complete: Some(complete),
    props: None,
    tree: None,
};
