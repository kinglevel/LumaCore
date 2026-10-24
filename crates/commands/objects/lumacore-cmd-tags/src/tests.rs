// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of the Tag object's words, against a show of its own.
//!
//! Owns:          the crate's tests.
//! Doesn't own:   the tree's tests (lumacore-tags), lines across crates (tests/catalog).
//! Entry points:  the #[test] fns.
//! Decisions:     §12 "Tags hardened".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::AppState;

fn run(state: &mut AppState, line: &str) -> String {
    match lumacore_objects::commands::dispatch(state, line) {
        lumacore_objects::DispatchResult::Output(r) => String::from(r),
        lumacore_objects::DispatchResult::Ask(a) => format!("ASK {}", a.view().text),
        _ => String::new(),
    }
}

/// The demo show, built once per test binary.
fn demo() -> lumacore_tree::ShowData {
    lumacore_objects::demo::fresh_state().root.show_data
}

/// The demo show (fixtures 101–106) with All › Lx1 › Right and All › Lx2 › Right.
fn rig() -> AppState {
    let mut s = AppState::new();
    s.root.show_data = demo();
    s.root.show_data.after_load();
    for line in [
        "Store Tag 1 \"All\"",
        "Store Tag 1 Tag \"Lx1\"",
        "Store Tag 1.1 Tag \"Right\"",
        "Store Tag 1 Tag \"Lx2\"",
        "Store Tag 1.2 Tag \"Right\"",
    ] {
        run(&mut s, line);
    }
    s
}

#[test]
fn store_builds_the_tree_by_path_and_owner() {
    let s = rig();
    let tags = &s.root.show_data.tags;
    let p = |t: &str| lumacore_tags::TagPath::parse(t).unwrap();
    assert_eq!(lumacore_tags::tag::find(tags, &p("1.1.1")).unwrap().name(), "Right");
    assert_eq!(lumacore_tags::tag::find(tags, &p("1.2")).unwrap().name(), "Lx2");
}

#[test]
fn a_name_under_two_parents_needs_its_owner() {
    let mut s = rig();
    let out = run(&mut s, "Info Tag \"Right\"");
    assert!(out.contains("E-TAG-006"), "{out}");
    let out = run(&mut s, "Info Tag \"Lx1\" Tag \"Right\"");
    assert!(out.contains("Tag 1.1.1"), "{out}");
}

#[test]
fn assign_tags_fixtures_and_a_tag_selects_them() {
    let mut s = rig();
    let out = run(&mut s, "Assign Tag 1.1.1 At Fixture 101 Thru 102");
    assert!(out.contains("Tagged 2"), "{out}");
    run(&mut s, "Assign Tag 1.1 At Fixture 103");
    // Lx1 alone reaches fixture 3; with /R also Right's.
    let out = run(&mut s, "Tag 1.1");
    assert!(out.starts_with("1 fixture"), "{out}");
    let out = run(&mut s, "Tag 1.1 /R");
    assert!(out.starts_with("3 fixtures"), "{out}");
    let out = run(&mut s, "Unassign Tag 1.1.1 At Fixture 102");
    assert!(out.contains("Untagged 1"), "{out}");
}

/// The fixture's side: `Unassign Fixture … At Tag <path>` takes that tag off, a bare `At Tag` every
/// tag (decisions §34 round 3 19); tags are a live column, no claim.
#[test]
fn unassign_fixture_at_tag_names_the_tag_or_every_tag() {
    let mut s = rig();
    run(&mut s, "Assign Tag 1.1.1 At Fixture 101 Thru 102");
    run(&mut s, "Assign Tag 1.2 At Fixture 101");
    let out = run(&mut s, "Unassign Fixture 101 At Tag 1.1.1");
    assert!(out.contains("Fixture 101 lost the tag"), "{out}");
    assert_eq!(s.root.show_data.devices.dmx.patch.fixture(101).expect("the fixture").meta.annotation.tags.len(), 1, "1.2 stays");
    let out = run(&mut s, "Unassign Fixture 101 Thru 102 At Tag");
    assert!(out.contains("lost the tags"), "{out}");
    assert!(s.root.show_data.devices.dmx.patch.fixtures.values().all(|f| f.meta.annotation.tags.is_empty()));
    assert!(run(&mut s, "Unassign Fixture 103 At Tag").contains("E-CMD-009"));
}

#[test]
fn thru_walks_one_parents_children() {
    let mut s = rig();
    let out = run(&mut s, "Info Tag 1.1 Thru 1.2");
    assert!(out.contains("Tag 1.1 \"Lx1\"") && out.contains("Tag 1.2 \"Lx2\""), "{out}");
    let out = run(&mut s, "Info Tag 1.1 Thru 2");
    assert!(out.contains("E-TAG-005"), "{out}");
}

#[test]
fn move_nests_with_the_guid_kept() {
    let mut s = rig();
    run(&mut s, "Store Tag 2 \"Busking\"");
    run(&mut s, "Assign Tag 1.2 At Fixture 104");
    // Nesting asks first (§34 "34g4 part 3c hardened" 3).
    let out = run(&mut s, "Move Tag 1.2 At Tag 2");
    assert!(out.starts_with("ASK") && out.contains("becomes a child of Tag 2") && out.contains("1 object"), "{out}");
    let out = run(&mut s, "Move Tag 1.2 At Tag 2 /NoConfirmation");
    assert!(out.contains("1.2 → 2.1"), "{out}");
    // Fixture 4 still carries it: the link is a GUID.
    let out = run(&mut s, "Tag 2.1");
    assert!(out.starts_with("1 fixture"), "{out}");
    let out = run(&mut s, "Move Tag 1 At Tag 1.1");
    assert!(out.contains("E-TAG-002"), "{out}");
}

/// `At Root` lifts a tag to the top, its children and its objects with it, without a question
/// (only nesting under a node asks; the round 3 audit).
#[test]
fn move_at_root_lifts_a_tag_to_the_top() {
    let mut s = rig();
    run(&mut s, "Assign Tag 1.1 At Fixture 101");
    let out = run(&mut s, "Move Tag 1.1 At Root");
    // The rig's own tags end at 901: the top level's next number follows them.
    assert!(!out.starts_with("ASK") && out.contains("1.1 → 902"), "{out}");
    let tags = &s.root.show_data.tags;
    let p = |t: &str| lumacore_tags::TagPath::parse(t).unwrap();
    assert_eq!(lumacore_tags::tag::find(tags, &p("902")).unwrap().name(), "Lx1");
    assert_eq!(lumacore_tags::tag::find(tags, &p("902.1")).unwrap().name(), "Right", "its child came along");
    let out = run(&mut s, "Tag 902");
    assert!(out.starts_with("1 fixture"), "{out}");
}

#[test]
fn delete_asks_for_children_and_keeps_the_objects() {
    let mut s = rig();
    let fixtures = s.root.show_data.devices.dmx.patch.fixtures.len();
    run(&mut s, "Assign Tag 1.1 At Fixture 101");
    let out = run(&mut s, "Delete Tag 1.1");
    assert!(out.starts_with("ASK") && out.contains("1 child tag"), "{out}");
    let out = run(&mut s, "Delete Tag 1.1 /KeepChildren /NoConfirmation");
    assert!(out.contains("1 object lost the tag") && out.contains("child tags now 1.3"), "{out}");
    assert_eq!(s.root.show_data.devices.dmx.patch.fixtures.len(), fixtures, "the objects stay");
    assert!(s.root.show_data.devices.dmx.patch.fixtures.values().all(|f| f.meta.annotation.tags.is_empty()));
}


#[test]
fn a_verb_on_a_tag_runs_on_what_it_carries() {
    let mut s = rig();
    run(&mut s, "Assign Tag 1.1 At Fixture 101 + 102");
    run(&mut s, "Assign Tag 1.1.1 At Fixture 103");
    // The current level: fixtures 101 and 102 get the value; 103 only with /R.
    let out = run(&mut s, "Tag 1.1 At 50");
    assert!(!out.contains("Error"), "{out}");
    let programmed = |s: &AppState| s.root.show_data.users.values().map(|u| u.programmer.values.len()).sum::<usize>();
    let two = programmed(&s);
    assert!(two > 0, "At reached the fixtures");
    run(&mut s, "Tag 1.1 At 60 /R");
    assert!(programmed(&s) > two, "/R reached fixture 103 too");
    // Nothing a tag carries takes Go: refused, nothing changes.
    let out = run(&mut s, "Go Tag 1.1");
    assert!(out.contains("E-CMD-009") && out.contains("Go"), "{out}");
    let out = run(&mut s, "Go Tag 2");
    assert!(out.contains("E-TAG-001") || out.contains("E-CMD-007"), "{out}");
}

#[test]
fn every_object_s_settings_have_a_meta_and_a_tags_tab() {
    let mut s = rig();
    run(&mut s, "Assign Tag 1.1 At Fixture 101");
    run(&mut s, "EditSetting Fixture 101");
    let Some(lumacore_wire::UiEvent::EditSettings { fields, .. }) = s.ui_requests.last() else { panic!("no settings") };
    let meta: Vec<&str> = fields.iter().filter(|f| f.section == "Meta").map(|f| f.name.as_str()).collect();
    assert!(meta.contains(&"Name") && meta.contains(&"GUID") && meta.contains(&"Path"), "{meta:?}");
    let tags = fields.iter().find(|f| f.section == "Tags").expect("a Tags tab");
    let lumacore_wire::FieldInput::Tags { nodes, assign, .. } = &tags.input else { panic!("{:?}", tags.input) };
    assert_eq!(assign, "Assign Tag {value} At Fixture 101");
    let lx1 = nodes.iter().find(|n| n.path == "1.1").unwrap();
    assert_eq!(lx1.state, "on");
    assert_eq!(nodes.iter().find(|n| n.path == "1").unwrap().state, "off");
    // A DataPool object is named with its owner.
    run(&mut s, "EditSetting Group 1");
    let Some(lumacore_wire::UiEvent::EditSettings { fields, .. }) = s.ui_requests.last() else { panic!("no settings") };
    let tags = fields.iter().find(|f| f.section == "Tags").expect("a Tags tab");
    let lumacore_wire::FieldInput::Tags { assign, .. } = &tags.input else { panic!() };
    assert_eq!(assign, "Assign Tag {value} At DataPool 1 Group 1");
}

/// A tag's own settings: its Action and its Meta, keyed by its GUID — a nested tag is the right
/// row, and a name a sibling has is refused.
#[test]
fn a_tag_has_its_own_settings() {
    let mut s = rig();
    run(&mut s, "EditSetting Tag 1.2");
    let Some(lumacore_wire::UiEvent::EditSettings { title, fields, .. }) = s.ui_requests.last() else { panic!("no settings") };
    assert_eq!(title, "Tag 1.2 \"Lx2\"");
    let names: Vec<(&str, &str)> = fields.iter().map(|f| (f.section.as_str(), f.name.as_str())).collect();
    assert_eq!(names[0], ("Tag", "Action"), "{names:?}");
    assert!(names.contains(&("Meta", "Name")) && names.contains(&("Meta", "GUID")) && names.contains(&("Tags", "Tags")), "{names:?}");
    let name = fields.iter().find(|f| f.name == "Name").unwrap();
    assert_eq!((name.value.as_str(), name.line.as_str()), ("Lx2", "Set Tag 1.2 \"Name\" {value}"));

    let out = run(&mut s, "Set Tag 1.2 \"Action\" Select");
    assert!(!out.contains("Error"), "{out}");
    let lx2 = &s.root.show_data.tags[&1].children[&2];
    assert_eq!(lx2.action, lumacore_tags::TagAction::SelectFixtures);
    assert!(s.root.show_data.tags[&1].children[&1].action.is_default(), "only the named tag");
    run(&mut s, "Set Tag 1.2 \"Paint\" \"#336699\"");
    assert_eq!(s.root.show_data.tags[&1].children[&2].meta.annotation.paint, Some(lumacore_tree::paint::Paint::Rgb([0x33, 0x66, 0x99])));
    run(&mut s, "Set Tag 1.2 \"Paint\" Theme \"Position\"");
    assert_eq!(s.root.show_data.tags[&1].children[&2].meta.annotation.paint, Some(lumacore_tree::paint::Paint::Theme("Position".into())));
    run(&mut s, "Set Tag 1.2 \"Priority\" 20");
    assert_eq!(s.root.show_data.tags[&1].children[&2].priority, 20);
    assert!(run(&mut s, "Set Tag 1.2 \"Paint\" \"Bluish\"").contains("E-SET-003"));
    let out = run(&mut s, "Set Tag 1.2 \"Name\" \"Lx1\"");
    assert!(out.contains("E-TAG-003"), "{out}");
    run(&mut s, "Set Tag 1.2 \"Name\" \"Floor\"");
    assert_eq!(s.root.show_data.tags[&1].children[&2].name(), "Floor");
}

/// Own Paint → the painted tag with the highest Priority (ties: the tag tree's order) → a tag
/// without a Paint takes its parent's → the pool's colour; tags link theme slots too.
#[test]
fn the_colour_order_is_own_then_tag_priority_then_parent_then_pool() {
    let mut s = lumacore_objects::demo::fresh_state();
    for line in [
        "Store Tag 910 \"Front\"",
        "Store Tag 911 \"Side\"",
        "Store Tag 910 Tag \"Low\"",
        "Set Tag 910 \"Paint\" Red",
        "Set Tag 911 \"Paint\" Theme \"Gobo\"",
        "Set Tag 911 \"Priority\" 20",
        "Assign Tag 910 + 911 At Group 3",
        "Assign Tag 910 At Group 4",
        "Assign Tag 910.1 At Group 5",
    ] {
        let out = run(&mut s, line);
        assert!(!out.contains("E-"), "{line}: {out}");
    }
    let dp = s.selected_datapool();
    let meta = |s: &AppState, n: u32| s.root.show_data.datapools[&dp].groups[&n].meta.clone();
    let p = lumacore_objects::paint::Painter::new(&s.root.show_data, "Admin");
    let three = p.of(&meta(&s, 3));
    assert_eq!(three.hex.as_deref(), lumacore_tree::media::PAINT_SLOTS.iter().find(|(n, _)| *n == "Gobo").map(|(_, c)| *c), "Side · 20 beats Front · 0");
    assert_eq!(three.from, lumacore_objects::paint::From::Tag { name: "Side".into(), priority: 20 });
    assert_eq!(three.from_text(), "tag Side · 20");
    assert_eq!(p.of(&meta(&s, 4)).hex, lumacore_tree::paint::Paint::parse("Red").unwrap().map(|p| p.word()), "one painted tag");
    let low = p.of(&meta(&s, 5));
    assert_eq!((low.hex.clone(), low.from_text()), (lumacore_tree::paint::Paint::parse("Red").unwrap().map(|p| p.word()), "tag Low · 0".to_string()), "the parent's colour");
    // A tie goes to the first in the tag tree.
    run(&mut s, "Set Tag 911 \"Priority\" 0");
    let p = lumacore_objects::paint::Painter::new(&s.root.show_data, "Admin");
    assert_eq!(p.of(&meta(&s, 3)).from_text(), "tag Front · 0");
    // Its own Paint overrides; none goes back to its tags.
    run(&mut s, "Set Group 3 \"Paint\" \"#010203\"");
    let p = lumacore_objects::paint::Painter::new(&s.root.show_data, "Admin");
    assert_eq!((p.of(&meta(&s, 3)).hex.as_deref(), p.of(&meta(&s, 3)).from), (Some("#010203"), lumacore_objects::paint::From::Own));
    let pool = p.of(&meta(&s, 6));
    assert_eq!((pool.hex.clone(), pool.from_text()), (None, "the pool".to_string()));
}

/// `Tag Group 3` (decisions 0297): the picker's event names the objects and their GUIDs; a path
/// before the objects is the Assign line's job; nothing changes.
#[test]
fn tag_with_objects_opens_the_picker_on_them() {
    let mut s = rig();
    run(&mut s, "Assign Tag 1.1 At Group 3");
    s.ui_requests.clear();
    let out = run(&mut s, "Tag Group 3");
    assert!(out.contains("Tags of Group 3: Tag 1.1 \"Lx1\""), "{out}");
    let picker = s.ui_requests.drain(..).find_map(|e| match e {
        lumacore_wire::UiEvent::TagPicker { object, guids, .. } => Some((object, guids.len())),
        _ => None,
    });
    assert_eq!(picker, Some(("Group 3".to_string(), 1)));
    run(&mut s, "Tag Group 1 Thru 3 + Sequence 1");
    let n = s.ui_requests.drain(..).find_map(|e| match e {
        lumacore_wire::UiEvent::TagPicker { guids, .. } => Some(guids.len()),
        _ => None,
    });
    assert_eq!(n, Some(4));
    let out = run(&mut s, "Tag 1 Group 3");
    assert!(out.contains("E-CMD-006") && out.contains("Assign Tag 1 At Group 3"), "{out}");
    assert!(!s.ui_requests.iter().any(|e| matches!(e, lumacore_wire::UiEvent::TagPicker { .. })));
}

/// A tap on a cell that already carries the armed tag answers so and files no Oops step.
#[test]
fn assigning_what_is_already_carried_changes_nothing() {
    let mut s = rig();
    run(&mut s, "Assign Tag 1.1 At Group 3");
    let steps = s.root.show_data.walk_identity().links.len();
    let out = run(&mut s, "Assign Tag 1.1 At Group 3");
    assert!(out.contains("1 group already carries Tag 1.1 \"Lx1\""), "{out}");
    let out = run(&mut s, "Unassign Tag 1.2 At Group 3");
    assert!(out.contains("1 group doesn't carry Tag 1.2"), "{out}");
    assert_eq!(s.root.show_data.walk_identity().links.len(), steps);
    let out = run(&mut s, "Assign Tag 1.1 At Group 3 + Group 4");
    assert!(out.contains("Tagged 2 groups"), "{out}");
}

