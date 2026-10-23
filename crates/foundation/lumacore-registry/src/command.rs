//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝
// FILE: command
//! The words a command or object is declared with (decisions §15, §31): its help (`Doc`, the help
//! categories, alias depth), undo, scope, argument shape, rights level, class, reach and family.
//! Pure data: commands' and objects' rows name them (decisions §34 21).
//!
//! Owns:          Doc, HelpCategory, AliasDepth, Undo (+ tracked, main and the common ones),
//!                Scope, Args, Rights, CommandClass, Reach, CommandFamily, CommandDecl
//! Doesn't own:   the rows (each crate's register), the help's colours (lumacore-state)
//! Entry points:  Doc, Undo, Scope, Args, Rights, CommandClass, Reach, CommandFamily
//! Decisions:     §15, §31, §34 21

use lumacore_base::{OopsCategory, OopsKind};

use crate::flag::FlagDef;
use crate::status::Status;
use crate::verb::Verb;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

// *************************************************************************************************
// Help categories
// Where a word shows in help.
// *************************************************************************************************

/// Help category — object-polymorphic model.
///
/// Commands are grouped by which object they act on, not by action family.
/// Global commands (navigation, system) get their own sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpCategory {
    // Object types — the things you work with
    Fixture,
    Group,
    Universe,
    Programmer,
    Playback,
    DataPool,
    // Global commands — not bound to a specific object
    Inspection,
    TreeNav,
    ProgrammerNav,
    Undo,
    System,
}

impl HelpCategory {
    pub fn label(self) -> &'static str {
        match self {
            HelpCategory::Fixture => "Fixture",
            HelpCategory::Group => "Group",
            HelpCategory::Universe => "Universe",
            HelpCategory::Programmer => "Programmer",
            HelpCategory::Playback => "Playback",
            HelpCategory::DataPool => "DataPool",
            HelpCategory::Inspection => "Inspection",
            HelpCategory::TreeNav => "Unix-tree Navigation",
            HelpCategory::ProgrammerNav => "Programmer Navigation",
            HelpCategory::Undo => "Undo",
            HelpCategory::System => "System",
        }
    }

    /// Whether this category represents an object type (vs a global command group).
    pub fn is_object(self) -> bool {
        matches!(
            self,
            HelpCategory::Fixture
                | HelpCategory::Group
                | HelpCategory::Universe
                | HelpCategory::Programmer
                | HelpCategory::Playback
                | HelpCategory::DataPool
                | HelpCategory::ProgrammerNav
        )
    }

    /// Minimum AliasDepth required to see commands in this category.
    pub fn min_depth(self) -> AliasDepth {
        match self {
            HelpCategory::Undo => AliasDepth::ProgLang,
            HelpCategory::System => AliasDepth::Full,
            _ => AliasDepth::Base,
        }
    }
}

// *************************************************************************************************
// A command
// Its declaration and the words of each field.
// *************************************************************************************************

/// How deep into the alias layers a user must be to see an entry in help.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AliasDepth {
    Base,
    ProgLang,
    Full,
}

impl AliasDepth {
    /// From a user profile's setting ("Base", "ProgLang", "Full"); unknown → Base.
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "proglang" => Self::ProgLang,
            "full" => Self::Full,
            _ => Self::Base,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Base => "Base",
            Self::ProgLang => "ProgLang",
            Self::Full => "Full",
        }
    }
}

/// Help documentation, shared by commands and objects.
pub struct Doc {
    /// Display name, e.g. "ClearAll", "ViewButton".
    pub display: &'static str,
    /// Primary object or category this entry belongs to in help.
    pub category: HelpCategory,
    /// Additional categories (polymorphic commands).
    pub also_applies_to: &'static [HelpCategory],
    /// One-liner syntax summary.
    pub syntax: &'static str,
    /// Full description shown by `Help <word>`.
    pub description: &'static str,
    /// Usage examples.
    pub examples: &'static [&'static str],
    /// Related words.
    pub related: &'static [&'static str],
    /// Alias depth required to see it in the help overview.
    pub alias_depth: AliasDepth,
    /// Hidden entries resolve but are left out of help.
    pub hidden: bool,
}

impl Doc {
    /// The defaults an entry's doc starts from (`..Doc::BASE`): no extra categories, examples
    /// or related words, base alias depth, shown in help.
    pub const BASE: Doc = Doc::new("", "");

    /// Minimal documentation: a display name and a description.
    pub const fn new(display: &'static str, description: &'static str) -> Self {
        Self {
            display,
            category: HelpCategory::System,
            also_applies_to: &[],
            syntax: "",
            description,
            examples: &[],
            related: &[],
            alias_depth: AliasDepth::Base,
            hidden: false,
        }
    }
}

/// What a command does to undo and replication. Which show subtrees it
/// changed is found by the change path (`terminal::change`), never listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Undo {
    /// Reads only: no change tracking.
    ReadOnly,
    /// Changes state: show changes replicate and are journaled, but get no
    /// undo frame (login, sessions).
    Untracked,
    /// Changes the show: an Oops step with this category label (refined from
    /// what changed when the category is generic, `dispatch::step_category`)
    /// and Main flag, replicated and journaled.
    Tracked(OopsKind),
}

impl Undo {
    pub fn is_mutating(self) -> bool {
        self != Self::ReadOnly
    }

    pub fn kind(self) -> Option<OopsKind> {
        match self {
            Self::Tracked(kind) => Some(kind),
            _ => None,
        }
    }
}

/// An Oops step of `category`.
pub const fn tracked(category: OopsCategory) -> Undo {
    Undo::Tracked(OopsKind { category, main: false })
}

/// An Oops step of `category` that is a **Main** action (Oops
/// Confirmation "Main": create, delete, move, insert, file actions, layout).
pub const fn main(category: OopsCategory) -> Undo {
    Undo::Tracked(OopsKind { category, main: true })
}

pub const PROGRAMMER: Undo = tracked(OopsCategory::Programmer);
pub const SELECTION: Undo = tracked(OopsCategory::Selection);
/// Generic show changes: the category comes from what changed.
pub const PROGRAMMER_SHOWDATA: Undo = tracked(OopsCategory::Show);
pub const PATCH: Undo = tracked(OopsCategory::Patch);
/// Patch, Unpatch, Import, Create: create / remove / file actions.
pub const PATCH_MAIN: Undo = main(OopsCategory::Patch);
pub const STORE: Undo = tracked(OopsCategory::Show);
/// Store, Copy, Move, Delete, rm, Remove (storing, moving or
/// deleting objects).
pub const STORE_MAIN: Undo = main(OopsCategory::Show);
pub const EDIT: Undo = tracked(OopsCategory::Show);
pub const KEYS: Undo = tracked(OopsCategory::Keys);
/// `Set`: the category comes from what changed (Patch, Users, Station, …).
pub const USER_SETTINGS: Undo = tracked(OopsCategory::Settings);
pub const UI: Undo = tracked(OopsCategory::UI);

/// Where a command runs in a network session (decisions.md §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// On the client's own station, never forwarded (navigation, help, views).
    Client,
    /// On the station it was typed on (station-local settings, power).
    Station,
    /// On the session master, as the sending user (anything that changes the show).
    Master,
}

/// What a command's arguments are, for tab completion and hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Args {
    /// Object keywords and object names (the default).
    Objects,
    /// An object, a tree node or a dot-path (`List`).
    ObjectsOrTree,
    /// Tree paths from the current position; `root`, and `..` when `up`.
    TreeNav { up: bool },
    /// Tree paths from the root, then shorthand node names (`ls`).
    TreeFromRoot,
    /// Tree paths and the object ids at the current position, from argument
    /// `from` on; `options` are the command's own flags (`--type`).
    TreeItems {
        from: usize,
        options: &'static [&'static str],
    },
    /// Command and object names (`Help`).
    Words,
    /// A value or an attribute of the selection (`At`).
    Value,
    /// An attribute of the selection, then `At` (`Attribute Pan At 50`).
    /// The running watches, or `all` (`Unwatch`).
    Watches,
}

/// The level a command, an object verb or a property needs (decisions §30
/// "12d hardening": a ladder, one level per user). Ordered: a user's
/// level ([`lumacore_tree::UserRights`]) passes everything at or below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rights {
    Guest,
    Playback,
    Presets,
    Program,
    Setup,
    Admin,
    /// Never from a user: reserved for the server's own root monitor.
    Internal,
}

impl Rights {
    pub fn label(self) -> &'static str {
        match self {
            Self::Internal => "Internal",
            Self::Guest => "Guest",
            Self::Playback => "Playback",
            Self::Presets => "Presets",
            Self::Program => "Program",
            Self::Setup => "Setup",
            Self::Admin => "Admin",
        }
    }
}

/// How the runtime treats a word: a read, a terminal-session change, a show
/// commit, a realtime action, or one that touches the runtime itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandClass {
    Query,
    Session,
    Commit,
    Realtime,
    DangerousRuntime,
}

/// What a word reaches besides the show and the station's runtime. Every
/// command and object verb declares it; the nightly command sweep runs only
/// `Show` (decisions §25 "Step 10 hardening").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reach {
    /// The show, users' state, the station's runtime — nothing outside.
    Show,
    /// The unix-like tree shell (`ls`, `cd`, `cat`, …): it can show the
    /// library directory on disk.
    TreeShell,
    /// Reads or writes files (shows, exports, the GDTF library).
    Drive,
}

/// A word's language family (the sentence kind it starts), declared on every
/// `CommandDef` / `ObjectDef`. (The sentence-preview code that read it is
/// archived in `docs/archive/code/language.rs`, decisions §29 "11b hardening".)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandFamily {
    Selection,
    Action,
    Assignment,
    StoreUpdate,
    Settings,
    Inspection,
    Navigation,
    Session,
    System,
    Runtime,
    Playback,
    Edit,
}

/// A command as the catalog holds it (decisions §34 21): everything but its handler, which
/// dispatch binds to this row (`lumacore-state`'s `CommandDef`).
pub struct CommandDecl {
    /// Primary verb (lowercase), the canonical name.
    pub verb: &'static str,
    /// Other spellings that resolve to this command (short forms, synonyms).
    pub aliases: &'static [&'static str],
    pub doc: Doc,
    pub undo: Undo,
    /// Language family for the sentence preview, if it has one.
    pub family: Option<CommandFamily>,
    pub scope: Scope,
    pub status: Status,
    /// The object verb this command is (`Store`, `Delete`, …): `Store Window 5`
    /// goes to the Window object's Store handler.
    pub object_verb: Option<Verb>,
    /// What its arguments are, for completion.
    pub args: Args,
    /// The level needed to run it. A line naming an object takes the
    /// object's level for this verb instead ([`ObjectDef::rights`]:
    /// `Store View` is Playback, `Store User` Admin).
    ///
    /// [`ObjectDef::rights`]: crate::objects::ObjectDef::rights
    pub rights: Rights,
    /// The level its arguments raise it to (`Claim Root.ShowData.Devices` → Setup, `Claim …
    /// /Force` → Admin); None = the arguments never matter.
    pub arg_rights: Option<fn(&[&str]) -> Rights>,
    /// How the runtime schedules it (query, session, commit, realtime, …).
    pub class: CommandClass,
    /// What it touches beyond the show (decisions §25 "Step 10 hardening"):
    /// the nightly sweep runs only `Show` words.
    pub reach: Reach,
    /// Its own flags (`Claim … /Force`); the target object's for this verb and the global
    /// ones come on top (decisions §31, [`super::flags`]).
    pub flags: &'static [FlagDef],
    /// The object a bare number after it names (a default keyword: `Store 2` = `Store Cue 2`;
    /// decisions §30 "Store Cue") — the handler runs it, the suggestions complete it.
    pub default_object: Option<&'static str>,
    /// A hold verb: `On` / `Off` right after it says press or release (Top, Flash — decisions
    /// §33 15).
    pub hold: bool,
    /// A tree-shell word: its statement is read with the Unix fence's rules, its flags as
    /// `-x` / `--name` (decisions §33 18).
    pub fence: bool,
}

/// [`CommandDef::BASE`]'s handler: never runs, every entry names its own (tidy
/// `every_command_states_what_decides_it`).
impl CommandDecl {
    /// What an entry leaves out (`..CommandDecl::BASE`, decisions §31): no aliases, no
    /// sentence family, built, not an object verb, object arguments, rights from the line
    /// alone, a plain commit. An entry always states `verb`, `doc` (its row's `run` beside it) and the four that
    /// decide what a line may do and where — `undo`, `scope`, `rights`, `reach` (tidy
    /// `every_command_states_what_decides_it`); their values here are the strictest, never
    /// meant to be used.
    pub const BASE: CommandDecl = CommandDecl {
        verb: "",
        aliases: &[],
        doc: Doc::BASE,
        undo: Undo::Untracked,
        family: None,
        scope: Scope::Master,
        status: Status::Ready,
        object_verb: None,
        args: Args::Objects,
        rights: Rights::Admin,
        arg_rights: None,
        class: CommandClass::Commit,
        reach: Reach::Drive,
        flags: &[],
        default_object: None,
        hold: false,
        fence: false,
    };

    /// Display label — the doc's display name, or the verb.
    pub fn label(&self) -> &str {
        if self.doc.display.is_empty() {
            self.verb
        } else {
            self.doc.display
        }
    }

    /// All names this command answers to.
    pub fn names(&self) -> impl Iterator<Item = &'static str> {
        std::iter::once(self.verb).chain(self.aliases.iter().copied())
    }
}
