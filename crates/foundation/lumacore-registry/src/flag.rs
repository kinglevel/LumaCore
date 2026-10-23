// ███████╗██╗      █████╗  ██████╗
// ██╔════╝██║     ██╔══██╗██╔════╝
// █████╗  ██║     ███████║██║  ███╗
// ██╔══╝  ██║     ██╔══██║██║   ██║
// ██║     ███████╗██║  ██║╚██████╔╝
// ╚═╝     ╚══════╝╚═╝  ╚═╝ ╚═════╝
// FILE: flag
//! A declared flag (decisions §31 "Flags are declared"): its name, short forms, what follows it,
//! the verbs it goes with, its help line and its fixed choices. Pure data: a command's, an
//! object's and the global flags are catalog rows (decisions §34 21).
//!
//! Owns:          FlagDef, FlagValue and how a flag is written (usage)
//! Doesn't own:   which flags exist (each crate's rows), reading them off a line (state's flags)
//! Entry points:  FlagDef::{new, value, rest, inline, alias, choices, on, named, usage}
//! Decisions:     §31, §34 21

use crate::verb::Verb;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What follows a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagValue {
    /// A switch (`/Overwrite`).
    None,
    /// The next word (`/Port 8000`), or `/Port=8000`; the text names it for help.
    One(&'static str),
    /// Every word up to the next flag (`/Tab Lines In`, `/Description "…"`).
    Rest(&'static str),
    /// Only inline, optional (`/Watch`, `/Watch=500`, `/d=2`).
    Inline(&'static str),
}

/// One declared flag.
#[derive(Debug, Clone, Copy)]
pub struct FlagDef {
    /// Its name as written in help (`Overwrite`), without the slash.
    pub name: &'static str,
    /// Short forms (`o`), without the slash.
    pub aliases: &'static [&'static str],
    pub value: FlagValue,
    /// On an object: the verbs it goes with (empty = every verb of the object). On a command
    /// or a global flag: empty.
    pub verbs: &'static [Verb],
    /// One line for help and the reference.
    pub doc: &'static str,
    /// The value words it takes, when they are a fixed set (`/By Family`).
    pub choices: &'static [&'static str],
}

// ██╗███╗   ██╗██╗████████╗
// ██║████╗  ██║██║╚══██╔══╝
// ██║██╔██╗ ██║██║   ██║
// ██║██║╚██╗██║██║   ██║
// ██║██║ ╚████║██║   ██║
// ╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝
// PART: INIT

impl FlagDef {
    pub const fn new(name: &'static str, doc: &'static str) -> Self {
        Self { name, aliases: &[], value: FlagValue::None, verbs: &[], doc, choices: &[] }
    }

    /// Its value is one of these words (`/By Name|Family|MainFirst`).
    pub const fn choices(self, choices: &'static [&'static str]) -> Self {
        Self { choices, ..self }
    }

    pub const fn alias(self, aliases: &'static [&'static str]) -> Self {
        Self { aliases, ..self }
    }

    /// Takes the next word (or `=value`).
    pub const fn value(self, hint: &'static str) -> Self {
        Self { value: FlagValue::One(hint), ..self }
    }

    /// Takes the words up to the next flag.
    pub const fn rest(self, hint: &'static str) -> Self {
        Self { value: FlagValue::Rest(hint), ..self }
    }

    /// Takes an optional inline `=value` only.
    pub const fn inline(self, hint: &'static str) -> Self {
        Self { value: FlagValue::Inline(hint), ..self }
    }

    /// On an object: only with these verbs.
    pub const fn on(self, verbs: &'static [Verb]) -> Self {
        Self { verbs, ..self }
    }

    /// Whether `word` (without the slash, any case) names this flag.
    pub fn named(&self, word: &str) -> bool {
        self.name.eq_ignore_ascii_case(word) || self.aliases.iter().any(|a| a.eq_ignore_ascii_case(word))
    }

    /// Whether it goes with `verb` (an object's flag).
    pub fn with_verb(&self, verb: Verb) -> bool {
        self.verbs.is_empty() || self.verbs.contains(&verb)
    }

    /// How it is written: `/Port <port>`, `/Tab <tab…>`, `/Watch[=<ms>]`, short forms after it
    /// (`/Overwrite (/o)`) — for help and the reference.
    pub fn usage(&self) -> String {
        let mut text = format!("/{}", self.name);
        match self.value {
            FlagValue::None => {}
            FlagValue::One(hint) => text.push_str(&format!(" <{hint}>")),
            FlagValue::Rest(hint) => text.push_str(&format!(" <{hint}…>")),
            FlagValue::Inline(hint) => text.push_str(&format!("[=<{hint}>]")),
        }
        if !self.aliases.is_empty() {
            let short: Vec<String> = self.aliases.iter().map(|a| format!("/{a}")).collect();
            text.push_str(&format!(" ({})", short.join(", ")));
        }
        text
    }

    /// The verbs it goes with, for an object's help (`Store, Copy`); empty = every verb.
    pub fn verbs_label(&self) -> String {
        self.verbs.iter().map(|v| v.label()).collect::<Vec<_>>().join(", ")
    }
}
