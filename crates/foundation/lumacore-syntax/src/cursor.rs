//! What the parser would accept where the cursor is — completion's one source (decision 5).

use crate::lex::TokenKind;
use crate::parse::Line;
use crate::{Span, WordId};

/// One kind of thing the parser would accept at a point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expect {
    Verb,
    Object,
    /// An object that may follow this one in an owner-first path.
    Child(WordId),
    /// A number, a range or quoted text.
    Ids,
    /// A `Root.` tree path.
    Tree,
    Helper,
    /// A value: a number, a value word, quoted text, a read.
    Value,
    Option,
    /// `On` / `Off` after a hold verb.
    Phase,
    /// A grammar word: `Thru`, `If`, `EndIf`.
    Keyword(&'static str),
    /// A tree-shell word.
    Fence,
    /// A tree-shell argument: a path, a flag, quoted text.
    FenceArg,
    /// `!name`.
    Alias,
}

/// Where completion works: the text to replace, what is typed of it, what fits there.
#[derive(Debug, Clone, PartialEq)]
pub struct Cursor {
    pub replace: Span,
    pub partial: String,
    pub expected: Vec<Expect>,
}

/// What fits at byte `offset` of a parsed line.
pub fn at_cursor(line: &Line, offset: usize) -> Cursor {
    let offset = offset.min(line.source.len()) as u32;
    let typing = line.tokens.iter().position(|t| {
        t.span.start < offset
            && offset <= t.span.end
            && matches!(t.kind, TokenKind::Word | TokenKind::Dotted | TokenKind::Number | TokenKind::Option | TokenKind::Alias | TokenKind::Quoted { .. } | TokenKind::Line { .. })
    });
    match typing {
        Some(i) => {
            let span = line.tokens[i].span;
            Cursor {
                replace: span,
                partial: line.source[span.start as usize..offset as usize].to_string(),
                expected: line.expects[i].clone(),
            }
        }
        None => {
            let next = line.tokens.iter().position(|t| t.span.start >= offset).unwrap_or(line.tokens.len());
            Cursor { replace: Span { start: offset, end: offset }, partial: String::new(), expected: line.expects[next].clone() }
        }
    }
}
