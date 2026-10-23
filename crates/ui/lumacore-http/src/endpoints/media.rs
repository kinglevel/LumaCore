//! Pictures the show embeds (decisions §30 "12a2").
//!
//! - `GET /api/media/<sha256>` — a picture from the station's store, by hash
//!   (immutable: the hash is the content). Open like the other reads (the
//!   lock screen shows a picture before anyone logs in).
//! - `POST /api/media?name=<file>` — upload a file into `library/media/`
//!   (≤ 16 MB); the client then runs the same `Import … "<name>"` line as
//!   anyone typing it.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use crate::HttpState;
use crate::auth::AuthUser;

/// The largest upload (an embedded file's limit).
pub const MAX_BYTES: usize = 16 * 1024 * 1024;

pub async fn get_media(State(state): State<HttpState>, Path(sha): Path<String>) -> Response {
    let console = state.console.clone();
    let found = tokio::task::spawn_blocking(move || console.media(&sha))
        .await
        .ok()
        .flatten();
    match found {
        Some((bytes, mime)) => (
            [
                (header::CONTENT_TYPE, mime),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable".to_string()),
            ],
            bytes,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct Upload {
    name: String,
}

pub async fn upload(
    State(state): State<HttpState>,
    AuthUser(user): AuthUser,
    Query(upload): Query<Upload>,
    body: Bytes,
) -> (StatusCode, Json<Value>) {
    let console = state.console.clone();
    let saved = tokio::task::spawn_blocking(move || console.save_media(&user, &upload.name, body.to_vec()))
        .await
        .unwrap_or_else(|e| Err(e.to_string()));
    match saved {
        Ok(name) => (StatusCode::OK, Json(json!({ "ok": true, "name": name }))),
        Err(message) => (crate::status_of(&message), Json(json!({ "ok": false, "message": message }))),
    }
}
