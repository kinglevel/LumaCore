//! Stored views and view buttons — per user, saved in the show.
//!
//! Every write runs the same command the terminal would (`Store View`, `View`,
//! `Delete View`, `Assign View … At ViewButton …`) through the `ConsoleBridge`,
//! so views land in `ShowData.UserProfiles.<user>` and the terminal and the UI
//! can't drift apart. Changes are broadcast as `views_changed` SSE events.
//!
//! Routes:
//!   GET    /api/views                    → `{views, buttons}` for the user
//!   POST   /api/views/{no}/recall        → `{screen}` recall onto a screen
//!   POST   /api/views/{no}/label         → `{name}` rename (contents untouched)
//!   DELETE /api/views/{no}               → delete the view
//!   PUT    /api/view-buttons/{slot}      → `{view}` assign (0 clears)

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::HttpState;
use crate::auth::{AuthUser, Device};

type Reply = Result<Json<Value>, (StatusCode, Json<Value>)>;

/// Run `cmd` as `user` from `device` on the station. The bridge waits for
/// the state thread, so it runs on a blocking thread, never on a tokio
/// worker (R1-67).
async fn run(state: &HttpState, user: &str, device: &Device, cmd: String) -> Reply {
    let console = state.console.clone();
    let user = user.to_string();
    let device = device.0.clone();
    let result = tokio::task::spawn_blocking(move || console.execute(&user, device.as_deref(), &cmd))
        .await
        .unwrap_or_else(|e| Err(e.to_string()));
    match result {
        Ok(message) => Ok(Json(json!({ "ok": true, "message": message }))),
        Err(message) => Err((
            crate::status_of(&message),
            Json(json!({ "ok": false, "message": message })),
        )),
    }
}

/// The logged-in user's views and view buttons.
pub async fn list_views(State(state): State<HttpState>, AuthUser(user): AuthUser) -> Json<Value> {
    Json(state.console.user_views(&user))
}

#[derive(Deserialize)]
pub struct ScreenBody {
    screen: u32,
}

pub async fn recall_view(
    State(state): State<HttpState>,
    AuthUser(user): AuthUser,
    device: Device,
    Path(no): Path<u32>,
    Json(body): Json<ScreenBody>,
) -> Reply {
    run(&state, &user, &device, format!("view {no} /screen {}", body.screen)).await
}

#[derive(Deserialize)]
pub struct LabelBody {
    name: String,
}

pub async fn label_view(
    State(state): State<HttpState>,
    AuthUser(user): AuthUser,
    device: Device,
    Path(no): Path<u32>,
    Json(body): Json<LabelBody>,
) -> Reply {
    let name = body.name.replace('"', "");
    run(
        &state,
        &user,
        &device,
        format!("label view {no} \"{}\"", name.trim()),
    ).await
}

pub async fn delete_view(
    State(state): State<HttpState>,
    AuthUser(user): AuthUser,
    device: Device,
    Path(no): Path<u32>,
) -> Reply {
    run(&state, &user, &device, format!("delete view {no}")).await
}

#[derive(Deserialize)]
pub struct ButtonBody {
    view: u32,
}

pub async fn assign_button(
    State(state): State<HttpState>,
    AuthUser(user): AuthUser,
    device: Device,
    Path(slot): Path<u32>,
    Json(body): Json<ButtonBody>,
) -> Reply {
    run(
        &state,
        &user,
        &device,
        format!("assign view {} at viewbutton {slot}", body.view),
    ).await
}
