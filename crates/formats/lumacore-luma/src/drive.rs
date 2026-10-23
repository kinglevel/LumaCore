// ██████╗ ██████╗ ██╗██╗   ██╗███████╗
// ██╔══██╗██╔══██╗██║██║   ██║██╔════╝
// ██║  ██║██████╔╝██║██║   ██║█████╗
// ██║  ██║██╔══██╗██║╚██╗ ██╔╝██╔══╝
// ██████╔╝██║  ██║██║ ╚████╔╝ ███████╗
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝
// FILE: drive

//! The one door from `core/` to the disk (decisions §25 "Step 10
//! hardening"). Every file read or write of a command goes through here, so
//! the nightly command sweep can shut the door: with [`set_off`] on this
//! thread, each call fails with `PermissionDenied` and is remembered
//! ([`take_refused`]) — a `Reach::Show` line that knocks is a finding.
//! In the sweep's **sandbox** ([`set_sandbox`], decisions §29) the door is
//! open for `Reach::Drive` lines, but only to relative paths without `..`
//! (the sweep runs in a throwaway copy of `library/`) and absolute paths
//! inside the sandbox; anything else is refused and remembered.
//! tidy `core_reaches_the_disk_only_through_drive` keeps it the only door.
//!
//! Owns:          the one disk door: every read and write of a command, the sweep's shut door and sandbox, library_file.
//! Doesn't own:   what is read or written (its callers), the background worker (background.rs).
//! Entry points:  read, write, write_atomic, library_file, set_off, set_sandbox, enter, door.
//! Decisions:     §25 "Step 10 hardening", §29, §34 25.10.

use std::cell::{Cell, RefCell};
use std::fs::{File, Metadata, OpenOptions, ReadDir};
use std::io;
use std::path::{Component, Path, PathBuf};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

thread_local! {
    static OFF: Cell<bool> = const { Cell::new(false) };
    static SANDBOX: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static REFUSED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Keep this thread inside `root` (an absolute directory; the process
/// works in it), or let it out again with None.
pub fn set_sandbox(root: Option<PathBuf>) {
    SANDBOX.with(|s| *s.borrow_mut() = root);
}

fn inside(root: &Path, path: &Path) -> bool {
    !path.components().any(|c| matches!(c, Component::ParentDir))
        && (!path.is_absolute() || path.starts_with(root))
}

/// This thread's door (off, sandbox), to carry into a worker thread a
/// command starts (`crate::background`): the worker meets the same door.
#[derive(Clone, Default)]
pub struct Door {
    off: bool,
    sandbox: Option<PathBuf>,
}

/// The current thread's door.
pub fn door() -> Door {
    Door {
        off: OFF.with(Cell::get),
        sandbox: SANDBOX.with(|s| s.borrow().clone()),
    }
}

/// Enter `door` on this thread.
pub fn enter(door: &Door) {
    OFF.with(|o| o.set(door.off));
    SANDBOX.with(|s| *s.borrow_mut() = door.sandbox.clone());
}

/// Close (or open) the disk for this thread.
pub fn set_off(off: bool) {
    OFF.with(|o| o.set(off));
}

/// What was refused on this thread since the last call (`"write library/x"`).
pub fn take_refused() -> Vec<String> {
    REFUSED.with(|r| std::mem::take(&mut *r.borrow_mut()))
}

fn gate(what: &str, path: &Path) -> io::Result<()> {
    let off = OFF.with(Cell::get);
    let outside = !off && SANDBOX.with(|s| s.borrow().as_ref().is_some_and(|root| !inside(root, path)));
    if !off && !outside {
        return Ok(());
    }
    let note = if outside { " (outside the sandbox)" } else { "" };
    REFUSED.with(|r| r.borrow_mut().push(format!("{what} {}{note}", path.display())));
    Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        if outside {
            "outside the command sweep's sandbox"
        } else {
            "drive access is off (command sweep)"
        },
    ))
}

/// Whether the door lets a write of `path` through, without writing (unit
/// tests that must leave no file behind still meet a closed door).
pub fn may_write(path: impl AsRef<Path>) -> io::Result<()> {
    gate("write", path.as_ref())
}

pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    gate("read", path.as_ref())?;
    std::fs::read_to_string(path)
}

pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    gate("read", path.as_ref())?;
    std::fs::read(path)
}

/// Files written, created, removed or renamed through here (a counter the
/// Backup feed follows: decisions §30 "12d", no polling of the disk).
static WRITES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn wrote() {
    WRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// How many writes went through so far (appends to logs don't count).
pub fn writes() -> u64 {
    WRITES.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    gate("write", path.as_ref())?;
    wrote();
    std::fs::write(path, contents)
}

/// Write `contents` so that `path` holds either the old file or the whole
/// new one, never a torn mix (R1-14/15): `<path>.tmp` in the same folder,
/// `sync_all`, rename over `path`, then (unix) fsync the folder so the
/// rename itself survives a power cut.
pub fn write_atomic(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    write_atomic_with(path, contents, |_| {})
}

/// [`write_atomic`], calling `on_chunk(written)` after every MiB (progress).
pub fn write_atomic_with(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
    mut on_chunk: impl FnMut(usize),
) -> io::Result<()> {
    use std::io::Write;
    let path = path.as_ref();
    gate("write", path)?;
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    gate("write", &tmp)?;
    let result = (|| {
        let mut file = File::create(&tmp)?;
        let mut written = 0;
        for chunk in contents.as_ref().chunks(1 << 20) {
            file.write_all(chunk)?;
            written += chunk.len();
            on_chunk(written);
        }
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
        return result;
    }
    #[cfg(unix)]
    {
        // Best effort: some file systems refuse to sync a folder; the file
        // itself is already whole on disk.
        let dir = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        };
        if let Ok(dir) = File::open(dir) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

/// Whether `name` can be a file name on every system a show travels to
/// (Linux, macOS, Windows): no `<>:"/\|?*`, no control character, no leading
/// dot, no trailing dot or space, not a Windows device name (`CON`, `NUL`,
/// `COM1` …, with or without an extension). The reason when it can't.
pub fn check_file_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("A file name can't be empty".into());
    }
    if let Some(c) = name
        .chars()
        .find(|c| c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
    {
        let shown = if c.is_control() { format!("U+{:04X}", c as u32) } else { c.to_string() };
        return Err(format!("A file name can't contain {shown}"));
    }
    if name.starts_with('.') {
        return Err("A file name can't start with a dot".into());
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Err("A file name can't end with a dot or a space".into());
    }
    let stem = name.split('.').next().unwrap_or(name).trim_end().to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && matches!(stem.as_bytes()[3], b'1'..=b'9'));
    if reserved {
        return Err(format!("\"{name}\" is a reserved device name on Windows"));
    }
    Ok(())
}

/// A file a command line names (issue 07, a common console library convention): `typed` is
/// relative to `folder` (one of the `library/…` folders), sub-folders allowed
/// (`"test/hej.json"` → `<folder>/test/hej.json`); an absolute path, a drive
/// letter or `..` is refused, and every part must be a valid file name. The
/// quotes are taken off; a bare `/word` is an option, never a file. Other
/// places on the disk are reached through the tree shell (Drives).
pub fn library_file(folder: &str, typed: &str) -> Result<PathBuf, String> {
    if typed.starts_with('/') {
        return Err(format!("'{typed}' is an option, not a file name — put a file name in quotes"));
    }
    let name = typed.strip_prefix('"').and_then(|t| t.strip_suffix('"')).unwrap_or(typed);
    if name.is_empty() {
        return Err("Which file?".into());
    }
    if name.starts_with(['/', '\\']) || name.contains(':') {
        return Err(format!("'{name}': a file name is relative to {folder}/ — no absolute path or drive"));
    }
    let mut path = PathBuf::from(folder);
    for part in name.split(['/', '\\']).filter(|p| !p.is_empty()) {
        if part == ".." || part == "." {
            return Err(format!("'{name}': a file stays inside {folder}/ (no ..)"));
        }
        check_file_name(part).map_err(|e| format!("'{name}': {e}"))?;
        path.push(part);
    }
    Ok(path)
}

pub fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    gate("mkdir", path.as_ref())?;
    std::fs::create_dir_all(path)
}

pub fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
    gate("list", path.as_ref())?;
    std::fs::read_dir(path)
}

pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    gate("remove", path.as_ref())?;
    wrote();
    std::fs::remove_file(path)
}

/// Copy a file (the OS streams it; nothing is read into memory).
pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64> {
    gate("read", from.as_ref())?;
    gate("write", to.as_ref())?;
    std::fs::copy(from, to)
}

pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    gate("rename", from.as_ref())?;
    gate("rename to", to.as_ref())?;
    wrote();
    std::fs::rename(from, to)
}

pub fn metadata(path: impl AsRef<Path>) -> io::Result<Metadata> {
    gate("stat", path.as_ref())?;
    std::fs::metadata(path)
}

pub fn create(path: impl AsRef<Path>) -> io::Result<File> {
    gate("create", path.as_ref())?;
    wrote();
    File::create(path)
}

pub fn open(path: impl AsRef<Path>) -> io::Result<File> {
    gate("open", path.as_ref())?;
    File::open(path)
}

/// Free bytes on the volume holding `path` (`df -Pk`; None when it can't
/// be told). The Backup menu warns below 15 GB (decisions §30 "12d").
pub fn free_bytes(path: &Path) -> Option<u64> {
    gate("stat", path).ok()?;
    let out = std::process::Command::new("df").arg("-Pk").arg(path).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().nth(1)?;
    let kb: u64 = line.split_whitespace().nth(3)?.parse().ok()?;
    Some(kb * 1024)
}

/// Open for appending, creating the file when missing.
pub fn append(path: impl AsRef<Path>) -> io::Result<File> {
    gate("append", path.as_ref())?;
    OpenOptions::new().create(true).append(true).open(path)
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
