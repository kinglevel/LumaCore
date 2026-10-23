// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests
//! Tests of `registry`: a crate enters once.
//!
//! Owns:          the unit tests of registry.rs
//! Doesn't own:   the checks (catalog/tests.rs)
//! Entry points:  the #[test] fns
//! Decisions:     §34 21

use crate::Registry;

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[test]
fn a_crate_enters_once() {
    let mut r = Registry::new();
    assert!(r.enter("lumacore-a"));
    assert!(r.enter("lumacore-b"));
    assert!(!r.enter("lumacore-a"));
    assert_eq!(r.crates, ["lumacore-a", "lumacore-b"]);
}

/// Rows of a type declared above the registry come back by their type, in order, and a crate's
/// check runs at freeze (decisions §34 22.7).
#[test]
fn rows_come_back_by_type_and_checks_run_at_freeze() {
    struct Row(&'static str);
    struct Other;
    static FIRST: [Row; 2] = [Row("a"), Row("b")];
    static SECOND: [Row; 1] = [Row("c")];
    static OTHER: [Other; 1] = [Other];
    let mut r = Registry::new();
    r.rows(&FIRST).rows(&OTHER).rows(&SECOND);
    let catalog = r.freeze().unwrap();
    let names: Vec<&str> = catalog.rows::<Row>().map(|r| r.0).collect();
    assert_eq!(names, ["a", "b", "c"]);
    assert_eq!(catalog.rows::<Other>().count(), 1);
    assert_eq!(catalog.rows::<u8>().count(), 0);

    fn no_c(c: &crate::Catalog) -> Vec<String> {
        c.rows::<Row>().filter(|r| r.0 == "c").map(|_| "row c: not allowed".to_string()).collect()
    }
    let mut r = Registry::new();
    r.rows(&FIRST).rows(&SECOND).check(no_c).check(no_c);
    let wrong = r.freeze().unwrap_err();
    assert_eq!(wrong.0, ["row c: not allowed"], "a check added twice runs once");
}
