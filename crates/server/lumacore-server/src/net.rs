// ███╗   ██╗███████╗████████╗
// ████╗  ██║██╔════╝╚══██╔══╝
// ██╔██╗ ██║█████╗     ██║
// ██║╚██╗██║██╔══╝     ██║
// ██║ ╚████║███████╗   ██║
// ╚═╝  ╚═══╝╚══════╝   ╚═╝
// FILE: net

//! The station's network I/O (decisions §23 "Step 8"): the shell server,
//! the UDP command port, the discovery beacon, mDNS, the HTTP bridge to the
//! state thread. Threads here do their own I/O and hand work to the state
//! thread as jobs; LumaSession requests go to `lumasession/`. Nothing in
//! `core/` or `terminal/` imports this module.
//!
//! Owns:          the station's network threads (decisions §23 "Step 8"): the command port's handler, the listener and discovery threads, the network view, the HTTP bridge.
//! Doesn't own:   the leaf I/O (lumacore-net), the shell port (lumacore-shell), LumaSession (lumacore-lumasession).
//! Entry points:  handler, spawn, sync, console_bridge.
//! Decisions:     §23 "Step 8", §26, §34 24.2.

#[cfg(feature = "http")]
pub(crate) mod console_bridge;
pub(crate) mod handler;
pub mod spawn;
pub(crate) mod sync;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// How long a station stays in the discovery cache after its last beacon.
const SESSION_DISCOVERY_TTL_SECS: u64 = 3;

// *************************************************************************************************
// Tests
// *************************************************************************************************

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
