//! Integration tests: spin up the HTTP engine and hit every endpoint.

use std::sync::Arc;

use arc_swap::ArcSwap;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use crate::router::build_router;
use crate::state::DiagnosticsSnapshot;
use lumacore_tree::runtime::ThreadHealthView;

fn thread(name: &str, alive: bool, panics: u64) -> ThreadHealthView {
    ThreadHealthView {
        name: name.to_string(),
        alive,
        last_activity_ms: 1_000,
        iterations: 42,
        last_loop_us: 900,
        max_loop_us: 1_500,
        queue_depth: 0,
        refused: 0,
        panics,
        scheduling: String::new(),
    }
}

/// Build a test app with pre-populated diagnostics.
fn test_app() -> axum::Router {
    let snap = Arc::new(DiagnosticsSnapshot {
        threads: vec![thread("Render.Tick", true, 0), thread("State", true, 0)],
        version: "0.1.0".to_string(),
        profile: "release".to_string(),
        target_os: "macos".to_string(),
        target_arch: "aarch64".to_string(),
        git_commit: "deadbeef".to_string(),
        build_id: "0.1.0/release/macos-aarch64/deadbeef".to_string(),
        uptime_secs: 600,
        show_path: Some("library/shows/test.luma".to_string()),
        session_count: 3,
        fixture_count: 24,
        universe_count: 2,
        output_count: 1,
        protocols_running: [("Sacn".to_string(), 1)].into(),
        sysmon_history: vec![
            r#"{"seq":1,"subsystem":"Server","message":"LumaCore started"}"#.to_string(),
            r#"{"seq":2,"subsystem":"sACN","message":"Output started"}"#.to_string(),
        ],
    });

    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(64);
    let state = super::test_http_state(Arc::new(ArcSwap::from(snap)), tx, None);

    build_router(state)
}

async fn get_json(app: axum::Router, path: &str) -> (StatusCode, serde_json::Value) {
    let response = app
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

/// R1-42: a REST command names its page's device (`X-Luma-Device`, else
/// `?device=`), so the station can refuse a locked device's lines.
#[tokio::test]
async fn rest_commands_carry_the_device() {
    let send = |req: Request<Body>| async move {
        let response = test_app().oneshot(req).await.unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<serde_json::Value>(&body).unwrap()["message"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };
    let token = super::TEST_TOKEN;
    let delete = |uri: String, device: Option<&str>| {
        let mut req = Request::builder().method("DELETE").uri(uri);
        if let Some(d) = device {
            req = req.header("X-Luma-Device", d);
        }
        req.body(Body::empty()).unwrap()
    };
    let header = send(delete(format!("/api/views/3?token={token}"), Some("tab-7"))).await;
    assert!(header.contains("device tab-7"), "{header}");
    let query = send(delete(format!("/api/views/3?token={token}&device=app-2"), None)).await;
    assert!(query.contains("device app-2"), "{query}");
    let bad = send(delete(format!("/api/views/3?token={token}"), Some("../etc"))).await;
    assert!(bad.contains("device none"), "{bad}");
}

#[tokio::test]
async fn health_endpoint_returns_threads() {
    let (status, json) = get_json(test_app(), "/api/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["state"], "ok");
    assert_eq!(json["threads"][0]["name"], "Render.Tick");
    assert_eq!(json["threads"][0]["max_loop_us"], 1_500);
    assert_eq!(json["threads"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn health_endpoint_reports_a_stopped_thread() {
    let snap = Arc::new(DiagnosticsSnapshot {
        threads: vec![thread("State", true, 0), thread("Render.Tick", false, 1)],
        ..Default::default()
    });
    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(1);
    let state = super::test_http_state(Arc::new(ArcSwap::from(snap)), tx, None);
    let app = build_router(state);

    let (status, json) = get_json(app, "/api/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["state"], "degraded");
    assert_eq!(json["threads"][1]["alive"], false);
}

#[tokio::test]
async fn metrics_endpoint_returns_threads_and_runtime() {
    let (status, json) = get_json(test_app(), "/api/metrics").await;
    assert_eq!(status, StatusCode::OK);

    // Thread fields.
    assert_eq!(json["threads"][1]["name"], "State");
    assert_eq!(json["threads"][1]["iterations"], 42);

    // Runtime fields.
    assert_eq!(json["runtime"]["fixtures"], 24);
    assert_eq!(json["runtime"]["universes"], 2);
    assert_eq!(json["runtime"]["outputs"], 1);
    assert_eq!(json["runtime"]["sessions"], 3);
    assert_eq!(json["runtime"]["protocols_running"]["Sacn"], 1);
}

#[tokio::test]
async fn info_endpoint_returns_build_and_stats() {
    let (status, json) = get_json(test_app(), "/api/info").await;
    assert_eq!(status, StatusCode::OK);

    // Build info.
    assert_eq!(json["build"]["version"], "0.1.0");
    assert_eq!(json["build"]["profile"], "release");
    assert_eq!(json["build"]["target_os"], "macos");
    assert_eq!(json["build"]["target_arch"], "aarch64");
    assert_eq!(json["build"]["git_commit"], "deadbeef");
    assert_eq!(
        json["build"]["build_id"],
        "0.1.0/release/macos-aarch64/deadbeef"
    );

    // Runtime stats.
    assert_eq!(json["uptime_secs"], 600);
    assert_eq!(json["show_path"], "library/shows/test.luma");
    assert_eq!(json["session_count"], 3);
    assert_eq!(json["fixture_count"], 24);
    assert_eq!(json["universe_count"], 2);
    assert_eq!(json["output_count"], 1);
    assert_eq!(json["protocols_running"]["Sacn"], 1);
}

#[tokio::test]
async fn sysmon_endpoint_returns_history() {
    let path = format!("/api/sysmon?token={}", super::TEST_TOKEN);
    let (status, json) = get_json(test_app(), &path).await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(json["count"], 2);
    let lines = json["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["message"], "LumaCore started");
    assert_eq!(lines[1]["subsystem"], "sACN");
}

#[tokio::test]
async fn sysmon_endpoint_needs_login() {
    let (status, _) = get_json(test_app(), "/api/sysmon").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Build a standalone trace router (trace endpoints moved to Perfetto module port).
fn trace_app() -> axum::Router {
    axum::Router::new()
        .route(
            "/api/trace",
            axum::routing::get(crate::endpoints::trace::get_trace),
        )
        .route(
            "/api/trace/download",
            axum::routing::get(crate::endpoints::trace::download_trace),
        )
}

#[tokio::test]
async fn trace_endpoint_returns_perfetto_format() {
    // Initialize the global trace collector and add a test event.
    lumacore_trace::collector::TRACE_COLLECTOR.init();
    lumacore_trace::collector::TRACE_COLLECTOR.record(lumacore_trace::collector::TraceEvent {
        name: "endpoint_test_zone",
        category: "test",
        phase: 'X',
        timestamp_us: 1000,
        duration_us: 500,
        pid: 1,
        tid: 1,
    });

    let (status, json) = get_json(trace_app(), "/api/trace").await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(json["displayTimeUnit"], "us");
    let trace_events = json["traceEvents"].as_array().unwrap();

    // Find our specific event (other tests may have added events too).
    let our_event = trace_events
        .iter()
        .find(|e| e["name"] == "endpoint_test_zone");
    assert!(
        our_event.is_some(),
        "should find our test event in trace output"
    );

    // Verify event structure.
    let event = our_event.unwrap();
    assert!(event["name"].is_string());
    assert!(event["cat"].is_string());
    assert!(event["ph"].is_string());
    assert!(event["ts"].is_number());
    assert!(event["pid"].is_number());
    assert!(event["tid"].is_number());
    assert!(
        event["dur"].is_number(),
        "complete event should have dur field"
    );
}

#[tokio::test]
async fn trace_endpoint_seconds_filter() {
    lumacore_trace::collector::TRACE_COLLECTOR.init();

    let (status, json) = get_json(trace_app(), "/api/trace?seconds=10").await;
    assert_eq!(status, StatusCode::OK);
    assert!(json["traceEvents"].is_array());
}

#[tokio::test]
async fn unknown_api_route_returns_404() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cors_headers_present() {
    let app = test_app();
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/health")
                .header("Origin", "http://localhost:3000")
                .header("Access-Control-Request-Method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response
            .headers()
            .contains_key("access-control-allow-origin"),
        "CORS allow-origin header should be present"
    );
}

#[tokio::test]
async fn diagnostics_update_reflected_in_endpoints() {
    let snap = Arc::new(DiagnosticsSnapshot::default());
    let store = Arc::new(ArcSwap::from(snap));
    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(1);
    let state = super::test_http_state(store.clone(), tx, None);

    // First request: default snapshot.
    let app = build_router(state.clone());
    let (_, json) = get_json(app, "/api/info").await;
    assert_eq!(json["uptime_secs"], 0);

    // Update the snapshot (simulates the server main loop).
    store.store(Arc::new(DiagnosticsSnapshot {
        uptime_secs: 999,
        version: "0.2.0".to_string(),
        ..Default::default()
    }));

    // Second request: should see the updated values.
    let app2 = build_router(state);
    let (_, json2) = get_json(app2, "/api/info").await;
    assert_eq!(json2["uptime_secs"], 999);
    assert_eq!(json2["build"]["version"], "0.2.0");
}

/// A command waiting on the state thread doesn't hold a tokio worker
/// (R1-67): with one worker thread, another request is still answered while
/// the command waits. (Run on the worker, the second request would wait for
/// the release 3 s later.)
#[tokio::test(flavor = "current_thread")]
async fn a_waiting_command_leaves_the_runtime_free() {
    let (state, hold) = super::held_http_state();
    let app = build_router(state);
    let releaser = hold.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        releaser.release();
    });
    let auth = format!("Bearer {}", super::TEST_TOKEN);
    let command = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/views/1/recall")
            .header("authorization", &auth)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"screen":1}"#))
            .unwrap(),
    );
    let started = std::time::Instant::now();
    let other = async {
        // Let the command start first.
        tokio::task::yield_now().await;
        let (status, _) = get_json(app.clone(), "/api/health").await;
        (status, started.elapsed())
    };
    let (command, (status, waited)) = tokio::join!(command, other);
    assert_eq!(status, StatusCode::OK);
    assert!(waited < std::time::Duration::from_secs(2), "the other request waited {waited:?}");
    assert_eq!(command.unwrap().status(), StatusCode::OK);
    hold.release();
}
