// ████████╗ ██████╗  ██████╗ ██╗
// ╚══██╔══╝██╔═══██╗██╔═══██╗██║
//    ██║   ██║   ██║██║   ██║██║
//    ██║   ██║   ██║██║   ██║██║
//    ██║   ╚██████╔╝╚██████╔╝███████╗
//    ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝
// FILE: tool
//! A group grid tool (decisions §30 "12e"): its id as typed, label, icon, toolbar group, what it
//! asks and whether it is built. Pure data: what it does to a group's members is bound in the
//! objects crate by id (decisions §34 21).
//!
//! Owns:          ToolDecl, ToolAsk
//! Doesn't own:   running a tool (lumacore-objects' tools), the toolbar (the kit)
//! Entry points:  ToolDecl, ToolAsk
//! Decisions:     §30 "12e", §34 21

use crate::status::Status;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a tool asks before it runs.
#[derive(Debug, Clone, Copy)]
pub enum ToolAsk {
    /// One of these (value, icon); the first is the default.
    Choice {
        label: &'static str,
        options: &'static [(&'static str, &'static str)],
    },
    /// A whole number in `min..=max`.
    Number {
        label: &'static str,
        min: i64,
        max: i64,
        default: i64,
    },
}

/// One tool as the catalog holds it.
#[derive(Debug)]
pub struct ToolDecl {
    /// As typed (quoted) in the line: `"RemoveGaps"`.
    pub id: &'static str,
    pub label: &'static str,
    /// A Lucide icon (the toolbar's button).
    pub icon: &'static str,
    /// The toolbar group it sits in (one of the registered groups).
    pub group: &'static str,
    pub ask: Option<ToolAsk>,
    pub status: Status,
}
