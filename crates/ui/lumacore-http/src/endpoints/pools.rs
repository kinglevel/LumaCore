//! `GET /api/pools/{pool}?datapool=n` — a pool window's content (decisions
//! §22 "Unified pool window"). Read-only: every gesture on a pool button is
//! a command line sent over the window's shell (`/ws/shell`).

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::HttpState;
use crate::auth::AuthUser;

#[derive(Deserialize)]
pub struct PoolQuery {
    /// 0 / absent = the user's selected datapool.
    pub datapool: Option<u32>,
}

pub async fn pool(
    AuthUser(user): AuthUser,
    Path(pool): Path<String>,
    Query(q): Query<PoolQuery>,
    State(state): State<HttpState>,
) -> impl IntoResponse {
    match state.console.pool(&user, &pool, q.datapool, &serde_json::Value::Null, &serde_json::Value::Null) {
        Some(v) => Json(v).into_response(),
        None => (StatusCode::NOT_FOUND, format!("no pool '{pool}'")).into_response(),
    }
}

/// `GET /api/pools/{pool}/{no}` — one object's editor content.
pub async fn pool_object(
    AuthUser(user): AuthUser,
    Path((pool, no)): Path<(String, u32)>,
    Query(q): Query<PoolQuery>,
    State(state): State<HttpState>,
) -> impl IntoResponse {
    match state.console.pool_object(&user, &pool, q.datapool, no) {
        Some(v) => Json(v).into_response(),
        None => (StatusCode::NOT_FOUND, format!("no {pool} {no}")).into_response(),
    }
}
