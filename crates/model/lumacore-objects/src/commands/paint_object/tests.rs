// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! `Paint <target>` on the rig (decisions 0296): the picker's event names the objects and the Set
//! line its answer runs; several rows are one line; no target is the usage.
//!
//! Owns:          the tests of paint_object.rs.
//! Doesn't own:   the picker (kit/paint.js), the Set line's own tests.
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0296-paint-verb-and-theme-slots.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use crate::{AppState, DispatchResult};
use lumacore_wire::UiEvent;

fn run(state: &mut AppState, line: &str) -> String {
    match crate::commands::dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => String::new(),
    }
}

fn picker(state: &mut AppState) -> Option<(String, String, String)> {
    state.ui_requests.drain(..).find_map(|e| match e {
        UiEvent::PaintPicker { object, line, value, .. } => Some((object, line, value)),
        _ => None,
    })
}

#[test]
fn paint_opens_the_picker_on_the_objects_with_their_set_line() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Set Group 1 \"Paint\" Blue");
    let out = run(&mut s, "Paint Group 1");
    assert!(out.contains("Set Group 1 \"Paint\" <value>"), "{out}");
    let (object, line, value) = picker(&mut s).expect("the picker");
    assert_eq!((object.as_str(), line.as_str()), ("Group 1", "Set Group 1 \"Paint\" {value}"));
    assert!(!value.is_empty(), "the Paint now");
    run(&mut s, "Paint Sequence 1");
    assert_eq!(picker(&mut s).map(|p| p.1), Some("Set Sequence 1 \"Paint\" {value}".to_string()));
    run(&mut s, "Paint Group 1 Thru 3");
    assert_eq!(picker(&mut s).map(|p| p.1), Some("Set Group 1 Thru 3 \"Paint\" {value}".to_string()));
    assert!(run(&mut s, "Paint").contains("E-CMD-006"));
    assert!(picker(&mut s).is_none());
}
