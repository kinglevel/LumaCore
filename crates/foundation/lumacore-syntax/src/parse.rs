//! The parser: recursive descent over the tokens, driven by the vocabulary (lum.md "Grammar").
//!
//! Total: every line gives a [`Line`]. What can't be accepted becomes a coded [`Diag`] and the
//! parser goes on, so completion and highlighting work on half-typed lines. At every token it
//! looks at, it records what it would accept there ([`Expect`]) — [`crate::at_cursor`] reads that.

use crate::ast::*;
use crate::cursor::Expect;
use crate::errors::{Diag, codes};
use crate::lex::{Token, TokenKind, lex};
use crate::{Span, Vocabulary, WordId};

/// Where a line comes from — what it may use (decisions §33 12, 13, 18, 21).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Typed on a command line: aliases, bare numbers and names, the fence.
    Typed,
    /// Stored by a user (key, macro, cue command) or sent by a machine: exact — no alias, no
    /// bare number or name.
    Stored,
    /// Written by LumaCore's own code (templates, registry lines): pure Lum — also no fence.
    Code,
}

/// A parsed line.
#[derive(Debug, Clone)]
pub struct Line {
    pub source: String,
    pub tokens: Vec<Token>,
    pub statements: Vec<Statement>,
    pub diags: Vec<Diag>,
    /// What the parser would have accepted at each token (one more for the end of the line).
    pub(crate) expects: Vec<Vec<Expect>>,
}

impl Line {
    pub fn is_clean(&self) -> bool {
        self.diags.is_empty()
    }
}

/// Parse a line. Never fails, never panics.
pub fn parse(src: &str, vocab: &Vocabulary, origin: Origin) -> Line {
    let tokens = lex(src);
    let mut p = Parser {
        src,
        toks: &tokens,
        pos: 0,
        vocab,
        origin,
        diags: Vec::new(),
        expects: vec![Vec::new(); tokens.len() + 1],
    };
    for t in &tokens {
        if t.kind == (TokenKind::Quoted { terminated: false }) {
            p.diag(codes::UNTERMINATED, t.span, "the quote isn't closed — end it with \"".into());
        }
        if t.kind == (TokenKind::Line { terminated: false }) {
            p.diag(codes::UNTERMINATED, t.span, "the line isn't closed — end it with `".into());
        }
    }
    let mut statements = Vec::new();
    loop {
        let statement = p.statement();
        if p.peek().is_some_and(|t| t.kind != TokenKind::Semi) {
            let t = p.peek().unwrap();
            p.diag(codes::UNEXPECTED, t.span, format!("'{}' can't stand here", t.text(src)));
            while p.peek().is_some_and(|t| t.kind != TokenKind::Semi) {
                p.pos += 1;
            }
        }
        if !matches!(statement.body, Body::Empty) {
            statements.push(statement);
        }
        if p.peek().is_some() {
            p.pos += 1; // the `;`
        } else {
            break;
        }
    }
    p.diags.sort_by_key(|d| d.span.start);
    Line { source: src.to_string(), tokens: tokens.clone(), statements, diags: p.diags, expects: p.expects }
}

struct Parser<'a> {
    src: &'a str,
    toks: &'a [Token],
    pos: usize,
    vocab: &'a Vocabulary,
    origin: Origin,
    diags: Vec<Diag>,
    expects: Vec<Vec<Expect>>,
}

impl<'a> Parser<'a> {
    // ── tokens ──

    fn peek(&self) -> Option<Token> {
        self.toks.get(self.pos).copied()
    }

    fn peek_at(&self, n: usize) -> Option<Token> {
        self.toks.get(self.pos + n).copied()
    }

    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos];
        self.pos += 1;
        t
    }

    fn text(&self, t: Token) -> &'a str {
        t.text(self.src)
    }

    /// The current token, unless the statement ends here.
    fn here(&self) -> Option<Token> {
        self.peek().filter(|t| t.kind != TokenKind::Semi)
    }

    fn end_span(&self) -> Span {
        let at = self.peek().map_or(self.src.len() as u32, |t| t.span.start);
        Span { start: at, end: at }
    }

    fn diag(&mut self, code: &'static str, span: Span, message: String) {
        self.diags.push(Diag { code, span, message });
    }

    /// What the parser accepts at the current token.
    fn expect(&mut self, what: &[Expect]) {
        self.expect_at(self.pos, what);
    }

    fn expect_at(&mut self, index: usize, what: &[Expect]) {
        let Some(slot) = self.expects.get_mut(index) else { return };
        for e in what {
            if !slot.contains(e) {
                slot.push(e.clone());
            }
        }
    }

    // ── words ──

    fn is_keyword(&self, t: Token, keyword: &str) -> bool {
        t.kind == TokenKind::Word && self.text(t).eq_ignore_ascii_case(keyword)
    }

    fn word(&self, t: Token) -> Option<WordId> {
        (t.kind == TokenKind::Word).then(|| self.vocab.lookup(self.text(t))).flatten()
    }

    fn object(&self, t: Token) -> Option<WordId> {
        self.word(t).filter(|&w| self.vocab.word(w).object.is_some())
    }

    fn verb(&self, t: Token) -> Option<WordId> {
        self.word(t).filter(|&w| self.vocab.word(w).verb.is_some())
    }

    fn unknown_word(&mut self, t: Token) {
        let text = self.text(t);
        let close = self.vocab.closest(text);
        let hint = match close.as_slice() {
            [] => String::new(),
            [one] => format!(" — did you mean {one}?"),
            more => format!(" — did you mean {}?", more.join(", ")),
        };
        self.diag(codes::UNKNOWN_WORD, t.span, format!("unknown word '{text}'{hint}"));
    }

    // ── statements ──

    fn statement(&mut self) -> Statement {
        let start = self.end_span();
        self.expect(&[Expect::Verb, Expect::Object, Expect::Ids, Expect::Fence, Expect::Alias]);
        let Some(t) = self.here() else {
            return Statement { span: start, body: Body::Empty };
        };
        let body = match t.kind {
            TokenKind::Alias => self.alias(),
            TokenKind::Word if self.peek_at(1).is_some_and(|q| q.kind == TokenKind::Question && q.span.start == t.span.end) => {
                self.pos += 2;
                Body::Hint { word: Spanned::new(self.text(t).to_string(), t.span) }
            }
            TokenKind::Word if self.word(t).is_some_and(|w| self.vocab.word(w).fence.is_some()) => self.fence(),
            _ => Body::Lum(self.lum()),
        };
        let end = self.toks[..self.pos].last().map_or(start, |t| t.span);
        Statement { span: start.to(end).to(t.span), body }
    }

    fn alias(&mut self) -> Body {
        let t = self.bump();
        if self.origin != Origin::Typed {
            self.diag(codes::PERSONAL_IN_STORED, t.span, "an alias works only in a line typed on a command line — write the full line".into());
        }
        let first = self.here();
        while self.here().is_some() {
            self.pos += 1;
        }
        let rest = match (first, self.toks[..self.pos].last()) {
            (Some(f), Some(l)) => {
                let span = f.span.to(l.span);
                Spanned::new(self.src[span.range()].to_string(), span)
            }
            _ => Spanned::new(String::new(), Span { start: t.span.end, end: t.span.end }),
        };
        Body::Alias { name: Spanned::new(self.text(t)[1..].to_string(), t.span), rest }
    }

    fn fence(&mut self) -> Body {
        let t = self.bump();
        let word = self.word(t).expect("fence word");
        if self.origin == Origin::Code {
            self.diag(codes::FENCE_IN_CODE, t.span, format!("'{}' is a tree-shell word — LumaCore's own lines are pure Lum", self.text(t)));
        }
        let flags = self.vocab.word(word).fence.clone().unwrap_or_default().flags;
        let mut args = Vec::new();
        loop {
            self.expect(&[Expect::FenceArg]);
            let Some(a) = self.here() else { break };
            match a.kind {
                TokenKind::Op('-') => {
                    let long = self.peek_at(1).is_some_and(|n| n.kind == TokenKind::Op('-') && n.span.start == a.span.end);
                    let name_at = if long { 2 } else { 1 };
                    let glued_to = if long { self.peek_at(1).unwrap().span.end } else { a.span.end };
                    for k in 1..=name_at {
                        self.expect_at(self.pos + k, &[Expect::FenceArg]);
                    }
                    match self.peek_at(name_at).filter(|n| n.kind == TokenKind::Word && n.span.start == glued_to) {
                        Some(n) => {
                            self.pos += name_at + 1;
                            let name = self.text(n).to_string();
                            let declared = flags.iter().find(|(f, _)| f.eq_ignore_ascii_case(&name));
                            if declared.is_none() {
                                self.diag(codes::OPTION, a.span.to(n.span), format!("'{}' has no flag '{}'", self.vocab.word(word).name, &self.src[a.span.to(n.span).range()]));
                            }
                            let value = if declared.is_some_and(|&(_, v)| v) {
                                self.expect(&[Expect::FenceArg]);
                                match self.here() {
                                    Some(v) if matches!(v.kind, TokenKind::Word | TokenKind::Number | TokenKind::Dotted | TokenKind::Quoted { .. }) => {
                                        self.pos += 1;
                                        Some(Spanned::new(unquote(self.text(v)).to_string(), v.span))
                                    }
                                    _ => {
                                        let at = self.end_span();
                                        self.diag(codes::MISSING, at, format!("'{}' needs a value", &self.src[a.span.to(n.span).range()]));
                                        None
                                    }
                                }
                            } else {
                                None
                            };
                            args.push(FenceArg::Flag { name: Spanned::new(name, a.span.to(n.span)), long, value });
                        }
                        None => {
                            self.pos += 1;
                            self.diag(codes::UNEXPECTED, a.span, "a flag is written -x or --name".into());
                        }
                    }
                }
                TokenKind::Op('/') | TokenKind::DotDot | TokenKind::Home | TokenKind::Word | TokenKind::Number | TokenKind::Dotted | TokenKind::Option => {
                    self.pos += 1;
                    args.push(FenceArg::Path(Spanned::new(self.text(a).to_string(), a.span)));
                }
                TokenKind::Quoted { .. } => {
                    self.pos += 1;
                    args.push(FenceArg::Text(Spanned::new(unquote(self.text(a)).to_string(), a.span)));
                }
                _ => {
                    self.pos += 1;
                    self.diag(codes::UNEXPECTED, a.span, format!("'{}' can't stand here", self.text(a)));
                }
            }
        }
        Body::Fence { word: Spanned::new(word, t.span), args }
    }

    fn lum(&mut self) -> Lum {
        let mut lum = Lum::default();

        if let Some(t) = self.here()
            && let TokenKind::Op(c @ ('+' | '-')) = t.kind
        {
            self.pos += 1;
            lum.select_op = Some(Spanned::new(set_op(c), t.span));
            self.expect(&[Expect::Object, Expect::Ids]);
        }

        if let Some(t) = self.here()
            && let Some(verb) = self.verb(t)
            && self.vocab.word(verb).object.is_none()
        {
            self.pos += 1;
            if let Some(op) = &lum.select_op {
                self.diag(codes::UNEXPECTED, op.span, "+ / - at the start takes a selection, not a verb".into());
            }
            lum.verb = Some(Spanned::new(verb, t.span));
            if self.vocab.word(verb).verb.as_ref().is_some_and(|v| v.hold) {
                self.expect(&[Expect::Phase]);
                match self.here() {
                    Some(p) if self.is_keyword(p, "On") || self.is_keyword(p, "Off") => {
                        self.pos += 1;
                        let phase = if self.is_keyword(p, "On") { Phase::On } else { Phase::Off };
                        lum.phase = Some(Spanned::new(phase, p.span));
                    }
                    _ => {
                        let name = &self.vocab.word(verb).name;
                        self.diag(codes::HOLD_PHASE, t.span, format!("{name} takes On or Off right after it: {name} On …, {name} Off …"));
                    }
                }
            }
            self.expect(&[Expect::Keyword("If")]);
            if self.here().is_some_and(|t| self.is_keyword(t, "If")) {
                lum.condition = Some(self.condition(true));
            }
        }

        let after_verb = lum.verb.is_some();
        self.expect(&[Expect::Object, Expect::Ids, Expect::Tree]);
        if after_verb {
            self.expect(&[Expect::Value, Expect::Helper, Expect::Option, Expect::Keyword("If")]);
        }
        lum.target = self.target();

        if self.origin != Origin::Typed
            && lum.verb.is_none()
            && let Some(target) = &lum.target
            && matches!(target.items.first(), Some((_, Item::Ids(_))))
        {
            let span = target.items[0].1.span();
            self.diag(codes::PERSONAL_IN_STORED, span, "a stored line names its object: a bare number or name depends on whose default object it runs with".into());
        }

        loop {
            self.expect(&[Expect::Helper, Expect::Value, Expect::Option, Expect::Keyword("If")]);
            let Some(t) = self.here() else { break };
            if t.kind == TokenKind::Option || self.is_keyword(t, "If") {
                break;
            }
            match self.slot() {
                Some(slot) => lum.slots.push(slot),
                None => {
                    if self.pos < self.toks.len() && self.peek() == Some(t) {
                        self.pos += 1;
                        if !self.diags.iter().any(|d| d.span == t.span) {
                            self.diag(codes::UNEXPECTED, t.span, format!("'{}' can't stand here", self.text(t)));
                        }
                    }
                }
            }
        }

        if lum.condition.is_none() && self.here().is_some_and(|t| self.is_keyword(t, "If")) {
            lum.condition = Some(self.condition(false));
        }

        loop {
            self.expect(&[Expect::Option]);
            let Some(t) = self.here() else { break };
            if t.kind != TokenKind::Option {
                self.diag(codes::UNEXPECTED, t.span, format!("'{}' after an option — options come last", self.text(t)));
                while self.here().is_some_and(|t| t.kind != TokenKind::Option) {
                    self.pos += 1;
                }
                continue;
            }
            if let Some(option) = self.option() {
                lum.options.push(option);
            }
        }
        lum
    }

    fn condition(&mut self, early: bool) -> Condition {
        let if_tok = self.bump();
        self.expect(&[Expect::Object, Expect::Tree]);
        let target = self.target();
        let mut span = if_tok.span;
        if let Some(t) = &target {
            span = span.to(t.span());
        } else {
            let at = self.end_span();
            self.diag(codes::MISSING, at, "If needs what to filter by: If Group 5".into());
        }
        if early {
            self.expect(&[Expect::Keyword("EndIf")]);
            match self.here() {
                Some(t) if self.is_keyword(t, "EndIf") => {
                    self.pos += 1;
                    span = span.to(t.span);
                }
                _ => {
                    let at = self.end_span();
                    self.diag(codes::MISSING, at, "an If right after the verb ends with EndIf: Store If Group 5 EndIf Preset 1.1".into());
                }
            }
        }
        Condition { target: target.unwrap_or(Target { items: Vec::new() }), early, span }
    }

    fn option(&mut self) -> Option<OptionUse> {
        let t = self.bump();
        let written = &self.text(t)[1..];
        let Some(def) = self.vocab.find_option(written).cloned() else {
            let close: Vec<String> = self
                .vocab
                .options()
                .iter()
                .filter(|o| {
                    let first = |s: &str| s.chars().next().map(|c| c.to_lowercase().to_string());
                    first(&o.name) == first(written)
                })
                .take(3)
                .map(|o| format!("/{}", o.name))
                .collect();
            let hint = if close.is_empty() { String::new() } else { format!(" — try {}", close.join(", ")) };
            self.diag(codes::OPTION, t.span, format!("unknown option '/{written}'{hint}"));
            return None;
        };
        let name = Spanned::new(def.name.clone(), t.span);
        if !def.takes_value {
            return Some(OptionUse { name, value: None });
        }
        self.expect(&[Expect::Value]);
        let value = match self.here() {
            // An option's value is no property slot: a property word there is a word
            // (`Sort FeatureGroup "FX" /By Name` — decision 64, the slot decides).
            Some(v) if v.kind != TokenKind::Option && v.kind == TokenKind::Word && self.vocab.lookup(self.text(v)).is_some_and(|w| self.vocab.word(w).property) => {
                self.pos += 1;
                self.vocab.lookup(self.text(v)).map(|w| Value::Word(Spanned::new(w, v.span)))
            }
            Some(v) if v.kind != TokenKind::Option => self.single_value(),
            _ => None,
        };
        if value.is_none() {
            let at = self.end_span();
            self.diag(codes::MISSING, at, format!("/{} needs a value: the next word or quoted text", def.name));
        }
        Some(OptionUse { name, value })
    }

    // ── targets ──

    fn target(&mut self) -> Option<Target> {
        let first = self.item()?;
        let mut items = vec![(None, first)];
        while let Some(t) = self.here() {
            let TokenKind::Op(c @ ('+' | '-')) = t.kind else { break };
            if !self.peek_at(1).is_some_and(|n| self.starts_item(n) && !self.starts_ids(n)) {
                break;
            }
            self.check_spaced(t);
            self.pos += 1;
            self.expect(&[Expect::Object, Expect::Tree]);
            match self.item() {
                Some(item) => items.push((Some(Spanned::new(set_op(c), t.span)), item)),
                None => break,
            }
        }
        Some(Target { items })
    }

    fn starts_ids(&self, t: Token) -> bool {
        matches!(t.kind, TokenKind::Number | TokenKind::Quoted { .. }) || self.is_keyword(t, "Thru")
    }

    fn starts_item(&self, t: Token) -> bool {
        self.starts_ids(t)
            || self.object(t).is_some()
            || self.is_keyword(t, "Self")
            || (t.kind == TokenKind::Dotted && !is_read(self.text(t)))
    }

    fn item(&mut self) -> Option<Item> {
        let t = self.here()?;
        match t.kind {
            TokenKind::Word if self.is_keyword(t, "Self") => {
                self.pos += 1;
                Some(Item::SelfRef(t.span))
            }
            TokenKind::Word if self.object(t).is_some() => Some(self.path()),
            TokenKind::Dotted if !is_read(self.text(t)) => {
                self.pos += 1;
                let text = self.text(t);
                if !is_root_path(text) {
                    self.diag(
                        codes::UNEXPECTED,
                        t.span,
                        format!("'{text}' isn't a form — nest with spaces (DataPool 2 Group 3) or a tree path from Root. (Root.ShowData.DataPools.2.Groups.3)"),
                    );
                }
                Some(Item::Tree(Spanned::new(split_path(text), t.span)))
            }
            _ if self.starts_ids(t) => self.ids(None).map(Item::Ids),
            _ => None,
        }
    }

    fn path(&mut self) -> Item {
        let mut segments = Vec::new();
        loop {
            let t = self.bump();
            let object = self.object(t).expect("object word");
            let counted = self.vocab.word(object).object.as_ref().is_some_and(|o| o.counted);
            self.expect(&[Expect::Child(object), Expect::Helper, Expect::Value, Expect::Option, Expect::Keyword("If")]);
            if counted {
                self.expect(&[Expect::Ids]);
            }
            // `Fixture Houselights 4`: a kind the object declares, only with ids after it.
            let kind = self.id_kind(object);
            let ids = if counted && self.here().is_some_and(|n| self.starts_ids(n)) { self.ids(Some(object)) } else { None };
            segments.push(Segment { object: Spanned::new(object, t.span), kind, ids });
            self.expect(&[Expect::Child(object)]);
            let Some(next) = self.here() else { break };
            let Some(next_object) = self.object(next) else { break };
            if self.vocab.is_child(object, next_object) {
                continue;
            }
            if self.vocab.is_owner_of(next_object, segments[0].object.node) {
                let (owner, child) = (&self.vocab.word(next_object).name, &self.vocab.word(segments[0].object.node).name);
                self.diag(codes::CHILD_FIRST, next.span, format!("owner first: {owner} … {child} …, not {child} … {owner} …"));
                continue;
            }
            break;
        }
        Item::Path(segments)
    }

    /// A word `object` declares as a kind of its ids, followed by ids: taken as the kind.
    fn id_kind(&mut self, object: WordId) -> Option<Spanned<String>> {
        let kinds = &self.vocab.word(object).object.as_ref()?.id_kinds;
        let t = self.here()?;
        if t.kind != TokenKind::Word || !self.peek_at(1).is_some_and(|n| self.starts_ids(n)) {
            return None;
        }
        let kind = kinds.iter().find(|k| k.eq_ignore_ascii_case(self.text(t)))?.clone();
        self.pos += 1;
        self.expect(&[Expect::Ids]);
        Some(Spanned::new(kind, t.span))
    }

    fn ids(&mut self, object: Option<WordId>) -> Option<Ids> {
        let first = self.range(object)?;
        let mut ranges = vec![(None, first)];
        loop {
            self.expect(&[Expect::Keyword("Thru")]);
            let Some(t) = self.here() else { break };
            match t.kind {
                TokenKind::Op(c @ ('+' | '-')) if self.peek_at(1).is_some_and(|n| self.starts_ids(n)) => {
                    self.check_spaced(t);
                    self.pos += 1;
                    self.expect(&[Expect::Ids]);
                    match self.range(object) {
                        Some(r) => ranges.push((Some(Spanned::new(set_op(c), t.span)), r)),
                        None => break,
                    }
                }
                TokenKind::Number if matches!(self.text(t).as_bytes()[0], b'+' | b'-') => {
                    let sign = self.text(t).as_bytes()[0] as char;
                    self.diag(codes::GLUED_OPERATOR, t.span, format!("space the {sign}: {sign} {}", &self.text(t)[1..]));
                    self.pos += 1;
                    let number = Spanned::new(self.text(t)[1..].to_string(), Span { start: t.span.start + 1, end: t.span.end });
                    ranges.push((Some(Spanned::new(set_op(sign), Span { start: t.span.start, end: t.span.start + 1 })), Range::One(Id::Number(number))));
                }
                _ => break,
            }
        }
        Some(Ids { ranges })
    }

    fn range(&mut self, object: Option<WordId>) -> Option<Range> {
        let t = self.here()?;
        if self.is_keyword(t, "Thru") {
            self.pos += 1;
            self.expect(&[Expect::Ids]);
            let to = self.id(object);
            let span = to.as_ref().map_or(t.span, |id| t.span.to(id.span()));
            return Some(Range::Thru { from: None, to, span });
        }
        let from = self.id(object)?;
        self.expect(&[Expect::Keyword("Thru")]);
        match self.here() {
            Some(thru) if self.is_keyword(thru, "Thru") => {
                self.pos += 1;
                self.expect(&[Expect::Ids]);
                let to = self.id(object);
                let span = to.as_ref().map_or(from.span().to(thru.span), |id| from.span().to(id.span()));
                Some(Range::Thru { from: Some(from), to, span })
            }
            _ => Some(Range::One(from)),
        }
    }

    fn id(&mut self, object: Option<WordId>) -> Option<Id> {
        let t = self.here()?;
        match t.kind {
            TokenKind::Number => {
                self.pos += 1;
                let text = self.text(t);
                if let Some(o) = object.and_then(|o| self.vocab.word(o).object.as_ref().map(|d| (o, d.id_parts)))
                    && o.1 > 0
                    && text.split('.').count() > o.1 as usize
                {
                    let name = &self.vocab.word(o.0).name;
                    let parts = o.1;
                    self.diag(codes::ID_PARTS, t.span, format!("{name} numbers take {parts} part{}", if parts == 1 { "" } else { "s" }));
                }
                Some(Id::Number(Spanned::new(text.to_string(), t.span)))
            }
            TokenKind::Quoted { .. } => {
                self.pos += 1;
                Some(Id::Name(Spanned::new(unquote(self.text(t)).to_string(), t.span)))
            }
            _ => None,
        }
    }

    /// `1+3`: inside a list an operator stands alone (decision 11).
    fn check_spaced(&mut self, op: Token) {
        let prev = self.pos.checked_sub(1).map(|i| self.toks[i]);
        let next = self.peek_at(1);
        let glued = prev.is_some_and(|p| p.span.end == op.span.start) || next.is_some_and(|n| n.span.start == op.span.end);
        if glued {
            let c = self.text(op);
            self.diag(codes::GLUED_OPERATOR, op.span, format!("space the {c}: 1 {c} 3, not 1{c}3"));
        }
    }

    // ── slots and values ──

    fn slot(&mut self) -> Option<Slot> {
        let t = self.here()?;
        if let Some(w) = self.word(t)
            && self.vocab.word(w).helper
        {
            self.pos += 1;
            self.expect(&[Expect::Value, Expect::Object, Expect::Tree]);
            let value = match self.here() {
                Some(v) if v.kind != TokenKind::Option && !self.is_keyword(v, "If") => self.value(),
                _ => None,
            };
            return Some(Slot::Helper { word: Spanned::new(w, t.span), value });
        }
        self.value().map(Slot::Value)
    }

    fn value(&mut self) -> Option<Value> {
        let t = self.here()?;
        match t.kind {
            TokenKind::Number => {
                self.pos += 1;
                let from = Spanned::new(self.text(t).to_string(), t.span);
                self.expect(&[Expect::Keyword("Thru")]);
                if self.here().is_some_and(|n| self.is_keyword(n, "Thru")) && self.peek_at(1).is_some_and(|n| n.kind == TokenKind::Number) {
                    self.pos += 1;
                    self.expect(&[Expect::Value]);
                    let to = self.bump();
                    return Some(Value::Spread { from, to: Spanned::new(self.text(to).to_string(), to.span) });
                }
                Some(Value::Number(from))
            }
            TokenKind::Op(c) => {
                self.pos += 1;
                self.expect(&[Expect::Value]);
                match self.here() {
                    Some(n) if n.kind == TokenKind::Number => {
                        self.pos += 1;
                        Some(Value::Relative { op: Spanned::new(c, t.span), number: Spanned::new(self.text(n).to_string(), n.span) })
                    }
                    _ => {
                        self.diag(codes::MISSING, t.span, format!("{c} needs a number after it: {c} 10"));
                        None
                    }
                }
            }
            TokenKind::Word if self.object(t).is_some() || self.is_keyword(t, "Self") => self.target().map(Value::Target),
            TokenKind::Dotted if !is_read(self.text(t)) => self.target().map(Value::Target),
            _ => self.single_value(),
        }
    }

    /// One token as a value: a word, quoted text, a read, a number (an option's value).
    fn single_value(&mut self) -> Option<Value> {
        let t = self.here()?;
        match t.kind {
            TokenKind::Number => {
                self.pos += 1;
                Some(Value::Number(Spanned::new(self.text(t).to_string(), t.span)))
            }
            TokenKind::Quoted { .. } => {
                self.pos += 1;
                Some(Value::Text(Spanned::new(unquote(self.text(t)).to_string(), t.span)))
            }
            TokenKind::Line { .. } => {
                self.pos += 1;
                let text = self.text(t);
                let inner = text.strip_prefix('`').unwrap_or(text);
                Some(Value::Line(Spanned::new(inner.strip_suffix('`').unwrap_or(inner).to_string(), t.span)))
            }
            TokenKind::Dotted if is_read(self.text(t)) => {
                self.pos += 1;
                let parts = split_path(self.text(t)).into_iter().skip(1).collect();
                Some(Value::Read(Spanned::new(parts, t.span)))
            }
            TokenKind::Dotted => self.target().map(Value::Target),
            // An object with its ids is a target, though its word is a value too (`/As Fixture
            // 104`; `Fixture` alone stays the value word).
            TokenKind::Word if self.object(t).is_some() && self.peek_at(1).is_some_and(|n| self.starts_ids(n)) => self.target().map(Value::Target),
            TokenKind::Word => {
                self.pos += 1;
                let text = self.text(t);
                match self.vocab.lookup(text) {
                    Some(w) if self.vocab.word(w).property => {
                        self.diag(codes::BARE_PROPERTY, t.span, format!("quote the property: \"{}\"", self.vocab.word(w).name));
                        Some(Value::Text(Spanned::new(self.vocab.word(w).name.clone(), t.span)))
                    }
                    Some(w) if self.vocab.word(w).value || self.vocab.word(w).verb.is_some() => Some(Value::Word(Spanned::new(w, t.span))),
                    Some(w) if self.vocab.word(w).object.is_some() => {
                        self.pos -= 1;
                        self.target().map(Value::Target)
                    }
                    Some(_) | None if ["Thru", "EndIf"].iter().any(|k| text.eq_ignore_ascii_case(k)) => {
                        self.diag(codes::UNEXPECTED, t.span, format!("'{text}' can't stand here"));
                        None
                    }
                    _ => {
                        self.unknown_word(t);
                        None
                    }
                }
            }
            _ => None,
        }
    }
}

fn set_op(c: char) -> SetOp {
    if c == '+' { SetOp::Plus } else { SetOp::Minus }
}

fn unquote(text: &str) -> &str {
    let inner = text.strip_prefix('"').unwrap_or(text);
    inner.strip_suffix('"').unwrap_or(inner)
}

fn is_read(text: &str) -> bool {
    text.split('.').next().is_some_and(|p| p.eq_ignore_ascii_case("Self"))
}

fn is_root_path(text: &str) -> bool {
    text.split('.').next().is_some_and(|p| p.eq_ignore_ascii_case("Root"))
}

/// `Root.ShowData.Stations."Tower"` → its parts, quoted parts kept with their quotes.
fn split_path(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut part = String::new();
    let mut quoted = false;
    for c in text.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                part.push(c);
            }
            '.' if !quoted => parts.push(std::mem::take(&mut part)),
            _ => part.push(c),
        }
    }
    parts.push(part);
    parts
}
