//! A class for every token, for colouring a line as it is typed.

use crate::lex::TokenKind;
use crate::parse::Line;
use crate::{Span, Vocabulary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Verb,
    Object,
    Helper,
    /// `Thru`, `If`, `EndIf`, `Self`, `On` / `Off` after a hold verb.
    Keyword,
    Value,
    Number,
    Text,
    Option,
    Operator,
    Path,
    Alias,
    Fence,
    /// A token a diagnostic points at.
    Error,
}

/// The class of every token of a parsed line, in order.
pub fn highlight(line: &Line, vocab: &Vocabulary) -> Vec<(Span, Class)> {
    let mut statement_start = true;
    let mut fence = false;
    line.tokens
        .iter()
        .map(|t| {
            let text = t.text(&line.source);
            let class = match t.kind {
                TokenKind::Semi => {
                    statement_start = true;
                    fence = false;
                    return (t.span, Class::Operator);
                }
                _ if line.diags.iter().any(|d| d.span.start <= t.span.start && t.span.end <= d.span.end && !d.span.is_empty()) => Class::Error,
                TokenKind::Word if ["Thru", "If", "EndIf", "Self"].iter().any(|k| text.eq_ignore_ascii_case(k)) => Class::Keyword,
                TokenKind::Word => match vocab.lookup(text).map(|w| vocab.word(w)) {
                    Some(w) if statement_start && w.fence.is_some() => {
                        fence = true;
                        Class::Fence
                    }
                    _ if fence => Class::Path,
                    Some(w) if w.object.is_some() => Class::Object,
                    Some(w) if statement_start && w.verb.is_some() => Class::Verb,
                    Some(w) if w.helper => Class::Helper,
                    Some(w) if w.value || w.verb.is_some() => Class::Value,
                    _ => Class::Error,
                },
                TokenKind::Number => Class::Number,
                TokenKind::Quoted { .. } | TokenKind::Line { .. } => Class::Text,
                TokenKind::Dotted | TokenKind::DotDot | TokenKind::Home => Class::Path,
                TokenKind::Option => Class::Option,
                TokenKind::Op(_) | TokenKind::Question => Class::Operator,
                TokenKind::Alias => Class::Alias,
                TokenKind::Unknown => Class::Error,
            };
            statement_start = false;
            (t.span, class)
        })
        .collect()
}
