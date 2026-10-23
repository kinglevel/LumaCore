// ███████╗██╗██╗     ███████╗███████╗
// ██╔════╝██║██║     ██╔════╝██╔════╝
// █████╗  ██║██║     █████╗  ███████╗
// ██╔══╝  ██║██║     ██╔══╝  ╚════██║
// ██║     ██║███████╗███████╗███████║
// ╚═╝     ╚═╝╚══════╝╚══════╝╚══════╝
// FILE: files

//! This device's way onto a drive (decisions §38 5): `POST /api/upload?drive=usb1&folder=mvr&name=x.mvr`
//! puts the body into that drive's library folder (Setup and up; the first write makes the
//! library tree; ≤ 512 MB). The rows a dialog shows come from the `files` feed on `/ws/feeds`.
//!
//! Owns:          the upload handler.
//! Doesn't own:   where the file goes and who may (`ConsoleBridge::save_upload`).
//! Entry points:  upload, MAX_BYTES.
//! Decisions:     §38 5.

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::StatusCode;
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

/// An .mvr with its GDTFs is often 20–300 MB.
pub const MAX_BYTES: usize = 512 * 1024 * 1024;

#[derive(serde::Deserialize)]
pub struct Upload {
    #[serde(default = "local")]
    pub drive: String,
    #[serde(default)]
    pub folder: String,
    pub name: String,
}

fn local() -> String {
    "local-library".to_string()
}

// ██╗ ██████╗
// ██║██╔═══██╗
// ██║██║   ██║
// ██║██║   ██║
// ██║╚██████╔╝
// ╚═╝ ╚═════╝
// PART: IO

pub async fn upload(State(state): State<HttpState>, AuthUser(user): AuthUser, Query(up): Query<Upload>, body: Bytes) -> (StatusCode, Json<Value>) {
    let console = state.console.clone();
    let saved = tokio::task::spawn_blocking(move || console.save_upload(&user, &up.drive, &up.folder, &up.name, body.to_vec()))
        .await
        .unwrap_or_else(|e| Err(e.to_string()));
    match saved {
        Ok(name) => (StatusCode::OK, Json(json!({ "ok": true, "name": name }))),
        Err(message) => (crate::status_of(&message), Json(json!({ "ok": false, "message": message }))),
    }
}
