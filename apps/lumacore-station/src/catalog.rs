//  ██████╗ █████╗ ████████╗ █████╗ ██╗      ██████╗  ██████╗
// ██╔════╝██╔══██╗╚══██╔══╝██╔══██╗██║     ██╔═══██╗██╔════╝
// ██║     ███████║   ██║   ███████║██║     ██║   ██║██║  ███╗
// ██║     ██╔══██║   ██║   ██╔══██║██║     ██║   ██║██║   ██║
// ╚██████╗██║  ██║   ██║   ██║  ██║███████╗╚██████╔╝╚██████╔╝
//  ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝ ╚═════╝  ╚═════╝
// FILE: catalog

//! The station's catalog (decisions §34 21, 24.1): the one list of every crate the program is
//! built from. Its binaries (`src/bin/`) install it and call one library function each.
//!
//! Owns:          register (every crate's rows, the protocols this station runs), install_catalog.
//! Doesn't own:   any row (each crate adds its own), the binaries' work (their crates).
//! Entry points:  install_catalog, register.
//! Decisions:     §34 21, 22.1, 24.1.

// ██╗███╗   ██╗██╗████████╗
// ██║████╗  ██║██║╚══██╔══╝
// ██║██╔██╗ ██║██║   ██║
// ██║██║╚██╗██║██║   ██║
// ██║██║ ╚████║██║   ██║
// ╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝
// PART: INIT

/// This build's git commit, stamped here at the top of the crate graph (`build.rs`; decisions §34
/// "Build times hardened" 1) and handed to `lumacore-luma`: build info, `Version`, saved shows.
/// `install_catalog` calls it; a main that reads the build id first (`--version`) calls it itself.
pub fn stamp() {
    lumacore_objects::set_git_commit(env!("LUMACORE_GIT_SHA"));
}

/// Builds, checks and installs the catalog once per process (the server, the REPL, the tools,
/// the sweep); a catalog that doesn't freeze is the reason the station can't start.
pub fn install_catalog() -> Result<(), String> {
    stamp();
    if lumacore_registry::installed() {
        return Ok(());
    }
    let mut r = lumacore_registry::Registry::new();
    register(&mut r);
    lumacore_registry::install(r.freeze().map_err(|e| e.to_string())?);
    Ok(())
}

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The station's rows in the catalog: the server and everything it builds on (the HTTP engine's
/// when it is built in), every protocol this station runs, and the sweep (decisions §34 21, 22.1).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_server::register(r);
    // Each protocol's object, after its row (decisions §34 "34f7c hardened" 9).
    lumacore_cmd_artnet::register(r);
    lumacore_cmd_sacn::register(r);
    lumacore_cmd_osc::register(r);
    lumacore_sweep::register(r);
    r.enter(env!("CARGO_PKG_NAME"));
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// A test in this crate reads the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
