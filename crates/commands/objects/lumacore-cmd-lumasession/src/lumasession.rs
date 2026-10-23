// ██╗     ██╗   ██╗███╗   ███╗ █████╗ ███████╗███████╗███████╗███████╗██╗ ██████╗ ███╗   ██╗
// ██║     ██║   ██║████╗ ████║██╔══██╗██╔════╝██╔════╝██╔════╝██╔════╝██║██╔═══██╗████╗  ██║
// ██║     ██║   ██║██╔████╔██║███████║███████╗█████╗  ███████╗███████╗██║██║   ██║██╔██╗ ██║
// ██║     ██║   ██║██║╚██╔╝██║██╔══██║╚════██║██╔══╝  ╚════██║╚════██║██║██║   ██║██║╚██╗██║
// ███████╗╚██████╔╝██║ ╚═╝ ██║██║  ██║███████║███████╗███████║███████║██║╚██████╔╝██║ ╚████║
// ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: lumasession

//! The LumaSession object (decisions §11): server-to-server sessions.
//!
//! Owns:          the LumaSession object: List, Info, Join, Leave, Claim, Pause.
//! Doesn't own:   the listings (the kernel's list, info).
//! Entry points:  LUMASESSION.
//! Decisions:     §11, §21.

use lumacore_objects::objects::{ObjectDef, out};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

// LumaSession: the server-to-server session (decisions §11, §21).
lumacore_objects::listed_object!(
lum lumacore_objects::objects::LumShape::ids(1),
default Info,
station LUMASESSION,
"LumaSession",
["lumasessions"],
"Server-to-server sessions (decisions §11): Join LumaSession \"name\" | <host>, Leave LumaSession (asks), Claim LumaSession (take the master role; asks), Claim LumaSession Station \"X\" (the master hands the role to member X; asks), List LumaSession (every station discovery sees), Info LumaSession",
[
    List => |s, a| out(lumacore_objects::commands::list::lumasessions(s, a)),
    Info => |s, _| out(lumacore_objects::commands::info::info_lumasession(s)),
    Join => |s, a| out(crate::join::cmd_join_lumasession(s, a)),
    Leave => |s, a| crate::leave::cmd_leave_lumasession(s, a),
    Claim => crate::claim::cmd_claim,
    Pause => |s, _| out(crate::claim::cmd_pause(s)),
]);
