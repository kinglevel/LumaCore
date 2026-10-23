//! 12d part 4 (decisions §30 "12d part 4 built"): the station table in the
//! show (wiring rows, pre-configured, multi-station `Set`, applied by each
//! station, undone, saved), the station asks (Online Off, HoldOutput On,
//! Leave, Claim, deleting lines / networks / stations / several rows),
//! `Claim LumaSession Station "X"`, Settings › Assets / About — through real
//! dispatch.

use super::dispatch;
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        other => panic!("'{line}' did not produce output: {}", kind(&other)),
    }
}

fn kind(r: &DispatchResult) -> &'static str {
    match r {
        DispatchResult::Ask(_) => "an ask",
        DispatchResult::Output(_) => "output",
        _ => "something else",
    }
}

fn station() -> AppState {
    let mut s = AppState::new();
    s.local_settings.station_id = "0b7c8a2e-5d4f-4c3b-9a1e-2f3d4c5b6a7e".into();
    s.root.station_settings.station_name = "FOH".into();
    run(&mut s, "Store Network \"Main\"");
    s
}

#[test]
fn settings_assets_and_about() {
    let mut s = station();
    run(&mut s, "Store AssetRoot \"Video\"");
    let out = run(&mut s, "Set AssetRoot \"Video\" \"Folder\" \"/Volumes/Media\"");
    assert!(out.contains("/Volumes/Media"), "{out}");
    assert_eq!(s.root.station_settings.asset_roots.get("video").map(String::as_str), Some("/Volumes/Media"));
    assert!(run(&mut s, "Set AssetRoot 1 \"Folder\" None").contains("unmapped"));
}
