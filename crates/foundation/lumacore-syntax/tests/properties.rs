//! Properties over any input (fixed seeds, fast): the parser is total, spans are sound, and a
//! clean line prints to one spelling that parses back the same.

mod common;

use lumacore_syntax::{Origin, at_cursor, highlight, lex, parse, print};
use proptest::prelude::*;

/// Lines made of Lum's own pieces, so most of them get past the first word.
fn lum_like() -> impl Strategy<Value = String> {
    let piece = prop_oneof![
        Just("Store"), Just("Group"), Just("Fixture"), Just("Sequence"), Just("Cue"), Just("DataPool"), Just("At"),
        Just("Thru"), Just("+"), Just("-"), Just("*"), Just("If"), Just("EndIf"), Just("Self"), Just("Flash"), Just("On"),
        Just("1"), Just("2.5"), Just("-3"), Just("\"Front\""), Just("\"x"), Just("/Merge"), Just("/Type"), Just(";"),
        Just("Root.ShowData.DataPools.2"), Just("Self.Name"), Just("cd"), Just(".."), Just("-r"), Just("!St"), Just("?"),
        Just("Full"), Just("Set"), Just("\"Name\""), Just("Name"), Just("Bühne"), Just("Go-"),
    ];
    prop::collection::vec((piece, prop::bool::ANY), 0..12)
        .prop_map(|parts| parts.into_iter().map(|(p, glue)| if glue { p.to_string() } else { format!("{p} ") }).collect())
}

/// Fixed seeds in `cargo test` (the same lines every time); the overnight run sets
/// `PROPTEST_RNG_SEED` to explore (like `lumacore-sweep`'s `src/tests.rs`).
fn fixed(cases: u32) -> ProptestConfig {
    let mut c = ProptestConfig { cases, failure_persistence: None, ..ProptestConfig::default() };
    if std::env::var_os("PROPTEST_RNG_SEED").is_none() {
        c.rng_seed = proptest::test_runner::RngSeed::Fixed(33);
    }
    c
}

proptest! {
    #![proptest_config(fixed(2000))]

    #[test]
    fn any_text_parses_without_panicking(src in "\\PC{0,60}") {
        let vocab = common::lum_vocab();
        for origin in [Origin::Typed, Origin::Stored, Origin::Code] {
            let line = parse(&src, &vocab, origin);
            for d in &line.diags {
                prop_assert!(src.is_char_boundary(d.span.start as usize) && src.is_char_boundary(d.span.end as usize));
                prop_assert!(d.span.start <= d.span.end && d.span.end as usize <= src.len());
            }
            let _ = highlight(&line, &vocab);
            let _ = print(&line, &vocab);
            for i in 0..=src.len() {
                if src.is_char_boundary(i) {
                    let c = at_cursor(&line, i);
                    prop_assert!(c.replace.end as usize <= src.len());
                }
            }
        }
    }

    #[test]
    fn tokens_cover_the_line_in_order(src in "\\PC{0,60}") {
        let mut last = 0;
        for t in lex(&src) {
            prop_assert!(t.span.start as usize >= last && t.span.start < t.span.end);
            prop_assert!(src.is_char_boundary(t.span.start as usize) && src.is_char_boundary(t.span.end as usize));
            last = t.span.end as usize;
        }
        prop_assert!(last <= src.len());
    }

    #[test]
    fn a_clean_line_prints_to_one_spelling(src in lum_like()) {
        let vocab = common::lum_vocab();
        let line = parse(&src, &vocab, Origin::Typed);
        if line.is_clean() {
            let once = print(&line, &vocab);
            let again = parse(&once, &vocab, Origin::Typed);
            prop_assert!(again.is_clean(), "{src:?} printed as {once:?}: {:?}", again.diags);
            prop_assert_eq!(print(&again, &vocab), once);
        }
    }
}
