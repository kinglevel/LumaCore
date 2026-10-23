// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `station`.
//!
//! Owns:          the unit tests of station.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12d part 4", §34 25.10.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn sandbox(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("lumacore-station-config-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    crate::drive::set_sandbox(Some(dir.clone()));
    dir
}

#[test]
fn half_a_settings_file_is_an_error() {
    let whole = r#"{"station_id":"abc","networks":{"show":"en1"}}"#;
    assert_eq!(LocalSettings::parse(whole).unwrap().station_id, "abc");
    assert!(LocalSettings::parse(&whole[..whole.len() / 2]).is_err());
    assert!(LocalSettings::parse("").is_err());
}

#[test]
fn a_broken_settings_file_is_never_written_over() {
    let dir = sandbox("broken");
    let file = dir.join("station_settings.json");
    let torn = r#"{"station_id":"abc","networks":{"sh"#;
    std::fs::write(&file, torn).unwrap();
    let s = LocalSettings::load_from(&file.to_string_lossy());
    assert!(s.unreadable.is_some());
    assert!(!s.station_id.is_empty(), "the station still runs, on an id in memory");
    s.save_to(&file.to_string_lossy());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), torn, "left as it was");
    crate::drive::set_sandbox(None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_first_start_writes_the_settings_whole() {
    let dir = sandbox("first");
    let file = dir.join("station_settings.json");
    let s = LocalSettings::load_from(&file.to_string_lossy());
    assert!(s.unreadable.is_none());
    let again = LocalSettings::parse(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(again.station_id, s.station_id);
    assert!(!dir.join("station_settings.json.tmp").exists());
    crate::drive::set_sandbox(None);
    let _ = std::fs::remove_dir_all(&dir);
}
