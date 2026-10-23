// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `universe`.
//!
//! Owns:          the unit tests of universe.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §34 25.10.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn universe_new_all_slots_free() {
    let u = Universe::new(Some("Main".to_string()));
    assert_eq!(u.occupied_count(), 0);
    assert_eq!(u.free_count(), 512);
    assert!(u.is_slot_free(1));
    assert!(u.is_slot_free(512));
}

#[test]
fn universe_out_of_range_address_not_free() {
    let u = Universe::new(None);
    assert!(!u.is_slot_free(0));
    assert!(!u.is_slot_free(513));
}

#[test]
fn universe_mark_occupied_and_free() {
    let mut u = Universe::new(None);
    let guid = Uuid::new_v4();
    u.mark_occupied(1, 24, guid);
    assert_eq!(u.occupied_count(), 24);
    assert!(!u.is_slot_free(1));
    assert!(!u.is_slot_free(24));
    assert!(u.is_slot_free(25));

    u.mark_free(1, 24);
    assert_eq!(u.occupied_count(), 0);
}

#[test]
fn universe_marks_skip_address_zero_and_past_512() {
    let mut u = Universe::new(None);
    let guid = Uuid::new_v4();
    u.mark_occupied(0, 3, guid); // a loaded break at address 0
    assert_eq!(u.occupied_count(), 2);
    u.mark_occupied(510, 24, guid);
    assert_eq!(u.occupied_count(), 5);
    u.mark_free(0, 3);
    u.mark_free(510, u16::MAX);
    assert_eq!(u.occupied_count(), 0);
}

#[test]
fn universe_release_owned_keeps_other_fixtures_slots() {
    let mut u = Universe::new(None);
    let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
    u.mark_occupied(1, 10, a);
    u.mark_occupied(5, 10, b); // overlaps 5–10 (a loaded show)
    u.release_owned(1, 10, a);
    assert_eq!(u.occupied_count(), 10);
    assert!(!u.is_slot_free(5));
}

#[test]
fn universe_first_free_block_empty_universe() {
    let u = Universe::new(None);
    assert_eq!(u.first_free_block(24), Some(1));
    assert_eq!(u.first_free_block(512), Some(1));
    assert_eq!(u.first_free_block(513), None);
}

#[test]
fn universe_first_free_block_with_occupied_start() {
    let mut u = Universe::new(None);
    let guid = Uuid::new_v4();
    u.mark_occupied(1, 10, guid); // 1-10 occupied
    // First free block of 5 starts at 11
    assert_eq!(u.first_free_block(5), Some(11));
}

#[test]
fn universe_first_free_block_fragmented() {
    let mut u = Universe::new(None);
    let g1 = Uuid::new_v4();
    let g2 = Uuid::new_v4();
    u.mark_occupied(1, 10, g1); // 1-10
    u.mark_occupied(15, 10, g2); // 15-24
    // Free regions: 11-14 (4 slots), 25-512
    assert_eq!(u.first_free_block(4), Some(11));
    assert_eq!(u.first_free_block(5), Some(25));
}
