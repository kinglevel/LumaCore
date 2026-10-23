//! GET /api/info — Build info, uptime, show stats.

use axum::Json;
use axum::extract::State;
use serde_json::json;

use crate::HttpState;

pub async fn get_info(State(state): State<HttpState>) -> Json<serde_json::Value> {
    let snap = state.diagnostics.load();
    Json(json!({
        "build": {
            "version": snap.version,
            "profile": snap.profile,
            "target_os": snap.target_os,
            "target_arch": snap.target_arch,
            "git_commit": snap.git_commit,
            "build_id": snap.build_id,
        },
        "uptime_secs": snap.uptime_secs,
        "show_path": snap.show_path,
        "session_count": snap.session_count,
        "fixture_count": snap.fixture_count,
        "universe_count": snap.universe_count,
        "output_count": snap.output_count,
        "protocols_running": snap.protocols_running,
    }))
}
