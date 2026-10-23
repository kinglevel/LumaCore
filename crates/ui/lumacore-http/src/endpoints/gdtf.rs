//  ██████╗ ██████╗ ████████╗███████╗
// ██╔════╝ ██╔══██╗╚══██╔══╝██╔════╝
// ██║  ███╗██║  ██║   ██║   █████╗
// ██║   ██║██║  ██║   ██║   ██╔══╝
// ╚██████╔╝██████╔╝   ██║   ██║
//  ╚═════╝ ╚═════╝    ╚═╝   ╚═╝
// FILE: gdtf

//! The Insert fixtures dialog's routes (decisions §34 round 3 2–4, `insert-fixtures.html` v6).
//!
//! - `GET /api/fixture-types` — the show's fixture types (your draft's when you hold it) and the
//!   files of `library/gdtf/`, with their modes and footprints.
//! - `GET /api/fixture-type?no=3&revision=0&mode=1` or `?file=Robe/x.gdtf&mode=1` — one mode's
//!   channel map and the file's details (404 when there is no such mode).
//! - `GET /api/fixture-type/thumbnail?no=3` or `?file=…` — the file's thumbnail (PNG or SVG), 404
//!   when it has none.
//! - `POST /api/gdtf?name=x.gdtf` — Upload from this device: the file into `library/gdtf/` (Setup
//!   and up, a file the importer reads, ≤ 16 MB); the dialog then picks it like any library file.
//!
//! Owns:          the three handlers.
//! Doesn't own:   the data (`ConsoleBridge::fixture_types` / `fixture_type` / `save_gdtf`).
//! Entry points:  fixture_types, fixture_type, thumbnail, upload.
//! Decisions:     §34 round 3 2–4.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use crate::HttpState;
use crate::auth::AuthUser;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Which mode: a show type by number or a library file, and the revision (0 = the newest) and
/// mode, from 1.
#[derive(serde::Deserialize)]
pub struct Which {
    pub no: Option<u32>,
    pub file: Option<String>,
    /// The drive a `file` is on (decisions §38); absent = the local library.
    pub drive: Option<String>,
    #[serde(default)]
    pub revision: usize,
    #[serde(default)]
    pub mode: usize,
}

#[derive(serde::Deserialize)]
pub struct Upload {
    name: String,
}

// ██╗ ██████╗
// ██║██╔═══██╗
// ██║██║   ██║
// ██║██║   ██║
// ██║╚██████╔╝
// ╚═╝ ╚═════╝
// PART: IO

/// `GET /api/fixture-types[?drive=usb1]`: the show's types and the library's files — a drive's
/// gdtf folder with `drive` (decisions §38 5: Insert fixtures' Library on the files element).
pub async fn fixture_types(State(state): State<HttpState>, AuthUser(user): AuthUser, Query(q): Query<std::collections::HashMap<String, String>>) -> Json<Value> {
    let console = state.console.clone();
    let drive = q.get("drive").cloned();
    Json(tokio::task::spawn_blocking(move || console.fixture_types(&user, drive.as_deref())).await.unwrap_or_default())
}

pub async fn fixture_type(State(state): State<HttpState>, AuthUser(user): AuthUser, Query(which): Query<Which>) -> (StatusCode, Json<Value>) {
    let console = state.console.clone();
    match tokio::task::spawn_blocking(move || console.fixture_type(&user, &which)).await.ok().flatten() {
        Some(v) => (StatusCode::OK, Json(v)),
        None => (StatusCode::NOT_FOUND, Json(json!({ "ok": false, "message": "No such fixture type or mode" }))),
    }
}

pub async fn thumbnail(State(state): State<HttpState>, AuthUser(user): AuthUser, Query(which): Query<Which>) -> Response {
    let console = state.console.clone();
    match tokio::task::spawn_blocking(move || console.fixture_thumbnail(&user, &which)).await.ok().flatten() {
        Some((bytes, mime)) => ([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "private, max-age=3600".to_string())], bytes).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn upload(State(state): State<HttpState>, AuthUser(user): AuthUser, Query(upload): Query<Upload>, body: Bytes) -> (StatusCode, Json<Value>) {
    let console = state.console.clone();
    let saved = tokio::task::spawn_blocking(move || console.save_gdtf(&user, &upload.name, body.to_vec())).await.unwrap_or_else(|e| Err(e.to_string()));
    match saved {
        Ok(name) => (StatusCode::OK, Json(json!({ "ok": true, "name": name }))),
        Err(message) => (crate::status_of(&message), Json(json!({ "ok": false, "message": message }))),
    }
}
