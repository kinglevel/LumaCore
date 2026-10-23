//! Unified command registry — single source of truth for all LumaCore commands.
//!
//! Every command is registered here with its verb, aliases, help documentation
//! and its handler (`run`). Objects (Fixture, Window, …) publish themselves in
//! `terminal/objects`; commands and objects share one namespace, so a word
//! always means exactly one thing. Dispatch, help, completion, the language
//! family preview and undo/replication all read these definitions — there are
//! no other keyword lists.
//!
//! To add a command: add a `CommandDef` to its family's `ROWS` (`registry/`) with its `run`
//! handler. Help, resolution, completion and undo follow automatically.

use lumacore_tree::OopsCategory;

use super::run::{self, out};
use super::{
    control, help, import, info, list, oops, playback, session,
    show, windows,
};
use crate::objects::{ObjectDef, Verb};
pub use crate::defs::command::*;
pub use super::hooks::Hook;
use crate::rights::{claim_rights, clear_rights};
use crate::DispatchResult;

/// Whether `word` is `Oops` (its confirmation is asked before the line runs
/// or goes to the master — `oops::confirm_first`).
pub fn is_oops(word: &str) -> bool {
    is_command(word, "oops")
}

/// Whether `word` names the command `verb` (by any of its names).
pub fn is_command(word: &str, verb: &str) -> bool {
    matches!(resolve(word), Ok(Word::Command(c)) if c.verb == verb)
}

// ── The families ─────────────────────────────────────────────────────────

// Flags of the commands below (decisions §31); the shared ones live in `flags`.
use super::flags::{self, FlagDef};

/// LoadShow, SaveShow, NewShow, CleanStart, DeleteShow, Restart and Shutdown read one set.
use super::run::{DISCARD, DISCARD_CHANGES, PATCH_DEFAULT, PATCH_EDIT, PATCH_MINE};
/// `List` (any target) and `List Undo /<category>` (decisions §29 "11c").
const LIST_FLAGS: &[FlagDef] = &[
    // `List Reference /Broken`: Reference is a listed object (macro-built) that only lists.
    crate::references::BROKEN,
    FlagDef::new("ThisStation", "Oops steps of this station only"),
    FlagDef::new("Programmer", "Oops steps of this category"),
    FlagDef::new("Selection", "Oops steps of this category"),
    FlagDef::new("Show", "Oops steps of this category"),
    FlagDef::new("Patch", "Oops steps of this category"),
    FlagDef::new("UI", "Oops steps of this category"),
    FlagDef::new("Users", "Oops steps of this category"),
    FlagDef::new("Settings", "Oops steps of this category"),
    FlagDef::new("Keys", "Oops steps of this category"),
    FlagDef::new("Station", "Oops steps of this category"),
];
/// `Import <pool> …`, `Import UserProfile … /As`, `Import Asset … /Hash /Size`, `Import Show …`.
const IMPORT_FLAGS: &[FlagDef] = &[
    flags::OVERWRITE,
    flags::VERSION,
    flags::RECOVERY,
    super::media::AS,
    super::media::HASH,
    super::media::SIZE,
];

pub(crate) mod rows;

/// One family's rows: its commands and its objects with their behaviour (decisions §34 25.13).
pub(crate) struct Family {
    pub commands: &'static [CommandDef],
    pub objects: &'static [&'static ObjectDef],
    pub hooks: &'static [Hook],
}

/// Every family (decisions §34 25.13): each pushes its handlers and objects (`rows`) and its
/// catalog rows (`commands`, `objects`); read them back with [`all`] / `objects::all`, a row's
/// behaviour with [`run_of`] / `objects::def_of`.
pub(crate) const FAMILIES: &[Family] = &[
    Family { commands: rows::dispatch::ROWS, objects: rows::dispatch::OBJECTS, hooks: rows::dispatch::HOOKS },
    Family { commands: rows::inspection::ROWS, objects: rows::inspection::OBJECTS, hooks: rows::inspection::HOOKS },
    Family { commands: rows::programmer::ROWS, objects: rows::programmer::OBJECTS, hooks: rows::programmer::HOOKS },
    Family { commands: rows::playback::ROWS, objects: rows::playback::OBJECTS, hooks: rows::playback::HOOKS },
    Family { commands: rows::patch::ROWS, objects: rows::patch::OBJECTS, hooks: rows::patch::HOOKS },
    Family { commands: rows::users::ROWS, objects: rows::users::OBJECTS, hooks: rows::users::HOOKS },
    Family { commands: rows::windows::ROWS, objects: rows::windows::OBJECTS, hooks: rows::windows::HOOKS },
    Family { commands: rows::media::ROWS, objects: rows::media::OBJECTS, hooks: rows::media::HOOKS },
];

/// Adds every family's commands and objects to the catalog.
pub(crate) fn register(r: &mut lumacore_registry::Registry) {
    for f in FAMILIES {
        r.rows(f.commands);
        r.commands(f.commands.iter().map(|c| &c.decl));
        r.rows(f.objects);
        r.objects(f.objects.iter().map(|o| &o.decl));
        r.rows(f.hooks);
    }
}

/// Every command with its handler, from the catalog.
pub fn all() -> impl Iterator<Item = &'static CommandDef> {
    lumacore_registry::get().rows::<CommandDef>()
}

// ── One namespace: commands and objects ──────────────────────────────────────

/// A word on the command line: a command or an object. Both registries share
/// one namespace, so every name resolves to exactly one of these.
#[derive(Clone, Copy)]
pub enum Word {
    Command(&'static CommandDecl),
    Object(&'static ObjectDef),
}

impl Word {
    /// Display label ("At", "Fixture").
    pub fn label(&self) -> &'static str {
        match self {
            Word::Command(c) => {
                if c.doc.display.is_empty() {
                    c.verb
                } else {
                    c.doc.display
                }
            }
            Word::Object(o) => o.keyword,
        }
    }

    pub fn doc(&self) -> &'static Doc {
        match self {
            Word::Command(c) => &c.doc,
            Word::Object(o) => &o.doc,
        }
    }

    /// Published but not implemented yet.
    pub fn is_planned(&self) -> bool {
        matches!(self, Word::Command(c) if c.status == Status::Planned)
    }

    /// Undo/replication behaviour of the word used as the first token.
    pub fn undo(&self) -> Undo {
        match self {
            Word::Command(c) => c.undo,
            Word::Object(o) => o.call_undo,
        }
    }

    pub fn family(&self) -> Option<CommandFamily> {
        match self {
            Word::Command(c) => c.family,
            Word::Object(o) => o.family,
        }
    }

    /// The role needed to use the word as the first token. Calling an object
    /// needs Programmer.
    pub fn rights(&self) -> Rights {
        match self {
            Word::Command(c) => c.rights,
            Word::Object(o) => (o.rights)(o.default_verb, &[]),
        }
    }

    /// How the runtime schedules a line starting with this word.
    pub fn class(&self) -> CommandClass {
        match self {
            Word::Command(c) => c.class,
            Word::Object(o) => o.call_class,
        }
    }

    /// All names this word answers to.
    pub fn names(&self) -> Vec<&'static str> {
        match self {
            Word::Command(c) => c.names().collect(),
            Word::Object(o) => std::iter::once(o.keyword)
                .chain(o.aliases.iter().copied())
                .collect(),
        }
    }

    /// Same command or object (not just the same spelling).
    pub fn same(&self, other: &Word) -> bool {
        match (self, other) {
            (Word::Command(a), Word::Command(b)) => std::ptr::eq(*a, *b),
            (Word::Object(a), Word::Object(b)) => std::ptr::eq(*a, *b),
            _ => false,
        }
    }
}

/// Every name in the namespace with the word it belongs to.
pub fn all_names() -> impl Iterator<Item = (&'static str, Word)> {
    let commands = lumacore_registry::get().commands().flat_map(|c| c.names().map(move |n| (n, Word::Command(c))));
    let objects = crate::objects::all().flat_map(|o| {
        std::iter::once(o.keyword)
            .chain(o.aliases.iter().copied())
            .map(move |n| (n, Word::Object(o)))
    });
    commands.chain(objects)
}

/// Every command and object, for help.
pub fn all_words() -> impl Iterator<Item = Word> {
    lumacore_registry::get().commands().map(Word::Command).chain(
        crate::objects::all()
            .map(Word::Object),
    )
}

/// Where a command line runs in a LumaSession (decisions §10, §21 "6b"):
/// `Master` when any statement changes the show or a user's state — a
/// member sends the line to its master — else on this station.
pub fn line_scope(line: &str) -> Scope {
    match super::line::Line::parse(line) {
        Ok(parsed) => scope_of(&parsed),
        // A line that doesn't lex runs here and reports its error.
        Err(_) => Scope::Station,
    }
}

/// [`line_scope`] of a line parsed once (decisions §31).
pub fn scope_of(line: &super::line::Line) -> Scope {
    let master = line.statements.iter().any(|s| statement_scope(&s.words) == Scope::Master);
    if master { Scope::Master } else { Scope::Station }
}

/// How far a line reaches (decisions §25 "Step 10 hardening"): the furthest
/// of its statements' — each its command's reach, or the object's for the
/// verb when the command acts on an object (`Delete Show "x"` is Drive).
pub fn line_reach(line: &str) -> Reach {
    match super::line::Line::parse(line) {
        Ok(parsed) => reach_of(&parsed),
        Err(_) => Reach::Show,
    }
}

/// [`line_reach`] of a line parsed once (decisions §31).
pub fn reach_of(line: &super::line::Line) -> Reach {
    line.statements.iter().map(|s| statement_reach(&s.words)).max().unwrap_or(Reach::Show)
}

fn statement_reach(words: &[String]) -> Reach {
    let Some(first) = words.first() else {
        return Reach::Show;
    };
    // A path that names the disk (`List Drives`, `cd Root.Drives.Home`)
    // reaches it whatever the word (decisions §29, finding 5).
    let names_drives = words[1..].iter().any(|w| {
        use crate::tree_path::{Node, names_drive};
        let w = w.trim_matches('"');
        names_drive(w) || w.split('.').next().and_then(|first| Node::Root.child(first)) == Some(Node::Drives)
    });
    let path = if names_drives { Reach::Drive } else { Reach::Show };
    match resolve(first) {
        Ok(Word::Command(cmd)) => {
            let object = words
                .get(1)
                .and_then(|w| crate::objects::find(w))
                .map_or(Reach::Show, |obj| (obj.reach)(cmd.object_verb.unwrap_or(obj.default_verb)));
            cmd.reach.max(object).max(path)
        }
        // An object used as a verb on another object (`Watch Show`).
        Ok(Word::Object(obj)) => {
            let other = words
                .get(1)
                .and_then(|w| crate::objects::find(w))
                .map_or(Reach::Show, |o| (o.reach)(o.default_verb));
            let args: Vec<&str> = words.iter().skip(1).map(String::as_str).collect();
            (obj.reach)(crate::objects::bare_verb(obj, &args)).max(other).max(path)
        }
        Err(_) => path,
    }
}

fn statement_scope(words: &[String]) -> Scope {
    let Some(first) = words.first() else {
        return Scope::Client;
    };
    let args: Vec<&str> = words[1..].iter().map(String::as_str).collect();
    match resolve(first) {
        Ok(Word::Command(cmd)) if cmd.scope != Scope::Master => cmd.scope,
        // `Set Station "Name" …`, `Store Window 5`: the object decides.
        Ok(Word::Command(cmd)) => match args.first() {
            Some(target) => match crate::objects::find(target) {
                Some(obj) => (obj.scope)(cmd.object_verb.unwrap_or(obj.default_verb), &args[1..]),
                None => path_scope(target),
            },
            None => Scope::Master,
        },
        // Alone it sets your default object, saved with your user (decisions §33 21).
        Ok(Word::Object(obj)) if args.is_empty() && obj.lum.counted() => Scope::Master,
        Ok(Word::Object(obj)) if obj.call_undo.is_mutating() => (obj.scope)(crate::objects::bare_verb(obj, &args), &args),
        Ok(Word::Object(_)) => Scope::Client,
        // A bare number selects in Fixture context.
        Err(_) => Scope::Master,
    }
}

/// A tree path as a target (`Set Root.StationSettings.Location …`): the
/// station's own subtrees stay here.
fn path_scope(target: &str) -> Scope {
    use crate::tree_path::Node;
    // The first name below Root decides (the rest may be a property, not a node).
    let mut names = target.trim_matches('"').split('.').filter(|s| !s.is_empty());
    let first = names.next().filter(|s| !s.eq_ignore_ascii_case("root")).or_else(|| names.next());
    match first.and_then(|name| Node::Root.child(name)) {
        Some(Node::StationSettings | Node::Runtime) => Scope::Station,
        _ => Scope::Master,
    }
}

/// Every name of every word, lowercased and sorted, built once (decisions §31 "parse once"):
/// `(lowercase name, name, word, position in [`all_names`])`. Exact lookups are a binary
/// search, prefixes a range; the position keeps §15's order where it matters (the tie rule).
static NAME_INDEX: std::sync::LazyLock<Vec<(String, &'static str, Word, usize)>> =
    std::sync::LazyLock::new(|| {
        let mut index: Vec<_> = all_names()
            .enumerate()
            .map(|(i, (name, word))| (name.to_lowercase(), name, word, i))
            .collect();
        index.sort_by(|a, b| a.0.cmp(&b.0).then(a.3.cmp(&b.3)));
        index
    });

/// The index entries whose lowercase name starts with `lower`, in registry order.
fn names_starting_with(lower: &str) -> Vec<(&'static str, Word)> {
    let index = &*NAME_INDEX;
    let start = index.partition_point(|e| e.0.as_str() < lower);
    let mut found: Vec<_> = index[start..]
        .iter()
        .take_while(|e| e.0.starts_with(lower))
        .map(|e| (e.3, e.1, e.2))
        .collect();
    found.sort_by_key(|e| e.0);
    found.into_iter().map(|(_, name, word)| (name, word)).collect()
}

/// The word with exactly this name (case-insensitive).
pub fn lookup_word(name: &str) -> Option<Word> {
    let lower = name.to_lowercase();
    let index = &*NAME_INDEX;
    let at = index.partition_point(|e| e.0 < lower);
    index.get(at).filter(|e| e.0 == lower).map(|e| e.2)
}

/// `line` with every word that was typed as a prefix (`Sto Gro 3`)
/// written out in full (`Store Group 3`): stored lines (keys, later macros)
/// keep their meaning when a new registry word arrives (R1-71, Q11). Quoted
/// text, numbers, flags and exact names stay as typed.
pub fn full_words(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut word = String::new();
    let mut quoted = false;
    let flush = |word: &mut String, out: &mut String| {
        let full = (!word.is_empty()
            && word.chars().next().is_some_and(char::is_alphabetic)
            && lookup_word(word).is_none())
        .then(|| resolve(word).ok())
        .flatten();
        match full {
            Some(w) => out.push_str(w.label()),
            None => out.push_str(word),
        }
        word.clear();
    };
    for c in line.chars() {
        if c == '"' {
            if !quoted {
                flush(&mut word, &mut out);
            }
            quoted = !quoted;
            out.push(c);
        } else if quoted {
            out.push(c);
        } else if c.is_whitespace() || c == ';' {
            flush(&mut word, &mut out);
            out.push(c);
        } else {
            word.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// Resolve what the user typed to a command or object.
///
/// 1. An exact name (verb, keyword or alias).
/// 2. A prefix: unique once aliases of the same word are folded together; if
///    several words match and one's name is a prefix of all the others'
///    (`fix` → Fixture over FixtureType), that one.
/// 3. Otherwise an error, with typo suggestions.
pub fn resolve(input: &str) -> Result<Word, String> {
    if let Some(word) = lookup_word(input) {
        return Ok(word);
    }
    let lower = input.to_lowercase();

    // Matching names, grouped by word.
    let mut matches: Vec<(Word, Vec<&'static str>)> = Vec::new();
    for (name, word) in names_starting_with(&lower) {
        match matches.iter_mut().find(|(w, _)| w.same(&word)) {
            Some((_, names)) => names.push(name),
            None => matches.push((word, vec![name])),
        }
    }

    match matches.len() {
        0 => {}
        1 => return Ok(matches[0].0),
        _ => {
            let shortest = |names: &[&str]| names.iter().map(|n| n.len()).min().unwrap_or(0);
            let base = matches.iter().find(|(_, names)| {
                matches.iter().all(|(_, others)| {
                    others.iter().any(|o| {
                        names
                            .iter()
                            .any(|n| o.to_lowercase().starts_with(&n.to_lowercase()))
                    })
                })
            });
            if let Some((word, _)) = base {
                return Ok(*word);
            }
            let mut labels: Vec<(usize, &str)> = matches
                .iter()
                .map(|(w, names)| (shortest(names), w.label()))
                .collect();
            labels.sort();
            let labels: Vec<&str> = labels.into_iter().map(|(_, l)| l).collect();
            return Err(format!(
                "Ambiguous command '{input}': matches {}",
                labels.join(", ")
            ));
        }
    }

    // Typo suggestions (normalized Levenshtein ≥ 0.6), one per word.
    let mut suggestions: Vec<&str> = Vec::new();
    let mut seen: Vec<Word> = Vec::new();
    for (name, word) in all_names() {
        if strsim::normalized_levenshtein(&lower, &name.to_lowercase()) >= 0.6
            && !seen.iter().any(|w| w.same(&word))
        {
            seen.push(word);
            suggestions.push(name);
        }
    }
    if suggestions.is_empty() {
        Err(format!(
            "Unknown command: '{input}'. Type 'help' for a list of commands."
        ))
    } else {
        Err(format!(
            "Unknown command: '{input}'. Did you mean: {}?",
            suggestions.join(", ")
        ))
    }
}

/// Every name in the namespace (display labels too), for verb lookup in the
/// completer.
pub fn completion_keywords() -> Vec<String> {
    let mut keywords: Vec<String> = Vec::new();
    for word in all_words() {
        keywords.push(word.label().to_string());
        for name in word.names() {
            keywords.push(name.to_string());
            keywords.push(name.to_lowercase());
        }
    }
    let mut seen = std::collections::HashSet::new();
    keywords.retain(|k| seen.insert(k.clone()));
    keywords
}

/// First-word suggestions: **one per command or object**, matched
/// case-insensitively against all of its names. The display name is offered
/// when it matches (`log` → `Login`, `Logout`); an alias only when the display
/// name doesn't (`logof` → `logoff`). Single letters are never offered.
pub fn verb_suggestions(prefix: &str) -> Vec<String> {
    let p = prefix.to_lowercase();
    let mut out: Vec<String> = Vec::new();
    for word in all_words() {
        if word.doc().hidden {
            continue;
        }
        let mut names = vec![word.label()];
        names.extend(word.names());
        if let Some(pick) = names
            .iter()
            .find(|n| n.len() > 1 && n.to_lowercase().starts_with(&p))
            && !out.iter().any(|o| o.eq_ignore_ascii_case(pick))
        {
            out.push(pick.to_string());
        }
    }
    out
}

/// Look up a command definition by canonical verb name.
pub fn lookup(verb: &str) -> Option<&'static CommandDecl> {
    lumacore_registry::get().commands().find(|c| c.verb == verb)
}

/// Look up a command definition by verb OR alias (case-insensitive).
pub fn lookup_any(input: &str) -> Option<&'static CommandDecl> {
    lumacore_registry::get().command(input)
}

/// All visible (non-hidden) command definitions, for help rendering.
pub fn visible_commands() -> impl Iterator<Item = &'static CommandDecl> {
    lumacore_registry::get().commands().filter(|c| !c.doc.hidden)
}

/// What runs a command's line: the handler bound to its row (decisions §34 21, by the row's
/// address — a row and its handler are one `CommandDef`).
pub fn run_of(decl: &CommandDecl) -> Run {
    crate::commands::registry::all()
        .find(|c| std::ptr::eq(&c.decl, decl))
        .map_or(crate::defs::command::planned_run, |c| c.run)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The resolve before the index (decisions §31 step 1), kept to prove the index answers
    /// the same: a linear scan of [`all_names`].
    fn linear_resolve(input: &str) -> Result<Word, String> {
        if let Some((_, w)) = all_names().find(|(n, _)| n.eq_ignore_ascii_case(input)) {
            return Ok(w);
        }
        let lower = input.to_lowercase();
        let mut matches: Vec<(Word, Vec<&'static str>)> = Vec::new();
        for (name, word) in all_names().filter(|(n, _)| n.to_lowercase().starts_with(&lower)) {
            match matches.iter_mut().find(|(w, _)| w.same(&word)) {
                Some((_, names)) => names.push(name),
                None => matches.push((word, vec![name])),
            }
        }
        match matches.len() {
            0 => Err("none".into()),
            1 => Ok(matches[0].0),
            _ => matches
                .iter()
                .find(|(_, names)| {
                    matches.iter().all(|(_, others)| {
                        others
                            .iter()
                            .any(|o| names.iter().any(|n| o.to_lowercase().starts_with(&n.to_lowercase())))
                    })
                })
                .map(|(w, _)| *w)
                .ok_or_else(|| "ambiguous".into()),
        }
    }

    #[test]
    fn the_name_index_resolves_every_prefix_like_the_linear_scan() {
        let mut checked = 0;
        for (name, _) in all_names() {
            for end in 1..=name.len() {
                for typed in [&name[..end], &name[..end].to_uppercase()] {
                    let new = resolve(typed);
                    let old = linear_resolve(typed);
                    match (&new, &old) {
                        (Ok(a), Ok(b)) => assert!(a.same(b), "'{typed}': {} vs {}", a.label(), b.label()),
                        (Err(_), Err(_)) => {}
                        _ => panic!("'{typed}': index {:?} vs linear {:?}", new.map(|w| w.label()), old.map(|w| w.label())),
                    }
                    checked += 1;
                }
            }
            assert!(lookup_word(name).is_some_and(|w| w.same(&lookup_word(&name.to_uppercase()).unwrap())));
        }
        assert!(checked > 1000, "{checked}");
        assert!(lookup_word("nosuchword").is_none());
    }

    #[test]
    fn aliases_resolve_to_their_command() {
        assert!(matches!(resolve("a"), Ok(Word::Command(c)) if c.verb == "at"));
        assert!(matches!(resolve("hiss"), Ok(Word::Command(c)) if c.verb == "oops"));
    }

    #[test]
    fn objects_resolve_as_words() {
        assert!(matches!(resolve("fixture").ok(), Some(Word::Object(o)) if o.keyword == "Fixture"));
        assert!(matches!(resolve("Fix").ok(), Some(Word::Object(o)) if o.keyword == "Fixture"));
        assert!(
            matches!(resolve("type").ok(), Some(Word::Object(o)) if o.keyword == "FixtureType")
        );
    }

    #[test]
    fn aliases_of_one_command_are_not_ambiguous() {
        // logout and logoff are the same command.
        assert!(matches!(resolve("logo").ok(), Some(Word::Command(c)) if c.verb == "logout"));
    }

    #[test]
    fn lookup_any_finds_by_alias() {
        let cmd = lookup_any("hiss").expect("should find oops by alias");
        assert_eq!(cmd.verb, "oops");
    }

    #[test]
    fn visible_commands_excludes_hidden() {
        let visible: Vec<_> = visible_commands().collect();
        assert!(visible.iter().all(|c| !c.doc.hidden));
        assert!(visible.iter().any(|c| c.verb == "at"));
        assert!(!visible.iter().any(|c| c.verb == "root"));
    }

    #[test]
    fn mutating_flags() {
        assert!(lookup("at").unwrap().undo.is_mutating());
        assert!(lookup("store").unwrap().undo.is_mutating());
        assert!(!lookup("list").unwrap().undo.is_mutating());
        assert!(!lookup("help").unwrap().undo.is_mutating());
    }
}
