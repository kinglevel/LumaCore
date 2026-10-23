// ██╗     ██╗███╗   ██╗███████╗███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██║     ██║████╗  ██║██╔════╝██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║     ██║██╔██╗ ██║█████╗  ███████╗       ██║   █████╗  ███████╗   ██║   ███████╗
// ██║     ██║██║╚██╗██║██╔══╝  ╚════██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ███████╗██║██║ ╚████║███████╗███████║       ██║   ███████╗███████║   ██║   ███████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: lines tests

//! Tests of `lines`.
//!
//! Owns:          the unit tests of lines.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §17, §31, §33 21.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn stored_lines_get_full_words_and_quotes_stay() {
    use lumacore_objects::commands::registry::full_words;
    assert_eq!(full_words("Stor Grou 3"), "Store Group 3");
    assert_eq!(
        full_words("Sto 3"),
        "Sto 3",
        "a registered alias is an exact name"
    );
    assert_eq!(
        full_words("Fixtu 1 Thru 4; At 50"),
        "Fixture 1 Thru 4; At 50"
    );
    assert_eq!(
        full_words("Label Group 1 \"sto; gro\" /Overwrite"),
        "Label Group 1 \"sto; gro\" /Overwrite"
    );
    // A `;` inside quotes isn't a statement break.
    assert!(check_line("Label Group 1 \"a; Frobnicate\"").is_empty());
}

#[test]
fn unknown_words_are_errors_and_planned_ones_warnings() {
    assert!(check_line("List Fixture; At 50").is_empty());
    assert!(matches!(check_line("Frobnicate 3").as_slice(), [Err(_)]));
}
