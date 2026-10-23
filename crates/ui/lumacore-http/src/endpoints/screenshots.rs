//! POST /api/screenshots?id=<capture>&screen=<live screen> — a client's
//! capture (decisions §29 "11d"): the PNG it rendered of its screen, as
//! seen, after a `Screenshot` asked for it over `/api/events`. The console
//! checks it was asked for and saves it in the station's library.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::HttpState;
use crate::auth::AuthUser;

#[derive(serde::Deserialize)]
pub struct Upload {
    id: u64,
    screen: u32,
}

/// The largest capture accepted (a 4K screen at 2× is ~30 MB as PNG).
pub const MAX_BYTES: usize = 64 * 1024 * 1024;

pub async fn upload(
    State(state): State<HttpState>,
    AuthUser(user): AuthUser,
    Query(upload): Query<Upload>,
    body: Bytes,
) -> (StatusCode, Json<Value>) {
    if !body.starts_with(b"\x89PNG\r\n\x1a\n") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "message": "not a PNG" })),
        );
    }
    let console = state.console.clone();
    let saved = tokio::task::spawn_blocking(move || {
        console.save_screenshot(&user, upload.id, upload.screen, body.to_vec())
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string()));
    match saved {
        Ok(path) => (StatusCode::OK, Json(json!({ "ok": true, "path": path }))),
        // Busy: try again (503); anything else: the capture isn't wanted (409).
        Err(message) if crate::status_of(&message) == StatusCode::SERVICE_UNAVAILABLE => {
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "ok": false, "message": message })))
        }
        Err(message) => (
            StatusCode::CONFLICT,
            Json(json!({ "ok": false, "message": message })),
        ),
    }
}
