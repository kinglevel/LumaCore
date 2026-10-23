// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The lines' words line in, reply out: `DMX … /Input`, the completion of DMX.
//!
//! Owns:          the tests of the lines' words.
//! Doesn't own:   lines that need the protocol rows (tests/catalog protocols, lines).
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §33 58.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::commands::dispatch;
use lumacore_objects::{AppState, DispatchResult};
use lumacore_protocols::InputPriority;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn with_universes(n: u32) -> AppState {
    let mut state = AppState::new();
    let mut ev = Vec::new();
    for _ in 0..n {
        lumacore_tree::UniverseOps::create_universe(&mut state.root.show_data, None, &mut ev);
    }
    state
}

#[test]
fn protocol_dmx_in_reads_what_the_input_lines_bring() {
    let mut state = with_universes(1);
    assert!(run(&mut state, "DMX 1 /Input").contains("nothing received"));
    let hub = lumacore_render::RenderHub::new(
        state.engine_input(std::sync::Arc::new(state.root.show_data.clone())),
    );
    let mut data = [0u8; 512];
    data[4] = 77;
    hub.set_inputs(
        1,
        std::sync::Arc::new(vec![lumacore_protocols::InputLayer {
            line: 1,
            universe: 1,
            band: InputPriority::Ltp,
            data,
            stamps: std::sync::Arc::new([1; 512]),
        }]),
    );
    state.playback = Some(hub);
    assert!(run(&mut state, "DMX 1.5 /Input").contains("= 77"));
    assert!(run(&mut state, "DMX 1 /Input").contains("5:77"));
    assert!(run(&mut state, "DMX 9 /Input").contains("does not exist"));
}

/// `DMX` completes, by its verb and its display name.
#[test]
fn dmx_completes() {
    let keywords = lumacore_objects::commands::completion_keywords();
    assert!(keywords.contains(&"dmx".to_string()) && keywords.contains(&"DMX".to_string()));
}
