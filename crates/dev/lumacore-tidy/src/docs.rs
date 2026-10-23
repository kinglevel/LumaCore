// ██████╗  ██████╗  ██████╗███████╗
// ██╔══██╗██╔═══██╗██╔════╝██╔════╝
// ██║  ██║██║   ██║██║     ███████╗
// ██║  ██║██║   ██║██║     ╚════██║
// ██████╔╝╚██████╔╝╚██████╗███████║
// ╚═════╝  ╚═════╝  ╚═════╝╚══════╝
// FILE: docs
//! Tidy rules for the docs (decisions docs/decisions/0282-docs-checked-in-the-tidy-gate):
//! `CLAUDE.md` stays under its cap because it loads into every session, and
//! `scripts/unix/docs_check` — the records, the paths, the links, the code-owned numbers — runs
//! inside the gate, so the docs fail mechanically when they rot.
//!
//! Owns:          the docs rules and the cap
//! Doesn't own:   the checks themselves (scripts/unix/docs_check), the rulebook check (rulebook.rs)
//! Entry points:  the #[test] rules below
//! Decisions:     docs/decisions/0282-docs-checked-in-the-tidy-gate

use std::process::Command;

use crate::files::workspace;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The most lines `CLAUDE.md` may have (the md upgrade's decision 10: it loads into every session).
const CLAUDE_MD_LINES: usize = 150;

/// The docs checker, from the workspace root.
const DOCS_CHECK: &str = "scripts/unix/docs_check";

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// `CLAUDE.md` stays under its cap: a rule that outgrows it moves to `.claude/rules/` or a record.
#[test]
fn claude_md_stays_short() {
    let text = std::fs::read_to_string(workspace().join("CLAUDE.md")).expect("CLAUDE.md");
    let lines = text.lines().count();
    assert!(lines <= CLAUDE_MD_LINES, "CLAUDE.md is {lines} lines, more than {CLAUDE_MD_LINES}: move a rule to .claude/rules/<area>.md or a record (decisions 0282)");
}

/// The docs pass `scripts/unix/docs_check`: every record well-formed, every path and link
/// resolving, no code-owned number in prose, the roadmap within its caps.
#[test]
fn the_docs_pass_docs_check() {
    let root = workspace();
    let out = Command::new(root.join(DOCS_CHECK)).current_dir(&root).output().unwrap_or_else(|e| panic!("{DOCS_CHECK} did not start ({e}); it needs python3 on the path"));
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success(), "{DOCS_CHECK} failed (decisions 0282):\n{text}");
}
