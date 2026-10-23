//! View-session endpoints — the live windows of one screen.
//!
//! The canonical state lives in `Root.Runtime` (`SharedScreens`); this
//! engine only renders it. Changes from any side (these endpoints, `set` on the
//! command line) are broadcast as `screen_changed` SSE events.
//!
//! Routes:
//!   GET  /api/screens               → all screens (summary)
//!   POST /api/screens               → join `{join}` or create; returns the session
//!   GET  /api/screens/{no}          → layout + windows (settings resolved with defaults)
//!   PUT  /api/screens/{no}/layout   → `{revision, root, windows}`: replace the structure
//!                                     (existing windows keep what they hold); returns
//!                                     `{revision}`, 409 + `{screen}` on a stale revision,
//!                                     423 from a DeskLocked device (`X-Luma-Device`)

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use lumacore_tree::runtime::{LayoutNode, Screen, Window};
use lumacore_tree::WindowType;

use crate::HttpState;
use crate::auth::AuthUser;

/// User for web screens until the HTTP engine knows who is logged in
/// (same default as the /ws/shell bridge).

#[derive(Serialize)]
pub struct ScreenSummary {
    no: u32,
    user: String,
    name: String,
    windows: usize,
    revision: u64,
}

pub async fn list_live_screens(State(state): State<HttpState>) -> Json<Vec<ScreenSummary>> {
    let store = state.live_screens.read().unwrap_or_else(|e| e.into_inner());
    Json(
        store
            .all()
            .map(|s| ScreenSummary {
                no: s.no,
                user: s.user.clone(),
                name: s.meta.name().unwrap_or_default().to_string(),
                windows: s.windows.len(),
                revision: s.revision,
            })
            .collect(),
    )
}

#[derive(Deserialize, Default)]
pub struct JoinRequest {
    /// A specific screen number (`?vs=`).
    join: Option<u32>,
    /// Screen config the client uses (default: the user's default).
    config: Option<u32>,
    /// Screen of that config (default 1).
    screen: Option<u32>,
}

/// Join a screen of the logged-in user: `join` if it is theirs, else
/// their screen `(config, screen)` — live if open, otherwise laid out from the
/// show.
pub async fn join_live_screen(
    State(state): State<HttpState>,
    AuthUser(user): AuthUser,
    body: Option<Json<JoinRequest>>,
) -> Json<Value> {
    let req = body.map(|Json(b)| b).unwrap_or_default();
    if let Some(no) = req.join {
        let store = state.live_screens.read().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = store.get(no).filter(|s| s.user.eq_ignore_ascii_case(&user)) {
            return Json(session_json(s));
        }
    }
    // Saved layout from the show first (console lock), then the store lock.
    let screen = req.screen.unwrap_or(1).max(1);
    let (config, saved) = state.console.screen_layout(&user, req.config, screen);
    let mut store = state.live_screens.write().unwrap_or_else(|e| e.into_inner());
    let no = store.open_screen(&user, config, screen, saved);
    Json(session_json(store.get(no).expect("just opened")))
}

pub async fn get_live_screen(
    State(state): State<HttpState>,
    Path(no): Path<u32>,
) -> Result<Json<Value>, StatusCode> {
    let store = state.live_screens.read().unwrap_or_else(|e| e.into_inner());
    store
        .get(no)
        .map(|s| Json(session_json(s)))
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
pub struct LayoutUpdate {
    /// The screen's revision the page last applied or saved: a PUT on an
    /// older one is refused (409 with the current screen), so a layout save
    /// never undoes a `Set Window` / `Label Self` that ran meanwhile.
    revision: u64,
    root: LayoutNode,
    windows: BTreeMap<u32, Window>,
}

type Refusal = (StatusCode, Json<Value>);

fn refusal(status: StatusCode, message: impl Into<String>) -> Refusal {
    (status, Json(json!({ "ok": false, "message": message.into() })))
}

/// Replace a layout — only the logged-in user's own screens, never from a
/// DeskLocked device (E-LOCK-001).
///
/// The page owns only the **structure**: the split tree, which windows exist
/// (numbers and kinds) and new windows. A window that already exists with the
/// same number and kind keeps what the station holds — settings, label,
/// colour, scroll and preset; those change only through commands (`Set
/// Window`, `Label Self`, `WindowPreset`). A new window's settings are kept
/// only where they differ from its type's defaults, so a later default still
/// reaches it (decisions §30 "12c").
pub async fn put_live_screen_layout(
    State(state): State<HttpState>,
    AuthUser(user): AuthUser,
    Path(no): Path<u32>,
    headers: HeaderMap,
    uri: Uri,
    Json(update): Json<LayoutUpdate>,
) -> Result<Json<Value>, Refusal> {
    if let Some(device) = crate::auth::device_of(&headers, &uri) {
        let console = state.console.clone();
        // The lock can't be checked (busy, a failed job): refused, never let
        // through unchecked (fails closed).
        let refused = tokio::task::spawn_blocking(move || console.device_lock_refusal(&device))
            .await
            .unwrap_or_else(|e| Err(e.to_string()));
        match refused {
            Ok(None) => {}
            Ok(Some(message)) => return Err(refusal(StatusCode::LOCKED, message)),
            Err(message) => return Err(refusal(StatusCode::SERVICE_UNAVAILABLE, message)),
        }
    }
    let types = types_by_name();
    let mut store = state.live_screens.write().unwrap_or_else(|e| e.into_inner());
    let current = match store.get(no) {
        None => return Err(refusal(StatusCode::NOT_FOUND, format!("no screen {no}"))),
        Some(s) if !s.user.eq_ignore_ascii_case(&user) => {
            return Err(refusal(
                StatusCode::FORBIDDEN,
                format!("screen {no} belongs to another user"),
            ));
        }
        Some(s) => s,
    };
    if update.revision != current.revision {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({
                "ok": false,
                "message": format!(
                    "Screen {no} changed meanwhile (revision {} → {}): its layout wasn't saved.",
                    update.revision, current.revision
                ),
                "screen": session_json(current),
            })),
        ));
    }
    let held = current.windows.clone();
    let mut windows = update.windows;
    for (n, w) in windows.iter_mut() {
        let ty = types.get(w.kind.as_str()).ok_or_else(|| {
            refusal(
                StatusCode::BAD_REQUEST,
                format!("unknown window type '{}'", w.kind),
            )
        })?;
        match held.get(n).filter(|old| old.kind == w.kind) {
            Some(old) => *w = old.clone(),
            None => w.settings = only_changed(ty, &w.settings),
        }
    }
    let revision = store
        .replace_layout(no, update.root, windows)
        .map_err(|e| refusal(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "ok": true, "revision": revision })))
}

fn types_by_name() -> BTreeMap<String, WindowType> {
    lumacore_registry::get().window_types().values().cloned()
        .map(|t| (t.name.clone(), t))
        .collect()
}

/// Keep only settings the type defines, coerced to their type, and only
/// where they differ from the type's default (a default isn't stored, so a
/// later build's default reaches the window). Invalid values are dropped
/// (the default applies).
fn only_changed(
    ty: &WindowType,
    settings: &serde_json::Map<String, Value>,
) -> serde_json::Map<String, Value> {
    settings
        .iter()
        .filter_map(|(k, v)| {
            let def = ty.setting(k)?;
            let v = ty.coerce_setting(def, v, false).ok()?;
            let default = ty
                .coerce_setting(def, &def.default, false)
                .unwrap_or_else(|_| def.default.clone());
            (v != default).then(|| (def.key.clone(), v))
        })
        .collect()
}

/// A screen for clients: window settings resolved with type defaults.
fn session_json(session: &Screen) -> Value {
    let types = types_by_name();
    let mut value = serde_json::to_value(session).unwrap_or(Value::Null);
    if let Some(windows) = value.get_mut("windows").and_then(Value::as_object_mut) {
        for w in windows.values_mut() {
            let kind = w["kind"].as_str().unwrap_or_default().to_string();
            if let (Some(ty), Some(stored)) = (types.get(&kind), w["settings"].as_object()) {
                w["settings"] = Value::Object(ty.resolve_settings(stored));
            }
        }
    }
    value
}
