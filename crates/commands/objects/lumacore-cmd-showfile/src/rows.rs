// ██████╗  ██████╗ ██╗    ██╗███████╗
// ██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██████╔╝██║   ██║██║ █╗ ██║███████╗
// ██╔══██╗██║   ██║██║███╗██║╚════██║
// ██║  ██║╚██████╔╝╚███╔███╔╝███████║
// ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: rows

//! The command rows of show files: SaveShow, LoadShow, NewShow, DeleteShow, CleanStart (decisions
//! §34 25.7, 25.13: each crate of 34g4 takes its file).
//!
//! Owns:          the showfile family's command rows (`ROWS`): each word's declaration and handler.
//! Doesn't own:   the handlers' work (their command modules), the catalog's checks (registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13.

use lumacore_objects::commands::registry::*;
use lumacore_objects::objects::ObjectDef;

use lumacore_objects::rights::saveshow_rights;

use crate::ask::SHOW_FILE_FLAGS;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The show file's commands with their handlers (Restart and Shutdown too: they ask "save
/// first?" — they go to `cmd-station` when it is cut, on this crate); `register` adds them.
pub static ROWS: &[CommandDef] = &[
    CommandDef {
        run: crate::words::cmd_clean_start,
        decl: CommandDecl {
            verb: "cleanstart",
            doc: Doc {
                display: "CleanStart",
                category: HelpCategory::System,
                syntax: "CleanStart [/Save|/NoSave] [/NoConfirmation]",
                description: "Start clean: an empty show (Admin, default screens), \
                          startup show None and default station settings — the way back when a show \
                          or the UI is unusable. Show files on disk stay. Asks first (Save / Don't \
                          Save with unsaved changes). Admin only; the app's home screen offers it for \
                          the local server.",
                examples: &[
                    "cleanstart          -- asks first",
                    "cleanstart /nosave  -- discard unsaved changes",
                ],
                related: &["newshow", "loadshow", "restart"],
                ..Doc::BASE
            },
            undo: PROGRAMMER_SHOWDATA,
            scope: Scope::Master,
            rights: Rights::Admin,
            reach: Reach::Drive,
            flags: SHOW_FILE_FLAGS,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::words::cmd_save_show,
        decl: CommandDecl {
            verb: "saveshow",
            doc: Doc {
                display: "SaveShow",
                category: HelpCategory::System,
                syntax: "SaveShow [\"name\"]",
                description: "Save the show to library/shows/<name>.luma. Without a name it saves the \
                          current show (a quick save); an untitled show gets a NewShow_<date> name.",
                examples: &[
                    "SaveShow                 -- quick save the current show",
                    "SaveShow \"Festival 26\"  -- save as",
                ],
                related: &["loadshow", "newshow", "deleteshow", "show"],
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            scope: Scope::Master,
            rights: Rights::Presets,
            arg_rights: Some(saveshow_rights),
            reach: Reach::Drive,
            flags: SHOW_FILE_FLAGS,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::words::cmd_load_show,
        decl: CommandDecl {
            verb: "loadshow",
            doc: Doc {
                display: "LoadShow",
                category: HelpCategory::System,
                syntax: "LoadShow \"name\" [/Save | /NoSave] [/NoConfirmation]",
                description: "Load a show from library/shows. Asks first; with unsaved changes it \
                          offers to save them (/Save or /NoSave answer that up front).",
                examples: &[
                    "LoadShow \"Festival 26\"",
                    "LoadShow \"Festival 26\" /NoSave /NoConfirmation",
                ],
                related: &["saveshow", "newshow", "show"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Master,
            rights: Rights::Setup,
            reach: Reach::Drive,
            flags: SHOW_FILE_FLAGS,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::words::cmd_new_show,
        decl: CommandDecl {
            verb: "newshow",
            doc: Doc {
                display: "NewShow",
                category: HelpCategory::System,
                syntax: "NewShow [\"name\"] [/Save | /NoSave] [/NoConfirmation]",
                description: "Start an empty show and save it as name (default NewShow_<date>UTC). \
                          Asks first, offering to save unsaved changes.",
                examples: &["NewShow \"Rehearsal\""],
                related: &["saveshow", "loadshow"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Master,
            rights: Rights::Admin,
            reach: Reach::Drive,
            flags: SHOW_FILE_FLAGS,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::words::cmd_delete_show,
        decl: CommandDecl {
            verb: "deleteshow",
            doc: Doc {
                display: "DeleteShow",
                category: HelpCategory::System,
                syntax: "DeleteShow \"name\" [/NoConfirmation]",
                description: "Delete a show file from library/shows (not the loaded show).",
                examples: &["DeleteShow \"Old Rehearsal\""],
                related: &["saveshow", "loadshow"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Master,
            rights: Rights::Program,
            reach: Reach::Drive,
            flags: SHOW_FILE_FLAGS,
            ..CommandDecl::BASE
        },
    },

];

/// The show file's objects with their behaviour; `register` adds them to the catalog.
pub static OBJECTS: &[&ObjectDef] = &[
    &crate::show::SHOW,
    &crate::copies::VERSION,
    &crate::copies::RECOVERY,
    &crate::library::LIBRARY,
];

/// What the show file adds to how a line runs: `Delete Show` asks itself.
pub static HOOKS: &[Hook] = &[Hook::DeleteNeverAsks(&crate::show::SHOW)];
