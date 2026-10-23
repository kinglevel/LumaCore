//! Show library for the Backup menu.
//!
//! Routes:
//!   GET /api/shows → the `shows` feed's build (current, facts, shows,
//!                    history, recovery, usage, disk)
//!   GET /api/show-parts?show=x[&version=|&recovery=] → the Load show dialog
//!
//! Read-only: Save / Load / New / Delete run as commands (`SaveShow`, …) over
//! the shell WebSocket so they echo in the command line.

use axum::Json;
use axum::extract::State;
use serde_json::Value;

use crate::HttpState;

pub async fn list_shows(State(state): State<HttpState>) -> Json<Value> {
    Json(state.console.shows())
}

/// `GET /api/show-parts?show=x[&version=<stamp>|&recovery=<copy>][&drive=<id>]`: the
/// Load show dialog's facts and parts (a one-shot load, from the file's
/// manifest; decisions §30 "12d part 5").
pub async fn show_parts(
    State(state): State<HttpState>,
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let show = q.get("show").cloned().unwrap_or_default();
    let (v, r, d) = (q.get("version").cloned(), q.get("recovery").cloned(), q.get("drive").cloned());
    let console = state.console.clone();
    let out = tokio::task::spawn_blocking(move || console.show_parts(&show, v.as_deref(), r.as_deref(), d.as_deref()))
        .await
        .unwrap_or(Value::Null);
    Json(out)
}
