//! Issue 07: a quoted word is a value — its quotes never end up in
//! what the show stores — and a bare `/word` is an option.

use super::dispatch;
use crate::AppState;

/// Every string in `v` that holds a quote character, with its path.
fn quoted_strings(v: &serde_json::Value, path: &str, out: &mut Vec<String>) {
    match v {
        serde_json::Value::String(s) if s.contains('"') => out.push(format!("{path} = {s}")),
        serde_json::Value::Array(a) => a.iter().enumerate().for_each(|(i, v)| quoted_strings(v, &format!("{path}[{i}]"), out)),
        serde_json::Value::Object(o) => o.iter().for_each(|(k, v)| quoted_strings(v, &format!("{path}.{k}"), out)),
        _ => {}
    }
}

/// Command lines are stored as typed (keys, macros, window bars): their
/// quotes belong to them.
fn is_command_text(path: &str) -> bool {
    ["command", "line", "lines", "gestures", "meow", "press", "release", "hold"].iter().any(|k| path.contains(&format!(".{k}")))
}

#[test]
fn quoted_names_are_stored_without_their_quotes() {
    let mut lines: Vec<&str> = Vec::new();
    for o in crate::objects::all() {
        lines.extend(o.doc.examples.iter().copied());
    }
    for c in crate::commands::registry::all() {
        lines.extend(c.doc.examples.iter().copied());
    }
    // Naming lines of every kind of object (not every one is an example).
    lines.extend([
        "Store Universe 6 \"Q N\"",
        "Label Universe 1 \"Q N\"",
        "Store Group 4 \"Q N\"",
        "Label Group 4 \"Q N\"",
        "Store User \"Q N\"",
        "Store DataPool 2 \"Q N\"",
        "Label DataPool 1 \"Q N\"",
        "Store Output 3 \"Q N\" /Protocol Sacn /Network \"Q Net\"",
        "Store Input 2 \"Q N\" /Protocol Sacn /Network \"Q Net\"",
        "Set Output 3 \"Network\" \"Q Net 2\"",
        "Store AssetRoot \"Q N\"",
        "Set Show \"Description\" \"Q N\"",
        "Store View 3 \"Q N\"",
        "Store Sequence 2 \"Q N\"",
        "Store Cue 1 \"Q N\"",
        "Label Fixture 1 \"Q N\"",
        "Set Fixture 1 \"Name\" \"Q N\"",
        "Label FixtureType 1 \"Q N\"",
        "Store Preset 1.1 \"Q N\"",
        "Store Macro 1 \"Q N\"",
        "Store UserProfile \"Q N\"",
        "Store ScreenConfig 2 \"Q N\"",
        "Store Timecode 1 \"Q N\"",
        "Store Executor 1 \"Q N\"",
        "Store WindowPreset 1 \"Q N\"",
        "Store Network \"Q N\"",
    ]);
    let mut found = Vec::new();
    for raw in lines {
        let line = raw.split("--").next().unwrap_or("").trim();
        if !line.contains('"') {
            continue;
        }
        let mut state: AppState = super::playback_tests::rig(&[1, 2]);
        let _ = dispatch(&mut state, "Claim Root.ShowData.Devices");
        let _ = dispatch(&mut state, "Fixture 1");
        lumacore_luma::drive::set_off(true);
        let _ = lumacore_sysmon::panic_report::guarded(|| dispatch(&mut state, line));
        lumacore_luma::drive::set_off(false);
        let json = serde_json::to_value(&state.root.show_data).unwrap();
        let mut strings = Vec::new();
        quoted_strings(&json, "ShowData", &mut strings);
        found.extend(strings.into_iter().filter(|p| !is_command_text(p)).map(|p| format!("`{line}`: {p}")));
    }
    assert!(found.is_empty(), "stored with quotes:\n{}", found.join("\n"));
}

#[test]
fn a_quoted_flag_is_a_name_and_a_bare_path_is_an_option() {
    let mut state: AppState = super::playback_tests::rig(&[1]);
    let run = |state: &mut AppState, line: &str| match dispatch(state, line) {
        crate::DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => String::new(),
    };
    run(&mut state, "Fixture 1");
    run(&mut state, "Store Group 3");
    run(&mut state, "Label Group 3 \"/nc\"");
    assert_eq!(state.user_groups()[&3].meta.identity.name.as_deref(), Some("/nc"), "a quoted flag is text");
    let out = run(&mut state, "Label Group 3 Wash Front");
    // A name with spaces is quoted: bare words are words (§33), refused as unknown.
    assert!(out.contains("E-SYN-001") && out.contains("Wash"), "{out}");
    run(&mut state, "Label Group 3 \"Front; Back\"");
    assert_eq!(state.user_groups()[&3].meta.identity.name.as_deref(), Some("Front Back"), "one statement; `;` isn't a name character");
    // A bare /word is never a file; a path stays inside its library folder.
    lumacore_luma::drive::set_off(true);
    let out = run(&mut state, "Export Fixture 1 /user");
    assert!(out.contains("E-CMD-018"), "an undeclared flag (decisions §31): {out}");
    for path in ["\"/tmp/x.json\"", "\"../x.json\"", "\"C:/x.json\""] {
        let out = run(&mut state, &format!("Export Fixture 1 {path}"));
        assert!(out.contains("E-CMD-008"), "{path}: {out}");
    }
    run(&mut state, "Claim Root.ShowData.Devices");
    let out = run(&mut state, "Import FixtureType \"/etc/passwd\"");
    assert!(out.contains("E-CMD-008"), "{out}");
    lumacore_luma::drive::set_off(false);
}

/// R1-33: names in quotes select and store (`Group "Wash"`, `Fixture "Spot 1"`,
/// `Store Group "Front"`); a quoted `"/v"` or `"/watch"` is no flag.
#[test]
fn quoted_names_select_and_store() {
    let mut state: AppState = super::playback_tests::rig(&[1, 2]);
    let run = |state: &mut AppState, line: &str| match dispatch(state, line) {
        crate::DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => String::new(),
    };
    run(&mut state, "Label Fixture 2 \"Spot 1\"");
    run(&mut state, "Fixture \"Spot 1\"");
    let selected = |s: &AppState| s.current_programmer().map(|p| p.selected_fixtures.len()).unwrap_or(0);
    assert_eq!(selected(&state), 1, "Fixture \"Spot 1\" selects it");
    let out = run(&mut state, "Store Group \"Front\"");
    assert!(!out.contains("Error"), "{out}");
    let front = state.user_groups().iter().find(|(_, g)| g.meta.identity.name.as_deref() == Some("Front")).map(|(n, _)| *n);
    assert!(front.is_some(), "stored and named: {out}");
    run(&mut state, "ResetAll");
    let out = run(&mut state, "Group \"Front\"");
    assert_eq!(selected(&state), 1, "Group \"Front\" selects it: {out}");
    // A quoted flag is text: here a name nothing has.
    let out = run(&mut state, "Group \"/watch\"");
    assert!(!out.contains("Watching"), "{out}");
}

