// ███████╗████████╗ █████╗ ████████╗██╗ ██████╗ ███╗   ██╗
// ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██║██╔═══██╗████╗  ██║
// ███████╗   ██║   ███████║   ██║   ██║██║   ██║██╔██╗ ██║
// ╚════██║   ██║   ██╔══██║   ██║   ██║██║   ██║██║╚██╗██║
// ███████║   ██║   ██║  ██║   ██║   ██║╚██████╔╝██║ ╚████║
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: station

//! The station's own words (decisions §34 25.7): the objects of this running station — Server,
//! Shell, Client, Clock, Journal, Health, UIEngine, Watch — and Restart, Shutdown, Shutoff,
//! Disconnect, Connect, Watch / Unwatch (and `<line> /Watch`, its hook), Echo, Debug, HissMe and
//! Banner; the splash the terminal shows when the REPL starts. Restart / Shutdown's "save first?"
//! is `lumacore-cmd-showfile`'s, which this crate sits on. The Station and Interface objects stay
//! in the kernel: the station table, the Network object, the menus and DeskLock name them.
//!
//! Owns:          the rows (`rows`), the station's objects, Watch, the splash.
//! Doesn't own:   Station / Interface (the kernel's), the save before power (cmd-showfile).
//! Entry points:  register, splash, handle_watch, watchable, ROWS, OBJECTS, HOOKS.
//! Decisions:     §25, §29, §34 25.7.

mod drives;
mod menus;
mod network;
mod objects;
mod rows;
mod settings;
mod splash;
mod stations;
mod table;
mod watch;
mod wiring;
mod words;

pub use drives::{DRIVE, DRIVE_SETTINGS};
pub use network::NETWORK;
pub use rows::{HOOKS, OBJECTS, ROWS};
pub use settings::{PRIORITY_CHOICES, STATION_SETTINGS, THEME_OVERRIDE_CHOICES};
pub use splash::banner_string as splash;
pub use stations::{INTERFACE, STATION};
pub use table::cmd_delete_station;
pub use watch::{handle_watch, watchable};
pub use wiring::{INTERFACE_SETTINGS, NETWORK_SETTINGS, names_a_station};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// Adds the station's rows and its `/Watch` hook to the catalog, after the show file's.
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_cmd_showfile::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(ROWS);
    r.commands(ROWS.iter().map(|c| &c.decl));
    r.rows(OBJECTS);
    r.objects(OBJECTS.iter().map(|o| &o.decl));
    r.rows(HOOKS);
    r.menus(menus::MENUS);
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

/// A test binary's catalog: the kernel's, the show file's and the station's (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
