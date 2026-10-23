// ██╗     ██╗███╗   ██╗███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██║     ██║████╗  ██║██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║     ██║██╔██╗ ██║█████╗         ██║   █████╗  ███████╗   ██║   ███████╗
// ██║     ██║██║╚██╗██║██╔══╝         ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ███████╗██║██║ ╚████║███████╗       ██║   ███████╗███████║   ██║   ███████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: line tests

//! Tests of `line`.
//!
//! Owns:          the unit tests of line.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §21 "6d", §34 25.14.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn line_defaults() {
    let rig = Uuid::new_v4();
    let out = Line::new(Some("Main".into()), Direction::Out, "Sacn", rig);
    assert_eq!(out.target, None);
    assert!(out.settings.is_empty(), "a new line takes its protocol's defaults");
    assert_eq!(out.network, rig);
    assert!(out.enabled);
    assert_eq!(out.node_type(), "Line");
    assert_eq!((out.mode, out.local_universe, out.amount), (Direction::Out, 1, 0), "a new line carries no universes");
    assert_eq!(out.run_label(), "—");
    let osc = Line::new(None, Direction::In, "Osc", rig);
    assert_eq!(osc.mode, Direction::In);
    assert_eq!(osc.input_priority, InputPriority::Ltp);
}

#[test]
fn a_line_is_a_run_of_slots_on_the_wire() {
    let mut line = Line::new(None, Direction::Out, "Sacn", Uuid::new_v4());
    (line.local_universe, line.amount, line.universe) = (5, 3, 101);
    assert_eq!(line.pairs().collect::<Vec<_>>(), vec![(5, 101), (6, 102), (7, 103)]);
    assert_eq!(line.run_label(), "5–7 → 101–103");
    line.mode = Direction::In;
    line.amount = 1;
    assert_eq!(line.run_label(), "5 ← 101");
    assert_eq!(serde_json::to_value(Direction::In).unwrap(), "Input");
    assert_eq!(Direction::parse("output"), Some(Direction::Out));
}

#[test]
fn bands_and_modes_parse_and_round_trip() {
    assert_eq!(InputPriority::parse("ltp"), Some(InputPriority::Ltp));
    assert_eq!(InputPriority::parse("Super"), Some(InputPriority::Super));
    assert!(InputPriority::Super < InputPriority::Lowest);
    assert_eq!(
        serde_json::to_string(&InputPriority::Ltp).unwrap(),
        "\"LTP\""
    );
    assert_eq!(MergeMode::parse("lowtp"), Some(MergeMode::LowTp));
    assert_eq!(MergeMode::parse("x"), None);
}
