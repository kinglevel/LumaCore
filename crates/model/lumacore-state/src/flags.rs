// ███████╗██╗      █████╗  ██████╗ ███████╗
// ██╔════╝██║     ██╔══██╗██╔════╝ ██╔════╝
// █████╗  ██║     ███████║██║  ███╗███████╗
// ██╔══╝  ██║     ██╔══██║██║   ██║╚════██║
// ██║     ███████╗██║  ██║╚██████╔╝███████║
// ╚═╝     ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝
// FILE: flags

//! Declared flags (decisions §31 "Flags are declared and a line is parsed once").
//!
//! A flag is a bare `/word` after a command (`Store Group 3 /Overwrite`); a quoted `"/word"` is
//! a value (§15 Quoting). Every flag a line may carry is declared here or on its word: the
//! [`GLOBAL_FLAGS`] on every line, a command's own on `CommandDef::flags`, an object's for a verb
//! on `ObjectDef::flags`. Completion, help, the reference and the sweep read the declarations;
//! a flag nobody declares is refused (E-CMD-018) — never ignored silently.
//!
//! Owns:          the global and shared flags (registered as rows), reading a handler's flags from its args.
//! Doesn't own:   every declared flag at once (commands::flags::all, reads the registry tables).
//! Entry points:  FlagDef, GLOBAL_FLAGS, has, value, rest_value, positional.
//! Decisions:     §31 "Flags are declared", §34 15.

use crate::defs::object::Verb;
/// The flag rows are the catalog's (decisions §34 21).
pub use lumacore_registry::{FlagDef, FlagValue};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

// *************************************************************************************************
// Global: valid on every line (/nc goes anywhere)
// *************************************************************************************************
pub const NO_CONFIRMATION: FlagDef = FlagDef::new(
    "NoConfirmation",
    "Answer the line's question with its default (a one-shot line can't ask)",
)
.alias(&["nc"]);
pub const NO_OOPS: FlagDef = FlagDef::new("NoOops", "File no Oops step for this line").alias(&["NU"]);
pub const VERBOSE: FlagDef = FlagDef::new("Verbose", "Show how the line was understood").alias(&["v"]);
pub const DEBUG: FlagDef = FlagDef::new("Debug", "Show the dispatch trace").alias(&["d"]);
pub const WATCH: FlagDef =
    FlagDef::new("Watch", "Run the line again and again (`/Watch=500` = every 500 ms)").inline("ms");

/// Valid on every line; a word's own flag of the same name wins (`Tree /d` = depth). Read them
/// from the catalog (`lumacore_registry::get().global_flags()`); this crate registers them.
pub(crate) const GLOBAL_FLAGS: &[FlagDef] = &[NO_CONFIRMATION, NO_OOPS, VERBOSE, DEBUG, WATCH];

// *************************************************************************************************
// Shared by several words
// *************************************************************************************************
pub const OVERWRITE: FlagDef = FlagDef::new("Overwrite", "Replace what is there").alias(&["o"]);
pub const MERGE: FlagDef = FlagDef::new("Merge", "Add to what is there");
pub const REMOVE: FlagDef = FlagDef::new("Remove", "Take the selection out of what is there");
pub const CREATE_SECOND_CUE: FlagDef = FlagDef::new("CreateSecondCue", "Store as a new cue right after it");
/// The store ask's answers (decisions §30 "Store Cue"): on Store, Store Cue and Store
/// Sequence.
pub const STORE_CUE: &[FlagDef] =
    &[MERGE.on(&[Verb::Store]), OVERWRITE.on(&[Verb::Store]), REMOVE.on(&[Verb::Store]), CREATE_SECOND_CUE.on(&[Verb::Store])];
pub const FORCE: FlagDef = FlagDef::new("Force", "Do it even though someone else holds it (Admin)");
pub const KEEP: FlagDef = FlagDef::new("Keep", "Keep the changes");
pub const REMEMBER: FlagDef = FlagDef::new("Remember", "Remember this answer");
pub const CLEAR: FlagDef = FlagDef::new("Clear", "Clear it");
pub const SCREEN: FlagDef = FlagDef::new("Screen", "The screen it is on").alias(&["s"]).value("screen");
pub const TYPE: FlagDef = FlagDef::new("Type", "The window type").alias(&["t"]).value("type");
pub const ALL: FlagDef = FlagDef::new("All", "Everything").alias(&["a"]);
/// What `Sort` / `Split` go by (decisions §33 88): each object declares its keys.
pub const BY: FlagDef =
    FlagDef::new("By", "What to sort, split, compact or show by (Number, Name, Family, MainFirst, Address, FixtureType)").value("key").choices(&["Number", "Name", "Family", "MainFirst", "Address", "FixtureType"]);
/// The answers of a tree node's Delete (Tag, MvrLayer, MvrGroup; decisions §12, §34 round 3 19).
pub const KEEP_CHILDREN: FlagDef = FlagDef::new("KeepChildren", "Keep what is under it: it moves up one level");
pub const WITH_CHILDREN: FlagDef = FlagDef::new("WithChildren", "Delete what is under it too");
/// `Move … At n /Skip` (decisions §34 round 3 19): jump over taken numbers instead of swapping.
pub const SKIP: FlagDef = FlagDef::new("Skip", "Jump over taken numbers instead of swapping with them");
pub const RECURSIVE: FlagDef = FlagDef::new("R", "Recursive: what is below too");
pub const VERSION: FlagDef =
    FlagDef::new("Version", "A saved version of the show (its stamp, List Version)").value("stamp");
pub const RECOVERY: FlagDef =
    FlagDef::new("Recovery", "A recovery copy of the show (List Recovery)").value("copy");
pub const SAVE: FlagDef = FlagDef::new("Save", "Save the show first");
/// Every file word's drive (decisions §38): absent = the local library.
pub const DRIVE: FlagDef = FlagDef::new("Drive", "The drive to read or write (List Drive); absent = the local library").value("id");
/// `Store Drive /Path "/srv/shows"`: the folder an added drive points at.
pub const PATH: FlagDef = FlagDef::new("Path", "The folder an added drive points at").value("folder");
pub const NO_SAVE: FlagDef = FlagDef::new("NoSave", "Don't save the show first");

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// One line naming the global flags, for help and the reference.
pub fn globals_line() -> String {
    lumacore_registry::get().global_flags().map(FlagDef::usage).collect::<Vec<_>>().join(", ")
}

// *************************************************************************************************
// Reading a handler's own args (decisions §31)
// *************************************************************************************************
//
// A handler gets its args as typed (quoted words keep their quotes). These read a declared
// flag in them — its name or a short form, any case, `=value` inline — so no handler compares
// `/word` strings itself. A quoted `"/x"` is a value and never counts; so is a bare `nc`.

/// `arg` as a use of `def`: `Some(inline value)` when it names it.
fn use_of<'a>(arg: &'a str, def: &FlagDef) -> Option<Option<&'a str>> {
    let body = arg.strip_prefix('/')?;
    let (name, inline) = match body.split_once('=') {
        Some((n, v)) => (n, Some(v)),
        None => (body, None),
    };
    def.named(name).then_some(inline)
}

/// A bare `/word` (a flag of any word), not a quoted value or a `/2`.
pub fn is_flag(arg: &str) -> bool {
    arg.strip_prefix('/').is_some_and(|b| b.starts_with(|c: char| c.is_ascii_alphabetic()))
}

/// Whether `arg` is `def` (its name or a short form, with or without `=value`).
pub fn is(arg: &str, def: &FlagDef) -> bool {
    use_of(arg, def).is_some()
}

/// Whether `args` carry `def`.
pub fn has(args: &[&str], def: &FlagDef) -> bool {
    args.iter().any(|a| use_of(a, def).is_some())
}

/// `def`'s value in `args`: `=value` inline, else the next arg when it isn't a flag (unquoted).
pub fn value(args: &[&str], def: &FlagDef) -> Option<String> {
    let i = args.iter().position(|a| use_of(a, def).is_some())?;
    match use_of(args[i], def).flatten() {
        Some(inline) => Some(inline.trim_matches('"').to_string()),
        // A switch or an inline-only flag (`/d=2`) never takes the next word.
        None if matches!(def.value, FlagValue::None | FlagValue::Inline(_)) => None,
        None => args.get(i + 1).filter(|n| !is_flag(n)).map(|n| n.trim_matches('"').to_string()),
    }
}

/// `def`'s value as every word up to the next flag (`/Tab Lines In`), or `=value` (unquoted).
pub fn rest_value(args: &[&str], def: &FlagDef) -> Option<String> {
    let i = args.iter().position(|a| use_of(a, def).is_some())?;
    if let Some(inline) = use_of(args[i], def).flatten() {
        return Some(inline.trim_matches('"').to_string());
    }
    let words: Vec<&str> =
        args[i + 1..].iter().take_while(|a| !is_flag(a)).map(|a| a.trim_matches('"')).collect();
    (!words.is_empty()).then(|| words.join(" "))
}

/// `args` without `def` (and its value, when it takes one): the other flags stay.
pub fn without<'a>(args: &[&'a str], def: &FlagDef) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i];
        i += 1;
        match use_of(a, def) {
            None => out.push(a),
            Some(None) if matches!(def.value, FlagValue::One(_)) && args.get(i).is_some_and(|n| !is_flag(n)) => i += 1,
            Some(_) => {}
        }
    }
    out
}

/// `args` without the flags and the values the given flags take (`Store Input 2 "x" /Port
/// 8000` → `2`, `"x"`).
pub fn positional<'a>(args: &[&'a str], with_values: &[&FlagDef]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i];
        i += 1;
        if !is_flag(a) {
            out.push(a);
            continue;
        }
        let takes_next = with_values.iter().any(|d| {
            use_of(a, d).is_some_and(|inline| inline.is_none() && matches!(d.value, FlagValue::One(_)))
        });
        if takes_next && args.get(i).is_some_and(|n| !is_flag(n)) {
            i += 1;
        }
    }
    out
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
