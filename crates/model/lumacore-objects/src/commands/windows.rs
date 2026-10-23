//! Windows on a screen (ScreenContent / ViewWidget).
//!
//! A window lives in a screen (screen) at
//! `Root.Runtime.Users.<user>.Screens.<screen>.Windows.<no>` and is
//! addressed as `[screen.]window`. Without a screen, the screen the command
//! came from is used (the screen with focus).
//!
//! ```text
//! Store Window "Groups"                  new Groups window, as a tab beside the focused window
//! Store Window 7 "Groups" /Pane 3        window 7, as a tab beside window 3
//! Store Window 2.7 "Terminal"            on screen 2
//! Store Window 5 "Terminal" /Overwrite   change window 5's type in place (settings reset)
//! Delete Window 5 | Delete Window 2.*    close a window | every window on screen 2
//! Label Window 5 "Front"                 tab title (no name asks; /Clear clears it)
//! Set Window 1.5 "Columns" 8               a typed setting (see Root.Resources.WindowTypes)
//! Window 5                               focus window 5 (default action)
//! Window                                 list the windows of this screen
//! ```
//!
//! Splitting panes is a layout edit in the UI, not a command: a new pane holds
//! an empty placeholder window, and storing into it needs no `/Overwrite`.

use crate::commands::flags as F;
use lumacore_tree::WindowType;
use lumacore_tree::runtime::{EMPTY_WINDOW, LayoutNode};

use super::views::target_screens;
use crate::AppState;
use crate::errors::render_code;
use crate::text::render_success;
use lumacore_wire::UiEvent;

/// Which windows a command names.
#[derive(Debug, PartialEq)]
enum Which {
    One(u32),
    All,
}

#[derive(Debug, Default)]
struct Parsed {
    /// Screen from `2.7` or `/Screen 2`.
    screen: Option<u32>,
    which: Option<Which>,
    pane: Option<u32>,
    overwrite: bool,
    /// `/Clear` (Label: the label goes).
    clear: bool,
    rest: Vec<String>,
}

/// `5`, `2.5`, `2.*`, `*` → (screen, which).
fn address(token: &str) -> Option<(Option<u32>, Which)> {
    let which = |t: &str| match t {
        "*" => Some(Which::All),
        n => n.parse::<u32>().ok().filter(|n| *n > 0).map(Which::One),
    };
    match token.split_once('.') {
        Some((screen, w)) => Some((
            Some(screen.parse::<u32>().ok().filter(|n| *n > 0)?),
            which(w)?,
        )),
        None => Some((None, which(token)?)),
    }
}

/// `Store Window … /Pane n` (decisions §31: declared on the Window object).
const PANE: F::FlagDef = F::FlagDef::new("Pane", "The pane it goes in").alias(&["p"]).value("pane");

fn parse(args: &[&str]) -> Result<Parsed, String> {
    let mut p = Parsed::default();
    let mut args = args;
    if let Some((screen, which)) = args.first().and_then(|t| address(t)) {
        p.screen = screen;
        p.which = Some(which);
        args = &args[1..];
    }
    // A flag's number: given and > 0 (`/Pane 2`, `/s=3`).
    let number = |def: &F::FlagDef| -> Result<Option<u32>, String> {
        if !F::has(args, def) {
            return Ok(None);
        }
        F::value(args, def)
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|n| *n > 0)
            .map(Some)
            .ok_or_else(|| format!("/{} needs a number", def.name))
    };
    p.pane = number(&PANE)?;
    if let Some(screen) = number(&F::SCREEN)? {
        p.screen = Some(screen);
    }
    p.overwrite = F::has(args, &F::OVERWRITE);
    p.clear = F::has(args, &F::CLEAR);
    p.rest = F::positional(args, &[&PANE, &F::SCREEN]).into_iter().map(str::to_string).collect();
    Ok(p)
}

fn unquote(parts: &[String]) -> Option<String> {
    let name = parts
        .join(" ")
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .trim()
        .to_string();
    (!name.is_empty()).then_some(name)
}

fn reply(result: Result<String, String>) -> String {
    result.map_or_else(|e| render_code(&e, "E-CMD-006"), |m| render_success(&m))
}

/// The one screen to act on.
fn screen(state: &AppState, p: &Parsed) -> Result<u32, String> {
    let user = state.effective_user_name().to_string();
    let given: Vec<u32> = p.screen.into_iter().collect();
    Ok(target_screens(state, &user, &given)?[0])
}

fn find_type<'a>(state: &'a AppState, query: &str) -> Result<&'a WindowType, String> {
    let types = || {
        state
            .root
            .resources
            .window_types
            .values()
            .filter(|t| !t.hidden)
    };
    types().find(|t| t.matches(query)).ok_or_else(|| {
        let names: Vec<&str> = types().map(|t| t.label.as_str()).collect();
        format!("Unknown window type '{query}'. Types: {}", names.join(", "))
    })
}

/// `(kind, label)` of a window, if it exists on `vs`.
fn window_info(state: &AppState, vs: u32, w: u32) -> Option<(String, Option<String>)> {
    let store = state.root.runtime.live_screens.read().ok()?;
    let win = store.get(vs)?.windows.get(&w)?;
    Some((win.kind.clone(), win.label.clone()))
}

fn type_label(state: &AppState, kind: &str) -> String {
    crate::vfs_runtime::window_type(&state.root, kind)
        .map_or_else(|| kind.to_string(), |t| t.label.clone())
}

fn one(p: &Parsed, usage: &str) -> Result<u32, String> {
    match p.which {
        Some(Which::One(w)) => Ok(w),
        _ => Err(usage.to_string()),
    }
}

/// `Store Window [[screen.]n] "Type" [/Pane n] [/Overwrite]` — args after `window`.
pub fn cmd_store_window(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let vs = screen(state, &p)?;
        let query =
            unquote(&p.rest).ok_or("Usage: Store Window [n] \"Type\" [/Pane n] [/Overwrite]")?;
        let ty = find_type(state, &query)?;
        let (kind, label) = (ty.name.clone(), ty.label.clone());
        let window = match p.which {
            Some(Which::All) => return Err("Store Window needs one window number, not *".into()),
            Some(Which::One(w)) => Some(w),
            None => None,
        };

        // An existing window: retype it (empty panes need no /Overwrite).
        if let Some(w) = window
            && let Some((old, _)) = window_info(state, vs, w)
        {
            if old != EMPTY_WINDOW && !p.overwrite {
                let old = type_label(state, &old);
                return Err(format!(
                    "Window {vs}.{w} exists ({old}). Use /Overwrite to change its type"
                ));
            }
            let mut store = state
                .root
                .runtime
                .live_screens
                .write()
                .map_err(|e| e.to_string())?;
            store.set_window_kind(vs, w, &kind)?;
            drop(store);
            focus_if_here(state, vs, w);
            return Ok(format!("Window {vs}.{w} \"{label}\""));
        }

        // A new window, beside /Pane, else the focused window on this screen.
        let beside = match p.pane {
            Some(pane) => {
                window_info(state, vs, pane)
                    .ok_or_else(|| format!("No window {vs}.{pane} for /Pane"))?;
                Some(pane)
            }
            None => state.current_window.filter(|w| {
                state.current_live_screen == Some(vs) && window_info(state, vs, *w).is_some()
            }),
        };
        let mut store = state
            .root
            .runtime
            .live_screens
            .write()
            .map_err(|e| e.to_string())?;
        let (w, _) = store.add_window(vs, &kind, window, beside)?;
        drop(store);
        focus_if_here(state, vs, w);
        Ok(format!("Window {vs}.{w} \"{label}\" stored"))
    }))
}

/// Ask the issuing client to focus a window it just stored (if it shows that
/// screen).
fn focus_if_here(state: &mut AppState, vs: u32, w: u32) {
    if state.current_live_screen == Some(vs) {
        state.ui_requests.push(UiEvent::FocusWindow { no: w });
    }
}

/// `Delete Window [screen.]n | [screen.]*` — args after `window`.
pub fn cmd_delete_window(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let vs = screen(state, &p)?;
        let gone: Vec<u32> = match p.which {
            Some(Which::One(w)) => vec![w],
            Some(Which::All) => {
                let store = state
                    .root
                    .runtime
                    .live_screens
                    .read()
                    .map_err(|e| e.to_string())?;
                store
                    .get(vs)
                    .map(|s| s.windows.keys().copied().collect())
                    .unwrap_or_default()
            }
            None => return Err("Usage: Delete Window [screen.]n | [screen.]*".into()),
        };
        let mut store = state
            .root
            .runtime
            .live_screens
            .write()
            .map_err(|e| e.to_string())?;
        store.remove_windows(vs, &gone)?;
        Ok(match p.which {
            Some(Which::All) => format!("Deleted all windows on screen {vs}"),
            _ => format!("Deleted Window {vs}.{}", gone[0]),
        })
    }))
}

/// `Label Window [screen.]n "name" | /Clear`. Without a name it asks (the
/// Label dialog, decisions §30 "12b": the current name, Please labels, an
/// empty answer clears) — the window menu's Rename sends the bare line.
pub fn cmd_label_window(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let vs = screen(state, &p)?;
        let w = one(&p, "Usage: Label Window [screen.]n \"name\" | /Clear")?;
        let name = unquote(&p.rest);
        if name.is_none() && !p.clear {
            let (_, current) = window_info(state, vs, w).ok_or_else(|| format!("No window {vs}.{w}"))?;
            let address = format!("{vs}.{w}");
            state.asking = Some(
                crate::dialogs::Ask::new(
                    format!("Label Window {address}"),
                    "A new name for the tab. Empty clears it (the type's name shows).",
                )
                .input("name", "Name", current.unwrap_or_default(), lumacore_wire::InputKind::Text)
                .button(
                    "Please",
                    lumacore_wire::ButtonKind::Please,
                    format!("Label Window {address} \"{{name}}\""),
                )
                .or_when_empty(format!("Label Window {address} /Clear"))
                .cancel("Cancel")
                .flags(format!("the name (Label Window {address} \"name\"), or /Clear")),
            );
            return Err(format!("Usage: Label Window {address} \"name\" | /Clear"));
        }
        let mut store = state
            .root
            .runtime
            .live_screens
            .write()
            .map_err(|e| e.to_string())?;
        store.update_window(vs, w, |win| {
            win.label = name.clone();
            Ok(())
        })?;
        Ok(match name {
            Some(name) => format!("Window {vs}.{w} labelled \"{name}\""),
            None => format!("Window {vs}.{w} label cleared"),
        })
    }))
}

/// `Reset Window [screen.]n` / `Reset Self` (decisions §30 "12c"): every
/// setting and the window bar back to the type's defaults; label, colour,
/// scroll and position stay. Its layout change is one UI Oops step.
pub fn cmd_reset_window(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let vs = screen(state, &p)?;
        let w = one(&p, "Usage: Reset Window [screen.]n  ·  Reset Self")?;
        let (kind, label) = window_info(state, vs, w).ok_or_else(|| format!("No window {vs}.{w}"))?;
        let mut store = state.root.runtime.live_screens.write().map_err(|e| e.to_string())?;
        store.update_window(vs, w, |win| {
            win.reset_settings();
            Ok(())
        })?;
        drop(store);
        let ty = type_label(state, &kind);
        let name = label.map(|l| format!(" \"{l}\"")).unwrap_or_default();
        Ok(format!(
            "Window {vs}.{w}{name} back to the {ty} defaults (settings and window bar; label, colour and position kept)"
        ))
    }))
}

/// `Reset UserProfile [<n>|"name"] Window /Type <type>` (decisions §30
/// "12c"): every window of that type in the profile's screen configs and
/// views back to the type's defaults (settings and bar; label, colour and
/// position stay), live screens of the profile's users included. One Oops
/// step; it asks first when it touches more than one window
/// (`/NoConfirmation` answers).
pub fn cmd_reset_profile_windows(state: &mut AppState, args: &[&str]) -> String {
    let confirmed = F::has(args, &F::NO_CONFIRMATION);
    let usage = "Usage: Reset UserProfile [<n>|\"name\"] Window /Type <type>";
    let kind = F::value(args, &F::TYPE);
    let words = F::positional(args, &[&F::TYPE]);
    let (profile, rest) = match words.as_slice() {
        [w, rest @ ..] if WINDOW.matches(w) => (state.root.show_data.profile_id_for_user(state.effective_user_name()), rest),
        [owner, w, rest @ ..] if WINDOW.matches(w) => (super::window_presets::profile_no(state, owner), rest),
        _ => return render_code(usage, "E-CMD-006"),
    };
    if !rest.is_empty() {
        return render_code(usage, "E-CMD-006");
    }
    let Some(profile) = profile else {
        return render_code("No such UserProfile.", "E-CMD-007");
    };
    if let Err(e) = super::media::may_change_profile(state, profile) {
        return crate::errors::reply(e, "E-RIGHTS-001");
    }
    let Some(kind) = kind else {
        return render_code(format!("{usage} — which window type?"), "E-CMD-006");
    };
    let ty = match find_type(state, &kind) {
        Ok(t) => t.clone(),
        Err(e) => return render_code(e, "E-CMD-008"),
    };
    let prof = &state.root.show_data.user_profiles[&profile];
    let pname = prof.meta.identity.name.clone().unwrap_or_else(|| format!("{profile}"));
    let pguid = prof.meta.identity.guid;
    // Where they are: saved screens and views (show data), live screens of
    // the profile's users (shown by their live number).
    let mut places: Vec<String> = Vec::new();
    for (c, config) in &prof.content.screen_configs {
        for (sn, screen) in &config.screens {
            for (wn, w) in &screen.windows {
                if w.kind == ty.name {
                    places.push(format!("ScreenConfig {c} Screen {sn} Window {wn}"));
                }
            }
        }
    }
    let mut in_views = 0;
    for view in prof.content.views.values() {
        for screen in view.screens.values() {
            in_views += screen.windows.values().filter(|w| w.kind == ty.name).count();
        }
    }
    let users: Vec<String> = state
        .root
        .show_data
        .users
        .values()
        .filter(|u| u.profile == pguid)
        .map(|u| u.login.clone())
        .collect();
    let live: Vec<(u32, u32)> = state
        .root
        .runtime
        .live_screens
        .read()
        .map(|store| {
            store
                .all()
                .filter(|s| users.iter().any(|u| u.eq_ignore_ascii_case(&s.user)))
                .flat_map(|s| s.windows.iter().filter(|(_, w)| w.kind == ty.name).map(move |(n, _)| (s.no, *n)))
                .collect()
        })
        .unwrap_or_default();
    // A live window is its saved screen's window: count it once.
    let count = places.len().max(live.len()) + in_views;
    if count == 0 {
        return render_code(format!("UserProfile \"{pname}\" has no {} windows.", ty.label), "E-CMD-009");
    }
    if count > 1 && !confirmed {
        let shown: Vec<String> = if live.is_empty() {
            places.iter().take(6).cloned().collect()
        } else {
            live.iter().map(|(s, n)| format!("Window {s}.{n}")).collect()
        };
        let views = if in_views > 0 { format!(" and {in_views} in stored views") } else { String::new() };
        state.asking = Some(
            crate::dialogs::Ask::new(
                format!("Reset {count} {} windows?", ty.label),
                format!(
                    "Profile \"{pname}\": {}{views} go back to the {} defaults (settings and bar). Labels, colours and positions stay. One Oops step.",
                    shown.join(", "),
                    ty.label
                ),
            )
            .button(
                &format!("Reset {count} windows"),
                lumacore_wire::ButtonKind::Danger,
                format!("Reset UserProfile {} Window /Type {} /NoConfirmation", super::super::objects::quote(&pname), ty.name),
            )
            .cancel("Cancel")
            .topic("reset:windows")
            .flags("/NoConfirmation"),
        );
        return render_success(&format!("Reset {count} {} windows? Asking first.", ty.label));
    }
    // The show's copies (one change: one Oops step) …
    if let Some(prof) = state.root.show_data.profile_mut(profile) {
        for config in prof.content.screen_configs.values_mut() {
            for screen in config.screens.values_mut() {
                screen.windows.values_mut().filter(|w| w.kind == ty.name).for_each(|w| w.reset_settings());
            }
        }
        for view in prof.content.views.values_mut() {
            for screen in view.screens.values_mut() {
                screen.windows.values_mut().filter(|w| w.kind == ty.name).for_each(|w| w.reset_settings());
            }
        }
    }
    state.show_dirty = true;
    // … and the live screens the same (their sync then finds nothing new).
    if let Ok(mut store) = state.root.runtime.live_screens.write() {
        for (s, n) in &live {
            let _ = store.update_window(*s, *n, |w| {
                w.reset_settings();
                Ok(())
            });
        }
    }
    render_success(&format!(
        "{count} {} window(s) of UserProfile \"{pname}\" back to the defaults (settings and bar)",
        ty.label
    ))
}

/// The preset a window came from, as the settings window shows it:
/// `from preset 2 "Big DMX"` (` · changed` once it differs).
fn preset_note(state: &AppState, vs: u32, w: u32) -> String {
    let Ok(store) = state.root.runtime.live_screens.read() else {
        return String::new();
    };
    let Some(screen) = store.get(vs) else {
        return String::new();
    };
    let Some(win) = screen.windows.get(&w) else {
        return String::new();
    };
    let Some(guid) = win.preset else {
        return String::new();
    };
    let found = state
        .root
        .show_data
        .profile_for_user(&screen.user)
        .and_then(|p| p.content.window_presets.get(&win.kind))
        .and_then(|pool| pool.iter().find(|(_, p)| p.meta.identity.guid == guid));
    match found {
        Some((no, p)) => {
            let changed = if super::window_presets::matches_preset(win, p) { "" } else { " · changed" };
            format!("from preset {no} \"{}\"{changed}", p.meta.name().unwrap_or_default())
        }
        None => "from a preset that is gone".to_string(),
    }
}

/// `EditSetting Window [screen.]n` (decisions §30 "12b2"): the window's
/// settings as typed fields, one tab per section; each change is `Set Window`.
pub fn window_fields(
    state: &AppState,
    args: &[&str],
) -> Result<WindowFields, String> {
    use lumacore_tree::window_types::{FOLLOW_SELECTED, SettingKind};
    use lumacore_wire::{FieldInput, ListItem, SettingField};
    let p = parse(args).map_err(|e| render_code(&e, "E-CMD-008"))?;
    let usage = "Usage: EditSetting Window [screen.]n";
    let vs = screen(state, &p).map_err(|e| render_code(&e, "E-CMD-007"))?;
    let w = one(&p, usage).map_err(|e| render_code(&e, "E-CMD-006"))?;
    let (kind, label, stored) = {
        let store = state.root.runtime.live_screens.read().map_err(|e| e.to_string())?;
        let win = store
            .get(vs)
            .and_then(|s| s.windows.get(&w))
            .ok_or_else(|| render_code(format!("No window {vs}.{w}"), "E-CMD-007"))?;
        (win.kind.clone(), win.label.clone(), win.settings.clone())
    };
    let ty = crate::vfs_runtime::window_type(&state.root, &kind)
        .ok_or_else(|| render_code(format!("Unknown window type '{kind}'"), "E-CMD-007"))?;
    if ty.settings.is_empty() {
        return Err(render_code(format!("{} has no settings.", ty.label), "E-CMD-015"));
    }
    let values = ty.resolve_settings(&stored);
    let one_section = ty.sections().len() < 2;
    let fields = ty
        .settings
        .iter()
        .map(|def| {
            let v = values.get(&def.key).cloned().unwrap_or_default();
            let input = match &def.kind {
                SettingKind::Bool => FieldInput::Toggle,
                SettingKind::Int { min, max, named } => FieldInput::Number {
                    min: *min,
                    max: *max,
                    named: named.iter().map(|(_, l)| l.clone()).collect(),
                },
                SettingKind::Enum { options } => FieldInput::Choice {
                    options: options.iter().map(|(_, l)| l.clone()).collect(),
                },
                SettingKind::Text { max_len } => FieldInput::Text { max_len: *max_len },
                SettingKind::Link { .. } => FieldInput::Link,
                SettingKind::Colour => FieldInput::Colour,
                SettingKind::List => FieldInput::List {
                    items: ty
                        .bar_ids()
                        .into_iter()
                        .map(|id| match (ty.setting(id), ty.action(id)) {
                            (Some(d), _) => ListItem { id: d.key.clone(), label: d.label.clone(), kind: "setting".into(), line: String::new() },
                            (_, Some(a)) => ListItem { id: a.id.clone(), label: a.label.clone(), kind: "action".into(), line: a.line.clone() },
                            _ => ListItem { id: id.to_string(), label: id.to_string(), kind: "setting".into(), line: String::new() },
                        })
                        .collect(),
                },
            };
            let pick = match &def.kind {
                SettingKind::Link { kind, follow, by_guid } => Some(link_pick(&state.root.show_data, kind, follow.clone(), *by_guid, &v)),
                _ => None,
            };
            let value = match (&def.kind, &pick) {
                (SettingKind::Link { .. }, Some(pick)) if v.as_str() != Some(FOLLOW_SELECTED) => pick
                    .items
                    .iter()
                    .find(|i| v.as_str() == Some(i.guid.as_str()) || v.as_u64() == Some(u64::from(i.no)))
                    .map(|i| format!("{} {}", i.no, i.name).trim().to_string())
                    .or_else(|| pick.missing.clone())
                    .unwrap_or_else(|| def.label_of(&v)),
                (SettingKind::Colour, _) if v.as_str().is_none_or(str::is_empty) => "Theme".to_string(),
                _ => def.label_of(&v),
            };
            SettingField {
                name: def.label.clone(),
                key: def.key.clone(),
                changed: v != def.default,
                reset: format!("Reset Window {vs}.{w} \"{}\"", setting_word(&def.key)),
                section: if one_section { String::new() } else { def.section.clone() },
                input,
                hint: def.hint.clone().unwrap_or_default(),
                value,
                line: format!("Set Window {vs}.{w} \"{}\" {{value}}", setting_word(&def.key)),
                pick,
                // A window's settings are its profile's (decisions §39).
                home: "user".into(),
                unit: String::new(),
                rights: String::new(),
                readonly: String::new(),
            }
        })
        .collect();
    Ok(WindowFields {
        title: format!("Window {vs}.{w} · {}", label.unwrap_or_else(|| ty.label.clone())),
        fields,
        subtitle: preset_note(state, vs, w),
        target: format!("Window {vs}.{w}"),
    })
}

/// A window's settings window: title, typed cells, the preset it came from
/// and the window as a command word.
pub struct WindowFields {
    pub title: String,
    pub fields: Vec<lumacore_wire::SettingField>,
    pub subtitle: String,
    pub target: String,
}

/// The pick list of a window setting's link: DataPools by GUID (with the
/// follow row), universes by number. Also the window bar's pick lists
/// (`windows` feed `picks`, `crate::window_state`).
pub fn link_pick(
    show: &lumacore_tree::ShowData,
    kind: &str,
    follow: Option<String>,
    by_guid: bool,
    current: &serde_json::Value,
) -> lumacore_wire::Pick {
    // tidy: not-keywords (the link kinds a window setting names)
    let items: Vec<lumacore_wire::PickItem> = match kind {
        "DataPool" => show
            .datapools
            .iter()
            .map(|(no, dp)| lumacore_wire::PickItem {
                no: *no,
                name: dp.meta.identity.name.clone().unwrap_or_default(),
                note: format!("{} groups", dp.groups.len()),
                guid: dp.meta.identity.guid.to_string(),
            })
            .collect(),
        // A sequence pick lists every DataPool's (`2.3` = DataPool 2's Sequence 3).
        "Sequence" => show
            .sequences()
            .map(|(dp, no, seq)| lumacore_tree_pick(dp, no, seq))
            .collect(),
        // The Presets window's feature group (decisions 0288): the show's groups, and All.
        "FeatureGroup" => std::iter::once(lumacore_wire::PickItem { no: 0, name: "All".into(), note: "every preset".into(), guid: "all".into() })
            .chain(show.feature_groups.iter().map(|(no, g)| lumacore_wire::PickItem {
                no: *no,
                name: g.meta.identity.name.clone().unwrap_or_default(),
                note: String::new(),
                guid: g.meta.identity.guid.to_string(),
            }))
            .collect(),
        "UserProfile" => show
            .user_profiles
            .iter()
            .map(|(no, p)| lumacore_wire::PickItem {
                no: *no,
                name: p.meta.identity.name.clone().unwrap_or_default(),
                note: format!("{} views", p.content.views.len()),
                guid: p.meta.identity.guid.to_string(),
            })
            .collect(),
        // A tag's number is a path (`1.2`): the list numbers the rows, the page answers by GUID.
        "Tag" => lumacore_tree::tags::tag::all(&show.tags)
            .into_iter()
            .enumerate()
            .map(|(i, (path, t))| lumacore_wire::PickItem {
                no: i as u32 + 1,
                name: format!("{path} {}", t.name()),
                note: String::new(),
                guid: t.meta.identity.guid.to_string(),
            })
            .collect(),
        _ => show
            .devices
            .dmx
            .patch
            .universes
            .iter()
            .map(|(no, u)| lumacore_wire::PickItem {
                no: *no,
                name: u.meta.identity.name.clone().unwrap_or_default(),
                note: String::new(),
                guid: u.meta.identity.guid.to_string(),
            })
            .collect(),
    };
    let missing = by_guid
        .then(|| current.as_str())
        .flatten()
        .filter(|g| *g != lumacore_tree::window_types::FOLLOW_SELECTED && !items.iter().any(|i| i.guid == *g))
        .map(|g| format!("missing · {}", g.get(..8).unwrap_or(g)));
    lumacore_wire::Pick {
        kind: kind.to_string(),
        items,
        optional: false,
        follow,
        missing,
        add: None,
        add_no: None,
        by_guid: kind == "Tag",
    }
}

/// One sequence in a pick list: its number in its DataPool (the note names the DataPool).
fn lumacore_tree_pick(dp: u32, no: u32, seq: &lumacore_tree::SequenceObject) -> lumacore_wire::PickItem {
    lumacore_wire::PickItem {
        no,
        name: seq.meta.identity.name.clone().unwrap_or_default(),
        note: format!("DataPool {dp} · {} cues", seq.cues.len()),
        guid: seq.meta.identity.guid.to_string(),
    }
}

/// A typed link value as stored (decisions §30 "12b2"): a DataPool by number,
/// name or GUID → its GUID; the follow word stays; anything else as typed.
pub fn resolve_window_link(state: &AppState, kind: &str, value: &str) -> String {
    let v = value.trim().trim_matches('"');
    if v.eq_ignore_ascii_case(lumacore_tree::window_types::FOLLOW_SELECTED) || uuid::Uuid::parse_str(v).is_ok() {
        return v.to_string();
    }
    // A sequence by number (in your selected DataPool) or name.
    if kind == "Sequence" {
        let show = &state.root.show_data;
        let found = match v.parse::<u32>() {
            Ok(0) => return lumacore_tree::window_types::FOLLOW_SELECTED.to_string(),
            Ok(n) => show.datapools.get(&state.selected_datapool()).and_then(|d| d.sequences.get(&n)),
            Err(_) => show.sequences().map(|(_, _, s)| s).find(|s| s.meta.identity.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(v))),
        };
        return found.map_or_else(|| v.to_string(), |s| s.meta.identity.guid.to_string());
    }
    if kind != "DataPool" {
        // Any other kind with an object (a Tag by path or name): the one resolver.
        let one = crate::objects::find(kind)
            .and_then(|def| crate::resolve::words(state, def, &[value.trim()]).ok())
            .and_then(|r| (r.members.len() == 1).then(|| r.members[0].guid).flatten());
        return one.map_or_else(|| v.to_string(), |g| g.to_string());
    }
    let pools = &state.root.show_data.datapools;
    let found = match v.parse::<u32>() {
        Ok(0) => return lumacore_tree::window_types::FOLLOW_SELECTED.to_string(),
        Ok(n) => pools.get(&n),
        Err(_) => pools.values().find(|d| d.meta.identity.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(v))),
    };
    found.map_or_else(|| v.to_string(), |d| d.meta.identity.guid.to_string())
}

/// The screen's held-full-screen window, as a `Set Window` key.
pub const MAXIMIZED: &str = "Maximized";

/// `Set Window [screen.]n Maximized On|Off|Toggle` (`Set Self "Maximized" On`
/// from a held pane, decisions §30 "12c2"): every client of the live screen
/// shows the window full screen (`maximized` in the screen's JSON). A
/// profile with `HoldFollows Off` keeps a held pane on its device only (the
/// page then sends nothing).
fn set_maximized(state: &mut AppState, vs: u32, w: u32, value: &str) -> String {
    let Ok(mut store) = state.root.runtime.live_screens.write() else {
        return render_code("screens unavailable", "E-CMD-010");
    };
    let current = store.get(vs).and_then(|s| s.maximized);
    let on = match value.to_ascii_lowercase().as_str() {
        "on" | "yes" | "true" | "1" => true,
        "off" | "no" | "false" | "0" => false,
        "toggle" => current != Some(w),
        _ => return render_code(format!("Maximized takes On, Off or Toggle, not '{value}'."), "E-CMD-008"),
    };
    let next = match (on, current) {
        (true, _) => Some(w),
        (false, Some(m)) if m == w => None,
        (false, other) => other,
    };
    if next == current {
        return render_success(&format!("Window {vs}.{w} {}", if on { "is maximized" } else { "isn't maximized" }));
    }
    match store.set_maximized(vs, next) {
        Ok(_) => render_success(&format!("Window {vs}.{w} {}", if on { "maximized" } else { "back in its pane" })),
        Err(e) => render_code(e, "E-CMD-007"),
    }
}

/// `Window [screen.]n` focuses it; bare `Window` lists this screen's windows.
pub fn cmd_window(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let vs = screen(state, &p)?;
        match p.which {
            None => list_windows(state, vs),
            Some(Which::All) => Err("Usage: Window [screen.]n".into()),
            Some(Which::One(w)) => {
                let (kind, label) =
                    window_info(state, vs, w).ok_or_else(|| format!("No window {vs}.{w}"))?;
                if state.current_live_screen != Some(vs) {
                    return Err(format!("Window {vs}.{w} is on another screen"));
                }
                state.ui_requests.push(UiEvent::FocusWindow { no: w });
                Ok(format!(
                    "Window {vs}.{w} \"{}\"",
                    label.unwrap_or_else(|| type_label(state, &kind))
                ))
            }
        }
    }))
}

/// `ReloadUI [/Screen "a,b"] [/All]`: UI engines reload the
/// screen the command came from, else all of the user's screens; `/All`
/// reloads every UI client.
pub fn cmd_reload_ui(state: &mut AppState, args: &[&str]) -> String {
    let user = state.effective_user_name().to_string();
    let all = F::has(args, &F::ALL);
    let mut given = Vec::new();
    if !F::positional(args, &[&F::SCREEN]).is_empty() {
        return render_code("Usage: ReloadUI [/Screen \"a,b\"] [/All]", "E-CMD-006");
    }
    if F::has(args, &F::SCREEN) {
        let Some(value) = F::value(args, &F::SCREEN) else {
            return render_code("/Screen needs screen numbers, e.g. /Screen \"1,2\"", "E-CMD-006");
        };
        for part in value.split(',') {
            match part.trim().parse::<u32>() {
                Ok(n) => given.push(n),
                Err(_) => return render_code(format!("/Screen: '{part}' is not a screen number"), "E-CMD-008"),
            }
        }
    }

    let Ok(store) = state.root.runtime.live_screens.read() else {
        return render_code("screens unavailable", "E-CMD-010");
    };
    if all {
        store.reload_ui(None);
        return render_success("Reloading all UIs");
    }
    let screens: Vec<u32> = if !given.is_empty() {
        if let Some(n) = given.iter().find(|n| {
            !store
                .get(**n)
                .is_some_and(|s| s.user.eq_ignore_ascii_case(&user))
        }) {
            return render_code(format!("No screen {n} for {user}"), "E-CMD-007");
        }
        given
    } else if let Some(vs) = state.current_live_screen {
        vec![vs]
    } else {
        store.for_user(&user).map(|s| s.no).collect()
    };
    if screens.is_empty() {
        return render_code(format!("No UI open for {user}"), "E-CMD-010");
    }
    store.reload_ui(Some(&screens));
    let list: Vec<String> = screens.iter().map(u32::to_string).collect();
    render_success(&format!("Reloading UI on screen {}", list.join(",")))
}

/// `List Window`: the windows of the screen the command came from.
pub fn list_current_windows(state: &AppState) -> String {
    let user = state.effective_user_name().to_string();
    match target_screens(state, &user, &[]) {
        Ok(screens) => reply(list_windows(state, screens[0])),
        Err(e) => render_code(&e, "E-CMD-007"),
    }
}

fn list_windows(state: &AppState, vs: u32) -> Result<String, String> {
    let store = state
        .root
        .runtime
        .live_screens
        .read()
        .map_err(|e| e.to_string())?;
    let session = store
        .get(vs)
        .ok_or_else(|| format!("No screen {vs}"))?;
    // Pane order, as on screen.
    let mut panes = Vec::new();
    fn walk(n: &LayoutNode, out: &mut Vec<Vec<u32>>) {
        match n {
            LayoutNode::Split { children, .. } => children.iter().for_each(|c| walk(c, out)),
            LayoutNode::Leaf { windows, .. } => out.push(windows.clone()),
        }
    }
    walk(&session.root, &mut panes);
    let mut out = format!("Screen {vs} \"{}\":\n", session.meta.name().unwrap_or_default());
    for (i, pane) in panes.iter().enumerate() {
        for w in pane {
            let Some(win) = session.windows.get(w) else {
                continue;
            };
            let ty = type_label(state, &win.kind);
            let focus = if state.current_window == Some(*w) {
                "  ◀"
            } else {
                ""
            };
            let label = win
                .label
                .as_ref()
                .map(|l| format!(" \"{l}\""))
                .unwrap_or_default();
            out.push_str(&format!(
                "  pane {:<2} {:>3}  {ty}{label}{focus}\n",
                i + 1,
                w
            ));
        }
    }
    Ok(out)
}

// ── Object: Window (see terminal/objects) ───────────────────────────────────

use crate::objects::{ObjectDef, Verb, filter, out};
use crate::DispatchResult;

fn window_call(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_window(state, args))
}
fn window_store(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_store_window(state, args))
}
fn window_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_delete_window(state, args))
}
fn window_label(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_label_window(state, args))
}
// ── A window's properties (decisions §33 58): its type's settings + Maximized ──

/// A row key `screen.window` → (live screen, window), if it is one of yours.
fn window_of(state: &AppState, key: &str) -> Option<(u32, u32)> {
    let p = parse(&[key]).ok()?;
    let vs = screen(state, &p).ok()?;
    let w = one(&p, "").ok()?;
    window_info(state, vs, w).map(|_| (vs, w))
}

fn window_get(state: &AppState, key: &str, name: &str) -> String {
    let Some((vs, w)) = window_of(state, key) else { return String::new() };
    let Ok(store) = state.root.runtime.live_screens.read() else { return String::new() };
    let Some(screen) = store.get(vs) else { return String::new() };
    if name.eq_ignore_ascii_case(MAXIMIZED) {
        return if screen.maximized == Some(w) { "On" } else { "Off" }.to_string();
    }
    let Some(win) = screen.windows.get(&w) else { return String::new() };
    let Some(ty) = crate::vfs_runtime::window_type(&state.root, &win.kind) else { return String::new() };
    let key = ty.setting(name).map_or(name, |d| d.key.as_str());
    match ty.resolve_settings(&win.settings).get(key) {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Bool(b)) => if *b { "On" } else { "Off" }.to_string(),
        Some(serde_json::Value::Null) | None => String::new(),
        Some(v) => v.to_string(),
    }
}

/// One setting of a window: its window type checks the value (Toggle, Now, relative, links);
/// `Default` (what Reset writes) puts the type's default back.
fn window_write(state: &mut AppState, key: &str, name: &str, value: &str) -> Result<String, String> {
    let (vs, w) = window_of(state, key).ok_or_else(|| render_code(format!("No window {key}"), "E-CMD-007"))?;
    let text = if name.eq_ignore_ascii_case(MAXIMIZED) {
        set_maximized(state, vs, w, value)
    } else {
        crate::vfs_runtime::apply_window_setting(state, vs, w, name, value.to_string())
    };
    crate::props::from_reply(text)
}

static WINDOW_FIXED: [crate::props::Property; 1] = [crate::props::Property {
    name: MAXIMIZED,
    kind: crate::props::PropKind::Switch,
    rights: crate::commands::registry::Rights::Guest,
    home: crate::props::Home::Station,
    section: "Window",
    doc: "every client of the screen shows the window full screen",
    default: Some("Off"),
    access: crate::props::Access::Named { get: window_get, set: window_write },
    ..crate::props::Property::BASE
}];

/// A window setting's key as lines name it: `auto_scroll` → `AutoScroll`.
fn setting_word(key: &str) -> String {
    key.split('_')
        .map(|p| {
            let mut c = p.chars();
            c.next().map_or_else(String::new, |f| f.to_ascii_uppercase().to_string() + c.as_str())
        })
        .collect()
}

/// Every window type's settings, once (the names, kinds and docs; a row's own type checks it).
fn window_setting_props() -> &'static [crate::props::Property] {
    static ALL: std::sync::OnceLock<Vec<crate::props::Property>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        let leak = |s: &str| -> &'static str { Box::leak(s.to_string().into_boxed_str()) };
        let mut seen = std::collections::BTreeSet::new();
        let mut out = Vec::new();
        for t in lumacore_registry::get().window_types().values() {
            for d in &t.settings {
                if !seen.insert(d.key.to_lowercase()) {
                    continue;
                }
                let camel = setting_word(&d.key);
                // The words its type takes (a choice's values and its labels — the bar sends
                // the label, `Set Self "Level" Errors` —, a number's names): Lum's vocabulary
                // knows them; the type checks them.
                use lumacore_tree::window_types::SettingKind;
                let words: Vec<String> = match &d.kind {
                    SettingKind::Enum { options } => options
                        .iter()
                        .flat_map(|(v, label)| {
                            let value = v.as_str().map(str::to_string);
                            let label = (!value.as_deref().is_some_and(|v| v.eq_ignore_ascii_case(label))).then(|| label.clone());
                            value.into_iter().chain(label)
                        })
                        .collect(),
                    SettingKind::Int { named, .. } => named.iter().map(|(_, n)| n.clone()).collect(),
                    SettingKind::Link { follow: Some(_), .. } => vec!["Selected".to_string()],
                    _ => Vec::new(),
                };
                // `Since Now`: a setting that takes the current moment.
                let words: Vec<String> = words.into_iter().chain(d.now.is_some().then(|| "Now".to_string())).collect();
                let words: Vec<&'static str> = words
                    .iter()
                    .filter(|w| w.chars().all(|c| c.is_ascii_alphanumeric()) && w.starts_with(|c: char| c.is_ascii_alphabetic()))
                    .map(|w| leak(w))
                    .collect();
                out.push(crate::props::Property {
                    name: leak(&camel),
                    named: Box::leak(words.into_boxed_slice()),
                    kind: crate::props::PropKind::Declared,
                    rights: crate::commands::registry::Rights::Guest,
                    home: crate::props::Home::Station,
                    section: leak(if d.section.is_empty() { "Window" } else { &d.section }),
                    doc: leak(d.hint.as_deref().unwrap_or(&d.label)),
                    default: Some("Default"),
                    access: crate::props::Access::Named { get: window_get, set: window_write },
                    ..crate::props::Property::BASE
                });
            }
        }
        out
    })
}

/// Your windows on your live screens, `screen.window` (`Self` is written out to one).
static WINDOW_SETTINGS: crate::props::Props = crate::props::Props {
    lists: &[&WINDOW_FIXED],
    dynamic: Some(window_setting_props),
    keyed: Some(crate::props::Keyed {
        rows: |s| {
            let user = s.effective_user_name();
            let Ok(store) = s.root.runtime.live_screens.read() else { return Vec::new() };
            store.all().filter(|sc| sc.user.eq_ignore_ascii_case(user)).flat_map(|sc| sc.windows.keys().map(move |w| format!("{}.{w}", sc.no))).collect()
        },
        row: |s, w| window_of(s, w.trim_matches('"')).map(|(vs, w)| format!("{vs}.{w}")),
    }),
    label: Some(|_, k| format!("Window {k}")),
    ..crate::props::Props::BASE
};
fn window_reset(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_reset_window(state, args))
}
fn window_list(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    out(list_current_windows(state))
}
/// Window numbers of the screen the command line belongs to.
fn window_complete(state: &AppState, prefix: &str) -> Vec<String> {
    let user = state.effective_user_name().to_string();
    let Ok(screens) = target_screens(state, &user, &[]) else {
        return Vec::new();
    };
    let Ok(store) = state.root.runtime.live_screens.read() else {
        return Vec::new();
    };
    let nos = store
        .get(screens[0])
        .map(|s| s.windows.keys().map(u32::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    filter(nos, prefix)
}

pub static WINDOW: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Window",
        lum: crate::objects::LumShape::ids(2),
        default_verb: crate::objects::Verb::Select,
        flags: &[PANE.on(&[Verb::Store]), F::SCREEN.on(&[Verb::Store]), F::OVERWRITE.on(&[Verb::Store]), F::CLEAR.on(&[Verb::Store])],
        aliases: &["windows"],
        doc: crate::commands::registry::Doc {
            display: "Window",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Window [[screen.]n] | Store Window [n] \"Type\" [/Pane n] [/Overwrite] | Set Window [screen.]n <setting> <value>|+ n|- n|* n|Default|Toggle|Now | Set Window [screen.]n Maximized On|Off|Toggle | Reset Window [screen.]n",
            description: "Windows on a screen (screen), addressed as [screen.]n. \
                          Window <n> focuses a window; bare Window lists this screen's windows. \
                          Store Window adds a window as a tab beside the focused one (or /Pane n); \
                          /Overwrite changes an existing window's type in place. \
                          Types are listed in Root.Resources.WindowTypes.",
            examples: &[
                "Store Window \"Groups\"                -- add a Groups window beside the focused one",
                "Store Window 5 \"Terminal\" /Overwrite -- change window 5 into a Terminal",
                "Label Window 5 \"Front\"               -- tab title",
                "Set Window 1.5 \"Columns\" 8           -- a window setting",
                "Set Window 1.4 \"Universe\" + 1        -- relative; Toggle too",
                "Reset Window 1.4 \"Universe\"          -- back to the type's default",
                "Set Self \"Maximized\" On              -- hold this window full screen on the screen",
                "Reset Window 1.4                     -- settings and bar back to the type's",
                "Delete Window 2.*                    -- clear screen 2",
            ],
            related: &["view", "store", "set"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_station,
        reach: crate::objects::show_reach,
        rights: crate::rights::screen,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Select, window_call),
        (Verb::Store, window_store),
        (Verb::Label, window_label),
        (Verb::Delete, window_delete),
        (Verb::Reset, window_reset),
        (Verb::List, window_list),
    ],
    delete_ask: None,
    complete: Some(window_complete),
    props: Some(&WINDOW_SETTINGS),
    tree: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_addresses_and_options() {
        let p = parse(&["2.7", "\"Groups\"", "/pane", "3", "/o"]).unwrap();
        assert_eq!(
            (p.screen, p.which, p.pane, p.overwrite),
            (Some(2), Some(Which::One(7)), Some(3), true)
        );
        assert_eq!(p.rest, vec!["\"Groups\""]);

        let p = parse(&["\"Groups\""]).unwrap();
        assert_eq!(p.which, None);
        assert_eq!(parse(&["2.*"]).unwrap().which, Some(Which::All));
        assert!(parse(&["5", "/pane"]).is_err());
        // An undeclared flag never gets here: dispatch refuses it (E-CMD-018, decisions §31).
    }
}
