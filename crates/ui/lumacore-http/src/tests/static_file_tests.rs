//! Tests for static file serving and SPA fallback.

use std::sync::Arc;

use arc_swap::ArcSwap;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use crate::HttpState;
use crate::router::build_router;
use crate::state::DiagnosticsSnapshot;

fn test_state_with_web_root(web_root: Option<std::path::PathBuf>) -> HttpState {
    let snap = Arc::new(DiagnosticsSnapshot::default());
    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(1);
    super::test_http_state(Arc::new(ArcSwap::from(snap)), tx, web_root)
}

#[tokio::test]
async fn serves_index_html_at_root() {
    // Create a temp directory with an index.html.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("index.html"),
        "<html><body>LumaCore Test</body></html>",
    )
    .unwrap();

    let state = test_state_with_web_root(Some(dir.path().to_path_buf()));
    let app = build_router(state);

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("LumaCore Test"));
}

#[tokio::test]
async fn serves_static_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "<html></html>").unwrap();

    // Create a subdirectory with a JS file.
    std::fs::create_dir_all(dir.path().join("assets")).unwrap();
    std::fs::write(dir.path().join("assets/app.js"), "console.log('hello');").unwrap();

    let state = test_state_with_web_root(Some(dir.path().to_path_buf()));
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/assets/app.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("console.log"));
}

#[tokio::test]
async fn spa_fallback_serves_index_for_unknown_routes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("index.html"),
        "<html><body>SPA Root</body></html>",
    )
    .unwrap();

    let state = test_state_with_web_root(Some(dir.path().to_path_buf()));
    let app = build_router(state);

    // A path that doesn't match any file should fall back to index.html
    // (React Router handles client-side routing).
    let response = app
        .oneshot(
            Request::builder()
                .uri("/dashboard/fixtures")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("SPA Root"));
}

#[tokio::test]
async fn api_routes_take_priority_over_static() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "<html></html>").unwrap();

    let state = test_state_with_web_root(Some(dir.path().to_path_buf()));
    let app = build_router(state);

    // API routes should still work even with static file serving enabled.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Should be a JSON response, not HTML.
    assert!(json["build"].is_object());
}

async fn get_path(state: HttpState, uri: &str) -> (StatusCode, String, String) {
    let response = build_router(state)
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let mime = response
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, mime, String::from_utf8_lossy(&body).into_owned())
}

#[tokio::test]
async fn without_a_web_root_the_embedded_webui_is_served() {
    let state = || test_state_with_web_root(None);
    let (status, mime, body) = get_path(state(), "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(mime.starts_with("text/html"), "{mime}");
    assert_eq!(body.as_bytes(), lumacore_webui::get("/index.html").unwrap().0);
    let (status, mime, _) = get_path(state(), "/kit/kit.js").await;
    assert_eq!((status, mime.as_str()), (StatusCode::OK, "text/javascript; charset=utf-8"));
    // A page route gets the station's page; a missing file is 404.
    let (status, _, body) = get_path(state(), "/some/page").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_bytes(), lumacore_webui::get("/index.html").unwrap().0);
    assert_eq!(get_path(state(), "/kit/nope.js").await.0, StatusCode::NOT_FOUND);
    // The API still wins.
    let (status, mime, _) = get_path(state(), "/api/info").await;
    assert_eq!(status, StatusCode::OK);
    assert!(mime.starts_with("application/json"), "{mime}");
}

#[tokio::test]
async fn a_web_root_that_is_missing_falls_back_to_the_embedded_webui() {
    let state = test_state_with_web_root(Some(std::path::PathBuf::from("/nonexistent/path")));
    let (status, _, body) = get_path(state, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_bytes(), lumacore_webui::get("/index.html").unwrap().0);
}
