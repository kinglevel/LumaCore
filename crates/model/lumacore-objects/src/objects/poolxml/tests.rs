// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of Import PoolXml: a small export on a folder drive, the real extract when it is beside
//! the station.
//!
//! Owns:          the unit tests of poolxml.rs.
//! Doesn't own:   the reader's tests (`lumacore-poolxml`).
//! Entry points:  the #[test] fns.
//! Decisions:     §40 13.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use std::path::Path;

use lumacore_tree::DriveKind;

use crate::commands::dispatch;
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        // Unclaim leaves the patch context (the tree shell's Enter).
        DispatchResult::ContextSwitch(c) => {
            state.context = c;
            String::new()
        }
        DispatchResult::Ask(a) => panic!("'{line}' asked: {}", a.title()),
        _ => panic!("'{line}' did not produce output"),
    }
}

/// The demo show with a folder drive holding `name` with `xml`.
fn on_a_drive(name: &str, xml: &str) -> AppState {
    let mut state = crate::demo::fresh_state();
    let dir = std::env::temp_dir().join(format!("lumacore-poolxml-{}-{}", name.replace('.', "-"), std::process::id()));
    let lib = dir.join("LumaCore/library/poolxml");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(lib.join(name), xml).unwrap();
    state.drives.push(crate::drives::Drive { id: "folder1".into(), kind: DriveKind::Folder, label: "Pools".into(), mount: dir, present: true, free: 1, total: 1, since_ms: 0 });
    state
}

const POOL: &str = r#"<PoolExport DataVersion="2.2.5.2"><Pool Name="Default">
<Groups>
  <Group Name="Odd ones"><SelectionData><Item IDType="0" ID="101" X="0" Y="0" Z="0"/><Item IDType="0" ID="103" X="1" Y="0" Z="0"/><Item IDType="0" ID="105" X="2" Y="0" Z="0"/><Item IDType="0" ID="99999" X="3" Y="0" Z="0"/></SelectionData></Group>
  <Group Name="Cells"><SelectionData><Item IDType="0" ID="101.1" X="0" Y="0" Z="0"/></SelectionData></Group>
  <Group Name="Mixed"><SelectionData><Item IDType="0" ID="102" X="5" Y="7" Z="0"/><Item IDType="0" ID="102.1" X="6" Y="7" Z="0"/></SelectionData></Group>
</Groups>
<Sequences><Sequence Name="A"/></Sequences>
</Pool></PoolExport>"#;

#[test]
fn the_groups_of_an_export_become_groups_of_the_datapool() {
    let mut state = on_a_drive("pool.xml", POOL);
    assert!(run(&mut state, "Import PoolXml \"pool.xml\" /Drive \"folder1\"").contains("E-CMD-008"), "Groups is the word");
    let out = run(&mut state, "Import PoolXml \"pool.xml\" Groups /Drive \"folder1\"");
    // After the rig's 77 groups.
    assert!(out.contains("Imported 2 groups (78 Thru 79)"), "{out}");
    assert!(out.contains("skipped 1 of sub-fixtures only: Cells") && out.contains("left out 1 sub-fixture item and 1 fixture the patch lacks") && out.contains("1 sequences"), "{out}");
    let groups = state.user_groups();
    let odd = &groups[&78];
    assert_eq!(odd.meta.name(), Some("Odd ones"));
    let fids: Vec<u32> = odd.fixtures().iter().filter_map(|g| state.root.show_data.fixture_id_for_guid(*g)).collect();
    assert_eq!(fids, [101, 103, 105]);
    assert_eq!(odd.members[2].cell.x, 2);
    assert_eq!(groups[&79].members.len(), 1, "Mixed keeps its fixture, loses its cell item");
    assert!(run(&mut state, "Import PoolXml \"pool.xml\" Groups /Drive \"folder1\" /DataPool 9").contains("E-CMD-007"));
}

/// The real extract, when it is beside the station (the demo's source, decisions §40 15): the
/// Default pool's 87 groups against the TI rig's fixtures.
#[test]
fn the_international_2026_groups_import_when_the_rig_is_in() {
    let root = Path::new(env!("LUMACORE_ROOT"));
    let (mvr, xml) = (root.join("library/mvr/ti2026.mvr"), root.join("library/poolxml/ti2026-groups.xml"));
    if !mvr.exists() || !xml.exists() {
        eprintln!("skipped: the sources are not here");
        return;
    }
    let mut state = AppState::new();
    let _ = crate::commands::navigate::cmd_enter(&mut state, &["patch"]);
    run(&mut state, "Claim Root.ShowData.Devices");
    let dir = std::env::temp_dir().join(format!("lumacore-poolxml-ti2026-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("LumaCore/library/mvr")).unwrap();
    std::fs::create_dir_all(dir.join("LumaCore/library/poolxml")).unwrap();
    std::fs::copy(&mvr, dir.join("LumaCore/library/mvr/ti2026.mvr")).unwrap();
    std::fs::copy(&xml, dir.join("LumaCore/library/poolxml/ti2026-groups.xml")).unwrap();
    state.drives.push(crate::drives::Drive { id: "folder1".into(), kind: DriveKind::Folder, label: "Rig".into(), mount: dir.clone(), present: true, free: 1, total: 1, since_ms: 0 });
    run(&mut state, "Import Mvr \"ti2026.mvr\" /Drive \"folder1\" /IdType Media");
    run(&mut state, "Import Mvr /Apply /Gone Recreate");
    run(&mut state, "Unclaim Root.ShowData.Devices /Keep /Default");
    let out = run(&mut state, "Import PoolXml \"ti2026-groups.xml\" Groups /Drive \"folder1\"");
    assert!(out.contains("Imported 77 groups") && out.contains("skipped 10 of sub-fixtures only") && out.contains("0 fixtures the patch lacks"), "{out}");
    let groups = state.user_groups();
    let photon = groups.values().find(|g| g.meta.name() == Some("Photon")).expect("Photon");
    assert_eq!(photon.members.len(), 245);
    assert!(groups.values().all(|g| g.meta.name() != Some("Icarus LEDs")), "cells only: skipped");
    let _ = std::fs::remove_dir_all(&dir);
}
