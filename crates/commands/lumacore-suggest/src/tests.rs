//! The suggestion engine (decisions §30 "12b2"): grammar first, then the
//! fuzzy score, then usage, a fixed tie-break; planned and no-rights items;
//! usage saved and replicated but never undone.

use super::*;
use lumacore_objects::DispatchResult;
use lumacore_objects::commands::dispatch;

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => String::new(),
    }
}

/// Admin with fixture 1 and Group 1 "All", 2 "Odd", 3 "Even".
fn show() -> AppState {
    let mut state = AppState::new();
    let fixture = lumacore_fixtures::Fixture::new(
        None,
        uuid::Uuid::new_v4(),
        uuid::Uuid::nil(),
        uuid::Uuid::nil(),
        uuid::Uuid::nil(),
        1,
        1,
    );
    state.root.show_data.devices.dmx.patch.insert_fid(1, fixture);
    state.acting_user = Some("Admin".to_string());
    run(&mut state, "Fixture 1");
    for (n, name) in [(1, "All"), (2, "Odd"), (3, "Even")] {
        run(&mut state, &format!("Store Group {n} \"{name}\""));
    }
    run(&mut state, "Clear");
    state
}

fn ask(state: &AppState, line: &str, history: &[String]) -> Suggestions {
    suggest(
        state,
        &Query {
            line,
            cursor: line.chars().count(),
            surface: Surface::Palette,
            history,
            now_ms: 1_000_000,
        },
    )
}

fn titles(s: &Suggestions) -> Vec<String> {
    s.items.iter().map(|i| i.title.clone()).collect()
}

#[test]
fn grammar_first_only_what_is_valid_at_the_cursor() {
    let state = show();
    // After Store: objects that can be stored (or are planned to be) —
    // never a verb.
    let s = ask(&state, "Store ", &[]);
    assert!(titles(&s).iter().any(|t| t == "Group"), "{:?}", titles(&s));
    assert!(s.items.iter().all(|i| i.kind == ItemKind::Keyword), "{:?}", titles(&s));
    let store = registry::lookup("store").unwrap().object_verb.unwrap();
    for i in &s.items {
        let o = objects::find(&i.title).expect("an object");
        assert!(o.supports(store) || o.is_planned(store), "{}", i.title);
    }
    // After a group keyword: its objects, with their names and GUIDs.
    let s = ask(&state, "Group ", &[]);
    let g3 = s.items.iter().find(|i| i.title == "Group 3").expect("Group 3");
    assert_eq!(g3.kind, ItemKind::Object);
    assert!(g3.subtitle.contains("Even") && g3.guid.is_some(), "{g3:?}");
    // One row per object (its number and its name both complete to it).
    assert_eq!(s.items.iter().filter(|i| i.title == "Group 3").count(), 1);
    assert_eq!(s.items.iter().find(|i| i.title == "Group 1").unwrap().insert, "1 ");
    // Typing the name finds it by name.
    let by_name = ask(&state, "Group \"Ev", &[]);
    assert_eq!(by_name.items[0].title, "Group 3", "{:?}", titles(&by_name));
    // The span replaced is the word at the cursor.
    let s = ask(&state, "Store Gr", &[]);
    assert_eq!(s.replace, [6, 8]);
    assert_eq!(s.items[0].title, "Group");
    assert_eq!(s.items[0].insert, "Group ");
    // Nothing valid, nothing offered.
    assert!(ask(&state, "qqqqzz", &[]).items.is_empty());
}

#[test]
fn the_fuzzy_score_ranks_and_the_order_is_fixed() {
    let state = show();
    // Not only prefixes: `grp` finds Group.
    let s = ask(&state, "Store grp", &[]);
    assert!(titles(&s).contains(&"Group".to_string()), "{:?}", titles(&s));
    // Sorted by score, the same every time.
    let a = ask(&state, "Gr", &[]);
    assert!(a.items.windows(2).all(|w| w[0].score >= w[1].score));
    assert_eq!(a, ask(&state, "Gr", &[]));
    // `Gr` at the start brings the groups themselves, in number order
    // while none is used.
    let groups: Vec<String> = titles(&a).into_iter().filter(|t| t.starts_with("Group ")).collect();
    assert_eq!(groups, ["Group 1", "Group 2", "Group 3"]);
}

#[test]
fn usage_boosts_and_is_recorded_from_dispatch() {
    let mut state = show();
    run(&mut state, "Group 3");
    run(&mut state, "Group 3");
    let user = state.root.show_data.user("Admin").unwrap();
    let guid = ask(&state, "Group ", &[])
        .items
        .iter()
        .find(|i| i.title == "Group 3")
        .and_then(|i| i.guid.clone())
        .unwrap();
    assert!(user.usage.get(&guid).is_some_and(|e| e.count >= 2), "{:?}", user.usage);
    assert!(user.usage.get(&word_key("Group")).is_some());
    // The used group comes first now, and says so.
    let s = suggest(
        &state,
        &Query { line: "Group ", cursor: 6, surface: Surface::Bar, history: &[], now_ms: now_ms() },
    );
    let first = s.items.iter().find(|i| i.kind == ItemKind::Object).unwrap();
    assert_eq!(first.title, "Group 3");
    assert!(first.subtitle.contains("used"), "{first:?}");
    // A line that failed counts nothing.
    let before = state.root.show_data.user("Admin").unwrap().usage.clone();
    run(&mut state, "Group 99 At 50 /Nonsense");
    run(&mut state, "Store Wibble 3");
    assert_eq!(state.root.show_data.user("Admin").unwrap().usage, before);
}

#[test]
fn usage_is_saved_and_replicated_but_never_undone() {
    let mut state = show();
    state.replication_outbox.clear();
    run(&mut state, "Group 2");
    let usage = state.root.show_data.user("Admin").unwrap().usage.clone();
    assert!(!usage.entries.is_empty());
    let guid = state.root.show_data.user("Admin").unwrap().meta.identity.guid;
    // Replicated with the line's other changes.
    let op = state.replication_outbox.last().expect("an op");
    assert!(op.affected_subtrees.contains(&lumacore_tree::ShowSubtree::Usage(guid)), "{op:?}");
    // Not in the Oops step, and Oops leaves it.
    let frame = state.undo_stack_mut("Admin").frames.back().cloned().expect("a step");
    assert!(frame.snapshots.iter().all(|s| s.subtree.is_undoable()));
    run(&mut state, "Oops");
    let after = &state.root.show_data.user("Admin").unwrap().usage;
    for (k, e) in &usage.entries {
        assert!(after.get(k).is_some_and(|a| a.count >= e.count), "{k} undone");
    }
    // Saved with the show.
    let bytes = lumacore_luma::show_file::encode(&state.root.show_data).unwrap().0;
    let back = lumacore_luma::show_file::from_bytes(&bytes).unwrap();
    assert_eq!(back.user("Admin").unwrap().usage, *after);
}

#[test]
fn planned_and_no_rights_items_are_listed_dimmed() {
    let mut state = show();
    // A planned word says so.
    let planned = lumacore_objects::commands::registry::all()
        .find(|c| c.status == registry::Status::Planned && !c.doc.hidden)
        .expect("a planned word");
    let s = ask(&state, planned.label(), &[]);
    let item = s.items.iter().find(|i| i.title == planned.label()).expect("listed");
    assert_eq!(item.status, ItemStatus::Planned);
    assert!(item.subtitle.contains("planned"));
    // A Guest sees Store, refused with its code.
    run(&mut state, "Store User \"Visitor\"");
    state.root.show_data.user_mut("Visitor").unwrap().rights = lumacore_tree::UserRights::Guest;
    state.acting_user = Some("Visitor".to_string());
    let s = ask(&state, "Stor", &[]);
    let store = s.items.iter().find(|i| i.title == "Store").expect("Store listed");
    assert_eq!(store.disabled.as_deref(), Some("E-RIGHTS-001"));
    assert!(store.subtitle.contains("Guest"), "{store:?}");
}

#[test]
fn history_comes_first_and_is_the_ghost_text() {
    let state = show();
    let history = vec!["Group 3 At 50".to_string(), "Group 3 At 80".to_string()];
    let s = ask(&state, "Group 3 At ", &history);
    assert_eq!(s.items[0].kind, ItemKind::History);
    assert_eq!(s.items[0].insert, "80", "newest first");
    assert_eq!(s.items[1].insert, "50");
    assert_eq!(s.hint.as_deref(), Some("80"));
    // `Gr` → the whole line.
    let s = ask(&state, "Gr", &history);
    assert_eq!(s.items[0].insert, "Group 3 At 80");
    assert_eq!(s.hint.as_deref(), Some("oup 3 At 80"));
    // No history: the first completion that extends the word.
    assert_eq!(ask(&state, "Store Gr", &[]).hint.as_deref(), Some("oup"));
}

#[test]
fn the_bar_offers_nothing_on_an_empty_line_and_no_actions() {
    let state = show();
    let bar = |line: &str| {
        suggest(
            &state,
            &Query { line, cursor: line.chars().count(), surface: Surface::Bar, history: &[], now_ms: 0 },
        )
    };
    assert!(bar("").items.is_empty());
    assert!(bar("Group ").items.iter().all(|i| i.actions.is_empty()));
    // The palette carries an object's actions as command lines.
    let s = ask(&state, "Group ", &[]);
    let g = s.items.iter().find(|i| i.title == "Group 1").unwrap();
    assert!(g.actions.iter().any(|a| a.line == "Group 1"), "{:?}", g.actions);
    assert_eq!(s.mode, "Auto");
}

#[test]
fn the_profile_setting_is_set_like_the_other_preferences() {
    let mut state = show();
    assert!(run(&mut state, "Set UserProfile \"Suggestions\" TabOnly").contains("TabOnly"));
    assert_eq!(mode(&state), SuggestionsMode::TabOnly);
    let out = run(&mut state, "Set UserProfile \"Suggestions\" Sometimes");
    assert!(out.contains("E-SYN-001"), "an undeclared word: {out}");
    run(&mut state, "Set UserProfile \"Suggestions\" Off");
    assert_eq!(ask(&state, "Gr", &[]).mode, "Off");
}

/// Cue is Store's default object (decisions §30 "Store Cue"): a number after Store (or
/// its short form) is offered first as the full line `Store Cue <n>`; the items replace from the
/// line's start.
#[test]
fn a_number_after_store_completes_to_store_cue() {
    let s = show();
    for line in ["S 1", "Store 1", "store 2.5"] {
        let out = ask(&s, line, &[]);
        let n = line.split_whitespace().last().unwrap();
        assert_eq!(out.items[0].title, format!("Store Cue {n}"), "{line}");
        assert_eq!(out.items[0].insert, format!("Store Cue {n}"));
        assert_eq!(out.replace[0], 0);
    }
    // Not for a word without a default object, nor past the first argument.
    assert!(ask(&s, "Delete 1", &[]).items.iter().all(|i| !i.title.starts_with("Delete Cue")));
    assert!(ask(&s, "Store Group 1", &[]).items.iter().all(|i| i.title != "Store Cue 1"));
}

// ----- Moved from lumacore-objects (34g4: completion is this crate's) -----

#[test]
fn hint_is_the_first_completion() {
    let state = AppState::new();
    let line = "Store Gr";
    let hint = crate::hint_input(line, line.len(), &state).expect("a hint");
    assert_eq!(format!("Gr{hint}"), "Group");
}

#[test]
fn completion_offers_the_declared_flags_after_a_slash() {
    let state = AppState::new();
    let complete = |line: &str| crate::grammar::complete_input(line, line.chars().count(), &state).1;
    let offered = complete("Store Group 3 /");
    assert!(offered.contains(&"/Overwrite".to_string()) && offered.contains(&"/Merge".to_string()), "{offered:?}");
    assert!(offered.contains(&"/NoConfirmation".to_string()), "the globals too: {offered:?}");
    assert!(!offered.contains(&"/Force".to_string()), "Delete's, not Store's: {offered:?}");
    assert_eq!(complete("Store Group 3 /ov"), vec!["/Overwrite".to_string()]);
    // A flag already on the line isn't offered again; a `/` in free text offers none.
    assert!(!complete("Store Group 3 /Overwrite /").contains(&"/Overwrite".to_string()));
    assert!(complete("Echo hello /").iter().all(|c| c != "/NoConfirmation"));
}

/// `/FixtureType ▸`: the show's types first (number, name), then library/gdtf/'s files.
#[test]
fn fixture_type_completes_the_shows_types_then_library_files() {
    std::fs::create_dir_all("library/gdtf/Sub").unwrap();
    std::fs::write("library/gdtf/Sub/T-r3-Spot.gdtf", b"PK test").unwrap();
    let mut state = AppState::new();
    let ft = lumacore_fixtures::FixtureType {
        meta: lumacore_base::NodeMeta::new(Some("LED Par".into())),
        manufacturer: "Test".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: Vec::new(),
    };
    state.root.show_data.devices.dmx.patch.fixture_types.insert(3, std::sync::Arc::new(ft));
    let complete = |line: &str| crate::grammar::complete_input(line, line.chars().count(), &state).1;
    let offered = complete("Store Fixture 1 /FixtureType ");
    assert_eq!(&offered[..2], &["3".to_string(), "\"LED Par\"".to_string()], "{offered:?}");
    assert!(offered.contains(&"\"Sub/T-r3-Spot.gdtf\"".to_string()), "{offered:?}");
    assert_eq!(complete("Store Fixture 1 /FixtureType \"sub/t"), vec!["\"Sub/T-r3-Spot.gdtf\"".to_string()]);
}

// ----- The completion golden record (moved from objects' tree-path golden, 34g4) -----

/// Lines whose Tab completion is recorded, each from the root.
const COMPLETE: &[&str] = &[
    "cd ",
    "cd Sh",
    "cd Root.",
    "cd Root.ShowData.",
    "cd Root.ShowData.D",
    "cd Root.ShowData.DataPools.",
    "cd Root.ShowData.DataPools.1.",
    "cd Root.ShowData.Devices.Dmx.Patch.",
    "cd Root.ShowData.Devices.Dmx.Patch.Fixtures.",
    "ls Root.Runtime.",
    "List Root.ShowData.",
    "Select Root.ShowData.DataPools.",
];

/// What Tab offers for each of [`COMPLETE`] on the demo show, from the root.
fn completion_golden() -> String {
    let mut state = lumacore_objects::demo::fresh_state();
    let mut out = String::from("# Generated by tab_completion_is_unchanged (lumacore-suggest src/tests.rs). Do not edit.\n\n## Completion\n\n");
    for line in COMPLETE {
        let _ = lumacore_objects::commands::dispatch(&mut state, "cd /");
        let (at, words) = crate::grammar::complete_input(line, line.chars().count(), &state);
        // The first few of a long list (the rig's thousand fixtures): the golden pins the shape.
        let shown = if words.len() > 8 { format!("{} … (+{} more)", words[..8].join(" "), words.len() - 8) } else { words.join(" ") };
        out.push_str(&format!("$ {line}<Tab>  (from {at})\n  {shown}\n"));
    }
    let station = state.root.station_settings.station_name.clone();
    if station.is_empty() { out } else { out.replace(&station, "<station>") }
}

#[test]
fn tab_completion_is_unchanged() {
    let file = lumacore_objects::demo::repo_root().join("crates/commands/lumacore-suggest/testdata/tab_completion.golden.txt");
    let made = completion_golden();
    if std::env::var_os("LUMACORE_BLESS").is_some() {
        std::fs::create_dir_all(file.parent().expect("dir")).expect("testdata dir");
        std::fs::write(&file, &made).expect("write the golden file");
        return;
    }
    let committed = std::fs::read_to_string(&file).unwrap_or_default();
    assert!(committed == made, "Tab completion changed; if on purpose, rewrite with LUMACORE_BLESS=1 and read the diff.");
}

/// `Fixture 10` on the demo show offers its numbers (decisions §40 5: the ids come by number,
/// never by slot).
#[test]
fn fixture_numbers_are_offered_by_number_not_slot() {
    let mut state = lumacore_objects::demo::fresh_state();
    state.acting_user = Some("Admin".to_string());
    let s = ask(&state, "Fixture 10", &[]);
    let t = titles(&s);
    assert!(t.iter().any(|x| x.contains("101")), "{t:?}");
}
