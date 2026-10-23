// ██████╗  ██████╗ ██╗    ██╗███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██╔══██╗██╔═══██╗██║    ██║██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██████╔╝██║   ██║██║ █╗ ██║███████╗       ██║   █████╗  ███████╗   ██║   ███████╗
// ██╔══██╗██║   ██║██║███╗██║╚════██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║  ██║╚██████╔╝╚███╔███╔╝███████║       ██║   ███████╗███████║   ██║   ███████║
// ╚═╝  ╚═╝ ╚═════╝  ╚══╝╚══╝ ╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: rows tests

//! The rows below a node on the demo rig: every row's path opens, a datapool is a branch by its
//! number, an object's text is `ls`'s.
//!
//! Owns:          the tests of tree_rows.rs.
//! Doesn't own:   `ls`'s drawing (lumacore-cmd-treeshell's tests), NAV (lumacore-terminal's).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0291-nav-mode-ls-rows-and-an-inspector, docs/decisions/0106-tree-paths-typed-node.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::{RowKind, rows};
use crate::navigation::Navigation;
use crate::tree_path::TreePath;

/// Every row below these nodes opens: its path parses, exists and `cd` reaches it by its
/// segments (a label never becomes a path, decisions 0291).
#[test]
fn every_row_opens_by_its_path() {
    let state = crate::demo::fresh_state();
    let mut todo = vec!["Root".to_string()];
    let mut seen = 0;
    while let Some(at) = todo.pop() {
        for row in rows(&state, &at) {
            seen += 1;
            if row.kind == RowKind::Dir || row.kind == RowKind::File || row.path.starts_with("Root.Drives") {
                continue;
            }
            let path = TreePath::parse_dot(&row.path).unwrap_or_else(|| panic!("{} doesn't parse", row.path));
            assert!(crate::vfs::path_exists(&state.root, &path), "{} doesn't exist", row.path);
            let segments = state.resolve_target(&row.path).unwrap_or_else(|e| panic!("{}: {e}", row.path));
            assert_eq!(crate::tree_path::dot(&segments), row.path, "cd {} lands elsewhere", row.path);
            // A show object opens; a live entry (a master's level, a line's state) is its text.
            if row.kind == RowKind::Object && row.guid.is_some() {
                assert!(crate::vfs::resolve_node(&row.path, &state.root).is_some(), "{} doesn't open", row.path);
            }
            // The tree goes on into nodes and datapools; the rig's pools are walked one level.
            if row.branch && (row.kind != RowKind::Object || row.type_name == "DataPool") && !row.path.contains("Runtime") {
                todo.push(row.path.clone());
            }
        }
    }
    assert!(seen > 1371, "the walk reached the fixtures: {seen}");
}

/// `ShowData.DataPools`: its declared node, then DataPool 1 by number, a branch with its count.
#[test]
fn a_datapool_is_a_numbered_branch() {
    let state = crate::demo::fresh_state();
    let rows = rows(&state, "Root.ShowData.DataPools");
    assert_eq!(rows[0].kind, RowKind::Node);
    assert_eq!(rows[0].segment, "DatapoolPriorityOrders");
    let dp = rows.iter().find(|r| r.kind == RowKind::Object).expect("DataPool 1");
    assert_eq!((dp.segment.as_str(), dp.path.as_str()), ("1", "Root.ShowData.DataPools.1"));
    assert!(dp.branch && dp.guid.is_some());
    assert_eq!(dp.text(), "1  \"Default\"  [77 children]  (DataPool)");
    assert_eq!(dp.target().as_deref(), Some("DataPool 1"));
}

/// A fixture's row is `ls`'s text and names its target.
#[test]
fn a_fixture_row_is_ls_text() {
    let state = crate::demo::fresh_state();
    let rows = rows(&state, "Root.ShowData.Devices.Dmx.Patch.Fixtures");
    assert_eq!(rows.len(), state.root.show_data.devices.dmx.patch.by_number().len());
    let f = rows.iter().find(|r| r.slot == Some(101)).expect("Fixture 101");
    assert_eq!(f.text(), "101  \"ACME PHOTON HYBRID(XL 500 BSW)\"  (Fixture)");
    assert_eq!(f.target().as_deref(), Some("Fixture 101"));
    assert!(!f.branch);
}
