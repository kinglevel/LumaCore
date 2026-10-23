// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `root`: List / Info Root by number and name, the default object, the refusals.
//!
//! Owns:          the unit tests of root.rs.
//! Doesn't own:   the tree's numbering (tree_path_tests), Move Tag At Root (cmd-tags).
//! Entry points:  the #[test] fns.
//! Decisions:     §33 53, 54.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use crate::commands::dispatch;
use crate::tree_path::Node;
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => String::new(),
    }
}

#[test]
fn list_root_numbers_the_top() {
    let mut s = AppState::new();
    let out = run(&mut s, "List Root");
    let show = Node::ShowData.no().unwrap();
    assert!(out.starts_with("Root (Root)"), "{out}");
    assert!(out.contains(&format!("{show:>3}  ShowData")), "{out}");
}

#[test]
fn a_number_or_a_name_lists_that_node() {
    let mut s = AppState::new();
    let (show, devices) = (Node::ShowData.no().unwrap(), Node::Devices.no().unwrap());
    let by_no = run(&mut s, &format!("Root {show}.{devices}"));
    assert!(by_no.starts_with(&format!("Root.ShowData.Devices (Root {show}.{devices})")), "{by_no}");
    let by_list = run(&mut s, &format!("List Root {show}.{devices}"));
    assert_eq!(by_no, by_list, "the default verb is List");
    let by_name = run(&mut s, "List Root \"ShowData\"");
    assert!(by_name.starts_with("Root.ShowData"), "{by_name}");
    let info = run(&mut s, &format!("Info Root {show}"));
    assert!(info.contains("under Root"), "{info}");
}

#[test]
fn root_refuses_what_it_cant_name() {
    let mut s = AppState::new();
    assert!(run(&mut s, "List Root 99").contains("E-CMD-007"));
    let drives = Node::Drives.no().unwrap();
    assert!(run(&mut s, &format!("List Root {drives}")).contains("E-CMD-010"));
    assert!(run(&mut s, "List Root 1 Thru 3").contains("E-CMD-008"));
}

#[test]
fn root_alone_is_the_default_object() {
    let mut s = AppState::new();
    run(&mut s, "Root");
    let show = Node::ShowData.no().unwrap();
    let out = run(&mut s, &show.to_string());
    assert!(out.starts_with("Root.ShowData"), "{out}");
}

/// A number is the child's No where `cd` / `ls` resolve a target (decisions §33 54, a
/// ChangeDestination); a collection's numbers stay its entries'.
#[test]
fn navigation_follows_the_numbers() {
    use crate::navigation::Navigation;
    let mut s = AppState::new();
    let (show, devices) = (Node::ShowData.no().unwrap(), Node::Devices.no().unwrap());
    let at = |s: &AppState, t: &str| s.resolve_target(t).map(|p| crate::tree_path::dot(&p));
    assert_eq!(at(&s, &show.to_string()).as_deref(), Ok("Root.ShowData"));
    assert_eq!(at(&s, &format!("Root.{show}.{devices}")).as_deref(), Ok("Root.ShowData.Devices"));
    s.navigation_path = vec!["ShowData".into()];
    assert_eq!(at(&s, &devices.to_string()).as_deref(), Ok("Root.ShowData.Devices"));
}
