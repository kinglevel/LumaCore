//! A command line parsed once (decisions §31 "Flags are declared and a line is parsed once").
//!
//! [`Line::parse`] lexes the line ([`super::tokens`]), splits it into statements at `;`,
//! resolves each statement's first word in the registry, and sorts its bare `/words` into the
//! flags it declares ([`super::flags`]) and the ones nobody declares. Rights, scope, reach,
//! undo, the asks and the handler all read the same [`Statement`].
//!
//! The args a handler gets stay exactly what [`super::tokens::args`] gives today (quoted words
//! keep their quotes, flags stay in place): the parse adds meaning, it doesn't change the text.

use std::ops::Range;

use super::flags::{FlagDef, FlagValue};
use super::registry::{self, Args, Word};
use super::tokens::{self, Kind, LexError, Token};
use crate::objects::{self, ObjectDef, Verb};

/// A whole command line.
#[derive(Clone)]
pub struct Line {
    pub text: String,
    /// One per `;`-separated statement (at least one; an empty line is one empty statement).
    pub statements: Vec<Statement>,
}

/// One statement of a line.
#[derive(Clone)]
pub struct Statement {
    /// Its text, trimmed (what [`tokens::statements`] gives).
    pub text: String,
    /// Its byte range in the line.
    pub span: Range<usize>,
    /// The first word in the registry; `Err` names why not (a bare number, a typo).
    pub word: Result<Word, String>,
    /// The verb it applies to objects named in it (`Store` for `Store Group 3`; `Call` for an
    /// object word itself).
    pub verb: Verb,
    /// The objects named in its arguments, in order (`DataPool 2 Group 3` → DataPool, Group).
    pub objects: Vec<&'static ObjectDef>,
    /// Every word without quotes, the first included (what [`tokens::words`] gives).
    pub words: Vec<String>,
    /// The words after the first, as handlers get them (quoted words keep their quotes).
    pub args: Vec<String>,
    /// The declared flags it carries.
    pub flags: Vec<FlagUse>,
    /// Bare `/words` no declaration takes (refused with E-CMD-018 once dispatch reads this).
    pub unknown_flags: Vec<String>,
}

/// A declared flag as used on a line.
#[derive(Debug, Clone)]
pub struct FlagUse {
    pub def: &'static FlagDef,
    /// Its value (`/Port 8000` → `8000`, `/Watch=500` → `500`), unquoted.
    pub value: Option<String>,
    /// The flag word as typed (`/o`).
    pub typed: String,
}

impl Line {
    /// The flags the statement being typed at the end of `before` (the line up to the word at
    /// the cursor) may take next, as `/Name` — what completion offers after a `/` (decisions
    /// §31). None when a `/` there is text or a path (`Echo`, `cd /`), or the line doesn't lex.
    pub fn flags_to_offer(before: &str) -> Option<Vec<String>> {
        let statement = Self::parse(before).ok()?.statements.pop()?;
        if statement.word.is_err() || statement.slash_words_are_text() {
            return None;
        }
        let mut out: Vec<String> = Vec::new();
        for def in statement.allowed_flags() {
            let name = format!("/{}", def.name);
            if statement.flag(def).is_none() && !out.contains(&name) {
                out.push(name);
            }
        }
        Some(out)
    }

    /// The refusal for the first flag nobody declares (E-CMD-018, decisions §31): it names the
    /// options the statement does take. None when every flag is declared.
    pub fn unknown_flag_refusal(&self) -> Option<String> {
        let (statement, flag) = self.unknown_flags().next()?;
        let mut own: Vec<String> = Vec::new();
        for def in statement.allowed_flags() {
            let name = format!("/{}", def.name);
            if !lumacore_registry::get().global_flags().any(|g| g.name == def.name) && !own.contains(&name) {
                own.push(name);
            }
        }
        let takes = if own.is_empty() { "none of its own".to_string() } else { own.join(", ") };
        Some(crate::errors::render_code(
            format!("'{flag}' isn't an option of {}: it takes {takes} — nothing ran.", statement.label()),
            "E-CMD-018",
        ))
    }

    /// Parse `text`; an unterminated quote is the lexer's error.
    pub fn parse(text: &str) -> Result<Self, LexError> {
        let tokens = tokens::lex(text)?;
        let mut statements = Vec::new();
        let mut from = 0;
        let mut part: Vec<Token> = Vec::new();
        for token in tokens {
            if token.kind == Kind::Semicolon {
                statements.extend(Statement::from_tokens(text, from..token.start, std::mem::take(&mut part)));
                from = token.end;
            } else {
                part.push(token);
            }
        }
        statements.extend(Statement::from_tokens(text, from..text.len(), part));
        if statements.is_empty() {
            statements.push(Statement::empty());
        }
        Ok(Self { text: text.to_string(), statements })
    }

    /// A line's statements as text, split where the lexer splits — a `;` inside quotes or a line
    /// in backticks (``Update `a; b` ``) stays in its statement; a line that doesn't lex splits
    /// at every `;` (dispatch then names its error).
    pub fn split(text: &str) -> Vec<String> {
        match Self::parse(text) {
            Ok(line) => line.statements.into_iter().map(|s| s.text).filter(|s| !s.trim().is_empty()).collect(),
            Err(_) => text.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        }
    }

    /// Every flag nobody declares, across the statements.
    pub fn unknown_flags(&self) -> impl Iterator<Item = (&Statement, &str)> {
        self.statements.iter().flat_map(|s| s.unknown_flags.iter().map(move |f| (s, f.as_str())))
    }
}

impl Statement {
    fn empty() -> Self {
        Self {
            text: String::new(),
            span: 0..0,
            word: Err(String::new()),
            verb: Verb::Call,
            objects: Vec::new(),
            words: Vec::new(),
            args: Vec::new(),
            flags: Vec::new(),
            unknown_flags: Vec::new(),
        }
    }

    /// The statement of `tokens` (the line's `range`); `None` when it is empty.
    fn from_tokens(line: &str, range: Range<usize>, tokens: Vec<Token>) -> Option<Self> {
        let text = line[range.clone()].trim();
        if text.is_empty() {
            return None;
        }
        let lead = line[range.clone()].len() - line[range.clone()].trim_start().len();
        let start = range.start + lead;
        let span = start..start + text.len();
        let first = tokens.first()?;
        let word = registry::resolve(&first.text);
        let words = tokens::words(&tokens);
        let args = tokens::args(&tokens[1..]);
        let objects: Vec<&'static ObjectDef> = tokens[1..]
            .iter()
            .filter(|t| matches!(t.kind, Kind::Word { quoted: false }))
            .filter_map(|t| objects::find(&t.text))
            .collect();
        let verb = match &word {
            Ok(Word::Command(cmd)) => cmd.object_verb.unwrap_or(Verb::Call),
            _ => Verb::Call,
        };
        let mut statement = Self {
            text: text.to_string(),
            span,
            word,
            verb,
            objects,
            words,
            args,
            flags: Vec::new(),
            unknown_flags: Vec::new(),
        };
        statement.sort_flags(&tokens[1..]);
        Some(statement)
    }

    /// The flags this statement may carry: its word's own, the named objects' for its verb,
    /// then the global ones (an own flag wins over a global of the same name).
    pub fn allowed_flags(&self) -> Vec<&'static FlagDef> {
        let mut allowed: Vec<&'static FlagDef> = Vec::new();
        match &self.word {
            Ok(Word::Command(cmd)) => allowed.extend(cmd.flags.iter()),
            Ok(Word::Object(obj)) => {
                let args: Vec<&str> = self.args.iter().map(String::as_str).collect();
                let verb = crate::objects::bare_verb(obj, &args);
                allowed.extend(obj.flags.iter().filter(|f| f.with_verb(verb)))
            }
            Err(_) => {}
        }
        for obj in &self.objects {
            allowed.extend(obj.flags.iter().filter(|f| f.with_verb(self.verb)));
        }
        allowed.extend(lumacore_registry::get().global_flags());
        allowed
    }

    /// The declared flag `name` (without the slash) as this statement understands it.
    fn find_flag(&self, name: &str) -> Option<&'static FlagDef> {
        self.allowed_flags().into_iter().find(|f| f.named(name))
    }

    /// Whether an unknown `/word` here is plain text, not a flag: free-text words (`Echo`) and
    /// words that take tree paths (`cd /ShowData`).
    pub fn slash_words_are_text(&self) -> bool {
        match &self.word {
            Ok(Word::Command(cmd)) => matches!(
                cmd.args,
                Args::Words | Args::TreeNav { .. } | Args::TreeFromRoot | Args::TreeItems { .. }
            ),
            _ => false,
        }
    }

    fn sort_flags(&mut self, tokens: &[Token]) {
        if self.word.is_err() {
            // A bare number, a typo, an edit context's own words: nothing to check against.
            return;
        }
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];
            i += 1;
            let Some(body) = flag_body(token) else { continue };
            let (name, inline) = match body.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (body, None),
            };
            let Some(def) = self.find_flag(name) else {
                if !self.slash_words_are_text() {
                    self.unknown_flags.push(token.text.clone());
                }
                continue;
            };
            let value = match def.value {
                FlagValue::None => None,
                FlagValue::Inline(_) => inline,
                FlagValue::One(_) => inline.or_else(|| {
                    let next = tokens.get(i).filter(|t| is_value(t))?;
                    i += 1;
                    Some(next.text.clone())
                }),
                FlagValue::Rest(_) => inline.or_else(|| {
                    let mut words = Vec::new();
                    while let Some(next) = tokens.get(i).filter(|t| is_value(t)) {
                        words.push(next.text.clone());
                        i += 1;
                    }
                    (!words.is_empty()).then(|| words.join(" "))
                }),
            };
            self.flags.push(FlagUse { def, value, typed: token.text.clone() });
        }
    }

    /// What it is called in a message: its word and the objects it names (`Store Group`).
    pub fn label(&self) -> String {
        let mut label = match &self.word {
            Ok(word) => word.label().to_string(),
            Err(_) => self.words.first().cloned().unwrap_or_default(),
        };
        if let Some(obj) = self.objects.last() {
            label.push(' ');
            label.push_str(obj.keyword);
        }
        label
    }

    /// The flag `def` as used here, if it is.
    pub fn flag(&self, def: &FlagDef) -> Option<&FlagUse> {
        self.flags.iter().find(|f| f.def.name == def.name)
    }
}

/// A bare `/word` (letter after the slash, no further slash): the word without its slash.
/// A quoted `"/word"` is a value (§15 Quoting), `/2` a relative number, `/a/b` a path.
fn flag_body(token: &Token) -> Option<&str> {
    if token.kind != (Kind::Word { quoted: false }) {
        return None;
    }
    let body = token.text.strip_prefix('/')?;
    let name = body.split('=').next().unwrap_or(body);
    (body.starts_with(|c: char| c.is_ascii_alphabetic()) && !name.contains('/')).then_some(body)
}

/// A token a flag may take as its value: a word that isn't itself a flag (`+` and `;` aren't).
fn is_value(token: &Token) -> bool {
    matches!(token.kind, Kind::Word { .. }) && flag_body(token).is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(text: &str) -> Statement {
        let line = Line::parse(text).expect("lexes");
        assert_eq!(line.statements.len(), 1, "{text}");
        line.statements.into_iter().next().unwrap()
    }

    fn names(s: &Statement) -> Vec<(&str, Option<&str>)> {
        s.flags.iter().map(|f| (f.def.name, f.value.as_deref())).collect()
    }

    #[test]
    fn statements_match_the_old_splitter() {
        for text in [
            "",
            "   ",
            "Fixture 1 At 50",
            "Fixture 1; At 50 ;  Oops",
            "Label Group 1 \"a; b\"; Group 1",
            ";;Store Group 3;",
            "Fixture 1 + 3 Thru 5 At 50",
        ] {
            let parsed: Vec<String> = Line::parse(text).unwrap().statements.into_iter().map(|s| s.text).collect();
            assert_eq!(parsed, tokens::statements(text), "{text:?}");
        }
    }

    #[test]
    fn args_are_what_handlers_get_today() {
        for text in ["Store Group 3 \"Front Wash\" /Overwrite", "Label Fixture 101 \"/nc\"", "Fixture 1 + 3 At 50"] {
            let s = one(text);
            let tokens = tokens::lex(text).unwrap();
            assert_eq!(s.args, tokens::args(&tokens[1..]), "{text}");
        }
    }

    #[test]
    fn spans_point_into_the_line() {
        let text = "  Fixture 1 ;  Group 2  ";
        for s in Line::parse(text).unwrap().statements {
            assert_eq!(&text[s.span.clone()], s.text);
        }
    }

    #[test]
    fn an_objects_flags_go_with_its_verb() {
        let s = one("Store Group 3 /o /Merge");
        assert_eq!(names(&s), vec![("Overwrite", None), ("Merge", None)]);
        assert!(s.unknown_flags.is_empty());
        // Delete Group takes no /Merge; Compact takes /By (decisions §35).
        let s = one("Delete Group 3 /Merge");
        assert_eq!(s.unknown_flags, vec!["/Merge"]);
        let s = one("Compact Group /By Name");
        assert!(s.unknown_flags.is_empty());
    }

    #[test]
    fn unknown_flags_are_collected() {
        let s = one("Store Group 5 /Foo /Overwite");
        assert_eq!(s.unknown_flags, vec!["/Foo", "/Overwite"]);
    }

    #[test]
    fn global_flags_go_on_every_line_and_an_own_flag_wins() {
        let s = one("Fixture 1 At 50 /nc /NoOops");
        assert_eq!(names(&s), vec![("NoConfirmation", None), ("NoOops", None)]);
        let s = one("List Group /d");
        assert_eq!(names(&s), vec![("Debug", None)]);
    }

    #[test]
    fn slashes_that_are_not_flags() {
        // Quoted: a value (§15 Quoting).
        let s = one("Label Fixture 101 \"/Foo\"");
        assert!(s.flags.is_empty() && s.unknown_flags.is_empty());
        // A relative number, a path, a tree path, free text.
        assert!(one("Fixture 1 At /2").unknown_flags.is_empty());
        assert!(one("Set AssetRoot \"Video\" \"Folder\" /Volumes/Media").unknown_flags.is_empty());
        assert!(one("Echo hello /Foo").unknown_flags.is_empty());
        // A line whose first word isn't a word (a bare number) isn't checked.
        assert!(one("101 /Foo").unknown_flags.is_empty());
    }

    #[test]
    fn a_nested_objects_flags_count() {
        let s = one("Store DataPool 2 Group 3 /Overwrite");
        assert!(s.unknown_flags.is_empty(), "{:?}", s.unknown_flags);
        assert_eq!(s.verb, Verb::Store);
        assert_eq!(s.objects.iter().map(|o| o.keyword).collect::<Vec<_>>(), vec!["DataPool", "Group"]);
    }
}
