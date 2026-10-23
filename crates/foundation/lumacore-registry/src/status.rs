// ███████╗████████╗ █████╗ ████████╗██╗   ██╗███████╗
// ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██║   ██║██╔════╝
// ███████╗   ██║   ███████║   ██║   ██║   ██║███████╗
// ╚════██║   ██║   ██╔══██║   ██║   ██║   ██║╚════██║
// ███████║   ██║   ██║  ██║   ██║   ╚██████╔╝███████║
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚══════╝
// FILE: status
//! Whether a declared thing works yet (decisions §15): `Ready`, or `Planned` — published so it can
//! be discovered, not built (running it says so, E-CMD-003). Commands, objects' verbs and tools
//! carry it (decisions §34 21).
//!
//! Owns:          Status
//! Doesn't own:   the "planned" reply (dispatch)
//! Entry points:  Status
//! Decisions:     §15, §34 21

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Whether a declared thing works yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ready,
    /// Published so it can be discovered, but not implemented: help and
    /// completion mark it, running it says so.
    Planned,
}
