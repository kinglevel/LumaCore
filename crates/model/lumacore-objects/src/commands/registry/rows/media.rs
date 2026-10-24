// ███╗   ███╗███████╗██████╗ ██╗ █████╗
// ████╗ ████║██╔════╝██╔══██╗██║██╔══██╗
// ██╔████╔██║█████╗  ██║  ██║██║███████║
// ██║╚██╔╝██║██╔══╝  ██║  ██║██║██╔══██║
// ██║ ╚═╝ ██║███████╗██████╔╝██║██║  ██║
// ╚═╝     ╚═╝╚══════╝╚═════╝ ╚═╝╚═╝  ╚═╝
// FILE: media

//! The command and object rows of media: Paint (the verb), Image, Icon, Theme, Slot, LockScreen, Asset, AssetRoot
//! (decisions §34 25.7, 25.13: each crate of 34g4 takes its file).
//!
//! Owns:          the media family's rows (`ROWS`, `OBJECTS`): each word's declaration and behaviour.
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

/// This family's commands with their handlers.
pub(crate) static ROWS: &[CommandDef] = &[CommandDef {
    run: crate::commands::typed::typed::<crate::commands::paint_object::PaintObject>,
    decl: CommandDecl {
        verb: "paint",
        doc: Doc {
            display: "Paint",
            category: HelpCategory::System,
            syntax: "Paint <object> <ids>",
            description: "Open the paint picker on objects (decisions 0296): Palette, Free or a link to \
                      a theme slot; its answer is one Set <object> \"Paint\" <value> line. Any object \
                      with a Paint: groups, sequences, cues, presets, macros, views, tags, fixtures. A \
                      terminal gets the Paint now and the line to type. Alt+A in PRG: Assign › Label › \
                      Paint › Tag.",
            examples: &[
                "Paint Group 1                 -- the picker on Group 1",
                "Paint Sequence 1              -- on Sequence 1",
                "Paint Group 1 Thru 3          -- one answer paints all three",
            ],
            related: &["set", "label", "assign"],
            ..Doc::BASE
        },
        undo: Undo::ReadOnly,
        scope: Scope::Client,
        rights: Rights::Guest,
        reach: Reach::Show,
        ..CommandDecl::BASE
    },
}];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub(crate) static OBJECTS: &[&ObjectDef] = &[
    &crate::commands::media::IMAGE,
    &crate::commands::media::THEME,
    &crate::commands::media::SLOT,
    &crate::commands::media::LOCK_SCREEN,
    &crate::commands::media::ICON,
    &crate::commands::media::ASSET_ROOT,
    &crate::commands::media::ASSET,
];

/// What this family adds to how a line runs (`commands::hooks`).
pub(crate) static HOOKS: &[Hook] = &[];
