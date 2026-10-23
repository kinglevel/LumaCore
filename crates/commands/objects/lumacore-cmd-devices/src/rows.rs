// ██████╗  ██████╗ ██╗    ██╗███████╗
// ██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██████╔╝██║   ██║██║ █╗ ██║███████╗
// ██╔══██╗██║   ██║██║███╗██║╚════██║
// ██║  ██║╚██████╔╝╚███╔███╔╝███████║
// ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: rows

//! The command and object rows of devices: Device, DeskLock, NormalTheme, ScreenConfig (decisions
//! §34 25.7, 25.13: each crate of 34g4 takes its file).
//!
//! Owns:          the devices family's rows (`ROWS`, `OBJECTS`): each word's declaration and behaviour.
//! Doesn't own:   the handlers' work (their command modules), the catalog's checks (registry).
//! Entry points:  ROWS, OBJECTS, HOOKS.
//! Decisions:     §15, §34 21, 25.7, 25.13.

use lumacore_objects::commands::registry::*;
use lumacore_objects::objects::ObjectDef;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The devices' commands (none of their own: the objects take On / Off / Toggle).
pub static ROWS: &[CommandDef] = &[];

/// The devices' objects; `register` adds them to the catalog.
pub static OBJECTS: &[&ObjectDef] = &[
    &crate::screen_config::SCREEN_CONFIG,
    &crate::lock::DESK_LOCK,
    &crate::lock::NORMAL_THEME,
    &crate::device::DEVICE,
];

/// What the devices add to how a line runs: a locked device runs only its lock keys.
pub static HOOKS: &[Hook] = &[Hook::Gate(crate::lock::lock_gate)];
