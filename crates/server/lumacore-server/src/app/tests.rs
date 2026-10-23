//  █████╗ ██████╗ ██████╗     ████████╗███████╗███████╗████████╗███████╗
// ██╔══██╗██╔══██╗██╔══██╗    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ███████║██████╔╝██████╔╝       ██║   █████╗  ███████╗   ██║   ███████╗
// ██╔══██║██╔═══╝ ██╔═══╝        ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║  ██║██║     ██║            ██║   ███████╗███████║   ██║   ███████║
// ╚═╝  ╚═╝╚═╝     ╚═╝            ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: app tests

//! Tests of `app`.
//!
//! Owns:          the unit tests of app.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §20, §30 "12d part 5", R1-17, §34 24.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn a_startup_show_that_panics_leaves_a_clean_state() {
    let mut state = lumacore_objects::AppState::new();
    state.local_settings.station_id = "keep-me".into();
    state.debug_mode = true;
    let result: Result<(), String> = load_startup_show(&mut state, "Bad", |state| {
        state.current_show = Some("half loaded".into());
        state.show_dirty = true;
        panic!("broken show");
    });
    let err = result.unwrap_err();
    assert!(err.contains("startup show \"Bad\" panicked on load"), "{err}");
    assert!(err.contains("report #") && err.ends_with("[E-INTERNAL-001]"), "{err}");
    assert_eq!(state.current_show, None, "the partly loaded state is gone");
    assert!(!state.show_dirty);
    assert_eq!(state.local_settings.station_id, "keep-me", "station settings stay");
    assert!(state.debug_mode);
    assert!(
        lumacore_sysmon::history()
            .iter()
            .any(|l| l.message.contains("startup show \"Bad\" panicked on load")
                && matches!(l.level, lumacore_sysmon::Level::Error)),
        "the System Monitor says so"
    );
    // An ordinary error passes through unchanged.
    let plain: Result<(), String> = load_startup_show(&mut state, "Gone", |_| Err("no file".into()));
    assert_eq!(plain.unwrap_err(), "no file");
}

mod marker_tests {
/// R1-17 (Q38): a marker left by a start that died skips the startup show
/// once and goes; a normal load leaves no marker.
#[test]
fn a_left_marker_skips_the_startup_show_once() {
    let dir = format!("target/test-marker-{}", std::process::id());
    let marker = format!("{dir}/.starting");
    std::fs::create_dir_all(&dir).unwrap();
    assert_eq!(super::guarded_by_marker(&dir, "Show", || Ok::<_, String>(1)), Ok(Ok(1)));
    assert!(!std::path::Path::new(&marker).exists(), "removed after the load");
    std::fs::write(&marker, "Bad Show").unwrap();
    let skipped = super::guarded_by_marker(&dir, "Bad Show", || Ok::<_, String>(2));
    assert!(skipped.as_ref().is_err_and(|e| e.contains("E-SHOW-006")), "{skipped:?}");
    assert!(!std::path::Path::new(&marker).exists(), "the next start loads again");
    std::fs::remove_dir_all(&dir).unwrap();
}
}
