// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `patch_draft`: the cell-by-cell rebase of a draft onto the live devices.
//!
//! Owns:          the unit tests of patch_draft.rs.
//! Doesn't own:   the claim and live-edit lines (lumacore-objects' patch_draft_tests).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 "34g4 patch hardened" 8.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::rebase;
use serde_json::json;

#[test]
fn untouched_cells_follow_the_live_side_and_both_changes_land() {
    let base = json!({"f": {"1": {"name": "A", "addr": 1}, "2": {"name": "B", "addr": 9}}});
    // The holder re-addressed 1; live renamed 1 and added 3.
    let ours = json!({"f": {"1": {"name": "A", "addr": 5}, "2": {"name": "B", "addr": 9}}});
    let theirs = json!({"f": {"1": {"name": "Spot", "addr": 1}, "2": {"name": "B", "addr": 9}, "3": {"name": "C", "addr": 20}}});
    let out = rebase(&base, &ours, &theirs).unwrap();
    assert_eq!(out, json!({"f": {"1": {"name": "Spot", "addr": 5}, "2": {"name": "B", "addr": 9}, "3": {"name": "C", "addr": 20}}}));
}

#[test]
fn a_cell_both_changed_is_a_conflict() {
    let base = json!({"f": {"1": {"addr": 1}}});
    let ours = json!({"f": {"1": {"addr": 5}}});
    let theirs = json!({"f": {"1": {"addr": 7}}});
    assert_eq!(rebase(&base, &ours, &theirs), Err(vec!["f.1.addr".to_string()]));
}

#[test]
fn what_the_holder_deleted_stays_deleted_unless_live_changed_it() {
    let base = json!({"f": {"1": {"addr": 1}, "2": {"addr": 2}}});
    let ours = json!({"f": {"2": {"addr": 2}}});
    assert_eq!(rebase(&base, &ours, &base).unwrap(), ours);
    let theirs = json!({"f": {"1": {"addr": 3}, "2": {"addr": 2}}});
    assert!(rebase(&base, &ours, &theirs).is_err(), "edited live, deleted in the draft");
}
