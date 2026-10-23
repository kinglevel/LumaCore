//! VFS for logged-in users' runtime state, their screens and windows:
//!
//! ```text
//! Root.Runtime.Users                                   logged-in users (by profile no)
//! Root.Runtime.Users.<user>.Modes                      Blind / Highlight / Freeze / Solo (§16)
//! Root.Runtime.Users.<user>.Screens               that user's screens
//! Root.Runtime.Users.<user>.Screens.<n>.Windows   the screen's windows
//! Root.Runtime.Users.<user>.Screens.<n>.Windows.<no>
//! ```
//!
//! Windows are edited with `Set Window <n> "<Setting>" …` (the one property registry); UI
//! engines pick the change up from the shared store and re-render.

use lumacore_tree::runtime::Screen;
use lumacore_tree::{Root, WindowType};
use serde_json::{Value, json};

use super::vfs::{DynChild, NodeView};
use crate::AppState;
use crate::tree_path::{Node, TreePath};

/// A parsed path below `Root.Runtime.Users`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtPath {
    Users,
    User(u32),
    Modes(u32),
    Undo(u32),
    Screens(u32),
    Screen(u32, u32),
    Windows(u32, u32),
    Window(u32, u32, u32),
}

/// The path below `Root.Runtime.Users` a tree path names.
pub fn from_path(path: &TreePath) -> Option<RtPath> {
    let user = || path.numbered(Node::LoggedIn);
    let screen = || path.numbered(Node::UserScreens);
    let window = || path.numbered(Node::ScreenWindows);
    Some(match path.node() {
        Node::LoggedIn => RtPath::Users,
        Node::LoggedInUser => RtPath::User(user()?),
        Node::UserModes => RtPath::Modes(user()?),
        Node::UserUndo => RtPath::Undo(user()?),
        Node::UserScreens => RtPath::Screens(user()?),
        Node::UserScreen => RtPath::Screen(user()?, screen()?),
        Node::ScreenWindows => RtPath::Windows(user()?, screen()?),
        Node::ScreenWindow => RtPath::Window(user()?, screen()?, window()?),
        _ => return None,
    })
}

/// `Runtime.Users.{n}` is numbered like `ShowData.Users` (decisions §10).
fn user_name(root: &Root, user_no: u32) -> Option<String> {
    root.show_data.users.get(&user_no).map(|u| u.login.clone())
}

pub fn window_type<'a>(root: &'a Root, kind: &str) -> Option<&'a WindowType> {
    root.resources
        .window_types
        .values()
        .find(|t| t.name == kind)
}

fn folder(name: &str, child_count: usize) -> DynChild {
    DynChild {
        display_name: name.to_string(),
        id: 0,
        node_type: "Folder",
        is_leaf: false,
        child_count,
        name: None,
        guid: None,
    }
}

/// Run `f` on the screen if it exists and belongs to `user_no`.
fn with_session<R>(
    root: &Root,
    user_no: u32,
    vs: u32,
    f: impl FnOnce(&Screen) -> R,
) -> Option<R> {
    let user = user_name(root, user_no)?;
    let store = root.runtime.live_screens.read().ok()?;
    store
        .get(vs)
        .filter(|s| s.user.eq_ignore_ascii_case(&user))
        .map(f)
}

pub fn children(path: RtPath, root: &Root) -> Option<Vec<DynChild>> {
    let store = root.runtime.live_screens.read().ok()?;
    Some(match path {
        RtPath::Users => root
            .show_data
            .users
            .iter()
            .filter_map(|(&no, u)| {
                let name = u.login.as_str();
                let count = store.for_user(name).count();
                let logged_in = root.runtime.user_ref(name).is_some();
                (count > 0 || logged_in).then(|| DynChild {
                    display_name: format!("{no} \"{name}\"  ({count} screens)"),
                    id: no,
                    node_type: "RuntimeUser",
                    is_leaf: false,
                    child_count: 3,
                    name: None,
                    guid: None,
                })
            })
            .collect(),
        // Modes, Undo and Screens are its declared children (`crate::tree_path`).
        RtPath::User(u) => {
            user_name(root, u)?;
            Vec::new()
        }
        RtPath::Modes(_) | RtPath::Undo(_) => Vec::new(),
        RtPath::Screens(u) => {
            let name = user_name(root, u)?;
            store
                .for_user(&name)
                .map(|s| DynChild {
                    display_name: format!("{} \"{}\"  ({} windows)", s.no, s.meta.name().unwrap_or_default(), s.windows.len()),
                    id: s.no,
                    node_type: "Screen",
                    is_leaf: false,
                    child_count: 1,
                    name: None,
                    guid: None,
                })
                .collect()
        }
        RtPath::Screen(u, v) => {
            drop(store);
            let count = with_session(root, u, v, |s| s.windows.len())?;
            vec![folder("Windows", count)]
        }
        RtPath::Windows(u, v) => {
            drop(store);
            with_session(root, u, v, |s| {
                s.windows
                    .values()
                    .map(|w| DynChild {
                        display_name: format!(
                            "{} \"{}\"",
                            w.no,
                            w.label.as_deref().unwrap_or_else(|| {
                                window_type(root, &w.kind)
                                    .map_or(w.kind.as_str(), |t| t.label.as_str())
                            })
                        ),
                        id: w.no,
                        node_type: "Window",
                        is_leaf: true,
                        child_count: 0,
                        name: None,
                        guid: None,
                    })
                    .collect()
            })?
        }
        RtPath::Window(..) => Vec::new(),
    })
}

/// Object view for numbered nodes (user, screen, window).
pub fn object(path: RtPath, root: &Root) -> Option<NodeView> {
    let view = |json: Value, node_type: &'static str, id: u32, name: String| NodeView::Object {
        json,
        node_type,
        id,
        guid: String::new(),
        name: Some(name),
    };
    match path {
        RtPath::User(u) => {
            let name = user_name(root, u)?;
            let store = root.runtime.live_screens.read().ok()?;
            let sessions: Vec<u32> = store.for_user(&name).map(|s| s.no).collect();
            let modes = root
                .runtime
                .user_ref(&name)
                .map(|r| r.modes)
                .unwrap_or_default();
            Some(view(
                json!({ "user": name, "modes": modes, "live_screens": sessions }),
                "RuntimeUser",
                u,
                name,
            ))
        }
        RtPath::Modes(u) => {
            let name = user_name(root, u)?;
            let modes = root
                .runtime
                .user_ref(&name)
                .map(|r| r.modes)
                .unwrap_or_default();
            Some(view(
                serde_json::to_value(modes).unwrap_or(Value::Null),
                "Modes",
                u,
                format!("{name} modes"),
            ))
        }
        RtPath::Undo(u) => {
            // The user's undo history (replicated, never saved; decisions §21).
            let name = user_name(root, u)?;
            let undo = root.runtime.user_ref(&name).map(|r| &r.undo);
            let frames: Vec<Value> = undo
                .map(|s| {
                    s.frames
                        .iter()
                        .rev()
                        .map(|f| {
                            json!({
                                "description": f.description,
                                "user": f.user,
                                "subtrees": f.snapshots.iter().map(|s| s.subtree.to_string()).collect::<Vec<_>>(),
                                "bytes": f.size_bytes,
                                "station": f.station,
                                "time_ms": f.time_ms,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(view(
                json!({
                    "frames": frames,
                    "redo": undo.map_or(0, |s| s.redo_frames.len()),
                    "bytes": undo.map_or(0, |s| s.memory_usage()),
                }),
                "Undo",
                u,
                format!("{name} undo"),
            ))
        }
        RtPath::Screen(u, v) => with_session(root, u, v, |s| {
            let json = json!({
                "no": s.no,
                "name": s.meta.name,
                "user": s.user,
                "revision": s.revision,
                "layout": s.root,
                "windows": s.windows.keys().collect::<Vec<_>>(),
            });
            view(json, "Screen", v, s.meta.name().unwrap_or_default().to_string())
        }),
        RtPath::Window(u, v, w) => with_session(root, u, v, |s| {
            let win = s.windows.get(&w)?;
            let mut json = serde_json::to_value(win).unwrap_or(Value::Null);
            let label = match window_type(root, &win.kind) {
                Some(ty) => {
                    json["settings"] = Value::Object(ty.resolve_settings(&win.settings));
                    ty.label.clone()
                }
                None => win.kind.clone(),
            };
            Some(view(json, "Window", w, label))
        })
        .flatten(),
        _ => None,
    }
}

/// Set one typed setting on window `w` of screen `v` (ownership already
/// checked by the caller). Shared by `set` on a tree path and `Set Window`.
///
/// Besides a value (decisions §30 "12c"): `Default` puts the type's default
/// back (`WindowBarItems Default` = the type's bar), `Toggle` flips an
/// on/off (or steps a choice), `+ 1` / `- 1` / `* 2` / `/ 2` are relative on
/// a number or a numbered link (a universe), `Now` marks the current point of
/// a setting that takes it (`Since Now`). A value that isn't one is refused
/// (E-CMD-008 for a relative one, E-SET-003 otherwise) and nothing changes.
pub fn apply_window_setting(
    state: &AppState,
    v: u32,
    w: u32,
    key: &str,
    value: String,
) -> String {
    use crate::errors::render_code;
    use crate::text::render_success;
    use lumacore_tree::window_types::{NowSource, SettingKind};

    let root = &state.root;
    let window = root.runtime.live_screens.read().ok().and_then(|store| {
        store
            .get(v)
            .and_then(|s| s.windows.get(&w))
            .map(|win| (win.kind.clone(), win.settings.clone()))
    });
    let Some((kind, stored)) = window else {
        return render_code(format!("No window {v}.{w}"), "E-CMD-007");
    };
    let Some(ty) = window_type(root, &kind) else {
        return render_code(format!("Unknown window type '{kind}'"), "E-CMD-007");
    };
    let Some(def) = ty.setting(key) else {
        let keys: Vec<&str> = ty.settings.iter().map(|s| s.key.as_str()).collect();
        return render_code(
            format!(
                "{} has no setting '{key}'. Settings: {}",
                ty.label,
                keys.join(", ")
            ),
            "E-SET-001",
        );
    };
    let now = ty.resolve_settings(&stored).get(&def.key).cloned().unwrap_or(Value::Null);
    let raw = value.trim().trim_matches('"').to_string();
    // tidy: not-keywords (the value words a window setting takes)
    let word = raw.to_ascii_lowercase();

    // `Default`: the stored value goes, the type's default shows.
    let coerced: Option<Value> = if word == "default" {
        None
    } else if word == "toggle" {
        Some(match &def.kind {
            SettingKind::Bool => json!(now.as_bool() != Some(true)),
            SettingKind::Enum { options } => {
                let i = options.iter().position(|(o, _)| *o == now).map_or(0, |i| (i + 1) % options.len());
                options[i].0.clone()
            }
            _ => {
                return render_code(
                    format!("{} {}: Toggle takes an on/off or a choice", ty.label, def.key),
                    "E-CMD-008",
                );
            }
        })
    } else if word == "now" && def.now.is_some() {
        Some(match def.now {
            Some(NowSource::WallMs) => json!(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_millis() as u64)
            ),
            None => Value::Null,
        })
    } else {
        // Relative on a number or a numbered link (a universe).
        let numbered = matches!(def.kind, SettingKind::Int { .. })
            || matches!(def.kind, SettingKind::Link { by_guid: false, .. });
        let relative = numbered
            .then(|| now.as_i64())
            .flatten()
            .and_then(|n| crate::props::relative(&raw, n));
        let raw = match relative {
            Some(Ok(n)) if matches!(def.kind, SettingKind::Link { .. }) => n.max(1).to_string(),
            Some(Ok(n)) => n.to_string(),
            Some(Err(e)) => return render_code(format!("{} {}: {e}", ty.label, def.key), "E-CMD-008"),
            None => raw,
        };
        // A link typed by number or name is stored by its GUID (decisions §30 "12b2").
        let raw = match &def.kind {
            SettingKind::Link { kind, by_guid: true, .. } => {
                crate::commands::windows::resolve_window_link(state, kind, &raw)
            }
            _ => raw,
        };
        match ty.coerce_setting(def, &Value::String(raw), true) {
            Ok(v) => Some(v),
            Err(e) => return render_code(&e, "E-SET-003"),
        }
    };

    let result = root
        .runtime
        .live_screens
        .write()
        .map_err(|e| e.to_string())
        .and_then(|mut store| {
            store.update_window(v, w, |win| {
                match &coerced {
                    Some(c) => {
                        win.settings.insert(def.key.clone(), c.clone());
                    }
                    None => {
                        win.settings.remove(&def.key);
                    }
                }
                Ok(())
            })
        });
    match result {
        Ok(_) => {
            let shown = def.label_of(coerced.as_ref().unwrap_or(&def.default));
            let shown = if shown.is_empty() { "\"\"".to_string() } else { shown };
            let default = if coerced.is_none() { " (default)" } else { "" };
            render_success(&format!(
                "Window {v}.{w} ({}): {} = {shown}{default}",
                ty.label, def.key
            ))
        }
        Err(e) => render_code(&e, "E-CMD-007"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(dot_path: &str) -> Option<RtPath> {
        from_path(&TreePath::parse_dot(dot_path)?)
    }

    #[test]
    fn parses_runtime_paths() {
        assert_eq!(parse("Root.Runtime.Users"), Some(RtPath::Users));
        assert_eq!(
            parse("Root.Runtime.Users.1.Screens.2.Windows.3"),
            Some(RtPath::Window(1, 2, 3))
        );
        assert_eq!(parse("Root.Runtime.Users.1.Modes"), Some(RtPath::Modes(1)));
        assert_eq!(parse("Root.Runtime.SessionEngines"), None);
        assert_eq!(parse("Root.Runtime.Users.Admin"), None);
    }
}
