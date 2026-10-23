// ██╗██████╗ ███████╗███╗   ██╗████████╗██╗███████╗██╗   ██╗██╗███╗   ██╗ ██████╗
// ██║██╔══██╗██╔════╝████╗  ██║╚══██╔══╝██║██╔════╝╚██╗ ██╔╝██║████╗  ██║██╔════╝
// ██║██║  ██║█████╗  ██╔██╗ ██║   ██║   ██║█████╗   ╚████╔╝ ██║██╔██╗ ██║██║  ███╗
// ██║██║  ██║██╔══╝  ██║╚██╗██║   ██║   ██║██╔══╝    ╚██╔╝  ██║██║╚██╗██║██║   ██║
// ██║██████╔╝███████╗██║ ╚████║   ██║   ██║██║        ██║   ██║██║ ╚████║╚██████╔╝
// ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝╚═╝        ╚═╝   ╚═╝╚═╝  ╚═══╝ ╚═════╝
// FILE: identifying

//! Who identifies what (decisions §34 round 3, "Identify, Probe and Find hardened"): one running
//! Identify per user, on the station that sends — never stored, never an Oops step, never
//! replicated as show data. The render engine gets every user's targets with the engine input;
//! the `Identify` object (lumacore-objects) starts, replaces and ends them, and ends what is left
//! over (logout, a lost holder, 5 minutes, leaving the patch).
//!
//! Owns:          Identifying, MAX_MS.
//! Doesn't own:   the words (`objects::patch::identify`), the output (`lumacore_render::identify`).
//! Entry points:  Identifying, AppState::identify.
//! Decisions:     §34 round 3, "Identify, Probe and Find hardened".

use lumacore_render::identify::Identify;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// How long an Identify runs before it ends itself.
pub const MAX_MS: u64 = 5 * 60 * 1000;

/// One user's running Identify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifying {
    pub targets: Vec<Identify>,
    /// What it names, for the warning strip and the System Monitor (`Fixture 104 + DMX 1.150`).
    pub what: String,
    /// When it started (session clock).
    pub started_ms: u64,
}
