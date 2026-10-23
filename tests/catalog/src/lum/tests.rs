// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! lum.md against the station's vocabulary (decisions §33 decision 3, part 2).
//!
//! Every ` ```lum ` example must parse with the station's words and every ` ```lum-error CODE `
//! example must fail with its code. Until part 3 registers the words §33 adds (and drops the
//! ones it removes), the lines that don't parse yet are listed in [`PENDING`]: the list may
//! only shrink — a listed line that parses fails the test until it is taken off — and the
//! branch merges when it is empty (decision 33).
//!
//! Owns:          the lum.md checks, PENDING, the golden record `testdata/lum_words.golden.txt`.
//! Doesn't own:   the parser (lumacore-syntax), the vocabulary (the kernel's lum).
//! Entry points:  the #[test] fns.
//! Decisions:     §33 3, 33, §34 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_syntax::{Origin, parse, print};

use lumacore_objects::lum::vocabulary;

const LUM_MD: &str = include_str!(concat!(env!("LUMACORE_ROOT"), "/docs/user/syntax/lum.md"));

/// Codes only the signature binding finds (§33 part 2b).
const BIND_CODES: &[&str] = &["E-SYN-014"];

/// lum.md lines whose words the registries don't have yet (or still have under the old
/// meaning). Part 3 empties it.
const PENDING: &[&str] = &[
    // The demo rig: second ids, folders, the pool XML reader (decisions §40, 2026-10-20).
    // Your tool (§34 round 3, "RunTool / EditTool": the Tool object, round 3 part 3d).
    "Tool \"Edit\"",
    "Tool 1",
    "Toggle Tool",
    "Info Tool",
    "Set UserProfile \"DefaultTool\" Tool \"Edit\"",
    // Clone, AttributeMap and the map's options (§34 round 3 19).
    "Set Fixture 101 Thru 106 \"FixtureType\" \"Two-wheel spot\" /Mode \"Mode 1\" /Map 3 /Remember",
    "Clone Fixture 101 Thru 106 At Fixture 201 Thru 206",
    "Clone Fixture 101 Thru 106 FeatureGroup \"Position\" + \"Color\" At Fixture 201 Thru 206",
    "Clone Fixture 101 Attribute \"Pan\" + \"Tilt\" At Fixture 201",
    "Clone Fixture 101 Thru 106 At Fixture 201 Thru 206 If Sequence 1 + 2 + 7",
    "Clone Fixture 101 At Fixture 201 /Merge /NoDependencies",
    "Clone Fixture 101 At Fixture 201 /Keep /KeepOutput /Map 3",
    "List AttributeMap",
    "Info AttributeMap 3",
    // The Users object and /From (Import parts, 20).
    "Import Patch + Users /From \"Festival\"",
    // The Alias object (13).
    "Store Alias \"St\" `Store Cue 1`",
    "Store Alias \"sg\" `Store Group`",
    "Store Alias \"au\" `Set Self \"AutoScroll\" Toggle`",
    "List Alias",
    "Delete Alias \"sg\"",
    // MVR import (§34 "Import MVR hardened" 3–4): the review's rows and the placement asks, part 4b.
];

fn fenced(info: &str) -> Vec<(String, Vec<String>)> {
    let mut blocks = Vec::new();
    let mut lines = LUM_MD.lines();
    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix("```") else {
            continue;
        };
        if !(rest == info || rest.starts_with(&format!("{info} "))) {
            continue;
        }
        let tag = rest.strip_prefix(info).unwrap().trim().to_string();
        let body = lines
            .by_ref()
            .take_while(|l| !l.starts_with("```"))
            .map(str::to_string)
            .collect();
        blocks.push((tag, body));
    }
    blocks
}

fn examples(info: &str) -> Vec<(String, String)> {
    fenced(info)
        .into_iter()
        .flat_map(|(tag, lines)| {
            lines
                .into_iter()
                .filter(|l| !l.trim().is_empty())
                .map(move |l| (tag.clone(), l))
        })
        .collect()
}

#[test]
fn every_lum_example_parses_with_the_registries() {
    let vocab = vocabulary();
    let mut failing = Vec::new();
    let mut fixed = Vec::new();
    for (_, line) in examples("lum") {
        let parsed = parse(&line, vocab, Origin::Typed);
        let ok = parsed.is_clean() && {
            let printed = print(&parsed, vocab);
            let again = parse(&printed, vocab, Origin::Typed);
            again.is_clean() && print(&again, vocab) == printed
        };
        match (ok, PENDING.contains(&line.as_str())) {
            (false, false) => failing.push(format!("{line}\n    {:?}", parsed.diags)),
            (true, true) => fixed.push(line),
            _ => {}
        }
    }
    assert!(
        fixed.is_empty(),
        "these lines parse now — take them off PENDING:\n{}",
        fixed.join("\n")
    );
    assert!(
        failing.is_empty(),
        "lum.md examples the registries' words don't parse:\n{}",
        failing.join("\n")
    );
}

#[test]
fn every_error_example_fails_with_its_code() {
    let vocab = vocabulary();
    let mut wrong = Vec::new();
    for (code, line) in examples("lum-error") {
        if BIND_CODES.contains(&code.as_str()) {
            continue;
        }
        let parsed = parse(&line, vocab, Origin::Typed);
        let ok = parsed.diags.iter().any(|d| d.code == code);
        match (ok, PENDING.contains(&line.as_str())) {
            (false, false) => wrong.push(format!(
                "{line} should fail with {code}, got {:?}",
                parsed.diags
            )),
            (true, true) => wrong.push(format!(
                "{line} fails with {code} now — take it off PENDING"
            )),
            _ => {}
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn every_pending_line_is_in_lum_md() {
    let all: Vec<String> = examples("lum")
        .into_iter()
        .chain(examples("lum-error"))
        .map(|(_, l)| l)
        .collect();
    let stale: Vec<&&str> = PENDING
        .iter()
        .filter(|p| !all.iter().any(|l| l == **p))
        .collect();
    assert!(
        stale.is_empty(),
        "PENDING names lines lum.md doesn't have: {stale:?}"
    );
}

/// The language's words, one per line with their roles — a golden record like
/// `tree_paths.golden.txt` (rewrite with `LUMACORE_BLESS=1`, read the diff).
#[test]
fn the_words_are_the_golden_record() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/lum_words.golden.txt");
    let listing = lumacore_syntax::grammar_listing(vocabulary());
    if std::env::var_os("LUMACORE_BLESS").is_some() {
        std::fs::write(path, &listing).unwrap();
        return;
    }
    let golden = std::fs::read_to_string(path).unwrap_or_default();
    assert!(
        golden == listing,
        "Lum's words changed — rewrite {path} with LUMACORE_BLESS=1 and read the diff"
    );
}

/// Every object declares how it reads a dotted id, and an object that nests itself reads a path
/// (lum.md "Every object declares how it reads a dotted id", decisions §12 "Tags hardened").
#[test]
fn an_object_that_nests_itself_reads_a_path() {
    use lumacore_objects::objects::IdReading;
    for o in lumacore_objects::objects::all() {
        let nests = o.lum.children.iter().any(|c| std::ptr::eq(*c, &o.decl));
        if nests {
            assert_eq!(o.lum.reading, IdReading::Path, "{} nests itself but reads {:?}", o.keyword, o.lum.reading);
        }
        match o.lum.reading {
            IdReading::None => assert_eq!(o.lum.id_parts, 0, "{}", o.keyword),
            IdReading::Whole => assert_eq!(o.lum.id_parts, 1, "{}", o.keyword),
            IdReading::Decimal => assert_eq!(o.lum.id_parts, 2, "{}", o.keyword),
            IdReading::Parts(n) => assert_eq!(o.lum.id_parts, n, "{}", o.keyword),
            IdReading::Path => assert_eq!(o.lum.id_parts, u8::MAX, "{}", o.keyword),
        }
    }
}
