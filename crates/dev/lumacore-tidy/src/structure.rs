// ███████╗████████╗██████╗ ██╗   ██╗ ██████╗████████╗██╗   ██╗██████╗ ███████╗
// ██╔════╝╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██║   ██║██╔══██╗██╔════╝
// ███████╗   ██║   ██████╔╝██║   ██║██║        ██║   ██║   ██║██████╔╝█████╗
// ╚════██║   ██║   ██╔══██╗██║   ██║██║        ██║   ██║   ██║██╔══██╗██╔══╝
// ███████║   ██║   ██║  ██║╚██████╔╝╚██████╗   ██║   ╚██████╔╝██║  ██║███████╗
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: structure
//! Tidy rules for the shape of the code (decisions §34, docs/dev/architecture/code-structure.md):
//! the file headers (banner, module doc, Parts in order, Sections), file length, module and test
//! file conventions, the crate edges and names, and every crate's README. They read every `.rs`
//! file of the workspace (crates/, apps/, tests/systemtest/). Rules that today's code doesn't meet yet
//! are ratchets: a count that may only go down, and must be lowered when it does.
//!
//! Owns:          the structure rules and their ratchet lists
//! Doesn't own:   the banner font (lumacore-headers), the door rules (doors.rs)
//! Entry points:  the #[test] rules below
//! Decisions:     §34, §23

use lumacore_headers::{Kind, PARTS, WIDTH, banner};

use crate::files::{GROUPS, crate_roots, ratchet, sources, workspace};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

// *************************************************************************************************
// Headers
// The banner, the module doc, Parts in order, Sections (decisions §34 1–6).
// *************************************************************************************************

/// The tag lines a banner stands over.
const TAGS: [(&str, Kind); 2] = [("// FILE: ", Kind::File), ("// PART: ", Kind::Part)];

/// Files without a header yet: the count only goes down as files move.
const HEADERLESS_AT_MOST: usize = 265;

// *************************************************************************************************
// Size
// No file over 800 lines without a written reason (§34 9).
// *************************************************************************************************

/// The longest a file may be without a written reason.
const HARD_LIMIT: usize = 800;

/// Files over the limit, each with its reason. A split file leaves the list; nothing joins it
/// without a reason a reviewer accepts.
const LONG_FILES: &[(&str, &str)] = &[
    ("crates/model/lumacore-objects/src/commands/attributes.rs", "split in §34 / §33 3d (2545 lines on 2026-10-15)"),
    ("apps/lumacore-app/src/app.rs", "split in §34 / §33 3d (2083 lines on 2026-10-15)"),
    ("crates/model/lumacore-tree/src/programmer.rs", "split in §34 / §33 3d (1751 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/playback.rs", "split in §34 / §33 3d (1738 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/vfs.rs", "split in §34 / §33 3d (1711 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/mod.rs", "split in §34 / §33 3d (1577 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/media.rs", "split in §34 / §33 3d (1529 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/groups.rs", "split in §34 / §33 3d (1527 lines on 2026-10-15)"),
    ("crates/model/lumacore-tree/src/window_types.rs", "split in §34 / §33 3d (1344 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/control.rs", "split in §34 / §33 3d (1313 lines on 2026-10-15)"),
    ("crates/ui/lumacore-http/src/websocket/feeds.rs", "split in §34 / §33 3d (1284 lines on 2026-10-15)"),
    ("crates/model/lumacore-tree/src/tree.rs", "split in §34 / §33 3d (1272 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/playback_tests.rs", "split in §34 / §33 3d (1271 lines on 2026-10-15)"),
    ("crates/server/lumacore-lumasession/src/link.rs", "split in §34 / §33 3d (1212 lines on 2026-10-15)"),
    ("crates/server/lumacore-lumasession/src/election.rs", "split in §34 / §33 3d (1212 lines on 2026-10-15)"),
    ("crates/server/lumacore-server/src/state.rs", "split in §34 / §33 3d (1184 lines on 2026-10-15)"),
    ("crates/formats/lumacore-luma/src/backups.rs", "split in §34 / §33 3d (1173 lines on 2026-10-15; 1090 in lumacore-luma)"),
    ("crates/model/lumacore-playback/src/lib.rs", "split in §34 / §33 3d (1156 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/info.rs", "split in §34 / §33 3d (1088 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/windows.rs", "split in §34 / §33 3d (1087 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/dispatch.rs", "split in §34 / §33 3d (1083 lines on 2026-10-15)"),
    ("crates/server/lumacore-lumasession/src/stream.rs", "split in §34 / §33 3d (1069 lines on 2026-10-15)"),
    ("crates/model/lumacore-state/src/app_state.rs", "split in §34 / §33 3d (1064 lines on 2026-10-15)"),
    ("crates/model/lumacore-objects/src/commands/users.rs", "split in §34 / §33 3d (1038 lines on 2026-10-15)"),
    ("crates/model/lumacore-tree/src/attribute_resolver.rs", "split in §34 / §33 3d (980 lines on 2026-10-15)"),
    ("crates/ui/lumacore-shell/src/shell.rs", "split in §34 / §33 3d (922 lines on 2026-10-15)"),
    ("crates/model/lumacore-runtime/src/live_screens.rs", "split in §34 / §33 3d (912 lines on 2026-10-15)"),
    ("crates/dev/lumacore-sweep/src/run.rs", "split in §34 / §33 3d (862 lines on 2026-10-15)"),
    ("crates/server/lumacore-server/src/builder.rs", "split in §34 / §33 3d (850 lines on 2026-10-15)"),
    ("crates/formats/lumacore-gdtf/src/importer.rs", "split in §34 / §33 3d (848 lines on 2026-10-15; the §40 reader tests)"),
];

// *************************************************************************************************
// Modules and tests
// foo.rs + foo/, tests in foo/tests.rs (§34 10, 11).
// *************************************************************************************************

/// `mod.rs` files left.
const MOD_RS_AT_MOST: usize = 9;

/// Test files that aren't `foo/tests.rs` yet.
const OLD_TEST_FILES_AT_MOST: usize = 43;

// *************************************************************************************************
// Crate edges
// One direction, the clients on the wire only (§34 7, §23).
// *************************************************************************************************

/// Which workspace crates each crate may depend on.
const EDGES: &[(&str, &[&str])] = &[
    // Vocabulary at the bottom: no workspace crate below them.
    ("lumacore-base", &[]),
    // The System Monitor, a logging system every crate can write to (§34 25.10).
    ("lumacore-sysmon", &[]),
    ("lumacore-wire", &[]),
    ("lumacore-trace", &[]),
    // A state thread's two job lanes, generic over its context (§34 24.2).
    ("lumacore-lanes", &["lumacore-sysmon", "lumacore-base"]),
    ("lumacore-headers", &[]),
    ("lumacore-syntax-derive", &[]),
    // The catalog every crate adds its declarations to (decisions §34 21).
    ("lumacore-registry", &["lumacore-base"]),
    // Small data crates on base.
    ("lumacore-syntax", &["lumacore-base", "lumacore-registry"]),
    ("lumacore-playback", &["lumacore-base"]),
    ("lumacore-effects", &["lumacore-base"]),
    ("lumacore-timecode", &["lumacore-base"]),
    ("lumacore-runtime", &["lumacore-base"]),
    // The show tree, and the engines and formats on it (dmx never on http, §23).
    ("lumacore-tree", &["lumacore-fixtures", "lumacore-protocols", "lumacore-base", "lumacore-playback", "lumacore-effects", "lumacore-timecode", "lumacore-runtime", "lumacore-tags", "lumacore-registry"]),
    ("lumacore-tags", &["lumacore-base", "lumacore-registry"]),
    ("lumacore-gdtf", &["lumacore-base", "lumacore-fixtures", "lumacore-registry"]),
    // The .luma show file and the station's disk (§34 25.10).
    ("lumacore-luma", &["lumacore-registry", "lumacore-base", "lumacore-wire", "lumacore-sysmon", "lumacore-tree", "lumacore-gdtf"]),
    ("lumacore-patch", &["lumacore-fixtures", "lumacore-tree", "lumacore-registry"]),
    // This station's drives: the probe, the library tree on a drive, the ids (decisions §38).
    ("lumacore-drives", &["lumacore-luma", "lumacore-tree"]),
    ("lumacore-mvr", &[]),
    ("lumacore-poolxml", &[]),
    // The leaf network I/O: OS NICs, sockets, mDNS, PTP (decisions §34 20.6, 22, 25.14).
    ("lumacore-net", &["lumacore-base", "lumacore-runtime", "lumacore-protocols", "lumacore-wire"]),
    // The protocol contract, below the tree: line data, the row, the driver traits, the merger
    // (decisions §34 22, 25.14); each protocol crate sits on it — never on the renderer.
    ("lumacore-protocols", &["lumacore-base", "lumacore-runtime", "lumacore-registry"]),
    // The fixture data the show holds, below the tree (§34 25.10).
    ("lumacore-fixtures", &["lumacore-base", "lumacore-protocols"]),
    ("lumacore-artnet", &["lumacore-protocols", "lumacore-net", "lumacore-registry"]),
    ("lumacore-sacn", &["lumacore-protocols", "lumacore-net", "lumacore-registry"]),
    ("lumacore-osc", &["lumacore-protocols", "lumacore-base", "lumacore-tree", "lumacore-registry"]),
    // The renderer, on the interface for its input layers (decisions §34 20.3, 22).
    ("lumacore-render", &["lumacore-fixtures", "lumacore-protocols", "lumacore-base", "lumacore-tree", "lumacore-trace", "lumacore-registry"]),
    // The engine registry and its wrappers, on the renderer; HTTP under its features (§34 20.4, 24.2).
    ("lumacore-engines", &["lumacore-base", "lumacore-tree", "lumacore-wire", "lumacore-trace", "lumacore-render", "lumacore-http"]),
    // The session clock sync, pure apart from its socket (§34 24.2).
    ("lumacore-clocksync", &["lumacore-sysmon", "lumacore-base", "lumacore-tree", "lumacore-wire"]),
    ("lumacore-http", &["lumacore-tree", "lumacore-trace", "lumacore-wire", "lumacore-registry", "lumacore-webui"]),
    // The WebUI: files only (decisions §36 1).
    ("lumacore-webui", &[]),
    // The station's three layers (decisions §34 7): state → objects → the station.
    ("lumacore-state", &["lumacore-fixtures", "lumacore-luma", "lumacore-drives", "lumacore-sysmon", "lumacore-base", "lumacore-tree", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-gdtf", "lumacore-patch", "lumacore-render", "lumacore-wire", "lumacore-trace", "lumacore-registry"]),
    ("lumacore-objects", &["lumacore-fixtures", "lumacore-luma", "lumacore-drives", "lumacore-mvr", "lumacore-poolxml", "lumacore-sysmon", "lumacore-state", "lumacore-base", "lumacore-tree", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-gdtf", "lumacore-patch", "lumacore-protocols", "lumacore-net", "lumacore-render", "lumacore-wire", "lumacore-trace", "lumacore-registry"]),
    ("lumacore-terminal", &["lumacore-views", "lumacore-cmd-protocols", "lumacore-cmd-devices", "lumacore-cmd-station", "lumacore-cmd-showfile", "lumacore-suggest", "lumacore-cmd-treeshell", "lumacore-cmd-keys", "lumacore-cmd-tags", "lumacore-objects", "lumacore-state", "lumacore-tree", "lumacore-base", "lumacore-wire", "lumacore-syntax", "lumacore-registry"]),
    // The line runtime: the drivers and listeners the show asks for (decisions §34 22.1).
    ("lumacore-lines", &["lumacore-objects", "lumacore-render", "lumacore-protocols", "lumacore-net", "lumacore-tree", "lumacore-base", "lumacore-registry"]),
    // The TCP shell port, hosting the terminals on a state thread's lanes (§34 24.2).
    ("lumacore-shell", &["lumacore-sysmon", "lumacore-lanes", "lumacore-wire", "lumacore-tree", "lumacore-objects", "lumacore-terminal", "lumacore-registry"]),
    // The server: the state thread, the command port, discovery, the hosts the crates above ask for (§34 24.2).
    ("lumacore-server", &["lumacore-cmd-protocols", "lumacore-cmd-devices", "lumacore-cmd-showfile", "lumacore-luma", "lumacore-drives", "lumacore-sysmon", "lumacore-registry", "lumacore-base", "lumacore-wire", "lumacore-lanes", "lumacore-trace", "lumacore-tree", "lumacore-objects", "lumacore-render", "lumacore-engines", "lumacore-net", "lumacore-clocksync", "lumacore-lines", "lumacore-views", "lumacore-terminal", "lumacore-shell", "lumacore-http", "lumacore-lumasession"]),
    // LumaSession, the station ⇄ station side, on a narrow SessionHost (§34 19.3, 24.2).
    ("lumacore-lumasession", &["lumacore-cmd-lumasession", "lumacore-luma", "lumacore-sysmon", "lumacore-registry", "lumacore-base", "lumacore-wire", "lumacore-lanes", "lumacore-tree", "lumacore-objects", "lumacore-net"]),
    // What every UI shows, on objects (decisions §34 20.7).
    // The suggestion engine, on the kernel (§34 25.9).
    ("lumacore-suggest", &["lumacore-registry", "lumacore-base", "lumacore-wire", "lumacore-tree", "lumacore-objects"]),
    // The tree shell's words, on the kernel (§34 25.7).
    ("lumacore-cmd-treeshell", &["lumacore-registry", "lumacore-syntax", "lumacore-wire", "lumacore-luma", "lumacore-objects"]),
    // The keys, on the kernel (§34 25.4); the users' crate may use them, never the other way.
    // The show file: its words, objects, Backup menu and the service the server starts with (§34 25.7).
    ("lumacore-cmd-showfile", &["lumacore-registry", "lumacore-base", "lumacore-sysmon", "lumacore-wire", "lumacore-tree", "lumacore-luma", "lumacore-objects"]),
    // The LumaSession words, on the kernel's outbox (§34 25.7).
    ("lumacore-cmd-lumasession", &["lumacore-registry", "lumacore-wire", "lumacore-tree", "lumacore-objects", "lumacore-cmd-station"]),
    // The station's own words, on the show file (§34 25.7).
    ("lumacore-cmd-station", &["lumacore-luma", "lumacore-sysmon", "lumacore-registry", "lumacore-base", "lumacore-wire", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-tree", "lumacore-protocols", "lumacore-net", "lumacore-objects", "lumacore-cmd-showfile"]),
    // The devices: the lock, its gate, the device's look and screen config (§34 25.17).
    ("lumacore-cmd-devices", &["lumacore-registry", "lumacore-sysmon", "lumacore-objects", "lumacore-cmd-station"]),
    // The DMX lines, on the station's network (§34 25.17).
    ("lumacore-cmd-protocols", &["lumacore-registry", "lumacore-wire", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-tree", "lumacore-protocols", "lumacore-objects", "lumacore-cmd-station"]),
    // Each protocol's object (decisions §34 "34f7c hardened" 9): on the shared line words, naming
    // its own protocol's row.
    ("lumacore-cmd-sacn", &["lumacore-registry", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-protocols", "lumacore-objects", "lumacore-cmd-protocols", "lumacore-sacn"]),
    ("lumacore-cmd-artnet", &["lumacore-registry", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-protocols", "lumacore-objects", "lumacore-cmd-protocols", "lumacore-artnet"]),
    ("lumacore-cmd-osc", &["lumacore-registry", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-protocols", "lumacore-objects", "lumacore-cmd-protocols", "lumacore-osc"]),
    ("lumacore-cmd-keys", &["lumacore-registry", "lumacore-base", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-sysmon", "lumacore-wire", "lumacore-tree", "lumacore-objects"]),
    ("lumacore-cmd-tags", &["lumacore-registry", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-wire", "lumacore-tags", "lumacore-tree", "lumacore-objects"]),
    ("lumacore-views", &["lumacore-protocols", "lumacore-luma", "lumacore-sysmon", "lumacore-objects", "lumacore-tree", "lumacore-base", "lumacore-net", "lumacore-render", "lumacore-registry"]),
    ("lumacore-station", &["lumacore-terminal", "lumacore-views", "lumacore-objects", "lumacore-state", "lumacore-tree", "lumacore-syntax", "lumacore-syntax-derive", "lumacore-gdtf", "lumacore-patch", "lumacore-base", "lumacore-net", "lumacore-render", "lumacore-engines", "lumacore-clocksync", "lumacore-lanes", "lumacore-shell", "lumacore-lumasession", "lumacore-server", "lumacore-sweep", "lumacore-protocols", "lumacore-lines", "lumacore-artnet", "lumacore-sacn", "lumacore-osc", "lumacore-cmd-artnet", "lumacore-cmd-sacn", "lumacore-cmd-osc", "lumacore-wire", "lumacore-http", "lumacore-trace", "lumacore-registry"]),
    // The command sweep and the fuzz entry points: on objects, LumaSession and a protocol, never
    // the server (§34 20.10, 24.2).
    ("lumacore-sweep", &["lumacore-cmd-protocols", "lumacore-cmd-devices", "lumacore-cmd-station", "lumacore-cmd-showfile", "lumacore-cmd-keys", "lumacore-cmd-tags", "lumacore-suggest", "lumacore-cmd-treeshell", "lumacore-base", "lumacore-luma", "lumacore-sysmon", "lumacore-registry", "lumacore-tree", "lumacore-objects", "lumacore-lumasession", "lumacore-osc", "lumacore-sacn", "lumacore-artnet", "lumacore-cmd-osc", "lumacore-cmd-sacn", "lumacore-cmd-artnet"]),
    // The source rules read the registry and the banner font (decisions §34 20).
    ("lumacore-tidy", &["lumacore-cmd-protocols", "lumacore-cmd-devices", "lumacore-cmd-station", "lumacore-cmd-lumasession", "lumacore-cmd-showfile", "lumacore-cmd-keys", "lumacore-cmd-tags", "lumacore-objects", "lumacore-cmd-treeshell", "lumacore-tree", "lumacore-headers", "lumacore-registry", "lumacore-http"]),
    // Clients compile only the wire and the WebUI's files (§23 "Step 8", §36 3).
    ("lumacore-app", &["lumacore-wire", "lumacore-webui"]),
    ("lumacore-cli", &["lumacore-wire"]),
    ("lumacore-scripts", &[]),
    // The system test drives a whole station.
    // The whole catalog's tests: dev-dependencies only (§34 25.15).
    ("lumacore-catalog-tests", &[]),
    ("systemtest", &["lumacore-station", "lumacore-objects", "lumacore-protocols", "lumacore-engines", "lumacore-base", "lumacore-tree", "lumacore-artnet", "lumacore-osc", "lumacore-net", "lumacore-render", "lumacore-wire"]),
];

/// The WebUI is files only (decisions §36 1): `lumacore-webui` has no dependencies of any kind, so
/// the app and the HTTP engine take its files without taking code.
#[test]
fn the_webui_is_only_files() {
    let dir = workspace().join("crates/ui/lumacore-webui");
    let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).expect("read lumacore-webui/Cargo.toml");
    let mut section = String::new();
    let mut wrong = Vec::new();
    for line in manifest.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            section = t.to_string();
            continue;
        }
        if section.ends_with("dependencies]") && !t.is_empty() && !t.starts_with('#') {
            wrong.push(format!("{section} {t}"));
        }
    }
    assert!(wrong.is_empty(), "lumacore-webui is files only (decisions §36 1):\n{}", wrong.join("\n"));
}

/// The app's OS code lives in `src/platform.rs` and `src/platform/` (decisions §36 4): no other
/// file of the app names an OS in a `cfg`.
#[test]
fn os_code_lives_in_platform() {
    let src = workspace().join("apps/lumacore-app/src");
    let platform = src.join("platform");
    let mut files = Vec::new();
    crate::files::rust_files(&src, &mut files);
    let mut wrong = Vec::new();
    for path in files {
        if path == src.join("platform.rs") || path.starts_with(&platform) {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read app source");
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains("target_os") || ["cfg(unix", "cfg(windows", "cfg!(unix", "cfg!(windows"].iter().any(|c| code.contains(c)) {
                wrong.push(format!("{}:{}: {}", path.display(), n + 1, line.trim()));
            }
        }
    }
    assert!(wrong.is_empty(), "OS code outside the app's platform modules (decisions §36 4):\n{}", wrong.join("\n"));
}

// *************************************************************************************************
// READMEs
// Every workspace crate's specification page (decisions §34 18).
// *************************************************************************************************

/// Crates without a README.md yet: it may only go down (each crate gets one in its pass, §34 18).
const READMELESS_AT_MOST: usize = 4;

/// READMEs older than the seven sections, until their crate's pass.
const README_LEGACY: &[&str] = &["systemtest"];

const README_SECTIONS: [&str; 7] = ["## Purpose", "## Owns / Doesn't own", "## Public surface", "## Rules", "## Edges", "## Decisions", "## Testing"];

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// Every workspace crate (crates/ by group, apps/*, tests/systemtest, tests/catalog, tests/fuzz)
/// with its folder.
fn workspace_crates() -> Vec<(String, std::path::PathBuf)> {
    let root = workspace();
    let mut out: Vec<(String, std::path::PathBuf)> = crate_roots()
        .into_iter()
        .map(|p| (p.file_name().unwrap_or_default().to_string_lossy().to_string(), p))
        .collect();
    for entry in std::fs::read_dir(root.join("apps")).expect("read apps/").flatten() {
        if entry.path().join("Cargo.toml").exists() {
            out.push((entry.file_name().to_string_lossy().to_string(), entry.path()));
        }
    }
    for single in ["systemtest", "catalog", "fuzz"] {
        let dir = root.join("tests").join(single);
        if dir.join("Cargo.toml").exists() {
            out.push((single.to_string(), dir));
        }
    }
    out.sort();
    out
}

/// A test or tidy rule of that name exists somewhere in the workspace.
fn fn_exists(name: &str) -> bool {
    let root = workspace();
    let needle = format!("fn {name}(");
    let mut dirs: Vec<std::path::PathBuf> = ["crates", "apps", "tests/systemtest", "tests/catalog"].iter().map(|d| root.join(d)).collect();
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let p = entry.path();
            if p.is_dir() && !p.ends_with("target") && !p.ends_with("node_modules") {
                dirs.push(p);
            } else if p.extension().is_some_and(|e| e == "rs") && std::fs::read_to_string(&p).is_ok_and(|t| t.contains(&needle)) {
                return true;
            }
        }
    }
    false
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

// *************************************************************************************************
// Headers
// *************************************************************************************************

/// Every tag line stands under the banner its title renders to — nobody draws one by hand.
#[test]
fn banners_match_their_tag() {
    let mut wrong = Vec::new();
    for (path, text) in sources() {
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            for (prefix, kind) in TAGS {
                let Some(title) = line.strip_prefix(prefix) else { continue };
                match banner(kind, title) {
                    Err(e) => wrong.push(format!("{path}:{} {e}", i + 1)),
                    Ok(want) => {
                        let have = if i >= 6 { &lines[i - 6..i] } else { &lines[..0] };
                        if have != &want[..6] {
                            wrong.push(format!("{path}:{} the art above \"{title}\" isn't its banner (./scripts/unix/headers)", i + 1));
                        }
                    }
                }
            }
        }
    }
    assert!(wrong.is_empty(), "banners that don't match their tag line:\n{}", wrong.join("\n"));
}

/// A file with headers has them in the one shape: its banner first, the module doc's four
/// labels, Parts from the vocabulary in order, nothing but `use` / `mod` / `#![…]` before the
/// first Part, and Section rules exactly 100 columns.
#[test]
fn headers_stand_in_order() {
    let mut wrong = Vec::new();
    for (path, text) in sources() {
        if !text.lines().any(|l| l.starts_with("// FILE: ")) {
            continue;
        }
        let lines: Vec<&str> = text.lines().collect();
        let first = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
        if lines.get(first + 6).is_none_or(|l| !l.starts_with("// FILE: ")) {
            wrong.push(format!("{path}: the file banner isn't the first thing in it"));
        }
        let doc: Vec<&str> = lines.iter().filter(|l| l.starts_with("//!")).copied().collect();
        // A crate root whose doc points at its README: the README holds the labels (§34 18).
        let crate_root = path.ends_with("src/lib.rs") && doc.iter().any(|l| l.contains("README.md"));
        for label in ["Owns:", "Doesn't own:", "Entry points:", "Decisions:"].into_iter().filter(|_| !crate_root) {
            if !doc.iter().any(|l| l.trim_start_matches("//!").trim_start().starts_with(label)) {
                wrong.push(format!("{path}: the module doc has no `{label}` line"));
            }
        }
        let mut last: Option<usize> = None;
        let mut in_part = false;
        let mut in_test = false;
        // Inside a `use` that spans lines (`pub use x::{\n a,\n b,\n};`).
        let mut in_use = false;
        for (i, line) in lines.iter().enumerate() {
            if in_use {
                in_use = !line.trim_end().ends_with(';');
                continue;
            }
            if let Some(title) = line.strip_prefix("// PART: ") {
                match PARTS.iter().position(|p| *p == title) {
                    None => wrong.push(format!("{path}:{} PART {title} isn't one of {}", i + 1, PARTS.join(", "))),
                    Some(at) if last.is_some_and(|l| at <= l) => wrong.push(format!("{path}:{} PART {title} stands out of order ({})", i + 1, PARTS.join(", "))),
                    Some(at) => last = Some(at),
                }
                in_part = true;
                in_test |= title == "TESTS";
                continue;
            }
            if line.starts_with("// ***") && line.chars().count() != WIDTH {
                wrong.push(format!("{path}:{} a Section rule is {} columns, not {WIDTH} (./scripts/unix/headers --section)", i + 1, line.chars().count()));
            }
            let t = line.trim_start();
            let code = !t.is_empty() && !t.starts_with("//") && !t.starts_with("#![");
            let allowed = ["use ", "pub use ", "pub(crate) use ", "mod ", "pub mod ", "pub(crate) mod ", "#[cfg(", "#[path", "}"];
            if code && !in_part && !in_test && t.contains("use ") && !t.trim_end().ends_with(';') {
                in_use = true;
            }
            if code && !in_part && !in_test && !allowed.iter().any(|a| t.starts_with(a)) {
                wrong.push(format!("{path}:{} code before the first PART", i + 1));
                in_part = true;
            }
        }
    }
    assert!(wrong.is_empty(), "headers out of shape:\n{}", wrong.join("\n"));
}

/// Every file has its header; the ones without only go down (decisions §34).
#[test]
fn files_have_headers() {
    let missing: Vec<String> = sources().into_iter().filter(|(_, t)| !t.lines().any(|l| l.starts_with("// FILE: "))).map(|(p, _)| p).collect();
    ratchet("files without a file banner", missing.len(), HEADERLESS_AT_MOST, &missing.iter().take(20).cloned().collect::<Vec<_>>().join("\n"));
}

// *************************************************************************************************
// Size
// *************************************************************************************************

/// No file over the limit unless listed with its reason, and a listed file is still long.
#[test]
fn files_stay_short() {
    let mut wrong = Vec::new();
    let all = sources();
    for (path, text) in &all {
        let n = text.lines().count();
        let listed = LONG_FILES.iter().any(|(p, _)| p == path);
        if n > HARD_LIMIT && !listed {
            wrong.push(format!("{path}: {n} lines, over {HARD_LIMIT} — split it (or list it with a reason)"));
        }
        if n <= HARD_LIMIT && listed {
            wrong.push(format!("{path}: {n} lines now — take it off LONG_FILES"));
        }
    }
    for (path, _) in LONG_FILES {
        if !all.iter().any(|(p, _)| p == path) {
            wrong.push(format!("{path}: listed in LONG_FILES but gone"));
        }
    }
    assert!(wrong.is_empty(), "file lengths (decisions §34 9):\n{}", wrong.join("\n"));
}

// *************************************************************************************************
// Modules and tests
// *************************************************************************************************

/// `mod.rs` files left: `foo.rs` + `foo/` is the one module style.
#[test]
fn modules_are_foo_rs_not_mod_rs() {
    let found: Vec<String> = sources().into_iter().map(|(p, _)| p).filter(|p| p.ends_with("/mod.rs")).collect();
    ratchet("mod.rs files", found.len(), MOD_RS_AT_MOST, &found.join("\n"));
}

/// Tests that aren't `foo/tests.rs` yet: sibling `*_tests.rs` files and `#[path]` test modules.
#[test]
fn tests_live_in_foo_tests_rs() {
    let found: Vec<String> = sources()
        .into_iter()
        .filter(|(p, t)| p.ends_with("_tests.rs") || t.contains("#[path = \""))
        .map(|(p, _)| p)
        .collect();
    ratchet("test files that aren't foo/tests.rs", found.len(), OLD_TEST_FILES_AT_MOST, &found.join("\n"));
}

// *************************************************************************************************
// Crate edges
// *************************************************************************************************

/// Which workspace crates each crate may depend on (`[dependencies]`, path deps). One direction:
/// vocabulary crates at the bottom, the station on top, the clients on the wire only.
#[test]
fn crate_edges_point_down() {
    let root = workspace();
    let mut wrong = Vec::new();
    let mut manifests: Vec<std::path::PathBuf> = crate_roots().iter().map(|p| p.join("Cargo.toml")).collect();
    for e in std::fs::read_dir(root.join("apps")).expect("read apps/").flatten() {
        if e.path().join("Cargo.toml").exists() {
            manifests.push(e.path().join("Cargo.toml"));
        }
    }
    manifests.push(root.join("tests/systemtest/Cargo.toml"));
    manifests.push(root.join("tests/catalog/Cargo.toml"));
    for m in manifests {
        let text = std::fs::read_to_string(&m).unwrap_or_default();
        let name = text.lines().find_map(|l| l.strip_prefix("name = \"").map(|n| n.trim_end_matches('"').to_string())).unwrap_or_default();
        // Only `[dependencies]` (and target / build sections); dev-dependencies may reach further.
        let mut deps = Vec::new();
        let mut section = String::new();
        for line in text.lines() {
            let t = line.trim();
            if t.starts_with('[') {
                section = t.to_string();
                continue;
            }
            if section == "[dependencies]" && t.contains("path = ") && let Some(dep) = t.split(['=', ' ']).next() {
                deps.push(dep.trim().to_string());
            }
        }
        let Some((_, allowed)) = EDGES.iter().find(|(c, _)| *c == name) else {
            wrong.push(format!("{name}: not in EDGES — say which crates it may depend on"));
            continue;
        };
        for d in deps {
            if !allowed.contains(&d.as_str()) {
                wrong.push(format!("{name} → {d}: not an allowed edge (decisions §34 7)"));
            }
        }
    }
    assert!(wrong.is_empty(), "crate edges:\n{}", wrong.join("\n"));
}

/// Every crate is `lumacore-<category>` (decisions §34 17) and sits in its group (§34 23):
/// `crates/<group>[/<family>…]/lumacore-<name>`, its folder and its package one name, the folder
/// listed in `GROUPS` (the station is an app, §34 24).
#[test]
fn crates_sit_in_their_group() {
    let crates = workspace().join("crates");
    let mut wrong = Vec::new();
    for path in crate_roots() {
        let folder = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let parent = path.parent().and_then(|p| p.strip_prefix(&crates).ok()).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
        let at = format!("crates/{}{folder}", if parent.is_empty() { String::new() } else { format!("{parent}/") });
        if folder.strip_prefix("lumacore-").is_none_or(str::is_empty) {
            wrong.push(format!("{at}: name it lumacore-<category>"));
        }
        if !GROUPS.iter().any(|(g, _)| *g == parent) {
            wrong.push(format!("{at}: not in a group — put it in one of GROUPS (decisions §34 23)"));
        }
        let manifest = std::fs::read_to_string(path.join("Cargo.toml")).expect("read Cargo.toml");
        let package = manifest
            .lines()
            .skip_while(|l| l.trim() != "[package]")
            .find_map(|l| l.trim().strip_prefix("name = ").map(|n| n.trim_matches('"').to_string()));
        if package.as_deref() != Some(folder.as_str()) {
            wrong.push(format!("{at}: its package is {package:?} — the folder's name"));
        }
    }
    for (group, _) in GROUPS {
        if !crates.join(group).is_dir() {
            wrong.push(format!("crates/{group}: a group in GROUPS with no folder"));
        }
    }
    assert!(wrong.is_empty(), "crate folders (decisions §34 17, 23):\n{}", wrong.join("\n"));
}

/// The developer tools never ship (decisions §34 25.8): no crate depends on one in `crates/dev/`
/// (`[dependencies]` / `[build-dependencies]`) unless it is a tool itself, the station (the
/// sweep's thin main), systemtest or the fuzz crate.
#[test]
fn dev_tools_never_ship() {
    let dev = workspace().join("crates/dev");
    let tools: Vec<String> = crate_roots()
        .into_iter()
        .filter(|p| p.starts_with(&dev))
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
        .collect();
    let mut wrong = Vec::new();
    for (name, dir) in workspace_crates() {
        if dir.starts_with(&dev) || matches!(name.as_str(), "lumacore-station" | "systemtest" | "fuzz") {
            continue;
        }
        let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        let mut section = String::new();
        for line in manifest.lines() {
            let t = line.trim();
            if t.starts_with('[') {
                section = t.to_string();
                continue;
            }
            let shipped = section == "[dependencies]" || section == "[build-dependencies]" || section.ends_with(".dependencies]");
            if let Some(tool) = tools.iter().find(|tool| t.starts_with(&format!("{tool} "))) && shipped {
                wrong.push(format!("{name} → {tool} ({section})"));
            }
        }
    }
    assert!(wrong.is_empty(), "a product crate depends on a developer tool (decisions §34 25.8):\n{}", wrong.join("\n"));
}

// *************************************************************************************************
// READMEs
// *************************************************************************************************

#[test]
fn crates_have_readmes() {
    let mut missing = Vec::new();
    let mut wrong = Vec::new();
    for (name, dir) in workspace_crates() {
        let Ok(text) = std::fs::read_to_string(dir.join("README.md")) else {
            missing.push(name);
            continue;
        };
        if README_LEGACY.contains(&name.as_str()) {
            continue;
        }
        let package = std::fs::read_to_string(dir.join("Cargo.toml"))
            .ok()
            .and_then(|m| m.lines().find_map(|l| l.trim().strip_prefix("name = ").map(|n| n.trim_matches('"').to_string())))
            .unwrap_or(name.clone());
        if text.lines().next() != Some(format!("# {package}").as_str()) {
            wrong.push(format!("{name}/README.md: the first line is `# {package}`"));
        }
        // The seven sections, in order.
        let heads: Vec<&str> = text.lines().filter(|l| l.starts_with("## ")).collect();
        if heads != README_SECTIONS {
            wrong.push(format!("{name}/README.md: its sections are {heads:?}, not {README_SECTIONS:?}"));
            continue;
        }
        let section = |head: &str| -> String {
            let from = text.find(head).map_or(text.len(), |i| i + head.len());
            let to = text[from..].find("\n## ").map_or(text.len(), |i| from + i);
            text[from..to].to_string()
        };
        // Every rule names its check, and a named test or tidy rule exists.
        let rules = section("## Rules");
        let mut rule = String::new();
        let mut items: Vec<String> = Vec::new();
        for line in rules.lines() {
            if line.starts_with("- ") {
                if !rule.is_empty() {
                    items.push(std::mem::take(&mut rule));
                }
                rule = line.to_string();
            } else if !rule.is_empty() && !line.trim().is_empty() {
                rule.push(' ');
                rule.push_str(line.trim());
            }
        }
        if !rule.is_empty() {
            items.push(rule);
        }
        for item in items {
            let Some(check) = item.rsplit_once('[').map(|(_, c)| c.trim_end().trim_end_matches(']').to_string()) else {
                wrong.push(format!("{name}/README.md: a rule names no check: {item}"));
                continue;
            };
            for part in check.split('+').map(str::trim) {
                for kind in ["tidy ", "test "] {
                    if let Some(f) = part.strip_prefix(kind)
                        && !fn_exists(f)
                    {
                        wrong.push(format!("{name}/README.md: no {kind}`{f}` (named by a rule)"));
                    }
                }
            }
        }
        // Edges: the workspace crates it depends on, as its Cargo.toml says.
        let edges = section("## Edges");
        // The "Depends on:" bullet with its wrapped lines, bold marks taken off.
        let bullet: String = edges
            .lines()
            .skip_while(|l| !l.contains("Depends on:"))
            .enumerate()
            .take_while(|(i, l)| *i == 0 || !l.trim_start().starts_with("- "))
            .map(|(_, l)| l.trim())
            .collect::<Vec<_>>()
            .join(" ")
            .replace("**", "");
        let named: std::collections::BTreeSet<String> = bullet
            .split_once("Depends on:")
            .map(|(_, list)| list.split(',').map(|c| c.trim().trim_end_matches('.').to_string()).filter(|c| !c.is_empty() && c != "nothing").collect())
            .unwrap_or_default();
        let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        let deps: std::collections::BTreeSet<String> = manifest
            .split("[dependencies]")
            .nth(1)
            .map(|d| d.split("\n[").next().unwrap_or(""))
            .unwrap_or("")
            .lines()
            .filter_map(|l| l.split_once(" = ").map(|(k, _)| k.trim().to_string()))
            .filter(|k| k.starts_with("lumacore-"))
            .collect();
        if named != deps {
            wrong.push(format!("{name}/README.md: Edges name {named:?}, its Cargo.toml depends on {deps:?}"));
        }
    }
    ratchet("crates without a README.md", missing.len(), READMELESS_AT_MOST, &missing.join(", "));
    assert!(wrong.is_empty(), "READMEs (decisions §34 18):\n{}", wrong.join("\n"));
}

// *************************************************************************************************
// The catalog and safety
// Every crate on the registry registers and installs; no unsafe in the source (§34 21).
// *************************************************************************************************

/// The crates whose `[dependencies]` name `lumacore-registry` (the registry itself aside).
fn registry_crates() -> Vec<(String, std::path::PathBuf)> {
    workspace_crates()
        .into_iter()
        .filter(|(name, dir)| {
            name != "lumacore-registry"
                && std::fs::read_to_string(dir.join("Cargo.toml"))
                    .ok()
                    .and_then(|m| m.split("[dependencies]").nth(1).map(|d| d.split("\n[").next().unwrap_or("").to_string()))
                    .is_some_and(|deps| deps.lines().any(|l| l.trim_start().starts_with("lumacore-registry ")))
        })
        .collect()
}

/// Every crate on the registry has its `register` (its rows, after the crates it builds on) and
/// the test-only hook that installs its catalog before its first test; every crate on state or
/// objects is on the registry (decisions §34 21, 21.8).
#[test]
fn crates_on_the_registry_register_and_install() {
    let mut wrong = Vec::new();
    for (name, dir) in workspace_crates() {
        let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        let deps = manifest.split("[dependencies]").nth(1).map(|d| d.split("\n[").next().unwrap_or("").to_string()).unwrap_or_default();
        let on = |c: &str| deps.lines().any(|l| l.trim_start().starts_with(&format!("{c} ")));
        // A crate on the station gets the whole catalog from it (decisions §34 24.1).
        if (on("lumacore-state") || on("lumacore-objects")) && !on("lumacore-registry") && !on("lumacore-station") {
            wrong.push(format!("{name}: it builds on state or objects but not on lumacore-registry"));
        }
    }
    for (name, dir) in registry_crates() {
        // The station's library is its catalog (decisions §34 24.1).
        let root = if name == "lumacore-station" { "src/catalog.rs" } else { "src/lib.rs" };
        let lib = std::fs::read_to_string(dir.join(root)).unwrap_or_default();
        if name != "lumacore-tidy" && !lib.contains("pub fn register(r: &mut lumacore_registry::Registry)") {
            wrong.push(format!("{name}/{root}: no `pub fn register(r: &mut lumacore_registry::Registry)`"));
        }
        if !(lib.contains("#[cfg(test)]\n#[ctor::ctor]") && lib.contains("lumacore_registry::install_with(")) {
            wrong.push(format!("{name}/{root}: no test-only `#[ctor::ctor]` hook installing its catalog"));
        }
    }
    assert!(wrong.is_empty(), "the catalog (decisions §34 21):\n{}", wrong.join("\n"));
}

/// No `unsafe` block, fn or impl and no `allow(unsafe_code)` written in the source: the
/// workspace lint is `deny` (not `forbid`) only so ctor's test hook expands (decisions §34 21.8).
#[test]
fn no_unsafe_code() {
    let mut wrong = Vec::new();
    for (path, text) in sources() {
        if path == "crates/dev/lumacore-tidy/src/structure.rs" {
            continue;
        }
        for (i, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            let words = code.split(|c: char| !(c.is_alphanumeric() || c == '_'));
            if words.clone().any(|w| w == "unsafe") || code.contains("unsafe_code") {
                wrong.push(format!("{path}:{} {}", i + 1, line.trim()));
            }
        }
    }
    assert!(wrong.is_empty(), "unsafe code (decisions §34 21.8):\n{}", wrong.join("\n"));
}
