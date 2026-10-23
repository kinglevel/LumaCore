// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The tree shell's words line in, reply out: cat, stat, head / tail, tree, find, grep, du, wc, diff,
//! cd and ls on dynamic children, and what the kernel's tests checked through these words (a
//! Guest's walk, no password hash, a fence word's path, Enter off the disk, the Undo steps).
//!
//! Owns:          the tests of the tree shell's words.
//! Doesn't own:   mv / cp / rm (door/tests.rs), the golden record of the tree (words/tests.rs).
//! Entry points:  the #[test] fns.
//! Decisions:     §31, §33 18, §34 25.7.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_fixtures::Fixture;
use uuid::Uuid;

use lumacore_objects::commands::dispatch;
use lumacore_objects::commands::navigate::cmd_ls;
use lumacore_objects::commands::resolve_verb;
use lumacore_objects::navigation::Navigation;
use lumacore_objects::{AppState, DispatchResult};

/// A fence line read with the catalog's vocabulary: `fl("head Fixtures -n 2")`.
fn fl(src: &str) -> lumacore_syntax::FenceLine {
    lumacore_objects::commands::typed::fence_line(src).unwrap_or_else(|r| panic!("{src}: {r:?}"))
}

/// Create an AppState with some fixtures and universes for VFS testing.
fn vfs_test_state() -> AppState {
    let mut state = AppState::new();
    let ft_guid = Uuid::new_v4();
    state.root.show_data.devices.dmx.patch.insert_fid(1, Fixture::new(Some("Front Wash L".to_string()),
            ft_guid,
            uuid::Uuid::nil(),
            uuid::Uuid::nil(),
            Uuid::nil(),
            1,
            6,
        ),
    );
    state.root.show_data.devices.dmx.patch.insert_fid(2, Fixture::new(Some("Front Wash R".to_string()),
            ft_guid,
            uuid::Uuid::nil(),
            uuid::Uuid::nil(),
            Uuid::nil(),
            7,
            6,
        ),
    );
    state.root.show_data.devices.dmx.patch.insert_fid(3, Fixture::new(Some("Back Light".to_string()),
            ft_guid,
            uuid::Uuid::nil(),
            uuid::Uuid::nil(),
            Uuid::nil(),
            13,
            6,
        ),
    );
    state.root.show_data.devices.dmx.patch.universes.insert(
        1,
        lumacore_fixtures::Universe::new(Some("Main".to_string())),
    );
    state
}

// *************************************************************************************************
// cat
// *************************************************************************************************
#[test]
fn cat_fixture_shows_json() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
        "Fixtures".to_string(),
    ];
    let result = crate::read::cmd_cat(&state, &fl("cat 1"));
    assert!(result.contains("Fixture"), "should show node type");
    // Should contain fixture data
    assert!(!result.contains("No such node"));
}

#[test]
fn cat_nonexistent_node() {
    let state = vfs_test_state();
    let result = crate::read::cmd_cat(&state, &fl("cat nosuchnode"));
    assert!(result.contains("No such node"));
}

#[test]
fn cat_structural_node() {
    let state = vfs_test_state();
    let result = crate::read::cmd_cat(&state, &fl("cat Root.ShowData.Devices.Dmx.Patch"));
    assert!(result.contains("Patch"));
    assert!(result.contains("children"));
}

// *************************************************************************************************
// stat
// *************************************************************************************************
#[test]
fn stat_fixture_shows_metadata() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
        "Fixtures".to_string(),
    ];
    let result = crate::read::cmd_stat(&state, &fl("stat 1"));
    assert!(result.contains("Type:"), "should show Type field");
    assert!(result.contains("Fixture"), "should show Fixture type");
    assert!(result.contains("GUID:"), "should show GUID");
}

#[test]
fn stat_structural_node() {
    let state = vfs_test_state();
    let result = crate::read::cmd_stat(&state, &fl("stat Root.ShowData.Devices.Dmx.Patch"));
    assert!(result.contains("Directory"));
    assert!(result.contains("Children:"));
}

// *************************************************************************************************
// head / tail
// *************************************************************************************************
#[test]
fn head_shows_first_items() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
    ];
    let result = crate::read::cmd_head(&state, &fl("head Fixtures -n 2"));
    assert!(result.contains("2/3"), "should show 2 of 3");
}

#[test]
fn tail_shows_last_items() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
    ];
    let result = crate::read::cmd_tail(&state, &fl("tail Fixtures -n 1"));
    assert!(result.contains("1/3"), "should show 1 of 3");
}

// *************************************************************************************************
// tree
// *************************************************************************************************
#[test]
fn tree_shows_structural_children() {
    let state = vfs_test_state();
    let result = crate::tree::cmd_tree(&state, &fl("tree Root.ShowData.Devices.Dmx.Patch"));
    assert!(result.contains("Fixtures"), "should show Fixtures");
    assert!(result.contains("Universes"), "should show Universes");
}

#[test]
fn tree_shows_dynamic_counts() {
    let state = vfs_test_state();
    let result = crate::tree::cmd_tree(&state, &fl("tree Root.ShowData.Devices.Dmx.Patch"));
    assert!(result.contains("(3)"), "should show fixture count");
}

#[test]
fn tree_depth_limit() {
    let state = vfs_test_state();
    let result = crate::tree::cmd_tree(&state, &fl("tree Root -d 1"));
    // Only depth 1: should show ShowData, Runtime, etc. but not Patch
    assert!(result.contains("ShowData"));
    // At depth=1 we shouldn't see Patch because it's at depth 3
    assert!(!result.contains("Patch"));
}

// *************************************************************************************************
// find
// *************************************************************************************************
#[test]
fn find_by_name_match() {
    let state = vfs_test_state();
    let result = crate::search::cmd_find(&state, &fl("find Front"));
    assert!(result.contains("2 match"), "should find 2 'Front' fixtures");
}

#[test]
fn find_with_type_filter() {
    let state = vfs_test_state();
    let result = crate::search::cmd_find(&state, &fl("find 1 --type Universe"));
    assert!(result.contains("1 match"), "should find 1 universe");
}

#[test]
fn find_no_match() {
    let state = vfs_test_state();
    let result = crate::search::cmd_find(&state, &fl("find zzzzzzz"));
    assert!(result.contains("No matches"));
}

#[test]
fn find_requires_pattern() {
    let state = vfs_test_state();
    let result = crate::search::cmd_find(&state, &fl("find"));
    assert!(result.contains("E-SYN-002"), "{result}");
}

// *************************************************************************************************
// grep
// *************************************************************************************************
#[test]
fn grep_finds_fixture_value() {
    let state = vfs_test_state();
    let result = crate::search::cmd_grep(&state, &fl("grep \"Back Light\""));
    assert!(
        result.contains("1 object"),
        "should find Back Light fixture"
    );
}

#[test]
fn grep_no_match() {
    let state = vfs_test_state();
    let result = crate::search::cmd_grep(&state, &fl("grep zzzzzzz"));
    assert!(result.contains("No matches"));
}

#[test]
fn grep_requires_pattern() {
    let state = vfs_test_state();
    let result = crate::search::cmd_grep(&state, &fl("grep"));
    assert!(result.contains("E-SYN-002"), "{result}");
}

// *************************************************************************************************
// du
// *************************************************************************************************
#[test]
fn du_counts_all_objects() {
    let state = vfs_test_state();
    let result = crate::count::cmd_du(&state, &fl("du Root.ShowData.Devices.Dmx.Patch"));
    assert!(
        result.contains("4 objects") || result.contains("total"),
        "should count 3 fixtures + 1 universe = 4: {result}"
    );
}

#[test]
fn du_fixtures_only() {
    let state = vfs_test_state();
    let result = crate::count::cmd_du(&state, &fl("du Root.ShowData.Devices.Dmx.Patch.Fixtures"));
    assert!(result.contains("3"), "should count 3 fixtures");
}

#[test]
fn du_empty_path() {
    let state = AppState::new();
    let result = crate::count::cmd_du(&state, &fl("du Root.ShowData.Devices.Dmx.Patch.Fixtures"));
    assert!(result.contains("0 objects"));
}

// *************************************************************************************************
// wc
// *************************************************************************************************
#[test]
fn wc_counts_direct_children() {
    let state = vfs_test_state();
    let result = crate::count::cmd_wc(&state, &fl("wc Root.ShowData.Devices.Dmx.Patch"));
    // Patch has >= 4 structural children
    assert!(result.contains("children"));
}

#[test]
fn wc_counts_dynamic_children() {
    let state = vfs_test_state();
    let result = crate::count::cmd_wc(&state, &fl("wc Root.ShowData.Devices.Dmx.Patch.Fixtures"));
    assert!(result.contains("3 children"), "fixtures count: {result}");
}

// *************************************************************************************************
// diff
// *************************************************************************************************
#[test]
fn diff_two_fixtures() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
        "Fixtures".to_string(),
    ];
    let result = crate::diff::cmd_diff(&state, &fl("diff 1 2"));
    // Different names, so should show diffs
    assert!(
        result.contains("differ"),
        "should show differences: {result}"
    );
}

#[test]
fn diff_same_fixture() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
        "Fixtures".to_string(),
    ];
    let result = crate::diff::cmd_diff(&state, &fl("diff 1 1"));
    assert!(
        result.contains("identical"),
        "same object should be identical: {result}"
    );
}

#[test]
fn diff_requires_two_paths() {
    let state = vfs_test_state();
    let result = crate::diff::cmd_diff(&state, &fl("diff 1"));
    assert!(result.contains("E-SYN-002"), "{result}");
}

#[test]
fn diff_nonexistent() {
    let state = vfs_test_state();
    let result = crate::diff::cmd_diff(
        &state,
        &fl("diff Root.ShowData.Devices.Dmx.Patch.Fixtures.1 Root.ShowData.Devices.Dmx.Patch.Fixtures.999"),
    );
    assert!(result.contains("No such node"));
}

// *************************************************************************************************
// dispatch integration tests
// *************************************************************************************************
#[test]
fn dispatch_cat_resolves() {
    let mut state = vfs_test_state();
    let result = dispatch(&mut state, "cat Root.ShowData.Devices.Dmx.Patch");
    match result {
        DispatchResult::Output(s) => {
            assert!(s.contains("Patch"), "cat should resolve Patch: {s}");
        }
        _ => panic!("Expected Output"),
    }
}

#[test]
fn dispatch_du_resolves() {
    let mut state = vfs_test_state();
    let result = dispatch(&mut state, "du Root.ShowData.Devices.Dmx.Patch.Fixtures");
    match result {
        DispatchResult::Output(s) => {
            assert!(s.contains("3"), "du should count 3 fixtures: {s}");
        }
        _ => panic!("Expected Output"),
    }
}

#[test]
fn dispatch_wc_resolves() {
    let mut state = vfs_test_state();
    let result = dispatch(&mut state, "wc Root.ShowData.Devices.Dmx.Patch");
    match result {
        DispatchResult::Output(s) => {
            assert!(s.contains("children"), "wc should count children: {s}");
        }
        _ => panic!("Expected Output"),
    }
}

#[test]
fn dispatch_tree_resolves() {
    let mut state = vfs_test_state();
    let result = dispatch(&mut state, "tree Root.ShowData.Devices.Dmx.Patch -d 1");
    match result {
        DispatchResult::Output(s) => {
            assert!(s.contains("Fixtures"), "tree should show Fixtures: {s}");
        }
        _ => panic!("Expected Output"),
    }
}

#[test]
fn dispatch_find_resolves() {
    let mut state = vfs_test_state();
    let result = dispatch(&mut state, "find Front");
    match result {
        DispatchResult::Output(s) => {
            assert!(s.contains("match"), "find should show matches: {s}");
        }
        _ => panic!("Expected Output"),
    }
}

#[test]
fn dispatch_stat_resolves() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
        "Fixtures".to_string(),
    ];
    let result = dispatch(&mut state, "stat 1");
    match result {
        DispatchResult::Output(s) => {
            assert!(s.contains("Fixture"), "stat should show Fixture type: {s}");
        }
        _ => panic!("Expected Output"),
    }
}

#[test]
fn dispatch_history_resolves() {
    let mut state = vfs_test_state();
    let result = dispatch(&mut state, "history");
    match result {
        DispatchResult::Output(s) => {
            assert!(!s.is_empty(), "history should return something");
        }
        _ => panic!("Expected Output"),
    }
}

// *************************************************************************************************
// cd into VFS dynamic children
// *************************************************************************************************
#[test]
fn cd_into_dynamic_fixture() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
        "Fixtures".to_string(),
    ];
    let result = state.cd_into("1");
    assert!(result.is_ok(), "should cd into fixture 1: {:?}", result);
    assert_eq!(state.navigation_path.last().unwrap(), "1");
}

#[test]
fn cd_into_nonexistent_dynamic() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
        "Fixtures".to_string(),
    ];
    let result = state.cd_into("999");
    assert!(result.is_err(), "should fail for nonexistent fixture");
}

// *************************************************************************************************
// ls with dynamic content
// *************************************************************************************************
#[test]
fn ls_shows_dynamic_fixtures() {
    let mut state = vfs_test_state();
    state.navigation_path = vec![
        "ShowData".to_string(),
        "Devices".to_string(),
        "Dmx".to_string(),
        "Patch".to_string(),
        "Fixtures".to_string(),
    ];
    let result = cmd_ls(&state, &fl("ls"));
    // Should show fixture objects, not just "(empty)"
    assert!(
        result.contains("Fixture") || result.contains("Front Wash"),
        "ls at Fixtures should show fixtures: {result}"
    );
}

#[test]
fn ls_shows_dynamic_and_structural() {
    let state = vfs_test_state();
    let result = cmd_ls(&state, &fl("ls Root.ShowData.Devices.Dmx.Patch.Fixtures"));
    // Should show fixture objects
    assert!(!result.contains("(empty)"), "should not be empty: {result}");
}

// *************************************************************************************************
// resolve_verb tests for new commands
// *************************************************************************************************
#[test]
fn resolve_verb_cat() {
    assert_eq!(resolve_verb("cat").unwrap(), "cat");
}

#[test]
fn resolve_verb_stat() {
    assert_eq!(resolve_verb("stat").unwrap(), "stat");
}

#[test]
fn resolve_verb_tree() {
    assert_eq!(resolve_verb("tree").unwrap(), "tree");
}

#[test]
fn resolve_verb_find() {
    assert_eq!(resolve_verb("find").unwrap(), "find");
}

#[test]
fn resolve_verb_grep() {
    assert_eq!(resolve_verb("grep").unwrap(), "grep");
}

#[test]
fn resolve_verb_du() {
    assert_eq!(resolve_verb("du").unwrap(), "du");
}

#[test]
fn resolve_verb_wc() {
    assert_eq!(resolve_verb("wc").unwrap(), "wc");
}

#[test]
fn resolve_verb_diff() {
    assert_eq!(resolve_verb("diff").unwrap(), "diff");
}

#[test]
fn resolve_verb_head() {
    assert_eq!(resolve_verb("head").unwrap(), "head");
}

#[test]
fn resolve_verb_tail() {
    assert_eq!(resolve_verb("tail").unwrap(), "tail");
}

// *************************************************************************************************
// what the kernel's tests checked through the tree shell's words
// *************************************************************************************************
fn run(state: &mut AppState, line: &str) -> String {
    match dispatch(state, line) {
        DispatchResult::Output(s) => String::from(s),
        _ => panic!("'{line}' did not produce output"),
    }
}

fn two_users() -> AppState {
    let mut state = AppState::new();
    run(&mut state, "Store User \"Visitor\" /Rights Guest");
    run(
        &mut state,
        "Store User \"LD\" /Rights Program /Password \"pw\"",
    );
    state
}

/// A Guest looks around with the tree shell (one rights gate for every line).
#[test]
fn a_guest_walks_the_tree() {
    let mut state = two_users();
    state.user = "Visitor".into();
    for line in [
        "cd Root",
        "ls",
        "pwd",
        "cat Root.ShowData.Users",
        "Enter Root",
    ] {
        let out = run(&mut state, line);
        assert!(!out.contains("E-RIGHTS"), "{line}: {out}");
    }
}

/// No tree-shell word shows a password hash, whoever asks.
#[test]
fn cat_and_grep_never_show_a_password_hash() {
    let mut state = two_users();
    let hash = state
        .root
        .show_data
        .user("LD")
        .unwrap()
        .password_hash
        .clone()
        .unwrap();
    let salt = hash.split('$').nth(2).unwrap().to_string();
    let no = state.root.show_data.user_no("LD").unwrap();
    for user in ["Visitor", "Admin"] {
        state.user = user.into();
        for line in [
            format!("cat Root.ShowData.Users.{no}"),
            "cat Root.ShowData.Users".to_string(),
        ] {
            let out = run(&mut state, &line);
            assert!(
                !out.contains(&salt),
                "{user}: '{line}' shows the hash: {out}"
            );
        }
        assert!(run(&mut state, &format!("cat Root.ShowData.Users.{no}")).contains("LD"));
        let out = run(&mut state, &format!("grep \"{salt}\" Root.ShowData.Users"));
        assert!(
            out.contains("No matches"),
            "{user}: grep finds the hash: {out}"
        );
    }
}

/// A tree path after a fence word is never a flag (`cd /ShowData`).
#[test]
fn a_fence_words_path_is_not_a_flag() {
    let line = lumacore_objects::commands::line::Line::parse("cd /ShowData").expect("lexes");
    assert!(line.statements[0].unknown_flags.is_empty());
}

/// `Enter` is a Show-reach word: it never touches the disk (decisions §29, finding 5).
#[test]
fn enter_stays_off_the_disk() {
    use lumacore_objects::commands::registry::{Reach, line_reach};
    let mut state = AppState::new();
    for line in ["Enter", "Enter Root"] {
        assert_eq!(line_reach(line), Reach::Show, "{line}");
        lumacore_luma::drive::set_off(true);
        let _ = dispatch(&mut state, line);
        lumacore_luma::drive::set_off(false);
        let refused = lumacore_luma::drive::take_refused();
        assert!(refused.is_empty(), "{line} reached the disk: {refused:?}");
    }
}

/// A user's Oops steps are readable in the tree (`Root.Runtime.Users.<n>.Undo`), never saved.
#[test]
fn cat_reads_a_users_undo_steps() {
    let mut state = vfs_test_state();
    run(&mut state, "Fixture 1 At 50");
    let undo = run(&mut state, "cat Root.Runtime.Users.1.Undo");
    assert!(undo.contains("Fixture 1 At 50"), "{undo}");
}

/// `cd Root 1.5` = `cd Root.1.5`: the Root object's numbers (decisions §33 54).
#[test]
fn cd_takes_roots_numbers() {
    use lumacore_objects::tree_path::Node;
    let mut state = AppState::new();
    let (show, devices) = (Node::ShowData.no().unwrap(), Node::Devices.no().unwrap());
    lumacore_objects::commands::navigate::cmd_cd(&mut state, &fl(&format!("cd Root {show}.{devices}")));
    assert_eq!(state.path_string(), "/ShowData/Devices");
    lumacore_objects::commands::navigate::cmd_cd(&mut state, &fl("cd /"));
    lumacore_objects::commands::navigate::cmd_cd(&mut state, &fl(&format!("cd {show}")));
    assert_eq!(state.path_string(), "/ShowData");
}

/// `ls` in `ShowData.DataPools` on the rig is one list (decisions 0291): `├──` before the
/// declared node, `└──` on the datapool that comes last — never `└──` above another row.
#[test]
fn ls_draws_the_last_glyph_on_the_last_row() {
    let mut state = lumacore_objects::demo::fresh_state();
    run(&mut state, "cd ShowData.DataPools");
    let ls = lumacore_objects::text::strip_ansi(&run(&mut state, "ls"));
    let rows: Vec<&str> = ls.lines().skip(1).collect();
    assert_eq!(rows.len(), 2, "{ls}");
    assert!(rows[0].starts_with("├── DatapoolPriorityOrders"), "{ls}");
    assert!(rows[1].starts_with("└── 1  \"Default\""), "{ls}");
}

/// `cat 1` in `ShowData` is `cd 1`'s node (Devices, by its number — decisions §33 54) under its
/// own name, never labelled "1"; a name that isn't there is refused (decisions 0291).
#[test]
fn cat_reads_the_node_cd_reaches() {
    let mut state = lumacore_objects::demo::fresh_state();
    run(&mut state, "cd ShowData");
    let cat = lumacore_objects::text::strip_ansi(&run(&mut state, "cat 1"));
    assert!(cat.starts_with("Devices/") && cat.contains("Dmx") && cat.contains("Metadata"), "{cat}");
    let none = lumacore_objects::text::strip_ansi(&run(&mut state, "cat Nothing"));
    assert!(none.contains("No such node") && none.contains("E-SET-004"), "{none}");
    let fixture = lumacore_objects::text::strip_ansi(&run(&mut state, "cat Devices.Dmx.Patch.Fixtures.101"));
    assert!(fixture.starts_with("Fixture 101"), "{fixture}");
}
