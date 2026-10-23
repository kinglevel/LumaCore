// ███╗   ███╗███████╗██████╗ ██╗ █████╗
// ████╗ ████║██╔════╝██╔══██╗██║██╔══██╗
// ██╔████╔██║█████╗  ██║  ██║██║███████║
// ██║╚██╔╝██║██╔══╝  ██║  ██║██║██╔══██║
// ██║ ╚═╝ ██║███████╗██████╔╝██║██║  ██║
// ╚═╝     ╚═╝╚══════╝╚═════╝ ╚═╝╚═╝  ╚═╝
// FILE: media

//! The command and object rows of media: Image, Icon, Theme, Paint, LockScreen, Asset, AssetRoot
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

/// This family's commands with their handlers (none of its own yet).
pub(crate) static ROWS: &[CommandDef] = &[];

/// This family's objects with their behaviour; `commands::register` adds them to the catalog.
pub(crate) static OBJECTS: &[&ObjectDef] = &[
    &crate::commands::media::IMAGE,
    &crate::commands::media::THEME,
    &crate::commands::media::PAINT,
    &crate::commands::media::LOCK_SCREEN,
    &crate::commands::media::ICON,
    &crate::commands::media::ASSET_ROOT,
    &crate::commands::media::ASSET,
];

/// What this family adds to how a line runs (`commands::hooks`).
pub(crate) static HOOKS: &[Hook] = &[];
