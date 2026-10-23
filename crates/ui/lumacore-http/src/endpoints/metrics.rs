//! GET /api/metrics — thread loop timings and show counts.

use axum::Json;
use axum::extract::State;
use serde_json::json;

use crate::HttpState;

pub async fn get_metrics(State(state): State<HttpState>) -> Json<serde_json::Value> {
    let snap = state.diagnostics.load();
    Json(json!({
        "threads": snap.threads,
        "runtime": {
            "fixtures": snap.fixture_count,
            "universes": snap.universe_count,
            "outputs": snap.output_count,
            "sessions": snap.session_count,
            "protocols_running": snap.protocols_running,
        },
    }))
}
