//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝
// FILE: command
//! What a command declares (decisions §15, §31): `CommandDef` — its words are the catalog's
//! (`lumacore_registry::command`, decisions §34 21) and re-exported here — the handler type, the
//! planned handler and the help's colours.
//!
//! Owns:          CommandDef, Run, planned_run, HelpColour
//! Doesn't own:   the words (the catalog), the commands' rows (the families in `commands::registry::rows`)
//! Entry points:  CommandDef, Run, planned_run, HelpColour
//! Decisions:     §15, §31, §34 15, §34 21

use nu_ansi_term::Color;

pub use lumacore_registry::command::{
    AliasDepth, Args, CommandClass, CommandDecl, CommandFamily, Doc, HelpCategory, PATCH, PATCH_MAIN, PROGRAMMER, PROGRAMMER_SHOWDATA, Reach, Rights,
    SELECTION, STORE, STORE_MAIN, EDIT, KEYS, USER_SETTINGS, UI, Scope, Undo, main, tracked,
};
pub use lumacore_registry::Status;

use crate::{AppState, DispatchResult};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A command or object handler: the arguments after the keyword.
pub type Run = fn(&mut AppState, &[&str]) -> DispatchResult;

/// The help's colour of a category (the terminal's; decisions §34 21 keeps colours out of the
/// catalog).
pub trait HelpColour {
    fn color(self) -> Color;
}

impl HelpColour for HelpCategory {
    fn color(self) -> Color {
        match self {
            HelpCategory::Fixture => Color::Fixed(214), // orange
            HelpCategory::Group => Color::Blue,
            HelpCategory::Universe => Color::Fixed(39), // light blue
            HelpCategory::Programmer => Color::Magenta,
            HelpCategory::Playback => Color::Fixed(220), // gold
            HelpCategory::DataPool => Color::Fixed(177), // light purple
            HelpCategory::Inspection => Color::Cyan,
            HelpCategory::TreeNav => Color::Green,
            HelpCategory::ProgrammerNav => Color::Fixed(48), // bright green
            HelpCategory::Undo => Color::Purple,
            HelpCategory::System => Color::Fixed(244), // grey
        }
    }
}

/// A command's row and its handler, side by side (decisions §34 21): the catalog holds `decl`,
/// dispatch runs `run` for it. Reads through to the row (`def.verb`).
pub struct CommandDef {
    /// What runs the line: the arguments after the keyword.
    pub run: Run,
    /// The row the catalog holds.
    pub decl: CommandDecl,
}

impl std::ops::Deref for CommandDef {
    type Target = CommandDecl;

    fn deref(&self) -> &CommandDecl {
        &self.decl
    }
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The handler of a `Status::Planned` command. Dispatch answers a planned
/// command itself (E-CMD-003) and never calls it; this keeps the same reply
/// if something does.
pub fn planned_run(_state: &mut AppState, _args: &[&str]) -> DispatchResult {
    DispatchResult::Output(crate::Reply::error(
        "This word isn't built yet (planned).",
        "E-CMD-003",
    ))
}
