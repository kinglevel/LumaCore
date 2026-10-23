//! The reference check (decisions §21 "6d hardening", a Total Reference
//! Update): after a load, join, merge or undo restore every stored link must
//! resolve and nothing may contain itself. Broken links are **kept** (a later
//! merge or undo can heal them) and listed — sysmon, `Info Show`,
//! `List Reference /Broken` — never silently dropped.
//!
//! The patch is checked the same way (R1-52, `lumacore_patch::check`):
//! overlapping fixtures, address 0, a break past 512, a break on a missing
//! universe, a mode missing from its type or a footprint that no longer
//! matches — listed with the links, the patch left as it is.

use super::AppState;
use lumacore_base::IdentityReport;
use nu_ansi_term::{Color, Style};

/// Walk the show, keep the report and note problems in sysmon.
pub fn check(state: &mut AppState, after: &str) -> IdentityReport {
    let report = state.root.show_data.identity();
    let mut lines = problems(&report);
    lines.extend(patch_problems(state));
    for line in lines.iter().take(20) {
        lumacore_sysmon::sysmon_note("References", format!("after {after}: {line}"));
    }
    let n = lines.len();
    if n > 20 {
        lumacore_sysmon::sysmon_note(
            "References",
            format!("after {after}: {} more (List Reference /Broken)", n - 20),
        );
    }
    state.references = Some((after.to_string(), report.clone()));
    report
}

/// How many things are wrong.
pub fn problem_count(r: &IdentityReport) -> usize {
    r.broken.len() + r.wrong_kind.len() + r.loops.len() + r.duplicates.len() + r.nil.len()
}

/// What's wrong with the patch, checked now (`lumacore_patch::check`).
pub fn patch_problems(state: &AppState) -> Vec<String> {
    lumacore_patch::check::problems(&state.root.show_data)
        .into_iter()
        .map(|l| format!("Patch: {l}"))
        .collect()
}

/// One line per problem.
pub fn problems(r: &IdentityReport) -> Vec<String> {
    let mut out = Vec::new();
    for l in &r.broken {
        out.push(format!("{} → missing {} {}", l.path, l.kind, l.target));
    }
    for (l, found) in &r.wrong_kind {
        out.push(format!(
            "{} → {} is a {found}, not a {}",
            l.path, l.target, l.kind
        ));
    }
    for chain in &r.loops {
        out.push(format!("contains itself: {}", chain.join(" → ")));
    }
    for (g, paths) in &r.duplicates {
        out.push(format!("shared GUID {g}: {}", paths.join(", ")));
    }
    for p in &r.nil {
        out.push(format!("{p} has no GUID"));
    }
    out
}

/// `Info Show`: "ok (n objects, m links)" or the problem count.
pub fn summary(state: &AppState) -> String {
    let report;
    let (after, r) = match &state.references {
        Some((after, r)) => (after.as_str(), r),
        None => {
            report = state.root.show_data.identity();
            ("now", &report)
        }
    };
    match problem_count(r) + patch_problems(state).len() {
        0 => format!("ok ({} objects, {} links; checked after {after})", r.objects, r.links),
        n => format!("{n} broken (after {after}) — List Reference /Broken"),
    }
}

/// `List Reference [/Broken]`: checks the show now; lists what's wrong
/// (with `/Broken` only the problems).
/// `List Reference /Broken` (decisions §31: declared on List, Reference is a listed object).
pub const BROKEN: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("Broken", "Only the broken links (List Reference)");

pub fn list(state: &mut AppState, args: &[&str]) -> String {
    let only_broken = crate::commands::flags::has(args, &BROKEN);
    let r = check(state, "List Reference");
    let mut lines = problems(&r);
    lines.extend(patch_problems(state));
    let mut out = vec![
        Style::new().bold().paint("References").to_string(),
        format!("  {} objects, {} links", r.objects, r.links),
    ];
    if lines.is_empty() {
        out.push(Color::Green.paint("  every link resolves").to_string());
        return out.join("\n");
    }
    if !only_broken {
        out.push(Color::Red.paint(format!("  {} broken", lines.len())).to_string());
    }
    out.extend(lines.into_iter().map(|l| format!("  {l}")));
    out.join("\n")
}
