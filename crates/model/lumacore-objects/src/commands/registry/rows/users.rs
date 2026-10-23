// ██╗   ██╗███████╗███████╗██████╗ ███████╗
// ██║   ██║██╔════╝██╔════╝██╔══██╗██╔════╝
// ██║   ██║███████╗█████╗  ██████╔╝███████╗
// ██║   ██║╚════██║██╔══╝  ██╔══██╗╚════██║
// ╚██████╔╝███████║███████╗██║  ██║███████║
//  ╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝╚══════╝
// FILE: users

//! The command rows of users: Login, Logout, Consent (decisions §34 25.7, 25.13: each crate of
//! 34g4 takes its file). A login waiting for the Meow License runs only these (the gate, 0290).
//!
//! Owns:          the users family's command rows (`ROWS`): each word's declaration and handler.
//! Doesn't own:   the handlers' work (their command modules), the catalog's checks (registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13, docs/decisions/0290-meow-license-consent-on-every-login.

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
        run: run::logout,
        decl: CommandDecl {
            verb: "logout",
            aliases: &["logoff"],
            doc: Doc {
                display: "Logout",
                category: HelpCategory::System,
                syntax: "Logout | Logoff",
                description: "Log out of this server. The app shows the server's user list; lumacore-cli \
                          returns to its outer prompt.",
                examples: &["logout    -- log out (pick another user)"],
                related: &["login", "disconnect", "shutdown", "exit"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Session),
            scope: Scope::Client,
            rights: Rights::Guest,
            class: CommandClass::Session,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: session::login,
        decl: CommandDecl {
            verb: "login",
            doc: Doc {
                display: "Login",
                category: HelpCategory::System,
                syntax: "Login [User] \"name\" [\"password\"]",
                description: "Switch to another user on this server: their command line, \
                          programmer and screens. The user must exist (Store User); a password is \
                          required if set and is masked in echo, history and logs.",
                examples: &[
                    "login \"LD\"           -- switch to user LD",
                    "login \"LD\" \"pw\"      -- with a password",
                ],
                related: &["user", "logout", "list userprofile"],
                ..Doc::BASE
            },
            undo: Undo::Untracked,
            family: Some(CommandFamily::Session),
            scope: Scope::Client,
            rights: Rights::Guest,
            class: CommandClass::Session,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: crate::commands::typed::typed::<crate::commands::consent::Consent>,
        decl: CommandDecl {
            verb: "consent",
            doc: Doc {
                display: "Consent",
                category: HelpCategory::System,
                syntax: "Consent",
                description: "Accept the Meow License: every login asks for it before it is finished \
                          (the dialog's \"I consent\" runs this line). Until then the login runs only \
                          Consent, Login and Logout (E-USER-010). The station log notes each consent.",
                examples: &["consent    -- accept the Meow License and finish the login"],
                related: &["login", "logout"],
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Session),
            scope: Scope::Client,
            rights: Rights::Guest,
            class: CommandClass::Session,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub(crate) static OBJECTS: &[&ObjectDef] = &[
    &crate::commands::users::USER,
    &crate::objects::system::USER_PROFILE,
    &crate::objects::system::SETTINGS,
];

/// What this family adds to how a line runs (`commands::hooks`).
pub(crate) static HOOKS: &[Hook] = &[Hook::Gate(crate::commands::consent::consent_gate)];
