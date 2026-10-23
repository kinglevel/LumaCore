//! Relative numbers on the station (decisions §30 "12b"): `At + 10`,
//! `At * 2`, `Set … + 3`; a value that isn't one is refused (E-CMD-008),
//! never read as 0 %.

use super::dispatch;
use crate::AppState;
use crate::DispatchResult;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn dimmer(state: &AppState) -> f64 {
    let guid = state.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").meta.identity.guid;
    state
        .current_programmer()
        .and_then(|p| p.get_value(guid, "Dimmer"))
        .and_then(lumacore_base::Value::percent)
        .unwrap()
}

fn with_fixture() -> AppState {
    let mut state = AppState::new();
    let fixture = lumacore_fixtures::Fixture::new(
        None,
        uuid::Uuid::new_v4(),
        uuid::Uuid::nil(),
        uuid::Uuid::nil(),
        uuid::Uuid::nil(),
        1,
        1,
    );
    state.root.show_data.devices.dmx.patch.insert_fid(1, fixture);
    run(&mut state, "Fixture 1");
    state
}

#[test]
fn at_takes_relative_values_and_refuses_nonsense() {
    let mut state = with_fixture();
    run(&mut state, "At 40");
    run(&mut state, "At + 10");
    assert_eq!(dimmer(&state), 50.0);
    run(&mut state, "At - 20");
    assert_eq!(dimmer(&state), 30.0);
    run(&mut state, "At * 2");
    assert_eq!(dimmer(&state), 60.0);
    run(&mut state, "At / 3");
    assert_eq!(dimmer(&state), 20.0);
    run(&mut state, "At * 10");
    assert_eq!(dimmer(&state), 100.0, "kept in 0–100 %");
    // Nonsense was 0 % before: now it is refused and nothing changes.
    for bad in ["At abc", "At + ", "At / 0"] {
        assert!(run(&mut state, bad).contains("E-CMD-008"), "{bad}");
        assert_eq!(dimmer(&state), 100.0, "{bad} changed nothing");
    }
}

#[test]
fn set_on_a_number_takes_relative_values() {
    let mut state = AppState::new();
    run(&mut state, "Store UserProfile 2 \"LD\"");
    run(&mut state, "Set UserProfile 2 LockScreen \"AutoLock\" 10");
    assert!(run(&mut state, "Set UserProfile 2 LockScreen \"AutoLock\" + 5").contains("= 15"));
    assert!(run(&mut state, "Set UserProfile 2 LockScreen \"AutoLock\" * 2").contains("= 30"));
    assert!(run(&mut state, "Set UserProfile 2 LockScreen \"AutoLock\" - 10").contains("= 20"));
    // `-20` is a negative value, not relative (the adjusters go below 0).
    assert!(run(&mut state, "Set UserProfile 2 Theme \"Brightness\" -20").contains("= -20"));
    assert!(run(&mut state, "Set UserProfile 2 LockScreen \"AutoLock\" / 0").contains("E-CMD-008"));
}

/// R1-06: NaN and infinity are never stored — spaced, glued, on a master or
/// a group cell (serde_json saves them as `null`, and the show won't load).
#[test]
fn non_finite_numbers_are_refused_everywhere() {
    let mut state = with_fixture();
    run(&mut state, "At 40");
    for bad in ["At + nan", "At * inf", "At +NaN", "At *inf", "At /0", "At -infinity"] {
        assert!(run(&mut state, bad).contains("E-CMD-008"), "{bad}");
        assert_eq!(dimmer(&state), 40.0, "{bad} changed nothing");
    }
    // Not a number: an unknown word to the typed Master (E-SYN-001), refused either way.
    for bad in ["Master 2.1 At nan", "Master 2.1 At inf"] {
        let out = run(&mut state, bad);
        assert!(out.contains("E-SYN-001") || out.contains("E-CMD-008"), "{bad}: {out}");
    }
    assert_eq!(state.root.show_data.playbacks.live.masters.grand.master, 1.0);
    run(&mut state, "Store Group 1");
    let out = run(&mut state, "Set Group 1 Fixture 1 \"Rotation\" \"nan\"");
    assert!(out.contains("degrees"), "{out}");
    let cells = state.root.show_data.datapools.values().flat_map(|d| d.groups.values()).flat_map(|g| &g.members);
    assert!(cells.into_iter().all(|m| m.cell.rotation.is_finite()), "no NaN in a group cell");
}

/// R1-09: a Label range visits only the objects that exist; the missing
/// numbers are named in one line, never looped over one by one.
#[test]
fn a_huge_label_range_visits_only_what_exists() {
    let mut state = with_fixture();
    let started = std::time::Instant::now();
    let out = crate::text::strip_ansi(&run(&mut state, "Label Fixture 1 Thru 4294967295 \"Spot\""));
    assert!(started.elapsed() < std::time::Duration::from_secs(1), "{:?}", started.elapsed());
    // The one range rule (§33 57): only existing numbers are visited, the missing ones skipped.
    assert!(out.contains("Fixture 1 labeled") && !out.contains("Fixture 2"), "{out}");
    let name = state.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").meta.identity.name.clone();
    assert_eq!(name.as_deref(), Some("Spot"));
}

/// R1-35: `Label Fixture 1 Thru 5` without a name is no longer the name
/// "Thru 5" for fixture 1: it needs a name or `/Clear`.
#[test]
fn a_label_range_without_a_name_needs_clear() {
    let mut state = with_fixture();
    run(&mut state, "Label Fixture 1 \"Spot\"");
    let out = run(&mut state, "Label Fixture 1 Thru 5");
    assert!(out.contains("E-CMD-006"), "{out}");
    let name = |s: &AppState| s.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").meta.identity.name.clone();
    assert_eq!(name(&state).as_deref(), Some("Spot"));
    run(&mut state, "Label Fixture 1 Thru 5 /Clear");
    assert_eq!(name(&state), None);
}

/// R1-12 (Q2): out-of-range values are clamped, never refused or
/// stored raw; a move never wraps past u32::MAX or lands on 0.
#[test]
fn values_clamp_and_moves_stay_in_range() {
    let mut state = with_fixture();
    run(&mut state, "At 150");
    assert_eq!(dimmer(&state), 100.0);
    run(&mut state, "At -5");
    assert_eq!(dimmer(&state), 0.0);
    for bad in ["Move Fixture 1 At 0", "Move Fixture 1 Thru 3 At 4294967295"] {
        assert!(run(&mut state, bad).contains("E-CMD-008"), "{bad}");
    }
    assert!(state.root.show_data.devices.dmx.patch.has_fid(1));
}
