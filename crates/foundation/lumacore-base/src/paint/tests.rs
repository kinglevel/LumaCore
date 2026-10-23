// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of the one Paint parser and how a Paint is saved (decisions 0294).
//!
//! Owns:          the unit tests of paint.rs.
//! Doesn't own:   the colour a viewer sees (lumacore-state `paint`, tested on the rig).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn a_paint_reads_hex_words_theme_links_and_none() {
    assert_eq!(Paint::parse("#3F73C8"), Ok(Some(Paint::Rgb([0x3f, 0x73, 0xc8]))));
    assert_eq!(Paint::parse("3f73c8"), Ok(Some(Paint::Rgb([0x3f, 0x73, 0xc8]))));
    assert_eq!(Paint::parse("\"#3f73c8\""), Ok(Some(Paint::Rgb([0x3f, 0x73, 0xc8]))));
    assert_eq!(Paint::parse("blue"), Ok(hex_rgb(PALETTE[9].1[NAMED_TONE]).map(Paint::Rgb)));
    assert_eq!(Paint::parse("White"), Ok(Some(Paint::Rgb([255, 255, 255]))));
    assert_eq!(Paint::parse("Theme \"Position\""), Ok(Some(Paint::Theme("Position".into()))));
    assert_eq!(Paint::parse("theme Special 1"), Ok(Some(Paint::Theme("Special 1".into()))));
    assert_eq!(Paint::parse("none"), Ok(None));
    assert_eq!(Paint::parse(""), Ok(None));
    assert_eq!(Paint::parse("\"\""), Ok(None));
}

#[test]
fn a_paint_refuses_what_is_none_of_them() {
    for bad in ["#12345", "#1234567", "#3f73c8ff", "Bluish", "Theme", "Theme \"\"", "Themes \"x\"", "#gg0000"] {
        assert!(Paint::parse(bad).is_err(), "{bad} should be refused");
    }
    assert!(Paint::parse("Theme \"a name far longer than any slot may be\"").is_err());
}

#[test]
fn a_paint_is_saved_as_hex_or_a_theme_link() {
    assert_eq!(serde_json::to_value(Paint::Rgb([1, 2, 3])).unwrap(), serde_json::json!("#010203"));
    assert_eq!(serde_json::to_value(Paint::Theme("Gobo".into())).unwrap(), serde_json::json!({ "Theme": "Gobo" }));
    let back: Paint = serde_json::from_value(serde_json::json!({ "Theme": "Gobo" })).unwrap();
    assert_eq!(back, Paint::Theme("Gobo".into()));
    let old: Paint = serde_json::from_value(serde_json::json!([60, 143, 60, 255])).unwrap();
    assert_eq!(old, Paint::Rgb([60, 143, 60]));
    assert!(serde_json::from_value::<Paint>(serde_json::json!("blue")).is_err());
}

#[test]
fn the_named_words_are_the_palette_and_read_back() {
    for (name, hex) in NAMED {
        let rgb = hex_rgb(hex).expect("a palette hex");
        assert_eq!(palette_name(rgb), Some(name));
        assert_eq!(Paint::parse(name).unwrap().unwrap().word(), hex);
    }
    assert_eq!(Paint::Theme("Position".into()).word(), "Theme \"Position\"");
}
