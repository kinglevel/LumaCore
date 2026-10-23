// ██████╗ ██████╗ ██╗██╗   ██╗███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██╔══██╗██╔══██╗██║██║   ██║██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║  ██║██████╔╝██║██║   ██║█████╗         ██║   █████╗  ███████╗   ██║   ███████╗
// ██║  ██║██╔══██╗██║╚██╗ ██╔╝██╔══╝         ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██████╔╝██║  ██║██║ ╚████╔╝ ███████╗       ██║   ███████╗███████║   ██║   ███████║
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: drive tests

//! Tests of `drive`.
//!
//! Owns:          the unit tests of drive.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §25 "Step 10 hardening", §29, §34 25.10.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn a_closed_door_refuses_and_remembers() {
    set_off(true);
    let err = read_to_string("Cargo.toml").unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(take_refused(), ["read Cargo.toml"]);
    set_off(false);
    assert!(read_to_string("Cargo.toml").is_ok());
    assert!(take_refused().is_empty());
}

#[test]
fn write_atomic_replaces_the_whole_file_and_leaves_no_temp() {
    let dir = std::env::temp_dir().join(format!("lumacore-drive-atomic-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("x.json");
    write_atomic(&path, b"old old old").unwrap();
    write_atomic(&path, b"new").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"new");
    assert!(!dir.join("x.json.tmp").exists());
    let mut chunks = 0;
    write_atomic_with(&path, vec![1u8; (1 << 20) + 1], |_| chunks += 1).unwrap();
    assert_eq!(chunks, 2);
    set_off(true);
    assert!(write_atomic(&path, b"no").is_err(), "gated like every drive function");
    set_off(false);
    let _ = take_refused();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_names_every_system_accepts() {
    for ok in ["Show 1", "Rock'n'Roll", "it's", "Ünïcödé", "a.b", "CONSOLE", "COM0", "LPT10"] {
        assert!(check_file_name(ok).is_ok(), "{ok}");
    }
    for bad in [
        "", "a<b", "a>b", "a:b", "a\"b", "a/b", "a\\b", "a|b", "a?b", "a*b", "a\tb", "a\u{7}b", ".hidden",
        "trailing.", "trailing ", "CON", "con", "nul.luma", "Com1", "LPT9.txt", "aux ",
    ] {
        assert!(check_file_name(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn the_sandbox_lets_only_its_own_paths_through() {
    let root = std::env::current_dir().unwrap();
    set_sandbox(Some(root.clone()));
    assert!(read_to_string("Cargo.toml").is_ok(), "relative, inside");
    assert!(read_to_string(root.join("Cargo.toml")).is_ok(), "absolute, inside");
    assert!(read_to_string("../Cargo.toml").is_err(), "climbs out");
    assert!(read_to_string("/etc/hosts").is_err(), "elsewhere");
    set_sandbox(None);
    assert_eq!(take_refused().len(), 2);
}
