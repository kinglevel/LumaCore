//! The one resolver against the demo show (the rig: fixtures 1 Thru 9999 with gaps, 77 groups
//! — 1 "Photon", 2 "Icarus", 3 "Blinder" … — in DataPool 1 "Default"; decisions §40 14).

use super::*;
use crate::commands::groups::GROUP;
use crate::objects::{DATAPOOL, FIXTURE};

fn demo() -> AppState {
    crate::demo::fresh_state()
}

fn nos(state: &AppState, object: &'static ObjectDef, src: &str) -> Result<Vec<u32>, String> {
    resolve_one(state, &target_of(src)?, object).map(|r| r.numbers())
}

#[test]
fn the_one_range_rule() {
    let s = demo();
    assert_eq!(nos(&s, &FIXTURE, "Fixture 101 Thru 104").unwrap(), [101, 102, 103, 104]);
    assert_eq!(nos(&s, &FIXTURE, "Fixture 104 Thru 101").unwrap(), [104, 103, 102, 101], "typed order");
    let fids: Vec<u32> = s.root.show_data.devices.dmx.patch.fids().into_iter().map(|(n, _)| n).collect();
    let from = |n: u32| fids.iter().copied().filter(|f| *f >= n).collect::<Vec<_>>();
    let to = |n: u32| fids.iter().copied().filter(|f| *f <= n).collect::<Vec<_>>();
    assert_eq!(nos(&s, &FIXTURE, "Fixture 8000 Thru").unwrap(), from(8000), "to the last");
    assert_eq!(nos(&s, &FIXTURE, "Fixture Thru 5").unwrap(), to(5), "from the first");
    assert_eq!(nos(&s, &FIXTURE, "Fixture 101 Thru 106 - 103 + 103").unwrap(), [101, 102, 104, 105, 106, 103]);
    assert_eq!(nos(&s, &FIXTURE, "Fixture 9990 Thru 9999").unwrap(), [9996, 9999], "missing numbers are skipped");
    assert_eq!(nos(&s, &FIXTURE, "Fixture 101 + 101").unwrap(), [101], "no duplicates");
    assert!(nos(&s, &FIXTURE, "Fixture 99999").unwrap_err().contains("E-CMD-007"));
}

#[test]
fn names_match_ignoring_case_and_bare_ids_take_the_given_object() {
    let s = demo();
    assert_eq!(nos(&s, &GROUP, "Group \"icarus\"").unwrap(), [2]);
    assert_eq!(nos(&s, &GROUP, "Group \"Blinder\" + 1").unwrap(), [3, 1]);
    assert_eq!(nos(&s, &GROUP, "2 + 3").unwrap(), [2, 3], "bare ids are the verb's object");
    assert!(nos(&s, &GROUP, "Group \"A\" Thru 3").unwrap_err().contains("E-CMD-008"));
}

#[test]
fn owners_and_tree_paths() {
    let s = demo();
    let r = resolve_one(&s, &target_of("DataPool 1 Group 2").unwrap(), &GROUP).unwrap();
    assert_eq!((r.datapool, r.numbers()), (Some(1), vec![2]));
    let r = resolve_one(&s, &target_of("DataPool \"default\" Group 3").unwrap(), &GROUP).unwrap();
    assert_eq!(r.numbers(), [3]);
    assert!(resolve_one(&s, &target_of("DataPool 9 Group 3").unwrap(), &GROUP).unwrap_err().contains("E-CMD-008"));
    let r = resolve_one(&s, &target_of("Root.ShowData.DataPools.1.Groups.3").unwrap(), &GROUP).unwrap();
    assert_eq!((r.object.keyword, r.numbers()), ("Group", vec![3]));
    let all = resolve_one(&s, &target_of("Root.ShowData.DataPools.1.Groups").unwrap(), &GROUP).unwrap();
    let every: Vec<u32> = s.user_groups().keys().copied().collect();
    assert!(!all.named && all.numbers() == every, "a collection names the kind");
    let pools = resolve_one(&s, &target_of("DataPool").unwrap(), &DATAPOOL).unwrap();
    assert!(!pools.named && pools.numbers() == [1]);
}

#[test]
fn members_carry_their_guid_and_name() {
    let s = demo();
    let r = resolve_one(&s, &target_of("Group 2").unwrap(), &GROUP).unwrap();
    assert_eq!(r.members[0].name.as_deref(), Some("Icarus"));
    assert_eq!(r.members[0].guid, Some(s.root.show_data.datapools[&1].groups[&2].meta.identity.guid));
}

#[test]
fn fixtures_join_groups_and_fixtures_in_order() {
    let s = demo();
    let odd = fixture_words(&s, &["Group", "2"]).unwrap();
    assert!(!odd.is_empty());
    let all = fixture_words(&s, &["Group", "1"]).unwrap();
    let even: Vec<u32> = all.iter().copied().filter(|f| !odd.contains(f)).collect();
    assert_eq!(fixture_words(&s, &["Group", "1", "-", "Group", "2"]).unwrap(), even);
    assert_eq!(fixture_words(&s, &["101", "Thru", "103"]).unwrap(), [101, 102, 103], "bare ids are fixtures");
    assert_eq!(fixture_words(&s, &["Fixture", "106", "+", "Fixture", "101"]).unwrap(), [106, 101]);
    assert!(fixture_words(&s, &["Sequence", "1"]).unwrap_err().contains("E-CMD-008"));
    assert!(fixture_words(&s, &["Fixture", "99999"]).unwrap_err().contains("E-CMD-007"));
}

#[test]
fn one_kind_at_a_time() {
    let s = demo();
    assert!(resolve_one(&s, &target_of("Group 1 + Fixture 101").unwrap(), &GROUP).unwrap_err().contains("E-CMD-008"));
}

