// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `tokens`.
//!
//! Owns:          the unit tests of tokens.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §15 "Quoting", §33 91, §34 15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn texts(line: &str) -> Vec<String> {
    words(&lex(line).unwrap())
}

#[test]
fn words_and_quotes() {
    assert_eq!(
        texts(r#"Patch "My Fixture" At 1.001"#),
        ["Patch", "My Fixture", "At", "1.001"]
    );
    assert_eq!(texts("Fixture 1+3"), ["Fixture", "1", "+", "3"]);
    // A quoted word is a value, an empty one too (decisions §33 7).
    assert_eq!(texts(r#"Label Fixture 1 """#), ["Label", "Fixture", "1", ""]);
}

#[test]
fn spans_point_into_the_line() {
    let line = r#"Store Group 1 "Front Wash""#;
    let tokens = lex(line).unwrap();
    assert_eq!(&line[tokens[3].start..tokens[3].end], r#""Front Wash""#);
    assert_eq!(tokens[3].kind, Kind::Word { quoted: true });
}

#[test]
fn unterminated_quote_is_an_error_at_the_quote() {
    let err = lex(r#"Label Group 1 "Front"#).unwrap_err();
    assert_eq!(err.at, 14);
    assert!(
        err.render(r#"Label Group 1 "Front"#)
            .ends_with(&format!("{}^", " ".repeat(14)))
    );
}

#[test]
fn statements_split_outside_quotes_only() {
    assert_eq!(statements("list; help"), ["list", "help"]);
    assert_eq!(
        statements(r#"echo "a; b"; list"#),
        [r#"echo "a; b""#, "list"]
    );
    assert_eq!(statements(""), [""]);
}

#[test]
fn word_start_includes_an_open_quote() {
    let line = r#"Group "Front W"#;
    assert_eq!(word_start(line, line.len()), 6);
    assert_eq!(word_start("Store Gr", 8), 6);
    assert_eq!(word_start("Store ", 6), 6);
}
