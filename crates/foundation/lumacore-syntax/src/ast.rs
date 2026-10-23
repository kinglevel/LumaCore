//! What a line means: the statement tree the parser builds (lum.md "Grammar").
//!
//! The tree keeps every piece with its span. It is generic — which slot is a property, a
//! value or a destination is the verb's signature's business (decision 7, 8, 17), applied when
//! the statement is bound to it.

use crate::{Span, WordId};

/// A piece of the line with where it was.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub span: Span,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    /// Nothing between two `;`.
    Empty,
    Lum(Lum),
    /// `!St 3 "Front"` — the alias's name and the words after it, as written (decision 13).
    Alias { name: Spanned<String>, rest: Spanned<String> },
    /// A tree-shell statement (decision 18).
    Fence { word: Spanned<WordId>, args: Vec<FenceArg> },
    /// `store?` — the word's quick hint.
    Hint { word: Spanned<String> },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Lum {
    /// `+ Group 3` / `- Fixture 2` — only with no verb.
    pub select_op: Option<Spanned<SetOp>>,
    pub verb: Option<Spanned<WordId>>,
    /// `On` / `Off` after a hold verb.
    pub phase: Option<Spanned<Phase>>,
    pub target: Option<Target>,
    pub slots: Vec<Slot>,
    pub condition: Option<Condition>,
    pub options: Vec<OptionUse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOp {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    On,
    Off,
}

/// What a statement acts on: items joined by `+` / `-`.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    /// The first item has no operator.
    pub items: Vec<(Option<Spanned<SetOp>>, Item)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// `DataPool 2 Group 3` — owner first.
    Path(Vec<Segment>),
    /// `Root.ShowData.DataPools.2` — node names with dots (decision 49).
    Tree(Spanned<Vec<String>>),
    /// `Self` — the window or object the line belongs to.
    SelfRef(Span),
    /// `3`, `1 Thru 4`, `"Tower"` with no object: the verb's default object, else yours.
    Ids(Ids),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub object: Spanned<WordId>,
    /// A kind its object declares before its ids (`Fixture Houselights 4`), as declared.
    pub kind: Option<Spanned<String>>,
    pub ids: Option<Ids>,
}

/// `1 Thru 4 + 7 - 3` — the one range rule.
#[derive(Debug, Clone, PartialEq)]
pub struct Ids {
    pub ranges: Vec<(Option<Spanned<SetOp>>, Range)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Range {
    One(Id),
    /// `1 Thru 4`, `1 Thru` (to the last), `Thru 4` (from the first).
    Thru { from: Option<Id>, to: Option<Id>, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Id {
    /// Digits with optional dots: `3`, `1.5`, `2.1`.
    Number(Spanned<String>),
    /// Quoted text, without its quotes.
    Name(Spanned<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Slot {
    /// `At 50`, `Fade 2`, `At Output 1` — a helper and its value.
    Helper { word: Spanned<WordId>, value: Option<Value> },
    Value(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(Spanned<String>),
    /// `+ 10`, `- 10`, `* 2`, `/ 2`.
    Relative { op: Spanned<char>, number: Spanned<String> },
    /// `21 Thru 30` over several rows.
    Spread { from: Spanned<String>, to: Spanned<String> },
    /// A declared value word (`Full`, `HTP`, `On`) — its id in the vocabulary.
    Word(Spanned<WordId>),
    /// Quoted text, without its quotes.
    Text(Spanned<String>),
    /// A command line as a value, without its backticks (decision 91).
    Line(Spanned<String>),
    /// Several command lines in a row, each in its backticks: a key's press cycle (decisions
    /// 0295, ``Set UIKey "user.e" "Press" `Edit` `EditSetting` ``).
    Lines(Vec<Spanned<String>>),
    /// `Self.DataPool.Name`, the parts after `Self`.
    Read(Spanned<Vec<String>>),
    /// An object, a path or a tree path as a value: `At Output 1`, `Attribute "Pan"`.
    Target(Target),
}

/// `If <target>` at the end, or `If <target> EndIf` right after the verb (decision 23).
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub target: Target,
    /// Written right after the verb (`Store If Group 5 EndIf Preset 1.1`); printed at the end.
    pub early: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionUse {
    /// The canonical name, without the slash.
    pub name: Spanned<String>,
    pub value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FenceArg {
    /// `..`, `/`, `3`, `ShowData`, `DataPools.1.Groups.1`, `Root.ShowData`.
    Path(Spanned<String>),
    /// `-n 5`, `--type Group`, `-r`.
    Flag { name: Spanned<String>, long: bool, value: Option<Spanned<String>> },
    /// Quoted text, without its quotes.
    Text(Spanned<String>),
}

impl Target {
    pub fn span(&self) -> Span {
        let first = self.items.first().map(|(_, i)| i.span()).unwrap_or_default();
        self.items.iter().fold(first, |s, (op, i)| {
            let s = s.to(i.span());
            op.as_ref().map_or(s, |o| s.to(o.span))
        })
    }
}

impl Value {
    pub fn span(&self) -> Span {
        match self {
            Value::Number(n) | Value::Text(n) | Value::Line(n) => n.span,
            Value::Relative { op, number } => op.span.to(number.span),
            Value::Spread { from, to } => from.span.to(to.span),
            Value::Lines(ls) => match (ls.first(), ls.last()) {
                (Some(a), Some(b)) => a.span.to(b.span),
                _ => Span::default(),
            },
            Value::Word(w) => w.span,
            Value::Read(r) => r.span,
            Value::Target(t) => t.span(),
        }
    }
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Path(segments) => {
                let first = segments[0].object.span;
                segments.iter().fold(first, |s, seg| {
                    let s = s.to(seg.object.span);
                    seg.ids.as_ref().map_or(s, |ids| s.to(ids.span()))
                })
            }
            Item::Tree(t) => t.span,
            Item::SelfRef(s) => *s,
            Item::Ids(ids) => ids.span(),
        }
    }
}

impl Ids {
    pub fn span(&self) -> Span {
        let first = self.ranges.first().map(|(_, r)| r.span()).unwrap_or_default();
        self.ranges.iter().fold(first, |s, (_, r)| s.to(r.span()))
    }
}

impl Range {
    pub fn span(&self) -> Span {
        match self {
            Range::One(id) => id.span(),
            Range::Thru { span, .. } => *span,
        }
    }
}

impl Id {
    pub fn span(&self) -> Span {
        match self {
            Id::Number(n) | Id::Name(n) => n.span,
        }
    }
}
