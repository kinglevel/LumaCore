//! Lum, LumaCore's command language (decisions §33).
//!
//! The grammar is `docs/user/syntax/lum.md` — [`GRAMMAR`] is its `grammar` block, and a test
//! fails when the two differ. The words are not fixed here: the station builds a
//! [`Vocabulary`] from its registries and every function reads it.
//!
//! - [`parse`] turns a line into [`Statement`]s. It is total: it never fails and never panics;
//!   what it can't accept becomes a coded [`Diag`] with a span, and parsing goes on.
//! - [`at_cursor`] says what the parser would accept where the cursor is (completion reads
//!   this — there is no second grammar).
//! - [`highlight`] classes every token; [`print`] writes the canonical line (what is forwarded,
//!   stored in keys and macros, journalled).
//! - [`bind`] fills a verb's typed arguments ([`Args`], `#[derive(Args)]` in
//!   `lumacore-syntax-derive`) from a statement: the verb's signature says which slot is what.

pub mod bind;
mod cursor;
mod errors;
mod grammar;
mod highlight;
mod lex;
mod parse;
mod print;
mod vocab;

pub mod ast;

pub use cursor::{Cursor, Expect, at_cursor};
pub use errors::{Diag, ERRORS, codes};
pub use grammar::{GRAMMAR, KEYWORDS, grammar_listing, is_word};
pub use highlight::{Class, highlight};
pub use lex::{Token, TokenKind, lex};
pub use parse::{Line, Origin, parse};
pub use print::print;
pub use vocab::{FenceDef, ObjectDef, OptionDef, VerbDef, Vocabulary, WordDef, WordId};

pub use ast::Statement;
pub use bind::{Args, Bound, FenceLine, CommandLine, FromPhase, FromTarget, FromValue, OnOff, Signature, Switch, bind};

/// A byte range in the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start: start as u32, end: end as u32 }
    }

    /// The covering span of two spans.
    pub fn to(self, other: Span) -> Span {
        Span { start: self.start.min(other.start), end: self.end.max(other.end) }
    }

    pub fn range(self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// This crate's rows in the catalog, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.errors(ERRORS);
}

/// This crate's tests read the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
