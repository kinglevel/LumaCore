// ███████╗██╗  ██╗ ██████╗ ██╗    ██╗███████╗██╗██╗     ███████╗
// ██╔════╝██║  ██║██╔═══██╗██║    ██║██╔════╝██║██║     ██╔════╝
// ███████╗███████║██║   ██║██║ █╗ ██║█████╗  ██║██║     █████╗
// ╚════██║██╔══██║██║   ██║██║███╗██║██╔══╝  ██║██║     ██╔══╝
// ███████║██║  ██║╚██████╔╝╚███╔███╔╝██║     ██║███████╗███████╗
// ╚══════╝╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚═╝     ╚═╝╚══════╝╚══════╝
// FILE: showfile

//! The show file (decisions §30 "12d part 5", §34 25.7): the words that save, load, start, delete
//! and take parts of shows — SaveShow, LoadShow, NewShow, DeleteShow, CleanStart, Import Show —
//! the Show, Version, Recovery and Library objects, the Backup menu, and the service the server
//! starts with (the startup show, the Admin user, the timed recovery copy, the show's status).
//! Restart and Shutdown ride here until `cmd-station` is cut: their "save first?" is this crate's.
//! The `.luma` file, the disk and the recovery copies' rules are `lumacore-luma` and the state's.
//!
//! Owns:          the rows (`rows`), the words, the objects, the Backup menu, the show-file service.
//! Doesn't own:   the file format and the disk (lumacore-luma), recovery's rules (lumacore-state).
//! Entry points:  register, load_show, save_show, clean_start, ensure_admin_user, autosave_tick, show_info, parts_view.
//! Decisions:     §20, §30 "12d part 5", §34 25.7.

mod ask;
mod copies;
mod import;
mod library;
mod load;
mod menu;
mod rows;
mod save;
mod show;
mod words;

pub use import::{by_no_or_name, parts_view};
pub use load::load_show;
pub use lumacore_luma::backups::{SHOWS_DIR, list_show_files, show_path};
pub use rows::{HOOKS, OBJECTS, ROWS};
pub use save::{ensure_admin_user, save_show};
pub use show::{autosave_in, autosave_tick, list_shows, show_info};
pub use ask::SHOW_FILE_FLAGS;
pub use words::{clean_start, cmd_power};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// Adds the show file's rows, hook and menu to the catalog, after the kernel's.
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_objects::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(ROWS);
    r.commands(ROWS.iter().map(|c| &c.decl));
    r.rows(OBJECTS);
    r.objects(OBJECTS.iter().map(|o| &o.decl));
    r.rows(HOOKS);
    r.menus(menu::MENUS);
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

/// A test binary's catalog: the kernel's and the show file's (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
