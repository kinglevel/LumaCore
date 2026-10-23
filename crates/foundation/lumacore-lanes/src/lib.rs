// ██╗      █████╗ ███╗   ██╗███████╗███████╗
// ██║     ██╔══██╗████╗  ██║██╔════╝██╔════╝
// ██║     ███████║██╔██╗ ██║█████╗  ███████╗
// ██║     ██╔══██║██║╚██╗██║██╔══╝  ╚════██║
// ███████╗██║  ██║██║ ╚████║███████╗███████║
// ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝
// FILE: lanes

//! A state thread's two job lanes — see the crate's README.md (decisions §26, §31, §34 24.2).

pub mod handle;
pub mod watchdog;

pub use handle::{Busy, CallError, Host, Job, Lane, Lanes, MACHINE_LANE, StateHandle, enter_state_thread, lanes};
pub use watchdog::{WATCHDOG_AFTER_MS, spawn_watchdog};
