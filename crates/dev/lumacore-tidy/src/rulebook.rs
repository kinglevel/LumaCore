// ██████╗ ██╗   ██╗██╗     ███████╗██████╗  ██████╗  ██████╗ ██╗  ██╗
// ██╔══██╗██║   ██║██║     ██╔════╝██╔══██╗██╔═══██╗██╔═══██╗██║ ██╔╝
// ██████╔╝██║   ██║██║     █████╗  ██████╔╝██║   ██║██║   ██║█████╔╝
// ██╔══██╗██║   ██║██║     ██╔══╝  ██╔══██╗██║   ██║██║   ██║██╔═██╗
// ██║  ██║╚██████╔╝███████╗███████╗██████╔╝╚██████╔╝╚██████╔╝██║  ██╗
// ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚══════╝╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝
// FILE: rulebook
//! The rulebook is checked both ways (decisions §34 20): `docs/meta/tidy.md` has one entry per
//! rule of this crate, each rule's scanner tests are named under it, and every ratchet's number
//! in the book is the one in the code — a rule without an entry, an entry without a rule, or a
//! number that moved fails here.
//!
//! Owns:          the rulebook check and the book's line format
//! Doesn't own:   the rules (one file per area), the book's prose
//! Entry points:  the_rulebook_matches_the_rules
//! Decisions:     §34 20

use std::collections::BTreeMap;

use crate::files::{crate_dir, rust_files, workspace};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The book, from the workspace root.
const BOOK: &str = "docs/meta/tidy.md";

/// The lines every entry has.
const FIELDS: [&str; 3] = ["- **Checks:**", "- **Why:**", "- **Escape:**"];

/// What the book says: each rule with its lines.
#[derive(Default)]
struct Book {
    /// Rule name → the entry's lines.
    rules: BTreeMap<String, Vec<String>>,
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The book's entries: `### <rule>` headings and the bullet lines under them.
fn read_book(text: &str) -> Book {
    let mut book = Book::default();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("### ") {
            let name = name.trim().trim_matches('`').to_string();
            book.rules.insert(name.clone(), Vec::new());
            current = Some(name);
        } else if line.starts_with("## ") {
            current = None;
        } else if let Some(name) = &current
            && line.starts_with("- **")
        {
            book.rules.get_mut(name).expect("the entry").push(line.to_string());
        }
    }
    book
}

/// The backticked names on a `- **<field>:**` line.
fn named(line: &str) -> Vec<String> {
    line.split('`').skip(1).step_by(2).map(str::to_string).collect()
}

/// `` `NAME` = 12 `` → (NAME, 12); `<what> = 12` (a Copies line) → (what, 12).
fn number_line(rest: &str) -> Option<(String, usize)> {
    let (what, n) = rest.rsplit_once(" = ")?;
    Some((what.trim().trim_matches('`').to_string(), n.trim().trim_end_matches('.').parse().ok()?))
}

/// This crate's tests, by name, and its sources' text.
fn crate_tests() -> (Vec<String>, String) {
    let mut files = Vec::new();
    rust_files(&crate_dir("lumacore-tidy"), &mut files);
    files.sort();
    let mut tests = Vec::new();
    let mut all = String::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read source");
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "#[test]"
                && let Some(name) = lines.get(i + 1).and_then(|l| l.trim().strip_prefix("fn "))
            {
                tests.push(name.split('(').next().unwrap_or(name).to_string());
            }
        }
        all.push_str(&text);
    }
    (tests, all)
}

/// The ratchet constants in `src` (`const X_AT_MOST: usize = N;`).
fn ratchets(src: &str) -> BTreeMap<String, usize> {
    src.lines()
        .filter_map(|l| l.trim().strip_prefix("const ").or_else(|| l.trim().strip_prefix("pub(crate) const ")))
        .filter_map(|l| {
            let (name, rest) = l.split_once(": usize = ")?;
            name.ends_with("_AT_MOST").then(|| Some((name.to_string(), rest.trim_end_matches(';').parse().ok()?)))?
        })
        .collect()
}

/// The COPIES rows in `src`: what → count.
fn copies(src: &str) -> BTreeMap<String, usize> {
    let Some(table) = src.split("const COPIES: &[(&str, &str, usize)] = &[").nth(1) else { return BTreeMap::new() };
    table
        .split("\n];")
        .next()
        .unwrap_or("")
        .lines()
        .filter_map(|l| {
            let l = l.trim().strip_prefix("(\"")?;
            let (what, rest) = l.split_once("\", ")?;
            let n = rest.trim_end_matches("),").rsplit(", ").next()?.parse().ok()?;
            Some((what.to_string(), n))
        })
        .collect()
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[test]
fn the_rulebook_matches_the_rules() {
    let text = std::fs::read_to_string(workspace().join(BOOK)).expect("docs/meta/tidy.md");
    let book = read_book(&text);
    let (tests, src) = crate_tests();
    let mut wrong = Vec::new();
    let mut self_tests = Vec::new();
    let mut book_ratchets = BTreeMap::new();
    let mut book_copies = BTreeMap::new();
    for (rule, lines) in &book.rules {
        if !tests.contains(rule) {
            wrong.push(format!("{BOOK}: `{rule}` is no test of lumacore-tidy"));
        }
        for field in FIELDS {
            if !lines.iter().any(|l| l.starts_with(field)) {
                wrong.push(format!("{BOOK}: `{rule}` has no `{field}` line"));
            }
        }
        for line in lines {
            if let Some(rest) = line.strip_prefix("- **Self-test:**") {
                self_tests.extend(named(rest));
            } else if let Some(rest) = line.strip_prefix("- **Ratchet:**") {
                match number_line(rest) {
                    Some((name, n)) => drop(book_ratchets.insert(name, n)),
                    None => wrong.push(format!("{BOOK}: `{rule}`: a Ratchet line is `` `NAME` = N ``: {line}")),
                }
            } else if let Some(rest) = line.strip_prefix("- **Copies:**") {
                match number_line(rest) {
                    Some((what, n)) => drop(book_copies.insert(what, n)),
                    None => wrong.push(format!("{BOOK}: `{rule}`: a Copies line is `<what> = N`: {line}")),
                }
            }
        }
    }
    for test in &tests {
        if !book.rules.contains_key(test) && !self_tests.contains(test) {
            wrong.push(format!("{BOOK}: no entry for `{test}` (a `### {test}` or a Self-test line)"));
        }
    }
    for test in &self_tests {
        if !tests.contains(test) {
            wrong.push(format!("{BOOK}: Self-test `{test}` is no test of lumacore-tidy"));
        }
    }
    if book_ratchets != ratchets(&src) {
        wrong.push(format!("{BOOK}: its ratchets are {book_ratchets:?}, the code's {:?}", ratchets(&src)));
    }
    if book_copies != copies(&src) {
        wrong.push(format!("{BOOK}: its copies are {book_copies:?}, the code's {:?}", copies(&src)));
    }
    assert!(wrong.is_empty(), "the rulebook (decisions §34 20):\n{}", wrong.join("\n"));
}

#[test]
fn rulebook_reader_reads_entries_and_numbers() {
    let book = read_book("# x\n## Area\n### `a_rule`\n- **Checks:** this\n- **Ratchet:** `X_AT_MOST` = 3\n## Next\n- **Checks:** not in a rule\n");
    assert_eq!(book.rules["a_rule"].len(), 2);
    assert_eq!(number_line(" `X_AT_MOST` = 3"), Some(("X_AT_MOST".to_string(), 3)));
    assert_eq!(number_line(" a not-found reply (errors::not_found) = 28"), Some(("a not-found reply (errors::not_found)".to_string(), 28)));
    assert_eq!(ratchets("const A_AT_MOST: usize = 4;\nconst B: usize = 1;"), BTreeMap::from([("A_AT_MOST".to_string(), 4)]));
    assert_eq!(copies("const COPIES: &[(&str, &str, usize)] = &[\n    (\"the thing (door)\", \"pat\\\"\", 6),\n];"), BTreeMap::from([("the thing (door)".to_string(), 6)]));
}
