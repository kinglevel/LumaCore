// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of the preset pool on the rig (decisions 0288): Store keeps every active value and picks
//! the mode, At / Call put the values on the selection with the preset's link, Preset n selects
//! its fixtures, the window's cells filter by feature group and tag and fit the selection.
//!
//! Owns:          the unit tests of preset.rs and its folder.
//! Doesn't own:   the pool window's feed (lumacore-views).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0288-one-preset-pool-follows-the-programmer.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_tree::PresetMode;

use crate::AppState;

fn run(state: &mut AppState, line: &str) -> String {
    match crate::commands::dispatch(state, line) {
        crate::DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        crate::DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => String::new(),
    }
}

fn preset(state: &AppState, no: u32) -> lumacore_tree::Preset {
    super::in_pool(state).and_then(|p| p.get(&no)).cloned().unwrap_or_else(|| panic!("Preset {no} isn't stored"))
}

fn prog_dimmer(state: &AppState, fixture: u32) -> Option<lumacore_base::Value> {
    let g = state.root.show_data.fixture_guids(&[fixture]).into_iter().next()?;
    state.current_programmer()?.values.iter().find(|((f, a), _)| *f == g && a.eq_ignore_ascii_case("Dimmer")).map(|(_, v)| v.value.clone())
}

/// The rig's first fixtures that have a Dimmer, past nothing in particular: 101–104 (the rig's).
const RIG: [u32; 4] = [101, 102, 103, 104];

#[test]
fn store_keeps_the_programmer_and_picks_the_mode() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Fixture 101 Thru 104 At 60");
    let out = run(&mut s, "Store Preset 901 \"Half\"");
    assert!(out.contains("Stored Preset 901 \"Half\""), "{out}");
    let p = preset(&s, 901);
    assert!(matches!(p.mode, PresetMode::Universal), "one value for every fixture: Universal ({:?})", p.mode);
    assert_eq!(p.values.len(), 1);
    run(&mut s, "Fixture 101 At 10");
    run(&mut s, "Fixture 102 At 20");
    run(&mut s, "Store Preset 902");
    let p = preset(&s, 902);
    assert!(matches!(p.mode, PresetMode::Selective), "{:?}", p.mode);
    assert!(p.values.iter().all(|e| e.fixture_id.is_some()));
    // A bare Store Preset takes the next free number (the rig has presets of its own).
    let free = crate::pools::door::next_free(&super::in_pool(&s).unwrap().keys().copied().collect::<Vec<_>>());
    let out = run(&mut s, "Store Preset");
    assert!(out.contains(&format!("Stored Preset {free}")), "{out}");
    assert!(run(&mut s, "ResetAll; Store Preset 903").contains("E-CMD-009"), "an empty programmer stores nothing");
}

#[test]
fn at_and_call_put_the_values_on_the_selection_with_the_link() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Fixture 101 Thru 104 At 60");
    run(&mut s, "Store Preset 901");
    run(&mut s, "ResetAll");
    assert!(run(&mut s, "At Preset 901").contains("E-CMD-009"), "nothing selected");
    run(&mut s, "Fixture 103");
    let out = run(&mut s, "At Preset 901");
    assert!(!out.contains("E-"), "{out}");
    let v = prog_dimmer(&s, 103).expect("a value");
    assert_eq!(v.preset, Some(preset(&s, 901).meta.identity.guid), "the value keeps its preset");
    // Call is At (written out before it runs).
    assert_eq!(super::apply::call_is_at(&s, "Call Preset 901"), Some(Ok("At Preset 901".to_string())));
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture 104");
    run(&mut s, "Call Preset 901");
    assert!(prog_dimmer(&s, 104).is_some(), "Call put it on the selection");
    // A line's own target: `Fixture 102 At Preset 901`.
    run(&mut s, "Fixture 102 At Preset 901");
    assert!(prog_dimmer(&s, 102).is_some());
}

#[test]
fn a_bare_preset_selects_its_fixtures() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Fixture 101 At 10");
    run(&mut s, "Fixture 102 At 20");
    run(&mut s, "Store Preset 902");
    run(&mut s, "ResetAll");
    let out = run(&mut s, "Preset 902");
    assert!(out.contains("2 fixtures selected"), "{out}");
    let sel = s.current_programmer().unwrap().selected_fixtures.clone();
    assert_eq!(sel, s.root.show_data.fixture_guids(&[101, 102]));
}

#[test]
fn merge_overwrite_and_remove() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Fixture 101 At 10");
    run(&mut s, "Fixture 102 At 20");
    run(&mut s, "Store Preset 902");
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture 103 At 30");
    let out = run(&mut s, "Store Preset 902 /Merge");
    assert!(out.contains("Merged into Preset 902"), "{out}");
    // Merge: the Dimmer is the programmer's now (its attribute is replaced as a whole).
    assert_eq!(preset(&s, 902).fixtures().count(), 1);
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture 101 Thru 102 At 50");
    run(&mut s, "Store Preset 902 /Overwrite");
    assert!(matches!(preset(&s, 902).mode, PresetMode::Universal));
    run(&mut s, "Store Preset 902 /Remove");
    assert!(preset(&s, 902).values.is_empty());
}

#[test]
fn the_cells_filter_by_feature_group_and_tag_and_fit_the_selection() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Fixture 101 At 10");
    run(&mut s, "Fixture 102 At 20");
    run(&mut s, "Store Preset 902 \"Pair\"");
    let groups = super::pool::groups_of(&s.root.show_data, &preset(&s, 902));
    let dimmer = groups.first().map(|(g, _)| g.clone()).expect("the Dimmer belongs to a feature group");
    let user = s.effective_user_name().to_string();
    let cells = |s: &AppState, feature: Option<&str>, tags: Vec<uuid::Uuid>| {
        let referenced = Default::default();
        let claims = Default::default();
        let ctx = crate::pools::PoolContext {
            show: &s.root.show_data,
            user: &user,
            datapool: s.selected_datapool(),
            claims: &claims,
            referenced: &referenced,
            now_ms: 0,
            profile: None,
            filter: crate::pools::PoolFilter { feature: feature.map(str::to_string), tags },
        };
        (super::pool::PRESET_POOL.cells)(&ctx)
    };
    assert!(cells(&s, Some(&dimmer), vec![]).iter().any(|c| c.no == 902), "it touches {dimmer}");
    assert!(!cells(&s, Some("NoSuchGroup"), vec![]).iter().any(|c| c.no == 902));
    assert!(cells(&s, Some("All"), vec![]).iter().any(|c| c.no == 902));
    // The fit: 101 + 102 selected = all; with 103 = some.
    run(&mut s, "ResetAll");
    run(&mut s, "Fixture 101 + 102");
    let fit = |s: &AppState| cells(s, None, vec![]).into_iter().find(|c| c.no == 902).and_then(|c| c.applicability);
    assert_eq!(fit(&s), Some("all"));
    run(&mut s, "Fixture 101 + 102 + 103");
    assert_eq!(fit(&s), Some("some"));
    // A tag narrows the pool (the Tag words are another crate's: `Assign Tag … At Preset` is the
    // catalog's test; here the preset carries the link).
    let tag = uuid::Uuid::new_v4();
    assert!(!cells(&s, None, vec![tag]).iter().any(|c| c.no == 902));
    let dp = s.selected_datapool();
    s.root.show_data.datapools.get_mut(&dp).unwrap().presets.get_mut(&902).unwrap().meta.annotation.tags.push(tag);
    assert!(cells(&s, None, vec![tag]).iter().any(|c| c.no == 902));
    let _ = RIG;
}
