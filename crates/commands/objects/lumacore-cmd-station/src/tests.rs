// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The station's words line in, reply out: the clock, Watch / Unwatch and `/watch`, which lines
//! may be watched, the journal, the Server's About settings, HissMe's level.
//!
//! Owns:          the tests of the station's words.
//! Doesn't own:   the kernel's machinery (its stand-in words, decisions §34 25.16).
//! Entry points:  the #[test] fns.
//! Decisions:     §29, §34 25.7.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::commands::dispatch;
use lumacore_objects::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn values(state: &AppState) -> usize {
    state.current_programmer().unwrap().values.len()
}

/// One dimmer fixture on universe 1 (as the kernel's coverage tests make it).
fn with_fixture() -> AppState {
    let mut state = AppState::new();
    let mut ev = Vec::new();
    lumacore_tree::UniverseOps::create_universe(&mut state.root.show_data, None, &mut ev);
    let ft = lumacore_fixtures::fixture_type::FixtureType {
        meta: lumacore_tree::node::NodeMeta::new(Some("Dim".into())),
        manufacturer: "M".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: vec![lumacore_fixtures::fixture_type::Revision {
            meta: lumacore_tree::node::NodeMeta::new(None),
            geometries: Vec::new(),
            wheels: Vec::new(),
            emitters: Vec::new(),
            filters: Vec::new(),
            attributes: Vec::new(),
            gdtf_version: "1.2".into(),
            file_hash: String::new(),
            modes: vec![lumacore_fixtures::fixture_type::Mode {
                meta: lumacore_tree::node::NodeMeta::new(None),
                channel_functions: vec![lumacore_fixtures::fixture_type::ChannelFunction {
                    attribute_name: "Dimmer".into(),
                    attribute_type: lumacore_fixtures::fixture_type::AttributeType::Dimmer,
                    byte_count: 1,
                    resolved: true,
                    ..Default::default()
                }],
                dmx_footprint: 1,
                geometry: String::new(),
                cells: Vec::new(),
                relations: Vec::new(),
            }],
        }],
    };
    let guid = ft.meta.identity.guid;
    let show = &mut state.root.show_data;
    show.devices
        .dmx
        .patch
        .fixture_types
        .insert(1, std::sync::Arc::new(ft));
    lumacore_patch::PatchEngine::patch_fixture(show, guid, 0, 0, 1, 1, None).unwrap();
    // Like an import: the type's attributes are defined with it.
    lumacore_tree::attribute_resolver::sync(show);
    state
}

#[test]
fn clock_returns_time_format() {
    let mut state = AppState::new();
    match dispatch(&mut state, "clock") {
        DispatchResult::Output(s) => {
            // Should match HH:MM:SS.mmm
            assert!(s.contains(':'), "clock output should contain ':': {s}");
            assert!(s.contains('.'), "clock output should contain '.': {s}");
            assert!(s.len() >= 12, "clock output too short: {s}");
        }
        other => panic!("Expected Output, got {:?}", std::mem::discriminant(&other)),
    }
}

#[test]
fn clock_ticks_between_calls() {
    let mut state = AppState::new();
    let t1 = match dispatch(&mut state, "clock") {
        DispatchResult::Output(s) => s,
        _ => panic!("Expected Output"),
    };
    // Sleep briefly to ensure time advances
    std::thread::sleep(std::time::Duration::from_millis(5));
    let t2 = match dispatch(&mut state, "clock") {
        DispatchResult::Output(s) => s,
        _ => panic!("Expected Output"),
    };
    // At minimum, milliseconds should differ (or seconds if we're unlucky at boundary)
    // We can't guarantee they differ if both land in the same ms, so sleep 5ms above
    assert_ne!(t1, t2, "Clock should tick between calls: t1={t1} t2={t2}");
}

#[test]
fn watch_registers_and_unwatch_clears() {
    let mut state = AppState::new();
    assert!(state.watches().is_empty());

    // Register via watch command
    let result = dispatch(&mut state, "watch clock");
    match result {
        DispatchResult::Output(s) => assert!(s.contains("Watching"), "Expected success: {s}"),
        _ => panic!("Expected Output"),
    }
    assert_eq!(state.watches().len(), 1);
    assert_eq!(state.watches()[0].command, "clock");
    assert_eq!(state.watches()[0].interval_ms, 200);

    // Duplicate rejected
    let result = dispatch(&mut state, "watch clock");
    match result {
        DispatchResult::Output(s) => {
            assert!(s.contains("already"), "Expected duplicate error: {s}")
        }
        _ => panic!("Expected Output"),
    }
    assert_eq!(state.watches().len(), 1);

    // Unwatch clears
    let result = dispatch(&mut state, "unwatch");
    match result {
        DispatchResult::Output(s) => assert!(s.contains("Stopped"), "Expected success: {s}"),
        _ => panic!("Expected Output"),
    }
    assert!(state.watches().is_empty());
}

#[test]
fn watch_flag_on_command() {
    let mut state = AppState::new();

    // clock /watch should register
    let result = dispatch(&mut state, "clock /watch");
    match result {
        DispatchResult::Output(s) => assert!(s.contains("Watching"), "Expected success: {s}"),
        _ => panic!("Expected Output"),
    }
    assert_eq!(state.watches().len(), 1);
    assert_eq!(state.watches()[0].command, "clock");
}

#[test]
fn watch_custom_interval() {
    let mut state = AppState::new();

    // Use handle_watch directly to avoid try_watch intercepting the /watch flag
    let result = crate::handle_watch(&mut state, &["clock", "/watch=500"]);
    assert!(result.contains("500ms"), "Expected 500ms: {result}");
    assert_eq!(state.watches()[0].interval_ms, 500);
}

#[test]
fn watch_interval_floor_at_50ms() {
    let mut state = AppState::new();

    let result = dispatch(&mut state, "watch clock /watch=10");
    match result {
        DispatchResult::Output(_) => {}
        _ => panic!("Expected Output"),
    }
    assert!(
        state.watches()[0].interval_ms >= 50,
        "Interval should floor at 50ms"
    );
}

#[test]
fn watch_empty_output_rejected() {
    let mut state = AppState::new();

    // "shutdown" produces Exit, not Output — /watch should reject it
    let result = dispatch(&mut state, "watch shutdown");
    match result {
        DispatchResult::Output(s) => assert!(
            s.contains("no output")
                || s.contains("Only commands")
                || s.contains("only read-only lines"),
            "Expected rejection: {s}"
        ),
        _ => panic!("Expected Output"),
    }
    assert!(state.watches().is_empty());
}

#[test]
fn unwatch_by_index() {
    let mut state = AppState::new();
    dispatch(&mut state, "watch clock");
    dispatch(&mut state, "watch list health");
    assert_eq!(state.watches().len(), 2);

    let result = dispatch(&mut state, "unwatch 1");
    match result {
        DispatchResult::Output(s) => {
            assert!(s.contains("clock"), "Should name removed watch: {s}")
        }
        _ => panic!("Expected Output"),
    }
    assert_eq!(state.watches().len(), 1);
    assert_eq!(state.watches()[0].command, "list health");
}

#[test]
fn unwatch_by_name() {
    let mut state = AppState::new();
    dispatch(&mut state, "watch clock");

    let result = dispatch(&mut state, "unwatch clock");
    match result {
        DispatchResult::Output(s) => assert!(s.contains("Stopped"), "Expected success: {s}"),
        _ => panic!("Expected Output"),
    }
    assert!(state.watches().is_empty());
}

#[test]
fn clear_buffer_forced_by_watches() {
    let mut state = AppState::new();
    assert!(!state.clear_buffer_active());

    dispatch(&mut state, "watch clock");
    assert!(
        state.clear_buffer_active(),
        "Watches should force clear_buffer on"
    );

    dispatch(&mut state, "unwatch");
    assert!(
        !state.clear_buffer_active(),
        "No watches should restore clear_buffer"
    );
}

#[test]
fn watch_re_executes_clock() {
    // Simulate what render_watches does: dispatch the watched command
    // and verify it produces fresh output each time
    let mut state = AppState::new();
    dispatch(&mut state, "watch clock");

    let t1 = match dispatch(&mut state, "clock") {
        DispatchResult::Output(s) => s,
        _ => panic!("Expected Output"),
    };
    std::thread::sleep(std::time::Duration::from_millis(5));
    let t2 = match dispatch(&mut state, "clock") {
        DispatchResult::Output(s) => s,
        _ => panic!("Expected Output"),
    };
    assert_ne!(
        t1, t2,
        "Watch should get fresh clock output: t1={t1} t2={t2}"
    );
}

#[test]
fn resolve_verb_prefix_uniq_to_unwatch() {
    // "unwa" should uniquely match "unwatch"
    assert_eq!(
        lumacore_objects::commands::resolve_verb("unwa").unwrap(),
        "unwatch"
    );
}

/// R1-65: only read-only lines can be watched; a refused line never ran.
#[test]
fn only_read_only_lines_can_be_watched() {
    let mut state = with_fixture();
    for line in [
        "Watch Store Group 5",
        "Fixture 1 At 50 /watch",
        "Watch Go Sequence 1",
    ] {
        let out = run(&mut state, line);
        assert!(
            out.contains("E-CMD-010") && out.contains("read-only"),
            "{line}: {out}"
        );
    }
    assert_eq!(values(&state), 0, "the refused line never ran");
    assert!(state.watches().is_empty());
    assert!(run(&mut state, "Watch Clock").contains("Watching"));
    assert!(run(&mut state, "Watch Fixture").contains("Watching"));
    // An object is watched as its List (`Watch Fixture 1 At 50` lists).
    assert!(run(&mut state, "Watch Fixture 1 At 50").contains("list Fixture 1 At 50"));
    assert!(crate::watchable("List Fixture") && crate::watchable("Info Clock"));
    assert!(!crate::watchable("At 50") && !crate::watchable("List Fixture; Store Group 4"));
}

#[test]
fn watch_is_one_word_for_starting_and_listing() {
    let mut state = AppState::new();
    let started = run(&mut state, "watch clock");
    assert!(!started.contains("Unknown"), "{started}");
    assert_eq!(state.watches().len(), 1, "watch clock should start a watch");
    let listed = run(&mut state, "list watch");
    assert!(listed.to_lowercase().contains("clock"), "{listed}");
}

#[test]
fn watches_are_ui_steps() {
    let mut state = AppState::new();
    run(&mut state, "Watch Clock");
    assert_eq!(state.watches().len(), 1);
    assert_eq!(
        state
            .undo_stack("Admin")
            .and_then(|u| u.frames.back())
            .map(|f| f.category),
        Some(lumacore_tree::OopsCategory::UI)
    );
    run(&mut state, "Oops");
    assert!(state.watches().is_empty());
    run(&mut state, "Redo");
    assert_eq!(state.watches().len(), 1);
}

/// List Journal shows what changed; Watch Events starts a watch on it.
#[test]
fn the_journal_lists_and_watches_what_changed() {
    let mut state = AppState::new();
    run(&mut state, "Store Network \"Rig\"");
    let listed = run(&mut state, "list journal");
    assert!(listed.contains("Store Network \"Rig\""), "{listed}");
    run(&mut state, "watch events");
    assert_eq!(
        state.watches().len(),
        1,
        "watch events should start a watch"
    );
}

/// Settings › About: the Server's read-only facts (EditSetting Server).
#[test]
fn the_servers_about_settings_are_read_only() {
    let mut state = AppState::new();
    let about = run(&mut state, "EditSetting Server");
    assert!(
        about.contains("ShowFormat") && about.contains(&lumacore_tree::FORMAT_VERSION.to_string()),
        "{about}"
    );
    assert!(run(&mut state, "Set Server \"ShowFormat\" 3").contains("read-only"));
}

/// The station's words are in the completion: clock, watch, unwatch, the stopwatch alias.
#[test]
fn the_station_words_complete() {
    let keywords = lumacore_objects::commands::completion_keywords();
    for w in ["clock", "watch", "unwatch", "stopwatch"] {
        assert!(keywords.contains(&w.to_string()), "Missing {w}");
    }
}

/// HissMe is the station's own (Rights::Internal): not even Admin runs it.
#[test]
fn hissme_is_internal() {
    use lumacore_objects::commands::registry::Rights;
    assert!(lumacore_objects::rights::authorize(Rights::Admin, "hissme").is_err());
}

/// Shell lists the terminal sessions (the LumaSession is another word).
#[test]
fn shell_lists_the_terminal_sessions() {
    let mut state = AppState::new();
    let out = run(&mut state, "List Shell");
    assert!(out.contains("Users") || out.contains("No runtime"), "{out}");
}

/// Issues 05 / 06: Show settings (`Set Show …`, decisions §33 46) and Restart / Shutdown
/// change ShowSettings, so they run the change path (replicated, no Oops step) and go to
/// the master.
#[test]
fn show_options_and_power_words_are_untracked_not_read_only() {
    use lumacore_objects::commands::registry::{Scope, Undo, line_scope};
    for line in ["Restart /NoSave", "Shutdown /NoSave"] {
        assert!(
            matches!(
                lumacore_objects::commands::undo_of_line(line),
                Undo::Untracked
            ),
            "{line}"
        );
    }
    assert_eq!(
        line_scope("Set Show \"Description\" \"Night 2\""),
        Scope::Master
    );
}

/// `turnoff` is Shutdown.
#[test]
fn turnoff_is_shutdown() {
    use lumacore_objects::commands::registry::{Word, lookup_any, resolve};
    assert!(matches!(resolve("turnoff"), Ok(Word::Command(c)) if c.verb == "shutdown"));
    assert_eq!(lookup_any("turnoff").map(|c| c.verb), Some("shutdown"));
}
