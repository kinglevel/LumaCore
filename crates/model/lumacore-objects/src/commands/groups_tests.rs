//! Step 7 behaviour (decisions §22): groups in the user's datapool, the console
//! selection rule (probed), the selection grid, Store options, group
//! masters on the rendered frame, references — through real dispatch.

use super::playback_tests::{dmx, rig, run};
use crate::AppState;
use lumacore_tree::{GridCell, GroupMasterMode, MasterLevel};

/// Fixture numbers of the acting user's selection, in order.
fn selection(state: &AppState) -> Vec<u32> {
    let guids = state
        .current_programmer()
        .unwrap()
        .selected_fixtures
        .clone();
    state.root.show_data.fixture_ids_for_guids(&guids)
}

fn cell(state: &AppState, fixture: u32) -> GridCell {
    let guid = state.root.show_data.fixture_guid(fixture).unwrap();
    state.current_programmer().unwrap().grid[&guid]
}

/// Six dimmers; Group 1 = 1–3, Group 2 = 4–6; nothing selected.
fn two_groups() -> AppState {
    let mut state = rig(&[1, 2, 3, 4, 5, 6]);
    run(&mut state, "Fixture 1 Thru 3");
    run(&mut state, "Store Group 1 \"Left\"");
    run(&mut state, "ResetAll");
    run(&mut state, "Fixture 4 Thru 6");
    run(&mut state, "Store Group 2 \"Right\"");
    run(&mut state, "ResetAll");
    state
}

#[test]
fn groups_selection_rule_matches_the_convention() {
    let mut state = two_groups();
    // Plain selects add until a value is set.
    run(&mut state, "Group 1");
    assert_eq!(selection(&state), vec![1, 2, 3]);
    run(&mut state, "Group 2");
    assert_eq!(selection(&state), vec![1, 2, 3, 4, 5, 6]);
    // After a value, the first plain select replaces …
    run(&mut state, "At 50");
    run(&mut state, "Group 1");
    assert_eq!(selection(&state), vec![1, 2, 3]);
    // … and the next one adds again.
    run(&mut state, "Group 2");
    assert_eq!(selection(&state), vec![1, 2, 3, 4, 5, 6]);
    // `+` always adds, even after a value.
    run(&mut state, "At 60");
    run(&mut state, "Fixture 1");
    assert_eq!(selection(&state), vec![1]);
    run(&mut state, "At 70");
    run(&mut state, "+ Group 2");
    assert_eq!(selection(&state), vec![1, 4, 5, 6]);
    // `A + B` after a value replaces with the union; `-` takes out.
    run(&mut state, "At 80");
    run(&mut state, "Group 2 + Group 1 - Fixture 5");
    assert_eq!(selection(&state), vec![4, 6, 1, 2, 3]);
    run(&mut state, "- Group 1");
    assert_eq!(selection(&state), vec![4, 6]);
    // Fixture selections follow the same rule and keep the typed order.
    run(&mut state, "At 10");
    run(&mut state, "Fixture 3 Thru 1");
    assert_eq!(selection(&state), vec![3, 2, 1]);
}

#[test]
fn groups_grid_cursor_appends_blocks_and_moves() {
    let mut state = two_groups();
    run(&mut state, "Group 1");
    run(&mut state, "Group 2");
    // The second group lands after the first along X.
    assert_eq!(cell(&state, 1).x, 0);
    assert_eq!(cell(&state, 4).x, 3);
    assert_eq!(run(&mut state, "Grid"), "Grid cursor 6/0/0");
    run(&mut state, "ResetAll");
    // An empty selection puts the cursor home; Grid moves it.
    run(&mut state, "Grid 0/2");
    run(&mut state, "Group 1");
    assert_eq!((cell(&state, 1).x, cell(&state, 1).y), (0, 2));
    // Stored spots come back from the group (NewLine moves to the next line).
    run(&mut state, "Set Group 2 \"MoveGridCursor\" NewLine");
    run(&mut state, "ResetAll");
    run(&mut state, "Group 2");
    run(&mut state, "Group 1");
    assert_eq!((cell(&state, 1).x, cell(&state, 1).y), (0, 1));
}

#[test]
fn groups_preserve_grid_off_closes_gaps() {
    let mut state = two_groups();
    run(&mut state, "Set Group 1 Fixture 2 \"X\" 4");
    run(&mut state, "Set Group 1 Fixture 3 \"X\" 8");
    run(&mut state, "Group 1");
    assert_eq!(cell(&state, 3).x, 8);
    run(&mut state, "ResetAll");
    run(&mut state, "Set Group 1 \"PreserveGrid\" Off");
    run(&mut state, "Group 1");
    assert_eq!((cell(&state, 2).x, cell(&state, 3).x), (1, 2));
}

#[test]
fn groups_store_over_existing_needs_an_option() {
    let mut state = two_groups();
    run(&mut state, "Fixture 5");
    // A UI asks Merge / Overwrite / Remove (decisions §30 "12b"); a one-shot
    // path is told the flags.
    let crate::DispatchResult::Ask(asked) = super::dispatch(&mut state, "Store Group 1") else {
        panic!("Store over a used group asks");
    };
    assert!(asked.refusal().contains("/Merge, /Overwrite or /Remove"));
    let guid = state.user_groups()[&1].meta.identity.guid;
    run(&mut state, "Store Group 1 /Merge");
    let g = &state.user_groups()[&1];
    assert_eq!(g.members.len(), 4);
    // AppendX: the merged member goes after the group's last X.
    assert_eq!(g.members[3].cell.x, 3);
    run(&mut state, "Store Group 1 /Remove");
    assert_eq!(state.user_groups()[&1].members.len(), 3);
    run(&mut state, "Store Group 1 /Overwrite");
    let g = &state.user_groups()[&1];
    assert_eq!(g.members.len(), 1);
    assert_eq!(g.meta.identity.guid, guid, "overwrite keeps the GUID");
    // Bare Store Group takes the first free number.
    run(&mut state, "Store Group");
    assert!(state.user_groups().contains_key(&3));
}

#[test]
fn groups_copy_gets_a_new_guid_and_refuses_occupied() {
    let mut state = two_groups();
    run(&mut state, "Copy Group 1 At 5");
    let (a, b) = (&state.user_groups()[&1], &state.user_groups()[&5]);
    assert_ne!(a.meta.identity.guid, b.meta.identity.guid);
    assert_eq!(a.fixtures(), b.fixtures());
    // Onto an existing group the one ask comes (decisions §35 5); the flags answer it.
    assert!(matches!(crate::commands::dispatch(&mut state, "Copy Group 2 At 5"), crate::DispatchResult::Ask(_)));
    run(&mut state, "Copy Group 2 At 5 /Merge");
    assert_eq!(state.user_groups()[&5].members.len(), 6);
}

#[test]
fn groups_are_pool_relative() {
    let mut state = two_groups();
    let mut other = lumacore_tree::Datapool::new(Some("Other".into()));
    other.groups.insert(9, lumacore_tree::Group::new(None));
    state.root.show_data.datapools.insert(2, other);
    // Group 9 is in another datapool: not this user's.
    assert!(run(&mut state, "Group 9").contains("E-GROUP-003"));
}

#[test]
fn groups_recall_is_undone_by_oops() {
    let mut state = two_groups();
    run(&mut state, "Group 1");
    assert_eq!(selection(&state).len(), 3);
    run(&mut state, "Oops");
    assert!(selection(&state).is_empty());
}

#[test]
fn groups_deleted_fixture_keeps_its_link() {
    let mut state = two_groups();
    state.root.show_data.devices.dmx.patch.remove_fid(2);
    state.root.show_data.after_load();
    // The member stays (GUID and grid spot), recall skips it.
    assert_eq!(state.user_groups()[&1].members.len(), 3);
    run(&mut state, "Group 1");
    assert_eq!(selection(&state), vec![1, 3]);
    let report = crate::references::check(&mut state, "test");
    assert!(
        report
            .broken
            .iter()
            .any(|l| l.path.contains("Members.Fixture"))
    );
}

#[test]
fn groups_delete_drops_the_master_level() {
    let mut state = two_groups();
    run(&mut state, "Set Group 1 \"Mode\" Scaling");
    run(&mut state, "Master Group 1 At 40");
    let guid = state.user_groups()[&1].meta.identity.guid;
    assert!(
        state
            .root
            .show_data
            .playbacks
            .live.masters.groups
            .contains_key(&guid)
    );
    // Its own level is not a reference that blocks the delete.
    let out = run(&mut state, "Delete Group 1 /nc");
    assert!(out.contains("deleted"), "{out}");
    assert!(
        !state
            .root
            .show_data
            .playbacks
            .live.masters.groups
            .contains_key(&guid)
    );
}

#[test]
fn groups_masters_act_on_the_frame() {
    let mut state = two_groups();
    run(&mut state, "Fixture 1 Thru 6 At 100");
    assert_eq!(dmx(&state, 1), 255);
    // Scaling multiplies.
    run(&mut state, "Set Group 1 \"Mode\" Scaling");
    run(&mut state, "Master Group 1 At 50");
    assert_eq!(dmx(&state, 1), 128);
    assert_eq!(dmx(&state, 4), 255);
    // Negative beats Positive where both act.
    run(&mut state, "Set Group 2 \"Mode\" Positive");
    run(&mut state, "Master Group 2 At 80");
    assert_eq!(dmx(&state, 4), 204);
    run(&mut state, "Fixture 4");
    run(&mut state, "Store Group 3");
    run(&mut state, "Set Group 3 \"Mode\" Negative");
    run(&mut state, "Master Group 3 At 30"); // owner first (`Master Group 3 At 30`) in §33 3d
    assert_eq!(dmx(&state, 4), 77);
    assert_eq!(dmx(&state, 5), 204);
    // Additive adds HTP to a dark fixture.
    run(&mut state, "Fixture 6 At 0");
    run(&mut state, "Fixture 6");
    run(&mut state, "Store Group 4 /Overwrite");
    run(&mut state, "Set Group 4 \"Mode\" Additive");
    run(&mut state, "Master Group 4 At 40");
    assert_eq!(dmx(&state, 6), 102);
    // Master React None ignores every master.
    run(&mut state, "Set Fixture 1 \"MasterReact\" None");
    assert_eq!(dmx(&state, 1), 255);
    // The grand master still applies after the group masters.
    run(&mut state, "Master 2.1 At 50");
    assert_eq!(dmx(&state, 2), 64);
    assert_eq!(dmx(&state, 1), 255);
}

#[test]
fn groups_master_fade_must_be_finite_and_bounded() {
    let mut state = two_groups();
    run(&mut state, "Set Group 1 \"Mode\" Scaling");
    for bad in ["inf", "NaN", "3601", "-1", "1e300"] {
        let out = run(&mut state, &format!("Master Group 1 At 50 Fade {bad}"));
        assert!(out.contains("E-CMD-008") || out.contains("E-SYN-"), "{bad}: {out}");
    }
    let out = run(&mut state, "Master Group 1 At 50 Fade 3600");
    assert!(out.contains("in 3600 s"), "{out}");
}

#[test]
fn groups_master_fade_restarts_settled_after_a_file_load() {
    let mut state = two_groups();
    run(&mut state, "Set Group 1 \"Mode\" Scaling");
    run(&mut state, "Master Group 1 At 0 Fade 3600");
    let guid = state.user_groups()[&1].meta.identity.guid;
    let dir = std::env::temp_dir().join(format!("lumacore-r1-03-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fade.luma");
    lumacore_luma::show_file::write(&path, &state.root.show_data, |_| {}).unwrap();
    let mut loaded = crate::AppState::new();
    crate::objects::load_show(&mut loaded, &path.to_string_lossy()).unwrap();
    // Saved mid-fade with a start on another clock: it stands at its target.
    let m = loaded.root.show_data.playbacks.live.masters.groups[&guid];
    assert_eq!(m, MasterLevel::snap(0.0));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn groups_master_fades() {
    let mut level = MasterLevel::snap(1.0);
    level.go_to(0.0, 2000, 10_000);
    assert!((level.at(11_000) - 0.5).abs() < 1e-9);
    assert!(level.fading(11_000));
    assert_eq!(level.at(12_500), 0.0);
    assert!(!level.fading(12_500));

    let mut state = two_groups();
    run(&mut state, "Set Group 1 \"Mode\" Scaling");
    let out = run(&mut state, "Master Group 1 At 0 Fade 2");
    assert!(out.contains("in 2 s"), "{out}");
    let guid = state.user_groups()[&1].meta.identity.guid;
    let m = state.root.show_data.playbacks.live.masters.groups[&guid];
    assert_eq!((m.from, m.to, m.fade_ms), (1.0, 0.0, 2000));
    // The compiled plan sees the fade (the tick evaluates it every frame).
    assert!(state.output_plan().value.fading(m.start_ms + 1000));
}

#[test]
fn groups_member_grid_and_order_are_set_by_commands() {
    let mut state = two_groups();
    run(
        &mut state,
        "Set Group 1 Fixture 3 \"X\" 5; Set Group 1 Fixture 3 \"Y\" 2; Set Group 1 Fixture 3 \"Rotation\" 90; Set Group 1 Fixture 3 \"MirrorX\" On; Set Group 1 Fixture 3 \"Order\" 1",
    );
    let g = &state.user_groups()[&1];
    let first = g.members[0];
    assert_eq!(
        state.root.show_data.fixture_id_for_guid(first.fixture),
        Some(3)
    );
    assert_eq!(
        (
            first.cell.x,
            first.cell.y,
            first.cell.rotation,
            first.cell.mirror_x
        ),
        (5, 2, 90.0, true)
    );
    assert_eq!(g.master, GroupMasterMode::None);
    let info = run(&mut state, "Info Group 1");
    assert!(info.contains("rot 90°"), "{info}");
    assert!(run(&mut state, "Set Group 1 \"Mode\" \"Loud\"").contains("E-SET-003"));
}

#[test]
fn groups_list_and_knock_out() {
    let mut state = two_groups();
    run(&mut state, "Set Group 2 \"Mode\" Positive");
    let list = run(&mut state, "List Group");
    assert!(list.contains("Positive 100%"), "{list}");
    run(&mut state, "Group 1 At 50");
    assert_eq!(selection(&state), vec![1, 2, 3]);
    assert_eq!(dmx(&state, 2), 128);
    run(&mut state, "Off Group 1");
    assert!(selection(&state).is_empty());
}

/// Every datapool's groups are in the tree by number (decisions §22, §29).
#[test]
fn groups_of_every_datapool_are_in_the_tree() {
    let mut state = two_groups();
    let mut other = lumacore_tree::Datapool::new(Some("Other".into()));
    other.groups.insert(9, lumacore_tree::Group::new(Some("Far".into())));
    state.root.show_data.datapools.insert(2, other);
    let listed = run(&mut state, "List Root.ShowData.DataPools");
    assert!(listed.contains('1') && listed.contains('2'), "{listed}");
    let node = crate::vfs::resolve_node("Root.ShowData.DataPools.2.Groups.9", &state.root);
    assert!(matches!(node, Some(crate::vfs::NodeView::Object { .. })), "{node:?}");
    assert!(crate::vfs::resolve_node("Root.ShowData.DataPools.1.Groups.9", &state.root).is_none());
    let here = run(&mut state, "List Root.ShowData.DataPools.2.Groups");
    assert!(here.contains("Far"), "{here}");
}

/// R1-40: copying an overlapping range copies what each source held before
/// the line ran — never a group this line just replaced.
#[test]
fn groups_copy_over_an_overlapping_range_reads_the_originals() {
    let mut state = rig(&[1, 2, 3]);
    for n in 1..=3 {
        run(&mut state, "ResetAll");
        run(&mut state, &format!("Fixture {n}"));
        run(&mut state, &format!("Store Group {n}"));
    }
    run(&mut state, "Copy Group 1 Thru 3 At 2 /Overwrite");
    let first_fixture = |s: &AppState, g: u32| {
        let guid = s.user_groups()[&g].members[0].fixture;
        s.root.show_data.devices.dmx.patch.fixture_by_guid(guid).and_then(|(_, f)| f.mvr.fid)
    };
    assert_eq!(first_fixture(&state, 2), Some(1), "Group 2 got Group 1");
    assert_eq!(first_fixture(&state, 3), Some(2), "Group 3 got the old Group 2, not Group 1 again");
    assert_eq!(first_fixture(&state, 4), Some(3), "Group 4 got the old Group 3");
    // A destination past u32::MAX is refused, never wrapped or panicking.
    let out = run(&mut state, "Copy Group 1 Thru 3 At 4294967295");
    assert!(out.contains("E-CMD-008"), "{out}");
}

/// R1-11 (Q3): group numbers are 1–9999 — store, copy and move.
#[test]
fn groups_numbers_stay_in_the_pool() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1");
    assert!(run(&mut state, "Store Group 1000000").contains("1–9999"));
    run(&mut state, "Store Group 9999");
    assert!(state.user_groups().contains_key(&9999));
    assert!(run(&mut state, "Copy Group 9999 At 10000").contains("E-CMD-008"));
    assert!(run(&mut state, "Move Group 9999 At 10000").contains("1–9999"));
    assert_eq!(state.user_groups().len(), 1);
}

/// R1-61 (Q35): the station asks before a Delete; the answer line carries
/// /NoConfirmation and deletes; /nc skips the question.
#[test]
fn groups_delete_is_asked_by_the_station() {
    use crate::DispatchResult;
    let mut state = two_groups();
    let DispatchResult::Ask(ask) = super::dispatch(&mut state, "Delete Group 1") else {
        panic!("Delete is asked");
    };
    assert!(state.user_groups().contains_key(&1), "nothing deleted before the answer");
    let line = ask.primary_line().unwrap();
    assert!(line.contains("/NoConfirmation"), "{line}");
    run(&mut state, &line);
    assert!(!state.user_groups().contains_key(&1));
    run(&mut state, "Delete Group 2 /nc");
    assert!(!state.user_groups().contains_key(&2));
}

/// R1-55: owner first — `<verb> DataPool 2 Group 3` acts on DataPool 2's
/// group, never on the selected DataPool's (a pinned pool window sends it).
#[test]
fn groups_owner_first_acts_in_the_named_datapool() {
    let mut state = rig(&[1, 2]);
    state.root.show_data.datapools.insert(2, lumacore_tree::Datapool::new(Some("Second".into())));
    state.root.show_data.after_load();
    run(&mut state, "Fixture 1");
    run(&mut state, "Store Group 3");
    run(&mut state, "ResetAll");
    run(&mut state, "Fixture 2");
    run(&mut state, "Store DataPool 2 Group 3");
    run(&mut state, "ResetAll");
    let dp = |s: &AppState, n: u32| s.root.show_data.datapools[&n].groups.contains_key(&3);
    assert!(dp(&state, 1) && dp(&state, 2), "one Group 3 in each");
    run(&mut state, "Label DataPool \"Second\" Group 3 \"Far\"");
    assert_eq!(state.root.show_data.datapools[&2].groups[&3].meta.identity.name.as_deref(), Some("Far"));
    run(&mut state, "Delete DataPool 2 Group 3 /nc");
    assert!(dp(&state, 1) && !dp(&state, 2), "only DataPool 2's group went");
    assert_eq!(state.selected_datapool(), 1, "the selection didn't move");
    assert!(run(&mut state, "Delete DataPool 9 Group 3 /nc").contains("E-CMD-007"));
}

/// R1 review 6: an owner-first line's question and editor keep the DataPool.
#[test]
fn groups_owner_first_keeps_its_datapool_in_asks_and_the_editor() {
    let mut state = rig(&[1, 2]);
    state.root.show_data.datapools.insert(2, lumacore_tree::Datapool::new(Some("Second".into())));
    state.root.show_data.after_load();
    run(&mut state, "Fixture 2");
    run(&mut state, "Store DataPool 2 Group 3");
    run(&mut state, "ResetAll");
    run(&mut state, "Fixture 1");
    let crate::DispatchResult::Ask(ask) = crate::commands::dispatch(&mut state, "Store DataPool 2 Group 3") else {
        panic!("storing over a group asks");
    };
    let lines: Vec<String> = ["Overwrite", "Merge"].iter().filter_map(|b| ask.line_of(b)).collect();
    assert!(!lines.is_empty(), "the ask has Overwrite/Merge buttons");
    assert!(lines.iter().all(|l| l.contains("DataPool 2 Group 3")), "{lines:?}");
    assert_eq!(state.datapool_override, None);
    run(&mut state, "ResetAll");
    // The editor opened owner first edits that DataPool's group.
    run(&mut state, "Edit DataPool 2 Group 3");
    run(&mut state, "Add Fixture 1");
    run(&mut state, "exit");
    assert_eq!(state.root.show_data.datapools[&2].groups[&3].members.len(), 2);
    assert!(!state.root.show_data.datapools[&1].groups.contains_key(&3));
}

/// R1-35 (Q6): `Group 1 Thru 3` with Group 2 missing selects 1 and 3 — no
/// error after a half-done selection; `Delete Fixture 1 Thru 3` deletes
/// every fixture of the range that exists.
#[test]
fn groups_ranges_skip_what_is_missing() {
    let mut state = rig(&[1, 2, 3]);
    for n in [1, 3] {
        run(&mut state, "ResetAll");
        run(&mut state, &format!("Fixture {n}"));
        run(&mut state, &format!("Store Group {n}"));
    }
    run(&mut state, "ResetAll");
    let out = run(&mut state, "Group 1 Thru 3");
    assert!(!out.contains("Error"), "{out}");
    let sel = state.current_programmer().unwrap().selected_fixtures.clone();
    assert_eq!(state.root.show_data.fixture_ids_for_guids(&sel), vec![1, 3]);
    run(&mut state, "Claim Root.ShowData.Devices");
    run(&mut state, "Delete Fixture 1 Thru 3 /nc");
    run(&mut state, "Unclaim Root.ShowData.Devices /Keep");
    assert!(state.root.show_data.devices.dmx.patch.fixtures.is_empty(), "all three went");
}

/// R1 review 10: the one range rule in every group verb — missing numbers
/// skipped, a bare `Thru`, `-` — and Copy / Move keep the spacing; a Copy
/// that can't land everywhere copies nothing.
#[test]
fn groups_ranges_follow_one_rule_and_keep_their_spacing() {
    let mut state = rig(&[1, 2]);
    for n in [1, 3, 4] {
        run(&mut state, "ResetAll");
        run(&mut state, "Fixture 1");
        run(&mut state, &format!("Store Group {n}"));
    }
    run(&mut state, "ResetAll");
    let have = |s: &AppState| s.user_groups().keys().copied().collect::<Vec<_>>();
    let out = run(&mut state, "Copy Group 1 Thru 3 At 10");
    assert!(!out.contains("Error"), "{out}");
    assert_eq!(have(&state), vec![1, 3, 4, 10, 12], "a gap stays a gap");
    assert!(matches!(crate::commands::dispatch(&mut state, "Copy Group 1 Thru 4 At 9"), crate::DispatchResult::Ask(_)), "taken: the one ask");
    assert_eq!(have(&state), vec![1, 3, 4, 10, 12], "nothing copied");
    run(&mut state, "Move Group 10 Thru 12 At 20");
    assert_eq!(have(&state), vec![1, 3, 4, 20, 22]);
    run(&mut state, "Delete Group Thru - 3 /nc");
    assert_eq!(have(&state), vec![3]);
    assert!(run(&mut state, "Delete Group 7 /nc").contains("E-CMD-007"), "the one resolver answers");
}

/// R1 review 14: a group above 9999 from an older show is named like any
/// other (called, labelled, moved down, deleted); only new numbers are capped.
#[test]
fn groups_above_the_cap_still_work_but_none_are_new() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1");
    run(&mut state, "Store Group 1");
    run(&mut state, "ResetAll");
    let dp = state.selected_datapool();
    let mut old = state.user_groups()[&1].clone();
    old.meta.identity.guid = uuid::Uuid::new_v4();
    state.root.show_data.datapools.get_mut(&dp).unwrap().groups.insert(12000, old);
    state.root.show_data.after_load();
    let out = run(&mut state, "Label Group 12000 \"Old\"");
    assert!(!out.contains("Error"), "{out}");
    assert!(!run(&mut state, "Group 12000").contains("Error"));
    assert!(run(&mut state, "Store Group 12001").contains("E-CMD-008"));
    run(&mut state, "ResetAll");
    assert!(run(&mut state, "Move Group 12000 At 5").contains('5'));
    run(&mut state, "Move Group 5 At 12000");
    assert!(state.user_groups().contains_key(&5), "moving up past the cap is refused");
    let back = state.user_groups()[&5].clone();
    let groups = &mut state.root.show_data.datapools.get_mut(&dp).unwrap().groups;
    groups.remove(&5);
    groups.insert(12000, back);
    assert!(!run(&mut state, "Delete Group 12000 /nc").contains("Error"));
    assert!(!state.user_groups().contains_key(&12000));
}

/// The RemOff macro's line (`docs/user/macros/macros.md`): every fixture
/// but the group's.
#[test]
fn groups_every_fixture_but_a_group() {
    let mut state = rig(&[1, 2, 3]);
    run(&mut state, "Fixture 2");
    run(&mut state, "Store Group 999");
    run(&mut state, "ResetAll");
    run(&mut state, "Fixture Thru - Group 999");
    assert_eq!(selection(&state), vec![1, 3]);
}

/// Every pool object carries its NodeMeta look (decisions §30 "12e"): `Set
/// Group n Color / Icon`, shown as `#rrggbb` and an icon name, `none` clears;
/// one Oops step each; the pool cell takes the colour.
#[test]
fn a_group_takes_a_colour_and_an_icon() {
    let mut state = two_groups();
    let out = run(&mut state, "Set Group 1 \"Paint\" \"#3C8F3C\"");
    assert!(out.contains("Paint = #3c8f3c"), "{out}");
    let meta = |s: &AppState| s.root.show_data.datapools[&1].groups[&1].meta.clone();
    assert_eq!(meta(&state).annotation.paint, Some(lumacore_base::Paint::Rgb([0x3c, 0x8f, 0x3c])));
    assert!(run(&mut state, "Set Group 1 \"Icon\" \"star\"").contains("Icon = star"));
    assert!(matches!(meta(&state).annotation.icon, Some(lumacore_tree::node::IconRef::Builtin(ref n)) if n == "star"));
    let settings = run(&mut state, "EditSetting Group 1");
    assert!(settings.contains("#3c8f3c") && settings.contains("Set Group 1 \"Paint\""), "{settings}");
    let bad = run(&mut state, "Set Group 1 \"Paint\" \"reddish\"");
    assert!(bad.contains("isn't a paint") && bad.contains("E-SET-003"), "{bad}");
    run(&mut state, "Oops");
    assert!(meta(&state).annotation.icon.is_none(), "Oops takes the icon back");
    run(&mut state, "Set Group 1 \"Paint\" none");
    assert_eq!(meta(&state).annotation.paint, None);
}

/// The editor tools (decisions §30 "12e"): `Grid Group n "<Tool>" ["<Arg>"]`
/// is one statement and one Oops step; a planned tool refuses; a wrong
/// choice is E-CMD-008.
#[test]
fn grid_tools_change_the_group_layout_one_step_each() {
    let mut state = two_groups();
    let spots = |s: &AppState| -> Vec<(i32, i32)> {
        s.root.show_data.datapools[&1].groups[&1].members.iter().map(|m| (m.cell.x, m.cell.y)).collect()
    };
    let before = spots(&state);
    assert!(run(&mut state, "Grid Group 1 \"Multiply\" 3").contains("Multiply \"3\""));
    assert_eq!(spots(&state), before.iter().map(|(x, y)| (x * 3, y * 3)).collect::<Vec<_>>());
    run(&mut state, "Grid Group 1 \"RemoveGaps\"");
    assert_eq!(spots(&state), before, "gaps closed again");
    run(&mut state, "Grid Group 1 \"Flip\" \"X\"");
    let flipped = spots(&state);
    assert_ne!(flipped, before);
    run(&mut state, "Oops");
    assert_eq!(spots(&state), before, "Oops undoes the flip alone");
    // Linearize by number puts the members back in fixture order.
    run(&mut state, "Set Group 1 Fixture 1 \"Order\" 3");
    run(&mut state, "Grid Group 1 \"Linearize\" \"Number\"");
    let order: Vec<u32> = {
        let g = &state.root.show_data.datapools[&1].groups[&1];
        g.members.iter().flat_map(|m| state.root.show_data.fixture_ids_for_guids(std::slice::from_ref(&m.fixture))).collect()
    };
    assert_eq!(order, vec![1, 2, 3]);
    assert!(run(&mut state, "Grid Group 1 \"Shuffle\"").contains("E-CMD-003"), "planned");
    assert!(run(&mut state, "Grid Group 1 \"Rotate\" \"Up\"").contains("E-CMD-008"));
    assert!(run(&mut state, "Grid Group 9 \"Transpose\"").contains("E-CMD-007"));
    // The grid cursor still works.
    assert!(run(&mut state, "Grid 2/1").contains("Grid cursor 2/1/0"));
    // The registry as JSON: groups, icons, questions.
    let json = crate::tools::tool_types_json();
    assert_eq!(json["groups"][0], "Order");
    let lin = json["tools"].as_array().unwrap().iter().find(|t| t["id"] == "Linearize").unwrap();
    assert_eq!(lin["icon"], "list-ordered");
    assert_eq!(lin["ask"]["options"][1]["value"], "LeftToRight");
}

/// Every built tool has its code and a planned one has none (decisions §34 21: a row's
/// behaviour is bound by its id).
#[test]
fn every_built_tool_is_bound_and_no_planned_one() {
    for tool in lumacore_registry::get().tools() {
        let bound = crate::tools::run_of(tool.id).is_some();
        assert_eq!(bound, tool.status == crate::commands::registry::Status::Ready, "tool {}", tool.id);
    }
}
