// ████████╗██████╗  █████╗  ██████╗███████╗
// ╚══██╔══╝██╔══██╗██╔══██╗██╔════╝██╔════╝
//    ██║   ██████╔╝███████║██║     █████╗
//    ██║   ██╔══██╗██╔══██║██║     ██╔══╝
//    ██║   ██║  ██║██║  ██║╚██████╗███████╗
//    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝
// FILE: trace
//! The Perfetto trace collector and the trace macros of every crate — see README.md.

pub mod collector;
pub mod format;
pub mod guard;
pub mod macros;

pub use collector::{TRACE_COLLECTOR, TraceCollector, TraceEvent};
pub use guard::PerfettoGuard;
pub use macros::{HeldZone, record_complete};
/// For the macros' Tracy forms (`$crate::tracy_client::span!`).
#[cfg(feature = "tracy")]
pub use tracy_client;
