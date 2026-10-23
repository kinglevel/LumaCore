//! Window type registry endpoint.
//!
//! Routes:
//!   GET /api/window-types → built-in window types with their settings schemas

use axum::Json;

use lumacore_tree::WindowType;

pub async fn list_window_types() -> Json<Vec<WindowType>> {
    Json(
        lumacore_registry::get().window_types().values().cloned()
            .collect(),
    )
}
