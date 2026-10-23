// ██████╗  █████╗ ████████╗ ██████╗██╗  ██╗
// ██╔══██╗██╔══██╗╚══██╔══╝██╔════╝██║  ██║
// ██████╔╝███████║   ██║   ██║     ███████║
// ██╔═══╝ ██╔══██║   ██║   ██║     ██╔══██║
// ██║     ██║  ██║   ██║   ╚██████╗██║  ██║
// ╚═╝     ╚═╝  ╚═╝   ╚═╝    ╚═════╝╚═╝  ╚═╝
// FILE: patch

//! The command rows of the patch: Park, Unpark (decisions §34 25.7, 25.13: each crate of 34g4
//! takes its file); fixtures are made with Store and addressed with Assign (§33 48, round 3 19).
//!
//! Owns:          the patch family's command rows (`ROWS`): each word's declaration and handler.
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
        run: |s, a| out(control::cmd_park(s, a)),
        decl: CommandDecl {
            verb: "park",
            doc: Doc {
                display: "Park",
                category: HelpCategory::Universe,
                syntax: "Park [Fixture <sel>] [<attribute>] [At <value>] | Park DMX <universe.address> [At <0-255>]",
                description: "Hold fixture attributes (or a raw DMX channel) at a fixed value, above every \
                          other source and the masters. Bare Park (or Park Fixture 3) parks every attribute \
                          at its current output; `At` parks the Dimmer; name an attribute to park only it. \
                          Saved with the show. Release with `Unpark`.",
                examples: &[
                    "Park                   -- the selection, as it looks now",
                    "Park Fixture 3 At 50   -- Fixture 3's Dimmer at 50%",
                    "Park Fixture 3 Pan At 30",
                    "Park DMX 2.1 At 255    -- a raw channel",
                ],
                related: &["unpark", "dmx"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
    CommandDef {
        run: |s, a| out(control::cmd_unpark(s, a)),
        decl: CommandDecl {
            verb: "unpark",
            doc: Doc {
                display: "Unpark",
                category: HelpCategory::Universe,
                syntax: "Unpark [Fixture <sel> [<attribute>] | DMX <universe.address>]",
                description: "Release parked values: all of them, a fixture's (or one attribute), or a raw channel.",
                examples: &[
                    "Unpark                 -- release everything parked",
                    "Unpark Fixture 3 Pan   -- release one attribute",
                    "Unpark DMX 2.1         -- release channel 2.1",
                ],
                related: &["park"],
                ..Doc::BASE
            },
            undo: PROGRAMMER,
            family: Some(CommandFamily::Action),
            scope: Scope::Master,
            rights: Rights::Program,
            reach: Reach::Show,
            ..CommandDecl::BASE
        },
    },
];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub(crate) static OBJECTS: &[&ObjectDef] = &[
    &crate::objects::patch::FIXTURE,
    &crate::objects::patch::dmx::DMX,
    &crate::objects::patch::identify::IDENTIFY,
    &crate::objects::patch::fixture::multipatch::MULTIPATCH,
    &crate::objects::patch::FIXTURE_TYPE,
    &crate::objects::patch::mvr::MVR,
    &crate::objects::patch::UNIVERSE,
    &crate::objects::patch::DATAPOOL,
    &crate::objects::patch::devices_meta::MVR_LAYER,
    &crate::objects::patch::devices_meta::MVR_GROUP,
    &crate::objects::patch::devices_meta::MVR_CLASS,
    &crate::objects::patch::devices_meta::MVR_POSITION,
];

/// What this family adds to how a line runs (`commands::hooks`).
pub(crate) static HOOKS: &[Hook] = &[];
