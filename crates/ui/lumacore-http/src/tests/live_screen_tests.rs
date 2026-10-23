//! The layout PUT (`PUT /api/screens/{no}/layout`, R1-56/57/58/42): only the
//! floor plan's structure comes from the page; what a window holds stays the
//! station's; a stale revision and a locked device are refused.

use std::sync::Arc;

use arc_swap::ArcSwap;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::HttpState;
use crate::router::build_router;
use crate::state::DiagnosticsSnapshot;

/// A station with Admin's screen (Terminal 1 + Groups 2); Groups 2 holds a
/// colour, scroll, label, a setting and a preset. Returns (state, screen).
fn station() -> (HttpState, u32) {
    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(8);
    let state = super::test_http_state(Arc::new(ArcSwap::from_pointee(DiagnosticsSnapshot::default())), tx, None);
    let no = {
        let mut store = state.live_screens.write().unwrap();
        let no = store.join_or_create("Admin", None);
        store
            .update_window(no, 2, |w| {
                w.appearance.color = Some("#ff0000".into());
                w.scroll.v = 40.0;
                w.label = Some("Busk".into());
                w.settings.insert("slots".into(), json!(12));
                w.preset = Some(uuid::Uuid::from_u128(7));
                Ok(())
            })
            .unwrap();
        no
    };
    (state, no)
}

fn revision(state: &HttpState, no: u32) -> u64 {
    state.live_screens.read().unwrap().get(no).unwrap().revision
}

async fn put(state: &HttpState, no: u32, body: Value, device: Option<&str>) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method("PUT")
        .uri(format!("/api/screens/{no}/layout"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", super::TEST_TOKEN));
    if let Some(d) = device {
        req = req.header("x-luma-device", d);
    }
    let response = build_router(state.clone())
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

/// What a page sends after a split drag: the tree, and every window as it
/// knows it (resolved settings, no colour, no scroll, no preset).
fn page_body(revision: u64, ratio: f64) -> Value {
    json!({
        "revision": revision,
        "root": {"type": "split", "dir": "h", "ratio": ratio, "children": [
            {"type": "leaf", "windows": [1], "active": 0},
            {"type": "leaf", "windows": [2, 3], "active": 0},
        ]},
        "windows": {
            "1": {"no": 1, "kind": "terminal", "settings": {}},
            "2": {"no": 2, "kind": "groups", "label": "Stale", "settings": {"slots": 0, "first_index": 1}},
            "3": {"no": 3, "kind": "groups", "settings": {"first_index": 1, "slots": 5}},
        }
    })
}

#[tokio::test]
async fn a_layout_put_keeps_colour_and_scroll() {
    let (state, no) = station();
    let (status, body) = put(&state, no, page_body(revision(&state, no), 0.3), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let store = state.live_screens.read().unwrap();
    let screen = store.get(no).unwrap();
    // The structure is the page's ...
    assert!(matches!(screen.root, lumacore_tree::runtime::LayoutNode::Split { ratio, .. } if (ratio - 0.3).abs() < 1e-9));
    // ... what window 2 holds stays the station's.
    let w = &screen.windows[&2];
    assert_eq!(w.appearance.color.as_deref(), Some("#ff0000"));
    assert_eq!(w.scroll.v, 40.0);
    assert_eq!(w.label.as_deref(), Some("Busk"));
    assert_eq!(w.preset, Some(uuid::Uuid::from_u128(7)));
}

#[tokio::test]
async fn a_layout_put_ignores_settings_for_existing_windows() {
    let (state, no) = station();
    let (status, body) = put(&state, no, page_body(revision(&state, no), 0.5), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let store = state.live_screens.read().unwrap();
    let screen = store.get(no).unwrap();
    // The page's resolved settings don't replace the stored ones.
    assert_eq!(screen.windows[&2].settings, json!({"slots": 12}).as_object().unwrap().clone());
    // A new window keeps only what differs from its type's defaults, so a
    // later default reaches it.
    assert_eq!(screen.windows[&3].settings, json!({"slots": 5}).as_object().unwrap().clone());
    assert!(screen.windows[&1].settings.is_empty());
}

#[tokio::test]
async fn a_layout_put_with_another_kind_is_a_new_window() {
    let (state, no) = station();
    let mut body = page_body(revision(&state, no), 0.5);
    body["windows"]["2"] = json!({"no": 2, "kind": "terminal", "label": "T", "settings": {}});
    let (status, reply) = put(&state, no, body, None).await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    let store = state.live_screens.read().unwrap();
    let w = &store.get(no).unwrap().windows[&2];
    assert_eq!(w.kind, "terminal");
    assert_eq!(w.label.as_deref(), Some("T"));
    assert_eq!(w.appearance.color, None);
    assert_eq!(w.preset, None);
}

#[tokio::test]
async fn a_stale_layout_put_is_refused() {
    let (state, no) = station();
    let seen = revision(&state, no);
    // A `Set Window` / `Label Self` lands between the page's read and its PUT.
    state
        .live_screens
        .write()
        .unwrap()
        .update_window(no, 2, |w| {
            w.settings.insert("slots".into(), json!(20));
            Ok(())
        })
        .unwrap();
    let (status, body) = put(&state, no, page_body(seen, 0.3), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    // The reply carries the current screen for the page to apply.
    assert_eq!(body["screen"]["revision"], json!(revision(&state, no)));
    assert_eq!(body["screen"]["windows"]["2"]["settings"]["slots"], json!(20));
    let store = state.live_screens.read().unwrap();
    let screen = store.get(no).unwrap();
    assert_eq!(screen.windows[&2].settings["slots"], json!(20), "the setting survives");
    assert!(!screen.windows.contains_key(&3), "nothing of the stale PUT landed");
}

#[tokio::test]
async fn a_layout_put_without_a_revision_is_refused() {
    let (state, no) = station();
    let mut body = page_body(0, 0.3);
    body.as_object_mut().unwrap().remove("revision");
    let (status, _) = put(&state, no, body, None).await;
    assert!(status.is_client_error(), "{status}");
}

#[tokio::test]
async fn a_locked_device_cant_change_its_layout() {
    let (state, no) = station();
    let (status, body) = put(&state, no, page_body(revision(&state, no), 0.3), Some(super::LOCKED_DEVICE)).await;
    assert_eq!(status, StatusCode::LOCKED, "{body}");
    assert!(body["message"].as_str().unwrap().contains("E-LOCK-001"), "{body}");
    assert!(!state.live_screens.read().unwrap().get(no).unwrap().windows.contains_key(&3));
    // Another device (or `?device=`) of the same user is fine.
    let rev = revision(&state, no);
    let (status, body) = put(&state, no, page_body(rev, 0.3), Some("tab-2")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// R1-68: after a panic while the screen store was held (a poisoned lock),
/// the screen endpoints still answer instead of panicking on every request.
#[tokio::test]
async fn a_poisoned_screen_store_still_answers() {
    let (state, no) = station();
    let screens = state.live_screens.clone();
    let _ = std::thread::spawn(move || {
        let _held = screens.write().unwrap();
        panic!("a panic while the screen store is held");
    })
    .join();
    assert!(state.live_screens.is_poisoned());
    for uri in ["/api/screens".to_string(), format!("/api/screens/{no}")] {
        let response = build_router(state.clone())
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{uri}");
        let _ = response.into_body().collect().await.unwrap();
    }
}
