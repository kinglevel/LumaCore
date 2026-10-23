// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Lines that need words from several command crates, line in → reply out (decisions §34 25.16):
//! the Tab completion over the whole catalog.
//!
//! Owns:          the cross-crate behaviour tests.
//! Doesn't own:   a crate's own words (beside them), the kernel's machinery (its stand-in words).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12b2", §34 25.16.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::AppState;

fn completions(state: &AppState, line: &str) -> Vec<String> {
    lumacore_suggest::complete_input(line, line.chars().count(), state).1
}

#[test]
fn completion_comes_from_the_registries() {
    let state = AppState::new();
    assert!(completions(&state, "Store ").iter().any(|c| c == "Group"));
    let cd = completions(&state, "cd ");
    assert!(
        cd.iter().any(|c| c == "root") && cd.iter().any(|c| c == ".."),
        "{cd:?}"
    );
    assert!(
        completions(&state, "Fixture 1 At ")
            .iter()
            .any(|c| c == "Full")
    );
    assert!(completions(&state, "help sto").iter().any(|c| c == "Store"));
    assert!(completions(&state, "unwatch ").iter().any(|c| c == "all"));
}

/// The rig's presets carry its tag (decisions 0288): `Assign Tag … At Preset` reaches the one
/// preset pool through the resolver, and the Presets window's tag filter reads the link.
#[test]
fn a_preset_carries_the_tag_assigned_at_it() {
    let mut state = lumacore_objects::demo::fresh_state();
    let wash = lumacore_tree::tags::tag::all(&state.root.show_data.tags)
        .into_iter()
        .find(|(_, t)| t.name() == "Wash")
        .map(|(_, t)| t.meta.identity.guid)
        .expect("the rig's Wash tag");
    let tags = |s: &AppState, n: u32| s.root.show_data.datapools[&1].presets[&n].meta.annotation.tags.clone();
    assert_eq!(tags(&state, 1), vec![wash], "the rig tags Preset 1");
    assert!(tags(&state, 3).is_empty());
    let out = match lumacore_objects::commands::dispatch(&mut state, "Assign Tag \"Wash\" At Preset 3") {
        lumacore_objects::DispatchResult::Output(r) => lumacore_objects::text::strip_ansi(&r),
        _ => String::new(),
    };
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(tags(&state, 3), vec![wash]);
}

/// Every built-in key's lines (the core keys, the Programmer's cycles; decisions 0295) use only
/// words the catalog has: a typed press is a fragment of them, every other line a whole one.
#[test]
fn every_built_in_keys_lines_are_the_catalogs_words() {
    let state = lumacore_objects::demo::fresh_state();
    for k in state.root.resources.ui_keys.values() {
        let problems = lumacore_cmd_keys::key_problems(k);
        assert!(problems.is_empty(), "UIKey {}: {problems:?}", k.id);
    }
}
