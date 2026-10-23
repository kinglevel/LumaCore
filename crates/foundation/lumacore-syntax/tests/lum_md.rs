//! lum.md is the spec (decisions §33 decision 3): its grammar block is the crate's grammar,
//! every ` ```lum ` line parses, every ` ```lum-error <code> ` line fails with its code.

mod common;

use lumacore_syntax::{Expect, Origin, TokenKind, Vocabulary, at_cursor, highlight, parse, print};

const LUM_MD: &str = include_str!(concat!(env!("LUMACORE_ROOT"), "/docs/user/syntax/lum.md"));

/// Codes only the signature binding can find (§33 part 2): the parser alone can't tell them.
const BIND_CODES: &[&str] = &["E-SYN-014"];

fn fenced(info: &str) -> Vec<(String, Vec<String>)> {
    let mut blocks = Vec::new();
    let mut lines = LUM_MD.lines();
    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix("```") else { continue };
        if !(rest == info || rest.starts_with(&format!("{info} "))) {
            continue;
        }
        let tag = rest.strip_prefix(info).unwrap().trim().to_string();
        let body = lines.by_ref().take_while(|l| !l.starts_with("```")).map(str::to_string).collect();
        blocks.push((tag, body));
    }
    blocks
}

#[test]
fn the_grammar_is_lum_md() {
    let block = LUM_MD
        .split("````grammar\n")
        .nth(1)
        .and_then(|rest| rest.split("\n````").next())
        .expect("lum.md has a ````grammar block");
    assert_eq!(block, lumacore_syntax::GRAMMAR, "lum.md's grammar block and the crate's GRAMMAR differ");
}

#[test]
fn every_lum_example_parses_and_prints_back() {
    let vocab = common::lum_vocab();
    let examples: Vec<String> = fenced("lum").into_iter().flat_map(|(_, lines)| lines).filter(|l| !l.trim().is_empty()).collect();
    assert!(examples.len() > 100, "lum.md has its examples ({})", examples.len());
    let mut failures = Vec::new();
    for line in &examples {
        let parsed = parse(line, &vocab, Origin::Typed);
        if !parsed.is_clean() {
            failures.push(format!("{line}\n    {:?}", parsed.diags));
            continue;
        }
        let printed = print(&parsed, &vocab);
        let again = parse(&printed, &vocab, Origin::Typed);
        if !again.is_clean() || print(&again, &vocab) != printed {
            failures.push(format!("{line}\n    printed as {printed:?}, which doesn't parse back the same: {:?}", again.diags));
        }
    }
    assert!(failures.is_empty(), "lum.md examples that don't parse:\n{}", failures.join("\n"));
}

#[test]
fn every_error_example_fails_with_its_code() {
    let vocab = common::lum_vocab();
    let blocks = fenced("lum-error");
    assert!(blocks.len() >= 8);
    for (code, lines) in blocks {
        if BIND_CODES.contains(&code.as_str()) {
            continue;
        }
        for line in lines.iter().filter(|l| !l.trim().is_empty()) {
            let parsed = parse(line, &vocab, Origin::Typed);
            assert!(parsed.diags.iter().any(|d| d.code == code), "{line:?} should fail with {code}, got {:?}", parsed.diags);
        }
    }
}

/// Completion is tied to the parser: at every token of a valid line, what the parser says it
/// accepts includes what is actually there.
#[test]
fn the_cursor_expects_what_comes_next() {
    let vocab = common::lum_vocab();
    for (_, lines) in fenced("lum") {
        for line in lines.iter().filter(|l| !l.trim().is_empty()) {
            let parsed = parse(line, &vocab, Origin::Typed);
            for t in &parsed.tokens {
                if matches!(t.kind, TokenKind::Op(_) | TokenKind::Semi | TokenKind::DotDot | TokenKind::Home | TokenKind::Question) {
                    continue;
                }
                let cursor = at_cursor(&parsed, t.span.start as usize);
                let text = t.text(line);
                assert!(
                    cursor.expected.iter().any(|e| accepts(e, t.kind, text, &vocab)),
                    "{line:?}: at {text:?} the parser expects {:?}",
                    cursor.expected
                );
            }
        }
    }
}

#[test]
fn clean_lines_highlight_without_errors() {
    let vocab = common::lum_vocab();
    for (_, lines) in fenced("lum") {
        for line in lines.iter().filter(|l| !l.trim().is_empty()) {
            let parsed = parse(line, &vocab, Origin::Typed);
            let classes = highlight(&parsed, &vocab);
            assert!(classes.iter().all(|(_, c)| *c != lumacore_syntax::Class::Error), "{line:?}: {classes:?}");
        }
    }
}

fn accepts(e: &Expect, kind: TokenKind, text: &str, vocab: &Vocabulary) -> bool {
    let keyword = |k: &str| text.eq_ignore_ascii_case(k);
    match kind {
        TokenKind::Word => {
            // A leading Thru opens ids (`Thru 4`, `Group Thru` = every group; lum.md "Targets").
            if keyword("Thru") && *e == Expect::Ids {
                return true;
            }
            if keyword("Thru") || keyword("If") || keyword("EndIf") {
                return *e == Expect::Keyword(if keyword("Thru") { "Thru" } else if keyword("If") { "If" } else { "EndIf" });
            }
            if keyword("Self") {
                return matches!(e, Expect::Object | Expect::Value | Expect::Tree);
            }
            let Some(w) = vocab.lookup(text).map(|id| vocab.word(id)) else {
                return *e == Expect::FenceArg;
            };
            match e {
                Expect::Verb => w.verb.is_some(),
                Expect::Object => w.object.is_some(),
                Expect::Child(owner) => w.object.is_some() && vocab.lookup(text).is_some_and(|id| vocab.is_child(*owner, id)),
                Expect::Helper => w.helper,
                Expect::Value => w.value || w.verb.is_some() || w.object.is_some(),
                Expect::Phase => keyword("On") || keyword("Off"),
                Expect::Fence => w.fence.is_some(),
                Expect::FenceArg => true,
                _ => false,
            }
        }
        TokenKind::Number | TokenKind::Quoted { .. } => matches!(e, Expect::Ids | Expect::Value | Expect::FenceArg),
        TokenKind::Dotted => matches!(e, Expect::Tree | Expect::Value | Expect::FenceArg),
        TokenKind::Option => matches!(e, Expect::Option | Expect::FenceArg),
        TokenKind::Alias => *e == Expect::Alias,
        _ => true,
    }
}
