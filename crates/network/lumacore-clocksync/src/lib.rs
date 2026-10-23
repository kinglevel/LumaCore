//  ██████╗██╗      ██████╗  ██████╗██╗  ██╗███████╗██╗   ██╗███╗   ██╗ ██████╗
// ██╔════╝██║     ██╔═══██╗██╔════╝██║ ██╔╝██╔════╝╚██╗ ██╔╝████╗  ██║██╔════╝
// ██║     ██║     ██║   ██║██║     █████╔╝ ███████╗ ╚████╔╝ ██╔██╗ ██║██║
// ██║     ██║     ██║   ██║██║     ██╔═██╗ ╚════██║  ╚██╔╝  ██║╚██╗██║██║
// ╚██████╗███████╗╚██████╔╝╚██████╗██║  ██╗███████║   ██║   ██║ ╚████║╚██████╗
//  ╚═════╝╚══════╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═══╝ ╚═════╝
// FILE: clocksync

//! The session clock sync — see the crate's README.md (decisions §24 "Session clock
//! (researched)", §34 24.2).

pub mod clock;

pub use clock::{Estimate, Filter, Sample, Target, answer, correction_for, follow, set_role, spawn_clock_sync};
