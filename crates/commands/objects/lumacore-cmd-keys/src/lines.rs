// ██╗     ██╗███╗   ██╗███████╗███████╗
// ██║     ██║████╗  ██║██╔════╝██╔════╝
// ██║     ██║██╔██╗ ██║█████╗  ███████╗
// ██║     ██║██║╚██╗██║██╔══╝  ╚════██║
// ███████╗██║██║ ╚████║███████╗███████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝
// FILE: lines

//! A key's command lines are checked when saved and again on load (decisions §17): an unknown word
//! is refused on save and flagged on load, a planned one warned, a bare number refused (E-SYN-008),
//! an undeclared flag refused (E-CMD-018).
//!
//! Owns:          the checks of a key's command lines: on save (Bind) and after a show replaced the last.
//! Doesn't own:   the parse (lumacore-syntax, the kernel's line), the sysmon log (lumacore-sysmon).
//! Entry points:  check_line, key_problems, check_after_load, check_loaded.
//! Decisions:     §17, §31, §33 21.

use lumacore_tree::TreeNode;
use lumacore_tree::ui_keys::UiKey;

use lumacore_objects::AppState;
use lumacore_objects::commands::registry::{self, Status, Word};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// What is wrong with a command line: one entry per `;` part whose first
/// word is unknown (`Err`) or a planned command (`Ok`, a warning).
pub fn check_line(line: &str) -> Vec<Result<String, String>> {
    // Statements split outside quotes only (R1-71: `Label Group 1 "a;b"`).
    let mut problems: Vec<Result<String, String>> = lumacore_objects::commands::tokens::statements(
        line,
    )
    .into_iter()
    .filter_map(|part| part.split_whitespace().next().map(str::to_string))
    .filter_map(|word| match registry::resolve(&word) {
        // A bare number or name depends on whose default object it runs with (§33 21).
        Err(_)
            if word.parse::<f64>().is_ok()
                || word.starts_with('"')
                || word.eq_ignore_ascii_case("thru") =>
        {
            Some(Err(format!(
                "'{word}' alone — a stored line names its object (`Fixture {word} …`) [E-SYN-008]"
            )))
        }
        Err(_) => Some(Err(format!("unknown word '{word}'"))),
        Ok(Word::Command(c)) if c.status == Status::Planned => {
            Some(Ok(format!("'{}' is planned (not built yet)", c.label())))
        }
        Ok(_) => None,
    })
    .collect();
    // A flag nobody declares would be refused when the key runs (decisions §31, E-CMD-018).
    if let Ok(parsed) = lumacore_objects::commands::line::Line::parse(line) {
        problems.extend(
            parsed
                .unknown_flags()
                .map(|(s, flag)| Err(format!("'{flag}' isn't an option of {}", s.label()))),
        );
    }
    problems
}

/// Every problem of a key's command lines, as `Gesture: problem` lines.
pub fn key_problems(key: &UiKey) -> Vec<String> {
    key.lines()
        .flat_map(|(gesture, meow, line)| {
            check_line(line).into_iter().map(move |p| {
                let p = p.unwrap_or_else(|e| e);
                format!(
                    "{}{}: {p}",
                    if meow { "Meow+" } else { "" },
                    gesture.label()
                )
            })
        })
        .collect()
}

/// After a show loads: flag the profile keys and plugin buttons whose
/// command lines no longer parse (§17 "re-checked on load"). Returns the
/// lines reported.
pub fn check_after_load(state: &AppState) -> Vec<String> {
    let show = &state.root.show_data;
    let profile_keys = show.user_profiles.values().flat_map(|p| {
        p.content
            .ui_keys
            .values()
            .map(move |k| (p.name().unwrap_or("?"), k))
    });
    let plugin_keys = show
        .datapools
        .values()
        .flat_map(|dp| dp.plugins.values())
        .flat_map(|p| p.buttons.iter().map(|k| ("plugin", k)));
    let report: Vec<String> = profile_keys
        .chain(plugin_keys)
        .flat_map(|(owner, key)| {
            key_problems(key)
                .into_iter()
                .map(move |p| format!("UIKey {} ({owner}) {p}", key.id))
        })
        .collect();
    for line in &report {
        lumacore_sysmon::sysmon_note("Keys", line);
    }
    report
}

/// `Hook::ShowReplaced`: the key lines checked after another show (the report goes to sysmon).
pub(crate) fn check_loaded(state: &mut AppState) {
    let _ = check_after_load(state);
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
