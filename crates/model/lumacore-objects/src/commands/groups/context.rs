//  ██████╗ ██████╗ ███╗   ██╗████████╗███████╗██╗  ██╗████████╗
// ██╔════╝██╔═══██╗████╗  ██║╚══██╔══╝██╔════╝╚██╗██╔╝╚══██╔══╝
// ██║     ██║   ██║██╔██╗ ██║   ██║   █████╗   ╚███╔╝    ██║
// ██║     ██║   ██║██║╚██╗██║   ██║   ██╔══╝   ██╔██╗    ██║
// ╚██████╗╚██████╔╝██║ ╚████║   ██║   ███████╗██╔╝ ██╗   ██║
//  ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚══════╝╚═╝  ╚═╝   ╚═╝
// FILE: context

//! A group's own part of the one context menu (decisions §34 round 3, 18, the approved
//! `context-menu.html`): Select, Edit members, Info — below the common actions and the Set part
//! every object gets from its verbs and properties.
//!
//! Owns:          CONTEXT (the Group row).
//! Doesn't own:   the menu (`lumacore_views::context`), the Group object (`commands::groups`).
//! Entry points:  CONTEXT.
//! Decisions:     §34 round 3, 18.

use crate::objects::{ContextDef, ContextItem, Verb};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

pub static CONTEXT: &[ContextDef] = &[ContextDef {
    object: &super::GROUP.decl,
    items: &[
        ContextItem { label: "Select", icon: "mouse-pointer-2", verb: Verb::SelectFixtures, line: "{object} {rows}", live: true, value: "" },
        ContextItem { label: "Edit members", icon: "users", verb: Verb::Edit, line: "Edit {object} {rows}", live: true, value: "" },
        ContextItem { label: "Info", icon: "info", verb: Verb::Info, line: "Info {object} {rows}", live: true, value: "" },
    ],
}];
