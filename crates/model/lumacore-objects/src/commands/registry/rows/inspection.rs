// ██╗███╗   ██╗███████╗██████╗ ███████╗ ██████╗████████╗██╗ ██████╗ ███╗   ██╗
// ██║████╗  ██║██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝██║██╔═══██╗████╗  ██║
// ██║██╔██╗ ██║███████╗██████╔╝█████╗  ██║        ██║   ██║██║   ██║██╔██╗ ██║
// ██║██║╚██╗██║╚════██║██╔═══╝ ██╔══╝  ██║        ██║   ██║██║   ██║██║╚██╗██║
// ██║██║ ╚████║███████║██║     ███████╗╚██████╗   ██║   ██║╚██████╔╝██║ ╚████║
// ╚═╝╚═╝  ╚═══╝╚══════╝╚═╝     ╚══════╝ ╚═════╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: inspection

//! The command rows of the reading verbs every object shares: Info, List, Help, Dump, EditSetting,
//! Import / Export, QuickSheet (decisions §34 25.7, 25.13: the kernel's, with dispatch).
//!
//! Owns:          the inspection family's command rows (`ROWS`): each word's declaration and handler.
//! Doesn't own:   the handlers' work (their command modules), the catalog's checks (registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13.

use crate::commands::registry::*;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This family's commands with their handlers; `commands::register` adds them to the catalog.
pub(crate) static ROWS: &[CommandDef] = &[
    CommandDef {
        run: |s, a| out(crate::commands::edit_setting::cmd_edit_setting(s, a)),
        decl: CommandDecl {
            verb: "editsetting",
            aliases: &["editsettings"],
            doc: Doc {
                display: "EditSetting",
                category: HelpCategory::Inspection,
                syntax: "EditSetting <object> <n> | EditSetting Show",
                description: "An object's settings: in the web and app a settings \
                          pop-up, in a terminal the settings with their values and the Set lines \
                          that change them. `EditSetting Show` gives the show settings.",
                examples: &["EditSetting Fixture 3", "EditSetting Output 1", "EditSetting Show"],
                related: &["set", "info"],
                ..Doc::BASE
            },
            object_verb: Some(Verb::EditSetting),
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Settings),
            scope: Scope::Client,
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(info::handle_info(s, a)),
        decl: CommandDecl {
            verb: "info",
            aliases: &["i"],
            doc: Doc {
                display: "Info",
                category: HelpCategory::Inspection,
                syntax: "Info <type> [<id>]",
                description: "Inspect any object in the show. \
                          Displays a formatted property sheet with all fields. \
                          Supports Fixture, Universe, FixtureType, Group, Show, Clock.",
                examples: &[
                    "Info Fixture 1      -- show fixture 1 details",
                    "Info Universe 1     -- show universe 1",
                    "Info Clock          -- timing diagnostics",
                ],
                related: &["list", "dump"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Inspection),
            scope: Scope::Client,
            object_verb: Some(Verb::Info),
            rights: Rights::Guest,
            class: CommandClass::Query,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(list::handle_list(s, a)),
        decl: CommandDecl {
            verb: "list",
            doc: Doc {
                display: "List",
                category: HelpCategory::Inspection,
                syntax: "List [<type>] [/r]",
                description: "List objects by type pool. \
                          Bare `list` shows a summary overview. \
                          Supports: fixture, group, universe, fixturetype, undo, keys, settings.",
                examples: &[
                    "list            -- show overview summary",
                    "list fixture    -- list all fixtures",
                    "list keys       -- show all key bindings",
                    "list settings   -- show user settings",
                ],
                related: &["ls", "info", "dump"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Inspection),
            scope: Scope::Client,
            object_verb: Some(Verb::List),
            args: Args::ObjectsOrTree,
            rights: Rights::Guest,
            class: CommandClass::Query,
            reach: Reach::Show,
            flags: LIST_FLAGS,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(help::cmd_help(s, a)),
        decl: CommandDecl {
            verb: "help",
            aliases: &["h", "?"],
            doc: Doc {
                display: "Help",
                category: HelpCategory::Inspection,
                syntax: "Help [<command>]",
                description: "Show the help overview or detailed documentation for one command. \
                          Bare `Help` lists all commands grouped by category. \
                          `Help <command>` shows syntax, description, examples, and related commands. \
                          Add `?` after any command for a quick syntax hint: `at?`",
                examples: &[
                    "help         -- show categorized overview",
                    "help at      -- full At documentation",
                    "at?          -- quick one-liner syntax hint",
                ],
                related: &["list"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Inspection),
            scope: Scope::Client,
            args: Args::Words,
            rights: Rights::Guest,
            class: CommandClass::Query,
            reach: Reach::Show,
            flags: &[flags::ALL],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(show::handle_dump(s, a)),
        decl: CommandDecl {
            verb: "dump",
            doc: Doc {
                display: "Dump",
                category: HelpCategory::Inspection,
                syntax: "Dump <type> [<id>]",
                description: "Serialize an object to the terminal as formatted JSON. \
                          Useful for debugging or exporting raw data. \
                          Supports: Fixture, Universe, FixtureType, Group, Programmer, ShowData.",
                examples: &[
                    "Dump Fixture 1     -- dump fixture 1 as JSON",
                    "Dump ShowData      -- dump entire show",
                ],
                related: &["info", "export"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Inspection),
            scope: Scope::Client,
            object_verb: Some(Verb::Dump),
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(import::handle_import(s, a)),
        // Refined from what changed: Patch for a GDTF, Show / Settings for
        // pictures and profiles (decisions §29 "11c").
        decl: CommandDecl {
            verb: "import",
            aliases: &["imp"],
            doc: Doc {
                display: "Import",
                category: HelpCategory::System,
                syntax: "Import FixtureType \"file.gdtf\" | Import Image|Icon|Asset … | Import UserProfile [\"name\"] … | Import Show \"name\" <parts | objects>",
                description: "Import a file into the show: a GDTF fixture type (the same \
                          FixtureTypeID adds a revision), a picture (Image, Icon, \
                          UserProfile Image|Icon from library/media/), a profile \
                          (library/userprofiles/*.lumaprofile) or an asset on an asset drive.",
                examples: &[
                    "Import FixtureType \"fixture.gdtf\"  -- a GDTF of library/gdtf/",
                    "Import Image \"stage.jpg\"           -- the show's Images",
                    "Import UserProfile Image \"bg.jpg\"  -- your profile's Images",
                    "Import UserProfile \"Bob\"           -- a profile file",
                    "Import Show \"Festival\" Patch       -- a part of another show, by GUID",
                ],
                related: &["patch", "library"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: main(OopsCategory::Show),
            scope: Scope::Master,
            object_verb: Some(Verb::Import),
            rights: Rights::Program,
            reach: Reach::Drive,
            flags: IMPORT_FLAGS,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(info::handle_export(s, a)),
        decl: CommandDecl {
            verb: "export",
            aliases: &["exp"],
            doc: Doc {
                display: "Export",
                category: HelpCategory::Inspection,
                syntax: "Export UserProfile [\"name\"] | Export Image|Icon <n> | Export <type> [<id>] \"file.json\"",
                description: "Write show data to a file of this station's library: a profile with its \
                          pictures (library/userprofiles/*.lumaprofile, for Import UserProfile in \
                          another show), a picture back to library/media/, or an object's JSON.",
                examples: &[
                    "Export UserProfile \"Bob\"   -- library/userprofiles/Bob.lumaprofile",
                    "Export Image 3",
                    "Export Fixture 1 \"f.json\"",
                ],
                related: &["import", "userprofile", "image"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            object_verb: Some(Verb::Export),
            rights: Rights::Program,
            reach: Reach::Drive,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, _| out(help::quicksheet(s)),
        decl: CommandDecl {
            verb: "quicksheet",
            doc: Doc {
                display: "",
                category: HelpCategory::System,
                syntax: "Quicksheet",
                description: "Show command reference and tree browser. Triggered by Tab on empty prompt.",
                related: &["help", "ls"],
                hidden: true,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            rights: Rights::Guest,
            class: CommandClass::Query,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub(crate) static OBJECTS: &[&ObjectDef] = &[&crate::objects::root::ROOT];

/// What this family adds to how a line runs (`commands::hooks`).
pub(crate) static HOOKS: &[Hook] = &[];
