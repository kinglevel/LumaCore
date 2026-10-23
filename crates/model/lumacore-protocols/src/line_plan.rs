// ██╗     ██╗███╗   ██╗███████╗    ██████╗ ██╗      █████╗ ███╗   ██╗
// ██║     ██║████╗  ██║██╔════╝    ██╔══██╗██║     ██╔══██╗████╗  ██║
// ██║     ██║██╔██╗ ██║█████╗      ██████╔╝██║     ███████║██╔██╗ ██║
// ██║     ██║██║╚██╗██║██╔══╝      ██╔═══╝ ██║     ██╔══██║██║╚██╗██║
// ███████╗██║██║ ╚████║███████╗    ██║     ███████╗██║  ██║██║ ╚████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝    ╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝
// FILE: line plan

//! What one output line sends (decisions §11 "Step 4c hardening"): the plan
//! `protocols::desired` makes and a line's driver (`lumacore_lines::lines`) follows. Pure
//! data, beside `InputPlan`, so the objects crate names it without the drivers.
//!
//! Owns:          LinePlan (what one output line sends).
//! Doesn't own:   the driver that follows it (lumacore_lines::lines), the plan that makes it (protocol_plan).
//! Entry points:  LinePlan.
//! Decisions:     §11 "Step 4c hardening", §34 20.3.

use std::net::Ipv4Addr;

use crate::line::LineSend;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What one output line sends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinePlan {
    pub line: u32,
    /// Unicast target; None = multicast (sACN) / ArtPoll + broadcast (Art-Net).
    pub unicast: Option<Ipv4Addr>,
    pub priority: u8,
    /// (local universe, protocol universe / Port-Address).
    pub routes: Vec<(u32, u16)>,
    /// Frames per second (1–60).
    pub rate_hz: u8,
    pub send: LineSend,
    /// µs between universes.
    pub packet_gap_us: u32,
    /// ArtSync / E1.31 sync after the line's universes.
    pub sync: bool,
}
