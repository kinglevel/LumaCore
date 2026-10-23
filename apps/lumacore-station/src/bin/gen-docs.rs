//! Writes the command and object reference (`docs/user/reference/`) from the
//! registries (decisions §29). The test `the_reference_is_current` (`tests/catalog`)
//! fails until it is run after a registry change.
//!
//! Usage:
//!     cargo run -p lumacore-station --bin gen-docs

use lumacore_objects::{demo, reference};

fn main() {
    lumacore_station::install_catalog().expect("the catalog");
    let dir = reference::reference_dir(&demo::repo_root());
    std::fs::create_dir_all(&dir).expect("create docs/user/reference");
    for (file, text) in [("commands.md", reference::commands()), ("objects.md", reference::objects())] {
        let path = dir.join(file);
        std::fs::write(&path, &text).expect("write the reference");
        println!("✓ {} ({} lines)", path.display(), text.lines().count());
    }
}
