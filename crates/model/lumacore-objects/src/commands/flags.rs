//! The flags' table reads (decisions §31, §34 15): every declared flag, from the registry's
//! contents. The declarations themselves live with the state (`crate::flags`).

pub use crate::flags::*;

/// Every declared flag: globals, every command's and every object's (help, the sweep, tidy).
pub fn all() -> impl Iterator<Item = &'static FlagDef> {
    lumacore_registry::get().global_flags()
        .chain(crate::commands::registry::all().flat_map(|c| c.flags.iter()))
        .chain(crate::objects::all().flat_map(|o| o.flags.iter()))
}

