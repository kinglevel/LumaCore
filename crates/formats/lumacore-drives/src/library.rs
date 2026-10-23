// ██╗     ██╗██████╗ ██████╗  █████╗ ██████╗ ██╗   ██╗
// ██║     ██║██╔══██╗██╔══██╗██╔══██╗██╔══██╗╚██╗ ██╔╝
// ██║     ██║██████╔╝██████╔╝███████║██████╔╝ ╚████╔╝
// ██║     ██║██╔══██╗██╔══██╗██╔══██║██╔══██╗  ╚██╔╝
// ███████╗██║██████╔╝██║  ██║██║  ██║██║  ██║   ██║
// ╚══════╝╚═╝╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝
// FILE: library

//! The library tree on a drive (decisions §38): `LumaCore/library/` below the volume's root, the
//! station library's folders, created on the first write — never by a read, never by an ask.
//!
//! Owns:          where a drive's library is, whether it exists, making it.
//! Doesn't own:   the volumes (probe.rs), which drive a line means (lumacore-state).
//! Entry points:  library_root, has_library, init_library, LIBRARY_DIR, LIBRARY_FOLDERS.
//! Decisions:     §38.

use std::path::{Path, PathBuf};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The library tree on a drive, below its root.
pub const LIBRARY_DIR: &str = "LumaCore/library";

/// The folders of a drive's library — the station library's, minus its caches and logs.
pub const LIBRARY_FOLDERS: &[&str] = &["shows", "gdtf", "mvr", "media", "userprofiles", "datapools", "macro", "presets"];

const README: &str = "LumaCore library on this drive\n\
==============================\n\n\
Files put in these folders are found by every LumaCore station this drive is in:\n\n\
  shows/         show files (.luma) — SaveShow \"x\" /Drive \"<id>\"\n\
  gdtf/          GDTF fixture types to pick (Insert fixtures)\n\
  mvr/           MVR scene files (Import Mvr)\n\
  media/         pictures to import\n\
  userprofiles/  profile files (.lumaprofile)\n\
  datapools/ macro/ presets/   exports\n\n\
The station made this tree on its first write to the drive (decisions §38).\n";

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Where a drive's library is: `<mount>/LumaCore/library`.
pub fn library_root(mount: &Path) -> PathBuf {
    mount.join(LIBRARY_DIR)
}

/// Whether the library tree exists on the drive (a read never creates it).
pub fn has_library(mount: &Path) -> bool {
    library_root(mount).is_dir()
}

/// Create the library tree on the drive: its folders and a README (the first write does this).
pub fn init_library(mount: &Path) -> std::io::Result<()> {
    let root = library_root(mount);
    for folder in LIBRARY_FOLDERS {
        lumacore_luma::drive::create_dir_all(root.join(folder))?;
    }
    let readme = root.join("README");
    if !readme.exists() {
        lumacore_luma::drive::write(&readme, README)?;
    }
    Ok(())
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_library_tree_is_made_once_with_its_folders_and_readme() {
        let dir = std::env::temp_dir().join(format!("lumacore-drives-lib-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!has_library(&dir));
        init_library(&dir).unwrap();
        assert!(has_library(&dir));
        for f in LIBRARY_FOLDERS {
            assert!(library_root(&dir).join(f).is_dir(), "{f}");
        }
        assert!(library_root(&dir).join("README").is_file());
        init_library(&dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
