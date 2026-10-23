// ███████╗██╗   ██╗███████╗███╗   ███╗ ██████╗ ███╗   ██╗
// ██╔════╝╚██╗ ██╔╝██╔════╝████╗ ████║██╔═══██╗████╗  ██║
// ███████╗ ╚████╔╝ ███████╗██╔████╔██║██║   ██║██╔██╗ ██║
// ╚════██║  ╚██╔╝  ╚════██║██║╚██╔╝██║██║   ██║██║╚██╗██║
// ███████║   ██║   ███████║██║ ╚═╝ ██║╚██████╔╝██║ ╚████║
// ╚══════╝   ╚═╝   ╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: sysmon

//! The System Monitor, the station's logging system — see the crate's README.md (decisions §34
//! 25.10): the log (`monitor`: `SysmonLine`s in a ring, the daily file, subscribers), feedback events
//! (flashes, messages, progress, statuses, presence) and panic reports. It knows no domain, so
//! every crate can write to it; the render tick never does.

mod monitor;
pub mod feedback;
pub mod panic_report;

pub use monitor::*;
