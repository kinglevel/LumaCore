// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The protocol objects with the line words (decisions §34 22, "34f7b hardening", "34f7c hardened"):
//! a Set over many lines, relative values, the on/off words, the protocol names and their settings.
//!
//! Owns:          the cross-crate protocol tests.
//! Doesn't own:   each crate's own words' tests.
//! Entry points:  the #[test] fns.
//! Decisions:     §34 22, "34f7b hardening".

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

fn rate(state: &AppState, n: u32) -> u32 {
    u32::from(state.root.show_data.protocols.line("Sacn", n).unwrap().rate)
}

fn steps(state: &AppState) -> usize {
    state.undo_stack("Admin").map_or(0, |u| u.frames.len())
}

fn with_lines(n: u32) -> AppState {
    let mut state = AppState::new();
    for i in 1..=n {
        let out = run(&mut state, &format!("Store Sacn {i}"));
        assert!(!out.contains("Error"), "{out}");
    }
    state
}

fn demo() -> AppState {
    lumacore_objects::demo::fresh_state()
}

/// A typed Store reads its slot, name and options (decisions §33 3d, §34 "34f7c hardened" 7).
#[test]
fn a_typed_store_reads_number_name_and_options() {
    let mut state = AppState::new();
    let out = run(&mut state, "Store ArtNet 2 \"Out to rig\" /Network \"Rig\" /LocalUniverse 3 /Amount 2 /PortAddress 0.1.0");
    let line = state.root.show_data.protocols.line("ArtNet", 2).unwrap();
    assert_eq!(line.meta.identity.name.as_deref(), Some("Out to rig"), "{out}");
    assert_eq!(line.protocol, "ArtNet");
    assert_eq!((line.local_universe, line.amount, line.universe), (3, 2, 0x10));
    assert_eq!(state.root.show_data.protocols.network_name(line.network), "Rig");
    assert!(out.contains("3–4 → 0.1.0–0.1.1"), "{out}");
}

/// A protocol is an object word, typed in any case and written out as its row declares it
/// (decisions §34 "34f7c hardened" 1); a protocol this build lacks is no word at all.
#[test]
fn a_protocol_is_an_object_named_by_its_row() {
    let mut state = AppState::new();
    run(&mut state, "Store artnet 1");
    run(&mut state, "Store SACN 1");
    assert!(state.root.show_data.protocols.line("ArtNet", 1).is_some() && state.root.show_data.protocols.line("Sacn", 1).is_some());
    let out = run(&mut state, "Store Dmx512 1");
    assert!(out.contains("E-CMD-006") && out.contains("ArtNet, Sacn, Osc"), "{out}");
    // Lines live in their kind's group.
    assert!(state.root.show_data.protocols.dmx_protocols.contains_key("ArtNet"));
    run(&mut state, "Store Osc 1 /User \"Admin\"");
    assert!(state.root.show_data.protocols.data_protocols.contains_key("Osc"));
    // A protocol has no way to become another (decisions §34 "34f7c hardened" 8).
    assert!(run(&mut state, "Move Sacn 1 At ArtNet 5").contains("E-"), "a move stays in its protocol");
    // Bare, the word names the protocol: Info reads what it is and can do.
    let info = run(&mut state, "Info ArtNet");
    assert!(info.contains("DMXProtocols") && info.contains("PortAddress") && info.contains("Discovery"), "{info}");
}

/// A protocol's own settings: `/Port` and `Set … "Priority"` exist only on a protocol that declares
/// them (decisions §34 "34f7c hardened" 7).
#[test]
fn a_protocol_setting_goes_only_to_its_protocols_lines() {
    let mut state = AppState::new();
    run(&mut state, "Store Sacn 1");
    run(&mut state, "Store ArtNet 1");
    assert!(run(&mut state, "Set Sacn 1 \"Priority\" 150").contains("150"));
    assert_eq!(state.root.show_data.protocols.line("Sacn", 1).unwrap().setting_u64("Priority"), Some(150));
    let refused = run(&mut state, "Set ArtNet 1 \"Priority\" 150");
    assert!(refused.contains("E-SET-"), "{refused}");
    assert!(state.root.show_data.protocols.line("ArtNet", 1).unwrap().settings.is_empty());
    // An unset setting reads the row's default.
    run(&mut state, "Store Sacn 3");
    assert_eq!(state.root.show_data.protocols.line("Sacn", 3).unwrap().setting_u64("Priority"), Some(100));
    // OSC declares its Port; sACN takes no /Port at all.
    run(&mut state, "Store Osc 1 /Port 9000 /User \"Admin\"");
    assert_eq!(state.root.show_data.protocols.line("Osc", 1).unwrap().setting_u64("Port"), Some(9000));
    assert!(run(&mut state, "Store Sacn 2 /Port 9000").contains("E-"));
    assert!(state.root.show_data.protocols.line("Sacn", 2).is_none());
}

#[test]
fn one_set_line_names_many_rows() {
    let mut state = with_lines(4);
    let before = steps(&state);
    let out = run(&mut state, "Set Sacn 1 Thru 3 \"Rate\" 44");
    assert!(out.contains("Sacn 1, Sacn 2, Sacn 3: Rate = 44"), "{out}");
    assert_eq!((rate(&state, 1), rate(&state, 3), rate(&state, 4)), (44, 44, 30));
    assert_eq!(steps(&state), before + 1, "one line = one Oops step");
    // A spread, relative per row, a list.
    run(&mut state, "Set Sacn 1 Thru 4 \"Rate\" 20 Thru 50");
    assert_eq!((1..=4).map(|n| rate(&state, n)).collect::<Vec<_>>(), vec![20, 30, 40, 50]);
    run(&mut state, "Set Sacn 1 + 4 \"Rate\" + 5");
    assert_eq!((rate(&state, 1), rate(&state, 4)), (25, 55));
    // Numbers that don't exist are skipped (the one range rule); none at all = no such row.
    let out = run(&mut state, "Set Sacn 1 + 9 \"Rate\" 30");
    assert!(out.contains("Sacn 1 Rate = 30"), "{out}");
    assert!(run(&mut state, "Set Sacn 8 + 9 \"Rate\" 30").contains("E-CMD-007"));
    assert!(run(&mut state, "Set Sacn 1 + 2 \"Rate\" 99").contains("E-SET-003"));
    assert!(run(&mut state, "Set Sacn a Thru b \"Rate\" 9").contains("E-SYN-001"), "ids are numbers or quoted");
}

#[test]
fn many_rows_spread_and_relative_values() {
    let mut state = demo();
    for n in 2..=4 {
        run(&mut state, &format!("Store Sacn {n}"));
    }
    let out = run(&mut state, "Set Sacn 1 Thru 4 \"Rate\" 21 Thru 30");
    assert!(!out.contains("Error"), "{out}");
    let rates: Vec<u32> = (1..=4).map(|n| rate(&state, n)).collect();
    assert_eq!(rates, [21, 24, 27, 30]);
    run(&mut state, "Set Sacn 1 \"Rate\" * 2");
    assert_eq!(rate(&state, 1), 42);
    run(&mut state, "Set Sacn 4 Thru 2 + 1 \"Rate\" 10");
    assert!((1..=4).all(|n| rate(&state, n) == 10));
}

#[test]
fn on_off_takes_four_words_and_nothing_else() {
    let mut state = demo();
    for word in ["On", "off", "Yes", "NO"] {
        assert!(!run(&mut state, &format!("Set Sacn 1 \"Sync\" {word}")).contains("Error"), "{word}");
    }
    for word in ["1", "true", "\"pinned\""] {
        let out = run(&mut state, &format!("Set Sacn 1 \"Sync\" {word}"));
        assert!(out.contains("E-SYN-014") || out.contains("E-SYN-001"), "{word}: {out}");
    }
}

/// `Store Sacn` offers its line options after a slash (the completion reads the declared flags).
#[test]
fn store_line_completes_its_options() {
    let state = AppState::new();
    let line = "Store Sacn 2 /";
    let offered = lumacore_suggest::grammar::complete_input(line, line.chars().count(), &state).1;
    assert!(offered.contains(&"/SacnUniverse".to_string()) && offered.contains(&"/LocalUniverse".to_string()), "{offered:?}");
    let line = "Store Osc 2 /";
    let offered = lumacore_suggest::grammar::complete_input(line, line.chars().count(), &state).1;
    assert!(offered.contains(&"/Port".to_string()) && !offered.contains(&"/SacnUniverse".to_string()), "{offered:?}");
}
