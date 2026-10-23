//! The lexer: one pass, total, every token with its span (lum.md "Tokens").
//!
//! Whitespace separates tokens and is not kept — the spans show where it was, so the parser
//! can tell a glued operator (`1+3`) from a spaced one (`1 + 3`).

use crate::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// `Store`, `Go-` (a glued trailing sign only before a space, `;` or the end).
    Word,
    /// `3`, `1.5`, `2.1.3`, `-120` (a sign glued to digits after a space belongs to the number).
    Number,
    /// `"Front"`; `terminated: false` when the line ends inside it.
    Quoted { terminated: bool },
    /// `` `Set Self "AutoScroll" Toggle` `` — a command line as a value (decision 91).
    Line { terminated: bool },
    /// `Root.ShowData.DataPools.2`, `Self.Name`, `DataPools.1.Groups.1` — one word with dots.
    Dotted,
    /// `/Overwrite` — the slash and the name.
    Option,
    /// `+ - * /` standing as their own token.
    Op(char),
    /// `..` (the tree shell's parent).
    DotDot,
    /// `~`, `~usb1`, `~usb1.mvr` — the local library, a drive's library, a path in it (decisions §38).
    Home,
    /// `!St` — an alias, the marker and the name.
    Alias,
    /// `?` (glued after a word: its quick hint).
    Question,
    Semi,
    /// Anything else (reported by the parser).
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn text(self, src: &str) -> &str {
        &src[self.span.range()]
    }
}

/// Split a line into tokens. Never fails: what isn't a token becomes [`TokenKind::Unknown`].
pub fn lex(src: &str) -> Vec<Token> {
    let bytes: Vec<(usize, char)> = src.char_indices().collect();
    let at = |i: usize| bytes.get(i).map(|&(_, c)| c);
    let offset = |i: usize| bytes.get(i).map_or(src.len(), |&(o, _)| o);
    let mut tokens = Vec::new();
    let mut i = 0;
    while let Some(c) = at(i) {
        let start = i;
        let prev_is_gap = start == 0 || at(start - 1).is_some_and(|p| p.is_whitespace() || p == ';');
        let kind = match c {
            c if c.is_whitespace() => {
                i += 1;
                continue;
            }
            ';' => {
                i += 1;
                TokenKind::Semi
            }
            '"' => {
                i += 1;
                while at(i).is_some_and(|c| c != '"') {
                    i += 1;
                }
                let terminated = at(i) == Some('"');
                if terminated {
                    i += 1;
                }
                TokenKind::Quoted { terminated }
            }
            '`' => {
                i += 1;
                while at(i).is_some_and(|c| c != '`') {
                    i += 1;
                }
                let terminated = at(i) == Some('`');
                if terminated {
                    i += 1;
                }
                TokenKind::Line { terminated }
            }
            '+' | '-' if prev_is_gap && at(i + 1).is_some_and(|d| d.is_ascii_digit()) => {
                i = number_end(&at, i + 1);
                TokenKind::Number
            }
            d if d.is_ascii_digit() => {
                i = number_end(&at, i);
                TokenKind::Number
            }
            '/' if at(i + 1).is_some_and(is_letter) => {
                i = name_end(&at, i + 1);
                TokenKind::Option
            }
            '!' if at(i + 1).is_some_and(is_letter) => {
                i = name_end(&at, i + 1);
                TokenKind::Alias
            }
            '+' | '-' | '*' | '/' => {
                i += 1;
                TokenKind::Op(c)
            }
            '.' if at(i + 1) == Some('.') => {
                i += 2;
                TokenKind::DotDot
            }
            '~' => {
                i += 1;
                if at(i).is_some_and(is_letter) {
                    i = name_end(&at, i);
                    while at(i) == Some('.') && at(i + 1).is_some_and(part_start) {
                        i = part_end(&at, i + 1);
                    }
                }
                TokenKind::Home
            }
            '?' => {
                i += 1;
                TokenKind::Question
            }
            l if is_letter(l) => {
                i = name_end(&at, i);
                if at(i) == Some('.') && at(i + 1).is_some_and(part_start) {
                    while at(i) == Some('.') && at(i + 1).is_some_and(part_start) {
                        i = part_end(&at, i + 1);
                    }
                    TokenKind::Dotted
                } else {
                    if matches!(at(i), Some('+' | '-'))
                        && at(i + 1).is_none_or(|n| n.is_whitespace() || n == ';')
                    {
                        i += 1;
                    }
                    TokenKind::Word
                }
            }
            _ => {
                i += 1;
                TokenKind::Unknown
            }
        };
        tokens.push(Token { kind, span: Span::new(offset(start), offset(i)) });
    }
    tokens
}

fn is_letter(c: char) -> bool {
    c.is_alphabetic()
}

fn part_start(c: char) -> bool {
    c.is_alphanumeric() || c == '"'
}

fn name_end(at: &impl Fn(usize) -> Option<char>, mut i: usize) -> usize {
    while at(i).is_some_and(|c| c.is_alphanumeric() || c == '_') {
        i += 1;
    }
    i
}

fn number_end(at: &impl Fn(usize) -> Option<char>, mut i: usize) -> usize {
    while at(i).is_some_and(|c| c.is_ascii_digit()) {
        i += 1;
    }
    while at(i) == Some('.') && at(i + 1).is_some_and(|c| c.is_ascii_digit()) {
        i += 1;
        while at(i).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
        }
    }
    i
}

/// One part of a dotted word: a quoted name or a run of letters, digits and `_`.
fn part_end(at: &impl Fn(usize) -> Option<char>, mut i: usize) -> usize {
    if at(i) == Some('"') {
        i += 1;
        while at(i).is_some_and(|c| c != '"') {
            i += 1;
        }
        return if at(i) == Some('"') { i + 1 } else { i };
    }
    name_end(at, i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(TokenKind, &str)> {
        lex(src).into_iter().map(|t| (t.kind, t.text(src))).collect()
    }

    #[test]
    fn words_numbers_quotes_options() {
        use TokenKind::*;
        assert_eq!(
            kinds(r#"Store Group 3 "Front" /Merge"#),
            vec![(Word, "Store"), (Word, "Group"), (Number, "3"), (Quoted { terminated: true }, "\"Front\""), (Option, "/Merge")]
        );
    }

    #[test]
    fn a_sign_glued_after_a_gap_is_the_number() {
        use TokenKind::*;
        assert_eq!(kinds("At -120"), vec![(Word, "At"), (Number, "-120")]);
        assert_eq!(kinds("At - 10"), vec![(Word, "At"), (Op('-'), "-"), (Number, "10")]);
        assert_eq!(kinds("1+3"), vec![(Number, "1"), (Op('+'), "+"), (Number, "3")]);
        assert_eq!(kinds("5-2"), vec![(Number, "5"), (Op('-'), "-"), (Number, "2")]);
    }

    #[test]
    fn dotted_words_and_numbers() {
        use TokenKind::*;
        assert_eq!(kinds("Cue 1.5"), vec![(Word, "Cue"), (Number, "1.5")]);
        assert_eq!(kinds("Root.ShowData.DataPools.2"), vec![(Dotted, "Root.ShowData.DataPools.2")]);
        assert_eq!(kinds(r#"Root.ShowData.Stations."Tower".Networks"#), vec![(Dotted, r#"Root.ShowData.Stations."Tower".Networks"#)]);
        assert_eq!(kinds("Self.DataPool.Name"), vec![(Dotted, "Self.DataPool.Name")]);
    }

    #[test]
    fn go_minus_is_one_word_only_before_a_gap() {
        use TokenKind::*;
        assert_eq!(kinds("Go- Sequence 2"), vec![(Word, "Go-"), (Word, "Sequence"), (Number, "2")]);
        assert_eq!(kinds("Go-;Go"), vec![(Word, "Go-"), (Semi, ";"), (Word, "Go")]);
        assert_eq!(kinds("Go-x"), vec![(Word, "Go"), (Op('-'), "-"), (Word, "x")]);
    }

    #[test]
    fn fence_pieces_alias_hint_and_unterminated_quotes() {
        use TokenKind::*;
        assert_eq!(kinds("cd .."), vec![(Word, "cd"), (DotDot, "..")]);
        assert_eq!(kinds("cd ~usb1.mvr"), vec![(Word, "cd"), (Home, "~usb1.mvr")]);
        assert_eq!(kinds("ls ~"), vec![(Word, "ls"), (Home, "~")]);
        assert_eq!(kinds("ls -r"), vec![(Word, "ls"), (Op('-'), "-"), (Word, "r")]);
        assert_eq!(kinds("!St 3"), vec![(Alias, "!St"), (Number, "3")]);
        assert_eq!(kinds("store?"), vec![(Word, "store"), (Question, "?")]);
        assert_eq!(kinds("\"open"), vec![(Quoted { terminated: false }, "\"open")]);
    }

    #[test]
    fn spans_cover_multibyte_text() {
        let src = "Label Group 1 \"Bühne\"";
        for t in lex(src) {
            assert!(src.is_char_boundary(t.span.start as usize) && src.is_char_boundary(t.span.end as usize));
        }
    }
}
