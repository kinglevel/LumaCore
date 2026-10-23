//! Router assembly: axum Router with CORS, endpoints, the two WebSockets and
//! static files (no SSE stream: everything live rides /ws/feeds, decisions
//! §30 "12d hardening").

use axum::Router;
use axum::routing::{delete, get, post, put};
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};

use crate::HttpState;

/// Build the complete axum router with all routes, middleware, and static files.
pub fn build_router(state: HttpState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        // ── diagnostics ───────────────────────────────────────────────
        .route("/api/health", get(crate::endpoints::health::get_health))
        .route("/api/metrics", get(crate::endpoints::metrics::get_metrics))
        .route("/api/info", get(crate::endpoints::info::get_info))
        // ── station + login ───────────────────────────────────────────
        .route("/api/station", get(crate::endpoints::station::get_station))
        .route("/api/login", post(crate::endpoints::station::login))
        .route("/api/logout", post(crate::endpoints::station::logout))
        .route("/api/unlock", post(crate::endpoints::station::unlock))
        .route("/api/me", get(crate::endpoints::station::me))
        .route("/api/ui-keys", get(crate::endpoints::station::ui_keys))
        .route("/api/sysmon", get(crate::endpoints::sysmon::get_sysmon))
        // ── screens (live windows, Root.Runtime) ────────────────
        .route(
            "/api/screens",
            get(crate::endpoints::live_screens::list_live_screens)
                .post(crate::endpoints::live_screens::join_live_screen),
        )
        .route(
            "/api/screens/{no}",
            get(crate::endpoints::live_screens::get_live_screen),
        )
        .route(
            "/api/screens/{no}/layout",
            axum::routing::put(crate::endpoints::live_screens::put_live_screen_layout),
        )
        .route(
            "/api/window-types",
            get(crate::endpoints::window_types::list_window_types),
        )
        .route(
            "/api/screenshots",
            post(crate::endpoints::screenshots::upload).layer(axum::extract::DefaultBodyLimit::max(
                crate::endpoints::screenshots::MAX_BYTES,
            )),
        )
        .route("/api/media/{sha}", get(crate::endpoints::media::get_media))
        .route(
            "/api/media",
            post(crate::endpoints::media::upload)
                .layer(axum::extract::DefaultBodyLimit::max(crate::endpoints::media::MAX_BYTES)),
        )
        .route("/api/shows", get(crate::endpoints::shows::list_shows))
        .route("/api/show-parts", get(crate::endpoints::shows::show_parts))
        .route("/api/network", get(crate::endpoints::menus::network))
        .route("/api/menu-types", get(crate::endpoints::menus::menu_types))
        .route("/api/tool-types", get(crate::endpoints::menus::tool_types))
        .route("/api/object-menus", get(crate::endpoints::menus::object_menus))
        .route("/api/fixture-types", get(crate::endpoints::gdtf::fixture_types))
        .route("/api/fixture-type", get(crate::endpoints::gdtf::fixture_type))
        .route("/api/fixture-type/thumbnail", get(crate::endpoints::gdtf::thumbnail))
        .route(
            "/api/gdtf",
            post(crate::endpoints::gdtf::upload).layer(axum::extract::DefaultBodyLimit::max(crate::endpoints::media::MAX_BYTES)),
        )
        // This device (decisions §38 5): one upload route for every file dialog, 512 MB.
        .route(
            "/api/upload",
            post(crate::endpoints::files::upload).layer(axum::extract::DefaultBodyLimit::max(crate::endpoints::files::MAX_BYTES)),
        )
        .route("/api/lumasessions", get(crate::endpoints::menus::lumasessions))
        // ── stored views + view buttons (per user, via commands) ──────
        .route("/api/views", get(crate::endpoints::views::list_views))
        .route(
            "/api/views/{no}",
            delete(crate::endpoints::views::delete_view),
        )
        .route(
            "/api/views/{no}/recall",
            post(crate::endpoints::views::recall_view),
        )
        .route(
            "/api/views/{no}/label",
            post(crate::endpoints::views::label_view),
        )
        .route(
            "/api/view-buttons/{slot}",
            put(crate::endpoints::views::assign_button),
        );

    // Two sockets only (decisions §30 "12b2 hardening": one feed rule; "12d":
    // no SSE stream either): every live feed rides /ws/feeds, the terminal
    // /ws/shell. A test holds it.
    let ws = Router::new()
        .route("/ws/feeds", get(crate::websocket::feeds::ws_feeds))
        .route("/api/pools/{pool}", get(crate::endpoints::pools::pool))
        .route("/api/pools/{pool}/{no}", get(crate::endpoints::pools::pool_object))
        .route("/api/feedback", get(crate::endpoints::station::feedback))
        .route("/ws/shell", get(crate::websocket::shell::ws_shell));

    let mut app = Router::new()
        .merge(api)
        .merge(ws)
        .layer(cors)
        .layer(CompressionLayer::new());

    // The WebUI (decisions §36 2): from `--web-root` when it points at a folder (live edits),
    // else the files embedded in `lumacore-webui`. no-store so WKWebView never uses a stale cache.
    use axum::http::{HeaderName, HeaderValue};
    use tower::ServiceBuilder;
    use tower_http::set_header::SetResponseHeaderLayer;
    let no_store = SetResponseHeaderLayer::overriding(
        HeaderName::from_static("cache-control"),
        HeaderValue::from_static("no-store"),
    );
    if let Some(ref web_root) = state.web_root
        && web_root.exists()
    {
        use tower_http::services::ServeDir;
        let serve_dir = ServeDir::new(web_root)
            .append_index_html_on_directories(true)
            .fallback(tower_http::services::ServeFile::new(
                web_root.join("index.html"),
            ));
        app = app.fallback_service(ServiceBuilder::new().layer(no_store).service(serve_dir));
    } else {
        use axum::handler::HandlerWithoutStateExt;
        app = app.fallback_service(ServiceBuilder::new().layer(no_store).service(embedded.into_service()));
    }

    app.with_state(state)
}

/// A WebUI file from the binary (decisions §36 2). A path without a file extension is a page
/// route and gets the station's page (as the disk fallback does); a missing file, and any
/// `/api/` or `/ws/` path no route took, is 404.
async fn embedded(uri: axum::http::Uri) -> axum::response::Response {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;
    let path = uri.path();
    if matches!(path.split('/').nth(1), Some("api" | "ws")) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let last = path.rsplit('/').next().unwrap_or("");
    let file = lumacore_webui::get(path).or_else(|| (!last.contains('.')).then(|| lumacore_webui::get(lumacore_webui::INDEX)).flatten());
    match file {
        Some((bytes, mime)) => ([(header::CONTENT_TYPE, mime)], bytes).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
