// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Object paint on the rig (decisions 0294): the one Paint parser on the line, painting many,
//! the colour order (own → the painted tag with the highest Priority → its parent's → the pool),
//! theme slot links per viewer, the theme's slots, feature groups on their slots, the objects
//! that can be painted, the picker's recent colours and mode (the tags' order: lumacore-cmd-tags).
//!
//! Owns:          the tests of Paint across lumacore-objects and lumacore-state's `paint`.
//! Doesn't own:   the feeds' colours (lumacore-views), the parser's own tests (lumacore-base).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_base::Paint;
use lumacore_base::paint::hex_rgb;

use crate::paint::Painter;
use crate::{AppState, DispatchResult};

fn run(state: &mut AppState, line: &str) -> String {
    match crate::commands::dispatch(state, line) {
        DispatchResult::Output(s) => crate::text::strip_ansi(&String::from(s)),
        DispatchResult::Ask(a) => format!("ASK {}", a.title()),
        _ => String::new(),
    }
}

fn rgb(hex: &str) -> Option<Paint> {
    hex_rgb(hex).map(Paint::Rgb)
}

fn group_paint(s: &AppState, no: u32) -> Option<Paint> {
    let dp = s.selected_datapool();
    s.root.show_data.datapools[&dp].groups[&no].meta.annotation.paint.clone()
}

fn slot(name: &str) -> &'static str {
    lumacore_tree::media::PAINT_SLOTS.iter().find(|(n, _)| *n == name).map(|(_, c)| *c).unwrap()
}

/// `Set … "Paint"` takes hex, a palette word (stored as its hex), a theme link or none; anything
/// else is refused with its code.
#[test]
fn a_paint_is_hex_a_palette_word_a_theme_link_or_none() {
    let mut s = crate::demo::fresh_state();
    assert!(run(&mut s, "Set Group 1 \"Paint\" \"#3F73C8\"").contains("Paint = #3f73c8"));
    assert_eq!(group_paint(&s, 1), rgb("#3f73c8"));
    let out = run(&mut s, "Set Group 1 \"Paint\" Blue");
    assert_eq!(group_paint(&s, 1), Paint::parse("Blue").unwrap(), "{out}");
    let out = run(&mut s, "Set Group 1 \"Paint\" Theme \"Position\"");
    assert!(out.contains("Paint = Theme \"Position\""), "{out}");
    assert_eq!(group_paint(&s, 1), Some(Paint::Theme("Position".into())));
    run(&mut s, "Set Group 1 \"Paint\" none");
    assert_eq!(group_paint(&s, 1), None);
    for bad in ["\"#12345\"", "\"reddish\"", "Theme \"\"", "\"#3f73c8ff\""] {
        let out = run(&mut s, &format!("Set Group 1 \"Paint\" {bad}"));
        assert!(out.contains("E-SET-003"), "{bad}: {out}");
    }
    assert_eq!(group_paint(&s, 1), None, "a refused value changes nothing");
}

/// One line paints a selection (the range rule): `Set Group 1 Thru 2 + 7 "Paint" Blue`.
#[test]
fn painting_many_is_one_line_through_the_range_rule() {
    let mut s = crate::demo::fresh_state();
    let out = run(&mut s, "Set Group 1 Thru 2 + 7 \"Paint\" Blue");
    assert!(!out.contains("E-"), "{out}");
    let blue = Paint::parse("Blue").unwrap();
    for n in [1, 2, 7] {
        assert_eq!(group_paint(&s, n), blue, "Group {n}");
    }
    assert_eq!(group_paint(&s, 3), None);
}

/// A theme link shows each viewer their own theme's slot; a slot the viewer's theme lacks is
/// kept, said and falls back to the tags, then the pool.
#[test]
fn a_theme_link_follows_each_viewers_theme_and_a_lost_slot_falls_back() {
    let mut s = crate::demo::fresh_state();
    let out = run(&mut s, "Store User \"Bob\"");
    assert!(!out.contains("E-"), "{out}");
    run(&mut s, "Set Group 1 \"Paint\" Theme \"Position\"");
    run(&mut s, "Set Theme Slot \"Position\" \"#3a62b8\"");
    let dp = s.selected_datapool();
    let g1 = s.root.show_data.datapools[&dp].groups[&1].meta.clone();
    let admin = Painter::new(&s.root.show_data, "Admin").of(&g1);
    let bob = Painter::new(&s.root.show_data, "Bob").of(&g1);
    assert_eq!((admin.hex.as_deref(), admin.from_text()), (Some("#3a62b8"), "Theme Position".to_string()));
    assert_eq!(bob.hex.as_deref(), Some(slot("Position")), "Bob's theme keeps LumaCore's");
    // A slot only Admin's theme has: Bob's link is broken and falls back to the pool.
    run(&mut s, "Store Theme Slot \"Haze\" \"#8fa3b8\"");
    run(&mut s, "Set Group 2 \"Paint\" Theme \"Haze\"");
    let g2 = s.root.show_data.datapools[&dp].groups[&2].meta.clone();
    assert_eq!(Painter::new(&s.root.show_data, "Admin").of(&g2).hex.as_deref(), Some("#8fa3b8"));
    let lost = Painter::new(&s.root.show_data, "Bob").of(&g2);
    assert_eq!((lost.hex.clone(), lost.broken.as_deref()), (None, Some("Haze")));
    assert_eq!(lost.from_text(), "the pool (Theme Haze is missing)");
    assert_eq!(group_paint(&s, 2), Some(Paint::Theme("Haze".into())), "the link is kept");
}

/// The theme's paint slots: LumaCore's changed and put back, one added, changed and removed;
/// a slot the theme lacks is refused; a profile's slots are its own.
#[test]
fn a_themes_paint_slots_are_set_added_and_reset() {
    let mut s = crate::demo::fresh_state();
    let theme = |s: &AppState| s.root.show_data.profile_for_user("Admin").unwrap().settings.theme.clone();
    let out = run(&mut s, "Set Theme Slot \"Position\" \"#3a62b8\"");
    assert!(out.contains("Paint \"Position\" = #3a62b8"), "{out}");
    assert_eq!(theme(&s).slot("position"), Some("#3a62b8"));
    assert!(theme(&s).slot_views().iter().any(|v| v.name == "Position" && v.changed));
    run(&mut s, "Reset Theme Slot \"Position\"");
    assert_eq!(theme(&s).slot("Position"), Some(slot("Position")));
    assert!(theme(&s).slots.is_empty(), "LumaCore's colour keeps no entry");

    let out = run(&mut s, "Store Theme Slot \"Haze\" \"#8fa3b8\"");
    assert!(out.contains("Added") && out.contains("#8fa3b8"), "{out}");
    run(&mut s, "Set Theme Slot \"Haze\" \"#000000\"");
    assert_eq!(theme(&s).slot("Haze"), Some("#000000"));
    assert!(run(&mut s, "Store Theme Slot \"Haze\"").contains("E-CMD-010"), "it exists already");
    assert!(run(&mut s, "Set Theme Slot \"Nope\" \"#000000\"").contains("E-CMD-007"));
    let out = run(&mut s, "Store Theme Slot \"Fog\"");
    assert!(out.contains("Added"), "{out}");
    assert!(theme(&s).slot("Fog").is_some(), "a new slot takes a palette colour");
    run(&mut s, "Reset Theme Slot \"Haze\"");
    assert_eq!(theme(&s).slot("Haze"), None, "an added slot goes");
    assert_eq!(theme(&s).slot_views().len(), lumacore_tree::media::PAINT_SLOTS.len() + 1);
    assert!(run(&mut s, "Set Theme Slot \"Gobo\" \"bluish\"").contains("E-SET-003"));
}

/// A feature group links the theme slot of its name; its own Paint wins; none is the slot again.
#[test]
fn a_feature_group_takes_the_theme_slot_of_its_name() {
    let mut s = crate::demo::fresh_state();
    let fg = |s: &AppState, name: &str| s.root.show_data.feature_groups.values().find(|g| g.meta.name() == Some(name)).unwrap().clone();
    let position = fg(&s, "Position");
    assert_eq!(position.meta.annotation.paint, Some(Paint::Theme("Position".into())), "seeded on its slot");
    let p = Painter::new(&s.root.show_data, "Admin");
    let paint = crate::commands::attributes::group_paint(&position);
    assert_eq!(p.of_or(&position.meta, Some(&paint)).hex.as_deref(), Some(slot("Position")));
    let out = run(&mut s, "Set FeatureGroup \"Position\" \"Paint\" Red");
    assert!(out.contains("Paint = #"), "{out}");
    assert_eq!(fg(&s, "Position").meta.annotation.paint, Paint::parse("Red").unwrap());
    run(&mut s, "Set FeatureGroup \"Position\" \"Paint\" none");
    let position = fg(&s, "Position");
    assert_eq!(position.meta.annotation.paint, None);
    assert_eq!(crate::commands::attributes::group_paint(&position), Paint::Theme("Position".into()));
    assert!(run(&mut s, "Set FeatureGroup \"Position\" \"Paint\" \"reddish\"").contains("E-SET-003"));
}

/// Every show object with a NodeMeta takes a Paint: the pools, cues, macros, views, executors,
/// effects, DataPools, profiles, images.
#[test]
fn every_object_with_a_meta_takes_a_paint() {
    for word in ["Group", "Preset", "Sequence", "Cue", "Macro", "View", "Executor", "Effect", "DataPool", "UserProfile", "Image", "Fixture", "TimingPreset", "CurvePreset"] {
        let o = crate::objects::find(word).unwrap_or_else(|| panic!("{word}"));
        assert!(o.props.is_some_and(|p| p.get("Paint").is_some()), "{word} takes a Paint");
    }
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Select Sequence 1");
    let out = run(&mut s, "Set Cue 2 \"Paint\" Green");
    assert!(out.contains("Cue 2 Paint = #"), "{out}");
    let (dp, seq) = s.selected_sequence().unwrap();
    let key = lumacore_tree::cue_key("2").unwrap();
    assert_eq!(s.root.show_data.datapools[&dp].sequences[&seq].cues[&key].meta.annotation.paint, Paint::parse("Green").unwrap());
    assert!(run(&mut s, "Set DataPool 1 \"Paint\" Teal").contains("DataPool 1 Paint = #"));
    assert_eq!(s.root.show_data.datapools[&1].meta.annotation.paint, Paint::parse("Teal").unwrap());
    let dp = s.selected_datapool();
    s.root.show_data.datapools.get_mut(&dp).unwrap().macros.insert(901, lumacore_tree::MacroItem { meta: lumacore_base::NodeMeta::new(Some("Rig".into())), lines: Vec::new() });
    assert!(run(&mut s, "Set Macro 901 \"Paint\" Theme \"FX\"").contains("Macro 901 Paint = Theme \"FX\""));
    let user = s.effective_user_name().to_string();
    s.root.show_data.profile_for_user_mut(&user).unwrap().content.views.insert(901, lumacore_tree::StoredView { meta: lumacore_base::NodeMeta::new(Some("Paint view".into())), screens: Default::default() });
    let out = run(&mut s, "Set View 901 \"Paint\" Violet");
    assert!(out.contains("View 901 Paint = #"), "{out}");
    let out = run(&mut s, "Set UserProfile \"Paint\" Amber");
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(s.root.show_data.profile_for_user("Admin").unwrap().meta.annotation.paint, Paint::parse("Amber").unwrap());
}

/// The picker's recent colours follow the painter's lines, newest first; its mode is a profile
/// setting.
#[test]
fn the_picker_remembers_recent_colours_and_its_mode() {
    let mut s = crate::demo::fresh_state();
    run(&mut s, "Set Group 1 \"Paint\" Blue");
    run(&mut s, "Set Group 2 \"Paint\" \"#010203\"");
    run(&mut s, "Set Group 3 \"Paint\" Theme \"Dimmer\"");
    run(&mut s, "Set Group 4 \"Paint\" Blue");
    let prefs = |s: &AppState| s.root.show_data.profile_for_user("Admin").unwrap().settings.preferences.clone();
    let blue = Paint::parse("Blue").unwrap().unwrap().word();
    assert_eq!(prefs(&s).paint_recent, vec![blue, "#010203".to_string()], "a theme link isn't a colour to remember");
    assert_eq!(prefs(&s).paint_mode, "Palette");
    // `Theme` is an object word too: the profile is named by its number (the paint feed's line).
    let no = s.root.show_data.profile_id_for_user("Admin").unwrap();
    let out = run(&mut s, &format!("Set UserProfile {no} \"PaintMode\" Theme"));
    assert!(!out.contains("E-"), "{out}");
    assert_eq!(prefs(&s).paint_mode, "Theme");
}
