//  ██████╗ █████╗ ████████╗ █████╗ ██╗      ██████╗  ██████╗
// ██╔════╝██╔══██╗╚══██╔══╝██╔══██╗██║     ██╔═══██╗██╔════╝
// ██║     ███████║   ██║   ███████║██║     ██║   ██║██║  ███╗
// ██║     ██╔══██║   ██║   ██╔══██║██║     ██║   ██║██║   ██║
// ╚██████╗██║  ██║   ██║   ██║  ██║███████╗╚██████╔╝╚██████╔╝
//  ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝ ╚═════╝  ╚═════╝
// FILE: catalog

//! The whole catalog's tests (decisions §34 25.15): what holds for every row whatever crate adds
//! it, the lum.md examples and the Lum words against every word, and the generated reference.
//! They run on the station's catalog — the kernel's own tests see only the kernel's rows once
//! the command crates are cut out (§34 25.13).
//!
//!
//! Owns:          the tests over the whole catalog (`rows`, `lum`).
//! Doesn't own:   a crate's tests of its own logic (beside its code), the catalog (the station).
//! Entry points:  `cargo test -p lumacore-catalog-tests`.
//! Decisions:     §33 3, 57, §34 21, 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod import;
#[cfg(test)]
mod kernel;
#[cfg(test)]
mod lines;
#[cfg(test)]
mod lum;
#[cfg(test)]
mod menus;
#[cfg(test)]
mod paths;
#[cfg(test)]
mod protocols;
#[cfg(test)]
mod rights;
#[cfg(test)]
mod rows;
#[cfg(test)]
mod wiring;
#[cfg(test)]
mod words;

/// The test binary's catalog: the station's, every crate's rows (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(lumacore_station::register);
}
