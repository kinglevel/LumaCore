// ██████╗  ██████╗ ██╗    ██╗███████╗
// ██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██████╔╝██║   ██║██║ █╗ ██║███████╗
// ██╔══██╗██║   ██║██║███╗██║╚════██║
// ██║  ██║╚██████╔╝╚███╔███╔╝███████║
// ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: rows

//! The command rows of LumaSession: Join, Leave, Invite, Dismiss (decisions §34 25.7, 25.13: each
//! crate of 34g4 takes its file).
//!
//! Owns:          the lumasession family's command rows (`ROWS`): each word's declaration and handler.
//! Doesn't own:   the handlers' work (their command modules), the catalog's checks (registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13.

use lumacore_objects::commands::registry::*;
use lumacore_objects::objects::{ObjectDef, Verb};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The LumaSession words with their handlers; `register` adds them to the catalog.
pub static ROWS: &[CommandDef] = &[
    CommandDef {
        run: |_, _| lumacore_objects::commands::run::objects_usage(Verb::Join),
        decl: CommandDecl {
            verb: "join",
            doc: Doc {
                display: "Join",
                category: HelpCategory::System,
                syntax: "Join LumaSession \"name\" | Join LumaSession <host[:port]>",
                description: "Join a LumaSession: by name (found by discovery: its master, else a standalone station of that name) or by address for a station discovery can't see. The station takes the session's show and stops sending: its master sends.",
                examples: &["Join LumaSession \"Festival\"", "Join LumaSession 10.0.0.5"],
                related: &["leave", "lumasession", "invite"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Session),
            scope: Scope::Station,
            object_verb: Some(Verb::Join),
            rights: Rights::Setup,
            reach: Reach::Show,
            flags: &[lumacore_objects::commands::flags::MERGE],
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| lumacore_objects::commands::run::objects_usage(Verb::Leave),
        decl: CommandDecl {
            verb: "leave",
            doc: Doc {
                display: "Leave",
                category: HelpCategory::System,
                syntax: "Leave LumaSession",
                description: "Leave the LumaSession. The station keeps a copy of the show and becomes standalone with its DMX output held until Set Station \"HoldOutput\" Off.",
                examples: &["Leave LumaSession"],
                related: &["join", "lumasession", "station"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Session),
            scope: Scope::Station,
            object_verb: Some(Verb::Leave),
            rights: Rights::Setup,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| lumacore_objects::commands::run::objects_usage(Verb::Invite),
        decl: CommandDecl {
            verb: "invite",
            doc: Doc {
                display: "Invite",
                category: HelpCategory::System,
                syntax: "Invite Station \"name\"",
                description: "Invite a station into this station's LumaSession. A station with Invite on joins without asking and takes the session's show (unsaved changes are kept in library/shows/.autosave first); a station that leads that LumaSession asks its operator instead (Use Mine / Use Master / Cancel).",
                examples: &["Invite Station \"FOH-2\""],
                related: &["dismiss", "station", "lumasession"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Session),
            scope: Scope::Station,
            object_verb: Some(Verb::Invite),
            rights: Rights::Setup,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |_, _| lumacore_objects::commands::run::objects_usage(Verb::Dismiss),
        decl: CommandDecl {
            verb: "dismiss",
            doc: Doc {
                display: "Dismiss",
                category: HelpCategory::System,
                syntax: "Dismiss Station \"name\"",
                description: "Send a member out of this LumaSession. It becomes standalone with its DMX output held.",
                examples: &["Dismiss Station \"FOH-2\""],
                related: &["invite", "station", "lumasession"],
                alias_depth: AliasDepth::Full,
                ..Doc::BASE
            },
            undo: Undo::ReadOnly,
            family: Some(CommandFamily::Session),
            scope: Scope::Station,
            object_verb: Some(Verb::Dismiss),
            rights: Rights::Setup,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
];

/// The LumaSession object; `register` adds it to the catalog.
pub static OBJECTS: &[&ObjectDef] = &[&crate::lumasession::LUMASESSION];

/// What this family adds to how a line runs (`commands::hooks`).
pub static HOOKS: &[Hook] = &[];
