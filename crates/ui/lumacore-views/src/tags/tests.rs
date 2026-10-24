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
    let direct = view(&show, "Admin", &json!(all.to_string()), false, &Value::Null);
    let paths: Vec<&str> = direct["nodes"].as_array().unwrap().iter().map(|n| n["path"].as_str().unwrap()).collect();
    assert_eq!(paths, ["1", "1.1", "1.1.1", "2"]);
    assert_eq!(direct["members"].as_array().unwrap().len(), 0, "{direct}");
    assert_eq!(direct["tag"]["lines"]["select"], "Tag 1");

    let deep = view(&show, "Admin", &json!(all.to_string()), true, &Value::Null);
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
    assert_eq!(view(&show, "Admin", &json!(""), true, &Value::Null)["missing"], false);
    let gone = view(&show, "Admin", &json!(Uuid::new_v4().to_string()), true, &Value::Null);
    assert_eq!((gone["missing"].as_bool(), gone["tag"].is_null()), (Some(true), true));
    assert_eq!(gone["nodes"].as_array().unwrap().len(), 4);
}

/// The tag picker's `on` (decisions 0297): each tag's state over those objects — on when every one
/// carries it, part when some do, off when none; without `on` no state at all.
#[test]
fn on_gives_each_tag_its_state_over_the_objects() {
    let (show, [_, _, _, busking]) = rig();
    let other = Uuid::new_v4();
    let v = view(&show, "Admin", &json!(""), false, &json!([busking.to_string(), other.to_string()]));
    let state = |path: &str| v["nodes"].as_array().unwrap().iter().find(|n| n["path"] == path).map(|n| (n["state"].clone(), n["carried"].clone())).unwrap();
    assert_eq!(state("1.1.1"), (json!("part"), json!(1)));
    assert_eq!(state("1"), (json!("off"), json!(0)));
    let one = view(&show, "Admin", &json!(""), false, &json!([busking.to_string()]));
    assert_eq!(one["nodes"][2]["state"], "on");
    assert!(view(&show, "Admin", &json!(""), false, &Value::Null)["nodes"][2].get("state").is_none());
}

