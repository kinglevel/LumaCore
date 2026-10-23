// ██████╗ ██████╗  ██████╗ ██████╗ ███████╗
// ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
// ██████╔╝██████╔╝██║   ██║██████╔╝█████╗
// ██╔═══╝ ██╔══██╗██║   ██║██╔══██╗██╔══╝
// ██║     ██║  ██║╚██████╔╝██████╔╝███████╗
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚══════╝
// FILE: probe

//! The volumes the OS has mounted (decisions §38): Linux reads `/proc/mounts` (volumes under
//! `/media/`, `/run/media/`, `/mnt/`; the device's `removable` flag in `/sys/block`; its UUID from
//! `/dev/disk/by-uuid`), macOS `/Volumes` (the boot volume left out), Windows the drive letters.
//! `LUMACORE_DRIVES_PROBE=<dir>` makes every folder in `<dir>` a volume instead (tests: a "stick"
//! comes and goes as a folder). Free space comes from one `df -kP` a tick.
//!
//! Owns:          the probe: mounted volumes, their keys, labels, kinds and space.
//! Doesn't own:   the ids (ids.rs), the library tree (library.rs), the watcher thread (the server).
//! Entry points:  probe, spaces, space_in, space, Volume, VolumeKind.
//! Decisions:     §38.

use std::path::{Path, PathBuf};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What the OS says a volume is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeKind {
    /// A fixed disk beyond the system one.
    Fixed,
    /// Removable: a stick, a disk in a case, a card.
    Removable,
}

/// A mounted volume as probed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    /// What makes it the same volume next time: its UUID, serial, or the label when the OS gives
    /// nothing better (`uuid:…`, `dev:…`, `label:…`, `probe:…`).
    pub key: String,
    /// The volume's label (KINGSTON), shown beside the id.
    pub label: String,
    pub mount: PathBuf,
    pub kind: VolumeKind,
    pub free: u64,
    pub total: u64,
}

/// Filesystems a volume may have (never a network share: those are LumaCore's own drives).
const LOCAL_FS: &[&str] = &[
    "vfat", "exfat", "ntfs", "ntfs3", "fuseblk", "ext2", "ext3", "ext4", "xfs", "btrfs", "hfsplus", "apfs", "f2fs", "udf", "iso9660",
];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The volumes the OS has mounted now (or the probe folder's, see the header).
pub fn probe() -> Vec<Volume> {
    if let Ok(dir) = std::env::var("LUMACORE_DRIVES_PROBE") {
        return probe_folder(Path::new(&dir));
    }
    probe_os()
}

/// `LUMACORE_DRIVES_PROBE`: every folder in `dir` is a removable volume.
fn probe_folder(dir: &Path) -> Vec<Volume> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let spaces = spaces();
    let mut out: Vec<Volume> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let (free, total) = space_in(&spaces, &e.path());
            Volume { key: format!("probe:{name}"), label: name, mount: e.path(), kind: VolumeKind::Removable, free, total }
        })
        .collect();
    out.sort_by(|a, b| a.label.cmp(&b.label));
    out
}

#[cfg(target_os = "linux")]
fn probe_os() -> Vec<Volume> {
    let Ok(mounts) = std::fs::read_to_string("/proc/mounts") else { return Vec::new() };
    let spaces = spaces();
    let mut out = Vec::new();
    for line in mounts.lines() {
        let mut f = line.split_whitespace();
        let (Some(dev), Some(mount), Some(fs)) = (f.next(), f.next(), f.next()) else { continue };
        if !LOCAL_FS.contains(&fs) {
            continue;
        }
        let mount = unescape(mount);
        if !(mount.starts_with("/media/") || mount.starts_with("/run/media/") || mount.starts_with("/mnt/")) {
            continue;
        }
        let path = PathBuf::from(&mount);
        let label = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| mount.clone());
        let device = dev.rsplit('/').next().unwrap_or(dev).to_string();
        let kind = if removable(&device) { VolumeKind::Removable } else { VolumeKind::Fixed };
        let key = uuid_of(dev).map_or_else(|| format!("dev:{device}"), |u| format!("uuid:{u}"));
        let (free, total) = space_in(&spaces, &path);
        out.push(Volume { key, label, mount: path, kind, free, total });
    }
    out
}

#[cfg(target_os = "linux")]
fn unescape(s: &str) -> String {
    s.replace("\\040", " ").replace("\\011", "\t").replace("\\012", "\n").replace("\\134", "\\")
}

/// `/sys/block/<disk>/removable` for the disk a partition is on (`sdb1` → `sdb`, `nvme0n1p1` →
/// `nvme0n1`, `mmcblk0p1` → `mmcblk0`).
#[cfg(target_os = "linux")]
fn removable(device: &str) -> bool {
    let disk = if device.starts_with("nvme") || device.starts_with("mmcblk") {
        device.rsplit_once('p').map_or(device, |(d, _)| d)
    } else {
        device.trim_end_matches(|c: char| c.is_ascii_digit())
    };
    std::fs::read_to_string(format!("/sys/block/{disk}/removable")).is_ok_and(|s| s.trim() == "1")
}

/// The UUID whose `/dev/disk/by-uuid` link points at `dev`.
#[cfg(target_os = "linux")]
fn uuid_of(dev: &str) -> Option<String> {
    let want = std::fs::canonicalize(dev).ok()?;
    for e in std::fs::read_dir("/dev/disk/by-uuid").ok()?.flatten() {
        if std::fs::canonicalize(e.path()).ok().as_deref() == Some(want.as_path()) {
            return Some(e.file_name().to_string_lossy().to_string());
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn probe_os() -> Vec<Volume> {
    let Ok(entries) = std::fs::read_dir("/Volumes") else { return Vec::new() };
    let root = std::fs::canonicalize("/").ok();
    let spaces = spaces();
    let mut out = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        // The boot volume is linked here too.
        if std::fs::canonicalize(&path).ok() == root || !path.is_dir() {
            continue;
        }
        let label = e.file_name().to_string_lossy().to_string();
        let (free, total) = space_in(&spaces, &path);
        // macOS gives no cheap stable id: the label is the key.
        out.push(Volume { key: format!("label:{label}"), label, mount: path, kind: VolumeKind::Removable, free, total });
    }
    out
}

#[cfg(target_os = "windows")]
fn probe_os() -> Vec<Volume> {
    // Drive letters beyond the system drive; kind and space are read once the Win32 calls land.
    (b'D'..=b'Z')
        .filter_map(|c| {
            let letter = c as char;
            let path = PathBuf::from(format!("{letter}:\\"));
            path.is_dir().then(|| Volume {
                key: format!("letter:{letter}"),
                label: format!("{letter}:"),
                mount: path,
                kind: VolumeKind::Removable,
                free: 0,
                total: 0,
            })
        })
        .collect()
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn probe_os() -> Vec<Volume> {
    Vec::new()
}

/// Free and total bytes per mounted filesystem, from one `df -kP` (unix; the watcher reads it
/// once a tick). Windows gives (0, 0) until its call lands.
#[cfg(unix)]
pub fn spaces() -> Vec<(PathBuf, u64, u64)> {
    let Ok(out) = std::process::Command::new("df").args(["-kP"]).output() else { return Vec::new() };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .skip(1)
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            if f.len() < 6 {
                return None;
            }
            let total = f[1].parse::<u64>().ok()? * 1024;
            let avail = f[3].parse::<u64>().ok()? * 1024;
            Some((PathBuf::from(f[5..].join(" ")), avail, total))
        })
        .collect()
}

#[cfg(not(unix))]
pub fn spaces() -> Vec<(PathBuf, u64, u64)> {
    Vec::new()
}

/// The space of the filesystem holding `path`: the longest mount that is a prefix of it.
pub fn space_in(spaces: &[(PathBuf, u64, u64)], path: &Path) -> (u64, u64) {
    let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    spaces
        .iter()
        .filter(|(m, _, _)| path.starts_with(m))
        .max_by_key(|(m, _, _)| m.as_os_str().len())
        .map_or((0, 0), |(_, a, t)| (*a, *t))
}

/// Free and total bytes of the filesystem holding `path` (0, 0 when it can't be read).
pub fn space(path: &Path) -> (u64, u64) {
    space_in(&spaces(), path)
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
    fn a_probe_folder_lists_its_folders_as_removable_volumes() {
        let dir = std::env::temp_dir().join(format!("lumacore-drives-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("KINGSTON")).unwrap();
        std::fs::create_dir_all(dir.join("SanDisk")).unwrap();
        std::fs::write(dir.join("a file"), "x").unwrap();
        let vols = probe_folder(&dir);
        assert_eq!(vols.iter().map(|v| v.label.as_str()).collect::<Vec<_>>(), ["KINGSTON", "SanDisk"]);
        assert!(vols.iter().all(|v| v.kind == VolumeKind::Removable && v.key.starts_with("probe:")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
