// ████████╗ ██████╗ ██╗  ██╗███████╗███╗   ██╗███████╗
// ╚══██╔══╝██╔═══██╗██║ ██╔╝██╔════╝████╗  ██║██╔════╝
//    ██║   ██║   ██║█████╔╝ █████╗  ██╔██╗ ██║███████╗
//    ██║   ██║   ██║██╔═██╗ ██╔══╝  ██║╚██╗██║╚════██║
//    ██║   ╚██████╔╝██║  ██╗███████╗██║ ╚████║███████║
//    ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝╚══════╝
// FILE: tokens

//! The command-line lexer — the one tokenizer for dispatch, `;` splitting and
//! completion. Tokens carry their byte span so errors and completion can point
//! at the exact place on the line.
//!
//! - Whitespace separates tokens; `"…"` keeps spaces inside one token (the
//!   quote characters themselves are dropped; `""` alone makes no token).
//! - `+` outside quotes is its own token (selection algebra).
//! - `;` outside quotes separates statements.
//! - `` `…` `` is a command line given as a value (decisions §33 91): one token, its quotes,
//!   `;` and `+` plain inside; it is never text and never a keyword.
//! - An unterminated quote or backtick is an error at its opening.
//!
//! Owns:          the station's command-line lexer: tokens with spans, words, args, statements, unquote.
//! Doesn't own:   Lum's parser (lumacore-syntax), what a word means (the registries).
//! Entry points:  lex, lex_lenient, words, args, statements, unquote, line_of, word_start.
//! Decisions:     §15 "Quoting", §33 91, §34 15.

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a token is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A word; `quoted` when any part of it was inside quotes.
    Word { quoted: bool },
    /// `+`
    Plus,
    /// `;` (statement separator)
    Semicolon,
    /// `` `List Fixture` `` — a command line as a value, its text without the backticks.
    Line,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    /// The text, without quote characters.
    pub text: String,
    /// Byte offsets of the token in the line (quotes included).
    pub start: usize,
    pub end: usize,
}

/// A line that can't be tokenized, with the byte offset it went wrong at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: &'static str,
    pub at: usize,
}

impl LexError {
    /// The error with the line and a caret under the offending place.
    pub fn render(&self, line: &str) -> String {
        let column = line[..self.at.min(line.len())].chars().count();
        format!("{}\n  {line}\n  {}^", self.message, " ".repeat(column))
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Tokenize a command line. With `lenient`, an unterminated quote runs to the
/// end of the line instead of failing (for completion while typing).
fn lex_with(line: &str, lenient: bool) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut text = String::new();
    let mut start: Option<usize> = None;
    let mut quoted = false;
    let mut quote_open: Option<usize> = None;

    let flush = |tokens: &mut Vec<Token>,
                 text: &mut String,
                 start: &mut Option<usize>,
                 quoted: &mut bool,
                 end: usize| {
        // A quoted `""` is a value too (an empty text: `Set User "x" "Password" ""`).
        if let Some(s) = start.take()
            && (!text.is_empty() || *quoted)
        {
            tokens.push(Token {
                kind: Kind::Word { quoted: *quoted },
                text: std::mem::take(text),
                start: s,
                end,
            });
        }
        text.clear();
        *quoted = false;
    };

    let mut line_open: Option<usize> = None;
    for (i, ch) in line.char_indices() {
        if let Some(open) = line_open {
            if ch == '`' {
                line_open = None;
                tokens.push(Token { kind: Kind::Line, text: std::mem::take(&mut text), start: open, end: i + 1 });
            } else {
                text.push(ch);
            }
            continue;
        }
        if let Some(_open) = quote_open {
            if ch == '"' {
                quote_open = None;
            } else {
                text.push(ch);
            }
            continue;
        }
        match ch {
            '`' => {
                flush(&mut tokens, &mut text, &mut start, &mut quoted, i);
                line_open = Some(i);
            }
            '"' => {
                quote_open = Some(i);
                quoted = true;
                start.get_or_insert(i);
            }
            c if c.is_whitespace() => flush(&mut tokens, &mut text, &mut start, &mut quoted, i),
            '+' | ';' => {
                flush(&mut tokens, &mut text, &mut start, &mut quoted, i);
                tokens.push(Token {
                    kind: if ch == '+' {
                        Kind::Plus
                    } else {
                        Kind::Semicolon
                    },
                    text: ch.to_string(),
                    start: i,
                    end: i + 1,
                });
            }
            c => {
                start.get_or_insert(i);
                text.push(c);
            }
        }
    }
    if let Some(open) = quote_open
        && !lenient
    {
        return Err(LexError {
            message: "Unterminated quote: add the closing \".",
            at: open,
        });
    }
    if let Some(open) = line_open {
        if !lenient {
            return Err(LexError { message: "Unterminated line: add the closing `.", at: open });
        }
        tokens.push(Token { kind: Kind::Line, text: std::mem::take(&mut text), start: open, end: line.len() });
    }
    flush(&mut tokens, &mut text, &mut start, &mut quoted, line.len());
    Ok(tokens)
}

/// Tokenize a command line; an unterminated quote is an error.
pub fn lex(line: &str) -> Result<Vec<Token>, LexError> {
    lex_with(line, false)
}

/// Tokenize a line that may still be being typed: an open quote runs to the end.
pub fn lex_lenient(line: &str) -> Vec<Token> {
    lex_with(line, true).unwrap_or_default()
}

/// The words of one statement as strings (`+` included, `;` not).
pub fn words(tokens: &[Token]) -> Vec<String> {
    tokens
        .iter()
        .filter(|t| t.kind != Kind::Semicolon)
        .map(|t| t.text.clone())
        .collect()
}

/// The words of one statement as handlers get them (`+` included, `;` not):
/// a quoted word keeps its quotes (`"Wash Test"`, `"/nc"`), so a handler can
/// tell a value from a keyword or an option — a bare `/word` is always an
/// option, a quoted word always a value (issue 07). The lexer has
/// no escapes, so a word never holds a quote of its own.
pub fn args(tokens: &[Token]) -> Vec<String> {
    tokens
        .iter()
        .filter(|t| t.kind != Kind::Semicolon)
        .map(|t| match t.kind {
            Kind::Word { quoted: true } => format!("\"{}\"", t.text),
            Kind::Line => format!("`{}`", t.text),
            _ => t.text.clone(),
        })
        .collect()
}

/// An argument without its quotes (`"Wash Test"` → `Wash Test`; a bare word
/// as it is): the text a handler stores.
/// A command-line argument's line (`` `List Fixture` `` → `List Fixture`); None = not a line
/// (decisions §33 91: quoted text is never a line).
pub fn line_of(arg: &str) -> Option<&str> {
    arg.strip_prefix('`').and_then(|a| a.strip_suffix('`'))
}

pub fn unquote(arg: &str) -> &str {
    arg.strip_prefix('"').and_then(|a| a.strip_suffix('"')).unwrap_or(arg)
}

/// Whether an argument was typed in quotes (a value, never a keyword or an
/// option).
pub fn is_quoted(arg: &str) -> bool {
    arg.len() >= 2 && arg.starts_with('"') && arg.ends_with('"')
}

/// Split a line into its `;`-separated statements (trimmed, non-empty; one
/// empty statement for an empty line). A line that doesn't tokenize is one
/// statement — dispatching it reports the error.
pub fn statements(line: &str) -> Vec<String> {
    let Ok(tokens) = lex(line) else {
        return vec![line.trim().to_string()];
    };
    let mut parts = Vec::new();
    let mut from = 0;
    for t in tokens.iter().filter(|t| t.kind == Kind::Semicolon) {
        parts.push(line[from..t.start].trim().to_string());
        from = t.end;
    }
    parts.push(line[from..].trim().to_string());
    parts.retain(|p| !p.is_empty());
    if parts.is_empty() {
        parts.push(String::new());
    }
    parts
}

/// Byte offset where the word under the cursor starts: the start of the token
/// containing `pos` (an open quote included, so `Group "Front W▸` completes
/// the whole quoted name), or `pos` itself after whitespace.
pub fn word_start(line: &str, pos: usize) -> usize {
    lex_lenient(&line[..pos])
        .last()
        .filter(|t| t.end == pos && t.kind != Kind::Semicolon)
        .map_or(pos, |t| t.start)
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
