// ███████╗████████╗ █████╗ ████████╗██╗ ██████╗ ███╗   ██╗
// ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██║██╔═══██╗████╗  ██║
// ███████╗   ██║   ███████║   ██║   ██║██║   ██║██╔██╗ ██║
// ╚════██║   ██║   ██╔══██║   ██║   ██║██║   ██║██║╚██╗██║
// ███████║   ██║   ██║  ██║   ██║   ██║╚██████╔╝██║ ╚████║
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: station
//! The station only assembles (decisions §34 24.1): its sources are the catalog and a thin main
//! per binary. Code that does work lives in a crate, whose edges structure.rs checks.
//!
//! Owns:          the station's file list, the_station_only_assembles
//! Doesn't own:   the crate edges (structure.rs), what a binary may call (review)
//! Entry points:  the_station_only_assembles
//! Decisions:     §23 "Step 8", §34 24.1, 34f11

use crate::files::{rust_files, station_dir};

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// `src/catalog.rs` and `src/bin/<binary>.rs`, nothing else: a module in the station would be
/// library code only the station can reach (rustc's `main.rs` → `rustc_driver`, Bevy's
/// `DefaultPlugins`: one assembler, the work in crates).
#[test]
fn the_station_only_assembles() {
    let root = station_dir();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let extra: Vec<String> = files
        .iter()
        .map(|f| f.strip_prefix(&root).unwrap_or(f).to_string_lossy().replace('\\', "/"))
        .filter(|rel| rel != "catalog.rs" && !(rel.starts_with("bin/") && rel.matches('/').count() == 1))
        .collect();
    assert!(
        extra.is_empty(),
        "the station holds only catalog.rs and src/bin/*.rs (decisions §34 24.1) — move these into a crate:\n  {}",
        extra.join("\n  ")
    );
}
