// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `insert`: the show's types with their modes and use, a library file matched to the
//! show by its hash, a mode's channel map.
//!
//! Owns:          the unit tests of insert.rs.
//! Doesn't own:   reading the library (gdtf_library).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 round 3 2–4.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn aura() -> (Vec<u8>, FixtureType) {
    let bytes = std::fs::read(std::path::Path::new(env!("LUMACORE_ROOT")).join("tests/fixtures/AuraXB.gdtf")).unwrap();
    let ft = lumacore_objects::commands::gdtf_library::read_bytes(&bytes).unwrap();
    (bytes, ft)
}

#[test]
fn the_show_lists_its_types_and_a_library_file_matches_by_hash() {
    let (bytes, ft) = aura();
    let mut devices = DevicesData::default();
    devices.dmx.patch.fixture_types.insert(3, std::sync::Arc::new(ft.clone()));
    let file = LibraryFile {
        file: "Martin/AuraXB.gdtf".into(),
        size: bytes.len() as u64,
        sha256: ft.revisions[0].file_hash.clone(),
        fixture_type: Ok(ft.clone()),
        details: serde_json::Value::Null,
    };
    let v = list(&devices, &[file]);
    assert_eq!(v["show"][0]["no"], 3);
    assert_eq!(v["show"][0]["used"], 0);
    assert!(v["show"][0]["modes"][0]["footprint"].as_u64().unwrap() > 0);
    assert_eq!(v["library"][0]["in_show"], 3, "the same file is the show's type 3");
    // A mode's channel map: each channel with its attribute and break.
    let m = mode(&ft, 0, 1, &serde_json::Value::Null).unwrap();
    let ch = m["channels"].as_array().unwrap();
    assert!(!ch.is_empty());
    assert!(ch.iter().any(|c| c["attribute"] == "Dimmer"), "{ch:?}");
    assert!(mode(&ft, 0, 99, &serde_json::Value::Null).is_none());
}
