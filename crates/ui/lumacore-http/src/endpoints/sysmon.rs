//! GET /api/sysmon — the System Monitor history (structured lines).

use axum::Json;
use axum::extract::State;
use serde_json::json;

use crate::HttpState;
use crate::auth::AuthUser;

pub async fn get_sysmon(
    State(state): State<HttpState>,
    AuthUser(_user): AuthUser,
) -> Json<serde_json::Value> {
    let lines: Vec<serde_json::Value> = state
        .console
        .sysmon_history()
        .iter()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    Json(json!({ "count": lines.len(), "lines": lines }))
}
