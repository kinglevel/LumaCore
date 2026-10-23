// ██████╗ ███████╗██╗   ██╗██╗ ██████╗███████╗███████╗
// ██╔══██╗██╔════╝██║   ██║██║██╔════╝██╔════╝██╔════╝
// ██║  ██║█████╗  ██║   ██║██║██║     █████╗  ███████╗
// ██║  ██║██╔══╝  ╚██╗ ██╔╝██║██║     ██╔══╝  ╚════██║
// ██████╔╝███████╗ ╚████╔╝ ██║╚██████╗███████╗███████║
// ╚═════╝ ╚══════╝  ╚═══╝  ╚═╝ ╚═════╝╚══════╝╚══════╝
// FILE: devices

//! The devices (decisions §30 "12a3", "12c2", §34 25.7): every window of an app shares one device
//! id, a browser tab has its own; the station keeps each device's DeskLock and normal theme
//! (station-local, never replicated, not an Oops step). `DeskLock` locks the device a line came
//! from (`On DeskLock Station`: every device of the station) and the lock gate refuses every other
//! line from it; `NormalTheme` shows the factory look; `Device` / `Set Device "ScreenConfig"` and
//! the `ScreenConfig` object pick which of the user's screen configs a device shows. The device
//! table is the state's (`device_table`), the toggle shape and the window toggles (Sidebars,
//! ViewBar, FullScreen) the kernel's `devices`.
//!
//! Owns:          the rows (`rows`), DeskLock / NormalTheme and the lock gate, Device, ScreenConfig.
//! Doesn't own:   the device table (lumacore-state), the toggle shape and window toggles (kernel).
//! Entry points:  register, allowed_while_locked, ROWS, OBJECTS, HOOKS.
//! Decisions:     §30 "12a3", "12c2", §34 25.7, 25.17.

mod device;
mod lock;
mod rows;
mod screen_config;

pub use lock::allowed_while_locked;
pub use rows::{HOOKS, OBJECTS, ROWS};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// Adds the devices' rows and the lock gate to the catalog, after the kernel's.
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_cmd_station::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
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

/// A test binary's catalog: the kernel's and the devices' (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
