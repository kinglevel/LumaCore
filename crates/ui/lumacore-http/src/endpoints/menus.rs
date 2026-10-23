//! Data for the Network, DMX Protocols and LumaSessions menus (decisions
//! §11 "Menus, LumaSession and staged patch").
//!
//! Routes (login required):
//!   GET /api/network      → interfaces + services, networks + this station's
//!                           map, lines with routes and live state, universes,
//!                           input sources, Art-Net nodes
//!   GET /api/lumasessions → this station's settings and status, every
//!                           station discovery sees, members
//!   GET /api/menu-types   → the menu registry (open, like /api/window-types)
//!   GET /api/tool-types   → the editor tools (open)
//!   GET /api/object-menus → every object's context menu (open)
//!
//! Read-only: every change runs as a command over the shell WebSocket.

use axum::Json;
use axum::extract::State;
use serde_json::Value;

use crate::HttpState;
use crate::auth::AuthUser;

pub async fn network(AuthUser(_user): AuthUser, State(state): State<HttpState>) -> Json<Value> {
    Json(state.console.network())
}

pub async fn lumasessions(
    AuthUser(_user): AuthUser,
    State(state): State<HttpState>,
) -> Json<Value> {
    Json(state.console.lumasessions())
}

/// The menu registry (decisions §30 "12d"): static, so open like the window
/// types (the page draws a menu's frame before a login answers).
pub async fn menu_types(State(state): State<HttpState>) -> Json<Value> {
    Json(state.console.menu_types())
}

/// Every object's context menu (decisions §34 round 3, 18): static and open like the menus.
pub async fn object_menus(State(state): State<HttpState>) -> Json<Value> {
    Json(state.console.object_menus())
}

/// The editor tools (decisions §30 "12e"): static and open like the menus.
pub async fn tool_types(State(state): State<HttpState>) -> Json<Value> {
    Json(state.console.tool_types())
}
