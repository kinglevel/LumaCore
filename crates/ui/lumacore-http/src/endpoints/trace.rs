//! GET /api/trace — Chrome Trace Event JSON for Perfetto.

use axum::Json;
use axum::extract::Query;
use axum::http::header;
use axum::response::IntoResponse;
use serde::Deserialize;

use lumacore_trace::collector::TRACE_COLLECTOR;

#[derive(Deserialize)]
pub struct TraceQuery {
    /// Only include events from the last N seconds.
    pub seconds: Option<u64>,
}

pub async fn get_trace(Query(query): Query<TraceQuery>) -> Json<serde_json::Value> {
    let events = TRACE_COLLECTOR.snapshot(query.seconds);
    let trace_events: Vec<serde_json::Value> = events
        .iter()
        .map(lumacore_trace::format::event_to_json)
        .collect();
    Json(serde_json::json!({
        "traceEvents": trace_events,
        "displayTimeUnit": "us",
    }))
}

/// GET /api/trace/download — downloadable .json file for Perfetto UI "Open trace file".
pub async fn download_trace(Query(query): Query<TraceQuery>) -> impl IntoResponse {
    let events = TRACE_COLLECTOR.snapshot(query.seconds);
    let trace_events: Vec<serde_json::Value> = events
        .iter()
        .map(lumacore_trace::format::event_to_json)
        .collect();
    let body = serde_json::to_string(&serde_json::json!({
        "traceEvents": trace_events,
        "displayTimeUnit": "us",
    }))
    .unwrap_or_default();

    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"lumacore-trace.json\"",
            ),
        ],
        body,
    )
}
