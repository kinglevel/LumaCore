// ██╗  ██╗███████╗██╗   ██╗███████╗
// ██║ ██╔╝██╔════╝╚██╗ ██╔╝██╔════╝
// █████╔╝ █████╗   ╚████╔╝ ███████╗
// ██╔═██╗ ██╔══╝    ╚██╔╝  ╚════██║
// ██║  ██╗███████╗   ██║   ███████║
// ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝
// FILE: keys

//! The keys (decisions §17, §34 25.4): keyboard shortcut → UiKey → command lines. A binding
//! (`KeyBinding`) only names a UiKey; the UiKey holds one command line per gesture plus its Meow
//! lines. Command lines are checked when they are saved (`Bind`) and again when a show loads
//! (`Hook::ShowReplaced`): unknown words are refused on save and flagged on load, planned
//! commands are flagged. The core keys (F9–F12, Pause, Print Screen) can't be bound. The keys'
//! data stays in the tree (`UiKeys` on the profile, `Root.Resources.UIKeys`); the users' crate
//! may use this one, never the other way round.
//!
//! Owns:          the rows (`rows`), Bind / Unbind, Shortcuts, UIKey, KeyBindings, the checks, E-KEY.
//! Doesn't own:   the key data (lumacore-tree `ui_keys`, `keybindings`), the core key list (base).
//! Entry points:  register, all_keys, find, bindings, command_for, parse_key_name, check_line.
//! Decisions:     §17, §30 "12a3", §34 25.4.

mod bind;
mod errors;
mod key_bindings;
mod lines;
mod lookup;
mod rows;
mod shortcuts;
mod ui_key;

pub use bind::parse_key_name;
pub use lines::{check_after_load, check_line, key_problems};
pub use lookup::{all_keys, bindings, command_for, find};
pub use rows::{HOOKS, OBJECTS, ROWS};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// Adds the keys' rows, hook and error codes to the catalog, after the kernel's.
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_objects::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.errors(errors::ERRORS);
    r.rows(ROWS);
    r.commands(ROWS.iter().map(|c| &c.decl));
    r.rows(OBJECTS);
    r.objects(OBJECTS.iter().map(|o| &o.decl));
    r.rows(HOOKS);
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;

/// A test binary's catalog: the kernel's and the keys' (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
