// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The armed tag on the rig (decisions 0297): which lines are armed, the tag's colour and what
//! carries it.
//!
//! Owns:          the tests of armed.rs.
//! Doesn't own:   the tap rules' tests (lumacore-terminal).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0297-tag-squares-and-picker.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::AppState;

fn run(state: &mut AppState, line: &str) -> String {
    match lumacore_objects::commands::dispatch(state, line) {
        lumacore_objects::DispatchResult::Output(r) => String::from(r),
        _ => String::new(),
    }
}

#[test]
fn only_assign_or_unassign_tag_path_at_is_armed() {
    let p = |t: &str| lumacore_tags::TagPath::parse(t).unwrap();
    assert_eq!(crate::arming("Assign Tag 1.2 At"), Some((true, p("1.2"))));
    assert_eq!(crate::arming("  unassign tag 3 at "), Some((false, p("3"))));
    for line in ["Assign Tag 1.2", "Assign Tag 1.2 At Group 3", "Assign Group 2 At", "Store Tag 1 At", "Assign Tag 1.02 At", "Assign Tag \"FOH\" At", ""] {
        assert_eq!(crate::arming(line), None, "{line}");
    }
}

#[test]
fn armed_names_the_tag_its_colour_and_its_carriers() {
    let mut s = lumacore_objects::demo::fresh_state();
    run(&mut s, "Store Tag 1 \"FOH\"");
    run(&mut s, "Set Tag 1 \"Paint\" Blue");
    run(&mut s, "Assign Tag 1 At Group 3 + Sequence 1");
    let a = crate::armed(&s, "Assign Tag 1 At").expect("armed");
    assert!(a.assign);
    assert_eq!((a.path.as_str(), a.colour.starts_with('#'), a.carriers.len()), ("1", true, 2));
    assert!(crate::armed(&s, "Assign Tag 77 At").is_none(), "no such tag");
}
