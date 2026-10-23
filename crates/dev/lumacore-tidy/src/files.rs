// ███████╗██╗██╗     ███████╗███████╗
// ██╔════╝██║██║     ██╔════╝██╔════╝
// █████╗  ██║██║     █████╗  ███████╗
// ██╔══╝  ██║██║     ██╔══╝  ╚════██║
// ██║     ██║███████╗███████╗███████║
// ╚═╝     ╚═╝╚══════╝╚══════╝╚══════╝
// FILE: files
//! The files the rules read (decisions §34 20): the workspace from its root, the station as the
//! rules name it — the state and objects crates' files are `core/…`, a crate cut out of the
//! station keeps its old folder name (`terminal/…`) — and the scanners every rule shares.
//!
//! Owns:          the paths, the station's names for files, the source scanners, the ratchet check
//! Doesn't own:   the rules themselves (one file per area)
//! Entry points:  workspace, GROUPS, crate_root, crate_roots, station_files, station_rel, sources, string_literals, ratchet
//! Decisions:     §34 18, 20, 23

use std::path::{Path, PathBuf};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Where crates live under `crates/` and what each folder is for (decisions §34 23): a group by
/// what its crates are about, a family (`network/protocols`) for siblings on one interface, sorted
/// by their row's kind. Every crate is `crates/<folder>/lumacore-<name>`; the station, the one
/// assembler, is `apps/lumacore-station` (§34 24).
pub(crate) const GROUPS: &[(&str, &str)] = &[
    ("foundation", "what everything stands on: values, the catalog, tracing, the wire, Lum, the state thread's lanes, the System Monitor"),
    ("model", "the show's data and its rules: the tree, its data, AppState, the objects (until §34 25 cuts them into commands/)"),
    ("formats", "files in and out: GDTF (later the .luma format, MVR)"),
    ("engine", "the threads that run against a deadline: the renderer, the engine registry"),
    ("network", "the station's network I/O and its protocol lines"),
    ("network/protocols", "the protocol family: the interface and its line runtime"),
    ("network/protocols/dmx", "protocols whose lines carry universes (Kind::Dmx)"),
    ("network/protocols/data", "protocols whose messages become command lines (Kind::Data)"),
    ("commands", "the command kernel, the suggestion engine and the commands of each domain (§34 25)"),
    ("commands/objects", "one crate per domain's commands: `lumacore-cmd-<domain>` (§34 25.6, 25.7)"),
    ("commands/objects/protocols/dmx", "one crate per DMX protocol's object, mirroring network/protocols/dmx (§34 \"34f7c hardened\" 9)"),
    ("commands/objects/protocols/data", "one crate per Data protocol's object, mirroring network/protocols/data (§34 \"34f7c hardened\" 9)"),
    ("ui", "what a screen shows and how it gets there: views, terminals, HTTP"),
    ("server", "one station, running: the server, LumaSession"),
    ("dev", "for developers, never in the product: tidy, the header tool, the sweep"),
];

/// Crates cut out of the station and the name their files keep for the rules (`terminal/…`).
pub(crate) const CUT_OUT: &[(&str, &str)] = &[
    ("lumacore-terminal", "terminal"),
    ("lumacore-lines", "protocols"),
    ("lumacore-osc", "protocols"),
    ("lumacore-engines", "engines"),
    ("lumacore-clocksync", "lumasession"),
    ("lumacore-lanes", "server"),
    ("lumacore-shell", "net"),
    ("lumacore-lumasession", "lumasession"),
    ("lumacore-server", "server"),
    ("lumacore-sweep", "sweep"),
    // The show file and the disk door were core/ (decisions §34 25.10).
    ("lumacore-luma", "core"),
    // The suggestion engine was core/suggest/ (decisions §34 25.9).
    ("lumacore-suggest", "core"),
    // The tree shell's words were core/commands/ (decisions §34 25.7).
    ("lumacore-cmd-treeshell", "core"),
    // The keys were core/keys.rs (decisions §34 25.4).
    ("lumacore-cmd-keys", "core"),
    // The show file was core/commands/backup.rs and core/show_import.rs (decisions §34 25.7).
    ("lumacore-cmd-showfile", "core"),
    // The LumaSession words were core/commands/session.rs and station_verbs.rs (§34 25.7).
    ("lumacore-cmd-lumasession", "core"),
    // The station's words were core/commands/ and core/objects/system.rs (§34 25.7).
    ("lumacore-cmd-station", "core"),
    // The devices were core/devices.rs (decisions §34 25.17).
    ("lumacore-cmd-devices", "core"),
    // The lines were core/objects/protocols.rs (decisions §34 25.17).
    ("lumacore-cmd-protocols", "core"),
];

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

// *************************************************************************************************
// Paths
// The workspace from its root, a crate's sources, the station's view.
// *************************************************************************************************

/// The workspace root (`LUMACORE_ROOT`, decisions §34 23.6).
pub(crate) fn workspace() -> PathBuf {
    Path::new(env!("LUMACORE_ROOT")).canonicalize().expect("workspace root")
}

/// A crate's folder, found in its group (`crate_root("lumacore-sacn")` =
/// `crates/network/protocols/dmx/lumacore-sacn`).
pub(crate) fn crate_root(name: &str) -> PathBuf {
    let crates = workspace().join("crates");
    GROUPS
        .iter()
        .map(|(group, _)| crates.join(group).join(name))
        .chain(std::iter::once(workspace().join("apps").join(name)))
        .find(|dir| dir.join("Cargo.toml").exists())
        .unwrap_or_else(|| panic!("crate {name}: not in any group of GROUPS nor in apps/ (decisions §34 23)"))
}

/// Every folder under `crates/` that holds a Cargo.toml (its build output and caches left out).
pub(crate) fn crate_roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut dirs = vec![workspace().join("crates")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let p = entry.path();
            if !p.is_dir() || p.ends_with("target") || p.ends_with("library") || p.ends_with("src") {
                continue;
            }
            if p.join("Cargo.toml").exists() {
                out.push(p);
            } else {
                dirs.push(p);
            }
        }
    }
    out.sort();
    out
}

/// A crate's sources (`crate_dir("lumacore-terminal")`).
pub(crate) fn crate_dir(name: &str) -> PathBuf {
    crate_root(name).join("src")
}

/// The station's own sources.
pub(crate) fn station_dir() -> PathBuf {
    crate_dir("lumacore-station")
}

/// The state crate's sources (decisions §34 7): its files were `core/…` of the station and the
/// rules read them under those names.
fn state_dir() -> PathBuf {
    crate_dir("lumacore-state")
}

/// The objects crate's sources (decisions §34 7, 16): the rest of the station's `core/`.
pub(crate) fn objects_dir() -> PathBuf {
    crate_dir("lumacore-objects")
}

/// The station's files under `sub` ("" = all, "core" = the core, "core/x" = a folder of it) and
/// the state and objects crates' (which count as `core/…`).
pub(crate) fn station_files(sub: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Some(inner) = sub.strip_prefix("core/") {
        for dir in [objects_dir(), state_dir()] {
            if dir.join(inner).is_dir() {
                rust_files(&dir.join(inner), &mut files);
            }
        }
        return files;
    }
    let own = if sub.is_empty() { station_dir() } else { station_dir().join(sub) };
    if own.is_dir() {
        rust_files(&own, &mut files);
    }
    if sub.is_empty() || sub == "core" {
        rust_files(&state_dir(), &mut files);
        rust_files(&objects_dir(), &mut files);
    }
    for (krate, name) in CUT_OUT {
        if sub.is_empty() || sub == *name {
            rust_files(&crate_dir(krate), &mut files);
        }
    }
    files
}

/// A file's name as the rules know it: `core/drive.rs` for the state crate's `drive.rs`, `core/lib.rs`
/// for the objects crate's root.
pub(crate) fn station_rel(file: &Path) -> String {
    let rel = match file.strip_prefix(state_dir()).or_else(|_| file.strip_prefix(objects_dir())) {
        Ok(r) => Path::new("core").join(r),
        Err(_) => CUT_OUT
            .iter()
            .find_map(|(krate, name)| file.strip_prefix(crate_dir(krate)).ok().map(|r| Path::new(name).join(r)))
            .unwrap_or_else(|| file.strip_prefix(station_dir()).unwrap_or(file).to_path_buf()),
    };
    rel.to_string_lossy().replace('\\', "/")
}

/// A test file: `foo_tests.rs` (the old way) or `foo/tests.rs` (decisions §34 11).
pub(crate) fn is_test_file(rel: &str) -> bool {
    rel.ends_with("_tests.rs") || rel == "tests.rs" || rel.ends_with("/tests.rs")
}

/// Every `.rs` file under `dir` (build output and `node_modules` skipped); `dir` must exist.
pub(crate) fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())).flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if !matches!(name, "target" | "node_modules" | ".git") {
                rust_files(&path, out);
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
}

/// Every `.rs` file of the workspace, as (path relative to the root, text).
pub(crate) fn sources() -> Vec<(String, String)> {
    let root = workspace();
    let mut files = Vec::new();
    for top in ["crates", "apps", "tests/systemtest", "tests/catalog"] {
        rust_files(&root.join(top), &mut files);
    }
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let rel = p.strip_prefix(&root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            (rel, text)
        })
        .collect()
}

// *************************************************************************************************
// Scanners
// What a source says, its tests left out.
// *************************************************************************************************

/// The source before its test module (tests may spell words freely). A `#[cfg(test)]` that
/// only declares a test file (`mod x_tests;`, the file is checked as itself) doesn't end it.
pub(crate) fn non_test_source(text: &str) -> &str {
    let mut from = 0;
    while let Some(at) = text[from..].find("#[cfg(test)]").map(|i| from + i) {
        let after = &text[at + "#[cfg(test)]".len()..];
        let item = after.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with("#[")).unwrap_or("");
        if !(item.starts_with("mod ") && item.ends_with(';')) {
            return &text[..at];
        }
        from = at + 1;
    }
    text
}

/// String literals with the byte offset they start at (comments skipped,
/// escapes honoured, raw strings handled).
pub(crate) fn string_literals(src: &str) -> Vec<(usize, String)> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'\'' => {
                // char literal ('"', '\'') — skip it whole; lifetimes have no closing quote nearby
                if bytes.get(i + 2) == Some(&b'\'') {
                    i += 3;
                } else if bytes.get(i + 1) == Some(&b'\\') && bytes.get(i + 3) == Some(&b'\'') {
                    i += 4;
                } else {
                    i += 1;
                }
            }
            b'r' if bytes.get(i + 1) == Some(&b'#') || bytes.get(i + 1) == Some(&b'"') => {
                let mut j = i + 1;
                let mut hashes = 0;
                while bytes.get(j) == Some(&b'#') {
                    hashes += 1;
                    j += 1;
                }
                if bytes.get(j) != Some(&b'"') {
                    i += 1;
                    continue;
                }
                let close: String = std::iter::once('"')
                    .chain("#".repeat(hashes).chars())
                    .collect();
                let start = j + 1;
                let end = src[start..].find(&close).map_or(src.len(), |e| start + e);
                out.push((i, src[start..end].to_string()));
                i = end + close.len();
            }
            b'"' => {
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && bytes[j] != b'"' {
                    j += if bytes[j] == b'\\' { 2 } else { 1 };
                }
                out.push((i, src[start..j.min(src.len())].to_string()));
                i = j + 1;
            }
            _ => i += 1,
        }
    }
    out
}

pub(crate) fn line_of(src: &str, pos: usize) -> usize {
    src[..pos].matches('\n').count() + 1
}

// *************************************************************************************************
// Ratchets
// A count that only goes down.
// *************************************************************************************************

/// A ratchet: `now` may not pass `at_most`, and when it drops the constant is lowered with it.
pub(crate) fn ratchet(what: &str, now: usize, at_most: usize, listing: &str) {
    assert!(now <= at_most, "{now} {what}, more than {at_most} (decisions §34):\n{listing}");
    assert!(now == at_most, "{now} {what} now — lower the constant to {now} (decisions §34)");
}
