// ██████╗ ███████╗████████╗ █████╗ ██╗██╗         ████████╗███████╗███████╗████████╗███████╗
// ██╔══██╗██╔════╝╚══██╔══╝██╔══██╗██║██║         ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║  ██║█████╗     ██║   ███████║██║██║            ██║   █████╗  ███████╗   ██║   ███████╗
// ██║  ██║██╔══╝     ██║   ██╔══██║██║██║            ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██████╔╝███████╗   ██║   ██║  ██║██║███████╗       ██║   ███████╗███████║   ██║   ███████║
// ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: detail tests

//! Tests of `detail`.
//!
//! Owns:          the unit tests of detail.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "Users & Profiles follows storage", §34 20.7.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn the_demo_has_profile_content_and_user_work() {
    let state = lumacore_objects::demo::fresh_state();
    let show = &state.root.show_data;
    let no = show.profile_id_for_user("Admin").unwrap();
    let p = view(show, "profile", no).unwrap();
    assert!(p["keys"].is_array() && p["images"].is_array() && p["screens"].is_array());
    assert!(p["encoder_bars"].as_u64().unwrap() >= 1);
    let u = view(show, "user", 1).unwrap();
    assert_eq!(u["work"].as_array().unwrap().len(), 5);
    assert!(view(show, "nope", 1).is_none());
}
