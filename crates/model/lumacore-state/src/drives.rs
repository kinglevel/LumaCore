// ██████╗ ██████╗ ██╗██╗   ██╗███████╗███████╗
// ██╔══██╗██╔══██╗██║██║   ██║██╔════╝██╔════╝
// ██║  ██║██████╔╝██║██║   ██║█████╗  ███████╗
// ██║  ██║██╔══██╗██║╚██╗ ██╔╝██╔══╝  ╚════██║
// ██████╔╝██║  ██║██║ ╚████╔╝ ███████╗███████║
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝╚══════╝
// FILE: drives

//! The station's drives on the state (decisions §38): the live list on `AppState::drives`, the
//! table in `StationSettings::drives` (the settings file), the mount map the tree shell reads,
//! and where a file word's `/Drive` leads. The rules are `lumacore-drives`' (`table::reconcile`,
//! `ids`, `library`); this is the glue on the state thread.
//!
//! Owns:          reconcile / refresh on the state, the mount map, AppState::drive / drive_library / ensure_library.
//! Doesn't own:   the probe, the table rule, the library tree (lumacore-drives), the watcher (the server), the Drive object (cmd-station).
//! Entry points:  reconcile, refresh, mount_of, mounted_ids, next_id, AppState::drive, drive_library, ensure_library.
//! Decisions:     §38.

use std::collections::BTreeMap;
use std::path::PathBuf;

pub use lumacore_drives::library::LIBRARY_DIR;
pub use lumacore_drives::table::{Drive, LOCAL, gb};
use lumacore_drives::probe::{Volume, VolumeKind};
/// For tests that mount a folder as a drive (the tree shell's drive tests).
pub use lumacore_drives::probe::{Volume as DriveVolume, VolumeKind as DriveVolumeKind};
use lumacore_tree::{DriveKind, DriveMemory};

use crate::AppState;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

thread_local! {
    /// id (lowercase) → the volume root of every present drive: the tree shell's `Root.Drives.<id>`.
    static MOUNTS: std::cell::RefCell<BTreeMap<String, PathBuf>> = const { std::cell::RefCell::new(BTreeMap::new()) };
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The next free id of a kind among the remembered ones.
pub fn next_id(kind: DriveKind, taken: &BTreeMap<String, DriveMemory>) -> String {
    lumacore_drives::ids::next_id(kind, |id| taken.keys().any(|k| k.eq_ignore_ascii_case(id)))
}

/// The volumes probed now (and the local library's free / total bytes) → the live list, the
/// table (saved when it changed), the mount map; what changed, one line each.
pub fn reconcile(state: &mut AppState, volumes: &[Volume], local: (u64, u64)) -> Vec<String> {
    let r = lumacore_drives::table::reconcile(&mut state.root.station_settings.drives, &state.drives, volumes, local, lumacore_tree::now_ms());
    if r.table_changed {
        state.local_settings.store_from(&state.root.station_settings);
    }
    MOUNTS.with(|m| {
        *m.borrow_mut() = r.drives.iter().filter(|d| d.present).map(|d| (d.id.to_lowercase(), d.mount.clone())).collect();
    });
    state.drives = r.drives;
    r.lines
}

/// After a setting changed the table (a label, a kind, an added or forgotten drive): the live
/// list again from the volumes present now, without a new probe.
pub fn refresh(state: &mut AppState) {
    let table = &state.root.station_settings.drives;
    let volumes: Vec<Volume> = state
        .drives
        .iter()
        .filter(|d| d.present && d.kind != DriveKind::LocalLibrary)
        .filter_map(|d| {
            let key = table.values().find(|m| m.path.is_none() && !m.key.is_empty() && m.label == d.label && m.kind == d.kind).map(|m| m.key.clone())?;
            Some(Volume { key, label: d.label.clone(), mount: d.mount.clone(), kind: VolumeKind::Removable, free: d.free, total: d.total })
        })
        .collect();
    let local = state.drives.first().map_or((0, 0), |d| (d.free, d.total));
    reconcile(state, &volumes, local);
}

/// The volume root of a present drive (the tree shell's `Root.Drives.<id>`).
pub fn mount_of(id: &str) -> Option<PathBuf> {
    MOUNTS.with(|m| m.borrow().get(&id.to_lowercase()).cloned())
}

/// Every present drive's id, in order (the tree shell's `ls Root.Drives`).
pub fn mounted_ids() -> Vec<String> {
    MOUNTS.with(|m| m.borrow().keys().cloned().collect())
}

impl AppState {
    /// A drive by id (any case, quotes allowed).
    pub fn drive(&self, id: &str) -> Option<&Drive> {
        let id = id.trim_matches('"');
        self.drives.iter().find(|d| d.id.eq_ignore_ascii_case(id))
    }

    /// Where a file word reads or writes: the drive's library (`/Drive "usb1"`), or the local
    /// library when none is named. Refused when the drive is unknown or out.
    pub fn drive_library(&self, id: Option<&str>) -> Result<PathBuf, String> {
        let Some(id) = id else { return Ok(PathBuf::from("library")) };
        let d = self.drive(id).ok_or_else(|| crate::errors::render_code(&format!("No drive \"{}\" (List Drive)", id.trim_matches('"')), "E-CMD-007"))?;
        if !d.present {
            return Err(crate::errors::render_code(&format!("Drive \"{}\" ({}) is out", d.id, d.label), "E-CMD-012"));
        }
        Ok(d.library())
    }

    /// A library folder a file word reads (`write` = false) or writes: `library/<folder>`, or the
    /// folder on the drive `/Drive` names (a write makes the drive's tree first).
    pub fn library_folder(&self, drive: Option<&str>, folder: &str, write: bool) -> Result<String, String> {
        let lib = match drive {
            Some(d) if write => self.ensure_library(d)?,
            other => self.drive_library(other)?,
        };
        Ok(lib.join(folder).to_string_lossy().into_owned())
    }

    /// The first write to a drive makes its library tree (never a read, never an ask).
    pub fn ensure_library(&self, id: &str) -> Result<PathBuf, String> {
        let d = self.drive(id).ok_or_else(|| crate::errors::render_code(&format!("No drive \"{}\" (List Drive)", id.trim_matches('"')), "E-CMD-007"))?;
        if !d.present {
            return Err(crate::errors::render_code(&format!("Drive \"{}\" ({}) is out", d.id, d.label), "E-CMD-012"));
        }
        if d.kind != DriveKind::LocalLibrary && !d.has_library() {
            lumacore_drives::library::init_library(&d.mount)
                .map_err(|e| crate::errors::render_code(&format!("Cannot make the library on {} ({}): {e}", d.id, d.mount.display()), "E-CMD-012"))?;
            lumacore_sysmon::sysmon_note("Drives", format!("{} \"{}\" library created", d.id, d.label));
        }
        Ok(d.library())
    }
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

    fn vol(key: &str, label: &str) -> Volume {
        Volume { key: key.into(), label: label.into(), mount: PathBuf::from(format!("/tmp/{label}")), kind: VolumeKind::Removable, free: 1, total: 2 }
    }

    #[test]
    fn a_file_word_finds_a_drive_or_the_local_library() {
        let mut s = AppState::new();
        reconcile(&mut s, &[vol("uuid:1", "KINGSTON"), vol("uuid:2", "SanDisk")], (0, 0));
        reconcile(&mut s, &[vol("uuid:2", "SanDisk")], (0, 0));
        assert!(s.drive_library(Some("\"usb1\"")).unwrap_err().contains("is out"));
        assert!(s.drive_library(Some("usb9")).unwrap_err().contains("No drive"));
        assert_eq!(s.drive_library(None).unwrap(), PathBuf::from("library"));
        assert!(s.drive_library(Some("USB2")).unwrap().ends_with("LumaCore/library"));
        assert_eq!(mounted_ids(), [LOCAL, "usb2"]);
        assert_eq!(mount_of("Usb2"), Some(PathBuf::from("/tmp/SanDisk")));
    }
}
