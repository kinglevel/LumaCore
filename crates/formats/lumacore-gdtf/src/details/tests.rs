// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `details`: a real file's facts, a broken one refused.
//!
//! Owns:          the unit tests of details.rs.
//! Doesn't own:   the importer's tests.
//! Entry points:  the #[test] fns.
//! Decisions:     §34 round 3 (Insert fixtures v6).

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("LUMACORE_ROOT")).join("tests/fixtures").join(name)).expect("the test fixture exists")
}

#[test]
fn a_file_tells_its_names_version_and_revisions() {
    let d = file_details(&fixture("AuraXB.gdtf")).unwrap();
    assert!(!d.manufacturer.is_empty() && !d.long_name.is_empty(), "{d:?}");
    assert!(d.gdtf_version.contains('.'));
    assert!(d.long_name.contains("Aura"), "{d:?}");
}

#[test]
fn the_thumbnail_is_the_png_beside_the_description() {
    let (bytes, mime) = thumbnail(&fixture("AuraXB.gdtf")).unwrap();
    assert_eq!(mime, "image/png");
    assert!(bytes.starts_with(b"\x89PNG"));
}

#[test]
fn a_broken_file_is_refused() {
    assert!(file_details(b"not a zip").is_err());
}
