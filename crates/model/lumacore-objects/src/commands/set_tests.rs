//! `Set` on the one property registry (decisions §33 7, 14, 16, 17, 57, 58).

use crate::commands::dispatch;
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        DispatchResult::Ask(a) => panic!("'{line}' asked: {} — {}", a.title(), a.view().text),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn demo() -> AppState {
    crate::demo::fresh_state()
}

#[test]
fn a_property_is_named_quoted() {
    let mut state = demo();
    let out = run(&mut state, "Set Fixture 101 \"Name\" \"Spot SR\"");
    assert!(out.contains("Name = \"Spot SR\""), "{out}");
    let out = run(&mut state, "Set Fixture 101 Name \"Spot SR\"");
    assert!(out.contains("E-SYN-005"), "a bare property name is refused (decision 7): {out}");
    assert!(run(&mut state, "Set Fixture 101 \"Nope\" 1").contains("E-SET-001"));
}

#[test]
fn a_tree_path_and_an_owner_name_the_row() {
    let mut state = demo();
    let out = run(&mut state, "Set Root.ShowData.DataPools.1.Groups.2 \"Note\" \"odd ones\"");
    assert!(!out.contains("Error"), "{out}");
    assert_eq!(state.root.show_data.datapools[&1].groups[&2].meta.annotation.note.as_deref(), Some("odd ones"));
    run(&mut state, "Set DataPool 1 Group 3 \"Note\" \"even\"");
    assert_eq!(state.root.show_data.datapools[&1].groups[&3].meta.annotation.note.as_deref(), Some("even"));
}

#[test]
fn a_field_it_has_but_doesnt_declare_is_read_only() {
    let mut state = demo();
    let out = run(&mut state, "Set Fixture 101 \"Patch\" 1");
    assert!(out.contains("E-SET-002") || out.contains("E-SET-001"), "{out}");
}

/// Decisions §33 86, 88: putting attributes into groups is Assign, Sort / Split are verbs, the
/// rest are properties — on the show's grouping, or under `UserProfile` your own.
#[test]
fn the_attribute_words_are_verbs_and_properties() {
    let mut s = demo();
    // ColorAdd_W is ColorAdd_WW's main attribute on the rig: a bare move asks about the sub.
    assert!(matches!(dispatch(&mut s, "Assign Attribute \"ColorAdd_W\" At FeatureGroup \"FX\""), DispatchResult::Ask(_)));
    let r = run(&mut s, "Assign Attribute \"ColorAdd_W\" At FeatureGroup \"FX\" /WithSubs");
    assert!(!r.contains("E-"), "{r}");
    let r = run(&mut s, "Sort FeatureGroup \"FX\" /By Name");
    assert!(!r.contains("E-"), "{r}");
    assert!(run(&mut s, "Sort FeatureGroup \"FX\"").contains("E-CMD-006"), "a key is needed");
    assert!(matches!(dispatch(&mut s, "Split FeatureGroup \"FX\" /By Family"), DispatchResult::Ask(_) | DispatchResult::Output(_)));
    let r = run(&mut s, "Set FeatureGroup \"FX\" \"Paint\" \"#b07be6\"");
    assert!(!r.contains("E-"), "{r}");
    let r = run(&mut s, "Set Attribute \"ColorAdd_W\" \"Readout\" Physical");
    assert!(!r.contains("E-"), "{r}");
    let r = run(&mut s, "Set UserProfile FeatureGroup \"FX\" \"Icon\" \"sparkles\"");
    assert!(!r.contains("E-"), "your own grouping: {r}");
    assert!(run(&mut s, "Set Attribute \"ColorAdd_W\" FeatureGroup \"FX\"").contains("E-"), "the old form is gone");
}

/// Decision 89: a template is applied with Call, onto the show's grouping or your own.
#[test]
fn a_template_is_called_at_a_layer() {
    let mut s = demo();
    assert!(run(&mut s, "List AttributeTemplate").contains("LumaCore FX"));
    let DispatchResult::Ask(ask) = dispatch(&mut s, "Call AttributeTemplate \"LumaCore FX\" At ShowData") else { panic!("asks") };
    let line = ask.line_of("Apply").unwrap();
    assert_eq!(line, "Call AttributeTemplate \"LumaCore FX\" At ShowData /NoConfirmation");
    assert!(!run(&mut s, &line).contains("E-"));
    assert!(!run(&mut s, "Call AttributeTemplate \"GDTF standard\" At UserProfile /NoConfirmation").contains("E-"));
    assert!(run(&mut s, "Call AttributeTemplate \"Nope\" At ShowData").contains("E-CMD-007"));
}
