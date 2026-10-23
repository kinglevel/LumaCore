// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of the upgrade steps (decisions 0289): one step per format from the oldest, the refusals,
//! and each step's rewrite on the shapes it was written for. The rig's round trip (today's demo
//! taken back to format 32 and up again) is the catalog's.
//!
//! Owns:          the unit tests of upgrade.rs.
//! Doesn't own:   the round trip on the rig (tests/catalog kernel).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0289-show-files-upgrade-step-by-step.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn every_format_from_the_oldest_has_its_step() {
    let froms: Vec<u32> = STEPS.iter().map(|s| s.from).collect();
    let want: Vec<u32> = (OLDEST_FORMAT..lumacore_tree::FORMAT_VERSION).collect();
    assert_eq!(froms, want, "a FORMAT_VERSION bump ships with its step (an empty one when nothing saved changed shape)");
}

#[test]
fn too_old_and_too_new_are_refused() {
    let ours = u64::from(lumacore_tree::FORMAT_VERSION);
    assert!(readable(ours).is_ok());
    assert!(readable(u64::from(OLDEST_FORMAT)).is_ok());
    assert_eq!(readable(u64::from(OLDEST_FORMAT) - 1).unwrap_err().1, "E-SHOW-002");
    assert_eq!(readable(ours + 1).unwrap_err().1, "E-SHOW-001");
    let mut show = json!({ "show_settings": { "format_version": OLDEST_FORMAT - 1 } });
    assert!(upgrade(&mut show).unwrap_err().contains("E-SHOW-002"));
}

#[test]
fn the_tint_gives_way_to_the_background_adjusters() {
    let mut show = json!({ "user_profiles": {
        "1": { "meta": { "name": "Bob" }, "settings": { "theme": { "adjust": { "brightness": 0, "tint": 30 } } } },
        "2": { "meta": { "name": "Ann" }, "settings": { "theme": { "adjust": { "tint": 0 } } } },
    }});
    let notes = tint_gives_way(&mut show);
    assert_eq!(show["user_profiles"]["1"]["settings"]["theme"]["adjust"], json!({ "brightness": 0, "background_hue": 0, "background_saturation": 0 }));
    assert_eq!(notes.len(), 1, "only a Tint that wasn't 0 is named: {notes:?}");
    assert!(notes[0].starts_with("Bob"));
}

#[test]
fn view_buttons_become_screen_ones() {
    let g = "6f1f4f0e-0000-4000-8000-000000000001";
    let mut show = json!({ "user_profiles": {
        "1": { "content": { "view_buttons": { "4": g } } },
        "2": { "content": { "view_buttons": {} } },
    }});
    let notes = view_buttons_per_screen(&mut show);
    assert_eq!(show["user_profiles"]["1"]["content"]["view_buttons"], json!({ "1": { "4": { "kind": "View", "guid": g } } }));
    assert_eq!(show["user_profiles"]["2"]["content"]["view_buttons"], json!({}));
    assert_eq!(notes, vec!["1 view buttons are screen 1's now".to_string()]);
}

#[test]
fn preset_groups_become_one_pool_in_order() {
    let preset = |name: &str| json!({ "meta": { "name": name }, "mode": "Selective", "values": [] });
    let mut show = json!({
        "datapools": { "1": { "preset_groups": {
            "4": { "preset_mode": "Selective", "presets": { "2": preset("Blue"), "1": preset("Red") } },
            "2": { "preset_mode": "Universal", "presets": { "10": preset("Stage") } },
        } } },
        "user_profiles": { "1": { "content": { "selected_preset_group": null, "views": {} } } },
    });
    let notes = one_preset_pool(&mut show);
    let pool = &show["datapools"]["1"]["presets"];
    assert_eq!((pool["1"]["meta"]["name"].as_str(), pool["2"]["meta"]["name"].as_str(), pool["3"]["meta"]["name"].as_str()), (Some("Stage"), Some("Red"), Some("Blue")));
    assert!(show["datapools"]["1"].get("preset_groups").is_none());
    assert!(show["user_profiles"]["1"]["content"].get("selected_preset_group").is_none());
    assert!(notes[0].contains("2.10 → 1") && notes[0].contains("4.2 → 3"), "{notes:?}");
}

#[test]
fn a_show_of_this_format_is_left_alone() {
    let mut show = json!({ "show_settings": { "format_version": lumacore_tree::FORMAT_VERSION } });
    assert_eq!(upgrade(&mut show).unwrap(), None);
}
