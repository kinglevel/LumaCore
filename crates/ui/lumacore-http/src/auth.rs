//! Who is calling: the login token from `POST /api/login`.
//!
//! The token travels as the `lumacore_token` cookie (set on login, so fetch,
//! SSE and the WebSocket carry it automatically), an `Authorization: Bearer`
//! header, or a `?token=` query parameter (non-browser clients). The console
//! verifies it (signed with the station secret; see `terminal/passwords.rs`).

use axum::Json;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use serde_json::{Value, json};

use crate::HttpState;

pub const COOKIE: &str = "lumacore_token";

/// The raw token from cookie, bearer header or `?token=`.
pub fn token_from(parts: &Parts) -> Option<String> {
    token_from_request(&parts.headers, &parts.uri)
}

pub fn token_from_request(
    headers: &axum::http::HeaderMap,
    uri: &axum::http::Uri,
) -> Option<String> {
    if let Some(bearer) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return Some(bearer.trim().to_string());
    }
    let from_cookie = headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|c| {
            c.trim()
                .strip_prefix(&format!("{COOKIE}="))
                .map(str::to_string)
        });
    if from_cookie.is_some() {
        return from_cookie;
    }
    uri.query().and_then(|q| {
        q.split('&').find_map(|kv| {
            kv.strip_prefix("token=")
                .map(|t| t.replace("%2E", ".").replace("%2e", "."))
        })
    })
}

/// A logged-in user. Rejects with 401 when there is no valid token.
pub struct AuthUser(pub String);

impl FromRequestParts<HttpState> for AuthUser {
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &HttpState,
    ) -> Result<Self, Self::Rejection> {
        token_from(parts)
            .and_then(|t| state.console.verify_token(&t))
            .map(AuthUser)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "ok": false, "message": "not logged in" })),
            ))
    }
}

/// The device a page belongs to (`?device=<id>`, decisions §30 "12a3"):
/// letters, digits and `-` only, at most 64.
pub fn device_from_request(uri: &axum::http::Uri) -> Option<String> {
    uri.query()?.split('&').find_map(|kv| device_id(kv.strip_prefix("device=")?))
}

/// The header a page's REST calls name their device with (R1-42).
pub const DEVICE_HEADER: &str = "x-luma-device";

fn device_id(id: &str) -> Option<String> {
    (!id.is_empty() && id.len() <= 64 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
        .then(|| id.to_string())
}

/// The device a REST call came from: the `X-Luma-Device` header, else
/// `?device=`. The station refuses a locked device's lines (E-LOCK-001).
pub struct Device(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for Device {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Device(device_of(&parts.headers, &parts.uri)))
    }
}

/// The device of a REST call: the `X-Luma-Device` header, else `?device=`.
pub fn device_of(headers: &axum::http::HeaderMap, uri: &axum::http::Uri) -> Option<String> {
    headers
        .get(DEVICE_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(device_id)
        .or_else(|| device_from_request(uri))
}
