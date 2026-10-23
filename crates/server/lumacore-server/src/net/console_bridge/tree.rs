// ████████╗██████╗ ███████╗███████╗
// ╚══██╔══╝██╔══██╗██╔════╝██╔════╝
//    ██║   ██████╔╝█████╗  █████╗
//    ██║   ██╔══██╗██╔══╝  ██╔══╝
//    ██║   ██║  ██║███████╗███████╗
//    ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝
// FILE: tree

//! The Tree window's two feeds on this server (decisions 0292): its rows and its inspector are
//! built on the state thread, as the subscribing user, because the tree reads more than the
//! published show — the runtime, the station's settings, the drives — through the one row builder
//! `ls` and NAV read. A machine-lane job like any HTTP read: busy answers nothing, and the build
//! comes again when the show moves.
//!
//! Owns:          the tree and inspector jobs, the acting user around them.
//! Doesn't own:   what they show (lumacore_views::tree_window), the feeds (lumacore-http).
//! Entry points:  tree, inspector.
//! Decisions:     docs/decisions/0292-tree-window-nav-in-the-web-and-app.

use serde_json::{Value, json};

use crate::state::StateHandle;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The `tree` feed's data for `user` (the window's `path`, `open`, `search`, `follow`).
pub(super) fn tree(state: &StateHandle, user: &str, params: &Value) -> Value {
    let text = |k: &str| params[k].as_str().unwrap_or("").to_string();
    let p = lumacore_views::tree_window::TreeParams {
        path: text("path"),
        open: text("open").split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect(),
        search: text("search"),
        follow: params["follow"].as_bool().unwrap_or(false),
    };
    as_user(state, user, move |s| lumacore_views::tree_window::tree(s, &p)).unwrap_or_else(|| json!({ "rows": [], "path": null, "busy": true }))
}

/// The `inspector` feed's data for `user`.
pub(super) fn inspector(state: &StateHandle, user: &str, path: &str) -> Value {
    let path = path.to_string();
    as_user(state, user, move |s| lumacore_views::tree_window::inspector(s, &path)).unwrap_or_else(|| json!({ "busy": true }))
}

/// Run `build` on the state thread as `user` (their datapool, their bookmarks), the acting user
/// put back after; None when the station is busy or the user is gone.
fn as_user<F>(state: &StateHandle, user: &str, build: F) -> Option<Value>
where
    F: FnOnce(&mut lumacore_objects::AppState) -> Value + Send + 'static,
{
    let user = user.to_string();
    state
        .call_within(crate::state::HTTP_WAIT, move |ctx| {
            let before = ctx.state.acting_user.clone();
            let out = ctx.state.begin_acting_as_existing_user(&user).map(|_| build(&mut ctx.state));
            ctx.state.acting_user = before;
            out
        })
        .ok()
        .flatten()
}
