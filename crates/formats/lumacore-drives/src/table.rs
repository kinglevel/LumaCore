// ████████╗ █████╗ ██████╗ ██╗     ███████╗
// ╚══██╔══╝██╔══██╗██╔══██╗██║     ██╔════╝
//    ██║   ███████║██████╔╝██║     █████╗
//    ██║   ██╔══██║██╔══██╗██║     ██╔══╝
//    ██║   ██║  ██║██████╔╝███████╗███████╗
//    ╚═╝   ╚═╝  ╚═╝╚═════╝ ╚══════╝╚══════╝
// FILE: table

//! The drive list and the remembered table (decisions §38): the local library, every remembered
//! drive — present or Out —, the ids bound to the volumes the first time they are seen. Pure:
//! the station settings' table and the volumes come in, the live list and what changed come out;
//! the state thread keeps them (`lumacore_state::drives`).
//!
//! Owns:          Drive, reconcile (ids assigned, the table updated, the sysmon lines), gb.
//! Doesn't own:   the probe (probe.rs), the state and its settings file, the thread.
//! Entry points:  reconcile, Drive, Reconciled, LOCAL, gb.
//! Decisions:     §38.

use std::collections::BTreeMap;
use std::path::PathBuf;

use lumacore_tree::{DriveKind, DriveMemory};

use crate::ids::next_id;
use crate::probe::{Volume, VolumeKind};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The local library's id (Drive 1).
pub const LOCAL: &str = "local-library";

/// One drive as it is now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drive {
    pub id: String,
    pub kind: DriveKind,
    /// The volume's label, or the label given (`Set Drive "usb1" "Label" …`).
    pub label: String,
    /// The volume's root (the whole stick); the local library's is `library/` itself.
    pub mount: PathBuf,
    /// In (mounted, the folder reachable) or Out.
    pub present: bool,
    pub free: u64,
    pub total: u64,
    /// When it was last seen (unix ms); 0 = never.
    pub since_ms: u64,
}

/// What a reconcile gave.
pub struct Reconciled {
    /// The live list: the local library first, then every remembered drive.
    pub drives: Vec<Drive>,
    /// One line per change (in, out), for the System Monitor.
    pub lines: Vec<String>,
    /// The table changed (a new volume, a last-seen stamp): the settings file is to be written.
    pub table_changed: bool,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Drive {
    /// Where its library is: `LumaCore/library` on a volume; the local library itself.
    pub fn library(&self) -> PathBuf {
        if self.kind == DriveKind::LocalLibrary {
            self.mount.clone()
        } else {
            crate::library::library_root(&self.mount)
        }
    }

    /// Whether the library tree exists (a read never makes it).
    pub fn has_library(&self) -> bool {
        self.kind == DriveKind::LocalLibrary || (self.present && crate::library::has_library(&self.mount))
    }

    pub fn state_word(&self) -> &'static str {
        if self.present { "In" } else { "Out" }
    }
}

/// The volumes probed now (and the local library's free / total bytes) against the remembered
/// table and the list as it was: new volumes get the next free id of their kind and are
/// remembered; every remembered drive is listed, present or Out.
pub fn reconcile(table: &mut BTreeMap<String, DriveMemory>, before: &[Drive], volumes: &[Volume], local: (u64, u64), now_ms: u64) -> Reconciled {
    let was_present: BTreeMap<&str, bool> = before.iter().map(|d| (d.id.as_str(), d.present)).collect();
    let mut table_changed = false;
    for v in volumes {
        if !table.values().any(|m| m.key == v.key) {
            let kind = match v.kind {
                VolumeKind::Removable => DriveKind::Usb,
                VolumeKind::Fixed => DriveKind::Ssd,
            };
            let id = next_id(kind, |id| table.keys().any(|k| k.eq_ignore_ascii_case(id)));
            table.insert(id, DriveMemory { key: v.key.clone(), kind, label: v.label.clone(), path: None, last_seen: now_ms });
            table_changed = true;
        }
    }
    let mut drives = vec![Drive {
        id: LOCAL.to_string(),
        kind: DriveKind::LocalLibrary,
        label: "Internal".to_string(),
        mount: PathBuf::from("library"),
        present: true,
        free: local.0,
        total: local.1,
        since_ms: 0,
    }];
    for (id, m) in table.iter_mut() {
        let (mount, present, free, total) = match &m.path {
            // An added folder: In while its path is there.
            Some(p) => {
                let path = PathBuf::from(p);
                let present = path.is_dir();
                let (free, total) = if present { crate::probe::space(&path) } else { (0, 0) };
                (path, present, free, total)
            }
            None => match volumes.iter().find(|v| v.key == m.key) {
                Some(v) => (v.mount.clone(), true, v.free, v.total),
                None => (PathBuf::new(), false, 0, 0),
            },
        };
        if present && was_present.get(id.as_str()) != Some(&true) {
            m.last_seen = now_ms;
            table_changed = true;
        }
        drives.push(Drive { id: id.clone(), kind: m.kind, label: m.label.clone(), mount, present, free, total, since_ms: m.last_seen });
    }
    let mut lines = Vec::new();
    for d in drives.iter().skip(1) {
        match (was_present.get(d.id.as_str()), d.present) {
            (Some(true), true) | (Some(false), false) | (None, false) => {}
            (_, true) => lines.push(format!("{} \"{}\" in ({} free, {})", d.id, d.label, gb(d.free), if d.has_library() { "library ready" } else { "no library yet" })),
            (Some(true), false) => lines.push(format!("{} \"{}\" out", d.id, d.label)),
        }
    }
    Reconciled { drives, lines, table_changed }
}

/// "28 GB", "1.6 TB", "512 MB".
pub fn gb(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= 1e12 {
        format!("{:.1} TB", b / 1e12)
    } else if b >= 1e9 {
        format!("{:.0} GB", b / 1e9)
    } else {
        format!("{:.0} MB", b / 1e6)
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
    fn ids_are_kind_plus_ordinal_bound_to_the_volume() {
        let mut table = BTreeMap::new();
        let r = reconcile(&mut table, &[], &[vol("uuid:1", "KINGSTON"), vol("uuid:2", "SanDisk")], (0, 0), 10);
        assert_eq!(r.drives.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), [LOCAL, "usb1", "usb2"]);
        assert_eq!(r.lines.len(), 2);
        assert!(r.table_changed);
        // KINGSTON out, SanDisk in first tomorrow: SanDisk keeps usb2, usb1 is listed Out.
        let r2 = reconcile(&mut table, &r.drives, &[vol("uuid:2", "SanDisk")], (0, 0), 20);
        assert_eq!(r2.lines, ["usb1 \"KINGSTON\" out"]);
        let u1 = r2.drives.iter().find(|d| d.id == "usb1").unwrap();
        assert!(!u1.present && u1.label == "KINGSTON");
        assert!(r2.drives.iter().find(|d| d.id == "usb2").unwrap().present);
        assert!(!r2.table_changed);
        // Back in: the same id, a new last-seen.
        let r3 = reconcile(&mut table, &r2.drives, &[vol("uuid:1", "KINGSTON"), vol("uuid:2", "SanDisk")], (0, 0), 30);
        assert_eq!(r3.lines.len(), 1);
        assert_eq!(table["usb1"].last_seen, 30);
    }

    #[test]
    fn an_added_folder_is_in_while_its_path_exists() {
        let dir = std::env::temp_dir().join(format!("lumacore-drives-folder-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut table = BTreeMap::new();
        table.insert("folder1".to_string(), DriveMemory { key: String::new(), kind: DriveKind::Folder, label: "NAS".into(), path: Some(dir.to_string_lossy().to_string()), last_seen: 0 });
        assert!(!reconcile(&mut table, &[], &[], (0, 0), 1).drives[1].present);
        std::fs::create_dir_all(&dir).unwrap();
        let r = reconcile(&mut table, &[], &[], (0, 0), 2);
        assert!(r.drives[1].present && r.lines[0].starts_with("folder1 \"NAS\" in"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
