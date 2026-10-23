//! GET /api/health — every long-lived thread's health (`Root.Runtime.Health`).

use axum::Json;
use axum::extract::State;
use serde_json::json;

use crate::HttpState;

pub async fn get_health(State(state): State<HttpState>) -> Json<serde_json::Value> {
    let snap = state.diagnostics.load();
    let state = if snap.threads.iter().all(|t| t.alive && t.panics == 0) {
        "ok"
    } else {
        "degraded"
    };
    Json(json!({ "state": state, "threads": snap.threads }))
}
