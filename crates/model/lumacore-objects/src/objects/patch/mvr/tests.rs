// ███╗   ███╗██╗   ██╗██████╗     ████████╗███████╗███████╗████████╗███████╗
// ████╗ ████║██║   ██║██╔══██╗    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██╔████╔██║██║   ██║██████╔╝       ██║   █████╗  ███████╗   ██║   ███████╗
// ██║╚██╔╝██║╚██╗ ██╔╝██╔══██╗       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║ ╚═╝ ██║ ╚████╔╝ ██║  ██║       ██║   ███████╗███████║   ██║   ███████║
// ╚═╝     ╚═╝  ╚═══╝  ╚═╝  ╚═╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: mvr tests

//! Tests of the Mvr object: the review's rule on a hand-built file against the demo show
//! (matches by UUID / FID / name, new, lost, clash, the scope), and the words — a bare Import
//! asks for the file, the claim is needed, `tests/fixtures/rig.mvr` on a folder drive opens a
//! review, List / Info show it, Cancel drops it.
//!
//! Owns:          the unit tests of mvr.rs, review.rs, import.rs, list.rs.
//! Doesn't own:   the file format's tests (lumacore-mvr), the page (e2e).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 "Import MVR hardened" 1–3.
// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS


use std::collections::BTreeMap;
use std::path::Path;

use lumacore_mvr::{MvrAddress, MvrFile, MvrFixture, MvrGroup, MvrLayer, MvrScene};
use lumacore_tree::{DriveKind, MvrMatch, MvrTake};
use uuid::Uuid;

use super::review::{Scope, build};
use super::Settings;
use crate::mvr_review::RowStatus;
use crate::commands::dispatch;
use crate::{AppState, DispatchResult};

fn demo() -> AppState {
    crate::demo::fresh_state()
}

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        DispatchResult::Ask(a) => panic!("'{line}' asked: {} — {}", a.title(), a.view().text),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn settings() -> Settings {
    Settings::from_preset(&lumacore_tree::MvrImportPreset::default())
}

/// A universe the show doesn't have (the rig takes 1–255 with gaps): the file's free addresses.
fn free_universe(state: &AppState) -> u32 {
    state.root.show_data.devices.dmx.patch.universes.keys().max().map_or(1, |m| m + 1)
}

/// A file with one layer, one group, and fixtures: `f1` matching a demo fixture by uuid, `f2` by
/// FID, `f3` new on a free number, `f4` new on a taken number (a clash).
fn file(state: &AppState) -> MvrFile {
    let patch = &state.root.show_data.devices.dmx.patch;
    let numbered = patch.by_number();
    let (no1, demo1) = numbered.iter().next().map(|(n, f)| (*n, *f)).unwrap();
    let no2 = *numbered.keys().nth(1).unwrap();
    let free = numbered.keys().max().unwrap() + 100;
    let layer = Uuid::new_v4();
    let group = Uuid::new_v4();
    let fx = |uuid: Uuid, name: &str, fid: Option<u32>, universe: u32, channel: u16| MvrFixture {
        uuid,
        name: name.into(),
        parent: group,
        layer,
        matrix: Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1000.0, 2000.0, 3000.0]]),
        gdtf_spec: "AuraXB.gdtf".into(),
        gdtf_mode: "Standard".into(),
        addresses: vec![MvrAddress { dmx_break: 0, universe, channel }],
        fixture_id: fid.map(|n| n.to_string()).unwrap_or_default(),
        fixture_id_numeric: fid,
        ..MvrFixture::default()
    };
    let addr1 = demo1.primary_break().map(|b| b.start_address).unwrap_or(1);
    let free_u = free_universe(state);
    let universe1 = demo1.primary_break().and_then(|b| patch.universes.iter().find(|(_, u)| u.meta.identity.guid == b.universe_id)).map(|(n, _)| *n).unwrap_or(1);
    MvrFile {
        scene: MvrScene {
            provider: "test".into(),
            layers: vec![MvrLayer { uuid: layer, name: "Stage".into() }],
            groups: vec![MvrGroup { uuid: group, name: "Truss".into(), parent: layer, layer }],
            fixtures: vec![
                fx(demo1.meta.identity.guid, "By uuid", Some(no1 + 1000), universe1, addr1),
                fx(Uuid::new_v4(), "By fid", Some(no2), free_u, 1),
                fx(Uuid::new_v4(), "New one", Some(free), free_u, 100),
                fx(Uuid::new_v4(), "Taken number", Some(no2 + 1), free_u, 200),
            ],
            ..MvrScene::default()
        },
        entries: vec!["GeneralSceneDescription.xml".into(), "AuraXB.gdtf".into()],
    }
}

#[test]
fn the_review_matches_by_uuid_then_fid_finds_new_and_clashing_rows() {
    let state = demo();
    let f = file(&state);
    let patch = &state.root.show_data.devices.dmx.patch;
    let no1 = patch.fids()[0].0;
    let no2 = patch.fids()[1].0;
    let r = build(&state.root.show_data.devices, &f, &settings(), None, &BTreeMap::new(), "t.mvr", None, None);
    let row = |no: u32| r.rows.iter().find(|x| x.no == no).unwrap_or_else(|| panic!("row {no}: {:?}", r.rows.iter().map(|x| (x.no, x.status)).collect::<Vec<_>>()));
    // By uuid: matched, the address the same, the name offered (differs).
    let r1 = row(no1);
    assert_eq!((r1.status, r1.match_key.as_str()), (RowStatus::Changed, "UUID"), "{:?}", r1.cells);
    let name = r1.cells.iter().find(|c| c.field == "name").unwrap();
    assert!(name.differs && name.mode == MvrTake::Offer && !name.taken);
    let addr = r1.cells.iter().find(|c| c.field == "address").unwrap();
    assert!(!addr.differs, "{addr:?}");
    // By FID: the file's address (a free universe) is free; the row isn't ticked (Offer) but Changed.
    let r2 = row(no2);
    assert_eq!((r2.status, r2.match_key.as_str()), (RowStatus::Changed, "FID"));
    assert!(!r2.taken);
    // New on a free number: taken (New fixtures = Take), group known as the file's.
    let r3 = r.rows.iter().find(|x| x.name == "New one").unwrap();
    assert_eq!(r3.status, RowStatus::New);
    assert!(r3.taken && r3.gdtf_embedded && !r3.type_known && r3.group_name == "Truss" && r3.group_missing);
    // The fourth names a number a demo fixture holds: with Auto it matches that fixture by FID.
    let r4 = r.rows.iter().find(|x| x.name == "Taken number").unwrap();
    assert_eq!((r4.status, r4.match_key.as_str()), (RowStatus::Changed, "FID"));
    // The summary counts them.
    assert!(r.summary().contains("1 new"), "{}", r.summary());
    // UUID only: "By fid" finds nothing and its number is a demo fixture's — a clash, never ticked.
    let only_uuid = Settings { match_key: MvrMatch::Uuid, ..settings() };
    let r = build(&state.root.show_data.devices, &f, &only_uuid, None, &BTreeMap::new(), "t.mvr", None, None);
    let r2 = r.rows.iter().find(|x| x.name == "By fid").unwrap();
    assert_eq!(r2.status, RowStatus::Clash);
    assert!(!r2.taken && r2.clash.as_deref().unwrap().contains("exists"));
    assert!(r.summary().contains("clash"), "{}", r.summary());
    // One key only: by name finds nothing here (names differ), so every file row is new or a clash.
    let only_name = Settings { match_key: MvrMatch::FixtureName, ..settings() };
    let r = build(&state.root.show_data.devices, &f, &only_name, None, &BTreeMap::new(), "t.mvr", None, None);
    assert!(r.rows.iter().all(|x| matches!(x.status, RowStatus::New | RowStatus::Clash | RowStatus::Lost)));
}

#[test]
fn a_scope_takes_the_file_under_its_node_and_lists_the_draft_rest_as_lost() {
    let state = demo();
    let f = file(&state);
    // The scope is the file's group: nothing of the draft's tree is in it (another uuid), so the
    // draft's fixtures in the MVR tree stay out — no Lost rows — and the file's rows are in.
    let group = f.scene.groups[0].uuid;
    let r = build(&state.root.show_data.devices, &f, &settings(), Some(&Scope::Node(group)), &BTreeMap::new(), "t.mvr", None, Some("MvrGroup 9"));
    assert_eq!(r.scope.as_deref(), Some("MvrGroup 9"));
    assert!(r.rows.iter().all(|x| x.status != RowStatus::Lost));
    assert_eq!(r.rows.iter().filter(|x| x.status == RowStatus::New).count(), 1);
    // A draft node as the scope: the file side is empty (its uuids aren't the draft's), the
    // draft's fixtures under the node that no file row matched are Lost.
    let node = state.root.show_data.devices.metadata.mvr.layers.values().next().map(|l| l.meta.identity.guid);
    if let Some(node) = node {
        let r = build(&state.root.show_data.devices, &f, &settings(), Some(&Scope::Node(node)), &BTreeMap::new(), "t.mvr", None, None);
        assert!(r.rows.iter().all(|x| x.status == RowStatus::Lost));
    }
}

#[test]
fn the_words_ask_need_the_claim_open_list_and_cancel_the_review() {
    let mut state = demo();
    // A bare Import Mvr asks with the files element on mvr/.
    let ask = match dispatch(&mut state, "Import Mvr") {
        DispatchResult::Ask(a) => *a,
        _ => panic!("a bare Import Mvr must ask for the file"),
    };
    assert!(matches!(ask.view().inputs[0].kind, lumacore_wire::InputKind::File { ref files } if files == "mvr"));
    assert!(run(&mut state, "Import Mvr \"rig.mvr\"").contains("E-CMD-010"), "needs the claim");
    assert!(run(&mut state, "Import Mvr /Cancel").contains("E-CMD-009"));
    run(&mut state, "Claim Root.ShowData.Devices");
    assert!(run(&mut state, "Import Mvr \"nope.mvr\"").contains("E-CMD-012"));
    // The fixture file on a folder drive (its library tree laid out by hand).
    let dir = std::env::temp_dir().join(format!("lumacore-mvr-test-{}", std::process::id()));
    let lib = dir.join("LumaCore/library/mvr");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::copy(Path::new(env!("LUMACORE_ROOT")).join("tests/fixtures/rig.mvr"), lib.join("rig.mvr")).unwrap();
    state.drives.push(crate::drives::Drive { id: "folder1".into(), kind: DriveKind::Folder, label: "Fixtures".into(), mount: dir.clone(), present: true, free: 1, total: 1, since_ms: 0 });
    // The file's universes land past the rig's, so the rows change addresses, never clash.
    let out = run(&mut state, "Import Mvr \"rig.mvr\" /Drive \"folder1\" /Names Take /UniverseOffset 110");
    assert!(out.contains("Review \"rig.mvr\""), "{out}");
    let review = state.patch_draft.as_ref().unwrap().review.clone().expect("a review on the draft");
    assert_eq!(review.drive.as_deref(), Some("folder1"));
    // Aura L1–L4 match the rig's 101–104 by FID with their names taken; the Spots meet 201–202
    // the same way, their GDTF in neither the zip nor the show.
    let r101 = review.row(101).expect("row 101");
    assert_eq!((r101.status, r101.match_key.as_str()), (RowStatus::Changed, "FID"));
    assert!(r101.taken, "Names Take ticks the row");
    let spot = review.rows.iter().find(|r| r.name == "Spot 1").unwrap();
    assert_eq!((spot.status, spot.no, spot.gdtf_embedded), (RowStatus::Changed, 201, false));
    assert!(run(&mut state, "List Mvr").contains("Spot 1"));
    let info = run(&mut state, "Info Mvr");
    assert!(info.contains("Match Auto") && info.contains("Names Take"), "{info}");
    assert!(run(&mut state, "Import Mvr /Cancel").contains("cancelled"));
    assert!(state.patch_draft.as_ref().unwrap().review.is_none());
    assert!(run(&mut state, "Import Mvr /Apply").contains("E-CMD-009"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn apply_asks_for_a_gone_group_then_writes_the_ticked_rows_and_keep_waits_for_it() {
    let mut state = demo();
    run(&mut state, "Claim Root.ShowData.Devices");
    let dir = std::env::temp_dir().join(format!("lumacore-mvr-apply-{}", std::process::id()));
    let lib = dir.join("LumaCore/library/mvr");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::copy(Path::new(env!("LUMACORE_ROOT")).join("tests/fixtures/rig.mvr"), lib.join("rig.mvr")).unwrap();
    state.drives.push(crate::drives::Drive { id: "folder1".into(), kind: DriveKind::Folder, label: "Fixtures".into(), mount: dir.clone(), present: true, free: 1, total: 1, since_ms: 0 });
    run(&mut state, "Import Mvr \"rig.mvr\" /Drive \"folder1\" /Names Take");
    // Keep is refused while the review stands.
    assert!(run(&mut state, "Unclaim Root.ShowData.Devices /Keep").contains("E-CMD-010"));
    // A cell: the name of 102 (taken by /Names Take) offered again, then taken.
    let name_taken = |state: &AppState| state.patch_draft.as_ref().unwrap().review.as_ref().unwrap().row(102).unwrap().cells.iter().any(|c| c.field == "name" && c.taken);
    assert!(name_taken(&state));
    assert!(run(&mut state, "Import Mvr 102 /Offer \"Name\"").contains("Offered name"));
    assert!(!name_taken(&state) && !state.patch_draft.as_ref().unwrap().review.as_ref().unwrap().row(102).unwrap().taken);
    assert!(run(&mut state, "Import Mvr 102 /Take \"Name\"").contains("Took name"));
    assert!(name_taken(&state));
    // Apply with a review that adds fixtures: the same file with offsets past the rig makes the
    // Auras new (numbers 20101…, universe 111 made on the way; the type comes from the zip);
    // their group "Upstage truss" is gone from the show → the ask, one per group; the Spots have
    // no GDTF anywhere and are skipped, named.
    run(&mut state, "Import Mvr /Cancel");
    run(&mut state, "Import Mvr \"rig.mvr\" /Drive \"folder1\" /FidOffset 20000 /UniverseOffset 110");
    let review = state.patch_draft.as_ref().unwrap().review.clone().unwrap();
    let r1101 = review.row(20101).expect("Aura L1 new at 20101");
    assert!(r1101.status == RowStatus::New && r1101.taken && r1101.group_missing && r1101.gdtf_embedded, "{r1101:?}");
    let ask = match dispatch(&mut state, "Import Mvr /Apply") {
        DispatchResult::Ask(a) => *a,
        DispatchResult::Output(o) => panic!("Apply must ask for the gone group, not: {}", String::from(o)),
        _ => panic!("Apply must ask for the gone group"),
    };
    assert_eq!(ask.title(), "Import MVR · a group is gone");
    // The primary button with the toggle off: every missing group takes the answer.
    let recreate = ask.primary_line().unwrap();
    assert!(recreate.starts_with("Import Mvr ") && recreate.ends_with("/Apply /Gone Recreate"), "{recreate}");
    let before = state.root.show_data.devices.dmx.patch.fixtures.len();
    let out = match dispatch(&mut state, &recreate) {
        DispatchResult::Output(o) => String::from(o),
        DispatchResult::Ask(a) => panic!("asked again: {}", a.title()),
        _ => panic!("no output"),
    };
    assert!(out.contains("Applied \"rig.mvr\"") && out.contains("4 new"), "{out}");
    assert!(out.contains("skipped") && out.contains("Spot 1"), "{out}");
    let show = &state.root.show_data;
    let patch = &show.devices.dmx.patch;
    assert_eq!(patch.fixtures.len(), before + 4);
    let f = &patch.fixture(20101).expect("resolved");
    assert_eq!(f.meta.name().unwrap(), "Aura L1");
    assert_eq!(show.universe_guid(111), f.primary_break().map(|b| b.universe_id));
    assert_eq!(f.primary_break().unwrap().start_address, 1);
    let truss = show.devices.metadata.mvr.groups.values().find(|g| g.meta.name() == Some("Upstage truss")).expect("the group recreated");
    assert_eq!(f.mvr.parent, Some(truss.meta.identity.guid));
    assert!(state.patch_draft.as_ref().unwrap().review.is_none());
    // Keep works again (the Auras bring an attribute the rig never had: answered Default).
    assert!(run(&mut state, "Unclaim Root.ShowData.Devices /Keep /Default").contains("kept"));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A file with a folder — a console's fixture-tree node: a fixture of a channel-less type holding
/// fixtures, no number, an untyped CustomId — and its two fixtures, in the file's order (b before
/// a). The GDTF is one the rig imported (`Generic@Dimmer.gdtf` by source name), so no zip is
/// read; the addresses sit on a universe the rig lacks.
fn tree_file() -> (MvrFile, Uuid, Uuid, Uuid) {
    let layer = Uuid::new_v4();
    let folder = Uuid::new_v4();
    let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
    let fx = |uuid: Uuid, name: &str, parent: Uuid, fid: Option<u32>, custom_id: Option<u32>, channel: u16| MvrFixture {
        uuid,
        name: name.into(),
        parent,
        layer,
        matrix: Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 0.0]]),
        gdtf_spec: "Generic@Dimmer.gdtf".into(),
        gdtf_mode: "Standard".into(),
        addresses: if channel == 0 { vec![] } else { vec![MvrAddress { dmx_break: 0, universe: 256, channel }] },
        fixture_id: fid.map(|n| n.to_string()).unwrap_or_default(),
        fixture_id_numeric: fid,
        custom_id,
        ..MvrFixture::default()
    };
    let file = MvrFile {
        scene: MvrScene {
            layers: vec![MvrLayer { uuid: layer, name: "Fixture Layer".into() }],
            fixtures: vec![fx(folder, "Stage 1", layer, None, Some(7777), 0), fx(b, "Spot B", folder, Some(20902), Some(0), 20), fx(a, "Spot A", folder, Some(20901), Some(0), 1)],
            ..MvrScene::default()
        },
        entries: vec![],
    };
    (file, folder, a, b)
}

/// A folder in the file comes in as a fixture holding fixtures (decisions §40 3), its id typed by
/// the preset or `/IdType` (§40 9, 10), in the file's order (§40 6); a second import finds it by
/// uuid and makes nothing twice.
#[test]
fn a_folder_in_the_file_is_a_fixture_holding_fixtures() {
    let mut state = demo();
    let (file, folder, a, b) = tree_file();
    let r = build(&state.root.show_data.devices, &file, &settings(), None, &BTreeMap::new(), "t.mvr", None, None);
    let row = |uuid: Uuid| r.rows.iter().find(|x| x.uuid == uuid).expect("a row");
    let f = row(folder);
    assert_eq!((f.status, f.no, f.id.as_str(), f.id_type.as_str(), f.cid, f.order), (RowStatus::New, 0, "Generic 7777", "Generic", Some(7777), 10), "{f:?}");
    assert!(f.taken && !f.group_missing && f.parent_fixture.is_none());
    assert_eq!((row(b).parent_fixture, row(b).order, row(a).order), (Some(folder), 10, 20));
    // `/IdType Media` types it instead.
    let mut media = settings();
    media.id_type = lumacore_fixtures::IdType::Media;
    let r2 = build(&state.root.show_data.devices, &file, &media, None, &BTreeMap::new(), "t.mvr", None, None);
    assert_eq!(r2.rows.iter().find(|x| x.uuid == folder).map(|x| x.id.as_str()), Some("Media 7777"));
    // Written: the folder has no number, its children sit under it, the order is the file's.
    run(&mut state, "Claim Root.ShowData.Devices");
    state.patch_draft.as_mut().expect("the draft").review = Some(r2);
    let out = run(&mut state, "Import Mvr /Apply");
    assert!(out.contains("3 new"), "{out}");
    let patch = &state.root.show_data.devices.dmx.patch;
    let (slot, fold) = patch.fixture_by_id(lumacore_fixtures::IdType::Media, 7777).expect("the folder");
    assert_eq!((fold.mvr.fid, fold.handle().as_str(), fold.mvr.order), (None, "Fixture Media 7777", 10));
    assert_eq!(patch.fixture(20902).map(|f| (f.mvr.parent, f.mvr.order)), Some((Some(folder), 10)));
    assert_eq!(patch.fixture(20901).map(|f| (f.mvr.parent, f.mvr.order)), Some((Some(folder), 20)));
    assert!(patch.fixtures.get(&slot).is_some());
    // The same file again: the folder is Same by uuid, nothing is made twice.
    let r3 = build(&state.root.show_data.devices, &file, &media, None, &BTreeMap::new(), "t.mvr", None, None);
    let f3 = r3.rows.iter().find(|x| x.uuid == folder).expect("a row");
    assert_eq!((f3.status, f3.matched), (RowStatus::Same, Some(folder)));
    state.patch_draft.as_mut().expect("the draft").review = Some(r3);
    let out = run(&mut state, "Import Mvr /Apply");
    assert!(!out.contains("new"), "{out}");
    // The rig's 23 folders plus this one.
    assert_eq!(state.root.show_data.devices.dmx.patch.fixtures.values().filter(|f| f.mvr.fid.is_none()).count(), 24);
}

/// `Import Mvr 20901 Thru 999999 /Apply` visits the rows that exist (the one range rule); a single
/// number must exist.
#[test]
fn apply_takes_a_range_over_the_rows_that_exist() {
    let mut state = demo();
    let (file, ..) = tree_file();
    run(&mut state, "Claim Root.ShowData.Devices");
    let r = build(&state.root.show_data.devices, &file, &settings(), None, &BTreeMap::new(), "t.mvr", None, None);
    state.patch_draft.as_mut().expect("the draft").review = Some(r);
    assert!(run(&mut state, "Import Mvr 999999 /Apply").contains("E-CMD-007"));
    let out = run(&mut state, "Import Mvr 20901 Thru 999999 /Apply");
    assert!(out.contains("2 new") && !out.contains("E-"), "the two numbered rows, not the folder: {out}");
}

/// The real rig, when its file is beside the station (the demo's source, decisions §40 15):
/// 1 348 fixtures and 23 folders, 106 universes, the folders nested as the desk drew them.
#[test]
fn the_international_2026_imports_with_its_tree() {
    let source = Path::new(env!("LUMACORE_ROOT")).join("library/mvr/ti2026.mvr");
    if !source.exists() {
        eprintln!("skipped: {} is not here", source.display());
        return;
    }
    // An empty show: the rig's numbers meet nothing.
    let mut state = AppState::new();
    let _ = crate::commands::navigate::cmd_enter(&mut state, &["patch"]);
    run(&mut state, "Claim Root.ShowData.Devices");
    let dir = std::env::temp_dir().join(format!("lumacore-mvr-ti2026-{}", std::process::id()));
    let lib = dir.join("LumaCore/library/mvr");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::copy(&source, lib.join("ti2026.mvr")).unwrap();
    state.drives.push(crate::drives::Drive { id: "folder1".into(), kind: DriveKind::Folder, label: "Rig".into(), mount: dir.clone(), present: true, free: 1, total: 1, since_ms: 0 });
    let out = run(&mut state, "Import Mvr \"ti2026.mvr\" /Drive \"folder1\" /IdType Media");
    let clashes: Vec<String> = state.patch_draft.as_ref().unwrap().review.as_ref().unwrap().rows.iter().filter(|r| r.clash.is_some()).map(|r| format!("{} {} {:?} parent={:?}", r.name, r.id, r.clash, r.parent_fixture)).collect();
    assert!(out.contains("1371 rows") && out.contains("1371 new"), "{out}\n{clashes:#?}");
    // The file's one MVR group (PIXSTICK) is new to the show: recreated on the line.
    let out = run(&mut state, "Import Mvr /Apply /Gone Recreate");
    assert!(out.contains("1371 new"), "{out}");
    let patch = &state.root.show_data.devices.dmx.patch;
    assert_eq!(patch.fixtures.values().filter(|f| f.mvr.fid.is_none()).count(), 23, "the folders");
    assert_eq!(patch.fids().len(), 1348, "the rig's numbered fixtures");
    let (_, icarus) = patch.fixture_by_id(lumacore_fixtures::IdType::Media, 7003).expect("ACME Icarus");
    assert_eq!(icarus.meta.name().unwrap_or(""), "ACME Icarus");
    let children = patch.fixtures.values().filter(|f| f.mvr.parent == Some(icarus.meta.identity.guid)).count();
    assert_eq!(children, 286);
    let (_, stage) = patch.fixture_by_id(lumacore_fixtures::IdType::Media, 1).expect("Stage 1 Mainstage");
    assert_eq!(icarus.mvr.parent, Some(stage.meta.identity.guid), "nested two deep");
    let _ = std::fs::remove_dir_all(&dir);
}
