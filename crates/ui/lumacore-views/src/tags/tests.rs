// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `tags`.
//!
//! Owns:          the unit tests of tags.rs.
//! Doesn't own:   the Tag words' tests (`lumacore-cmd-tags`).
//! Entry points:  the #[test] fns.
//! Decisions:     §12 "Tags hardened", §34 20.7.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

/// All › Lx1 › Right, and Busking. Busking carries Right; DataPool 1 (when the show has one)
/// carries Lx1.
fn rig() -> (ShowData, [Uuid; 4]) {
    let mut show = ShowData::default();
    for (path, name) in [(vec![1], "All"), (vec![1, 1], "Lx1"), (vec![1, 1, 1], "Right"), (vec![2], "Busking")] {
        tree::insert(&mut show.tags, &TagPath(path), Tag::new(Some(name.into()))).unwrap();
    }
    let guid = |p: Vec<u32>| tree::find(&show.tags, &TagPath(p)).unwrap().meta.identity.guid;
    let ids = [guid(vec![1]), guid(vec![1, 1]), guid(vec![1, 1, 1]), guid(vec![2])];
    tree::find_mut(&mut show.tags, &TagPath(vec![2])).unwrap().meta.annotation.tags.push(ids[2]);
    if let Some(dp) = show.datapools.values_mut().next() {
        dp.meta.annotation.tags.push(ids[1]);
    }
    (show, ids)
}

/// The tree comes in path order with counts; a picked tag lists what carries it — Direct only
/// its own, Deep also through the tags under it, marked "from <tag>"; the lines name its path.
#[test]
fn the_pane_lists_a_tags_members_direct_or_deep() {
    let (show, [all, _, _, _]) = rig();
    let pools = usize::from(!show.datapools.is_empty());
    let direct = view(&show, &json!(all.to_string()), false);
    let paths: Vec<&str> = direct["nodes"].as_array().unwrap().iter().map(|n| n["path"].as_str().unwrap()).collect();
    assert_eq!(paths, ["1", "1.1", "1.1.1", "2"]);
    assert_eq!(direct["members"].as_array().unwrap().len(), 0, "{direct}");
    assert_eq!(direct["tag"]["lines"]["select"], "Tag 1");

    let deep = view(&show, &json!(all.to_string()), true);
    let members = deep["members"].as_array().unwrap();
    assert_eq!(members.len(), 1 + pools, "{deep}");
    let busking = members.iter().find(|m| m["kind"] == "Tag").expect("the tag Busking carries Right");
    assert_eq!((busking["no"].as_str(), busking["name"].as_str(), busking["via"].as_str()), (Some("2"), Some("Busking"), Some("from Right")));
    assert_eq!(busking["target"], "Tag 2");
    assert_eq!(deep["tag"]["lines"]["select"], "Tag 1 /R");
    assert_eq!(deep["nodes"][0]["count"], 1 + pools);
}

/// Nothing picked shows the tree alone; a GUID that is gone says missing.
#[test]
fn a_tag_that_is_gone_is_missing() {
    let (show, _) = rig();
    assert_eq!(view(&show, &json!(""), true)["missing"], false);
    let gone = view(&show, &json!(Uuid::new_v4().to_string()), true);
    assert_eq!((gone["missing"].as_bool(), gone["tag"].is_null()), (Some(true), true));
    assert_eq!(gone["nodes"].as_array().unwrap().len(), 4);
}
