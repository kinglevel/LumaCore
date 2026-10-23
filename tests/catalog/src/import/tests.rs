// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Import Show across the crates (decisions §30 "12d part 5", §34 25.16): parts and single objects
//! by GUID in one Oops step, Network and Station among them.
//!
//!
//! Owns:          the cross-crate import tests.
//! Doesn't own:   each crate's own words' tests.
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12d part 5", §34 25.16.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::commands::dispatch;
use lumacore_objects::{AppState, DispatchResult};

fn add_user(state: &mut AppState, no: u32, name: &str) {
    let mut u = state.root.show_data.users.values().next().unwrap().clone();
    u.meta.identity.guid = uuid::Uuid::new_v4();
    u.login = name.into();
    state.root.show_data.users.insert(no, u);
}

fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => lumacore_objects::text::strip_ansi(&s),
        _ => panic!("'{line}' did not produce output"),
    }
}

#[test]
fn import_show_takes_parts_and_single_objects_by_guid_in_one_step() {
    let _lib = Library::new("import");
    let mut src = bare_station();
    add_user(&mut src, 9, "LD");
    run(&mut src, "Store Network \"Main\"");
    run(&mut src, "Store Network \"Video\"");
    run(&mut src, "SaveShow \"src\"");

    let mut s = bare_station();
    let frames = |s: &AppState| {
        s.undo_stack(s.effective_user_name())
            .map_or(0, |u| u.frames.len())
    };
    let before = frames(&s);
    let out = run(&mut s, "Import Show \"src\" Network \"Main\"");
    assert!(
        out.contains("Imported from \"src\": Network \"Main\""),
        "{out}"
    );
    let names: Vec<_> = s
        .root
        .show_data
        .protocols
        .networks
        .values()
        .filter_map(|n| n.meta.identity.name.clone())
        .collect();
    assert_eq!(names, ["Main"], "only the picked network: {out}");
    assert_eq!(frames(&s), before + 1, "one Oops step");
    // The same GUID again updates it in place; the whole part adds the rest.
    let out = run(&mut s, "Import Show \"src\" Protocols");
    assert!(out.contains("1 updated · 1 added"), "{out}");
    assert_eq!(s.root.show_data.protocols.networks.len(), 2);
    // Users: LD comes, the src's Admin (another object of that name) doesn't.
    let out = run(&mut s, "Import Show \"src\" Users");
    assert!(out.contains("skipped"), "{out}");
    assert!(s.root.show_data.user("LD").is_some());
    assert_eq!(
        s.root
            .show_data
            .users
            .values()
            .filter(|u| u.login == "Admin")
            .count(),
        1
    );
    // Refusals name what is wrong.
    assert!(run(&mut s, "Import Show \"src\" Sequence 7").contains("no Sequence 7"));
    assert!(run(&mut s, "Import Show \"src\"").contains("Name what to take"));
    assert!(run(&mut s, "Import Show \"nope\" Patch").contains("No show"));
    // The dialog's view of the file: facts and parts from the manifest.
    let v = lumacore_cmd_showfile::parts_view("src", None, None);
    assert_eq!(v["loads"], true);
    let protocols = &v["parts"][2];
    assert_eq!(protocols["key"], "Protocols");
    assert!(
        protocols["objects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["ref"] == "Network 1")
    );
    let users = &v["parts"][3];
    assert!(
        users["objects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["ref"] == "User \"LD\"")
    );
}

/// A fresh show library for this test's thread; removed when dropped (never the real one).
struct Library(std::path::PathBuf);

impl Library {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("lumacore-catalog-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        lumacore_luma::backups::use_dir(Some(dir.clone()));
        Self(dir)
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        lumacore_luma::backups::use_dir(None);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A station named FOH with nothing stored (Import Show's tests).
fn bare_station() -> AppState {
    let mut s = AppState::new();
    s.root.station_settings.station_name = "FOH".into();
    s
}
