//! Station info and login (thin clients, browsers).
//!
//! Routes:
//!   GET  /api/station → `{name, id, version, build, http_port, shell_port, show,
//!                         users: [{name, has_password, remote_login}]}` (public;
//!                         live as the open `station` feed on /ws/feeds)
//!   POST /api/login   → `{user, password?}` → `{ok, user, token}` + `lumacore_token` cookie; the
//!                       token opens nothing until a shell with it consents to the Meow License
//!                       (decisions 0290)
//!   POST /api/logout  → ends the token (refused from then on) and clears the cookie
//!   GET  /api/me      → `{user, screen_configs: {default, configs: [{no, name, screens}]}}`
//!                       for the current token (401 when not logged in; `{consent: true, user}`
//!                       while its login waits for the Meow License)

use std::net::SocketAddr;

use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::HttpState;
use crate::auth::{AuthUser, COOKIE};

pub async fn get_station(State(state): State<HttpState>) -> Json<Value> {
    Json(state.console.station())
}

#[derive(Deserialize)]
pub struct LoginBody {
    user: String,
    #[serde(default)]
    password: Option<String>,
}

fn cookie(value: &str, max_age: u64) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "{COOKIE}={value}; Path=/; SameSite=Lax; Max-Age={max_age}"
    ))
    .unwrap_or_else(|_| HeaderValue::from_static(""))
}

pub async fn login(
    State(state): State<HttpState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(body): Json<LoginBody>,
) -> Response {
    let remote = !peer.ip().is_loopback();
    // The bridge waits for the state thread: a blocking thread, never a
    // tokio worker (R1-67).
    let console = state.console.clone();
    let from = peer.ip().to_string();
    let result = tokio::task::spawn_blocking(move || {
        console.login(&body.user, body.password.as_deref(), remote, &from)
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string()));
    match result {
        Ok((user, token)) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::SET_COOKIE, cookie(&token, 30 * 86_400));
            (
                headers,
                Json(json!({ "ok": true, "user": user, "token": token })),
            )
                .into_response()
        }
        Err(message) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "message": message })),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct UnlockBody {
    device: String,
    #[serde(default)]
    password: Option<String>,
}

/// POST /api/unlock `{device, password}` — the lock surface unlocks its
/// device with the logged-in user's password (decisions §30 "12a3").
pub async fn unlock(
    State(state): State<HttpState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request_headers: HeaderMap,
    uri: axum::http::Uri,
    Json(body): Json<UnlockBody>,
) -> Response {
    // Unlocking is not a login (decisions 0290): a user just switched to on the lock surface
    // unlocks with their password while their login still waits for the Meow License.
    let token = crate::auth::token_from_request(&request_headers, &uri);
    let Some(user) = token
        .as_deref()
        .and_then(|t| state.console.verify_token(t).or_else(|| state.console.pending_consent(t)))
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "ok": false, "message": "not logged in" }))).into_response();
    };
    let console = state.console.clone();
    let from = peer.ip().to_string();
    let result = tokio::task::spawn_blocking(move || {
        console.unlock(&user, &body.device, body.password.as_deref(), &from)
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string()));
    match result {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(message) => (StatusCode::UNAUTHORIZED, Json(json!({ "ok": false, "message": message }))).into_response(),
    }
}

pub async fn logout(
    State(state): State<HttpState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request_headers: HeaderMap,
    uri: axum::http::Uri,
) -> Response {
    if let Some(token) = crate::auth::token_from_request(&request_headers, &uri) {
        let console = state.console.clone();
        let from = peer.ip().to_string();
        let _ = tokio::task::spawn_blocking(move || console.logout(&token, &from)).await;
    }
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie("", 0));
    (headers, Json(json!({ "ok": true }))).into_response()
}

/// GET /api/feedback — notification settings + what is active now.
pub async fn feedback(State(state): State<HttpState>, AuthUser(user): AuthUser) -> Json<Value> {
    let active: Vec<Value> = state
        .console
        .feedback_active(&user)
        .iter()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    Json(json!({ "settings": state.console.feedback_settings(&user), "active": active }))
}

/// GET /api/ui-keys — the logged-in user's UI keys and gesture timing.
pub async fn ui_keys(State(state): State<HttpState>, AuthUser(user): AuthUser) -> Json<Value> {
    Json(state.console.ui_keys(&user))
}

pub async fn me(State(state): State<HttpState>, request_headers: HeaderMap, uri: axum::http::Uri) -> Response {
    let token = crate::auth::token_from_request(&request_headers, &uri);
    if let Some(user) = token.as_deref().and_then(|t| state.console.verify_token(t)) {
        let screens = state.console.screen_configs(&user);
        let look = state.console.look(&user);
        return Json(json!({ "user": user, "screen_configs": screens, "look": look })).into_response();
    }
    // A login waiting for the Meow License (decisions 0290): the page asks its consent again.
    if let Some(user) = token.as_deref().and_then(|t| state.console.pending_consent(t)) {
        let message = format!("{user}, accept the Meow License first");
        let body = json!({ "ok": false, "consent": true, "user": user, "code": "E-USER-010", "message": message });
        return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
    }
    (StatusCode::UNAUTHORIZED, Json(json!({ "ok": false, "message": "not logged in" }))).into_response()
}
