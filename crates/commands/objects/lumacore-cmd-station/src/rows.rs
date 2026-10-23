// ██████╗  ██████╗ ██╗    ██╗███████╗
// ██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██████╔╝██║   ██║██║ █╗ ██║███████╗
// ██╔══██╗██║   ██║██║███╗██║╚════██║
// ██║  ██║╚██████╔╝╚███╔███╔╝███████║
// ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: rows

//! The command rows of this station: Restart, Shutdown, Connect, Disconnect, Unwatch, Echo, Debug,
//! the splash (decisions §34 25.7, 25.13: each crate of 34g4 takes its file).
//!
//! Owns:          the station family's command rows (`ROWS`): each word's declaration and handler.
//! Doesn't own:   the handlers' work (their command modules), the catalog's checks (registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13.

use lumacore_objects::commands::registry::*;
use lumacore_objects::commands::session;
use lumacore_objects::objects::{ObjectDef, out};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// This family's commands with their handlers; `commands::register` adds them to the catalog.
pub static ROWS: &[CommandDef] = &[
    // Eject Drive "usb1" (decisions §38): the one new verb of the drives round, planned until built.
    CommandDef {
        run: lumacore_objects::commands::typed::typed::<crate::drives::EjectPlanned>,
        decl: CommandDecl {
            verb: "eject",
            doc: lumacore_objects::commands::registry::Doc {
                display: "Eject",
                category: lumacore_objects::commands::help::HelpCategory::System,
                syntax: "Eject Drive \"<id>\"",
                description: "Unmount a drive so it can be taken out (the row goes Out). Not built yet (planned).",
                examples: &["Eject Drive \"usb1\""],
                related: &["drive"],
                ..lumacore_objects::commands::registry::Doc::BASE
            },
            undo: lumacore_objects::commands::registry::Undo::ReadOnly,
            family: None,
            scope: lumacore_objects::commands::registry::Scope::Station,
            status: lumacore_objects::commands::registry::Status::Planned,
            rights: lumacore_objects::commands::registry::Rights::Setup,
            reach: lumacore_objects::commands::registry::Reach::Drive,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: lumacore_objects::commands::run::disconnect,
        decl: CommandDecl {
            verb: "disconnect",
            doc: Doc {
                display: "Disconnect",
                category: HelpCategory::System,
                syntax: "Disconnect",
                description: "Log out and leave this server. The app shows its server list (discovered \
                          servers, Start local server, manual address).",
                examples: &["disconnect    -- back to the server list"],
                related: &["connect", "logout"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Session),
            scope: Scope::Client,
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(session::cmd_connect(s, a)),
        decl: CommandDecl {
            verb: "connect",
            doc: Doc {
                display: "Connect",
                category: HelpCategory::System,
                syntax: "Connect \"Station\" | Connect [IP] host[:port] [User \"name\" [\"password\"]]",
                description: "Switch this app window to another LumaCore server, found by station name \
                          (discovery) or address, and log in. Asks for the password when the user \
                          has one and none is given; the password is masked in echo, history and \
                          logs. LumaCore app only.",
                examples: &[
                    "connect \"FOH\"                       -- the station named FOH",
                    "connect ip 10.0.0.5 user \"LD\"        -- by address, as LD",
                    "connect 10.0.0.5:47656 user \"LD\" \"pw\"",
                ],
                related: &["disconnect", "login", "logout"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Session),
            scope: Scope::Client,
            rights: Rights::Guest,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::words::shutoff,
        decl: CommandDecl {
            verb: "lumacore-shutoff",
            doc: Doc {
                display: "LumaCore-Shutoff",
                category: HelpCategory::System,
                syntax: "lumacore-shutoff",
                description: "Shut down the LumaCore server (asks for confirmation). \
                          Use this instead of `shutdown` when you want a safety prompt \
                          to prevent accidental shutdowns during a live show.",
                examples: &["lumacore-shutoff    -- prompts 'Shut down the server? (yes/no)'"],
                related: &["shutdown"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Station,
            rights: Rights::Setup,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(crate::watch::handle_unwatch(s, a)),
        decl: CommandDecl {
            verb: "unwatch",
            aliases: &["stopwatch"],
            doc: Doc {
                display: "Unwatch",
                category: HelpCategory::System,
                syntax: "Unwatch [<id | name | all>]",
                description: "Stop watching one or all active watch commands.",
                examples: &[
                    "unwatch           -- stop all watches",
                    "unwatch 1         -- stop watch #1",
                    "unwatch clock     -- stop watching clock",
                ],
                related: &["watch"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: UI,
            family: Some(CommandFamily::Inspection),
            scope: Scope::Client,
            args: Args::Watches,
            rights: Rights::Guest,
            class: CommandClass::Session,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::words::echo,
        decl: CommandDecl {
            verb: "echo",
            doc: Doc {
                display: "Echo",
                category: HelpCategory::System,
                syntax: "Echo \"text\"",
                description: "Show text in the command line output and the System Monitor. \
                          Useful in macros to mark steps.",
                examples: &["echo \"Act 2 starts\"    -- a marker in the System Monitor"],
                related: &["list", "help"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Client,
            // Free text: a `/word` in it is text, not a flag (decisions §31).
            args: Args::Words,
            rights: Rights::Guest,
            class: CommandClass::Query,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| out(crate::splash::banner_string()),
        decl: CommandDecl {
            verb: "banner",
            doc: Doc {
                display: "",
                category: HelpCategory::System,
                syntax: "Banner",
                description: "Show ASCII banner.",
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
    CommandDef {
        run: crate::words::debug,
        decl: CommandDecl {
            verb: "debug",
            doc: Doc {
                display: "",
                category: HelpCategory::System,
                syntax: "Debug",
                description: "Toggle debug mode (Admin only).",
                alias_depth: AliasDepth::Full,
                hidden: true,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            scope: Scope::Station,
            rights: Rights::Admin,
            class: CommandClass::DangerousRuntime,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| {
            out(lumacore_objects::errors::render_code(
                "HissMe is reserved for the internal root monitor.",
                "E-RIGHTS-001",
            ))
        },
        decl: CommandDecl {
            verb: "hissme",
            doc: Doc {
                display: "HissMe",
                category: HelpCategory::System,
                syntax: "HissMe",
                description: "Crash report of the server, for the internal root monitor only.",
                alias_depth: AliasDepth::Full,
                hidden: true,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Runtime),
            scope: Scope::Station,
            rights: Rights::Internal,
            class: CommandClass::DangerousRuntime,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| lumacore_cmd_showfile::cmd_power(s, a, true),
        // Its save stamps ShowSettings: replicated before exit (issue 06).
        decl: CommandDecl {
            verb: "restart",
            doc: Doc {
                display: "Restart",
                category: HelpCategory::System,
                syntax: "Restart [/Save | /NoSave] [/NoAutoClose] [/NoConfirmation]",
                description: "Restart LumaCore: shut down and start the program again — \
                          a rebuilt binary is picked up. The startup show (by default the last show) \
                          is loaded again.",
                examples: &["restart", "restart /Save /NoConfirmation"],
                related: &["shutdown", "show"],
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            scope: Scope::Station,
            rights: Rights::Setup,
            reach: Reach::Drive,
            // Its "save first?" answer appends /Save or /NoSave (`backup::parse`).
            flags: lumacore_cmd_showfile::SHOW_FILE_FLAGS,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| lumacore_cmd_showfile::cmd_power(s, a, false),
        // Its save stamps ShowSettings: replicated before exit (issue 06).
        decl: CommandDecl {
            verb: "shutdown",
            aliases: &["turnoff"],
            doc: Doc {
                display: "Shutdown",
                category: HelpCategory::System,
                syntax: "Shutdown [/Save | /NoSave] [/NoAutoClose] [/NoConfirmation]",
                description: "Quit LumaCore. Asks first — in the UI with a 10 s \
                          countdown unless /NoAutoClose — and offers to save unsaved changes. \
                          `Exit` navigates up a context level, not out of the app.",
                examples: &[
                    "shutdown    -- quit LumaCore (asks first)",
                    "shutdown /Save /NoConfirmation -- save and quit at once",
                    "turnoff     -- alias for shutdown",
                ],
                related: &["exit", "lumacore-shutoff"],
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            family: Some(CommandFamily::Runtime),
            scope: Scope::Station,
            rights: Rights::Setup,
            class: CommandClass::DangerousRuntime,
            reach: Reach::Drive,
            // Its "save first?" answer appends /Save or /NoSave (`backup::parse`).
            flags: lumacore_cmd_showfile::SHOW_FILE_FLAGS,
            ..CommandDecl::BASE
        },
    },
];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub static OBJECTS: &[&ObjectDef] = &[
    &crate::drives::DRIVE,
    // The station and its network (decisions §34 25.17).
    &crate::stations::STATION,
    &crate::stations::INTERFACE,
    &crate::network::NETWORK,
    &crate::objects::SERVER,
    &crate::objects::SHELL,
    &crate::objects::CLIENT,
    &crate::objects::CLOCK,
    &crate::objects::UI_ENGINE,
    &crate::watch::WATCH,
    &crate::objects::JOURNAL,
    &crate::objects::HEALTH,
];

/// What this family adds to how a line runs (`commands::hooks`).
pub static HOOKS: &[Hook] = &[
    Hook::Intercept(crate::watch::try_watch),
    Hook::SetCheckedElsewhere(&crate::stations::STATION, crate::wiring::names_a_station),
    // Import Show "x" Network "Rig" / Station "Tower": their GUIDs in the other show.
    Hook::ImportByGuid(&crate::network::NETWORK, |s, _, id| {
        lumacore_cmd_showfile::by_no_or_name(&s.protocols.networks, id, |x| {
            (x.meta.identity.guid.to_string(), x.meta.identity.name.clone())
        })
    }),
    Hook::ImportByGuid(&crate::stations::STATION, |s, _, id| {
        lumacore_cmd_showfile::by_no_or_name(&s.stations, id, |x| {
            (x.meta.identity.guid.to_string(), x.meta.identity.name.clone())
        })
    }),
];
