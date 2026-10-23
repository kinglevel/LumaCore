// ███████╗████████╗ █████╗ ████████╗███████╗
// ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██╔════╝
// ███████╗   ██║   ███████║   ██║   █████╗
// ╚════██║   ██║   ██╔══██║   ██║   ██╔══╝
// ███████║   ██║   ██║  ██║   ██║   ███████╗
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚══════╝
// FILE: state

//! The station's state (decisions §34 7, 15): AppState, the change path, Oops, the property
//! registry's types, errors, dialogs, the disk door, show files, backups, media, the outbox to
//! the LumaSession and the registries' types. It names nothing above it — no table, no object,
//! no command — so everything from the objects up builds on it.
//!
//! Owns:          AppState and every module listed below; Reply, DispatchResult, build_info.
//! Doesn't own:   the registry tables and the objects (`lumacore-objects`), dispatch and the station (`lumacore`).
//! Entry points:  AppState, Reply, DispatchResult, the modules.
//! Decisions:     §23, §34 7, §34 15.

pub mod app_state;
pub mod change;
pub mod consent;
pub mod defs;
pub mod device_table;
pub mod dialogs;
pub mod drives;
pub mod errors;
pub mod flags;
pub mod gesture;
pub mod identifying;
pub mod look;
pub mod oops;
pub mod parser;
pub mod passwords;
pub mod patch_draft;
pub mod mvr_review;
pub mod pools;
pub mod programmers;
pub mod props;
pub mod recovery;
pub mod session_link;
pub mod show_feedback;
pub mod session_log;
pub mod stations;
pub mod text;
pub mod tokens;
pub mod tree_path;

pub use app_state::{AppState, UndoKind};
/// Reading a show file is the show file format's (decisions §34 25.10).
pub use lumacore_luma::show_file::parse_show;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The build's identity, stamped into saved shows (the show file format's, decisions §20, §34 25.10).
pub use lumacore_luma::{build_info, set_git_commit};

/// A command's answer (decisions §31 "typed replies"): the text every client shows, and
/// whether it failed with which code. Dispatch, the journal and the station's readers use
/// `failed` / `code`, never the text; the text goes to clients unchanged (like a console's
/// command line feedback, §23). Reads as `&str` (`Deref`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Reply {
    pub text: String,
    /// An error (an `Error:` reply).
    pub failed: bool,
    /// Its code (`E-CMD-007`), when it has one.
    pub code: Option<String>,
}

impl Reply {
    /// A reply that worked.
    pub fn ok(text: impl Into<String>) -> Self {
        Self { text: text.into(), failed: false, code: None }
    }

    /// Several replies as one (lines joined): failed when any is, with the first code.
    pub fn joined(replies: impl IntoIterator<Item = Reply>) -> Self {
        let mut out = Self::default();
        let mut texts = Vec::new();
        for r in replies {
            if r.failed && !out.failed {
                out.failed = true;
                out.code = r.code;
            }
            texts.push(r.text);
        }
        out.text = texts.join("\n");
        out
    }

    /// A coded error: `message [code]`, rendered as every error is (`errors::render_code`).
    pub fn error(message: impl std::fmt::Display, code: &str) -> Self {
        Self { text: crate::errors::render_code(message, code), failed: true, code: Some(code.to_string()) }
    }
}

/// Text from a handler that still answers with a `String`: whether it is an error is read
/// from the text here, once (`errors::outcome_of`) — the one place a reply's text is read back.
impl From<String> for Reply {
    fn from(text: String) -> Self {
        let (failed, code) = crate::errors::outcome_of(&text);
        Self { text, failed, code }
    }
}

impl From<&str> for Reply {
    fn from(text: &str) -> Self {
        Self::from(text.to_string())
    }
}

impl From<Reply> for String {
    fn from(reply: Reply) -> Self {
        reply.text
    }
}

impl std::ops::Deref for Reply {
    type Target = str;
    fn deref(&self) -> &str {
        &self.text
    }
}

impl PartialEq<&str> for Reply {
    fn eq(&self, other: &&str) -> bool {
        self.text == *other
    }
}

impl std::fmt::Display for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

/// Result of dispatching a single command.
pub enum DispatchResult {
    /// Normal output to print.
    Output(Reply),
    /// Exit the REPL.
    Exit,
    /// Disconnect the current shell session and return to the outer client.
    Logout,
    /// Switch sticky context.
    ContextSwitch(crate::ReplContext),
    /// Ask the user (decisions §30 "12b"): a shell opens the dialog
    /// (`dialogs::open`), a one-shot path refuses it (`Ask::refusal`).
    Ask(Box<crate::dialogs::Ask>),
}

pub const INTERNAL_ROOT_USER: &str = "__lumacore_root__";

/// Metadata stored per imported FixtureType (source file path, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureTypeSource {
    /// Original file name (basename) of the imported .gdtf file.
    pub file_name: String,
}

/// Context string for the sticky prompt context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplContext {
    Fixture,
    Patch,
    Universe,
    Group,
    Sequence,
    Executor,
    Macro,
    Preset,
    Timecode,
    Effect,
    EffectEngine,
    DataPool,
    UserProfile,
    Custom(String),
}

impl std::fmt::Display for ReplContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplContext::Fixture => write!(f, "Fixture"),
            ReplContext::Patch => write!(f, "Patch"),
            ReplContext::Universe => write!(f, "Universe"),
            ReplContext::Group => write!(f, "Group"),
            ReplContext::Sequence => write!(f, "Sequence"),
            ReplContext::Executor => write!(f, "Executor"),
            ReplContext::Macro => write!(f, "Macro"),
            ReplContext::Preset => write!(f, "Preset"),
            ReplContext::Timecode => write!(f, "Timecode"),
            ReplContext::Effect => write!(f, "Effect"),
            ReplContext::EffectEngine => write!(f, "EffectEngine"),
            ReplContext::DataPool => write!(f, "DataPool"),
            ReplContext::UserProfile => write!(f, "UserProfile"),
            ReplContext::Custom(s) => write!(f, "{s}"),
        }
    }
}

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This crate's rows in the catalog, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_syntax::register(r);
    lumacore_tree::register(r);
    lumacore_patch::register(r);
    lumacore_gdtf::register(r);
    lumacore_luma::register(r);
    lumacore_render::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.errors(errors::ERRORS);
    r.global_flags(flags::GLOBAL_FLAGS);
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// This crate's tests read the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
