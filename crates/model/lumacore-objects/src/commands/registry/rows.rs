// ██████╗  ██████╗ ██╗    ██╗███████╗
// ██╔══██╗██╔═══██╗██║    ██║██╔════╝
// ██████╔╝██║   ██║██║ █╗ ██║███████╗
// ██╔══██╗██║   ██║██║███╗██║╚════██║
// ██║  ██║╚██████╔╝╚███╔███╔╝███████║
// ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝
// FILE: rows

//! The command rows, one file per family (decisions §34 25.13; each becomes its crate in 34g4).
//!
//! Owns:          the family modules.
//! Doesn't own:   the rows' types and the catalog push (registry.rs).
//! Entry points:  each family's ROWS.
//! Decisions:     §34 25.7, 25.13.

pub(crate) mod dispatch;
pub(crate) mod inspection;
pub(crate) mod media;
pub(crate) mod patch;
pub(crate) mod playback;
pub(crate) mod programmer;
pub(crate) mod users;
pub(crate) mod windows;
