//! Builds the demo show file `library/shows/demo.luma` from the sources in git through
//! `lumacore_objects::demo::build` (decisions §40 14–17; the file is gitignored, the catalog
//! test `the_demo_show_builds_loads_and_matches_its_golden` checks the build).
//!
//! Usage:
//!     cargo run -p lumacore-station --bin gen-demo              # always builds and writes
//!     cargo run -p lumacore-station --bin gen-demo -- --if-stale  # only when missing or stale

use lumacore_objects::demo;

fn main() {
    lumacore_station::install_catalog().expect("the catalog");
    let root = demo::repo_root();
    let if_stale = std::env::args().any(|a| a == "--if-stale");
    if if_stale && !demo::stale(&root) {
        println!("✓ {} is current", demo::demo_path(&root).display());
        return;
    }
    let built = demo::build(&root);
    for w in &built.warnings {
        eprintln!("  WARN: {w}");
    }
    let sd = &built.state.root.show_data;
    let patch = &sd.devices.dmx.patch;
    println!("  fixtures:  {} (+ {} folders)", patch.fids().len(), patch.fixtures.values().filter(|f| f.mvr.fid.is_none()).count());
    println!("  universes: {}", patch.universes.len());
    println!("  lines:     {}", sd.protocols.all_lines().count());
    println!("  users:     {}", sd.user_profiles.len());
    println!("  groups:    {}", sd.datapools.values().map(|d| d.groups.len()).sum::<usize>());
    if let Err(e) = demo::check(&built.state) {
        eprintln!("✗ {e}");
        std::process::exit(1);
    }
    if !built.warnings.is_empty() {
        eprintln!("✗ {} command line(s) complained", built.warnings.len());
        std::process::exit(1);
    }
    let path = demo::write(&root, &built).unwrap_or_else(|e| {
        eprintln!("✗ {e}");
        std::process::exit(1)
    });
    println!("✓ demo.luma written to {}", path.display());
}
