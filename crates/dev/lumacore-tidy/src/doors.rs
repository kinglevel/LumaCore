// ██████╗  ██████╗  ██████╗ ██████╗ ███████╗
// ██╔══██╗██╔═══██╗██╔═══██╗██╔══██╗██╔════╝
// ██║  ██║██║   ██║██║   ██║██████╔╝███████╗
// ██║  ██║██║   ██║██║   ██║██╔══██╗╚════██║
// ██████╔╝╚██████╔╝╚██████╔╝██║  ██║███████║
// ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: doors
//! Shared things go through their one door (decisions §34 12): tree paths through
//! `core::tree_path`, the disk through `core::drive`, error lines through `core::errors`, targets
//! through `core::resolve`, the TUI's colours through `tui_look` — and what one door does has no
//! copies elsewhere.
//!
//! Owns:          the door rules, COPIES, OLD_RESOLVERS and their ratchets
//! Doesn't own:   the doors themselves (named in code-structure.md)
//! Entry points:  the #[test] rules below
//! Decisions:     §25, §29, §30 "12b", §31, §33 57, §34 12

use crate::files::{crate_dir, is_test_file, line_of, non_test_source, rust_files, sources, station_files, station_rel, string_literals};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The resolvers and lookups that predate the one resolver (decisions §33 57 — strict): their
/// uses may only go down until part 5 removes them. Everything new goes through
/// `core::resolve`; a new range parser, "number or name" lookup or `unquote` copy is refused
/// here when it reuses one of these, and in review otherwise.
const OLD_RESOLVERS: &[&str] = &[
    "parse_selection(",
    "resolve_parsed(",
    "number_list(",
    "set_rows::split(",
    "parse_target(",
    "fn unquote(",
    "find_group_by_name(",
    "group_by_name(",
    "find_profile(",
    "profile_no(",
    "find_station(",
    "resolve_station(",
    "row_named(",
    "network_id(",
    "bar_no(",
    "group_ref(",
    "root_no(",
    "asset_row(",
    "pool_item(",
];

/// Lower it as uses go (part 3d moves the handlers; part 5 brings it to 0).
const OLD_RESOLVER_USES_AT_MOST: usize = 91;

/// Copies of what one shared door does, by the pattern that gives them away, with how many are
/// left. Each count only goes down; the door is named in code-structure.md.
const COPIES: &[(&str, &str, usize)] = &[
    ("the next free number (pools::next_free)", "(1..).find(|", 6),
    ("the next free number, after the last (pools::next_free)", "next_back().map_or(1", 23),
    ("the DataPool scope (AppState::in_datapool)", "std::mem::replace(&mut state.datapool_override", 6),
    ("a not-found reply (errors::not_found)", "not found.\"", 24),
    ("an ObjectDef spelled out field by field (ObjectDef::new and its builders)", "ObjectDef = ObjectDef {", 42),
    ("on/off words beyond the four (props::on_off)", "\"true\" |", 5),
    ("a trace macro outside lumacore-trace (lumacore_trace::trace_zone!)", "tracy_client::", 0),
];

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

// *************************************************************************************************
// Doors
// One way in for paths, the disk, error lines and colours.
// *************************************************************************************************

/// Tree paths are typed (decisions §31): a `"Root.…"` path literal lives only in
/// `core/tree_path.rs` (and tests); everything else names a `tree_path::Node`. Anything else
/// names `// tidy: tree-path-ok (<reason>)` on its line or the one above.
#[test]
fn tree_paths_are_typed() {
    let mut files = Vec::new();
    files.extend(station_files(""));
    let mut violations = Vec::new();
    for file in files {
        let rel = station_rel(&file);
        if rel == "core/tree_path.rs" || is_test_file(&rel) {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source");
        let src = non_test_source(&text);
        let lines: Vec<&str> = src.lines().collect();
        for (pos, lit) in string_literals(src) {
            let line = line_of(src, pos);
            let excused = [line, line.saturating_sub(1)]
                .iter()
                .any(|&l| l > 0 && lines.get(l - 1).is_some_and(|t| t.contains("tidy: tree-path-ok")));
            if lit.get(..5).is_some_and(|p| p.eq_ignore_ascii_case("root.")) && !excused {
                violations.push(format!("{rel}:{line} \"{lit}\""));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "tree path literals outside core::tree_path (decisions §31) — name a Node:\n  {}",
        violations.join("\n  ")
    );
}

/// `core/` reaches the disk only through `core::drive` (decisions §25 "Step
/// 10 hardening"), so the command sweep can close it. Anything else names
/// `// tidy: drive-ok (<reason>)` on its line or the one above.
#[test]
fn core_reaches_the_disk_only_through_drive() {
    let mut files = Vec::new();
    files.extend(station_files("core"));
    let mut violations = Vec::new();
    for file in files {
        let rel = station_rel(&file);
        if rel == "core/drive.rs" || is_test_file(&rel) {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source");
        let lines: Vec<&str> = non_test_source(&text).lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            let opens = ["std::fs::File::", "std::fs::OpenOptions", "use std::fs;", "std::fs::{self"]
                .iter()
                .any(|p| code.contains(p))
                || code.match_indices("std::fs::").any(|(at, _)| {
                    code[at + 9..].starts_with(|c: char| c.is_ascii_lowercase())
                });
            let excused = line.contains("tidy: drive-ok")
                || i.checked_sub(1).is_some_and(|p| lines[p].contains("tidy: drive-ok"));
            if opens && !excused {
                violations.push(format!("{rel}:{} {}", i + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "disk access outside core::drive (decisions §25):\n  {}",
        violations.join("\n  ")
    );
}

/// Every error reply has a code (decisions §29 "Codes everywhere"): only
/// `core::errors` renders an error line (`render_code`, `render`, `reply`);
/// nothing else calls `text::render_error`.
#[test]
fn error_replies_carry_a_code() {
    let mut files = Vec::new();
    files.extend(station_files(""));
    let mut violations = Vec::new();
    for file in files {
        let rel = station_rel(&file);
        if rel == "core/errors.rs" || rel == "core/text.rs" || is_test_file(&rel) {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source");
        for (i, line) in non_test_source(&text).lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains("render_error(") || code.contains("text::render_error") {
                violations.push(format!("{rel}:{} {}", i + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "error reply without a code — use errors::render_code / errors::reply (decisions §29):\n  {}",
        violations.join("\n  ")
    );
}

/// The TUI follows the user's theme (decisions §30 "12b"): drawing code in
/// `terminal/` names colours by role (`tui_look::role`); only the theme
/// module (`tui_look.rs`) and the ANSI encoder (`command_line.rs`) name a
/// colour. Output text keeps its ANSI slot names (`nu_ansi_term`), which the
/// theme pass turns into the user's colours.
#[test]
fn terminal_colours_only_in_the_look() {
    let mut failures = Vec::new();
    let mut files = Vec::new();
    rust_files(&crate_dir("lumacore-terminal"), &mut files);
    for file in files {
        let name = file.file_name().unwrap_or_default().to_string_lossy().to_string();
        if name == "tui_look.rs" || name == "command_line.rs" || is_test_file(&station_rel(&file)) {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source");
        for (n, line) in non_test_source(&text).lines().enumerate() {
            if line.trim_start().starts_with("//") || line.contains("nu_ansi_term") {
                continue;
            }
            if line.contains("Color::") {
                failures.push(format!("{name}:{}: {} — use tui_look::role", n + 1, line.trim()));
            }
        }
    }
    assert!(failures.is_empty(), "raw colours in the TUI:\n{}", failures.join("\n"));
}

// *************************************************************************************************
// The one resolver
// The old lookups only go down (§33 57).
// *************************************************************************************************

#[test]
fn old_resolvers_only_go_down() {
    let files = station_files("");
    let mut uses: Vec<(String, usize)> = Vec::new();
    for path in &files {
        if station_rel(path) == "core/resolve.rs" || is_test_file(&path.to_string_lossy()) {
            continue;
        }
        let text = std::fs::read_to_string(path).unwrap();
        let src = non_test_source(&text);
        for pattern in OLD_RESOLVERS {
            let n = src.lines().filter(|l| !l.trim_start().starts_with("//")).map(|l| l.matches(pattern).count()).sum::<usize>();
            if n > 0 {
                uses.push((format!("{} {pattern}", station_rel(path)), n));
            }
        }
    }
    let total: usize = uses.iter().map(|(_, n)| n).sum();
    let listing = uses.iter().map(|(w, n)| format!("  {n} {w}")).collect::<Vec<_>>().join("\n");
    assert!(
        total <= OLD_RESOLVER_USES_AT_MOST,
        "{total} uses of the old resolvers, more than {OLD_RESOLVER_USES_AT_MOST} — resolve through core::resolve (decisions §33 57):\n{listing}"
    );
    assert!(total == OLD_RESOLVER_USES_AT_MOST, "{total} uses of the old resolvers now — lower OLD_RESOLVER_USES_AT_MOST to {total}");
}

// *************************************************************************************************
// Copies
// Each copy count only goes down (§34 12, 13).
// *************************************************************************************************

/// No new copy of what a shared door does; the counts are lowered as copies go.
#[test]
fn shared_doors_have_no_copies() {
    let all = sources();
    let mut wrong = Vec::new();
    for (what, pattern, at_most) in COPIES {
        let mut found = Vec::new();
        for (path, text) in &all {
            // The rule's own table, the one object door (§35) and the trace macros' crate (§34 20).
            if path == "crates/dev/lumacore-tidy/src/doors.rs" || path.ends_with("pools/door.rs") || path.starts_with("crates/foundation/lumacore-trace/") {
                continue;
            }
            let n = text.matches(pattern).count();
            if n > 0 {
                found.push(format!("  {n} {path}"));
            }
        }
        let now: usize = found.iter().map(|l| l.trim().split(' ').next().and_then(|n| n.parse::<usize>().ok()).unwrap_or(0)).sum();
        if now > *at_most {
            wrong.push(format!("{now} copies of {what} (`{pattern}`), more than {at_most} — use the shared door:\n{}", found.join("\n")));
        } else if now < *at_most {
            wrong.push(format!("{now} copies of {what} now — lower its count to {now}"));
        }
    }
    assert!(wrong.is_empty(), "shared doors (decisions §34 12):\n{}", wrong.join("\n"));
}
