//! Step 4a behaviour (decisions §4, §7, §13, §27): programmers LTP between
//! users, fixture claims, cues with tracking, playback state, park, the
//! grand master and Solo — through real dispatch, checked on the rendered
//! frame.

use super::dispatch;
use crate::AppState;
use crate::DispatchResult;
use lumacore_fixtures::fixture_type::{AttributeType, ChannelFunction, FixtureType, Mode, Revision};
use lumacore_tree::node::NodeMeta;

pub fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// Universe 1 with one-channel dimmers: fixture n at address n.
pub fn rig(ids: &[u32]) -> AppState {
    let mut state = AppState::new();
    let patch = &mut state.root.show_data.devices.dmx.patch;
    let universe = lumacore_fixtures::Universe::new(None);
    let ug = universe.meta.identity.guid;
    patch.universes.insert(1, universe);
    let ft = FixtureType {
        meta: NodeMeta::new(Some("Dim".into())),
        manufacturer: "Test".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: vec![Revision {
            meta: NodeMeta::new(None),
            geometries: Vec::new(),
            wheels: Vec::new(),
            emitters: Vec::new(),
            filters: Vec::new(),
            attributes: Vec::new(),
            gdtf_version: "1.2".into(),
            file_hash: String::new(),
            modes: vec![Mode {
                meta: NodeMeta::new(None),
                channel_functions: vec![ChannelFunction {
                    attribute_name: "Dimmer".into(),
                    attribute_type: AttributeType::Dimmer,
                    dmx_offset: 0,
                    byte_count: 1,
                    default_value: 0,
                    default_fraction: None,
                    physical_range: None,
                    is_virtual: false,
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
    let ftg = ft.meta.identity.guid;
    let (rev, mode) = ft.mode_guids(0, 0).unwrap();
    patch.fixture_types.insert(1, std::sync::Arc::new(ft));
    for &id in ids {
        patch.insert_fid(id, lumacore_fixtures::Fixture::new(None, ftg, rev, mode, ug, id as u16, 1),
        );
    }
    state.root.show_data.after_load();
    // As a patch through the change path would leave it: every attribute
    // defined (decisions §30 "12f-a").
    lumacore_tree::attribute_resolver::sync(&mut state.root.show_data);
    state.root.show_data.attribute_defs.values_mut().for_each(|d| d.new = false);
    state
}

/// What fixture `id` outputs (its address in universe 1).
pub(super) fn dmx(state: &AppState, id: u32) -> u8 {
    state.published_frame(1).unwrap()[id as usize - 1]
}

fn as_user(state: &mut AppState, user: &str) {
    state.user = user.to_string();
    state.root.runtime.user(user);
}

#[test]
fn playback_programmers_are_ltp_and_a_touch_moves_the_value() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Fixture 1 At 50");
    assert_eq!(dmx(&state, 1), 128);
    // Debug can select the same fixture: selecting claims nothing.
    as_user(&mut state, "Debug");
    assert!(run(&mut state, "Fixture 1").contains("Selected"));
    run(&mut state, "At 100");
    assert_eq!(dmx(&state, 1), 255);
    // The value moved: Admin's programmer no longer has it.
    let g1 = state.root.show_data.fixture_guid(1).unwrap();
    let admin = &state.root.show_data.users[&1].programmer;
    assert!(admin.get_value(g1, "Dimmer").is_none());
    // In Blind, Debug's values don't touch the stage or take anything …
    as_user(&mut state, "Admin");
    run(&mut state, "Fixture 1 At 20");
    as_user(&mut state, "Debug");
    run(&mut state, "On Blind");
    run(&mut state, "Fixture 1 At 70");
    assert_eq!(dmx(&state, 1), 51);
    // … and leaving Blind is a fresh touch: Debug takes over.
    run(&mut state, "Off Blind");
    assert_eq!(dmx(&state, 1), 179);
    let admin = &state.root.show_data.users[&1].programmer;
    assert!(admin.get_value(g1, "Dimmer").is_none());
}

#[test]
fn playback_claimed_fixtures_are_locked_for_others() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Fixture 2 At 30");
    // Admin holds Fixture 2: Debug can't claim it …
    as_user(&mut state, "Debug");
    let out = run(&mut state, "Claim Fixture 2");
    assert!(out.contains("E-PROG-002"), "{out}");
    // … but can claim Fixture 1, which Admin then can't select or set.
    assert!(run(&mut state, "Claim Fixture 1").contains("Claimed"));
    as_user(&mut state, "Admin");
    assert!(run(&mut state, "Fixture 1").contains("E-PROG-001"));
    assert!(run(&mut state, "Fixture 1 At 50").contains("E-PROG-001"));
    // Admin clears Debug's programmer.
    as_user(&mut state, "Debug");
    run(&mut state, "Fixture 1 At 40");
    as_user(&mut state, "Admin");
    assert!(run(&mut state, "Clear Programmer User 2").contains("Debug"));
    assert!(
        state.root.show_data.users[&2]
            .programmer
            .values
            .is_empty()
    );
}

/// R1-43: values B prepared in Blind don't take a fixture A claimed since —
/// neither when B leaves Blind nor when B knocks them in (`On`).
#[test]
fn playback_leaving_blind_and_knock_in_respect_claims() {
    let mut state = rig(&[5]);
    let g5 = state.root.show_data.fixture_guid(5).unwrap();
    run(&mut state, "Fixture 5");
    run(&mut state, "Store Group 1");
    run(&mut state, "Fixture 5 At 20");
    as_user(&mut state, "Debug");
    run(&mut state, "On Blind");
    run(&mut state, "Fixture 5 At 70");
    as_user(&mut state, "Admin");
    assert!(run(&mut state, "Claim Fixture 5").contains("Claimed"));
    as_user(&mut state, "Debug");
    run(&mut state, "Off Blind");
    let admin = &state.root.show_data.users[&1].programmer;
    assert!(admin.get_value(g5, "Dimmer").is_some(), "A's value stays on A's claimed fixture");
    // (Which of the two values the merge outputs is open: R1 Q29.)
    // Knocking in B's values is refused: by the fixture (dispatch's claim
    // check) and through a group (the knock in itself).
    let out = run(&mut state, "On Fixture 5");
    assert!(out.contains("E-CLAIM-001") || out.contains("E-PROG-001"), "{out}");
    let out = run(&mut state, "On Group 1");
    assert!(out.contains("E-PROG-001"), "{out}");
    let admin = &state.root.show_data.users[&1].programmer;
    assert!(admin.get_value(g5, "Dimmer").is_some());
}

#[test]
fn playback_cues_track_and_go_goto_off_snap() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Fixture 2 At 50");
    assert!(run(&mut state, "Store Sequence 1 Cue 1").contains("2 values"));
    run(&mut state, "Select Sequence 1");
    run(&mut state, "Fixture 2 At 10");
    // Fixture 1 is unchanged: it is stored as a blocked value (Q16 b).
    let out = run(&mut state, "Store Cue 2");
    assert!(out.contains("2 values") && out.contains("1 blocked"), "{out}");
    run(&mut state, "Fixture 1 At Release");
    run(&mut state, "Store Cue 3");
    run(&mut state, "ResetAll");
    assert_eq!(dmx(&state, 1), 0);

    run(&mut state, "Go");
    assert_eq!((dmx(&state, 1), dmx(&state, 2)), (255, 128));
    run(&mut state, "Go Sequence 1");
    // Fixture 1 tracks from cue 1.
    assert_eq!((dmx(&state, 1), dmx(&state, 2)), (255, 26));
    run(&mut state, "Go");
    assert_eq!(dmx(&state, 1), 0, "cue 3 releases fixture 1");
    run(&mut state, "Go-");
    assert_eq!(dmx(&state, 1), 255);
    run(&mut state, "Goto Cue 1");
    assert_eq!(dmx(&state, 2), 128);
    assert!(run(&mut state, "List Sequence 1").contains("▶"));
    // Running state is saved in the show and never undone.
    let seq1 = state.root.show_data.datapools[&1].sequences[&1].meta.identity.guid;
    assert!(state.root.show_data.playbacks.live.sequences.contains_key(&seq1));
    run(&mut state, "Off Sequence 1");
    assert_eq!(dmx(&state, 1), 0);
    run(&mut state, "Oops");
    assert!(!state.root.show_data.playbacks.live.sequences.contains_key(&seq1));
    assert!(run(&mut state, "Goto Cue 9").contains("E-PLAY-001"));
}

#[test]
fn playback_programmer_and_cues_merge_ltp_and_freeze_lifts() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Select Sequence 1");
    run(&mut state, "ResetAll");
    run(&mut state, "Fixture 1 At 20");
    // The cue goes after the programmer value: the cue wins.
    run(&mut state, "Go");
    assert_eq!(dmx(&state, 1), 255);
    // A new touch wins again.
    run(&mut state, "Fixture 1 At 30");
    assert_eq!(dmx(&state, 1), 77);
    // Freeze keeps the programmer above later cues.
    run(&mut state, "On Freeze");
    run(&mut state, "Goto Cue 1");
    assert_eq!(dmx(&state, 1), 77);
    run(&mut state, "Off Freeze");
    assert_eq!(dmx(&state, 1), 255);
}

#[test]
fn playback_park_grand_master_and_solo() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Fixture 1 Thru 2 At 100");
    // Grand master scales intensity; switched off it is a blackout.
    run(&mut state, "Master 2.1 At 50");
    assert_eq!(dmx(&state, 1), 128);
    run(&mut state, "Off Master 2.1");
    assert_eq!(dmx(&state, 2), 0);
    // Parked values ignore the masters.
    assert!(run(&mut state, "Park Fixture 1 At 40").contains("1 value"));
    assert_eq!(dmx(&state, 1), 102);
    run(&mut state, "On Master 2.1");
    run(&mut state, "Master 2.1 At 100");
    // Bare Park freezes the selection as it looks now.
    run(&mut state, "Fixture 2");
    run(&mut state, "Park");
    run(&mut state, "Fixture 2 At 0");
    assert_eq!(dmx(&state, 2), 255);
    run(&mut state, "Unpark Fixture 2");
    assert_eq!(dmx(&state, 2), 0);
    // A raw channel stays parkable (unpatched channels).
    run(&mut state, "Park DMX 1.10 At 77");
    assert_eq!(dmx(&state, 10), 77);
    // Solo: fixtures outside the soloing user's selection go dark (not parked ones).
    run(&mut state, "Unpark");
    run(&mut state, "Fixture 1 Thru 2 At 100");
    run(&mut state, "Fixture 2");
    run(&mut state, "On Solo");
    assert_eq!((dmx(&state, 1), dmx(&state, 2)), (0, 255));
    run(&mut state, "Off Solo");
    assert_eq!(dmx(&state, 1), 255);
}

#[test]
fn playback_state_survives_a_save_and_load() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Select Sequence 1");
    run(&mut state, "ResetAll");
    run(&mut state, "Go");
    let json = serde_json::to_string(&state.root.show_data).unwrap();
    let mut loaded = AppState::new();
    crate::objects::replace_show(&mut loaded, serde_json::from_str(&json).unwrap());
    // Resumed, snapped.
    assert_eq!(dmx(&loaded, 1), 255);
}

/// One-only selections (decisions §29): the selected sequence is what bare
/// playback words act on; nothing selected says so; Select takes only
/// DataPool / Sequence / Timecode; deleting the object clears it.
#[test]
fn the_selected_sequence_is_the_default_and_never_guessed() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 2 Cue 1");
    assert!(run(&mut state, "Go").contains("E-PLAY-003"), "nothing selected");
    assert!(run(&mut state, "Select Sequence 9").contains("E-PLAY-001"));
    assert!(run(&mut state, "Select Group 1").contains("E-CMD-014"));
    assert!(run(&mut state, "Select Fixture 1").contains("E-CMD-014"));
    assert!(run(&mut state, "Select Root.ShowData.DataPools.1.Sequences.2").contains("Sequence 2 selected"));
    assert_eq!(state.selected_sequence(), Some((1, 2)));
    let go = run(&mut state, "Go");
    assert!(!go.contains("Error"), "{go}");
    run(&mut state, "Deselect Sequence");
    assert_eq!(state.selected_sequence(), None);
    run(&mut state, "Select Sequence 2");
    // The sequence goes away (as a delete would): the next line clears it.
    state.root.show_data.datapools.get_mut(&1).unwrap().sequences.remove(&2);
    run(&mut state, "Fixture 1 At 50");
    let user = state.effective_user_name().to_string();
    assert_eq!(state.root.show_data.user(&user).unwrap().selected.sequence, None, "deleted → cleared");
}

/// Cue timing (decisions §29): a cue's newer values fade from where the
/// channel is, by its part's timing, on the session clock; a dimmer going
/// down takes the out fade.
#[test]
fn cue_parts_fade_from_where_the_channel_is() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 0");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 2");
    run(&mut state, "Select Sequence 1");
    run(&mut state, "ResetAll");
    assert!(run(&mut state, "Set Cue 2 \"Fade\" 2").contains("Fade 2"));
    assert!(run(&mut state, "Set Cue 1 \"OutFade\" 4").contains("OutFade 4"));
    let plan_at = |state: &AppState, previous: Option<&lumacore_render::Plan>, now: u64| {
        let input = state.engine_input(std::sync::Arc::new(state.root.show_data.clone()));
        lumacore_render::Plan::compile_after(&input, previous.map(|p| &p.channel), previous, now)
    };
    let dimmer = |plan: &lumacore_render::Plan| {
        let i = plan.channel.channels.iter().position(|c| c.dimmer).expect("a dimmer");
        plan.value.channels[i]
    };
    run(&mut state, "Go");
    let one = plan_at(&state, None, 0);
    assert_eq!(dimmer(&one).at(0), 0.0);
    run(&mut state, "Go");
    let two = plan_at(&state, Some(&one), 1_000);
    let ch = dimmer(&two);
    assert!((ch.at(1_000) - 0.0).abs() < 1e-9, "starts where it was");
    assert!((ch.at(2_000) - 0.5).abs() < 1e-9, "half way after 1 of 2 s: {}", ch.at(2_000));
    assert_eq!(ch.at(3_000), 1.0);
    // Back to cue 1 in the middle of the fade: from where it is, out fade.
    run(&mut state, "Goto Cue 1");
    let back = plan_at(&state, Some(&two), 2_000);
    let ch = dimmer(&back);
    assert!((ch.at(2_000) - 0.5).abs() < 1e-9, "from where it was");
    assert!((ch.at(4_000) - 0.25).abs() < 1e-9, "dimmer down: the 4 s out fade");
}

/// Highlight (decisions §29): GDTF highlight values only, per user's
/// selection, scaled by the user's Highlight master; Solo by its master.
#[test]
fn highlight_uses_gdtf_values_and_the_masters_scale() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1");
    run(&mut state, "At 10");
    let dimmer = |state: &AppState| {
        let plan = state.output_plan();
        let i = plan.channel.channels.iter().position(|c| c.dimmer).expect("a dimmer");
        plan.value.channels[i].at(0)
    };
    // The rig's fixture type has no highlight values: nothing happens.
    run(&mut state, "On Highlight");
    assert!((dimmer(&state) - 0.1).abs() < 1e-9);
    // Give its dimmer a GDTF highlight value of full.
    for ft in state.root.show_data.devices.dmx.patch.fixture_types.values_mut().map(std::sync::Arc::make_mut) {
        for rev in &mut ft.revisions {
            for mode in &mut rev.modes {
                for cf in &mut mode.channel_functions {
                    cf.highlight = Some(1.0);
                }
            }
        }
    }
    state.patch_generation += 1;
    assert!((dimmer(&state) - 1.0).abs() < 1e-9, "highlighted: {}", dimmer(&state));
    assert!(run(&mut state, "Master \"Highlight\" At 50").contains("50%"));
    assert!((dimmer(&state) - 0.55).abs() < 1e-9, "half way: {}", dimmer(&state));
    run(&mut state, "Set Master \"Highlight\" \"Mode\" Disabled");
    assert!((dimmer(&state) - 1.0).abs() < 1e-9);
    run(&mut state, "Off Highlight");
    assert!((dimmer(&state) - 0.1).abs() < 1e-9);
    run(&mut state, "Set Master \"Highlight\" \"Mode\" AlwaysOn");
    assert!((dimmer(&state) - 0.55).abs() < 1e-9, "on without the key: {}", dimmer(&state));
    assert!(run(&mut state, "Master \"Solo\"").contains("100%"));
    assert!(run(&mut state, "Master 7 At 5").contains("E-CMD-007"));
}

/// The plan the engine would compile now, after `previous` (session `now`).
fn plan_at(state: &AppState, previous: Option<&lumacore_render::Plan>, now: u64) -> lumacore_render::Plan {
    let input = state.engine_input(std::sync::Arc::new(state.root.show_data.clone()));
    lumacore_render::Plan::compile_after(&input, previous.map(|p| &p.channel), previous, now)
}

/// Fixture `id`'s dimmer channel in `plan` (found by the fixture's GUID).
fn dimmer_of(state: &AppState, plan: &lumacore_render::Plan, id: u32) -> lumacore_render::plan::ChannelValue {
    let guid = state.root.show_data.fixture_guid(id).expect("patched");
    let f = plan.channel.fixture_index[&guid];
    let i = plan
        .channel
        .channels
        .iter()
        .position(|c| c.dimmer && c.fixture == f)
        .expect("a dimmer");
    plan.value.channels[i]
}

/// Cue 1 (Fixture 1 at 100, Fade 10) in Sequence 1, selected, the
/// programmer empty; and the plan before any Go.
fn faded_cue() -> (AppState, lumacore_render::Plan) {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Select Sequence 1");
    run(&mut state, "Set Cue 1 \"Fade\" 10");
    run(&mut state, "ResetAll");
    let dark = plan_at(&state, None, 0);
    (state, dark)
}

/// R1-26: a Devices change (a new channel tier) keeps running fades — the
/// old→new index map carries them by (fixture GUID, attribute, ordinal).
#[test]
fn a_patch_change_keeps_running_fades() {
    let (mut state, dark) = faded_cue();
    run(&mut state, "Go");
    let going = plan_at(&state, Some(&dark), 0);
    assert!(dimmer_of(&state, &going, 1).at(0).abs() < 1e-9, "starts dark");
    // Fixture 1 becomes number 3 (same GUID); a new fixture takes number
    // 1 at address 2, so every channel index shifts.
    let patch = &mut state.root.show_data.devices.dmx.patch;
    let moved = patch.remove_fid(1).unwrap();
    let added = lumacore_fixtures::Fixture::new(
        None,
        moved.mvr.fixture_type,
        moved.mvr.revision,
        moved.mvr.mode,
        moved.mvr.breaks[0].universe_id,
        2,
        1,
    );
    patch.insert_fid(3, moved);
    patch.insert_fid(1, added);
    state.root.show_data.after_load();
    state.patch_generation += 1;
    let later = plan_at(&state, Some(&going), 5_000);
    assert!(!std::ptr::eq(later.channel.as_ref(), going.channel.as_ref()), "a new tier");
    assert_eq!(later.channel.channels.len(), 2);
    let ch = dimmer_of(&state, &later, 3);
    assert!((ch.at(5_000) - 0.5).abs() < 1e-9, "still half way: {}", ch.at(5_000));
    assert!(ch.fade.is_some_and(|f| f.running(5_000)), "still fading");
    assert_eq!(ch.at(10_000), 1.0);
    assert!(dimmer_of(&state, &later, 1).fade.is_none(), "the new fixture has none");
}

/// R1-27: only a Go / Goto / Go- of the winning sequence starts a cue fade;
/// a handover (Clear, Blind, Off Freeze, Unpark) snaps.
#[test]
fn handovers_to_a_cue_snap() {
    // Clear: the programmer's 0 goes, the cue's full is back at once.
    let (mut state, dark) = faded_cue();
    run(&mut state, "Go");
    let going = plan_at(&state, Some(&dark), 0);
    let full = plan_at(&state, Some(&going), 20_000);
    run(&mut state, "Fixture 1 At 0");
    let held = plan_at(&state, Some(&full), 21_000);
    assert_eq!(dimmer_of(&state, &held, 1).at(21_000), 0.0);
    // `Reset` is the programmer's Clear (three stages).
    for _ in 0..3 {
        run(&mut state, "Reset");
    }
    assert!(state.current_programmer().unwrap().values.is_empty(), "cleared");
    let cleared = plan_at(&state, Some(&held), 22_000);
    let ch = dimmer_of(&state, &cleared, 1);
    assert!(ch.fade.is_none(), "Clear snaps: {:?}", ch.fade);
    assert_eq!(ch.at(22_000), 1.0);

    // Blind: the Blind user's programmer leaves the stage: a snap.
    run(&mut state, "Fixture 1 At 0");
    let held = plan_at(&state, Some(&cleared), 23_000);
    run(&mut state, "On Blind");
    let blind = plan_at(&state, Some(&held), 24_000);
    let ch = dimmer_of(&state, &blind, 1);
    assert!(ch.fade.is_none(), "Blind snaps: {:?}", ch.fade);
    assert_eq!(ch.at(24_000), 1.0);

    // Off Freeze: the Go under Freeze already happened; leaving Freeze is
    // a handover.
    let (mut state, dark) = faded_cue();
    run(&mut state, "On Freeze");
    run(&mut state, "Fixture 1 At 0");
    let frozen = plan_at(&state, Some(&dark), 0);
    run(&mut state, "Go");
    let under = plan_at(&state, Some(&frozen), 1_000);
    assert_eq!(dimmer_of(&state, &under, 1).at(1_000), 0.0, "frozen wins");
    run(&mut state, "Off Freeze");
    let off = plan_at(&state, Some(&under), 2_000);
    let ch = dimmer_of(&state, &off, 1);
    assert!(ch.fade.is_none(), "Off Freeze snaps: {:?}", ch.fade);
    assert_eq!(ch.at(2_000), 1.0);

    // Unpark: back to the cue at once.
    run(&mut state, "ResetAll");
    run(&mut state, "Park Fixture 1 At 0");
    let parked = plan_at(&state, Some(&off), 3_000);
    assert_eq!(dimmer_of(&state, &parked, 1).at(3_000), 0.0);
    run(&mut state, "Unpark Fixture 1");
    let unparked = plan_at(&state, Some(&parked), 4_000);
    let ch = dimmer_of(&state, &unparked, 1);
    assert!(ch.fade.is_none(), "Unpark snaps: {:?}", ch.fade);
    assert_eq!(ch.at(4_000), 1.0);

    // A Go still fades.
    run(&mut state, "Goto Cue 1");
    let again = plan_at(&state, Some(&unparked), 5_000);
    assert!(dimmer_of(&state, &again, 1).fade.is_none(), "same value: nothing to fade");
    run(&mut state, "Off Sequence 1");
    let off = plan_at(&state, Some(&again), 6_000);
    run(&mut state, "Go");
    let go = plan_at(&state, Some(&off), 7_000);
    assert!(dimmer_of(&state, &go, 1).fade.is_some(), "a Go fades");
}

/// R1-27: a fade under Highlight starts from the channel's own value, not
/// from the highlighted one.
#[test]
fn a_fade_under_highlight_starts_from_the_unhighlighted_value() {
    let (mut state, _) = faded_cue();
    for ft in state.root.show_data.devices.dmx.patch.fixture_types.values_mut().map(std::sync::Arc::make_mut) {
        for rev in &mut ft.revisions {
            for mode in &mut rev.modes {
                for cf in &mut mode.channel_functions {
                    cf.highlight = Some(1.0);
                }
            }
        }
    }
    state.patch_generation += 1;
    run(&mut state, "Fixture 1");
    run(&mut state, "On Highlight");
    let lit = plan_at(&state, None, 0);
    assert_eq!(dimmer_of(&state, &lit, 1).at(0), 1.0, "highlighted");
    run(&mut state, "Go");
    let going = plan_at(&state, Some(&lit), 0);
    run(&mut state, "Off Highlight");
    let plain = plan_at(&state, Some(&going), 5_000);
    let ch = dimmer_of(&state, &plain, 1);
    assert_eq!(ch.fade.map(|f| f.from), Some(0.0), "from the un-highlighted 0");
    assert!((ch.at(5_000) - 0.5).abs() < 1e-9, "half way: {}", ch.at(5_000));
}

/// R1-27: storing into the running cue mid-fade retargets the fade from
/// where the channel is, over the time left.
#[test]
fn a_store_into_the_running_cue_retargets_its_fade() {
    let (mut state, dark) = faded_cue();
    run(&mut state, "Go");
    let going = plan_at(&state, Some(&dark), 0);
    run(&mut state, "Fixture 1 At 80");
    let held = plan_at(&state, Some(&going), 5_000);
    run(&mut state, "Store Cue 1 /Merge");
    run(&mut state, "ResetAll");
    // Compiled from the plan before the programmer touch (the engine may
    // never have rendered the touch).
    let stored = plan_at(&state, Some(&going), 5_000);
    let ch = dimmer_of(&state, &stored, 1);
    assert!((ch.at(5_000) - 0.5).abs() < 1e-9, "no jump: {}", ch.at(5_000));
    assert!(ch.fade.is_some_and(|f| f.running(5_000)), "still fading");
    assert!((ch.at(7_500) - 0.65).abs() < 1e-9, "towards 80 %: {}", ch.at(7_500));
    assert!((ch.at(10_000) - 0.8).abs() < 1e-9, "there at 10 s");
    let _ = held;
}

/// R1-30: a typed cue number with more than 3 decimals is refused — never
/// rounded into another cue.
#[test]
fn cue_numbers_with_more_than_three_decimals_are_refused() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Store Sequence 1 Cue 2");
    for bad in ["1.9999", "1.0004"] {
        let out = run(&mut state, &format!("Store Sequence 1 Cue {bad}"));
        assert!(out.contains("E-CMD-008"), "{bad}: {out}");
    }
    // A number starts with a digit (lum.md): `0.5`, not `.5`.
    for good in ["1.5", "2.25", "0.5"] {
        let out = run(&mut state, &format!("Store Sequence 1 Cue {good}"));
        assert!(out.contains("Stored"), "{good}: {out}");
    }
    let keys: Vec<u32> = state.root.show_data.datapools[&1].sequences[&1].cues.keys().copied().collect();
    assert_eq!(keys, vec![500, 1000, 1500, 2000, 2250]);
}

/// R1-30: Goto on a running sequence keeps its fader, rate, paused and who
/// started it; only the cue and the asserts change.
#[test]
fn goto_on_a_running_sequence_keeps_its_state() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Fixture 1 At 50");
    run(&mut state, "Store Sequence 1 Cue 2");
    run(&mut state, "Select Sequence 1");
    run(&mut state, "Go");
    run(&mut state, "Go");
    let guid = state.root.show_data.datapools[&1].sequences[&1].meta.identity.guid;
    let starter = uuid::Uuid::new_v4();
    {
        let st = state.root.show_data.playbacks.live.sequences.get_mut(&guid).unwrap();
        st.fader = 0.5;
        st.rate = 2.0;
        st.paused = true;
        st.started_by = starter;
    }
    let before = state.root.show_data.playbacks.live.sequences[&guid].clone();
    as_user(&mut state, "Debug");
    run(&mut state, "Goto Sequence 1 Cue 1");
    let st = &state.root.show_data.playbacks.live.sequences[&guid];
    assert_eq!(st.current_cue, 1000);
    assert_eq!((st.fader, st.rate, st.paused, st.started_by), (0.5, 2.0, true, starter));
    assert!(st.asserted > before.asserted, "asserted again");
    assert!(st.go_stamps.is_empty());
}

/// R1-31: after Store the cue plays what the programmer showed.
#[test]
fn store_cue_makes_the_cue_play_the_programmer() {
    // A value equal to what tracks in takes the cue's own value out.
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 50");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Fixture 1 At 80");
    run(&mut state, "Store Sequence 1 Cue 2");
    run(&mut state, "Select Sequence 1");
    run(&mut state, "Fixture 1 At 50");
    let out = run(&mut state, "Store Cue 2 /Merge");
    assert!(out.contains("1 blocked"), "{out}");
    run(&mut state, "ResetAll");
    run(&mut state, "Goto Cue 2");
    assert_eq!(dmx(&state, 1), 128, "cue 2 plays 50 %");
    // Blocked: a later change in cue 1 doesn't track through cue 2 (Q16 b).
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Cue 1 /Overwrite");
    run(&mut state, "ResetAll");
    run(&mut state, "Goto Cue 2");
    assert_eq!(dmx(&state, 1), 128, "the blocked 50 % holds");

    // Tracking off: nothing tracks in, so the equal value is stored.
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 50");
    run(&mut state, "Store Sequence 1 Cue 1");
    state.root.show_data.sequence_mut(1, 1).unwrap().settings.tracking = false;
    let out = run(&mut state, "Store Sequence 1 Cue 2");
    assert!(out.contains("1 value") && !out.contains("blocked"), "{out}");
    run(&mut state, "ResetAll");
    run(&mut state, "Goto Sequence 1 Cue 2");
    assert_eq!(dmx(&state, 1), 128, "cue 2 plays 50 % with tracking off");

    // A value in part 1 is replaced, not left to shadow part 0.
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 50");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Store Sequence 1 Cue 2");
    let g1 = state.root.show_data.fixture_guid(1).unwrap();
    let cue2 = state.root.show_data.sequence_mut(1, 1).unwrap().cues.get_mut(&2000).unwrap();
    cue2.parts.insert(
        1,
        lumacore_tree::CuePart {
            values: vec![lumacore_tree::CueValue {
                fixture_id: g1,
                attribute: "Dimmer".into(),
                timing: Default::default(),
                value: lumacore_tree::CueData::Value(lumacore_tree::Value::normalized(80.0)),
            }],
            ..Default::default()
        },
    );
    run(&mut state, "Fixture 1 At 30");
    run(&mut state, "Store Sequence 1 Cue 2 /Merge");
    let cue2 = &state.root.show_data.datapools[&1].sequences[&1].cues[&2000];
    assert_eq!(cue2.values().count(), 1, "one value left: {:?}", cue2.parts);
    run(&mut state, "ResetAll");
    run(&mut state, "Goto Sequence 1 Cue 2");
    assert_eq!(dmx(&state, 1), 77, "cue 2 plays 30 %");
}

/// R1-39: `Select Sequence 3` selects sequence number 3, even when sequence
/// 1 is named "3"; a name is used only when no number matches.
#[test]
fn playback_select_by_number_beats_a_name() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Store Sequence 3 Cue 1");
    state.root.show_data.sequence_mut(1, 1).unwrap().meta.identity.name = Some("3".into());
    state.root.show_data.sequence_mut(1, 3).unwrap().meta.identity.name = Some("Main".into());
    run(&mut state, "Select Sequence 3");
    assert_eq!(state.selected_sequence(), Some((1, 3)));
    run(&mut state, "Select Sequence Main");
    assert_eq!(state.selected_sequence(), Some((1, 3)));
}

/// R1-32: an `If` filter narrows every word that takes a selection — a value
/// line and Full — instead of being parsed and dropped.
#[test]
fn playback_if_filters_value_lines_and_full() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Fixture 1 At 80");
    run(&mut state, "Fixture 2 At 10");
    let dimmer = |s: &AppState, n: u32| {
        let guid = s.root.show_data.fixture_guid(n).unwrap();
        s.current_programmer().and_then(|p| p.get_value(guid, "Dimmer")).and_then(lumacore_base::Value::percent)
    };
    run(&mut state, "Fixture 1 Thru 2 If Dimmer > 50 At 20");
    assert_eq!(dimmer(&state, 1), Some(20.0));
    assert_eq!(dimmer(&state, 2), Some(10.0), "fixture 2 fails the If");
    run(&mut state, "Full Fixture 1 Thru 2 If Dimmer < 15");
    assert_eq!(dimmer(&state, 1), Some(20.0), "fixture 1 fails the If");
    assert_eq!(dimmer(&state, 2), Some(100.0));
}

/// R1-36: a value for an attribute the fixture doesn't have is refused (it
/// was stored and never output); a name in another case takes the mode's
/// spelling.
#[test]
fn playback_values_only_for_attributes_the_fixture_has() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1");
    let out = run(&mut state, "Attribute \"Pan\" At 50");
    assert!(out.contains("E-CMD-009") && out.contains("Pan"), "{out}");
    run(&mut state, "Attribute \"dimmer\" At 40");
    let guid = state.root.show_data.fixture_guid(1).unwrap();
    let prog = state.current_programmer().unwrap();
    assert!(prog.get_value(guid, "Dimmer").is_some(), "stored under the mode's spelling");
    assert!(prog.get_value(guid, "dimmer").is_none() && prog.get_value(guid, "Pan").is_none());
    // R1-37: a relative value after the attribute takes both words.
    run(&mut state, "Attribute \"Dimmer\" At + 10");
    let prog = state.current_programmer().unwrap();
    assert_eq!(prog.get_value(guid, "Dimmer").and_then(lumacore_base::Value::percent), Some(50.0));
}

/// R1-37 (Q8): `At + 10` adds to your programmer value, else to what
/// the fixture outputs now (a cue at 30 % → 40 %), not to 0.
#[test]
fn playback_relative_adds_to_the_output_without_a_programmer_value() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 30");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "ResetAll");
    run(&mut state, "ResetAll");
    run(&mut state, "ResetAll");
    run(&mut state, "Go Sequence 1");
    run(&mut state, "Fixture 1");
    run(&mut state, "At + 10");
    let guid = state.root.show_data.fixture_guid(1).unwrap();
    let v = state.current_programmer().and_then(|p| p.get_value(guid, "Dimmer")).and_then(lumacore_base::Value::percent);
    assert_eq!(v.map(f64::round), Some(40.0));
    run(&mut state, "At + 10");
    let v = state.current_programmer().and_then(|p| p.get_value(guid, "Dimmer")).and_then(lumacore_base::Value::percent);
    assert_eq!(v.map(f64::round), Some(50.0), "then to the programmer value");
}

/// R1-38 (Q9): deleting a fixture removes its programmer values and its
/// place in every selection in the same step; Oops brings both back.
#[test]
fn playback_a_deleted_fixture_leaves_no_values_behind() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Fixture 1 Thru 2");
    run(&mut state, "At 50");
    let g1 = state.root.show_data.fixture_guid(1).unwrap();
    let held = |s: &AppState| {
        let p = s.current_programmer().unwrap();
        (p.get_value(g1, "Dimmer").is_some(), p.selected_fixtures.contains(&g1))
    };
    assert_eq!(held(&state), (true, true));
    run(&mut state, "Claim Root.ShowData.Devices");
    let out = run(&mut state, "Delete Fixture 1 /nc");
    assert!(!state.root.show_data.devices.dmx.patch.has_fid(1), "{out}");
    assert_eq!(held(&state), (true, true), "the draft isn't live yet: nothing is cleaned");
    run(&mut state, "Unclaim Root.ShowData.Devices /Keep");
    assert_eq!(held(&state), (false, false));
    run(&mut state, "Oops");
    assert!(state.root.show_data.devices.dmx.patch.has_fid(1));
    assert_eq!(held(&state), (true, true), "one step: Oops brings both back");
}

/// R1-50 (Q19): an address on a universe that doesn't exist creates it and says so — it never
/// waits for an answer on the server's terminal (`Store Fixture … At DMX 99.001`, round 3 19).
#[test]
fn playback_patch_into_a_missing_universe_creates_it() {
    let mut state = rig(&[1]);
    run(&mut state, "Claim Root.ShowData.Devices");
    let out = run(&mut state, "Store Fixture 2 At DMX 99.001 /FixtureType 1");
    assert!(out.contains("Universe 99 created") && out.contains("Stored"), "{out}");
    let patch = &state.root.show_data.devices.dmx.patch;
    assert!(patch.universes.contains_key(&99));
    // A new universe never takes 99 again.
    let mut show = state.root.show_data.clone();
    let mut ev = Vec::new();
    for _ in 0..120 {
        let id = lumacore_tree::UniverseOps::create_universe(&mut show, None, &mut ev).unwrap();
        assert_ne!(id, 99);
    }
}

/// The usual console convention: `At 10 Attribute "Pan"` sets that attribute like
/// `Attribute "Pan" At 10`; the Dimmer is untouched.
#[test]
fn playback_value_before_the_attribute() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1");
    run(&mut state, "At 70");
    let out = run(&mut state, "At 10 Attribute \"Pan\"");
    assert!(out.contains("E-CMD-009"), "the rig has no Pan: {out}");
    let guid = state.root.show_data.fixture_guid(1).unwrap();
    let dim = |s: &AppState| s.current_programmer().and_then(|p| p.get_value(guid, "Dimmer")).and_then(lumacore_base::Value::percent);
    assert_eq!(dim(&state).map(f64::round), Some(70.0), "the Dimmer is untouched");
    run(&mut state, "At 30 Attribute \"Dimmer\"");
    assert_eq!(dim(&state).map(f64::round), Some(30.0));
}

/// R1-62 (Q10): in a line with Oops each statement is its
/// own step and Oops undoes the newest one before it.
#[test]
fn playback_oops_in_a_line_undoes_the_statement_before_it() {
    let mut state = rig(&[1, 2, 3]);
    run(&mut state, "Fixture 1");
    run(&mut state, "Fixture 2; Fixture 3; Oops");
    let selected = state.current_programmer().unwrap().selected_fixtures.clone();
    let ids = state.root.show_data.fixture_ids_for_guids(&selected);
    assert_eq!(ids, vec![1, 2], "Fixture 3 undone, Fixture 2 kept");
    run(&mut state, "Oops");
    let selected = state.current_programmer().unwrap().selected_fixtures.clone();
    assert_eq!(state.root.show_data.fixture_ids_for_guids(&selected), vec![1], "Fixture 2 was its own step");
}

/// R1-27 (Q12): a release stored in a timed cue fades by that cue's timing
/// (dimmer down = out-fade) when a Go brings the cue; it no longer snaps.
#[test]
fn a_release_in_a_timed_cue_fades() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Select Sequence 1");
    run(&mut state, "Fixture 1 At Release");
    run(&mut state, "Store Cue 2");
    run(&mut state, "Set Cue 2 \"OutFade\" 4");
    run(&mut state, "ResetAll");
    run(&mut state, "ResetAll");
    run(&mut state, "ResetAll");
    run(&mut state, "Go");
    let full = plan_at(&state, None, 0);
    let full = plan_at(&state, Some(&full), 10_000);
    assert_eq!(dimmer_of(&state, &full, 1).at(10_000), 1.0);
    run(&mut state, "Go");
    let released = plan_at(&state, Some(&full), 20_000);
    let ch = dimmer_of(&state, &released, 1);
    assert!(ch.fade.is_some(), "the release fades");
    assert!((ch.at(22_000) - 0.5).abs() < 1e-9, "half way after 2 of 4 s: {}", ch.at(22_000));
    assert_eq!(ch.at(24_000), 0.0);
    // The next plan (nothing new) lets the fade run on.
    let later = plan_at(&state, Some(&released), 21_000);
    assert!(dimmer_of(&state, &later, 1).fade.is_some_and(|f| f.running(21_000)));
}

/// R1-30 (Q15): `Go Cue 5` / `Go- Cue 2` are refused, never run ignoring the cue.
#[test]
fn playback_go_with_a_cue_is_refused() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Select Sequence 1");
    let out = run(&mut state, "Go Cue 5");
    assert!(out.contains("E-CMD-006") && out.contains("Goto Cue 5"), "{out}");
    assert!(state.root.show_data.playbacks.live.sequences.is_empty(), "nothing ran");
    assert!(run(&mut state, "Go- Cue 2").contains("E-CMD-006"));
}

/// R1 review 10: `Delete Universe 1 Thru 4` follows the one range rule —
/// missing ones skipped; a universe in use goes too and its fixtures lose
/// their address there (decisions §11 "Issue 11 hardening" 5).
#[test]
fn playback_delete_universe_takes_a_range() {
    let mut state = rig(&[1]);
    let mut ev = Vec::new();
    for _ in 0..3 {
        lumacore_tree::UniverseOps::create_universe(&mut state.root.show_data, None, &mut ev).unwrap();
    }
    state.root.show_data.devices.dmx.patch.universes.remove(&3);
    run(&mut state, "Claim Root.ShowData.Devices");
    let out = run(&mut state, "Delete Universe 1 Thru 4 /nc");
    assert!(state.root.show_data.devices.dmx.patch.universes.is_empty(), "{out}");
    assert!(out.contains("Deleted Universes 1, 2, 4") && out.contains("Fixture 1"), "{out}");
    let f = &state.root.show_data.devices.dmx.patch.fixture(1).expect("resolved");
    assert!(f.mvr.breaks.is_empty(), "Fixture 1 stays, without an address");
}

/// Delete Universe asks once and names the fixtures that lose their address there (decisions §11
/// "Issue 11 hardening" 5; the round 3 audit).
#[test]
fn deleting_a_universe_in_use_names_its_fixtures_first() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Claim Root.ShowData.Devices");
    let out = match dispatch(&mut state, "Delete Universe 1") {
        DispatchResult::Ask(a) => a.view().text.clone(),
        other => panic!("must ask, got {:?}", std::mem::discriminant(&other)),
    };
    assert!(out.contains("Fixture 1") && out.contains("2"), "{out}");
    assert!(state.root.show_data.devices.dmx.patch.universes.contains_key(&1), "nothing gone before the answer");
}

/// R1 review 12: a relative value finds the attribute whatever its case and
/// works in the value's own unit (a physical value stays physical).
#[test]
fn playback_relative_values_find_the_attribute_and_keep_the_unit() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1");
    run(&mut state, "At 50");
    let guid = state.root.show_data.devices.dmx.patch.fixture(1).expect("resolved").meta.identity.guid;
    let now = |s: &AppState| s.current_programmer().and_then(|p| p.get_value(guid, "Dimmer")).and_then(lumacore_base::Value::absolute);
    run(&mut state, "Attribute \"dimmer\" At + 10");
    assert_eq!(now(&state), Some(lumacore_base::Absolute::Normalized(60.0)));
    // `At <value> Att "x"`: the alias / prefix works like the word.
    run(&mut state, "At + 5 Att \"dimmer\"");
    assert_eq!(now(&state), Some(lumacore_base::Absolute::Normalized(65.0)));
    let me = state.root.show_data.user_no(state.effective_user_name()).unwrap();
    state.root.show_data.users.get_mut(&me).unwrap().programmer.set_value(guid, "Dimmer", lumacore_base::Value::physical(400.0));
    run(&mut state, "Attribute \"DIMMER\" At + 10");
    assert_eq!(now(&state), Some(lumacore_base::Absolute::Physical(410.0)), "physical stays physical, not 0 + 10");
}

/// R1 review (9a42479): a value for a fixture that isn't in the patch is
/// never kept.
#[test]
fn playback_no_value_for_a_fixture_that_is_gone() {
    let mut state = rig(&[1]);
    let gone = uuid::Uuid::new_v4();
    let set = crate::programmers::program(&mut state, vec![(gone, "Dimmer".into(), lumacore_base::Value::normalized(50.0))]);
    assert_eq!(set, Ok(0));
    assert!(state.current_programmer().and_then(|p| p.get_value(gone, "Dimmer")).is_none());
}

/// Masters live in `Playbacks.Live.Masters` and are never undone (decisions §29, §30 "12f"):
/// no move — grand, its caps, a group master, a user's mode master — files an Oops step.
#[test]
fn masters_are_never_undone_and_take_relative_input() {
    let mut state = rig(&[1, 2]);
    let steps = |s: &AppState| s.undo_stack("Admin").map_or(0, |u| u.frames.len());
    run(&mut state, "Fixture 1 Thru 2");
    run(&mut state, "Store Group 3");
    let before = steps(&state);
    assert!(run(&mut state, "Master 2.1 At 50").contains("Master 2.1 Grand at 50%"));
    assert!(run(&mut state, "Master 2.1 At + 10").contains("60%"));
    assert!(run(&mut state, "Master 2.1 At +5").contains("65%"));
    assert!(run(&mut state, "Master 2.1 At * 2").contains("100%"), "clamped to 100");
    assert!(run(&mut state, "Master \"Grand\" At - 30").contains("70%"));
    run(&mut state, "Off Master 2.1");
    assert!(!state.root.show_data.playbacks.grand().1);
    run(&mut state, "On Master 2.1");
    run(&mut state, "Master 2.4 At 40");
    run(&mut state, "Master \"Highlight\" At 30");
    assert!(run(&mut state, "Master Group 3 At 50").contains("50%"));
    assert!(run(&mut state, "Master Group 3 At + 10").contains("60%"));
    assert_eq!(steps(&state), before, "a master move filed an Oops step");
    let g = state.root.show_data.playbacks.live.masters.grand;
    assert_eq!((g.master, g.on, g.solo), (0.7, true, 0.4));
    // An unreadable level is refused, never 0 (E-CMD-008).
    let out = run(&mut state, "Master 2.1 At + x");
    assert!(out.contains("E-CMD-008") || out.contains("E-SYN-"), "{out}");
    assert_eq!(state.root.show_data.playbacks.live.masters.grand.master, 0.7);
}

/// The other master groups are published as planned; `Master 1` (the old grand master)
/// points at 2.1 (decisions §30 "12f").
#[test]
fn planned_masters_say_so() {
    let mut state = rig(&[1]);
    let one = run(&mut state, "Master 1 At 50");
    assert!(one.contains("E-CMD-003") && one.contains("Master 2.1"), "{one}");
    for n in ["2.3", "2.5", "3.1", "4.2", "5"] {
        let r = run(&mut state, &format!("Master {n} At 50"));
        assert!(r.contains("E-CMD-003"), "{n}: {r}");
    }
    assert!(run(&mut state, "Master 9 At 50").contains("E-CMD-007"));
    assert!(run(&mut state, "List Master").contains("2.1 Grand"));
}

/// Master 2.2 / 2.4 cap everybody's Highlight / Solo: the engine gets each user's own strength
/// times the grand cap.
#[test]
fn the_grand_caps_scale_everybodys_modes() {
    let mut state = rig(&[1]);
    run(&mut state, "On Solo");
    run(&mut state, "On Highlight");
    run(&mut state, "Master 2.4 At 50");
    run(&mut state, "Master 2.2 At 25");
    let input = state.engine_input(std::sync::Arc::new(state.root.show_data.clone()));
    let me = input.users.first().expect("a logged-in user");
    assert!((me.solo - 0.5).abs() < 1e-9, "{}", me.solo);
    assert!((me.highlight - 0.25).abs() < 1e-9, "{}", me.highlight);
}

/// The gesture channel (decisions §30 "12f"): a move is quiet — no journal entry, no Usage, no
/// System Monitor line; the end files one entry naming what moved; the same line typed stays a
/// normal command.
#[test]
fn a_gesture_files_one_entry_at_its_end() {
    use crate::gesture::GestureLine;
    let mut state = rig(&[1]);
    let journal = |s: &AppState| s.root.runtime.journal.len();
    let usage = |s: &AppState| s.root.show_data.users.values().map(|u| u.usage.entries.len()).sum::<usize>();
    let (j0, u0) = (journal(&state), usage(&state));
    state.sysmon_pending.clear();
    for v in ["90", "80", "63.5"] {
        state.gesture = Some(GestureLine::Move { id: "g".into() });
        run(&mut state, &format!("Master 2.1 At {v}"));
        state.gesture = None;
    }
    assert_eq!((journal(&state), usage(&state)), (j0, u0), "a move was recorded");
    assert!(state.sysmon_pending.is_empty(), "{:?}", state.sysmon_pending);
    assert_eq!(state.root.show_data.playbacks.live.masters.grand.master, 0.635);
    state.gesture = Some(GestureLine::End { id: "g".into(), summary: "Fader Master 2.1: 100 → 63.5".into() });
    run(&mut state, "Master 2.1 At 63.5");
    state.gesture = None;
    assert_eq!(journal(&state), j0 + 1);
    let last = state.root.runtime.journal.entries().last().unwrap();
    assert_eq!(last.command, "Fader Master 2.1: 100 → 63.5");
    assert!(state.sysmon_pending.iter().any(|(cat, text)| cat == "Faders" && text.starts_with("Fader Master 2.1: 100 → 63.5 · ")));
    // Typed, the same line is an ordinary command: journaled as typed.
    run(&mut state, "Master 2.1 At 50");
    assert_eq!(state.root.runtime.journal.entries().last().unwrap().command, "Master 2.1 At 50");
}

/// Every sequence has its fixed CueZero and OffCue (decisions §32): CueZero takes stored values
/// (`Store Cue 0`) and tracks into every cue; the OffCue holds only timing; neither is a Goto
/// target; a file without them is refused.
#[test]
fn cue_zero_and_off_cue_are_fixed_parts_of_a_sequence() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Fixture 1 At 100");
    assert!(run(&mut state, "Store Sequence 1 Cue 1").contains("Stored Cue 1"));
    run(&mut state, "Off Fixture 1");
    run(&mut state, "Fixture 2 At 40");
    assert!(run(&mut state, "Store Sequence 1 Cue 0").contains("CueZero"));
    let seq = &state.root.show_data.datapools[&1].sequences[&1];
    assert_eq!(seq.cue_zero.values().count(), 1);
    // CueZero tracks into cue 1.
    assert!(seq.tracked(1000).keys().any(|(_, a)| *a == "Dimmer"));
    assert_eq!(seq.tracked(1000).len(), 2);
    let off = run(&mut state, "Store Sequence 1 Cue \"OffCue\"");
    assert!(off.contains("E-CMD-010"), "{off}");
    assert!(run(&mut state, "Set Sequence 1 Cue \"OffCue\" \"OutFade\" 2").contains("OutFade 2 s"));
    assert!(run(&mut state, "List Cue Sequence 1").contains("OutFade 2 s"));
    assert!(run(&mut state, "Goto Sequence 1 Cue 0").contains("E-CMD-010"));
    // A sequence without CueZero / OffCue doesn't load.
    let mut json = serde_json::to_value(&state.root.show_data).unwrap();
    let seqs = json["datapools"]["1"]["sequences"].as_object_mut().unwrap();
    seqs.values_mut().next().unwrap().as_object_mut().unwrap().remove("cue_zero");
    assert!(serde_json::from_value::<lumacore_tree::ShowData>(json).is_err());
}

/// CueZeroMode fills CueZero's defaults part: the used dimmers, every used attribute, or
/// nothing; what was stored into CueZero stays.
#[test]
fn cue_zero_mode_fills_the_defaults() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Fixture 1 Thru 2 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    let zero = |s: &AppState| s.root.show_data.datapools[&1].sequences[&1].cue_zero.parts.get(&1).map_or(0, |p| p.values.len());
    assert_eq!(zero(&state), 0, "Off by default");
    assert!(run(&mut state, "Set Sequence 1 \"CueZeroMode\" OnlyUsedDimmers").contains("CueZeroMode"));
    assert_eq!(zero(&state), 2);
    run(&mut state, "Set Sequence 1 \"CueZeroMode\" Off");
    assert_eq!(zero(&state), 0);
}

/// Echoes (decisions §32, §30 "12f echoes"): Top On runs the first cue beside the live run,
/// intensity HTP; Top Off plays the OffCue and the echo goes; MaxEchoes refuses the newest;
/// Off Sequence ends them; EchoMax and a lost holder end one by themselves; never saved, never
/// undone.
#[test]
fn echoes_run_beside_the_live_playback() {
    let mut state = rig(&[1]);
    let steps = |s: &AppState| s.undo_stack("Admin").map_or(0, |u| u.frames.len());
    run(&mut state, "Fixture 1 At 100");
    run(&mut state, "Store Sequence 1 Cue 1");
    run(&mut state, "Fixture 1 At 30");
    run(&mut state, "Store Sequence 1 Cue 2");
    run(&mut state, "Off Fixture 1");
    run(&mut state, "Goto Sequence 1 Cue 2");
    assert_eq!(dmx(&state, 1), 77, "the live run at cue 2 (30 %)");
    let before = steps(&state);
    assert!(run(&mut state, "Top On Sequence 1").contains("echo 1 → Cue 1"));
    assert_eq!(dmx(&state, 1), 255, "HTP: the echo's 100 % over the live 30 %");
    assert_eq!(steps(&state), before, "an echo is never undone");
    assert!(run(&mut state, "Top Off Sequence 1").contains("echo 1 → OffCue"));
    assert_eq!(dmx(&state, 1), 77, "the OffCue (no timing) took the echo to 0");
    crate::commands::playback::echo_tick(&mut state, &Default::default());
    assert!(state.root.show_data.playbacks.echoes.sequences.is_empty(), "gone once faded");
    // MaxEchoes: a full list refuses the newest Top On.
    run(&mut state, "Set Sequence 1 \"MaxEchoes\" 2");
    run(&mut state, "Top On Sequence 1");
    run(&mut state, "Top On Sequence 1");
    assert!(run(&mut state, "Top On Sequence 1").contains("E-CMD-010"));
    let guid = state.root.show_data.datapools[&1].sequences[&1].meta.identity.guid;
    assert_eq!(state.root.show_data.playbacks.echoes.sequences[&guid].len(), 2);
    // Never saved: the file has none.
    let (bytes, _) = lumacore_luma::show_file::encode(&state.root.show_data).unwrap();
    let back = lumacore_luma::show_file::from_bytes(&bytes).unwrap();
    assert!(back.playbacks.echoes.sequences.is_empty());
    // Off Sequence ends them with the live run.
    assert!(run(&mut state, "Off Sequence 1").contains("2 echoes ending"));
    assert!(state.root.show_data.playbacks.echoes.sequences[&guid].iter().all(|e| e.off_ms.is_some()));
    crate::commands::playback::echo_tick(&mut state, &Default::default());
    // EchoMax and a holder that went away.
    run(&mut state, "Top On Sequence 1");
    run(&mut state, "Top On Sequence 1");
    {
        let list = state.root.show_data.playbacks.echoes.sequences.get_mut(&guid).unwrap();
        list[0].started_ms = list[0].started_ms.saturating_sub(31_000);
        list[1].holder = Some("a client that left".into());
    }
    crate::commands::playback::echo_tick(&mut state, &Default::default());
    assert!(state.root.show_data.playbacks.echoes.sequences[&guid].iter().all(|e| e.off_ms.is_some()));
    // A bare Top is a usage error.
    assert!(run(&mut state, "Top").contains("E-CMD-006"));
}

/// Value timing (decisions §30 "12f"): a programmer value's own Fade travels into the cue and
/// times that channel on a Go; `Fade Cue` gives it back to the cue; no value → nothing to time.
#[test]
fn value_timing_travels_into_the_cue() {
    let mut state = rig(&[1, 2]);
    run(&mut state, "Fixture 1");
    assert!(run(&mut state, "Attribute \"Dimmer\" Fade 2").contains("E-CMD-009"), "no value yet");
    run(&mut state, "At 100");
    assert!(run(&mut state, "Attribute \"Dimmer\" Fade 2").contains("Fade 2 s"));
    run(&mut state, "Store Sequence 1 Cue 1");
    let v = &state.root.show_data.datapools[&1].sequences[&1].cues[&1000].parts[&0].values[0];
    assert_eq!(v.timing.fade, Some(2.0));
    run(&mut state, "Off Fixture 1");
    run(&mut state, "Goto Sequence 1 Cue 1");
    // The winning value carries the value's own timing into the plan (a Go fades by it).
    let plan = state.output_plan();
    let guid = state.root.show_data.devices.dmx.patch.fixtures.values().next().map(|f| f.meta.identity.guid).unwrap();
    let timing = plan.resolved(guid, "Dimmer").and_then(|r| r.timing).expect("the sequence's value");
    assert_eq!((timing.fade_in, timing.fade_out), (2000, 2000));
    run(&mut state, "Fixture 1");
    run(&mut state, "At 50");
    assert!(run(&mut state, "Attribute \"Dimmer\" Fade Cue").contains("from the cue"));
    assert!(state.current_programmer().unwrap().values.values().all(|pv| pv.timing.fade.is_none()));
}

/// An encoder drag on programmer values is one Oops step (decisions §30 "12f"): the moves file
/// none, the end files one from where the drag began.
#[test]
fn a_programmer_drag_is_one_oops_step() {
    use crate::gesture::GestureLine;
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1");
    run(&mut state, "At 0");
    let steps = |s: &AppState| s.undo_stack("Admin").map_or(0, |u| u.frames.len());
    let before = steps(&state);
    for v in [10, 20, 30] {
        state.gesture = Some(GestureLine::Move { id: "enc".into() });
        run(&mut state, &format!("Attribute \"Dimmer\" At {v}"));
        state.gesture = None;
    }
    state.gesture = Some(GestureLine::End { id: "enc".into(), summary: "Fader Dimmer: 0 → 40".into() });
    run(&mut state, "Attribute \"Dimmer\" At 40");
    state.gesture = None;
    assert_eq!(steps(&state), before + 1, "one step for the drag");
    assert_eq!(dmx(&state, 1), 102);
    run(&mut state, "Oops");
    assert_eq!(dmx(&state, 1), 0, "Oops goes back to where the drag began");
}

/// `Redo <n>` redoes the newest n undone steps; the Oops key and `Menu "Oops"` exist (decisions
/// §30 "12f": the Oops window).
#[test]
fn redo_takes_a_count_and_the_oops_window_is_a_menu() {
    let mut state = rig(&[1]);
    run(&mut state, "Fixture 1");
    for v in [10, 20, 30, 40] {
        run(&mut state, &format!("At {v}"));
    }
    run(&mut state, "Oops 3");
    assert_eq!(dmx(&state, 1), 26, "back at 10 %");
    assert!(run(&mut state, "Redo 2").contains("Redone 2 steps"));
    assert_eq!(dmx(&state, 1), 77, "30 %");
    // Typed (decisions §33 part 2): the parser and the signature refuse, the handler checks the count.
    assert!(run(&mut state, "Redo zork").contains("E-SYN-001"));
    assert!(run(&mut state, "Redo 1.5").contains("E-SYN-014"));
    assert!(run(&mut state, "Redo 0").contains("E-CMD-008"));
    assert!(run(&mut state, "Redo 5").contains("Redone 1 step"), "only one was left");
    assert!(lumacore_tree::ui_keys::builtin_ui_keys().values().any(|k| k.id == "oops" && k.hold == "Menu \"Oops\""));
    assert!(run(&mut state, "List Menu").contains("Oops"));
}

/// Sequences are DataPool objects (decisions §32, §30 "12f"): `Sequence 1` is your selected
/// DataPool's; your selected sequence keeps working from another pool (it is a GUID).
#[test]
fn sequences_belong_to_their_datapool() {
    let mut state = rig(&[1]);
    state.root.show_data.datapools.insert(2, lumacore_tree::Datapool::new(Some("Festival".into())));
    run(&mut state, "Select DataPool 2");
    run(&mut state, "Fixture 1");
    run(&mut state, "At 100");
    assert!(run(&mut state, "Store Sequence 1 Cue 1").contains("Stored Cue 1"));
    assert!(state.root.show_data.datapools[&2].sequences.contains_key(&1));
    assert!(state.root.show_data.datapools[&1].sequences.is_empty());
    run(&mut state, "Select Sequence 1");
    run(&mut state, "Select DataPool 1");
    // Typed, `Sequence 1` is this pool's — there is none.
    assert!(run(&mut state, "Go Sequence 1").contains("E-PLAY-001"));
    // Bare, the selected sequence (in DataPool 2) runs.
    let go = run(&mut state, "Go");
    assert!(!go.contains("Error"), "{go}");
    let guid = state.root.show_data.datapools[&2].sequences[&1].meta.identity.guid;
    assert!(state.root.show_data.playbacks.live.sequences.contains_key(&guid));
}
