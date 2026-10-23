// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `door`: next free, spacing, copy with new GUIDs, the swap, Insert's shift, Compact,
//! Delete naming what it leaves broken.
//!
//! Owns:          the unit tests of door.rs (on a Group pool declared here).
//! Doesn't own:   line-in / reply-out tests (the objects crate's verbs).
//! Entry points:  the #[test] fns.
//! Decisions:     §35.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use lumacore_tree::{Datapool, Group};

fn groups(s: &mut AppState, dp: Option<u32>) -> Option<&mut BTreeMap<u32, Group>> {
    let dp = dp?;
    Some(&mut s.root.show_data.datapools.entry(dp).or_insert_with(|| Datapool::new(None)).groups)
}

fn groups_ref(s: &AppState, dp: Option<u32>) -> Option<&BTreeMap<u32, Group>> {
    s.root.show_data.datapools.get(&dp?).map(|d| &d.groups)
}

static POOL: BTreePool<Group> = BTreePool {
    object: "Group",
    in_datapool: true,

    max: crate::pools::MAX_NO,
    map: groups,
    map_ref: groups_ref,
    name: |g| g.meta.identity.name.clone(),
    identify: |g, guid| g.meta.identity.guid = guid,
    rules: PoolRules {
        can_delete: |_, g| if g.meta.identity.name.as_deref() == Some("Locked") { Err("Locked stays.".into()) } else { Ok(()) },
        deleted: |_, _| {},
        changed: |_| {},
        merge: Some(|into, from| {
            let n = into.merge_members(&from.members, true);
            format!("{n} added")
        }),
        orders: &[],
    },
};

fn state(numbers: &[(u32, &str)]) -> AppState {
    let mut s = AppState::new();
    for &(n, name) in numbers {
        groups(&mut s, Some(1)).unwrap().insert(n, Group::new(Some(name.into())));
    }
    s
}

fn names(s: &AppState) -> Vec<(u32, String)> {
    groups_ref(s, Some(1)).unwrap().iter().map(|(n, g)| (*n, g.meta.identity.name.clone().unwrap_or_default())).collect()
}

fn guid(s: &AppState, n: u32) -> Uuid {
    groups_ref(s, Some(1)).unwrap()[&n].meta.identity.guid
}

#[test]
fn next_free_is_the_first_hole_and_a_block_keeps_its_spacing() {
    assert_eq!(next_free(&[1, 2, 4]), 3);
    assert_eq!(next_free(&[]), 1);
    assert_eq!(next_free_block(&[1, 2, 4, 6], &[0, 1]), Some(7));
    assert_eq!(next_free_block(&[2], &[0, 2]), Some(1));
    assert_eq!(spacing(&[1, 3, 4], 10).unwrap(), vec![10, 12, 13]);
    assert!(spacing(&[1], 0).is_err());
}

#[test]
fn a_copy_gets_new_guids_and_an_existing_target_asks_first() {
    let mut s = state(&[(1, "A"), (2, "B"), (5, "E")]);
    let out = copy(&POOL, &mut s, Some(1), &[1, 2], Some(1), &[4, 5], OnExisting::Ask);
    assert_eq!(out, Err(Exists { taken: vec![5] }));
    assert_eq!(names(&s).len(), 3, "an ask writes nothing");
    let e_guid = guid(&s, 5);
    copy(&POOL, &mut s, Some(1), &[1, 2], Some(1), &[4, 5], OnExisting::Overwrite).unwrap();
    assert_eq!(names(&s), vec![(1, "A".into()), (2, "B".into()), (4, "A".into()), (5, "B".into())]);
    assert_ne!(guid(&s, 4), guid(&s, 1));
    assert_eq!(guid(&s, 5), e_guid, "an overwrite keeps the target's GUID");
}

#[test]
fn overlapping_copies_read_every_source_first() {
    let mut s = state(&[(1, "A"), (2, "B"), (3, "C")]);
    copy(&POOL, &mut s, Some(1), &[1, 2, 3], Some(1), &[2, 3, 4], OnExisting::Overwrite).unwrap();
    assert_eq!(names(&s).into_iter().map(|(_, n)| n).collect::<Vec<_>>(), vec!["A", "A", "B", "C"]);
}

#[test]
fn a_move_onto_a_taken_number_swaps_and_keeps_guids() {
    let mut s = state(&[(3, "C"), (7, "G"), (8, "H")]);
    let (c, g) = (guid(&s, 3), guid(&s, 7));
    relocate(&POOL, &mut s, Some(1), &[3], Some(1), &[7]);
    assert_eq!(names(&s), vec![(3, "G".into()), (7, "C".into()), (8, "H".into())]);
    assert_eq!((guid(&s, 7), guid(&s, 3)), (c, g));
}

#[test]
fn a_range_moves_slot_by_slot_free_moves_taken_swaps() {
    let mut s = state(&[(1, "A"), (2, "B"), (3, "C"), (5, "E")]);
    // 1 → 4 (free), 2 → 5 (taken: E swaps into a freed slot).
    relocate(&POOL, &mut s, Some(1), &[1, 2], Some(1), &[4, 5]);
    assert_eq!(names(&s), vec![(1, "E".into()), (3, "C".into()), (4, "A".into()), (5, "B".into())]);
    // Overlap: 3, 4, 5 → 4, 5, 6.
    relocate(&POOL, &mut s, Some(1), &[3, 4, 5], Some(1), &[4, 5, 6]);
    assert_eq!(names(&s), vec![(1, "E".into()), (4, "C".into()), (5, "A".into()), (6, "B".into())]);
}

#[test]
fn a_move_between_datapools_keeps_the_guid() {
    let mut s = state(&[(3, "C")]);
    groups(&mut s, Some(2)).unwrap().insert(8, Group::new(Some("X".into())));
    let c = guid(&s, 3);
    relocate(&POOL, &mut s, Some(1), &[3], Some(2), &[8]);
    assert_eq!(groups_ref(&s, Some(2)).unwrap()[&8].meta.identity.guid, c);
    assert_eq!(names(&s), vec![(3, "X".into())], "the one in the way swaps into the freed slot");
}

#[test]
fn insert_shifts_up_to_the_first_hole() {
    let mut s = state(&[(5, "E"), (9, "I"), (10, "J"), (12, "L")]);
    insert(&POOL, &mut s, Some(1), &[5], Some(1), 9).unwrap();
    assert_eq!(names(&s), vec![(9, "E".into()), (10, "I".into()), (11, "J".into()), (12, "L".into())]);
}

#[test]
fn compact_closes_the_holes_in_order_or_by_name() {
    let mut s = state(&[(2, "b"), (5, "a"), (9, "c")]);
    let b = guid(&s, 2);
    compact(&POOL, &mut s, Some(1), None, &Order::Number).unwrap();
    assert_eq!(names(&s), vec![(1, "b".into()), (2, "a".into()), (3, "c".into())]);
    assert_eq!(guid(&s, 1), b);
    compact(&POOL, &mut s, Some(1), None, &Order::Name).unwrap();
    assert_eq!(names(&s), vec![(1, "a".into()), (2, "b".into()), (3, "c".into())]);
    assert!(compact(&POOL, &mut s, Some(1), None, &Order::By("Address".into())).is_err());
}

#[test]
fn compact_of_some_rows_starts_at_their_first() {
    let mut s = state(&[(1, "a"), (10, "b"), (14, "c"), (20, "d")]);
    compact(&POOL, &mut s, Some(1), Some(&[10, 14]), &Order::Number).unwrap();
    assert_eq!(names(&s).into_iter().map(|(n, _)| n).collect::<Vec<_>>(), vec![1, 10, 11, 20]);
}

#[test]
fn delete_refuses_what_its_pool_keeps() {
    let mut s = state(&[(1, "Locked"), (2, "B")]);
    let out = delete(&POOL, &mut s, Some(1), &[1, 2]);
    assert_eq!(out, vec!["Locked stays.".to_string(), "Group 2 deleted.".to_string()]);
    assert_eq!(names(&s), vec![(1, "Locked".into())]);
}
