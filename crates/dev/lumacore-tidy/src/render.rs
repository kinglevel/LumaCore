// ██████╗ ███████╗███╗   ██╗██████╗ ███████╗██████╗
// ██╔══██╗██╔════╝████╗  ██║██╔══██╗██╔════╝██╔══██╗
// ██████╔╝█████╗  ██╔██╗ ██║██║  ██║█████╗  ██████╔╝
// ██╔══██╗██╔══╝  ██║╚██╗██║██║  ██║██╔══╝  ██╔══██╗
// ██║  ██║███████╗██║ ╚████║██████╔╝███████╗██║  ██║
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝
// FILE: render
//! The render rule (decisions §24 "Step 9 hardening"): between `// tidy: render-begin` and
//! `// tidy: render-end` the per-frame path never locks, logs, formats or allocates.
//!
//! Owns:          RENDER_FILES, RENDER_FORBIDDEN, the region scanner, the rule
//! Doesn't own:   the allocator check in debug builds (assert_no_alloc), the budget test
//! Entry points:  render_paths_never_lock_log_or_allocate
//! Decisions:     §24 "Step 9 hardening"

use crate::files::crate_dir;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Files with per-frame code (crate, file under its `src/`), between
/// `// tidy: render-begin` and `// tidy: render-end`: the render tick, the
/// engine's frame, the lines' send. Each must keep at least one region.
const RENDER_FILES: &[(&str, &str)] = &[
    ("lumacore-render", "render.rs"),
    ("lumacore-render", "engine.rs"),
    ("lumacore-lines", "lines.rs"),
    ("lumacore-sacn", "source.rs"),
    ("lumacore-sacn", "driver.rs"),
    ("lumacore-artnet", "source.rs"),
    ("lumacore-artnet", "driver.rs"),
];

/// What the per-frame path never does: lock, log, format, build strings or
/// collections, allocate. A line may say why it is fine with
/// `// tidy: render-ok (<reason>)`.
const RENDER_FORBIDDEN: &[&str] = &[
    "Mutex",
    "RwLock",
    "log::",
    "println!",
    "eprintln!",
    "format!",
    "String",
    "to_string(",
    "to_owned(",
    "Vec::new",
    "Vec::with_capacity",
    "vec!",
    "HashMap",
    "HashSet",
    "BTreeMap",
    "BTreeSet",
    ".collect",
    "Box::new",
    "Arc::new",
    ".clone_from",
];

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// Forbidden things in the render regions of `src`: (line, token).
fn render_violations(src: &str) -> (usize, Vec<(usize, &'static str)>) {
    let mut regions = 0;
    let mut inside = false;
    let mut found = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("// tidy: render-begin") {
            inside = true;
            regions += 1;
            continue;
        }
        if t.starts_with("// tidy: render-end") {
            inside = false;
            continue;
        }
        if !inside || t.starts_with("//") || line.contains("tidy: render-ok (") {
            continue;
        }
        let code = line.split("//").next().unwrap_or(line);
        for token in RENDER_FORBIDDEN {
            // A whole word: `ArrayVec::new` (on the stack) is not `Vec::new`.
            let whole = code.match_indices(token).any(|(at, _)| {
                !token.starts_with(|c: char| c.is_alphanumeric())
                    || !code[..at]
                        .chars()
                        .next_back()
                        .is_some_and(|c| c.is_alphanumeric() || c == '_')
            });
            if whole {
                found.push((i + 1, *token));
            }
        }
    }
    (regions, found)
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[test]
fn render_paths_never_lock_log_or_allocate() {
    let mut failures = Vec::new();
    for (krate, file) in RENDER_FILES {
        let src = std::fs::read_to_string(crate_dir(krate).join(file)).expect("render file exists");
        let (regions, found) = render_violations(&src);
        if regions == 0 {
            failures.push(format!("{krate}/src/{file}: no `// tidy: render-begin` region"));
        }
        for (line, token) in found {
            failures.push(format!("{krate}/src/{file}:{line}: `{token}` in the per-frame path"));
        }
    }
    assert!(failures.is_empty(), "render rule (decisions §24):\n{}", failures.join("\n"));
}

#[test]
fn render_rule_scanner_finds_violations() {
    let src = "fn a() { let v = Vec::new(); }\n// tidy: render-begin (x)\nfn b() { let m = Mutex::new(1); let a = ArrayVec::new(); }\nfn c() { let s = format!(\"x\"); } // tidy: render-ok (test)\n// tidy: render-end\nfn d() { log::info!(\"\"); }\n";
    let (regions, found) = render_violations(src);
    assert_eq!(regions, 1);
    assert_eq!(found, vec![(3, "Mutex")]);
}
