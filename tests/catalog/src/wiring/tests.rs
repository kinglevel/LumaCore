// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The rig wiring with lines, users, patch and the show file around it (decisions §34 25.16):
//! lines that send only once mapped, gateways, the station asks, Import Show of networks.
//!
//! Owns:          the cross-crate wiring tests.
//! Doesn't own:   each crate's own words' tests.
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §30 "12d part 4", §34 25.16.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::commands::dispatch;
use lumacore_objects::{AppState, DispatchResult};
use lumacore_wire::{FieldInput, UiEvent};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn last_menu(state: &AppState) -> (String, Option<String>, bool, String) {
    match state.ui_requests.last() {
        Some(UiEvent::OpenMenu {
            menu,
            tab,
            toggle,
            grids,
            ..
        }) => (menu.clone(), tab.clone(), *toggle, grids.clone()),
        other => panic!("no OpenMenu: {other:?}"),
    }
}

fn steps(state: &AppState) -> usize {
    state.undo_stack("Admin").map_or(0, |u| u.frames.len())
}

fn ui() -> AppState {
    let mut state = AppState::new();
    state.ui_client = true;
    state
}

fn demo() -> AppState {
    lumacore_objects::demo::fresh_state()
}

/// The line the ask's `button` runs (the ask itself changes nothing).
fn ask(state: &mut AppState, line: &str, button: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Ask(a) => a
            .line_of(button)
            .unwrap_or_else(|| panic!("'{line}': no button {button}")),
        other => panic!("'{line}' didn't ask: {}", kind(&other)),
    }
}

fn kind(r: &DispatchResult) -> &'static str {
    match r {
        DispatchResult::Ask(_) => "an ask",
        DispatchResult::Output(_) => "output",
        _ => "something else",
    }
}

fn main_guid(s: &AppState) -> uuid::Uuid {
    s.root
        .show_data
        .protocols
        .networks
        .values()
        .find(|n| n.meta.identity.name.as_deref() == Some("Main"))
        .unwrap()
        .meta
        .guid()
}

fn station() -> AppState {
    let mut s = AppState::new();
    s.local_settings.station_id = "0b7c8a2e-5d4f-4c3b-9a1e-2f3d4c5b6a7e".into();
    s.root.station_settings.station_name = "FOH".into();
    run(&mut s, "Store Network \"Main\"");
    s
}

/// A show with a bit of everything, made with commands.
fn full_show() -> AppState {
    let mut state = AppState::new();
    let patch = &mut state.root.show_data.devices.dmx.patch;
    let universe = lumacore_fixtures::Universe::new(Some("Main".into()));
    let ug = universe.meta.identity.guid;
    patch.universes.insert(1, universe);
    let ft = lumacore_fixtures::FixtureType {
        meta: lumacore_tree::NodeMeta::new(Some("Dim".into())),
        manufacturer: "Test".into(),
        gdtf_fixture_type_id: None,
        category: Default::default(),
        revisions: vec![lumacore_fixtures::Revision {
            meta: lumacore_tree::NodeMeta::new(None),
            geometries: Vec::new(),
            wheels: Vec::new(),
            emitters: Vec::new(),
            filters: Vec::new(),
            attributes: Vec::new(),
            gdtf_version: "1.2".into(),
            file_hash: String::new(),
            modes: vec![lumacore_fixtures::Mode {
                meta: lumacore_tree::NodeMeta::new(None),
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
    let (ftg, (rev, mode)) = (ft.meta.identity.guid, ft.mode_guids(0, 0).unwrap());
    patch.fixture_types.insert(1, std::sync::Arc::new(ft));
    for id in 1..=4u32 {
        patch.insert_fid(id, lumacore_fixtures::Fixture::new(None, ftg, rev, mode, ug, id as u16, 1),
        );
    }
    state.root.show_data.after_load();
    for line in [
        "Fixture 1 Thru 2",
        "Store Group 1 \"Front\"",
        "At 50",
        "Store Sequence 1 Cue 1",
        "Select Sequence 1",
        "At 80",
        "Store Cue 2",
        "Clear",
        "Clear",
        "Clear",
        "Store Network \"Rig\"",
        "Store Sacn 1 /Network \"Rig\"",
        "Assign Universe 1 At Sacn 1 /Universe 1",
        "Store Osc 2 /Network \"Rig\" /Port 9000 /User \"Admin\"",
        "Store User \"Busk\"",
        "Park DMX 1.10 At 5",
        "Go",
    ] {
        run(&mut state, line);
    }
    state
}

#[test]
fn edit_setting_fields_say_home_unit_and_level() {
    let mut state = AppState::new();
    run(&mut state, "EditSetting Station");
    let Some(UiEvent::EditSettings { fields, .. }) = state.ui_requests.last() else {
        panic!()
    };
    let prio = fields.iter().find(|f| f.name == "Priority").unwrap();
    assert_eq!(
        (prio.home.as_str(), prio.rights.as_str()),
        ("station", "Setup")
    );
    assert_eq!(prio.line, "Set Station \"Priority\" {value}");
    assert!(matches!(&prio.input, FieldInput::Choice { options } if options.len() == 5));
    run(&mut state, "Store Sacn 1");
    run(&mut state, "EditSetting Sacn 1");
    let Some(UiEvent::EditSettings { fields, .. }) = state.ui_requests.last() else {
        panic!()
    };
    let rate = fields.iter().find(|f| f.name == "Rate").unwrap();
    assert_eq!(
        (rate.home.as_str(), rate.rights.as_str()),
        ("show", "Setup")
    );
}

#[test]
fn grid_columns_live_on_the_profile_and_are_never_an_oops_step() {
    let mut state = ui();
    let before = steps(&state);
    let out = run(
        &mut state,
        "Set GridColumns \"dmx.lines\" \"Columns\" \"no,name,rate\"",
    );
    assert!(out.contains("columns no,name,rate"), "{out}");
    run(
        &mut state,
        "Set GridColumns \"dmx.lines\" \"Widths\" \"name=180,rate=90\"",
    );
    run(
        &mut state,
        "Set GridColumns \"dmx.lines\" \"Sort\" \"rate Desc\"",
    );
    run(
        &mut state,
        "Set GridColumns \"dmx.lines\" \"ColumnSet\" Condensed",
    );
    run(&mut state, "Set GridColumns \"dmx.lines\" \"NewLine\" Off");
    // The patch's Offset cell: spare slots after each fixture (§34 round 3).
    let out = run(&mut state, "Set GridColumns \"patch.fixtures\" \"Offset\" 4");
    assert!(out.contains("offset 4"), "{out}");
    assert!(run(&mut state, "Set GridColumns \"patch.fixtures\" \"Offset\" 600").contains("E-"));
    let g = state
        .root
        .show_data
        .profile_for_user("Admin")
        .unwrap()
        .settings
        .ui
        .grids["dmx.lines"]
        .clone();
    assert_eq!(g.columns, vec!["no", "name", "rate"]);
    assert_eq!(g.widths["name"], 180);
    assert_eq!(
        g.sort.as_ref().map(|s| (s.col.as_str(), s.dir.as_str())),
        Some(("rate", "desc"))
    );
    assert_eq!(
        (g.set.as_deref(), g.new_line),
        (Some("Condensed"), Some(false))
    );
    let patch = state.root.show_data.profile_for_user("Admin").unwrap().settings.ui.grids["patch.fixtures"].clone();
    assert_eq!(patch.offset, Some(4));
    assert_eq!(steps(&state), before);
    // The menu carries them.
    run(&mut state, "Menu DMXProtocols");
    assert!(last_menu(&state).3.contains("\"dmx.lines\""));
    // Wrong words are refused with codes; Reset puts the factory columns back.
    assert!(
        run(
            &mut state,
            "Set GridColumns \"dmx.lines\" \"Columns\" \"nope\""
        )
        .contains("E-SET-003")
    );
    assert!(run(&mut state, "Set GridColumns \"nope\" \"Columns\" \"a\"").contains("E-CMD-007"));
    assert!(
        run(
            &mut state,
            "Set GridColumns \"dmx.lines\" \"Colour\" \"red\""
        )
        .contains("E-SET-001")
    );
    run(&mut state, "Reset GridColumns \"dmx.lines\"");
    run(&mut state, "Reset GridColumns \"patch.fixtures\"");
    assert!(
        state
            .root
            .show_data
            .profile_for_user("Admin")
            .unwrap()
            .settings
            .ui
            .grids
            .is_empty()
    );
    assert!(run(&mut state, "List GridColumns").contains("network.interfaces"));
    assert!(run(&mut state, "Info GridColumns \"dmx.lines\"").contains("Set Rate"));
    // Anyone may arrange their own grids.
    run(&mut state, "Store User \"G\" /Rights Guest");
    state.user = "G".into();
    assert!(
        !run(
            &mut state,
            "Set GridColumns \"dmx.lines\" \"ColumnSet\" Full"
        )
        .contains("Error")
    );
}

#[test]
fn station_settings_are_typed_and_need_setup() {
    let mut state = AppState::new();
    let out = run(&mut state, "Set Station \"Name\" \"FOH\"");
    assert!(
        out.contains("Name ") && out.contains("→ \"FOH\""),
        "old → new: {out}"
    );
    assert_eq!(state.root.station_settings.station_name, "FOH");
    assert!(run(&mut state, "Set Station \"Priority\" High").contains("High"));
    assert!(run(&mut state, "Set Station \"Priority\" \"Loud\"").contains("E-SET-003"));
    assert!(run(&mut state, "Set Station \"MulticastBase\" alternative").contains("Alternative"));
    assert!(state.root.station_settings.multicast_alternative);
    assert!(matches!(
        dispatch(&mut state, "Set Station \"HoldOutput\" Toggle"),
        DispatchResult::Ask(_)
    ));
    assert!(
        run(
            &mut state,
            "Set Station \"HoldOutput\" Toggle /NoConfirmation"
        )
        .contains("held")
    );
    assert!(
        run(&mut state, "Set Station \"HoldOutput\" Toggle").contains("resumed"),
        "resuming never asks"
    );
    assert!(run(&mut state, "Set Station Nope 1").contains("E-SET-001"));
    // The row may be named: this station, or another one (sent to it when
    // it is reachable; a name nobody knows is no row).
    assert!(run(&mut state, "Set Station \"FOH\" \"Location\" \"Front\"").contains("Front"));
    assert!(run(&mut state, "Set Station \"Tower\" \"Location\" \"x\"").contains("E-CMD-007"));
    run(&mut state, "Store Station \"Tower\"");
    let out = run(&mut state, "Set Station \"Tower\" \"Location\" \"x\"");
    assert!(
        out.contains("E-CMD-010") && out.contains("isn't reachable"),
        "{out}"
    );
    // Levels: Program is refused, Setup may.
    for (user, level) in [("P", "Program"), ("S", "Setup")] {
        run(
            &mut state,
            &format!("Store User \"{user}\" /Rights {level}"),
        );
    }
    state.user = "P".into();
    let out = run(&mut state, "Set Station \"Location\" \"x\"");
    assert!(
        out.contains("E-RIGHTS-001") && out.contains("needs Setup"),
        "{out}"
    );
    state.user = "S".into();
    assert!(run(&mut state, "Set Station \"Location\" \"Stage\"").contains("Stage"));
}

/// Decision 16 / 17: a bare owner keyword is your own; a quoted id never collides with the
/// quoted property after it.
#[test]
fn a_bare_owner_is_your_own() {
    let mut state = demo();
    let mine = state.root.show_data.profile_id_for_user("Admin").unwrap();
    let out = run(&mut state, "Set UserProfile Theme \"Base\" Night");
    assert!(!out.contains("Error"), "{out}");
    assert_eq!(
        state.root.show_data.user_profiles[&mine]
            .settings
            .theme
            .preset,
        "Night"
    );
    let out = run(&mut state, "Set Station \"Location\" \"Stage left\"");
    assert!(out.contains("Stage left"), "this station: {out}");
    assert_eq!(state.root.station_settings.location, "Stage left");
}

/// Merge of bugfixes (decisions §30): every Delete asks once, in dispatch —
/// a grid delete names the count and the rows, refuses what can't go before
/// asking, and its answer runs without a second pop-up.
#[test]
fn a_grid_delete_asks_once_naming_its_rows() {
    let mut s = station();
    for n in [1, 4, 7] {
        run(&mut s, &format!("Store Sacn {n} /Network \"Main\""));
    }
    // The one range rule: an open range names the rows that are there.
    let DispatchResult::Ask(open) = dispatch(&mut s, "Delete Sacn 4 Thru") else {
        panic!("an open range asks");
    };
    assert_eq!(open.view().text, "Delete 2 Sacns (4 + 7)?");
    let DispatchResult::Ask(a) = dispatch(&mut s, "Delete Sacn 1 Thru 7") else {
        panic!("deleting lines asks");
    };
    assert_eq!(a.view().text, "Delete 3 Sacns (1 + 4 + 7)?");
    let line = a.line_of("Delete").unwrap();
    assert_eq!(line, "Delete Sacn 1 + 4 + 7 /NoConfirmation");
    // The answer runs: no second question.
    match dispatch(&mut s, &line) {
        DispatchResult::Output(out) => assert!(out.contains("Sacn 7 deleted"), "{out}"),
        other => panic!("the answer asked again: {}", kind(&other)),
    }
    assert_eq!(s.root.show_data.protocols.all_lines().count(), 0);
    // One line asks once too, and its answer doesn't ask again.
    run(&mut s, "Store Sacn 2 /Network \"Main\"");
    let line = ask(&mut s, "Delete Sacn 2", "Delete");
    assert_eq!(line, "Delete Sacn 2 /NoConfirmation");
    assert!(
        matches!(dispatch(&mut s, &line), DispatchResult::Output(_)),
        "one pop-up only"
    );
    // A station row named by a quoted name: one question, then deleted.
    run(&mut s, "Store Station \"Tower\"");
    let line = ask(&mut s, "Delete Station \"Tower\"", "Delete");
    assert_eq!(line, "Delete Station \"Tower\" /NoConfirmation");
    assert!(
        matches!(dispatch(&mut s, &line), DispatchResult::Output(_)),
        "one pop-up only"
    );
    assert!(
        !s.root
            .show_data
            .stations
            .values()
            .any(|r| r.meta.name() == Some("Tower"))
    );
    // A network in use: refused before any question.
    run(&mut s, "Store ArtNet 1 /Network \"Main\" /Mode Input");
    assert!(matches!(
        dispatch(&mut s, "Delete Network \"Main\""),
        DispatchResult::Output(_)
    ));
}

#[test]
fn dangerous_station_lines_are_station_asks() {
    let mut s = station();
    // Online Off and HoldOutput On ask; Online On / HoldOutput Off don't.
    let line = ask(&mut s, "Set Station \"Online\" Off", "Go offline");
    assert_eq!(line, "Set Station \"Online\" Off /NoConfirmation");
    assert!(s.root.station_settings.online);
    let line = ask(&mut s, "Set Station \"HoldOutput\" Toggle", "Hold output");
    run(&mut s, &line);
    assert!(s.root.station_settings.output_held);
    assert!(run(&mut s, "Set Station \"HoldOutput\" Off").contains("resumed"));
    // Lines and networks ask, even one; several ask once.
    run(&mut s, "Store Sacn 1 /Network \"Main\"");
    run(&mut s, "Store Sacn 2 /Network \"Main\"");
    run(&mut s, "Store ArtNet 1 /Network \"Main\" /Mode Input");
    let line = ask(&mut s, "Delete Sacn 1 Thru 2", "Delete");
    // One statement (one Oops step) naming the rows that can go.
    assert_eq!(line, "Delete Sacn 1 Thru 2 /NoConfirmation");
    assert_eq!(s.root.show_data.protocols.lines("Sacn").map_or(0, |m| m.len()), 2);
    let out = run(&mut s, &line);
    assert!(
        out.contains("Sacn 1 deleted") && out.contains("Sacn 2 deleted"),
        "{out}"
    );
    assert!(
        run(&mut s, "Delete Network \"Main\"").contains("used by lines"),
        "refused before any ask"
    );
    assert!(
        run(&mut s, "Delete Sacn 7").contains("E-CMD-007"),
        "the one resolver answers"
    );
    // Several users: one ask (one user keeps its own).
    run(&mut s, "Store User \"A\" /Rights Program");
    run(&mut s, "Store User \"B C\" /Rights Program");
    let line = ask(&mut s, "Delete User \"A\" + \"B C\"", "Delete");
    // The names keep their quotes (issue 07).
    assert_eq!(line, "Delete User \"A\" + \"B C\" /NoConfirmation");
    run(&mut s, &line);
    assert!(s.root.show_data.user("B C").is_none() && s.root.show_data.user("A").is_none());
}

#[test]
fn rows_made_in_advance_are_wired_for_several_stations_in_one_line() {
    let mut s = station();
    assert!(
        run(
            &mut s,
            "Set Station \"Tower\" Network \"Main\" \"Interface\" \"en1\""
        )
        .contains("E-CMD-007"),
        "no row yet"
    );
    assert!(run(&mut s, "Store Station \"Tower\"").contains("row 1"));
    assert!(run(&mut s, "Store Station \"Delay\"").contains("row 2"));
    assert!(
        run(&mut s, "Store Station \"tower\"").contains("E-CMD-010"),
        "one row per station"
    );
    let out = run(
        &mut s,
        "Set Station \"Tower\" + \"Delay\" Network \"Main\" \"Interface\" \"en2\"",
    );
    assert!(
        out.contains("Station \"Tower\"") && out.contains("Station \"Delay\""),
        "{out}"
    );
    let main = main_guid(&s);
    for n in [1, 2] {
        assert_eq!(
            s.root.show_data.stations[&n]
                .networks
                .get(&main)
                .map(String::as_str),
            Some("en2")
        );
    }
    // Another station's rows don't touch this station's wiring.
    assert!(!s.root.station_settings.networks.contains_key("main"));
    // Services of interfaces this station doesn't have (pre-configured).
    run(
        &mut s,
        "Set Station 1 Thru 2 Interface \"eth7\" + \"eth8\" \"HTTP\" No",
    );
    assert!(!s.root.show_data.stations[&2].services_of("eth8").http);
    // A wiring line naming no network of the show says so.
    assert!(
        run(
            &mut s,
            "Set Station \"Tower\" Network \"Nope\" \"Interface\" \"en1\""
        )
        .contains("E-CMD-007")
    );
    // Rights: Program may not.
    run(&mut s, "Store User \"P\" /Rights Program");
    s.user = "P".into();
    assert!(
        run(
            &mut s,
            "Set Station \"Tower\" Network \"Main\" \"Interface\" \"en3\""
        )
        .contains("E-RIGHTS-001")
    );
    s.user = "Admin".into();
    // List / Info show the table.
    assert!(run(&mut s, "List Station").contains("Station table"));
    let info = run(&mut s, "Info Station \"Tower\"");
    assert!(
        info.contains("Network Main") && info.contains("en2"),
        "{info}"
    );
    // Deleting a station row asks.
    let line = ask(&mut s, "Delete Station \"Delay\"", "Delete");
    assert!(
        s.root.show_data.stations.contains_key(&2),
        "the ask changes nothing"
    );
    assert!(run(&mut s, &line).contains("station table"));
    assert!(!s.root.show_data.stations.contains_key(&2));
    assert!(
        run(&mut s, "Delete Station \"Ghost\"").contains("E-CMD-007"),
        "refused, not asked"
    );
}

/// `EditSetting` (decisions §29): the settings with their Set lines, and the
/// pop-up event for UIs.
#[test]
fn edit_setting_lists_settings_and_asks_the_ui() {
    let mut state = AppState::new();
    let out = run(&mut state, "EditSetting Station");
    assert!(
        out.contains("Location") && out.contains("Set Station \"Location\""),
        "{out}"
    );
    assert!(matches!(
        state.ui_requests.last(),
        Some(lumacore_wire::UiEvent::EditSettings { fields, .. }) if !fields.is_empty()
    ));
    assert!(run(&mut state, "EditSetting Fixture 999").contains("E-CMD-007"));
    assert!(
        run(&mut state, "EditSetting Executor 1").contains("E-CMD-015"),
        "an object without properties"
    );
    assert!(run(&mut state, "EditSetting").contains("E-CMD-006"));
}

#[test]
fn identity_links_follow_renames_and_moves() {
    let mut state = full_show();
    // The line keeps its network through a rename (the link is the GUID).
    run(&mut state, "Set Network 1 \"Name\" \"Stage\"");
    let show = &state.root.show_data;
    let line = show.protocols.line("Sacn", 1).unwrap();
    assert_eq!(show.protocols.network_name(line.network), "Stage");
    assert!(
        run(&mut state, "Delete Network 1").contains("used by lines"),
        "refused before any ask"
    );
    // A moved group is the same object.
    let g = state.root.show_data.datapools[&1].groups[&1].meta.identity.guid;
    run(&mut state, "Move Group 1 At 9");
    assert_eq!(state.root.show_data.datapools[&1].groups[&9].meta.identity.guid, g);
    assert!(state.root.show_data.identity().is_clean());
}
