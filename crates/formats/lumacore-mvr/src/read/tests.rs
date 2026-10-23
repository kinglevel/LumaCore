// ██████╗ ███████╗ █████╗ ██████╗     ████████╗███████╗███████╗████████╗███████╗
// ██╔══██╗██╔════╝██╔══██╗██╔══██╗    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██████╔╝█████╗  ███████║██║  ██║       ██║   █████╗  ███████╗   ██║   ███████╗
// ██╔══██╗██╔══╝  ██╔══██║██║  ██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║  ██║███████╗██║  ██║██████╔╝       ██║   ███████╗███████║   ██║   ███████║
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝        ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: read tests

//! Tests of `read`: an MVR built in memory — the scene with nested groups, a child fixture, both
//! address forms, a matrix, an embedded and a missing GDTF — and what `read` and `head` make of
//! it, and what they refuse.
//!
//! Owns:          the unit tests of read.rs (and scene.rs through it).
//! Doesn't own:   the station's side (the files feed, the import).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 "34g4 patch hardened" 10, "Import MVR hardened".
// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS


use std::io::{Cursor, Write};

use uuid::Uuid;
use zip::write::SimpleFileOptions;

use crate::scene::{MvrAddress, parse_matrix};
use crate::{MvrError, head, read};

const L1: &str = "11111111-1111-1111-1111-111111111111";
const G1: &str = "22222222-2222-2222-2222-222222222222";
const F1: &str = "33333333-3333-3333-3333-333333333333";
const F2: &str = "44444444-4444-4444-4444-444444444444";
const F3: &str = "55555555-5555-5555-5555-555555555555";
const C1: &str = "66666666-6666-6666-6666-666666666666";

fn scene_xml() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<GeneralSceneDescription verMajor="1" verMinor="6" provider="Vectorworks" providerVersion="2026">
  <UserData/>
  <Scene>
    <AUXData>
      <Class uuid="{C1}" name="Truss"/>
      <Position uuid="77777777-7777-7777-7777-777777777777" name="Upstage"/>
    </AUXData>
    <Layers>
      <Layer uuid="{L1}" name="Stage">
        <ChildList>
          <GroupObject uuid="{G1}" name="Truss 2">
            <ChildList>
              <Fixture uuid="{F1}" name="Aura L1">
                <Matrix>{{1,0,0}}{{0,1,0}}{{0,0,1}}{{-4200,2100,6000}}</Matrix>
                <GDTFSpec>AuraXB.gdtf</GDTFSpec>
                <GDTFMode>Standard</GDTFMode>
                <Addresses><Address break="0">1</Address></Addresses>
                <FixtureID>101</FixtureID>
                <FixtureIDNumeric>101</FixtureIDNumeric>
                <UnitNumber>1</UnitNumber>
                <Classing>{C1}</Classing>
                <ChildList>
                  <Fixture uuid="{F3}" name="Aura L1 sub">
                    <GDTFSpec>AuraXB.gdtf</GDTFSpec><GDTFMode>Standard</GDTFMode>
                    <Addresses><Address break="0">2.5</Address></Addresses>
                    <FixtureIDNumeric>103</FixtureIDNumeric>
                  </Fixture>
                </ChildList>
              </Fixture>
              <SceneObject uuid="88888888-8888-8888-8888-888888888888" name="Truss piece"/>
            </ChildList>
          </GroupObject>
          <Fixture uuid="{F2}" name="Spot 1">
            <GDTFSpec>Missing Spot.gdtf</GDTFSpec><GDTFMode>Mode 1</GDTFMode>
            <Addresses><Address break="0">513</Address><Address break="1">1030</Address></Addresses>
            <FixtureIDNumeric>201</FixtureIDNumeric>
            <CustomId>7</CustomId><CustomIdType>1</CustomIdType>
          </Fixture>
        </ChildList>
      </Layer>
    </Layers>
  </Scene>
</GeneralSceneDescription>"#
    )
}

fn mvr(scene: &str, gdtfs: &[&str]) -> Vec<u8> {
    let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let o = SimpleFileOptions::default();
    w.start_file("GeneralSceneDescription.xml", o).unwrap();
    w.write_all(scene.as_bytes()).unwrap();
    for g in gdtfs {
        w.start_file(*g, o).unwrap();
        w.write_all(b"not really a gdtf").unwrap();
    }
    w.finish().unwrap().into_inner()
}

#[test]
fn the_scene_is_read_with_its_tree_addresses_and_gdtfs() {
    let file = read(&mvr(&scene_xml(), &["AuraXB.gdtf"])).unwrap();
    let s = &file.scene;
    assert_eq!((s.ver_major, s.ver_minor), (1, 6));
    assert_eq!(s.provider, "Vectorworks");
    assert_eq!(s.layers.len(), 1);
    assert_eq!(s.groups.len(), 1);
    assert_eq!(s.groups[0].parent, Uuid::parse_str(L1).unwrap());
    assert_eq!(s.fixtures.len(), 3);
    assert_eq!(s.classes[0].name, "Truss");
    assert_eq!(s.positions.len(), 1);
    assert_eq!(s.other_objects, 1);
    let f1 = &s.fixtures[0];
    assert_eq!(f1.name, "Aura L1");
    assert_eq!(f1.parent, Uuid::parse_str(G1).unwrap());
    assert_eq!(f1.layer, Uuid::parse_str(L1).unwrap());
    assert_eq!(f1.matrix.unwrap()[3], [-4200.0, 2100.0, 6000.0]);
    assert_eq!(f1.addresses, [MvrAddress { dmx_break: 0, universe: 1, channel: 1 }]);
    assert_eq!(f1.fixture_id_numeric, Some(101));
    assert_eq!(f1.classing, Some(Uuid::parse_str(C1).unwrap()));
    // The child fixture: its parent is the fixture; `2.5` is universe.channel.
    let f3 = &s.fixtures[1];
    assert_eq!(f3.parent, Uuid::parse_str(F1).unwrap());
    assert_eq!(f3.addresses, [MvrAddress { dmx_break: 0, universe: 2, channel: 5 }]);
    // Absolute addresses: 513 = 2.001, 1030 = 3.006 on break 1.
    let f2 = &s.fixtures[2];
    assert_eq!(f2.addresses, [MvrAddress { dmx_break: 0, universe: 2, channel: 1 }, MvrAddress { dmx_break: 1, universe: 3, channel: 6 }]);
    assert_eq!((f2.custom_id, f2.custom_id_type), (Some(7), Some(1)));
    assert_eq!(f2.parent, Uuid::parse_str(L1).unwrap());
    // The head: two GDTFs named, one embedded, one missing.
    let h = file.head();
    assert_eq!((h.layers, h.groups, h.fixtures), (1, 1, 3));
    assert_eq!(h.gdtf_named, ["AuraXB.gdtf", "Missing Spot.gdtf"]);
    assert_eq!(h.gdtf_embedded, 1);
    assert_eq!(h.gdtf_missing, ["Missing Spot.gdtf"]);
    assert!(file.has_gdtf("auraxb"));
    assert_eq!(crate::gdtf_entry(Cursor::new(mvr(&scene_xml(), &["AuraXB.gdtf"])), "AuraXB.gdtf").unwrap().as_deref(), Some(&b"not really a gdtf"[..]));
    assert_eq!(crate::gdtf_entry(Cursor::new(mvr(&scene_xml(), &[])), "AuraXB.gdtf").unwrap(), None);
    assert_eq!(head(&mvr(&scene_xml(), &["gdtf/AuraXB.GDTF", "gdtf/Missing Spot.gdtf"])).unwrap().gdtf_missing.len(), 0);
}

#[test]
fn what_is_refused() {
    assert!(matches!(read(b"not a zip"), Err(MvrError::NotAZip(_))));
    let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
    w.start_file("other.txt", SimpleFileOptions::default()).unwrap();
    w.write_all(b"x").unwrap();
    assert!(matches!(read(&w.finish().unwrap().into_inner()), Err(MvrError::NoScene)));
    assert!(matches!(read(&mvr("<GeneralSceneDescription><Scene>", &[])), Err(MvrError::BadXml(_))));
    assert!(matches!(read(&mvr("<Other/>", &[])), Err(MvrError::BadXml(_))));
    let bad = scene_xml().replace(&format!("uuid=\"{F2}\""), "uuid=\"nope\"");
    assert!(matches!(read(&mvr(&bad, &[])), Err(MvrError::BadUuid("Fixture", _))));
    assert_eq!(MvrAddress::parse(0, "0"), None);
    assert_eq!(MvrAddress::parse(0, "1.513"), None);
    assert_eq!(parse_matrix("{1,0,0}{0,1,0}"), None);
}
#[test]
fn a_file_before_1_6_carries_its_number_in_the_fixture_id_text() {
    // Capture, Vectorworks and Depence 1.4 / 1.5 files: `<FixtureID>4005</FixtureID>`, no numeric tag.
    let xml = scene_xml()
        .replace("<FixtureID>101</FixtureID>\n                <FixtureIDNumeric>101</FixtureIDNumeric>", "<FixtureID>4005</FixtureID>")
        .replace("<FixtureIDNumeric>201</FixtureIDNumeric>", "<FixtureID>Spot A</FixtureID>");
    let file = read(&mvr(&xml, &[])).unwrap();
    let s = &file.scene;
    assert_eq!((s.fixtures[0].fixture_id.as_str(), s.fixtures[0].fixture_id_numeric), ("4005", Some(4005)));
    // A name in the text is no number.
    assert_eq!((s.fixtures[2].fixture_id.as_str(), s.fixtures[2].fixture_id_numeric), ("Spot A", None));
}
