//! Integration tests for the Perfetto engine's standalone HTTP port.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use tower_http::cors::{Any, CorsLayer};

use lumacore_trace::collector::TRACE_COLLECTOR;

/// Build the minimal Perfetto router (same as PerfettoEngine uses).
fn perfetto_app() -> axum::Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    axum::Router::new()
        .route(
            "/api/trace",
            axum::routing::get(crate::endpoints::trace::get_trace),
        )
        .route(
            "/api/trace/download",
            axum::routing::get(crate::endpoints::trace::download_trace),
        )
        .layer(cors)
}

#[tokio::test]
async fn perfetto_serves_trace_json_on_port() {
    TRACE_COLLECTOR.init();
    let app = perfetto_app();

    let req = Request::builder()
        .uri("/api/trace")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("traceEvents").is_some());
    assert_eq!(json["displayTimeUnit"], "us");
}

#[tokio::test]
async fn perfetto_download_has_content_disposition() {
    TRACE_COLLECTOR.init();
    let app = perfetto_app();

    let req = Request::builder()
        .uri("/api/trace/download")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let disposition = resp
        .headers()
        .get("content-disposition")
        .expect("should have content-disposition header")
        .to_str()
        .unwrap();
    assert!(disposition.contains("attachment"));
    assert!(disposition.contains("lumacore-trace.json"));
}

#[tokio::test]
async fn perfetto_trace_contains_recorded_events() {
    TRACE_COLLECTOR.init();

    // Record a test event
    {
        let _guard = lumacore_trace::PerfettoGuard::new("test_category", "test_event");
        // Guard records a complete event on drop
    }

    let app = perfetto_app();
    let req = Request::builder()
        .uri("/api/trace")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let events = json["traceEvents"].as_array().unwrap();
    assert!(
        !events.is_empty(),
        "should contain at least the recorded event"
    );
}

#[tokio::test]
async fn perfetto_seconds_filter_works() {
    TRACE_COLLECTOR.init();

    let app = perfetto_app();
    // Request with very small window — should still return valid JSON
    let req = Request::builder()
        .uri("/api/trace?seconds=1")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("traceEvents").is_some());
}

#[tokio::test]
async fn perfetto_cors_headers_present() {
    TRACE_COLLECTOR.init();
    let app = perfetto_app();

    // Send a preflight OPTIONS-like GET with Origin header
    let req = Request::builder()
        .uri("/api/trace")
        .header("origin", "https://ui.perfetto.dev")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        resp.headers().get("access-control-allow-origin").is_some(),
        "CORS allow-origin header should be present"
    );
}
